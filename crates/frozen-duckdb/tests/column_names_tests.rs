//! Regression tests for `Statement::column_names()` execution ordering.
//!
//! TR3 (v26.9.21): `column_names()` used to panic via the schema unwrap in
//! `RawStatement::schema()` (raw_statement.rs:218) when called before the
//! statement was executed. It must return `Error::StatementNotExecuted`
//! instead of panicking.

use frozen_duckdb::duckdb::{Connection, Error, Result};

/// TR3 regression: pre-execution `column_names()` returns a proper error
/// instead of panicking with "called `Option::unwrap()` on a `None` value".
#[test]
fn test_column_names_before_execution_returns_error() -> Result<()> {
    let db = Connection::open_in_memory()?;
    let stmt = db.prepare("SELECT 1 AS x, 2 AS y")?;
    match stmt.column_names() {
        Err(Error::StatementNotExecuted) => {}
        Err(e) => panic!("Unexpected error type: {e:?}"),
        Ok(names) => panic!("Expected an error before execution, got {names:?}"),
    }
    Ok(())
}

/// Post-execution `column_names()` still returns the column names.
#[test]
fn test_column_names_after_execution() -> Result<()> {
    let db = Connection::open_in_memory()?;
    let mut stmt = db.prepare("SELECT 1 AS x, 2 AS y")?;
    stmt.execute([])?;
    assert_eq!(stmt.column_names()?, vec!["x".to_string(), "y".to_string()]);
    Ok(())
}

/// An empty result (zero rows) still exposes the column names.
#[test]
fn test_column_names_empty_result() -> Result<()> {
    let db = Connection::open_in_memory()?;
    db.execute_batch("CREATE TABLE t(a INTEGER, b VARCHAR);")?;
    let mut stmt = db.prepare("SELECT a, b FROM t WHERE 1 = 0")?;
    stmt.execute([])?;
    assert_eq!(stmt.column_names()?, vec!["a".to_string(), "b".to_string()]);
    Ok(())
}
