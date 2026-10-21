//! Lane WO2 court: proves the `chrono` feature (declared this lane, KD-06 class
//! defect — previously always-false gate at src/duckdb/types/mod.rs:17) actually
//! compiles and binds. Chicago discipline: real in-memory DuckDB, no mocks.
#![cfg(feature = "chrono")]

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use frozen_duckdb::Connection;

#[test]
fn naive_datetime_timestamp_round_trip() {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    conn.execute_batch("CREATE TABLE t (ts TIMESTAMP)")
        .expect("create table");

    let dt = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2016, 2, 23).unwrap(),
        NaiveTime::from_hms_micro_opt(23, 56, 4, 12_345).unwrap(),
    );

    // Native bind: NaiveDateTime -> TIMESTAMP via ToSql.
    let inserted = conn
        .execute("INSERT INTO t (ts) VALUES (?)", [dt])
        .expect("bind NaiveDateTime");
    assert_eq!(inserted, 1);

    // Read back as TIMESTAMP with round-trip fidelity.
    let back: NaiveDateTime = conn
        .query_row("SELECT ts FROM t", [], |r| r.get(0))
        .expect("read back TIMESTAMP");
    assert_eq!(back, dt, "NaiveDateTime TIMESTAMP round trip must be exact");
}

#[test]
fn datetime_utc_timestamptz_round_trip() {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    let utc = Utc
        .with_ymd_and_hms(2023, 1, 1, 12, 0, 0)
        .single()
        .expect("valid utc datetime");

    conn.execute("CREATE TABLE t (ts TIMESTAMPTZ)", [])
        .expect("create table");
    conn.execute("INSERT INTO t (ts) VALUES (?)", [utc])
        .expect("bind DateTime<Utc>");

    let back: chrono::DateTime<Utc> = conn
        .query_row("SELECT ts FROM t", [], |r| r.get(0))
        .expect("read back TIMESTAMPTZ");
    assert_eq!(
        back, utc,
        "DateTime<Utc> TIMESTAMPTZ round trip must be exact"
    );
}

#[test]
fn server_side_timestamp_filter_sees_native_bind() {
    // The bound value must land as a real TIMESTAMP on the server side, not a
    // lossy string: a native bind participates in TIMESTAMP comparison/arithmetic.
    let conn = Connection::open_in_memory().expect("open in-memory db");
    let dt = NaiveDateTime::parse_from_str("2023-06-15 08:30:00", "%Y-%m-%d %H:%M:%S").unwrap();

    let hit: i32 = conn
        .query_row(
            "SELECT 1 WHERE ?::TIMESTAMP = TIMESTAMP '2023-06-15 08:30:00'",
            [dt],
            |r| r.get(0),
        )
        .expect("server-side TIMESTAMP equality");
    assert_eq!(hit, 1);

    // tz-relative: absolute epoch depends on the DuckDB session timezone, the
    // delta does not. Zero delta proves the bind landed as a real TIMESTAMP.
    let delta_us: i64 = conn
        .query_row(
            "SELECT epoch_us(?::TIMESTAMP) - epoch_us(TIMESTAMP '2023-06-15 08:30:00')",
            [dt],
            |r| r.get(0),
        )
        .expect("epoch delta of bound timestamp");
    assert_eq!(delta_us, 0);
}
