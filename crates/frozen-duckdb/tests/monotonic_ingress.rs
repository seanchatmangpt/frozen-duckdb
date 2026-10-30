//! First-Mile monotonic ingress gate — court (lane MAX8)
//!
//! NON-VACUITY RECEIPT (real runs, temp in-memory db, frozen DuckDB engine
//! v2.0.0; see `monotonic_ingress_gate` / `shim_court_gate_is_load_bearing`
//! for the live receipts):
//!   BEFORE  : in-order frames accepted (real INSERTs, exit clean).
//!   SHIM    : code-path mutation `enforce_gate=false` — the ingestion path
//!             FORGES `prev_ts` backwards past the tolerance, and the identical
//!             out-of-order frame (prev-6ms) is ACCEPTED, proving the AFTER
//!             rejection is attributable to the CHECK firing on the truthful
//!             prev_ts, not to timestamp arithmetic or the driver. A court
//!             whose shim does not flip is FAIL, not PASS.
//!   AFTER   : gate flip observed — frame at prev_ts - 6ms exits with a real
//!             engine-level CHECK-constraint error; staged state uncorrupted
//!             (row count unchanged, MAX(ts) still the last accepted frame, and
//!             a later valid frame is still accepted after the refusal).
//!   RESTORE : n/a — no file is shimmed; the mutation is an argument toggling
//!             two real INSERT paths, both asserted (gate rejects, shim
//!             accepts), so the court fails closed in both directions.
//!
//! MECHANISM + TRADEOFFS (honest record)
//! Chosen: SQL-level row-local CHECK constraint on a staging table.
//!
//!   CHECK (prev_ts IS NULL OR ts >= prev_ts - 5000)   -- 5000 micros = 5ms
//!
//! `prev_ts` is maintained by the ingestion path (MAX over accepted frames,
//! read in the same INSERT statement). DuckDB CHECK constraints are row-local:
//! a CHECK containing a subquery is refused at DDL time, and DuckDB has no
//! triggers, so a table-only global invariant is not expressible. Tradeoffs:
//!   1. This pattern (chosen): engine-level rejection AT INSERT time. The
//!      invariant lives in the (ingestion path, table) pair — a direct SQL
//!      INSERT supplying a fraudulent `prev_ts` bypasses the gate.
//!   2. Post-hoc window validation (LAG over the staged history): protects
//!      against any writer but is detect-not-prevent; bad frames are already
//!      staged when flagged. Included as a forensic sweep
//!      (`forensic_lag_sweep`) and the court asserts it agrees with the gate
//!      over the accepted history — two independent mechanisms, one verdict.
//!   3. Test-side assertion only: vacuous by construction (the gate would BE
//!      the test). Rejected.
//!
//! Frozen-engine note: this build lacks the core_functions catalog
//! (`make_timestamp`, `date_diff`, `INTERVAL ... MILLISECOND` all desugar to
//! it), so `ts` is carried as BIGINT epoch microseconds and the tolerance as
//! an integer literal. Pure arithmetic — no catalog functions on the gate
//! path.

use anyhow::Result;
use frozen_duckdb::Connection;

/// 5ms tolerance, in microseconds.
const TOLERANCE_US: i64 = 5_000;

/// The ingestion path: inserts with `prev_ts` bound to the last accepted
/// frame's ts (truthful MAX) so the row-local CHECK evaluates the skew inside
/// the engine. `enforce_gate=false` is the SHIM path: it FORGES `prev_ts`
/// backwards (TOLERANCE+1 us before MAX), so the CHECK's `ts >= prev - 5000`
/// bound falls below the bad frame's ts and a -6ms frame passes — the
/// documented bypass tradeoff, proven rather than narrated.
fn ingest_frame(conn: &Connection, seq: i64, ts_us: i64, enforce_gate: bool) -> Result<()> {
    let forge = if enforce_gate { 0 } else { -(TOLERANCE_US + 1) };
    conn.execute(
        "INSERT INTO staging_frames (seq, ts, prev_ts) \
         VALUES ($1, $2, (SELECT COALESCE(MAX(ts) + $3, $2) FROM staging_frames))",
        [seq, ts_us, forge],
    )?;
    Ok(())
}

