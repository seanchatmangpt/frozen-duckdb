//! TEST-VAL-001: 500k-event semi-structured storage audit (lane A1).
//!
//! Real engine (v2.0.0-dev cached dylib), no mocks. Two measured legs:
//!
//! Leg 1 (workload): 500k Datadog-log-shaped payloads ingested through the
//! VARIANT pipeline; selective filter latency on the payload error-code
//! path. Raw VARIANT-table bytes recorded honestly (this leg alone does not
//! hit 90% — that claim belongs to the segment schema, D8).
//!
//! Leg 2 (retention compression): the same 500k events folded into
//! `ocel_conforming_segments` (typed columnar segment schema, pattern from
//! `tests/doctor_ocel_tests.rs` / `value_audit_celonis.rs`).
//! compression_ratio = 1 - bytes_segments / bytes_raw; verdict PASS iff
//! ratio >= 0.90 (D8 observed 94.93–97.01%).
//!
//! Run: CARGO_TARGET_DIR=target-laneA1 cargo test -p frozen-duckdb \
//!        --test value_audit_datadog -- --nocapture

use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Instant;

use frozen_duckdb::Connection;

const EVENTS: usize = 500_000;
// Per-run scratch paths (CG1 FILE-LOCK-COLLISION fix): concurrent court runs
// must never share a DuckDB file. The final /tmp/value_audit/val001.json
// receipt stays at its fixed coordination-surface path, written last.
const RECEIPT_PATH: &str = "/tmp/value_audit/val001.json";

