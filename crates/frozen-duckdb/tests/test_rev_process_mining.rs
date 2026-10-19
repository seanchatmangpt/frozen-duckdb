//! TEST-REV-002 — operator bottleneck query court (process mining), NATIVE leg.
//!
//! Operator query, verbatim shape:
//!
//! ```sql
//! SELECT from, to,
//!        sum(transition_count)              AS execution_volume,
//!        avg(t_end - t_start)               AS avg_dwell,
//!        quantile_cont(t_end - t_start, 0.95) AS p95_dwell
//! FROM ocel_conforming_segments
//! GROUP BY from, to
//! ORDER BY execution_volume DESC;
//! ```
//!
//! Contract: 250k ocel_conforming_segments (D8 schema/generator pattern,
//! `tests/doctor_ocel_tests.rs`), median-of-20 latency, spec line < 10 ms
//! (reported real, not asserted as pass/fail gate — the spec line is the
//! comparison target), memory ≤ 128 MB via native proxy: RSS delta bounded.
//!
//! BROWSER LEG — LOUD SKIP (registered divergence, not faked): the
//! wasm/OPFS-in-Chrome leg of TEST-REV-002 is BLOCKED on FR-01 (wasm build
//! of the frozen engine) and FR-02 (OPFS adapter). No browser harness exists
//! in this repo. This module is the native leg only; a `#[ignore]`-style
//! placeholder is recorded below (`browser_leg_blocked`) so the cell reads
//! BLOCKED, never silently green.

use frozen_duckdb::Connection;
use std::process::Command;
use std::time::Instant;

const SEGMENTS: i64 = 250_000;
const LATENCY_RUNS: usize = 20;
const LATENCY_SPEC_MS: f64 = 10.0;
/// Native proxy for the worker's 128 MB memory budget: RSS delta between
/// post-ingest baseline and post-query steady state must stay within this.
/// (Documented approximation on macOS — `ps rss=`; see doctor_leak_audit.rs.)
const RSS_TOLERANCE_BYTES: u64 = 128 * 1024 * 1024;

/// One RSS sample in bytes (documented approximation on macOS).
fn rss_bytes() -> u64 {
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status").expect("read /proc/self/status");
        for line in status.lines() {
            if let Some(rest) = line.strip_prefix("VmRSS:") {
                let kb: u64 = rest
                    .trim()
                    .trim_end_matches("kB")
                    .trim()
                    .parse()
                    .expect("parse VmRSS");
                return kb * 1024;
            }
        }
        panic!("VmRSS not found in /proc/self/status");
    }
    #[cfg(not(target_os = "linux"))]
    {
        let out = Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
            .expect("run ps");
        assert!(out.status.success(), "ps exited {:?}", out.status.code());
        String::from_utf8_lossy(&out.stdout)
            .trim()
            .parse::<u64>()
            .expect("parse ps rss") * 1024
    }
}

/// The operator's bottleneck query. `p95_fn` is `quantile_cont` when the
/// engine supports it, else the recorded `quantile_disc` fallback.
fn bottleneck_sql(p95_fn: &str) -> String {
    format!(
        "SELECT \"from\", \"to\", \
                sum(transition_count) AS execution_volume, \
                avg(epoch_us(t_end) - epoch_us(t_start)) AS avg_dwell_us, \
                {p95_fn}(epoch_us(t_end) - epoch_us(t_start), 0.95) AS p95_dwell_us \
         FROM ocel_conforming_segments \
         GROUP BY \"from\", \"to\" \
         ORDER BY execution_volume DESC"
    )
}

/// Loud-skip placeholder for the blocked browser leg. Never faked.
#[test]
#[ignore = "BLOCKED FR-01/FR-02: wasm build + OPFS adapter for the frozen engine do not exist; browser leg cannot run natively — loud registered divergence, not a fake pass"]
fn browser_leg_blocked() {}

