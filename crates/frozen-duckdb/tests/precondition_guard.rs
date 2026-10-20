//! Lane MAX5 court — STALE_PRECONDITION_REFUSED two-phase guard.
//!
//! Non-vacuity protocol: prove the guard can REFUSE (mutation witness), not
//! only that it can pass. Chicago-style: real temp-file DuckDB database, real
//! out-of-band mutation, assertion on real observed state — no mocks.
//!
//! RECEIPT (2026-10-19, lane MAX5, subject
//! crates/frozen-duckdb/src/duckdb/precondition.rs + tests/precondition_guard.rs):
//! `cargo test -p frozen-duckdb --test precondition_guard` →
//! `test result: ok. 4 passed; 0 failed` (exit 0), plus
//! `cargo clippy -p frozen-duckdb -- -D warnings` exit 0. See final message
//! of the authoring lane for the exact run output.

use frozen_duckdb::duckdb::{Connection, precondition::{PreconditionError, PreconditionGuard}};
use std::path::PathBuf;
use tempfile::TempDir;

fn short_hash(h: &str) -> String {
    h.chars().take(4).collect()
}

fn got_hash_of(err: &PreconditionError) -> &str {
    match err {
        PreconditionError::Stale { got, .. } => got,
        _ => unreachable!("court only inspects Stale"),
    }
}

fn open_temp_db() -> (TempDir, PathBuf, Connection) {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("court.duckdb");
    let conn = Connection::open(&path).expect("open temp db");
    conn.execute_batch(
        "CREATE TABLE ledger (id INTEGER, amount BIGINT, note TEXT);
         INSERT INTO ledger VALUES (1, 100, 'a'), (2, 200, 'b'), (3, 300, NULL);",
    )
    .expect("seed table");
    (dir, path, conn)
}

/// Court 1: analyze → out-of-band mutation → guarded execute → REFUSED with
/// the typed message, and the mutation is NOT applied.
#[test]
fn court_1_stale_precondition_refused() {
    let (_dir, _path, conn) = open_temp_db();
    let expected = PreconditionGuard::analyze(&conn, "ledger").expect("analyze");

    // Out-of-band mutation (not via the guard): state drifts.
    conn.execute_batch("INSERT INTO ledger VALUES (4, 400, 'rogue');")
        .expect("out-of-band mutation");

    // Phase 2 attempt with the stale hash must refuse.
    let err = PreconditionGuard::execute_if_unchanged(
        &conn,
        "ledger",
        &expected,
        "INSERT INTO ledger VALUES (9, 999, 'guard-ran')",
    )
    .expect_err("stale precondition must refuse");

    match &err {
        PreconditionError::Stale { expected, got } => {
            assert_ne!(expected, got, "hashes must differ after mutation");
        }
        other => panic!("expected Stale, got {other:?}"),
    }
    let msg = err.to_string();
    assert!(
        msg.starts_with("STALE_PRECONDITION_REFUSED: expected "),
        "typed message missing: {msg}"
    );
    // Both hash4 tokens present and distinct, in `expected ... got` order.
    let parts: Vec<String> = msg
        .split_whitespace()
        .map(|t| t.trim_end_matches(',').to_string())
        .collect();
    // Token layout: ["STALE_PRECONDITION_REFUSED:", "expected", "<h4>,",
    // "got", "<h4>"].
    assert_eq!(parts[1], "expected", "order: {msg}");
    assert_eq!(parts[3], "got", "order: {msg}");
    assert_eq!(parts[2], short_hash(&expected), "expected hash4: {msg}");
    assert_eq!(parts[4], short_hash(got_hash_of(&err)), "got hash4: {msg}");
    assert_ne!(parts[2], parts[4], "hash4 tokens must differ: {msg}");

    // Refusal is pre-execution: the guarded row is absent.
    let n: i64 = conn
        .query_row("SELECT count(*) FROM ledger WHERE id = 9", [], |r| r.get(0))
        .expect("count");
    assert_eq!(n, 0, "guarded SQL must not run on stale precondition");
}

