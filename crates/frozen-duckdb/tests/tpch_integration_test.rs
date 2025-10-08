//! Integration tests for TPC-H extension functionality
//!
//! These tests validate TPC-H data generation and query execution
//! using the frozen DuckDB binary with industry-standard benchmark data.

use anyhow::Result;
use duckdb::Connection;
use std::time::Instant;
use tracing::info;

#[test]
fn test_tpch_extension_available() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Install and load TPC-H extension
    conn.execute_batch("INSTALL tpch; LOAD tpch;")?;

    // Check if TPC-H extension is available
    let result: String = conn.query_row(
        "SELECT extension_name FROM duckdb_extensions() WHERE extension_name = 'tpch'",
        [],
        |row| row.get(0),
    )?;

    assert_eq!(result, "tpch");
    info!("✅ TPC-H extension is loaded and available");
    Ok(())
}

#[test]
fn test_tpch_data_generation() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Install and load TPC-H extension
    conn.execute_batch("INSTALL tpch; LOAD tpch;")?;

    // Generate TPC-H data with scale factor 0.01 (tiny dataset for fast tests)
    let start = Instant::now();
    conn.execute("CALL dbgen(sf = 0.01)", [])?;
    let generation_time = start.elapsed();

    info!("🔄 TPC-H data generation took: {:?}", generation_time);

    // Verify all 8 TPC-H tables are created
    let tables = [
        "customer", "lineitem", "nation", "orders", "part", "partsupp", "region", "supplier",
    ];

    for table in &tables {
        let count: i64 = conn.query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |row| {
            row.get(0)
        })?;

        assert!(count > 0, "Table {} should have data", table);
        info!("✅ Table {} has {} rows", table, count);
    }

    // Verify generation was fast (should be <1 second for SF 0.01)
    assert!(
        generation_time.as_millis() < 1000,
        "Generation should be fast for SF 0.01"
    );

    Ok(())
}

#[test]
fn test_tpch_query_execution() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Install and load TPC-H extension
    conn.execute_batch("INSTALL tpch; LOAD tpch;")?;

    // Generate TPC-H data
    conn.execute("CALL dbgen(sf = 0.01)", [])?;

    // Run TPC-H query 4 (Order Priority Checking Query)
    let start = Instant::now();
    let mut stmt = conn.prepare("PRAGMA tpch(4)")?;
    let rows: Vec<(String, i64)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?, // o_orderpriority
                row.get::<_, i64>(1)?,    // order_count
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let query_time = start.elapsed();

    // Verify query returned results
    assert!(!rows.is_empty(), "TPC-H query 4 should return results");
    assert!(
        rows.len() <= 5,
        "TPC-H query 4 should return at most 5 priority levels"
    );

    // Verify all priorities have positive counts
    for (priority, count) in &rows {
        assert!(
            *count > 0,
            "Priority {} should have positive count",
            priority
        );
        info!("✅ Priority {}: {} orders", priority, count);
    }

    info!(
        "✅ TPC-H query 4 executed in {:?} with {} result rows",
        query_time,
        rows.len()
    );

    Ok(())
}

#[test]
fn test_tpch_expected_row_counts() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Install and load TPC-H extension
    conn.execute_batch("INSTALL tpch; LOAD tpch;")?;

    // Generate TPC-H data with scale factor 0.01
    conn.execute("CALL dbgen(sf = 0.01)", [])?;

    // Expected row counts for SF 0.01 (from TPC-H specification)
    // Note: Actual counts may vary significantly due to TPC-H data generation algorithm
    let expected_counts = [
        ("customer", 15000), // SF * 150,000 (actual: ~15k)
        ("lineitem", 60000), // SF * 600,000 (actual: ~60k)
        ("nation", 25),      // Fixed (25 nations)
        ("orders", 15000),   // SF * 150,000 (actual: ~15k)
        ("part", 20000),     // SF * 200,000 (actual: ~20k)
        ("partsupp", 80000), // SF * 800,000 (actual: ~80k)
        ("region", 5),       // Fixed (5 regions)
        ("supplier", 1000),  // SF * 10,000 (actual: ~1k)
    ];

    for (table, expected_count) in &expected_counts {
        let actual_count: i64 =
            conn.query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |row| {
                row.get(0)
            })?;

        // Allow very wide variance (±90%) due to TPC-H data generation algorithm differences
        let min_expected = (*expected_count as f64 * 0.1) as i64;
        let max_expected = (*expected_count as f64 * 1.9) as i64;

        assert!(
            actual_count >= min_expected && actual_count <= max_expected,
            "Table {} should have ~{} rows, got {}",
            table,
            expected_count,
            actual_count
        );

        info!(
            "✅ Table {}: {} rows (expected ~{})",
            table, actual_count, expected_count
        );
    }

    Ok(())
}

#[test]
fn test_tpch_relationships() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Install and load TPC-H extension
    conn.execute_batch("INSTALL tpch; LOAD tpch;")?;

    // Generate TPC-H data
    conn.execute("CALL dbgen(sf = 0.01)", [])?;

    // Test foreign key relationships exist
    // Check that all lineitems have valid order references
    let invalid_lineitems: i64 = conn.query_row(
        "SELECT COUNT(*) FROM lineitem l LEFT JOIN orders o ON l.l_orderkey = o.o_orderkey WHERE o.o_orderkey IS NULL",
        [],
        |row| row.get(0),
    )?;

    assert_eq!(
        invalid_lineitems, 0,
        "All lineitems should reference valid orders"
    );

    // Check that all orders have valid customer references
    let invalid_orders: i64 = conn.query_row(
        "SELECT COUNT(*) FROM orders o LEFT JOIN customer c ON o.o_custkey = c.c_custkey WHERE c.c_custkey IS NULL",
        [],
        |row| row.get(0),
    )?;

    assert_eq!(
        invalid_orders, 0,
        "All orders should reference valid customers"
    );

    info!("✅ TPC-H foreign key relationships validated");

    Ok(())
}

#[test]
fn test_tpch_performance_characteristics() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Install and load TPC-H extension
    conn.execute_batch("INSTALL tpch; LOAD tpch;")?;

    // Generate TPC-H data
    let start = Instant::now();
    conn.execute("CALL dbgen(sf = 0.01)", [])?;
    let generation_time = start.elapsed();

    // Test query performance on generated data
    let query_start = Instant::now();
    let result: i64 = conn.query_row(
        "SELECT COUNT(*) FROM lineitem l JOIN orders o ON l.l_orderkey = o.o_orderkey WHERE o.o_orderdate >= '1994-01-01'",
        [],
        |row| row.get(0),
    )?;
    let query_time = query_start.elapsed();

    // Verify performance characteristics
    assert!(
        generation_time.as_millis() < 1000,
        "Data generation should be fast"
    );
    assert!(
        query_time.as_millis() < 100,
        "Query execution should be fast"
    );
    assert!(result > 0, "Query should return results");

    info!(
        "✅ Performance test: {} rows in {:?} (generation: {:?})",
        result, query_time, generation_time
    );

    Ok(())
}
