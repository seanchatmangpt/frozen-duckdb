//! MAX4 court: `column_count()` on a prepared-but-not-executed statement must
//! not panic (previously `raw_statement.rs` FIXME at line ~207).
//!
//! Contract (documented in `RawStatement::column_count`): pre-execution the
//! count comes from `duckdb_prepared_statement_column_count` on the prepared
//! statement; post-execution it still reads the executed arrow result. Both
//! calls are on the public `Statement::column_count` path.

use frozen_duckdb::duckdb::Connection;

#[test]
fn column_count_before_execution_does_not_panic() {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    conn.execute_batch("CREATE TABLE t (a INTEGER, b TEXT);")
        .expect("create table");

    let stmt = conn
        .prepare("SELECT a, b FROM t")
        .expect("prepare selects 2 columns");

    // Pre-execution: old code panicked here (result_unwrap on None).
    let n = stmt.column_count();
    assert_eq!(n, 2, "prepared statement reports its result arity");
}

#[test]
fn column_count_post_execution_unchanged() {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    conn.execute_batch("CREATE TABLE t (a INTEGER, b TEXT);")
        .expect("create table");

    let mut stmt = conn.prepare("SELECT a, b FROM t").expect("prepare");
    stmt.execute([]).expect("execute");
    assert_eq!(stmt.column_count(), 2, "post-execution path unchanged");
}
