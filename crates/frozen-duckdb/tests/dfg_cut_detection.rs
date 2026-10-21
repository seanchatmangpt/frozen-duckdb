//! MAX9 — Lifecycle in-SQL cut-detection court (DFG support + bottleneck).
//!
//! Spec surface, verbatim DFG formula:
//!
//! ```sql
//! SELECT source_act, target_act,
//!        SUM(weight * (is_cover + is_incomparable)) AS support
//! FROM pot_edges_table
//! GROUP BY source_act, target_act
//! ORDER BY support DESC;
//! ```
//!
//! Phases (all real, Chicago-style: in-engine synthetic corpus, no mocks):
//!
//! 1. `pot_edges_table` (source_act, target_act, weight, is_cover,
//!    is_incomparable) with 1,000,000 synthetic edges, generated
//!    deterministically in-engine. NOTE: this frozen engine statically
//!    links `core_functions` but does NOT autoload it — every connection
//!    in this court issues `LOAD core_functions` first (measured: without
//!    it, even `sum` is a Catalog Error; `INSTALL` is banned in this
//!    build: "does not link the loadable_extensions library").
//! 2. The spec's exact DFG SQL above, run unmodified.
//! 3. Bottleneck extraction: `quantile_cont(t_end - t_start, 0.95)` per
//!    edge — the REV-002 pattern (dwell as microsecond integers from
//!    BIGINT t_start_us/t_end_us).
//! 4. Oracle check: brute-force Rust-side recomputation on a fixed 10k-edge
//!    subset — support sums compared EXACTLY, p95 (sorted linear
//!    interpolation at idx = 0.95*(n-1)) within 1e-6 us of native
//!    `quantile_cont`, over 12 edge groups.
//! 5. Throughput: 1M-edge DFG aggregate in < 100 ms, median-of-5, real
//!    measured, hard-asserted (a court that cannot fail is not a court).
//!
//! RECEIPT (2026-10-19, native leg, Apple Silicon workstation):
//! `CARGO_TARGET_DIR=target-lane9 cargo test -p frozen-duckdb --test
//! dfg_cut_detection -- --nocapture` — see `[dfg]` eprintln lines and
//! `/tmp/value_audit/dfg_cut_detection.json` (written by the test).

use frozen_duckdb::Connection;
use std::collections::HashMap;
use std::time::Instant;

const EDGES: i64 = 1_000_000;
const THROUGHPUT_SPEC_MS: f64 = 100.0;
const THROUGHPUT_RUNS: usize = 5;
const ORACLE_SUBSET: i64 = 10_000;
/// Tolerance for p95 comparison (floating interpolation on integer micros).
const P95_EPS_US: f64 = 1e-6;

/// Deterministic in-engine corpus: 1M edges over a 4-action x 12-channel
/// space, weights cycling 1..=13, is_cover on i%3=0, is_incomparable on
/// i%7=0 (both true when i%21=0). Event dwell is deterministic per i.
fn create_corpus(conn: &Connection) {
    const ACTIONS: &str =
        "(VALUES (0,'create-order'),(1,'pick-item'),(2,'pack-item'),(3,'ship-order'))";
    const CHANNELS: &str = "(VALUES (0,'web'),(1,'mobile'),(2,'pos'),(3,'partner'),(4,'edi'),(5,'phone'),(6,'kiosk'),(7,'email'),(8,'chat'),(9,'api'),(10,'store'),(11,'fax'))";
    conn.execute_batch(&format!(
        "CREATE TABLE pot_edges_table(
            source_act VARCHAR,
            target_act VARCHAR,
            weight BIGINT,
            is_cover BOOLEAN,
            is_incomparable BOOLEAN);
         INSERT INTO pot_edges_table
         SELECT a.act || '@' || c.ch AS source_act,
                ta.act || '@' || c.ch AS target_act,
                (i % 13) + 1 AS weight,
                (i % 3 = 0) AS is_cover,
                (i % 7 = 0) AS is_incomparable
         FROM generate_series(0, {EDGES} - 1) t(i)
         JOIN {ACTIONS} AS a(idx, act) ON a.idx = i % 4
         JOIN {ACTIONS} AS ta(idx, act) ON ta.idx = (i + 1) % 4
         JOIN {CHANNELS} AS c(idx, ch) ON c.idx = i % 12;

         CREATE TABLE pot_edge_events(
            event_id BIGINT,
            source_act VARCHAR,
            target_act VARCHAR,
            t_start_us BIGINT,  -- microsecond epoch
            t_end_us BIGINT);
         INSERT INTO pot_edge_events
         SELECT i,
                a.act || '@' || c.ch,
                ta.act || '@' || c.ch,
                1767225600000000 + i * 1000000,
                1767225600000000 + i * 1000000 + ((i % 89) + 1) * 1000000
         FROM generate_series(0, {EDGES} - 1) t(i)
         JOIN {ACTIONS} AS a(idx, act) ON a.idx = i % 4
         JOIN {ACTIONS} AS ta(idx, act) ON ta.idx = (i + 1) % 4
         JOIN {CHANNELS} AS c(idx, ch) ON c.idx = i % 12;"
    ))
    .expect("create + load 1M-edge pot_edges_table + pot_edge_events");
}

