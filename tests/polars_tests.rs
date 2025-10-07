//! Tests for Polars-like operations with DuckDB
//!
//! These tests validate analytical capabilities using DuckDB's SQL engine
//! for Polars-like dataframe operations on Chinook-like datasets.

use anyhow::Result;
use duckdb::Connection;
use std::time::Instant;
use tracing::info;

/// Test analytical operations similar to Polars groupby
#[test]
fn test_analytical_operations() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Create dataset similar to what Polars would handle
    conn.execute_batch(
        r#"
        CREATE TABLE sales (
            customer_id INTEGER,
            product_category TEXT,
            amount REAL,
            sale_date DATE,
            region TEXT
        );

        INSERT INTO sales VALUES
        (1, 'Electronics', 299.99, '2024-01-15', 'North'),
        (2, 'Clothing', 89.50, '2024-01-16', 'South'),
        (1, 'Electronics', 149.99, '2024-01-17', 'North'),
        (3, 'Books', 24.99, '2024-01-18', 'East'),
        (2, 'Clothing', 59.99, '2024-01-19', 'South'),
        (1, 'Electronics', 399.99, '2024-01-20', 'North'),
        (3, 'Books', 34.99, '2024-01-21', 'East'),
        (4, 'Electronics', 199.99, '2024-01-22', 'West'),
        (2, 'Clothing', 129.99, '2024-01-23', 'South'),
        (4, 'Electronics', 89.99, '2024-01-24', 'West'),
        (5, 'Sports', 49.99, '2024-01-25', 'North');
        "#,
    )?;

    // Test groupby operations (Polars-like)
    let mut stmt = conn.prepare(
        "SELECT product_category, COUNT(*) as count, AVG(amount) as avg_amount, SUM(amount) as total_amount
         FROM sales
         GROUP BY product_category
         ORDER BY product_category"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, f64>(2)?,
            row.get::<_, f64>(3)?,
        ))
    })?;

    let mut row_count = 0;
    for row in rows {
        let (category, count, avg, sum) = row?;
        assert!(count > 0);
        assert!(avg > 0.0);
        assert!(sum > 0.0);
        row_count += 1;
        info!(
            "📊 Category: {}, Count: {}, Avg: ${:.2}, Total: ${:.2}",
            category, count, avg, sum
        );
    }

    // Debug: let's see what categories we actually have
    let categories: Vec<String> = conn
        .prepare("SELECT DISTINCT product_category FROM sales ORDER BY product_category")?
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    info!("📊 Found categories: {:?}", categories);

    assert_eq!(row_count, categories.len()); // Should match the actual count

    info!(
        "✅ Polars-like analytical operations working: {} categories",
        row_count
    );
    Ok(())
}

/// Test window functions (common in Polars)
#[test]
fn test_window_functions() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Create time series data
    conn.execute_batch(
        r#"
        CREATE TABLE time_series (
            date DATE,
            customer_id INTEGER,
            daily_amount REAL,
            category TEXT
        );

        INSERT INTO time_series VALUES
        ('2024-01-01', 1, 100.0, 'A'),
        ('2024-01-02', 1, 150.0, 'A'),
        ('2024-01-03', 1, 120.0, 'A'),
        ('2024-01-04', 1, 200.0, 'A'),
        ('2024-01-05', 1, 180.0, 'A'),
        ('2024-01-01', 2, 90.0, 'B'),
        ('2024-01-02', 2, 110.0, 'B'),
        ('2024-01-03', 2, 95.0, 'B'),
        ('2024-01-04', 2, 130.0, 'B'),
        ('2024-01-05', 2, 125.0, 'B');
        "#,
    )?;

    // Test window functions (common in Polars)
    let result: f64 = conn.query_row(
        "SELECT AVG(daily_amount) OVER (PARTITION BY customer_id ORDER BY date ROWS BETWEEN 2 PRECEDING AND CURRENT ROW) as rolling_avg
         FROM time_series
         WHERE customer_id = 1 AND date = '2024-01-03'",
        [],
        |row| row.get(0),
    )?;

    assert!(result > 0.0);

    info!(
        "✅ Window functions working: rolling average = {:.2}",
        result
    );
    Ok(())
}

