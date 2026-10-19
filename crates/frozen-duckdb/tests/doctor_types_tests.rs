//! TEST-008 + TEST-010 (doctor lane D5): DECIMAL(18,4) exactness and Arrow bridge.
//!
//! TEST-008: DECIMAL(18,4) exactness over 100k rows. Aggregation expected
//! values are computed in the test with i128 arithmetic (software-decimal
//! baseline), never with floats. Division-by-zero behavior is recorded from
//! the actual engine response, never assumed.
//!
//! TEST-010: Arrow bridge — ~1M rows via range(), fetched through the crate's
//! public Arrow path (`Statement::query_arrow`), asserting row count, spot
//! column values, and clean deallocation (drop + no panic).

use frozen_duckdb::{Connection, Value};

/// Row payload for TEST-008: row i holds DECIMAL(18,4) value (i % 10_000) / 100.
/// Unscaled integer (scale 4) = (i % 10_000) * 100 — exact in i128.
fn unscaled(i: i64) -> i128 {
    ((i % 10_000) as i128) * 100
}

/// TEST-008 — DECIMAL(18,4) exactness: 100k rows, sum/avg/division vs i128 baseline.
#[test]
fn test_008_decimal_18_4_exactness_100k_rows() {
    let conn = Connection::open_in_memory().expect("open in-memory db");

    conn.execute_batch(
        r#"
        CREATE TABLE t (v DECIMAL(18,4));
        INSERT INTO t SELECT ((i % 10000) / 100.0)::DECIMAL(18,4) FROM range(100000) tbl(i);
        "#,
    )
    .expect("create and seed 100k-row decimal table");

    // Baseline: sum of unscaled (scale-4) integers computed in-test via i128.
    let expected_unscaled_sum: i128 = (0i64..100_000).map(unscaled).sum();
    let n: i128 = 100_000;

    // --- Aggregation exactness: SUM ---
    let sum_str: String = conn
        .query_row("SELECT SUM(v)::VARCHAR FROM t", [], |r| r.get(0))
        .expect("sum query");
    let sum_decimal: rust_decimal::Decimal = sum_str.parse().expect("sum parses as decimal");
    assert!(
        sum_decimal.scale() <= 4,
        "unexpected sum scale {}",
        sum_decimal.scale()
    );
    let sum_unscaled: i128 = sum_decimal.mantissa() * 10i128.pow(4 - sum_decimal.scale());
    assert_eq!(
        sum_unscaled, expected_unscaled_sum,
        "SUM over 100k DECIMAL(18,4) rows diverges from i128 baseline"
    );

    // --- Aggregation exactness: AVG ---
    // AVG returns a wider decimal; compare via VARCHAR to the exact rational.
    // expected_unscaled_sum / 10000 / n must equal AVG exactly at scale 4+.
    let avg_str: String = conn
        .query_row("SELECT AVG(v)::VARCHAR FROM t", [], |r| r.get(0))
        .expect("avg query");
    let avg_decimal: rust_decimal::Decimal = avg_str.parse().expect("avg parses as decimal");
    // expected exact average unscaled at scale 4: expected_unscaled_sum / n must divide evenly
    assert_eq!(
        expected_unscaled_sum % n,
        0,
        "test design requires integral average at scale 4"
    );
    let expected_avg_unscaled = expected_unscaled_sum / n;
    let got_avg_unscaled = avg_decimal.mantissa() * 10i128.pow(4 - avg_decimal.scale());
    assert_eq!(
        got_avg_unscaled, expected_avg_unscaled,
        "AVG over 100k DECIMAL(18,4) rows diverges from i128 baseline"
    );

    // --- Exact division: SUM / 2 (power-of-two, exact at scale 4) ---
    let half_str: String = conn
        .query_row(
            "SELECT (SUM(v) / 2::DECIMAL(18,4))::VARCHAR FROM t",
            [],
            |r| r.get(0),
        )
        .expect("division query");
    let half_decimal: rust_decimal::Decimal = half_str.parse().expect("half parses");
    let half_unscaled = half_decimal.mantissa() * 10i128.pow(4 - half_decimal.scale());
    let expected_half_unscaled = expected_unscaled_sum / 2;
    assert_eq!(
        expected_unscaled_sum % 2,
        0,
        "test design requires even unscaled sum"
    );
    assert_eq!(
        half_unscaled, expected_half_unscaled,
        "SUM(v)/2 diverges from i128 baseline"
    );

    // --- Spot-check individual rows against the i128 generator (scalar, exact) ---
    for i in [0i64, 1, 9999, 50000, 99999] {
        let got: String = conn
            .query_row(
                "SELECT ((i % 10000) / 100.0)::DECIMAL(18,4)::VARCHAR FROM range(100000) tbl(i) WHERE i = ?",
                [i],
                |r| r.get(0),
            )
            .expect("spot query");
        let d: rust_decimal::Decimal = got.parse().expect("spot decimal");
        let unscaled_got = d.mantissa() * 10i128.pow(4 - d.scale());
        assert_eq!(unscaled_got, unscaled(i), "row {i} value diverges");
    }

    // --- Division by zero: record ACTUAL engine behavior, never assume ---
    // RECORDED (this run): DuckDB promotes DECIMAL(18,4)/0 to DOUBLE and returns
    // +infinity — no error, no NULL. Assert that recorded behavior explicitly:
    // the result column types as Double and holds an infinite value.
    let inf: f64 = conn
        .query_row(
            "SELECT 1::DECIMAL(18,4) / 0",
            [],
            |r: &frozen_duckdb::Row| r.get::<_, f64>(0),
        )
        .expect("division-by-zero returns an infinity-valued DOUBLE column");
    assert!(
        inf.is_infinite() && inf > 0.0,
        "RECORDED DIVISION-BY-ZERO BEHAVIOR diverged: expected +inf DOUBLE, got {inf}"
    );
}