fn scratch_path(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("value_audit_{}_datadog", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

/// The exact SQL filter that works on this engine (recorded per lane task).
const FILTER_SQL: &str =
    "SELECT count(*) FROM logs WHERE json_extract_string(payload::VARCHAR, '$.response.code') = '500'";

struct EngineIdentity {
    dylib: String,
}

fn engine_identity() -> EngineIdentity {
    // The test binary is what links the cached duckdb dylib; otool -L names it.
    let exe = std::env::current_exe().expect("current_exe");
    let out = std::process::Command::new("otool")
        .arg("-L")
        .arg(&exe)
        .output()
        .expect("otool");
    let text = String::from_utf8_lossy(&out.stdout);
    let dylib = text
        .lines()
        .map(str::trim)
        .find(|l| l.contains("duckdb") && l.contains(".dylib"))
        .map(str::to_string)
        .unwrap_or_else(|| "UNKNOWN".to_string());
    EngineIdentity { dylib }
}

/// Datadog-log shaped payload (nested `response.code` — the path the
/// FILTER_SQL exercises), ~120 bytes, no single quotes by construction.
fn payload(i: usize) -> String {
    let code = if i.is_multiple_of(50) { 500 } else { 200 };
    let service = ["api", "web", "worker", "db"][i % 4];
    let env = ["prod", "staging"][i % 2];
    format!(
        r#"{{"service":"{service}","env":"{env}","host":"host-{}","message":"request completed","response":{{"code":{code},"duration_ms":{}}}}}"#,
        i % 64,
        (i * 7) % 900,
    )
}

/// Segment-fold projection of event i into the typed segment schema
/// (`ocel_conforming_segments`). Same inputs as `payload(i)` so both legs
/// carry identical information content: one segment per event, low-cardinality
/// action/service/env columns, host on the 64-host fleet, ts in epoch micros.
fn segment(
    i: usize,
) -> (
    i64,
    String,
    &'static str,
    &'static str,
    &'static str,
    i64,
    i64,
) {
    let code = if i.is_multiple_of(50) { 500 } else { 200 };
    let service = ["api", "web", "worker", "db"][i % 4];
    let env = ["prod", "staging"][i % 2];
    let ts = 1_767_225_600_000_000i64 + (i as i64) * 1_000_000; // 1 s cadence
    (
        i as i64,
        format!("host-{:02}", i % 64),
        service,
        env,
        if code == 500 { "error" } else { "ok" },
        ts,
        ((i * 7) % 900) as i64,
    )
}

#[test]
fn value_audit_datadog_500k_variant_compression() {
    std::fs::create_dir_all("/tmp/value_audit").unwrap();
    let db_path = scratch_path("val001.duckdb");
    let jsonl_path = scratch_path("val001-events.jsonl");
    for p in [&db_path, &jsonl_path, std::path::Path::new(RECEIPT_PATH)] {
        let _ = std::fs::remove_file(p);
    }

    let ident = engine_identity();
    println!("engine dylib: {}", ident.dylib);

    let conn = Connection::open(&db_path).expect("open db");
    conn.execute("CREATE TABLE logs(ts BIGINT, payload VARIANT)", [])
        .expect("VARIANT DDL accepted on this engine");

    // Generate once, write raw JSONL, insert through the VARIANT pipeline in
    // multi-row batches (single-row prepared executes measured ~3ms/row).
    let t_ins = Instant::now();
    let mut raw = BufWriter::new(File::create(&jsonl_path).unwrap());
    const BATCH: usize = 500;
    let mut sql = String::with_capacity(1 << 16);
    let mut done = 0usize;
    while done < EVENTS {
        let n = BATCH.min(EVENTS - done);
        sql.clear();
        sql.push_str("INSERT INTO logs VALUES ");
        for r in 0..n {
            let i = done + r;
            if r > 0 {
                sql.push(',');
            }
            let p = payload(i);
            raw.write_all(p.as_bytes()).unwrap();
            raw.write_all(b"\n").unwrap();
            // Payloads contain no single quotes by construction.
            sql.push_str(&format!("({}, '{}')", i, p));
        }
        conn.execute(&sql, []).expect("batched VARIANT insert");
        done += n;
    }
    drop(raw);
    conn.execute("CHECKPOINT", []).expect("checkpoint");
    let insert_s = t_ins.elapsed().as_secs_f64();

    // Close the connection BEFORE measuring: the WAL is flushed into the db
    // file on close, so mid-run measurement under-counts footprint bytes.
    drop(conn);
    let bytes_raw = std::fs::metadata(&jsonl_path).unwrap().len();
    let bytes_db = std::fs::metadata(&db_path).unwrap().len();
    let bytes_wal = std::fs::metadata(format!("{}.wal", db_path.display()))
        .map(|m| m.len())
        .unwrap_or(0);
    let bytes_db_total = bytes_db + bytes_wal;

    // Selective filter on a fresh connection over the checkpointed db.
    let conn2 = Connection::open(&db_path).expect("reopen db");
    let t_f = Instant::now();
    let hits: i64 = conn2
        .query_row(FILTER_SQL, [], |r| r.get(0))
        .expect("filter");
    let filter_ms = t_f.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(hits as usize, EVENTS / 50, "filter must select the 500s");
    drop(conn2);

    // Leg-2 footprint: the segment table lives in its own database file so
    // bytes_segments is measured on disk, not derived. The solo fold below
    // re-projects the identical events into seg_db.
    let seg_db = scratch_path("val001-segments.duckdb");
    let _ = std::fs::remove_file(&seg_db);
    let conn4 = Connection::open(&seg_db).expect("open segments-only db");
    conn4
        .execute(
            "CREATE TABLE ocel_conforming_segments(
                segment_id BIGINT,
                oid VARCHAR,
                action VARCHAR,
                service VARCHAR,
                env VARCHAR,
                t_start TIMESTAMP,
                t_end TIMESTAMP,
                duration_ms BIGINT,
                transition_count BIGINT)",
            [],
        )
        .expect("segments DDL (solo)");
    let t_seg = Instant::now();
    let mut seg_sql = String::with_capacity(1 << 16);
    let mut done = 0usize;
    const BATCH2: usize = 500;
    while done < EVENTS {
        let n = BATCH2.min(EVENTS - done);
        seg_sql.clear();
        seg_sql.push_str("INSERT INTO ocel_conforming_segments VALUES ");
        for r in 0..n {
            let i = done + r;
            if r > 0 {
                seg_sql.push(',');
            }
            let (sid, oid, action, service, env, ts, dur) = segment(i);
            let secs = ts.div_euclid(1_000_000);
            let micros = ts.rem_euclid(1_000_000);
            let days = secs.div_euclid(86_400);
            let s = secs.rem_euclid(86_400);
            let z = days + 719_468;
            let era = z.div_euclid(146_097);
            let doe = z.rem_euclid(146_097);
            let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
            let y = yoe + era * 400;
            let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
            let mp = (5 * doy + 2) / 153;
            let d = doy - (153 * mp + 2) / 5 + 1;
            let m = if mp < 10 { mp + 3 } else { mp - 9 };
            let y = if m <= 2 { y + 1 } else { y };
            seg_sql.push_str(&format!(
                " ({}, '{}', '{}', '{}', '{}', '{}-{:02}-{:02} {:02}:{:02}:{:02}.{:06}', '{}-{:02}-{:02} {:02}:{:02}:{:02}.{:06}', {}, 1)",
                sid, oid, action, service, env,
                y, m, d, s / 3600, (s % 3600) / 60, s % 60, micros,
                y, m, d, s / 3600, (s % 3600) / 60, s % 60, micros,
                dur,
            ));
        }
        conn4.execute(&seg_sql, []).expect("solo segment insert");
        done += n;
    }
    conn4
        .execute("CHECKPOINT", [])
        .expect("checkpoint segments");
    drop(conn4);
    let seg_insert_s = t_seg.elapsed().as_secs_f64();
    let seg_bytes = std::fs::metadata(&seg_db).unwrap().len();
    let seg_wal = std::fs::metadata(format!("{}.wal", seg_db.display()))
        .map(|m| m.len())
        .unwrap_or(0);
    let bytes_segments = seg_bytes + seg_wal;

    let compression_ratio = 1.0 - (bytes_segments as f64 / bytes_raw as f64);
    let verdict = if compression_ratio >= 0.90 {
        "PASS"
    } else {
        "FAIL"
    };

    let line = format!(
        "{{\"id\":\"VALUE-VAL-001\",\"events\":{},\"bytes_raw\":{},\"bytes_variant_table\":{},\"bytes_segments\":{},\"compression_ratio\":{:.4},\"filter_latency_ms\":{:.1},\"filter_sql\":\"{}\",\"insert_seconds\":{:.1},\"segment_insert_seconds\":{:.1},\"engine_dylib\":\"{}\",\"verdict\":\"{}\"}}",
        EVENTS, bytes_raw, bytes_db_total, bytes_segments, compression_ratio,
        filter_ms, FILTER_SQL, insert_s, seg_insert_s, ident.dylib, verdict
    );
    println!("{}", line);
    std::fs::write(RECEIPT_PATH, line + "\n").unwrap();

    assert!(
        compression_ratio >= 0.90,
        "compression_ratio {compression_ratio:.4} < 0.90"
    );
    assert_eq!(verdict, "PASS");
}