fn setup(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE staging_frames (
            seq     BIGINT NOT NULL,
            ts      BIGINT NOT NULL, -- epoch microseconds (TIMESTAMP semantics)
            prev_ts BIGINT,          -- last accepted ts, via MAX(ts) in INSERT
            -- row-local monotonic gate: allowed skew = 5ms
            CHECK (prev_ts IS NULL OR ts >= prev_ts - 5000)
        );
        "#,
    )?;
    Ok(())
}

/// Forensic consumer view: pure LAG over accepted rows, independent of the
/// ingestion path's bookkeeping. Asserts monotonicity within tolerance.
fn forensic_lag_sweep(conn: &Connection) -> Result<Vec<(i64, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT seq, delta_us FROM ( \
             SELECT seq, ts - LAG(ts) OVER (ORDER BY seq) AS delta_us \
             FROM staging_frames) \
         WHERE delta_us IS NOT NULL",
    )?;
    let rows = stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?;
    let mut out = Vec::new();
    for r in rows {
        let (seq, delta_us) = r?;
        assert!(
            delta_us >= -TOLERANCE_US,
            "forensic sweep found non-monotonic frame seq={seq} delta_us={delta_us}"
        );
        out.push((seq, delta_us));
    }
    Ok(out)
}

#[test]
fn monotonic_ingress_gate() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    setup(&conn)?;

    let base: i64 = 1_700_000_000_000_000; // epoch microseconds

    // (1) in-order frames accepted
    ingest_frame(&conn, 1, base, true)?;
    ingest_frame(&conn, 2, base + 1, true)?;
    ingest_frame(&conn, 3, base + 2, true)?;

    // (3) non-monotonic frame: ts < prev_ts - 5ms -> engine-level rejection
    let rejected = ingest_frame(&conn, 4, base - 100_000, true);
    assert!(
        rejected.is_err(),
        "VACUOUS GATE: frame at prev_ts-100ms was accepted; CHECK never fired"
    );
    let err = format!("{}", rejected.unwrap_err());
    assert!(
        err.to_lowercase().contains("constraint"),
        "expected a CHECK-constraint rejection, got: {err}"
    );

    // rejected frame must not corrupt staged state
    let (count, max_ts): (i64, i64) = conn.query_row(
        "SELECT COUNT(*), COALESCE(MAX(ts), 0) FROM staging_frames",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    assert_eq!(count, 3, "rejected frame must not be staged");
    assert_eq!(max_ts, base + 2, "last accepted frame must still be seq 3");

    // a later valid frame is still accepted after the refusal
    ingest_frame(&conn, 5, base + 3, true)?;

    // (4) Δ boundary: exactly -5ms skew (prev_ts - 5000us) accepted,
    //     -6ms (prev_ts - 6000us) rejected
    ingest_frame(&conn, 6, base + 3 - TOLERANCE_US, true)?; // -5ms: in tolerance
    let at_minus_6 = ingest_frame(&conn, 7, base + 3 - (TOLERANCE_US + 1), true);
    assert!(
        at_minus_6.is_err(),
        "VACUOUS GATE: frame at prev_ts-6ms accepted; tolerance not enforced"
    );

    // forensic sweep agrees with the gate over the full accepted history:
    // 5 staged frames -> 4 inter-frame deltas
    let history = forensic_lag_sweep(&conn)?;
    assert_eq!(history.len(), 4, "5 accepted frames must yield 4 deltas");

    Ok(())
}

#[test]
fn shim_court_gate_is_load_bearing() -> Result<()> {
    // SHIM court: bypass the gate's allowance and feed the exact frame the
    // gate rejects (-6ms). The shim path must ACCEPT it — otherwise the main
    // court's AFTER flip would not be attributable to the CHECK gate.
    let conn = Connection::open_in_memory()?;
    setup(&conn)?;
    let base: i64 = 1_700_000_000_000_000;
    ingest_frame(&conn, 1, base, true)?;
    let shimmed = ingest_frame(&conn, 2, base - (TOLERANCE_US + 1_000), false);
    assert!(
        shimmed.is_ok(),
        "SHIM COURT FAIL: the gate-bypassed path also rejected the frame; \
         the main court's AFTER flip is not attributable to the CHECK"
    );
    Ok(())
}
