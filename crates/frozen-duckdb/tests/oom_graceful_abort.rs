//! GATE-V-03 OOM graceful-abort court (durable tests/ port of R7's rig,
//! per docs/design/gate-v03-memory-ceiling.md).
//!
//! Probe = materializing CTAS (2^17 x 2^10 rows, 2GiB nominal) under a
//! 256MB `memory_limit` cap with spill fenced (`temp_directory=''`).
//! The cap must yield a typed "Out of Memory" error — never a panic or
//! process kill — and engine state must survive on the same connection
//! (`SELECT 42`, small CTAS + INSERT + SELECT round-trip).
//!
//! Two-sided non-vacuity:
//! - cap side: probe genuinely memory-bound (streaming count(*) form is
//!   vacuous by construction — see design doc witness).
//! - completion side (`#[ignore]`d, heavy for CI): at `memory_limit=4GB`
//!   the SAME probe completes — the probe tests the cap, not the query.
//!   Run once manually: `cargo test -p frozen-duckdb --test oom_graceful_abort -- --ignored --nocapture`

use frozen_duckdb::{Config, Connection};

const PROBE_SQL: &str = "CREATE TABLE probe_t AS SELECT a.range AS x, b.range AS y \
                         FROM range(131072) a, range(1024) b;";

/// Spill fence: without this the engine spills to disk and never hits
/// the cap (witnessed in the design doc; the cap is a memory cap, not a
/// workload cap).
const SPILL_FENCE: &str = "SET temp_directory=''";

#[test]
fn oom_at_256mb_is_typed_and_state_survives() {
    let config = Config::default()
        .with("memory_limit", "256MB")
        .expect("config with memory_limit=256MB");
    let conn = Connection::open_in_memory_with_flags(config).expect("open in-memory db");

    conn.execute_batch(SPILL_FENCE)
        .expect("spill fence (temp_directory='')");

    // The probe must fail TYPED — a panic/abort kills the test process,
    // so reaching this assert at all is the no-crash proof.
    let err = conn
        .execute_batch(PROBE_SQL)
        .expect_err("2GiB probe must OOM under 256MB cap with spill fenced");

    let msg = err.to_string();
    assert!(
        msg.contains("Out of Memory"),
        "expected typed Out of Memory error, got: {msg}"
    );

    // Same connection, state preserved.
    let v: i32 = conn
        .query_row("SELECT 42", [], |r| r.get(0))
        .expect("post-OOM SELECT 42");
    assert_eq!(v, 42);

    // Same connection, small CTAS + INSERT + SELECT round-trip.
    conn.execute_batch(
        "CREATE TABLE t7 AS SELECT 1 AS x; \
         INSERT INTO t7 VALUES (2);",
    )
    .expect("post-OOM CTAS + INSERT");
    let v: i32 = conn
        .query_row("SELECT sum(x) FROM t7", [], |r| r.get(0))
        .expect("post-OOM SELECT round-trip");
    assert_eq!(v, 3);
}

/// Completion-side non-vacuity upper leg: the same probe at a 4GB cap
/// must COMPLETE (cap not hit). Marked #[ignore]: requires ~2GiB+ of
/// real process RSS — too heavy for CI. Witnessed once manually (see
/// test output in the lane receipt); run again via
/// `cargo test -p frozen-duckdb --test oom_graceful_abort -- --ignored --nocapture`.
#[test]
#[ignore = "heavy: ~2GiB+ real RSS; too heavy for CI; run manually with --ignored"]
fn same_probe_completes_at_4gb_cap() {
    let config = Config::default()
        .with("memory_limit", "4GB")
        .expect("config with memory_limit=4GB");
    let conn = Connection::open_in_memory_with_flags(config).expect("open in-memory db");

    conn.execute_batch(SPILL_FENCE)
        .expect("spill fence (temp_directory='')");

    // Upper leg: the probe COMPLETES — proving the 256MB leg tests the
    // cap, not the query (non-vacuity completion side).
    conn.execute_batch(PROBE_SQL)
        .expect("same probe must complete at 4GB cap (spill fenced)");

    let count: i64 = conn
        .query_row("SELECT count(*) FROM probe_t", [], |r| r.get(0))
        .expect("count on completed probe");
    assert_eq!(count, 134_217_728);
}
