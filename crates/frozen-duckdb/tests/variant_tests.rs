//! VARIANT round-trip integration tests (real DuckDB, no mocks).
//!
//! Working form on this build (v2.0.0 cached dylib), verified by running:
//! - `CREATE TABLE t(v VARIANT)` DDL and `'{...}'::VARIANT` casts are accepted.
//! - Inserting a JSON payload through a varchar bind is implicitly cast
//!   varchar -> VARIANT on insert.
//! - A raw VARIANT column read surfaces through the FFI as
//!   `Struct([("metadata", Blob), ("value", Blob)])`, so `row.get::<_, String>`
//!   on the raw column returns `InvalidColumnType`. The working text read is an
//!   explicit `v::VARCHAR` cast in SQL — the tests below pin that form.

use frozen_duckdb::{params, Connection};

/// Round trip: varchar bind -> VARIANT column -> SQL ::VARCHAR cast -> String.
#[test]
fn variant_roundtrip_varchar_bind_and_read() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute("CREATE TABLE t(v VARIANT)", [])
        .expect("CREATE TABLE with VARIANT column should be accepted on this build");
    conn.execute(
        "INSERT INTO t VALUES (?)",
        params!["{\"a\": 1, \"b\": [1, 2, 3]}"],
    )
    .expect("varchar bind should implicitly cast to VARIANT on insert");

    let s: String = conn
        .query_row("SELECT v::VARCHAR FROM t", [], |row| row.get(0))
        .expect("VARIANT -> VARCHAR cast read should work");
    assert_eq!(s, "{\"a\": 1, \"b\": [1, 2, 3]}");
}

/// Round-trip through a bare-SELECT literal cast.
#[test]
fn variant_cast_literal_roundtrip() {
    let conn = Connection::open_in_memory().unwrap();
    let s: String = conn
        .query_row("SELECT '{\"k\": true}'::VARIANT::VARCHAR", [], |row| {
            row.get(0)
        })
        .expect("varchar -> VARIANT -> varchar cast round trip should work");
    assert_eq!(s, "{\"k\": true}");
}

/// Raw VARIANT reads surface as Struct(metadata, value) through the FFI on this
/// build: reading the raw column as String must be a typed error, not a
/// silently-mangled value.
#[test]
fn variant_raw_column_read_is_typed_error() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute("CREATE TABLE t(v VARIANT)", []).unwrap();
    conn.execute("INSERT INTO t VALUES (?)", params!["{\"a\": 1}"])
        .unwrap();
    let err = conn
        .query_row("SELECT v FROM t", [], |row| {
            let _: String = row.get(0)?;
            Ok(())
        })
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("Invalid column type") && msg.contains("Struct"),
        "expected typed InvalidColumnType(Struct), got: {msg}"
    );
}

/// FromSql for serde_json::Value against the VARIANT text rendering.
/// Requires the crate's `serde_json` feature (gates the FromSql impl).
#[cfg(feature = "serde_json")]
#[test]
fn variant_to_serde_json_value() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute("CREATE TABLE t(v VARIANT)", []).unwrap();
    conn.execute(
        "INSERT INTO t VALUES (?)",
        params![r#"{"name": "frozen", "n": 42}"#],
    )
    .unwrap();
    let v: serde_json::Value = conn
        .query_row("SELECT v::VARCHAR FROM t", [], |row| row.get(0))
        .expect("FromSql<serde_json::Value> should parse the VARIANT JSON rendering");
    assert_eq!(v["name"], "frozen");
    assert_eq!(v["n"], 42);
}

/// Column metadata for a VARIANT column must not panic: execute first (column
/// metadata is only valid post-execution on this FFI), then read name and type.
#[test]
fn variant_column_type_metadata_no_panic() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute("CREATE TABLE t(v VARIANT)", []).unwrap();
    let mut stmt = conn.prepare("SELECT v::VARCHAR AS v FROM t").unwrap();
    stmt.execute([]).unwrap();
    assert_eq!(stmt.column_count(), 1);
    assert_eq!(stmt.column_name(0).unwrap(), "v");
    let ty = stmt.column_type(0);
    assert_eq!(
        ty.to_string(),
        "Utf8",
        "cast column reports Utf8 (arrow string projection); raw VARIANT reports the metadata/value struct"
    );
}

/// The raw VARIANT column reports Type::Variant once the statement has run.
#[test]
fn variant_column_reports_variant_type() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute("CREATE TABLE t(v VARIANT)", []).unwrap();
    let mut stmt = conn.prepare("SELECT v FROM t").unwrap();
    stmt.execute([]).unwrap();
    let ty = stmt.column_type(0);
    assert_eq!(
        ty.to_string(),
        "Struct(metadata Binary, value Binary)",
        "raw VARIANT column projects as struct(metadata blob, value blob) through this FFI"
    );
}