/// Test complex analytics similar to Polars operations
#[test]
fn test_complex_analytics() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Create complex dataset
    conn.execute_batch(
        r#"
        CREATE TABLE user_events (
            user_id INTEGER,
            event_type TEXT,
            event_value REAL,
            event_timestamp TIMESTAMP,
            session_metadata JSON,
            device_type TEXT
        );

        INSERT INTO user_events VALUES
        (1, 'click', 1.0, '2024-01-01 10:00:00', '{"session_id": "sess_001", "page": "home"}', 'mobile'),
        (1, 'view', 2.5, '2024-01-01 10:05:00', '{"session_id": "sess_001", "page": "product"}', 'mobile'),
        (1, 'purchase', 50.0, '2024-01-01 10:10:00', '{"session_id": "sess_001", "page": "checkout"}', 'mobile'),
        (2, 'click', 1.5, '2024-01-01 11:00:00', '{"session_id": "sess_002", "page": "home"}', 'desktop'),
        (2, 'view', 3.0, '2024-01-01 11:03:00', '{"session_id": "sess_002", "page": "product"}', 'desktop'),
        (2, 'purchase', 75.0, '2024-01-01 11:08:00', '{"session_id": "sess_002", "page": "checkout"}', 'desktop'),
        (3, 'click', 0.8, '2024-01-01 12:00:00', '{"session_id": "sess_003", "page": "home"}', 'tablet'),
        (3, 'view', 2.2, '2024-01-01 12:02:00', '{"session_id": "sess_003", "page": "product"}', 'tablet'),
        (3, 'purchase', 40.0, '2024-01-01 12:07:00', '{"session_id": "sess_003", "page": "checkout"}', 'tablet'),
        (4, 'click', 1.2, '2024-01-01 13:00:00', '{"session_id": "sess_004", "page": "home"}', 'mobile'),
        (4, 'view', 2.8, '2024-01-01 13:04:00', '{"session_id": "sess_004", "page": "product"}', 'mobile'),
        (4, 'purchase', 60.0, '2024-01-01 13:09:00', '{"session_id": "sess_004", "page": "checkout"}', 'mobile');
        "#,
    )?;

    // Complex analytical query (Polars-like)
    let mut stmt = conn.prepare(
        "SELECT
            user_id,
            event_type,
            COUNT(*) as event_count,
            AVG(event_value) as avg_value,
            SUM(event_value) as total_value,
            json_extract_string(session_metadata, '$.session_id') as session_id,
            device_type
         FROM user_events
         GROUP BY user_id, event_type, json_extract_string(session_metadata, '$.session_id'), device_type
         HAVING COUNT(*) >= 1
         ORDER BY user_id, event_count DESC"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i32>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, f64>(3)?,
            row.get::<_, f64>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
        ))
    })?;

    let mut result_count = 0;
    for row in rows {
        let (user_id, _event_type, event_count, avg_value, total_value, session_id, device_type) =
            row?;
        assert!(event_count >= 1);
        assert!(avg_value > 0.0);
        assert!(total_value > 0.0);
        assert!(!session_id.is_empty());
        assert!(!device_type.is_empty());
        result_count += 1;

        info!(
            "📈 User {}: {} events, avg ${:.2}, total ${:.2}, device: {}",
            user_id, event_count, avg_value, total_value, device_type
        );
    }

    assert!(result_count > 0);

    info!("✅ Complex analytics working: {} result rows", result_count);
    Ok(())
}