#[test]
fn rev002_native_bottleneck_query() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open_in_memory().expect("open in-memory frozen-duckdb");

    // ---- Corpus: 250k segments, D8 pattern (deterministic, in-engine) ----
    // Transition segments: from/to action pairs drawn cyclically from 4
    // actions x 12 channels (D8 shape), 1 s cadence, per-segment
    // transition_count derived from block position so conservation is checkable.
    conn.execute_batch(&format!(
        "CREATE TABLE ocel_conforming_segments(
            segment_id BIGINT,
            oid VARCHAR,
            \"from\" VARCHAR,
            \"to\" VARCHAR,
            channel VARCHAR,
            t_start TIMESTAMP,
            t_end TIMESTAMP,
            transition_count BIGINT);
         INSERT INTO ocel_conforming_segments
         SELECT i AS segment_id,
                'obj-' || (i % 5000) AS oid,
                a.actions[(i % 4) + 1] || '@' || c.channels[(i % 12) + 1] AS \"from\",
                a.actions[((i + 1) % 4) + 1] || '@' || c.channels[(i % 12) + 1] AS \"to\",
                c.channels[(i % 12) + 1] AS channel,
                TIMESTAMP '2026-01-01 00:00:00' + INTERVAL (i) SECOND AS t_start,
                TIMESTAMP '2026-01-01 00:00:00' + INTERVAL (i + 7) SECOND AS t_end,
                (i % 97) + 1 AS transition_count
         FROM generate_series(0, {SEGMENTS} - 1) AS t(i),
              (SELECT ['create-order','pick-item','pack-item','ship-order'] AS actions) a,
              (SELECT ['web','mobile','pos','partner','edi','phone','kiosk','email','chat','api','store','fax'] AS channels) c;"
    ))
    .expect("create + load 250k segments");

    let (count, tc_sum): (i64, i64) = conn
        .query_row(
            "SELECT count(*), sum(transition_count) FROM ocel_conforming_segments",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap_or_else(|e| panic!("count query: {e}"));
    assert_eq!(count, SEGMENTS, "corpus row count");
    // sum((i%97)+1) for i in [0, 250000): 2577 full cycles of 1..=97 plus
    // 1..=31 remainder = 2577*4753 + 496.
    let expected_tc: i64 = (SEGMENTS / 97) * 4753 + (SEGMENTS % 97) * (SEGMENTS % 97 + 1) / 2;
    assert_eq!(
        tc_sum, expected_tc,
        "corpus conservation: sum(transition_count) analytic"
    );
    eprintln!("[rev002] corpus: {SEGMENTS} segments, sum(transition_count) = {tc_sum}");

    // ---- Probe quantile_cont; record fallback if unsupported ----
    let p95_fn = match conn.query_row(
        "SELECT quantile_cont(x, 0.95) FROM (SELECT unnest([1,2,3,4,100]) AS x)",
        [],
        |r| r.get::<_, f64>(0),
    ) {
        Ok(v) => {
            // DuckDB continuous interpolation at q=0.95 over [1,2,3,4,100]:
            // index 3.8 -> 4 + 0.8*(100-4) = 80.8.
            assert!((v - 80.8).abs() < 1e-9, "quantile_cont sanity: got {v}");
            eprintln!("[rev002] quantile_cont SUPPORTED (probe = {v})");
            "quantile_cont"
        }
        Err(e) => {
            eprintln!(
                "[rev002] quantile_cont UNSUPPORTED on this engine ({e}); using quantile_disc fallback (recorded)"
            );
            "quantile_disc"
        }
    };

    // Warm the plan/page cache once (outside the timed window), and take the
    // RSS baseline AFTER ingest + warmup so the delta isolates query working set.
    let warm_sql = bottleneck_sql(p95_fn);
    let mut stmt = conn.prepare(&warm_sql).expect("prepare warmup");
    let rows_seen: usize = stmt
        .query_map([], |r| {
            let _vol: i64 = r.get(2)?;
            Ok(())
        })?
        .filter(|r| r.is_ok())
        .count();
    drop(stmt);
    assert_eq!(rows_seen, 12, "distinct (from,to) edges in corpus");
    let rss_before = rss_bytes();

    // ---- Median-of-20 latency over the verbatim operator query ----
    let mut latencies_ms: Vec<f64> = Vec::with_capacity(LATENCY_RUNS);
    let mut last_vol_top: i64 = 0;
    for _ in 0..LATENCY_RUNS {
        let t0 = Instant::now();
        let mut stmt = conn.prepare(&warm_sql).expect("prepare timed run");
        let mut rows = stmt.query_map([], |r| {
            let vol: i64 = r.get(2)?;
            Ok(vol)
        })?;
        let mut first_vol: Option<i64> = None;
        while let Some(row) = rows.next() {
            let v = row.expect("row read");
            if first_vol.is_none() {
                first_vol = Some(v);
                // Top edge of the first run only (ORDER BY ... DESC row 1).
                if last_vol_top == 0 {
                    last_vol_top = v;
                }
            }
        }
        drop(stmt);
        latencies_ms.push(t0.elapsed().as_secs_f64() * 1000.0);
        assert!(first_vol.is_some(), "query returned zero rows");
    }
    latencies_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_ms = latencies_ms[LATENCY_RUNS / 2];
    let min_ms = latencies_ms[0];
    let max_ms = latencies_ms[LATENCY_RUNS - 1];
    eprintln!(
        "[rev002] latency (ms) median-of-20 = {median_ms:.3} (min {min_ms:.3}, max {max_ms:.3}); spec < {LATENCY_SPEC_MS} ms"
    );

    let rss_after = rss_bytes();
    let rss_delta = (rss_after as i64 - rss_before as i64) as i64;
    eprintln!(
        "[rev002] RSS delta over 20 query runs = {rss_delta:+} bytes ({:+.1} MiB); tolerance {RSS_TOLERANCE_BYTES} bytes",
        rss_delta as f64 / (1024.0 * 1024.0)
    );

    // ---- Assertions on real observed state ----
    // Cross-check the top edge volume against an independent aggregate.
    let expected_top: i64 = conn
        .query_row(
            "SELECT max(ev) FROM (SELECT sum(transition_count) AS ev \
             FROM ocel_conforming_segments GROUP BY \"from\", \"to\")",
            [],
            |r| r.get(0),
        )
        .expect("top-edge cross-check");
    assert_eq!(
        last_vol_top, expected_top,
        "top edge execution_volume mismatch (ORDER BY ... DESC row 1 vs max aggregate)"
    );
    assert!(
        (rss_after as i64 - rss_before as i64) <= RSS_TOLERANCE_BYTES as i64,
        "RSS grew {rss_delta} bytes over 20 bottleneck-query runs; 128 MB budget exceeded"
    );
    eprintln!(
        "[rev002] PASS line vs spec: median {median_ms:.3} ms vs <{LATENCY_SPEC_MS} ms spec -> {}",
        if median_ms < LATENCY_SPEC_MS { "WITHIN SPEC" } else { "OVER SPEC (reported real)" }
    );

    // ---- Receipt: /tmp/value_audit/rev002.json ----
    let receipt = format!(
        "{{\"leg\":\"native\",\"browser_leg\":\"BLOCKED FR-01\",\"test\":\"TEST-REV-002\",\"segments\":{SEGMENTS},\"p95_fn\":\"{p95_fn}\",\"latency_ms_median\":{median_ms:.3},\"latency_ms_min\":{min_ms:.3},\"latency_ms_max\":{max_ms:.3},\"latency_spec_ms\":{LATENCY_SPEC_MS},\"within_spec\":{},\"rss_delta_bytes\":{rss_delta},\"rss_tolerance_bytes\":{RSS_TOLERANCE_BYTES}}}",
        median_ms < LATENCY_SPEC_MS
    );
    std::fs::create_dir_all("/tmp/value_audit").expect("mkdir /tmp/value_audit");
    std::fs::write("/tmp/value_audit/rev002.json", receipt).expect("write receipt");
    eprintln!("[rev002] receipt written: /tmp/value_audit/rev002.json");
    Ok(())
}
