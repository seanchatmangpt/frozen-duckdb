//! Sovereign leg of the empirical economic court (lane G5).
//!
//! Workload parity: seed 20261019, 100k `events` rows, 3 queries (cohort /
//! funnel / event-window shape) — SQL constants identical to
//! crates/frozen-duckdb/tests/gate_v01_latency.rs (rev2). Emits a machine
//! receipt (JSON) on stdout.
//!
//! egress_bytes = 0 and invoice_usd = 0.00 BY CONSTRUCTION: the query runs
//! in-process against a local libduckdb; nothing leaves the machine.

use frozen_duckdb::duckdb::types::ToSql;
use frozen_duckdb::Connection;
use std::time::Instant;

const N_ROWS: i64 = 100_000;
const WARMUP_ITERS: usize = 10;
const MEASURED_ITERS: usize = 50;
const SEED: i64 = 20261019;

// rev2 SQL constants — identical to gate_v01_latency.rs and to court.ts.
const QUERIES: [(&str, &str); 3] = [
    (
        "q1-cohort-filter",
        "SELECT count(*), sum(value) FROM events WHERE event_id BETWEEN 90000 AND 90100 AND ts % 7 = 3",
    ),
    (
        "q2-funnel-groupby",
        "WITH per_case AS (SELECT case_id, count(*) AS n, sum(value) AS s FROM events GROUP BY case_id) SELECT bucket, count(*) AS cases, sum(s) AS total FROM (SELECT s / 1000 AS bucket, s FROM per_case) t GROUP BY bucket ORDER BY bucket",
    ),
    (
        "q3-event-window",
        "SELECT sum(rolling) FROM (SELECT avg(value) OVER (PARTITION BY case_id ORDER BY ts ROWS BETWEEN 49 PRECEDING AND CURRENT ROW) AS rolling FROM events)",
    ),
];

fn seeded_value(i: i64) -> f64 {
    let mut x = (i
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407))
        >> 11;
    x &= (1i64 << 53) - 1;
    x as f64 / (1i64 << 53) as f64
}

fn build_table(conn: &Connection) {
    conn.execute_batch(
        "CREATE TABLE events (event_id BIGINT, case_id BIGINT, ts BIGINT, value DOUBLE)",
    )
    .expect("create events table");
    let mut app = conn.appender("events").expect("appender");
    for i in 0..N_ROWS {
        app.append_row([&i as &dyn ToSql, &(i % 5_000), &i, &seeded_value(i)])
            .expect("append row");
    }
    app.flush().expect("flush appender");
}

fn p95(samples: &mut [f64]) -> f64 {
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((samples.len() as f64 * 0.95).ceil() as usize).saturating_sub(1);
    samples[idx]
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Run one query once, draining all rows; returns (latency_ms, rows_returned).
fn run_once(conn: &Connection, sql: &str) -> (f64, usize) {
    let t0 = Instant::now();
    let mut stmt = conn.prepare(sql).expect("prepare");
    let mut rows = stmt.query([]).expect("execute");
    let mut n = 0usize;
    while rows.next().expect("row").is_some() {
        n += 1;
    }
    (t0.elapsed().as_secs_f64() * 1000.0, n)
}

// Lane S5 scorecard extension: the 2 additional workload classes (point
// filter, cohort retention journey, OCEL rehydration — point/journey/ocel are
// 3 new; grouped + windowed reuse QUERIES). Default output (no --full) is
// unchanged so court.ts's 3-query contract still holds.
const QUERIES_EXTRA: [(&str, &str); 3] = [
    (
        "q4-point-filter",
        "SELECT count(*), sum(value) FROM events WHERE event_id = 90000",
    ),
    (
        "q5-cohort-journey",
        "WITH first_ts AS (SELECT case_id, min(ts) AS f FROM events GROUP BY case_id) SELECT (e.ts - f.f) / 10000 AS step, count(DISTINCT e.case_id) AS retained FROM events e JOIN first_ts f ON e.case_id = f.case_id GROUP BY step ORDER BY step",
    ),
    (
        "q6-ocel-rehydrate",
        "SELECT case_id, ts, event_id, value FROM events ORDER BY case_id, ts, event_id",
    ),
];

fn main() {
    let full = std::env::args().any(|a| a == "--full");
    let conn = Connection::open_in_memory().expect("open in-memory duckdb");
    build_table(&conn);

    let n: i64 = conn
        .query_row("SELECT count(*) FROM events", [], |r| r.get(0))
        .expect("count");
    assert_eq!(n, N_ROWS, "seeded dataset must have exactly N_ROWS rows");

    let engine = format!(
        "frozen-duckdb v{} (bundled libduckdb, native in-process; leg=native-baseline, wasm leg FR-01-blocked)",
        env!("CARGO_PKG_VERSION")
    );

    println!("{{");
    println!("  \"leg\": \"native-baseline\",");
    println!("  \"engine\": {},", json_escape(&engine));
    println!("  \"seed\": {},", SEED);
    println!("  \"rows\": {},", N_ROWS);
    println!("  \"network_bytes\": 0,");
    println!("  \"invoice_usd\": 0.00,");
    println!("  \"queries\": [");

    let workload: Vec<(&str, &str)> = if full {
        QUERIES.iter().chain(QUERIES_EXTRA.iter()).copied().collect()
    } else {
        QUERIES.to_vec()
    };

    for (qi, (qid, sql)) in workload.iter().enumerate() {
        // Correctness pin: must execute and return at least one row. The first
        // execution doubles as the COLD measurement (cold table/page touch).
        let (cold_ms, first_rows) = run_once(&conn, sql);
        assert!(first_rows > 0, "{} must return rows", qid);

        let mut samples = Vec::with_capacity(MEASURED_ITERS);
        let mut rows_returned = 0usize;
        for it in 0..(WARMUP_ITERS + MEASURED_ITERS) {
            let (dt, nr) = run_once(&conn, sql);
            if it >= WARMUP_ITERS {
                samples.push(dt);
            }
            rows_returned = nr;
        }
        let lat = p95(&mut samples);

        if qi > 0 {
            println!(",");
        }
        println!(
            "    {{\"id\": \"{}\", \"p95_latency_ms\": {:.3}, \"cold_latency_ms\": {:.3}, \"egress_bytes\": 0, \"invoice_usd\": 0.00, \"rows_returned\": {}}}",
            qid, lat, cold_ms, rows_returned
        );
    }

    println!("\n  ]");
    println!("}}");
}
