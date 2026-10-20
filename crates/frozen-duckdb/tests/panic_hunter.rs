//! U12 panic hunter — structured attack battery over the PUBLIC API.
//!
//! Court: every cell must end in either a typed `Err` or a clean success.
//! A PANIC = BLOCKER. Run: `cargo test -p frozen-duckdb --test panic_hunter`.
//!
//! Crate law: unwrap/expect on external input at the boundary is banned.

use frozen_duckdb::{Connection, Config, Result, Value};

/// Assert the call does NOT panic; Ok or typed Err are both fine.
/// Each attack is its own #[test] so a panic isolates to a named cell.
macro_rules! no_panic {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            let f = || -> Result<()> { $body };
            // Only panics matter to this court; outcome is otherwise free.
            let _ = f();
        }
    };
}

// ---------------------------------------------------------------------------
// 1. Null bytes in SQL strings
// ---------------------------------------------------------------------------

no_panic!(nul_in_execute_batch, {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("SELECT 1;\0SELECT 2;")
});

no_panic!(nul_in_prepare, {
    let conn = Connection::open_in_memory()?;
    let mut stmt = conn.prepare("SELECT \0")?;
    let _ = stmt.query([]);
    Ok(())
});

no_panic!(trailing_nul_sql, {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("SELECT 1\x00")
});

// ---------------------------------------------------------------------------
// 2. Config keys/values with NUL (open path)
// ---------------------------------------------------------------------------

no_panic!(nul_in_config_key, {
    let cfg = Config::default().with("ke\0y", "v")?;
    let _ = Connection::open_in_memory_with_flags(cfg);
    Ok(())
});

no_panic!(nul_in_config_value, {
    let cfg = Config::default().with("threads", "4\0x")?;
    let _ = Connection::open_in_memory_with_flags(cfg);
    Ok(())
});

// ---------------------------------------------------------------------------
// 3. Appender table/schema names with NUL
// ---------------------------------------------------------------------------

no_panic!(nul_in_appender_table, {
    let conn = Connection::open_in_memory()?;
    let _ = conn.appender("t\0ab");
    Ok(())
});

no_panic!(nul_in_appender_schema, {
    let conn = Connection::open_in_memory()?;
    let _ = conn.appender_to_db("t", "s\0h");
    Ok(())
});

// ---------------------------------------------------------------------------
// 4. 10MB strings
// ---------------------------------------------------------------------------

no_panic!(ten_mb_string_roundtrip, {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("CREATE TABLE t(s VARCHAR)")?;
    let big = "y".repeat(10 * 1024 * 1024);
    let mut stmt = conn.prepare("INSERT INTO t VALUES (?)")?;
    stmt.execute([&big])?;
    let val: String = conn.query_row("SELECT s FROM t", [], |r| r.get(0))?;
    assert_eq!(val.len(), 10 * 1024 * 1024);
    Ok(())
});

// ---------------------------------------------------------------------------
// 5. Emoji / RTL unicode identifiers
// ---------------------------------------------------------------------------

no_panic!(emoji_identifiers, {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("CREATE TABLE \"🦆table\" (\"👍col\" INT)")
});

no_panic!(rtl_identifiers, {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("CREATE TABLE \"א\u{202E}tbl\" (c INT)")?;
    conn.execute_batch("INSERT INTO \"א\u{202E}tbl\" VALUES (1)")?;
    let n: i64 = conn.query_row("SELECT count(*) FROM \"א\u{202E}tbl\"", [], |r| r.get(0))?;
    assert_eq!(n, 1);
    Ok(())
});

// ---------------------------------------------------------------------------
// 6. Deeply nested parens (1000+)
// ---------------------------------------------------------------------------

no_panic!(deeply_nested_parens, {
    let conn = Connection::open_in_memory()?;
    let sql = format!("SELECT {}1{};", "(".repeat(2000), ")".repeat(2000));
    conn.execute_batch(&sql)
});

// ---------------------------------------------------------------------------
// 7. Empty everything
// ---------------------------------------------------------------------------

no_panic!(empty_sql, {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("")
});

no_panic!(empty_table_name, {
    let conn = Connection::open_in_memory()?;
    let _ = conn.appender("");
    Ok(())
});

no_panic!(empty_prepare, {
    let conn = Connection::open_in_memory()?;
    let mut stmt = conn.prepare("")?;
    let _ = stmt.query([]);
    Ok(())
});

