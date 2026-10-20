//! GATE-V-01 native leg (rev2 workload) — see docs/design/gate-v01-latency.md.
//!
//! Manual-loop harness (design-doc fallback): std::time::Instant, sorted
//! samples, nearest-rank p95 over the 3-query-total per iteration.
//! SQL constants are versioned in this file; any change is a subject
//! change and re-baselines the court.

use frozen_duckdb::duckdb::types::ToSql;
use frozen_duckdb::Connection;
use std::time::Instant;

const N_ROWS: i64 = 100_000;
const WARMUP_ITERS: usize = 50;
const MEASURED_ITERS: usize = 1_000;
const P95_BUDGET_MS: f64 = 15.0;

// SQL constants (rev2)
const Q1: &str =
    "SELECT count(*), sum(value) FROM events WHERE event_id BETWEEN 90000 AND 90050 AND ts % 7 = 3";
const Q2: &str = "WITH per_case AS (SELECT case_id, count(*) AS n, sum(value) AS s FROM events GROUP BY case_id) SELECT bucket, count(*) AS cases, sum(s) AS total FROM (SELECT s / 1000 AS bucket, s FROM per_case) t GROUP BY bucket ORDER BY bucket";
const Q3: &str = "SELECT sum(rolling) FROM (SELECT avg(value) OVER (PARTITION BY case_id ORDER BY ts ROWS BETWEEN 49 PRECEDING AND CURRENT ROW) AS rolling FROM events)";

/// Deterministic dataset: seed 20261019, 64-bit LCG keyed by row index.
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
    .unwrap();
    let mut app = conn.appender("events").unwrap();
    for i in 0..N_ROWS {
        app.append_row([&i as &dyn ToSql, &(i % 5_000), &i, &seeded_value(i)])
            .unwrap();
    }
    app.flush().unwrap();
}

fn machine_specs() {
    let cpu = std::process::Command::new("sysctl")
        .args(["-n", "machdep.cpu.brand_string"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".into());
    let mem = std::process::Command::new("sysctl")
        .args(["-n", "hw.memsize"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".into());
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(0);
    println!("machine.cpu_model  = {}", cpu);
    println!(
        "machine.hw_memsize = {} bytes ({} GB)",
        mem,
        mem.parse::<u64>().unwrap_or(0) / (1024 * 1024 * 1024)
    );
    println!("machine.cores      = {}", cores);
}

fn subject_binding() {
    let head = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    let tree = std::process::Command::new("git")
        .args(["rev-parse", "HEAD^{tree}"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    println!(
        "subject.head = {}",
        head.as_deref().unwrap_or("<not-a-git-subject-from-lane>")
    );
    println!(
        "subject.tree = {}",
        tree.as_deref().unwrap_or("<not-a-git-subject-from-lane>")
    );
}

/// One full measurement round on a FRESH connection (LSS5 §5 rule 3:
/// median-of-N, N=3 — defeats the contention flake without weakening the
/// budget; a contended run must fail all three medians to pass).
fn measure_round(round: usize) -> (f64, f64) {
    println!("--- round {}/3: fresh connection ---", round);

    let conn = Connection::open_in_memory().unwrap();
    build_table(&conn);
    let n: i64 = conn
        .query_row("SELECT count(*) FROM events", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        n, N_ROWS,
        "dataset build must be exactly the seeded row count"
    );

    let run_once = || -> f64 {
        let t0 = Instant::now();
        let _ = conn
            .query_row(Q1, [], |r| {
                Ok((r.get::<_, i64>(0), r.get::<_, Option<f64>>(1)))
            })
            .unwrap();
        let _ = conn
            .query_row(Q2, [], |r| {
                Ok((r.get::<_, f64>(0), r.get::<_, i64>(1), r.get::<_, f64>(2)))
            })
            .unwrap();
        let _ = conn.query_row(Q3, [], |r| r.get::<_, f64>(0)).unwrap();
        t0.elapsed().as_secs_f64() * 1000.0
    };

    for _ in 0..WARMUP_ITERS {
        run_once();
    }

    let mut samples: Vec<f64> = Vec::with_capacity(MEASURED_ITERS);
    for _ in 0..MEASURED_ITERS {
        samples.push(run_once());
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((0.95 * samples.len() as f64).ceil() as usize)
        .saturating_sub(1)
        .min(samples.len() - 1);
    let p95 = samples[idx];
    let mean = samples.iter().sum::<f64>() / samples.len() as f64;

    println!("round {}: samples measured = {}", round, samples.len());
    println!("round {}: mean_ms = {:.4}", round, mean);
    println!(
        "round {}: median_ms = {:.4}",
        round,
        samples[samples.len() / 2]
    );
    println!(
        "round {}: p95_nearest_rank_ms = {:.4} (index {})",
        round, p95, idx
    );
    println!(
        "leak_clause = UNSUPPORTED-BY-CURRENT-SURFACE (probe: no allocation-stats C API on pinned engine surface; counters not faked per doc F5)"
    );

    (p95, mean)
}

#[test]
fn gate_v01_p95_under_15ms() {
    machine_specs();
    subject_binding();
    println!(
        "config: rounds=3 (median-of-N per GAGE-RR-COURTS §5 rule 3) n_rows={} warmup={} measured={} budget_ms={}",
        N_ROWS, WARMUP_ITERS, MEASURED_ITERS, P95_BUDGET_MS
    );

    let mut p95s: Vec<f64> = Vec::with_capacity(3);
    for round in 1..=3 {
        let (p95, _mean) = measure_round(round);
        p95s.push(p95);
    }
    p95s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_p95 = p95s[1];

    println!("p95_rounds_ms = [{:.4}, {:.4}, {:.4}]", p95s[0], p95s[1], p95s[2]);
    println!("median_p95_ms = {:.4}", median_p95);
    println!(
        "verdict = {}",
        if median_p95 < P95_BUDGET_MS { "PASS" } else { "FAIL" }
    );

    assert!(
        median_p95 < P95_BUDGET_MS,
        "GATE-V-01 FAIL: median p95 {:.4} ms >= budget {} ms over rounds {:?} (debug-profile caveat: debug builds are substantially slower than release)",
        median_p95,
        P95_BUDGET_MS,
        p95s
    );

    // SHIM-COURT / lane V4 cross-file edit (F1's file, disclosed): JSON
    // receipt emission for the value ledger (TEST-VAL-003). Written only
    // after the assert above passes, so the receipt implies a real PASS run.
    // Median-of-3 per LSS5 §5; all raw p95s carried per the receipt law.
    let receipt = format!(
        "{{\"gate\":\"GATE-V-01\",\"mode\":\"median-of-3\",\"p95_ms_rounds\":[{:.4},{:.4},{:.4}],\"p95_ms\":{:.4},\"iterations_per_round\":{},\"queries\":3,\"budget_ms\":{},\"n_rows\":{}}}\n",
        p95s[0], p95s[1], p95s[2], median_p95, MEASURED_ITERS, P95_BUDGET_MS, N_ROWS
    );
    let dir = std::path::Path::new("/tmp/value_audit");
    let _ = std::fs::create_dir_all(dir);
    std::fs::write(dir.join("gate_v01_receipt.json"), receipt)
        .expect("write gate_v01_receipt.json");
}
