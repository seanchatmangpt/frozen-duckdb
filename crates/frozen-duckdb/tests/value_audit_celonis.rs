//! TEST-VAL-002 — value-audit court: 250k conforming OCEL 2.0 segments in
//! `ocel_conforming_segments`, then the operator's DFG cycle-time SQL run
//! verbatim, latency measured (median of 20 runs, no sleeps), all DFG edges
//! asserted present.
//!
//! Schema/DDL shape reuses `tests/doctor_ocel_tests.rs` (TEST-018 leg-B
//! table). Real engine, real rows, no mocks (Chicago school).
//!
//! Receipt: VALUE-VAL-002.

use frozen_duckdb::Connection;
use std::fs;
use std::io::Write as IoWrite;
use std::time::Instant;

const SEGMENTS: i64 = 250_000;
const OBJECTS: i64 = 50_000;
/// 4 actions, cyclic: create-order -> pick-item -> pack-item -> ship-order
/// -> create-order -> ...  => exactly 4 distinct DFG edges.
const ACTIONS: [&str; 4] = ["create-order", "pick-item", "pack-item", "ship-order"];

/// DFG cycle-time SQL, verbatim from the operator (TEST-VAL-002).
const DFG_SQL: &str = "SELECT from_state_id, to_state_id, count(*), avg(t_end - t_start) \
     FROM ocel_conforming_segments GROUP BY 1, 2";

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock before epoch")
        .as_nanos();
    let dir =
        std::env::temp_dir().join(format!("val002-{}-{}-{}", label, std::process::id(), nanos));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

/// Deterministic segment i -> (oid, from_state, to_state, t_start_us,
/// t_end_us). States cycle over the 4 actions; timestamps advance 1 s per
/// segment; duration is deterministic (125 ms + (i % 40) * 25 ms) so
/// avg(t_end - t_start) is a real per-edge interval.
fn segment(i: i64) -> (String, &'static str, &'static str, i64, i64) {
    let from = ACTIONS[(i % 4) as usize];
    let to = ACTIONS[((i + 1) % 4) as usize];
    let t_start = 1_767_225_600_000_000i64 + i * 1_000_000; // micros, 1 s cadence
    let dur_us = 125_000i64 + (i % 40) * 25_000;
    (
        format!("obj-{}", i % OBJECTS),
        from,
        to,
        t_start,
        t_start + dur_us,
    )
}

/// Epoch micros -> "YYYY-MM-DD HH:MM:SS.SSSSSS" (UTC, no external crates) —
/// same civil-from-days math as doctor_ocel_tests::chrono_compat, extended
/// with the fractional second.
fn chrono_compat_us(ts_us: i64) -> String {
    let secs = ts_us.div_euclid(1_000_000);
    let micros = ts_us.rem_euclid(1_000_000);
    let days = secs.div_euclid(86_400);
    let s = secs.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02}.{micros:06}",
        s / 3600,
        (s % 3600) / 60,
        s % 60
    )
}

fn scalar_i64(conn: &Connection, sql: &str) -> i64 {
    let mut out: i64 = 0;
    conn.query_row(sql, [], |r| {
        out = r.get(0)?;
        Ok(())
    })
    .unwrap_or_else(|e| panic!("scalar_i64 `{sql}`: {e}"));
    out
}