/// The spec's DFG support SQL. The spec writes `is_cover + is_incomparable`
/// over booleans; DuckDB has no BOOLEAN+BOOLEAN, so each flag is cast to
/// INTEGER — the identical arithmetic, typed.
const DFG_SQL: &str = "SELECT source_act, target_act, \
       SUM(weight * (is_cover::INTEGER + is_incomparable::INTEGER)) AS support \
       FROM pot_edges_table \
       GROUP BY source_act, target_act \
       ORDER BY support DESC";

/// Bottleneck extraction, REV-002 pattern: p95 dwell per directed edge.
const BOTTLENECK_SQL: &str = "SELECT source_act, target_act, \
       quantile_cont(t_end_us - t_start_us, 0.95) AS p95_dwell_us \
       FROM pot_edge_events \
       GROUP BY source_act, target_act \
       ORDER BY p95_dwell_us DESC";

#[test]
fn dfg_cut_detection_court() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open_in_memory().expect("open in-memory frozen-duckdb");
    // Statically linked, NOT autoloaded (measured this session).
    conn.execute_batch("LOAD core_functions")
        .expect("LOAD core_functions");

    // ---- Phase 1: corpus ----
    create_corpus(&conn);
    let (n, cov, inc): (i64, i64, i64) = conn.query_row(
        "SELECT count(*), \
                count(*) FILTER (WHERE is_cover), \
                count(*) FILTER (WHERE is_incomparable) \
         FROM pot_edges_table",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    assert_eq!(n, EDGES, "corpus row count");
    assert_eq!(cov, 333_334, "is_cover analytic count (i%3=0 over 1M)");
    assert_eq!(
        inc, 142_858,
        "is_incomparable analytic count (i%7=0 over 1M)"
    );
    eprintln!("[dfg] corpus: {n} edges (is_cover={cov}, is_incomparable={inc})");

    // ---- Phase 2: spec DFG SQL (full 1M, real run) ----
    let mut stmt = conn.prepare(DFG_SQL)?;
    let mut dfg_rows: Vec<(String, String, i64)> = Vec::new();
    {
        let iter = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?;
        for row in iter {
            dfg_rows.push(row?);
        }
    }
    drop(stmt);
    assert!(!dfg_rows.is_empty(), "DFG aggregate returned zero rows");
    // Descending support ordering is part of the spec's contract.
    for w in dfg_rows.windows(2) {
        assert!(
            w[0].2 >= w[1].2,
            "support not descending: {} ({}) before {} ({})",
            w[0].0,
            w[0].2,
            w[1].0,
            w[1].2
        );
    }
    eprintln!(
        "[dfg] DFG aggregate: {} distinct edges, top support = {} ({} -> {})",
        dfg_rows.len(),
        dfg_rows[0].2,
        dfg_rows[0].0,
        dfg_rows[0].1
    );

    // ---- Phase 3: bottleneck extraction (p95 per edge, full 1M) ----
    let mut stmt = conn.prepare(BOTTLENECK_SQL)?;
    let mut bottleneck_groups: usize = 0;
    let mut worst: Option<(String, String, f64)> = None;
    {
        let iter = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, f64>(2)?,
            ))
        })?;
        for row in iter {
            let (s, t, p95) = row?;
            bottleneck_groups += 1;
            if worst.as_ref().is_none_or(|w| p95 > w.2) {
                worst = Some((s, t, p95));
            }
        }
    }
    drop(stmt);
    assert_eq!(
        bottleneck_groups,
        dfg_rows.len(),
        "bottleneck edge set must match DFG edge set"
    );
    let worst = worst.expect("bottleneck query returned zero groups");
    eprintln!(
        "[dfg] bottleneck: worst p95 dwell = {:.0} us on {} -> {}",
        worst.2, worst.0, worst.1
    );

    // ---- Phase 4: oracle check — brute-force Rust on a fixed 10k subset ----
    let mut subset = conn.prepare(
        "SELECT rowid, source_act, target_act, weight, is_cover, is_incomparable \
         FROM pot_edges_table WHERE (rowid * 7919) % 1000000 < 10000",
    )?;
    let mut raw: Vec<(i64, String, String, i64, bool, bool)> = Vec::new();
    {
        let iter = subset.query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
            ))
        })?;
        for row in iter {
            raw.push(row?);
        }
    }
    drop(subset);
    assert_eq!(
        raw.len() as i64,
        ORACLE_SUBSET,
        "oracle subset size (rowid*7919 % 1e6 < 10000, stride coprime with all cycles)"
    );

    // Brute-force support: plain HashMap group-by, sum w*(cov+inc).
    let mut brute_support: HashMap<(String, String), i64> = HashMap::new();
    for (_, s, t, w, cov, inc) in &raw {
        *brute_support.entry((s.clone(), t.clone())).or_insert(0) +=
            w * ((*cov as i64) + (*inc as i64));
    }
    // SQL-side support for the same subset.
    let mut stmt = conn.prepare(
        "SELECT source_act, target_act, \
                SUM(weight * (is_cover::INTEGER + is_incomparable::INTEGER)) AS support \
         FROM pot_edges_table WHERE (rowid * 7919) % 1000000 < 10000 \
         GROUP BY source_act, target_act",
    )?;
    let mut sql_support: HashMap<(String, String), i64> = HashMap::new();
    {
        let iter = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?;
        for row in iter {
            let (s, t, sup) = row?;
            sql_support.insert((s, t), sup);
        }
    }
    drop(stmt);
    assert_eq!(
        brute_support, sql_support,
        "ORACLE MISMATCH: SQL DFG support diverges from brute-force Rust computation"
    );
    eprintln!(
        "[dfg] oracle support: {} subset edges, {} groups, EXACT match",
        raw.len(),
        sql_support.len()
    );

    // Brute-force p95 on the same subset edges: fetch dwells, sort,
    // linear-interp quantile at idx = 0.95*(n-1) (quantile_cont semantics).
    let mut durations: HashMap<(String, String), Vec<f64>> = HashMap::new();
    {
        let mut stmt = conn.prepare(
            "SELECT source_act, target_act, (t_end_us - t_start_us) AS dwell_us \
             FROM pot_edge_events WHERE (event_id * 7919) % 1000000 < 10000",
        )?;
        let iter = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?;
        for row in iter {
            let (s, t, d) = row?;
            durations.entry((s, t)).or_default().push(d as f64);
        }
    }
    let mut brute_p95: HashMap<(String, String), f64> = HashMap::new();
    for (k, mut ds) in durations {
        ds.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = 0.95 * (ds.len() - 1) as f64;
        let lo = idx.floor() as usize;
        let hi = idx.ceil() as usize;
        brute_p95.insert(k, ds[lo] + (idx - idx.floor()) * (ds[hi] - ds[lo]));
    }
    // SQL-side p95 (native quantile_cont) for the same subset.
    let mut stmt = conn.prepare(
        "SELECT source_act, target_act, \
                quantile_cont(t_end_us - t_start_us, 0.95) AS p95 \
         FROM pot_edge_events WHERE (event_id * 7919) % 1000000 < 10000 \
         GROUP BY source_act, target_act",
    )?;
    let mut sql_p95: HashMap<(String, String), f64> = HashMap::new();
    {
        let iter = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, f64>(2)?,
            ))
        })?;
        for row in iter {
            let (s, t, p) = row?;
            sql_p95.insert((s, t), p);
        }
    }
    drop(stmt);
    assert_eq!(
        brute_p95.len(),
        sql_p95.len(),
        "p95 oracle group-count mismatch"
    );
    for (k, bv) in &brute_p95 {
        let sv = sql_p95
            .get(k)
            .unwrap_or_else(|| panic!("oracle p95 missing group {k:?}"));
        assert!(
            (sv - bv).abs() <= P95_EPS_US,
            "ORACLE MISMATCH p95 for {k:?}: sql={sv} brute={bv}"
        );
    }
    eprintln!(
        "[dfg] oracle p95: {} groups, native quantile_cont vs brute-force Rust within {} us",
        sql_p95.len(),
        P95_EPS_US
    );

    // ---- Phase 5: throughput — 1M-edge DFG in <100 ms, median-of-5 ----
    let mut latencies_ms: Vec<f64> = Vec::with_capacity(THROUGHPUT_RUNS);
    for _ in 0..THROUGHPUT_RUNS {
        let t0 = Instant::now();
        let mut stmt = conn.prepare(DFG_SQL)?;
        let n_rows = stmt
            .query_map([], |r| r.get::<_, i64>(2).map(|_| ()))?
            .count();
        drop(stmt);
        assert!(n_rows > 0, "timed DFG run returned zero rows");
        latencies_ms.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    latencies_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_ms = latencies_ms[THROUGHPUT_RUNS / 2];
    let min_ms = latencies_ms[0];
    let max_ms = latencies_ms[THROUGHPUT_RUNS - 1];
    eprintln!(
        "[dfg] throughput: 1M-edge DFG median-of-5 = {median_ms:.3} ms (min {min_ms:.3}, max {max_ms:.3}); spec < {THROUGHPUT_SPEC_MS} ms"
    );
    assert!(
        median_ms < THROUGHPUT_SPEC_MS,
        "throughput spec violated: median {median_ms:.3} ms >= {THROUGHPUT_SPEC_MS} ms for 1M-edge DFG"
    );

    // ---- Receipt (real measured numbers only) ----
    let top_edge = format!("{}->{}", dfg_rows[0].0, dfg_rows[0].1);
    let receipt = format!(
        "{{\"lane\":\"MAX9\",\"test\":\"dfg_cut_detection\",\"edges\":{EDGES},\
         \"dfg_groups\":{},\"top_support\":{},\"top_edge\":\"{top_edge}\",\
         \"worst_p95_us\":{:.0},\
         \"throughput_ms_median\":{median_ms:.3},\"throughput_ms_min\":{min_ms:.3},\
         \"throughput_ms_max\":{max_ms:.3},\"spec_ms\":{THROUGHPUT_SPEC_MS},\
         \"within_spec\":true,\"oracle_subset\":{ORACLE_SUBSET},\"oracle\":\"EXACT_MATCH\"}}",
        dfg_rows.len(),
        dfg_rows[0].2,
        worst.2,
    );
    std::fs::create_dir_all("/tmp/value_audit").expect("mkdir /tmp/value_audit");
    std::fs::write("/tmp/value_audit/dfg_cut_detection.json", receipt).expect("write receipt");
    eprintln!("[dfg] receipt written: /tmp/value_audit/dfg_cut_detection.json");
    Ok(())
}
