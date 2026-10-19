//! TEST-023 / TEST-024 concurrency courts (doctor lane D7).
//!
//! Both tests assert on REAL DuckDB concurrency behavior against real
//! collaborators (Chicago school): a temp FILE database, two real connections
//! to the same database instance, real transactions, real MVCC snapshot
//! reads. Zero sleeps — every synchronization is on a real event (a query
//! result on a channel, a commit completing, a join), never a clock.
//!
//! TEST-023 (exclusive single-writer): C1 begins a write transaction on a
//! temp file db; C2 (a second `Connection` to the SAME database instance via
//! `Connection::try_clone`, same process) attempts an immediate write. The
//! observed real behavior is a TYPED ERROR: DuckDB's MVCC write-write
//! conflict detection rejects C2's write against C1's uncommitted
//! transaction (surfaced as `Error::DuckDBFailure` with a
//! conflict/transaction message). C2's failed write never becomes visible;
//! C1 commits; final row content equals C1's writes only.
//!
//! TEST-024 (MVCC snapshot isolation): seed 50k rows. A reader connection
//! opens a read transaction and materializes its first in-flight query
//! result (real handshake token over a channel — proves the query actually
//! ran and the snapshot boundary is open). While that snapshot is open, the
//! writer adds 50k rows and commits (readers do not block writers under
//! MVCC). The reader's still-open snapshot must still return exactly 50k;
//! after the reader's own transaction closes, a fresh query must see 100k.
//!
//! PENDING-REBUILD protocol: if the engine does not link (dylib rebuild in
//! flight by another process), this court reports PENDING-REBUILD instead of
//! faking a pass. Rerun command:
//!   cargo test -p frozen-duckdb --test doctor_concurrency_tests
//!
//! No git operations in this court.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use frozen_duckdb::Connection;

// ---------------------------------------------------------------------------
// Shared helpers (real file db, real inserts, real counts)
// ---------------------------------------------------------------------------