#[test]
fn val_002_dfg_cycle_time_250k() {
    let dir = temp_dir("val002");
    let db = dir.join("val002.db");
    let conn = Connection::open(&db).expect("open frozen-duckdb connection");

    // Same DDL shape as doctor_ocel_tests.rs leg B, extended with the DFG
    // state columns the operator's SQL selects.
    conn.execute_batch(
        "CREATE TABLE ocel_conforming_segments(
            segment_id BIGINT,
            oid VARCHAR,
            action VARCHAR,
            from_state_id VARCHAR,
            to_state_id VARCHAR,
            t_start TIMESTAMP,
            t_end TIMESTAMP,
            transition_count BIGINT);",
    )
    .expect("create segments DDL");

    // Deterministic bulk insert (chunked multi-row VALUES, 1000/chunk).
    let t_ingest = Instant::now();
    const CHUNK: i64 = 1_000;
    let mut start = 0;
    while start < SEGMENTS {
        let mut sql = String::with_capacity(1 << 18);
        sql.push_str("INSERT INTO ocel_conforming_segments VALUES ");
        let end = (start + CHUNK).min(SEGMENTS);
        for i in start..end {
            if i > start {
                sql.push(',');
            }
            let (oid, from, to, t0, t1) = segment(i);
            sql.push_str(&format!(
                " ({}, '{}', '{}', '{}', '{}', '{}', '{}', 1)",
                i,
                oid,
                from,
                from,
                to,
                chrono_compat_us(t0),
                chrono_compat_us(t1)
            ));
        }
        sql.push(';');
        conn.execute_batch(&sql).expect("segment chunk insert");
        start = end;
    }
    let ingest = t_ingest.elapsed();

    // Structural sanity: row count matches the generator.
    let rows = scalar_i64(&conn, "SELECT count(*) FROM ocel_conforming_segments;");
    assert_eq!(rows, SEGMENTS, "segment row count");

    // Expected distinct edges from the generator: 4 (cyclic), each with
    // SEGMENTS/4 segments.
    let expected_edges: i64 = ACTIONS.len() as i64;
    let expected_per_edge: i64 = SEGMENTS / expected_edges;

    // Median of 20 runs of the verbatim DFG SQL (no sleeps; each run does
    // real work over the real 250k-row table).
    const RUNS: usize = 20;
    let mut samples_ms: Vec<f64> = Vec::with_capacity(RUNS);
    let mut dfg_groups: i64 = 0;
    let mut edge_rows: Vec<(String, String, i64, (i32, i32, i64))> = Vec::new();
    for run in 0..RUNS {
        let t0 = Instant::now();
        let mut stmt = conn.prepare(DFG_SQL).expect("prepare DFG SQL");
        let mut it = stmt
            .query_map([], |r| {
                // avg(interval) comes back as a real INTERVAL value; read it
                // structurally (months/days/micros) instead of via text.
                let avg = match r.get_ref(3).expect("avg col") {
                    frozen_duckdb::duckdb::types::ValueRef::Interval {
                        months,
                        days,
                        nanos,
                    } => (months, days, nanos),
                    other => panic!("avg column not Interval: {other:?}"),
                };
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    avg,
                ))
            })
            .expect("run DFG SQL");
        edge_rows.clear();
        for row in it.by_ref() {
            edge_rows.push(row.expect("DFG row"));
        }
        drop(stmt);
        samples_ms.push(t0.elapsed().as_secs_f64() * 1000.0);
        if run == 0 {
            dfg_groups = edge_rows.len() as i64;
        }
    }
    samples_ms.sort_by(|a, b| a.partial_cmp(b).expect("finite ms"));
    let latency_ms_median = samples_ms[RUNS / 2];
    let latency_ms_min = samples_ms[0];

    // All edges present: group count == generator's distinct edge count, and
    // every cyclic edge (a -> b) appears with the exact expected count.
    assert_eq!(
        dfg_groups, expected_edges,
        "DFG group count != expected distinct edges (cyclic 4-action generator)"
    );
    for i in 0..expected_edges {
        let from = ACTIONS[(i % 4) as usize];
        let to = ACTIONS[((i + 1) % 4) as usize];
        let hit = edge_rows
            .iter()
            .find(|(f, t, _, _)| f == from && t == to)
            .unwrap_or_else(|| panic!("DFG edge {from} -> {to} missing"));
        assert_eq!(hit.2, expected_per_edge, "edge {from} -> {to} count");
    }

    // Ephemeral per-edge averages (avg(t_end - t_start) is a real interval;
    // print for the receipt, do not over-assert the engine's interval text).
    for (f, t, c, avg) in &edge_rows {
        // interval nanos -> ms (all our intervals are sub-second, micros
        // granularity carried in nanos).
        let avg_ms = avg.2 as f64 / 1_000_000.0;
        eprintln!("VALUE-VAL-002 DFG edge {f} -> {t}: count={c} avg_ms={avg_ms:.3}");
    }

    let verdict = if dfg_groups == expected_edges
        && samples_ms.iter().all(|v| v.is_finite())
        && latency_ms_median < 100.0
    {
        "PASS"
    } else {
        "FAIL"
    };

    // Emit /tmp/value_audit/val002.json
    let out_dir = std::path::Path::new("/tmp/value_audit");
    fs::create_dir_all(out_dir).expect("create /tmp/value_audit");
    let json = format!(
        "{{\"segments\":{}, \"dfg_groups\":{}, \"latency_ms_median\":{:.3}, \
         \"latency_ms_min\":{:.3}, \"verdict\":\"{}\"}}",
        SEGMENTS, dfg_groups, latency_ms_median, latency_ms_min, verdict
    );
    let mut f = fs::File::create(out_dir.join("val002.json")).expect("create val002.json");
    f.write_all(json.as_bytes()).expect("write val002.json");
    drop(f);

    eprintln!(
        "VALUE-VAL-002 segments={SEGMENTS} dfg_groups={dfg_groups} \
         latency_ms_median={latency_ms_median:.3} latency_ms_min={latency_ms_min:.3} \
         ingest_s={:.2} verdict={verdict}",
        ingest.as_secs_f64()
    );

    // Cleanup: temp db dir.
    let _ = fs::remove_dir_all(&dir);

    assert_eq!(verdict, "PASS", "TEST-VAL-002 verdict FAIL");
}
