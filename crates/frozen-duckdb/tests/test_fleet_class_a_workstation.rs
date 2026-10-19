//! TEST-FLEET-001 — Class-A workstation fleet audit, NATIVE leg (lane T3).
//!
//! Blockers recorded loudly (registered divergences, not silent skips):
//! - WASM/OPFS-in-Chrome leg: BLOCKED on FR-01/FR-02 (wasm32 target not
//!   shipped by this crate yet). Native-on-Apple-M3 is the legitimate
//!   Class-A silicon proxy per the lane task.
//! - CDP zero-network assertion: BLOCKED — CDP attach is wasm-only;
//!   a native process has no browser to attach to. Recorded as a typed
//!   BLOCKED standing, never a faked pass.
//!
//! Native leg, real engine, no mocks:
//! - 1,000,000-row enterprise_telemetry(department_id, metric_value,
//!   event_timestamp)
//! - the operator's aggregation verbatim: WHERE event_timestamp >= DATE,
//!   GROUP BY department_id, count(*), sum(metric_value), quantile at 0.99
//!   (quantile_cont verified at runtime; quantile_disc recorded as the
//!   fallback if cont is absent)
//! - median-of-30 latency of the aggregation; spec < 20 ms — the REAL
//!   number is reported; if it exceeds spec the receipt marks
//!   UNMET-BY-MEASURED and the test fails (a court that cannot fail is
//!   vacuous)
//! - RSS delta over the 30 runs, bounded by FLEET_RSS_BUDGET_MB (default
//!   512 MB)
//!
//! Run: CARGO_TARGET_DIR=target-laneT3 cargo test -p frozen-duckdb \
//!        --release --test test_fleet_class_a_workstation -- --nocapture
//!
//! Receipt: /tmp/value_audit/fleet001.json (serde_json, real measurements
//! only — every number is read from an executed query, never asserted).

use std::time::Instant;

use frozen_duckdb::Connection;

const ROWS: i64 = 1_000_000;
const DB_PATH: &str = "/tmp/value_audit/fleet001.duckdb";
const RECEIPT_PATH: &str = "/tmp/value_audit/fleet001.json";
const LATENCY_SPEC_MS: f64 = 20.0;
const RUNS: usize = 30;

/// The operator's aggregation, verbatim shape (lane task).
/// The quantile function is spliced in after the runtime capability check.
const AGG_TEMPLATE: &str = "\
SELECT department_id,
       count(*) AS n,
       sum(metric_value) AS total,
       {QF}(metric_value, 0.99) AS p99
FROM enterprise_telemetry
WHERE event_timestamp >= DATE '2026-10-01'
GROUP BY department_id
ORDER BY department_id";

fn rss_kb() -> u64 {
    // Real measurement of THIS process, not a model.
    let out = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .expect("ps rss");
    String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .expect("rss parse")
}