/// Court 2: analyze → no mutation → guarded execute → PASSES, and the
/// returned new_hash equals a fresh analyze after the action.
#[test]
fn court_2_clean_execute_passes() {
    let (_dir, _path, conn) = open_temp_db();
    let h0 = PreconditionGuard::analyze(&conn, "ledger").expect("analyze");

    let out = PreconditionGuard::execute_if_unchanged(
        &conn,
        "ledger",
        &h0,
        "INSERT INTO ledger VALUES (5, 500, 'clean')",
    )
    .expect("clean guarded execute");

    assert!(out.rows_affected >= 1);
    let h1 = PreconditionGuard::analyze(&conn, "ledger").expect("re-analyze");
    assert_eq!(out.new_hash, h1, "returned hash is the real post-state");
    assert_ne!(out.new_hash, h0, "hash must move after the insert");
    let n: i64 = conn
        .query_row("SELECT count(*) FROM ledger WHERE id = 5", [], |r| r.get(0))
        .expect("count");
    assert_eq!(n, 1);
}

/// Court 3: the two-phase commit analog — mutation SQL carries the
/// precondition hash in a `/* PRECONDITION:<hex> */` header; a mismatched
/// hash aborts BEFORE execution (state unchanged), a matching hash applies.
#[test]
fn court_3_two_phase_analog() {
    let (_dir, _path, conn) = open_temp_db();
    let h0 = PreconditionGuard::analyze(&conn, "ledger").expect("analyze");

    // 3a. Mismatched carried hash: aborts before execution.
    let stale_sql = format!(
        "/* PRECONDITION:{} */ UPDATE ledger SET amount = 0 WHERE id = 1",
        "deadbeef"
    );
    let err = PreconditionGuard::execute_two_phase(&conn, "ledger", &stale_sql)
        .expect_err("mismatched carried hash must abort");
    assert!(matches!(err, PreconditionError::Stale { .. }));
    let amount: i64 = conn
        .query_row("SELECT amount FROM ledger WHERE id = 1", [], |r| r.get(0))
        .expect("amount");
    assert_eq!(amount, 100, "abort must be pre-execution: no partial apply");

    // 3b. Matching carried hash: applies, state moves.
    let good_sql = format!("/* PRECONDITION:{h0} */ UPDATE ledger SET amount = 111 WHERE id = 1");
    let out = PreconditionGuard::execute_two_phase(&conn, "ledger", &good_sql)
        .expect("matching carried hash must apply");
    let amount: i64 = conn
        .query_row("SELECT amount FROM ledger WHERE id = 1", [], |r| r.get(0))
        .expect("amount");
    assert_eq!(amount, 111);
    let h1 = PreconditionGuard::analyze(&conn, "ledger").expect("re-analyze");
    assert_eq!(out.new_hash, h1);
    assert_ne!(out.new_hash, h0);
}

/// Court 4: hash is state-sensitive — identical schemas with different data
/// hash differently; identical data hashes identically (across two temp dbs,
/// order-stable). Also a restore-witness: reverting the mutation restores
/// the original hash byte-exact.
#[test]
fn court_4_hash_sensitivity_and_restore() {
    let (_d1, _p1, conn1) = open_temp_db();
    let (_d2, _p2, conn2) = open_temp_db();
    let h1a = PreconditionGuard::analyze(&conn1, "ledger").expect("h1a");
    let h2a = PreconditionGuard::analyze(&conn2, "ledger").expect("h2a");
    assert_eq!(h1a, h2a, "identical contents must hash identically");

    let h1b = PreconditionGuard::analyze(&conn1, "ledger").expect("h1b");
    conn1
        .execute_batch("DELETE FROM ledger WHERE id = 2;")
        .expect("mutate");
    let h1c = PreconditionGuard::analyze(&conn1, "ledger").expect("h1c");
    assert_ne!(h1b, h1c, "deleted row must change the hash");

    // Restore: re-insert the exact deleted row → hash returns byte-exact.
    conn1
        .execute_batch("INSERT INTO ledger VALUES (2, 200, 'b');")
        .expect("restore");
    let h1d = PreconditionGuard::analyze(&conn1, "ledger").expect("h1d");
    assert_eq!(h1b, h1d, "restored state must hash byte-exact to baseline");
}

/// SHA-256 known-answer tests (FIPS 180-4 vectors) — the hash itself is not vacuous.
#[test]
fn sha256_known_answers() {
    use frozen_duckdb::duckdb::precondition::sha256_hex;
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    // 56-byte input crosses the padding boundary (two blocks). Vector
    // computed independently: `printf 'x%.0s' {1..56} | shasum -a 256`.
    let long = vec![b'x'; 56];
    assert_eq!(
        sha256_hex(&long),
        "04c26261370ee7541549d16dee320c723e3fd14671e66a099afe0a377c16888e"
    );
}