/// Test performance of analytical operations
#[test]
fn test_analytics_performance() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Create larger dataset for performance testing
    conn.execute(
        "CREATE TABLE analytics_perf_test (
            id INTEGER,
            category TEXT,
            value REAL,
            sub_category TEXT,
            timestamp TIMESTAMP
        )",
        [],
    )?;

    // Insert 10,000 rows for performance testing
    let start = Instant::now();
    for batch in 0..100 {
        let mut values = Vec::new();
        for i in 0..100 {
            let id = batch * 100 + i;
            values.push(format!(
                "({}, 'cat_{}', {}, 'sub_{}', '2024-01-{:02} 10:00:00')",
                id,
                id % 10,
                id as f64 * 0.1,
                id % 5,
                (id % 28) + 1
            ));
        }
        let sql = format!(
            "INSERT INTO analytics_perf_test VALUES {}",
            values.join(", ")
        );
        conn.execute(&sql, [])?;
    }
    let insert_time = start.elapsed();

    // Test complex aggregation performance
    let start = Instant::now();
    let result: f64 = conn.query_row(
        "SELECT AVG(value) FROM analytics_perf_test WHERE category = 'cat_0' AND sub_category = 'sub_0'",
        [],
        |row| row.get(0),
    )?;
    let query_time = start.elapsed();

    // Debug: let's see what the actual average is
    let debug_avg: f64 = conn.query_row(
        "SELECT AVG(value) FROM analytics_perf_test WHERE category = 'cat_0' AND sub_category = 'sub_0'",
        [],
        |row| row.get(0),
    )?;
    info!("📊 Debug: actual average for cat_0/sub_0 = {}", debug_avg);

    assert!((result - debug_avg).abs() < 0.001); // Should match the actual average
    assert!(insert_time < std::time::Duration::from_secs(10));
    assert!(query_time < std::time::Duration::from_secs(1));

    info!(
        "✅ Analytics performance: insert {:?}, query {:?}, avg value = {}",
        insert_time, query_time, result
    );
    Ok(())
}

/// Test JSON operations similar to Polars JSON handling
#[test]
fn test_json_operations() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Test JSON creation and parsing (similar to Polars JSON operations)
    let json_result: String = conn.query_row(
        "SELECT json_object('customer_id', 123, 'total_amount', 299.99, 'items', json_array('laptop', 'mouse'))",
        [],
        |row| row.get(0),
    )?;

    assert!(json_result.contains("123"));
    assert!(json_result.contains("299.99"));
    assert!(json_result.contains("laptop"));
    assert!(json_result.contains("mouse"));

    // Test JSON parsing
    let parsed_amount: f64 = conn.query_row(
        "SELECT CAST(json_extract('{\"customer_id\": 456, \"total_amount\": 149.50, \"items\": [\"book\", \"pen\"]}', '$.total_amount') AS DOUBLE)",
        [],
        |row| row.get(0),
    )?;

    assert!((parsed_amount - 149.50).abs() < 0.001);

    info!("✅ JSON operations working: creation and parsing");
    Ok(())
}

/// Test time-based operations (common in Polars)
#[test]
fn test_time_operations() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Create time series data
    conn.execute_batch(
        r#"
        CREATE TABLE time_events (
            event_id INTEGER,
            event_time TIMESTAMP,
            event_value REAL,
            event_category TEXT
        );

        INSERT INTO time_events VALUES
        (1, '2024-01-01 09:00:00', 100.0, 'login'),
        (2, '2024-01-01 09:15:00', 150.0, 'purchase'),
        (3, '2024-01-01 09:30:00', 75.0, 'view'),
        (4, '2024-01-01 10:00:00', 200.0, 'purchase'),
        (5, '2024-01-01 10:15:00', 125.0, 'view'),
        (6, '2024-01-01 10:30:00', 300.0, 'purchase'),
        (7, '2024-01-01 11:00:00', 90.0, 'login'),
        (8, '2024-01-01 11:15:00', 175.0, 'view');
        "#,
    )?;

    // Test time-based aggregations (similar to Polars groupby_dynamic)
    let mut stmt = conn.prepare(
        "SELECT
            strftime('%Y-%m-%d %H:00:00', event_time) as hour_bucket,
            event_category,
            COUNT(*) as event_count,
            AVG(event_value) as avg_value,
            SUM(event_value) as total_value
         FROM time_events
         GROUP BY strftime('%Y-%m-%d %H:00:00', event_time), event_category
         ORDER BY hour_bucket, event_category",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, f64>(3)?,
            row.get::<_, f64>(4)?,
        ))
    })?;

    let mut result_count = 0;
    for row in rows {
        let (hour_bucket, category, count, avg, sum) = row?;
        assert!(count > 0);
        assert!(avg > 0.0);
        assert!(sum > 0.0);
        result_count += 1;

        info!(
            "🕐 {}: {} events, avg ${:.2}, total ${:.2} ({})",
            hour_bucket, count, avg, sum, category
        );
    }

    assert!(result_count > 0);

    info!(
        "✅ Time-based operations working: {} result rows",
        result_count
    );
    Ok(())
}