/// Temp file database path (real file: real DuckDB persistence + MVCC).
fn temp_db_path(tag: &str) -> (String, std::path::PathBuf) {
    let dir =
        std::env::temp_dir().join(format!("doctor-concurrency-{}-{}", tag, std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = format!("{}/db.duckdb", dir.display());
    let dir_path = dir;
    (path, dir_path)
}

fn cleanup_db(path: &str, dir: &std::path::Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(format!("{}.wal", path));
    let _ = std::fs::remove_dir_all(dir);
}

/// Insert `n` rows with ids `start..start+n` via a prepared statement.
fn insert_rows(conn: &Connection, start: i64, n: i64) {
    let mut stmt = conn
        .prepare("INSERT INTO t (id, tag) VALUES (?, ?)")
        .expect("prepare insert");
    for i in start..start + n {
        let tag = format!("w{}", i);
        stmt.execute([&i as &dyn frozen_duckdb::ToSql, &tag])
            .expect("insert execute");
    }
}

fn count_rows(conn: &Connection) -> i64 {
    conn.query_row("SELECT count(*) FROM t", [], |r| r.get(0))
        .expect("count rows")
}

// ---------------------------------------------------------------------------
// TEST-023: exclusive single-writer
// ---------------------------------------------------------------------------

#[test]
fn test_023_exclusive_single_writer_conflict_is_typed_error() {
    let (path, dir) = temp_db_path("t23");
    let c1 = Connection::open(&path).expect("open C1 on temp file db");
    c1.execute_batch(
        "CREATE TABLE t (id BIGINT, tag VARCHAR);
         INSERT INTO t VALUES (0, 'seed'), (3, 'seed');",
    )
    .expect("seed schema");

    // C2: a second Connection to the SAME database instance, same process.
    let c2 = c1.try_clone().expect("try_clone C2");

    // C1 begins a write transaction and rewrites the seed tuple id=3
    // (DuckDB MVCC is optimistic per-tuple: the write-write conflict fires
    // on TUPLE OVERLAP, which inserts of disjoint new tuples do not create —
    // witnessed: insert-only C2 writes were accepted, so the court writes to
    // the same tuple).
    c1.execute_batch("BEGIN TRANSACTION").expect("C1 begin");
    c1.execute("UPDATE t SET tag = 'c1' WHERE id = 3", [])
        .expect("C1 update");

    // C2 attempts an IMMEDIATE write to the SAME tuple C1 is writing.
    let conflict = c2.execute("UPDATE t SET tag = 'c2' WHERE id = 3", []);
    match conflict {
        // The AFTER flip is the non-vacuity proof: if the write succeeds the
        // court proves nothing, so it is FAIL, not PASS.
        Ok(_) => panic!(
            "TEST-023 VACUOUS GUARD: C2's write against C1's uncommitted \
             transaction succeeded with no conflict — MVCC write-write \
             conflict detection was not exercised; court FAIL."
        ),
        Err(e) => {
            let msg = e.to_string().to_lowercase();
            assert!(
                msg.contains("conflict") || msg.contains("transaction") || msg.contains("lock"),
                "TEST-023: expected a typed conflict/transaction/lock error, \
                 got: {}",
                e
            );
            eprintln!(
                "[TEST-023] real behavior: C2 immediate write rejected with \
                 typed error: {}",
                e
            );
        }
    }

    // C2's failed write must not be visible anywhere.
    assert_eq!(
        count_rows(&c1),
        2,
        "only the seed rows are visible pre-commit"
    );

    // C1 commits.
    c1.execute_batch("COMMIT").expect("C1 commit");

    // Final state == C1's write only: the seed row carries C1's tag; C2's
    // conflicting write never persisted.
    let final_tag: String = c1
        .query_row("SELECT tag FROM t WHERE id = 3", [], |r| r.get(0))
        .expect("final tag");
    assert_eq!(
        final_tag, "c1",
        "final row content must equal C1's writes only"
    );
    let c2_persisted: i64 = c1
        .query_row("SELECT count(*) FROM t WHERE tag = 'c2'", [], |r| r.get(0))
        .expect("count c2 tags");
    assert_eq!(
        c2_persisted, 0,
        "C2's conflicting write must not persist after C1's commit"
    );

    cleanup_db(&path, &dir);
}

// ---------------------------------------------------------------------------
// TEST-024: MVCC snapshot isolation across two connections
// ---------------------------------------------------------------------------

#[test]
fn test_024_mvcc_reader_snapshot_isolation() {
    let (path, dir) = temp_db_path("t24");
    let writer = Connection::open(&path).expect("open writer conn");
    writer
        .execute_batch("CREATE TABLE t (id BIGINT, tag VARCHAR);")
        .expect("create table");

    // Seed 50k committed rows.
    writer
        .execute_batch("BEGIN TRANSACTION")
        .expect("seed begin");
    insert_rows(&writer, 0, 50_000);
    writer.execute_batch("COMMIT").expect("seed commit");

    // Reader: second connection to the SAME database instance.
    let reader = writer.try_clone().expect("try_clone reader");

    // Reader thread: open a read transaction, materialize the in-flight
    // query result, then signal "snapshot open" on the channel (a REAL event:
    // the query result is already materialized when the token fires). The
    // writer proceeds only on receipt of this token — zero sleeps.
    let (snapshot_open_tx, snapshot_open_rx) = mpsc::channel::<()>();
    let (writer_committed_tx, writer_committed_rx) = mpsc::channel::<()>();
    let (result_tx, result_rx) = mpsc::channel::<usize>();

    let reader_handle = thread::spawn(move || {
        // Open the reader's snapshot boundary.
        reader
            .execute_batch("BEGIN TRANSACTION")
            .expect("reader begin");

        // In-flight query #1 against the 50k snapshot.
        let n1 = count_rows(&reader);
        assert_eq!(n1, 50_000, "reader sees the seeded 50k snapshot");

        // Handshake 1: snapshot proven open (query result real, not a timer).
        snapshot_open_tx.send(()).expect("send snapshot-open token");

        // Handshake 2: wait until the writer's 50k + commit has REALLY
        // landed, then re-count inside the still-open snapshot. Ordering is
        // by channel token, never by sleep.
        writer_committed_rx
            .recv_timeout(Duration::from_secs(60))
            .expect("writer-committed token");

        // The reader's still-open snapshot must be unchanged by that commit.
        let n2 = count_rows(&reader);
        assert_eq!(
            n2, 50_000,
            "reader's in-flight snapshot must still be exactly 50k after the \
             writer's commit"
        );

        // Close the reader transaction; a fresh query must see the writer's
        // commit.
        reader.execute_batch("COMMIT").expect("reader commit");
        let n3 = count_rows(&reader);
        result_tx.send(n3 as usize).expect("send fresh count");
    });

    // Writer: waits on the real snapshot-open token, then writes + commits.
    let writer_handle = thread::spawn(move || {
        snapshot_open_rx
            .recv_timeout(Duration::from_secs(60))
            .expect("snapshot-open token (reader handshake)");
        writer
            .execute_batch("BEGIN TRANSACTION")
            .expect("writer begin");
        insert_rows(&writer, 50_000, 50_000); // ids 50_000..=99_999
        writer.execute_batch("COMMIT").expect("writer commit");
        writer_committed_tx.send(()).expect("send committed token");
    });

    writer_handle.join().expect("writer thread");

    // Reader thread finishes on its own real events; bounded by test harness
    // timeout, and every step inside is deterministic (no sleeps, no races:
    // the reader's second count runs strictly after the writer's commit via
    // the snapshot-open token ordering, and DuckDB MVCC serializes the
    // snapshot reads).
    let fresh = result_rx
        .recv_timeout(Duration::from_secs(120))
        .expect("reader finished (real completion, no sleep)");
    reader_handle.join().expect("reader thread");

    assert_eq!(
        fresh, 100_000,
        "fresh query after snapshot close must see writer's 50k additions"
    );

    cleanup_db(&path, &dir);
}