/// TEST-010 — Arrow bridge: ~1M rows via range(), query_arrow, spot values, drop.
#[test]
fn test_010_arrow_bridge_1m_rows_range() {
    use frozen_duckdb::arrow::array::Int64Array;
    use frozen_duckdb::arrow::record_batch::RecordBatch;

    let conn = Connection::open_in_memory().expect("open in-memory db");

    let mut stmt = conn
        .prepare("SELECT i FROM range(1000000) tbl(i)")
        .expect("prepare range query");
    let batches: Vec<RecordBatch> = stmt.query_arrow([]).expect("query_arrow").collect();

    let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    assert_eq!(total_rows, 1_000_000, "arrow row count mismatch");

    // Spot column values: first batch head and last batch tail.
    let first = batches.first().expect("at least one batch");
    let col = first
        .column(0)
        .as_any()
        .downcast_ref::<Int64Array>()
        .expect("range column is Int64");
    assert_eq!(col.value(0), 0, "first arrow value");
    assert_eq!(
        col.value(col.len() - 1),
        (col.len() - 1) as i64,
        "first batch tail is contiguous"
    );

    let last = batches.last().expect("at least one batch");
    let lcol = last
        .column(0)
        .as_any()
        .downcast_ref::<Int64Array>()
        .expect("range column is Int64");
    assert_eq!(
        lcol.value(lcol.len() - 1),
        999_999,
        "last arrow value is 999999"
    );

    // Deallocation: drop everything, then do real follow-on work — no panic.
    drop(batches);
    drop(stmt);
    drop(conn);
    let conn2 = Connection::open_in_memory().expect("post-drop connection still works");
    let n: i64 = conn2
        .query_row("SELECT count(*) FROM range(10)", [], |r| r.get(0))
        .expect("post-drop query");
    assert_eq!(n, 10);
}

/// Bridge sanity shared by both tests: the Value enum carries a Decimal variant
/// reachable through the public API (exercised via SUM::VARCHAR parse above;
/// here directly via row.get::<Value>).
#[test]
fn test_decimal_value_enum_roundtrip() {
    let conn = Connection::open_in_memory().expect("open");
    let v: Value = conn
        .query_row("SELECT 12.3456::DECIMAL(18,4)", [], |r| r.get(0))
        .expect("decimal value");
    match v {
        Value::Decimal(d) => {
            assert_eq!(d.scale(), 4);
            assert_eq!(d.mantissa(), 123_456);
        }
        other => panic!("expected Value::Decimal, got {other:?}"),
    }
}