no_panic!(empty_param_bind, {
    let conn = Connection::open_in_memory()?;
    let mut stmt = conn.prepare("SELECT 42")?;
    let mut rows = stmt.query([])?;
    let v: i32 = rows.next()?.unwrap().get(0)?;
    assert_eq!(v, 42);
    Ok(())
});

no_panic!(empty_schema_appender, {
    let conn = Connection::open_in_memory()?;
    let _ = conn.appender_to_db("t", "");
    Ok(())
});

// ---------------------------------------------------------------------------
// 8. i64/u64 boundary binds
// ---------------------------------------------------------------------------

no_panic!(i64_boundaries, {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("CREATE TABLE t(x BIGINT)")?;
    let mut stmt = conn.prepare("INSERT INTO t VALUES (?)")?;
    for v in [i64::MIN, i64::MIN + 1, -1, 0, 1, i64::MAX - 1, i64::MAX] {
        stmt.execute([&v])?;
    }
    let n: i64 = conn.query_row("SELECT count(*) FROM t", [], |r| r.get(0))?;
    assert_eq!(n, 7);
    Ok(())
});

no_panic!(u64_boundaries, {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("CREATE TABLE t(x BIGINT)")?;
    let mut stmt = conn.prepare("INSERT INTO t VALUES (?)")?;
    for v in [0u64, 1, i64::MAX as u64, u64::MAX] {
        stmt.execute([&v])?;
    }
    Ok(())
});

// ---------------------------------------------------------------------------
// 9. f64 NaN/Inf binds
// ---------------------------------------------------------------------------

no_panic!(f64_nan_inf_binds, {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("CREATE TABLE t(x DOUBLE)")?;
    let mut stmt = conn.prepare("INSERT INTO t VALUES (?)")?;
    for v in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.0f64, f64::MIN, f64::MAX] {
        stmt.execute([&v])?;
    }
    let n: i64 = conn.query_row("SELECT count(*) FROM t", [], |r| r.get(0))?;
    assert_eq!(n, 6);
    Ok(())
});

// ---------------------------------------------------------------------------
// 10. Huge BLOB
// ---------------------------------------------------------------------------

no_panic!(huge_blob, {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("CREATE TABLE t(b BLOB)")?;
    let blob = vec![0u8; 8 * 1024 * 1024];
    let mut stmt = conn.prepare("INSERT INTO t VALUES (?)")?;
    stmt.execute([&blob])?;
    let n: i64 = conn.query_row("SELECT octet_length(b) FROM t", [], |r| r.get(0))?;
    assert_eq!(n, 8 * 1024 * 1024);
    Ok(())
});

// ---------------------------------------------------------------------------
// 11. VARCHAR with embedded NUL via bind
// ---------------------------------------------------------------------------

no_panic!(varchar_embedded_nul_bind, {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("CREATE TABLE t(s VARCHAR)")?;
    let s = "before\0after";
    let mut stmt = conn.prepare("INSERT INTO t VALUES (?)")?;
    stmt.execute([&s])?;
    let got: String = conn.query_row("SELECT s FROM t", [], |r| r.get(0))?;
    assert_eq!(got, "before\0after");
    Ok(())
});

// ---------------------------------------------------------------------------
// 12. Negative interval + absurd DECIMAL
// ---------------------------------------------------------------------------

no_panic!(negative_interval, {
    let conn = Connection::open_in_memory()?;
    let v: String = conn.query_row(
        "SELECT (DATE '2026-01-01' - DATE '2026-06-01')",
        [],
        |r| r.get(0),
    )?;
    let _ = v;
    Ok(())
});

no_panic!(absurd_decimal_precision, {
    let conn = Connection::open_in_memory()?;
    // DuckDB caps DECIMAL at precision 38; above must be a typed error.
    let r = conn.execute_batch("CREATE TABLE t(d DECIMAL(77, 40))");
    assert!(r.is_err(), "DECIMAL(77,40) must be a typed error");
    Ok(())
});

no_panic!(decimal_38_boundary, {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("CREATE TABLE t(d DECIMAL(38, 10))")?;
    conn.execute_batch("INSERT INTO t VALUES (1.5)")?;
    let v: Value = conn.query_row("SELECT d FROM t", [], |r| r.get(0))?;
    let _ = v;
    Ok(())
});