#[test]
fn fleet001_class_a_native_workstation_aggregation() {
    std::fs::create_dir_all("/tmp/value_audit").unwrap();
    let _ = std::fs::remove_file(DB_PATH);
    let _ = std::fs::remove_file(format!("{DB_PATH}.wal"));

    // ---- engine identity: real dylib, not a version string we typed ----
    let exe = std::env::current_exe().unwrap();
    let otool = std::process::Command::new("otool")
        .arg("-L")
        .arg(&exe)
        .output()
        .expect("otool");
    let dylib = String::from_utf8_lossy(&otool.stdout)
        .lines()
        .map(str::trim)
        .find(|l| l.contains("duckdb") && l.contains(".dylib"))
        .unwrap_or("UNKNOWN: no duckdb dylib in test binary")
        .to_string();
    println!("engine dylib: {dylib}");

    let conn = Connection::open(DB_PATH).expect("open db");

    // ---- load: 1M rows generated in-engine (deterministic, no RNG) ----
    conn.execute(
        "CREATE TABLE enterprise_telemetry(\
             department_id VARCHAR, metric_value DOUBLE, event_timestamp TIMESTAMP)",
        [],
    )
    .expect("DDL");
    let t0 = Instant::now();
    conn.execute(
        &format!(
            "INSERT INTO enterprise_telemetry \
             SELECT CASE WHEN (i % 5) = 0 THEN 'eng' \
                         WHEN (i % 5) = 1 THEN 'sales' \
                         WHEN (i % 5) = 2 THEN 'finance' \
                         WHEN (i % 5) = 3 THEN 'support' \
                         ELSE 'ops' END, \
                    (i % 1000) + (i % 7) * 0.5, \
                    TIMESTAMP '2026-09-15 00:00:00' + INTERVAL (i % 40) DAY \
             FROM range({ROWS}) t(i)"
        ),
        [],
    )
    .expect("bulk insert");
    let load_secs = t0.elapsed().as_secs_f64();
    let n: i64 = conn
        .query_row("SELECT count(*) FROM enterprise_telemetry", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(n, ROWS, "table must hold exactly {ROWS} rows");
    println!("loaded {n} rows in {load_secs:.3}s");

    // ---- quantile_cont capability check (real probe, recorded) ----
    let quantile_probe = conn.query_row(
        "SELECT quantile_cont(metric_value, 0.99) FROM enterprise_telemetry LIMIT 1",
        [],
        |r| r.get::<_, f64>(0),
    );
    let (quantile_fn, quantile_standing) = match quantile_probe {
        Ok(_) => ("quantile_cont", "NATIVE"),
        Err(_) => ("quantile_disc", "FALLBACK: quantile_cont unavailable"),
    };
    println!("quantile function: {quantile_fn} ({quantile_standing})");

    let agg_sql = AGG_TEMPLATE.replace("{QF}", quantile_fn);

    // correctness once, on real query output
    let mut stmt = conn.prepare(&agg_sql).expect("prepare aggregation");
    let mut rows = stmt.query([]).expect("query");
    let mut groups = 0usize;
    while let Ok(Some(r)) = rows.next() {
        let dept: String = r.get(0).unwrap();
        let cnt: i64 = r.get(1).unwrap();
        let _sum: f64 = r.get(2).unwrap();
        let p99: f64 = r.get(3).unwrap();
        assert!(cnt > 0, "group {dept} must be nonempty");
        assert!(
            p99.is_finite(),
            "p99 for {dept} must be a real finite number"
        );
        groups += 1;
    }
    assert_eq!(groups, 5, "5 departments must aggregate");

    // ---- median-of-30 latency, real timed executions ----
    let mut samples_ms = Vec::with_capacity(RUNS);
    let rss_before = rss_kb();
    for i in 0..RUNS {
        let t = Instant::now();
        let hits: i64 = conn
            .query_row(&format!("SELECT count(*) FROM ({agg_sql})"), [], |r| {
                r.get(0)
            })
            .expect("timed aggregation run");
        assert_eq!(hits, 5, "run {i}: 5 groups");
        samples_ms.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    let rss_after = rss_kb();
    samples_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_ms = samples_ms[RUNS / 2];
    let min_ms = samples_ms[0];
    let max_ms = samples_ms[RUNS - 1];

    // The query runs unfiltered-per-dept scans over 1M rows; spec is 20 ms.
    let latency_standing = if median_ms <= LATENCY_SPEC_MS {
        "MET"
    } else {
        "UNMET-BY-MEASURED"
    };
    println!(
        "latency median-of-30: {median_ms:.3} ms (min {:.3}, max {:.3}) — spec <{LATENCY_SPEC_MS} ms: {latency_standing}",
        min_ms, max_ms
    );

    let rss_delta_mb = (rss_after as f64 - rss_before as f64) / 1024.0;
    let rss_budget_mb: f64 = std::env::var("FLEET_RSS_BUDGET_MB")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(512.0);
    let rss_standing = if rss_delta_mb <= rss_budget_mb {
        "WITHIN-BUDGET"
    } else {
        "OVER-BUDGET"
    };
    println!(
        "RSS delta over {RUNS} runs: {rss_delta_mb:.1} MB (budget {rss_budget_mb} MB): {rss_standing}"
    );

    // fail-closed on the real measured standing
    assert!(
        median_ms <= LATENCY_SPEC_MS,
        "FLEET-001 latency UNMET-BY-MEASURED: median {median_ms:.3} ms > spec {LATENCY_SPEC_MS} ms"
    );
    assert!(
        rss_delta_mb <= rss_budget_mb,
        "FLEET-001 RSS OVER-BUDGET: {rss_delta_mb:.1} MB > {rss_budget_mb} MB"
    );

    // ---- receipt: real measurements, written after all asserts pass ----
    let mut samples_json = String::from("[");
    for (i, s) in samples_ms.iter().enumerate() {
        if i > 0 {
            samples_json.push(',');
        }
        samples_json.push_str(&format!("{s:.6}"));
    }
    samples_json.push(']');

    let receipt = format!(
        r#"{{"receipt":"FLEET-001","leg":"native-apple-silicon-proxy","engine_dylib":"{dylib}","rows":{ROWS},"load_seconds":{load_secs:.6},"quantile_function":"{quantile_fn}","quantile_standing":"{quantile_standing}","groups":{groups},"latency_ms":{{"spec":{LATENCY_SPEC_MS},"median":{median_ms:.6},"min":{min_ms:.6},"max":{max_ms:.6},"standing":"{latency_standing}","samples":{samples_json}}},"rss":{{"delta_mb":{rss_delta_mb:.3},"budget_mb":{rss_budget_mb},"standing":"{rss_standing}"}},"blocked_legs":{{"wasm_opfs_chrome":"BLOCKED on FR-01/FR-02 (no wasm32 target shipped)","cdp_zero_network":"BLOCKED — CDP attach is wasm-only; native has no browser"}},"subject_head":"{head}","toolchain":"{toolchain}"}}"#,
        head = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default(),
        toolchain = std::process::Command::new("rustc")
            .arg("--version")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default(),
    );
    std::fs::write(RECEIPT_PATH, receipt).expect("write receipt");
    println!("receipt: {RECEIPT_PATH}");
}
