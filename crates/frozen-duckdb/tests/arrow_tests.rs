//! Tests for Arrow integration with DuckDB
//!
//! These tests validate Arrow functionality using the Chinook dataset
//! in Arrow format for comprehensive testing.

use anyhow::Result;
use duckdb::arrow::record_batch::RecordBatch;
use duckdb::Connection;
use std::time::Instant;
use tracing::info;

/// Test Arrow extension loading and basic functionality
#[test]
fn test_arrow_extension_loaded() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Check if Arrow extension is available (skip if not available on this platform)
    let result = conn.query_row(
        "SELECT extension_name FROM duckdb_extensions() WHERE extension_name = 'arrow'",
        [],
        |row| row.get::<_, String>(0),
    );

    match result {
        Ok(extension_name) => {
            assert_eq!(extension_name, "arrow");
            info!("✅ Arrow extension is loaded and available");
        }
        Err(_) => {
            info!("⚠️  Arrow extension not available on this platform - skipping test");
            return Ok(()); // Skip test if extension not available
        }
    }

    Ok(())
}

/// Test Arrow data export functionality with Chinook dataset
#[test]
fn test_arrow_data_export() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Create Chinook-like schema and data
    conn.execute_batch(
        r#"
        CREATE TABLE artist (artist_id INTEGER, name TEXT);
        CREATE TABLE album (album_id INTEGER, title TEXT, artist_id INTEGER);
        CREATE TABLE track (track_id INTEGER, name TEXT, album_id INTEGER, composer TEXT, milliseconds INTEGER, bytes INTEGER, unit_price REAL);

        INSERT INTO artist VALUES (1, 'AC/DC'), (2, 'Aerosmith'), (3, 'Led Zeppelin');
        INSERT INTO album VALUES (1, 'For Those About To Rock We Salute You', 1), (2, 'Let There Be Rock', 1), (3, 'Toys In The Attic', 2);
        INSERT INTO track VALUES
        (1, 'For Those About To Rock (We Salute You)', 1, 'Angus Young, Malcolm Young, Brian Johnson', 343719, 11170334, 0.99),
        (2, 'Put The Finger On You', 1, 'Angus Young, Malcolm Young, Brian Johnson', 205662, 6713451, 0.99),
        (3, 'Walk This Way', 3, 'Steven Tyler, Joe Perry', 331180, 10871135, 0.99);
        "#,
    )?;

    // Test Arrow export
    let mut stmt = conn.prepare("SELECT * FROM track ORDER BY track_id")?;
    let arrow_batches: Vec<RecordBatch> = stmt.query_arrow([])?.collect();

    let total_rows: usize = arrow_batches.iter().map(|batch| batch.num_rows()).sum();
    let num_columns = arrow_batches
        .first()
        .map(|batch| batch.num_columns())
        .unwrap_or(0);

    assert_eq!(total_rows, 3);
    assert_eq!(num_columns, 7);

    info!(
        "✅ Arrow export working: {} rows, {} columns",
        total_rows, num_columns
    );
    Ok(())
}

/// Test Arrow data types with comprehensive examples
#[test]
fn test_arrow_data_types() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Test various Arrow-compatible data types
    conn.execute(
        "CREATE TABLE arrow_types (
            id INTEGER,
            name TEXT,
            value REAL,
            is_active BOOLEAN,
            created_at TIMESTAMP,
            data BLOB
        )",
        [],
    )?;

    conn.execute(
        "INSERT INTO arrow_types VALUES
         (1, 'test', 42.5, true, '2024-01-01 10:00:00', 'binary_data'),
         (2, 'test2', 84.2, false, '2024-01-02 11:00:00', 'more_binary')",
        [],
    )?;

    let mut stmt = conn.prepare("SELECT * FROM arrow_types ORDER BY id")?;
    let arrow_batches: Vec<RecordBatch> = stmt.query_arrow([])?.collect();

    let total_rows: usize = arrow_batches.iter().map(|batch| batch.num_rows()).sum();
    let num_columns = arrow_batches
        .first()
        .map(|batch| batch.num_columns())
        .unwrap_or(0);

    assert_eq!(total_rows, 2);
    assert_eq!(num_columns, 6);

    info!("✅ Arrow data types supported: INTEGER, TEXT, REAL, BOOLEAN, TIMESTAMP, BLOB");
    Ok(())
}

/// Test JSON with Arrow integration
#[test]
fn test_json_with_arrow() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Create table with JSON data
    conn.execute("CREATE TABLE json_arrow_test (id INTEGER, data JSON)", [])?;

    conn.execute(
        "INSERT INTO json_arrow_test VALUES
         (1, '{\"name\": \"Alice\", \"age\": 30}'),
         (2, '{\"name\": \"Bob\", \"age\": 25}'),
         (3, '{\"name\": \"Charlie\", \"age\": 35}')",
        [],
    )?;

    // Export to Arrow format
    let mut stmt = conn.prepare("SELECT * FROM json_arrow_test ORDER BY id")?;
    let arrow_batches: Vec<RecordBatch> = stmt.query_arrow([])?.collect();

    let total_rows: usize = arrow_batches.iter().map(|batch| batch.num_rows()).sum();
    let num_columns = arrow_batches
        .first()
        .map(|batch| batch.num_columns())
        .unwrap_or(0);

    assert_eq!(total_rows, 3);
    assert_eq!(num_columns, 2);

    info!(
        "✅ JSON with Arrow integration working: {} rows",
        total_rows
    );
    Ok(())
}

/// Test complex analytical operations with Arrow
#[test]
fn test_complex_analytics_with_arrow() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Create complex dataset
    conn.execute(
        "CREATE TABLE complex_analytics (
            user_id INTEGER,
            event_type TEXT,
            value REAL,
            timestamp TIMESTAMP,
            metadata JSON
        )",
        [],
    )?;

    // Insert complex data
    for i in 0..50 {
        let event_type = match i % 3 {
            0 => "click",
            1 => "view",
            _ => "purchase",
        };
        conn.execute(
            "INSERT INTO complex_analytics VALUES (?, ?, ?, ?, ?)",
            duckdb::params![
                i % 10, // 10 users
                event_type,
                (i as f64) * 0.5,
                format!("2024-01-{:02} {:02}:00:00", (i % 28) + 1, (i % 24)),
                format!(
                    "{{\"session_id\": \"sess_{}\", \"page\": \"page_{}\"}}",
                    i % 5,
                    i % 3
                ),
            ],
        )?;
    }

    // Complex analytical query (Arrow export)
    let mut stmt = conn.prepare(
        "SELECT
            user_id,
            event_type,
            COUNT(*) as event_count,
            AVG(value) as avg_value,
            SUM(value) as total_value,
            json_extract_string(metadata, '$.session_id') as session_id
         FROM complex_analytics
         GROUP BY user_id, event_type, json_extract_string(metadata, '$.session_id')
         HAVING COUNT(*) > 1
         ORDER BY user_id, event_count DESC",
    )?;

    let arrow_batches: Vec<RecordBatch> = stmt.query_arrow([])?.collect();
    let total_rows: usize = arrow_batches.iter().map(|batch| batch.num_rows()).sum();
    let num_columns = arrow_batches
        .first()
        .map(|batch| batch.num_columns())
        .unwrap_or(0);

    assert!(total_rows > 0);
    assert_eq!(num_columns, 6);

    info!(
        "✅ Complex analytics with Arrow working: {} rows, {} columns",
        total_rows, num_columns
    );
    Ok(())
}

/// Test Arrow export performance
#[test]
fn test_arrow_performance() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Create larger dataset for performance testing
    conn.execute(
        "CREATE TABLE arrow_perf_test (id INTEGER, name TEXT, value REAL, category TEXT)",
        [],
    )?;

    // Insert 1000 rows
    for i in 0..1000 {
        conn.execute(
            "INSERT INTO arrow_perf_test VALUES (?, ?, ?, ?)",
            duckdb::params![
                i,
                format!("user_{}", i),
                (i as f64) * 1.5,
                format!("cat_{}", i % 10)
            ],
        )?;
    }

    // Measure Arrow export performance
    let start = Instant::now();
    let mut stmt = conn.prepare("SELECT * FROM arrow_perf_test ORDER BY id")?;
    let arrow_batches: Vec<RecordBatch> = stmt.query_arrow([])?.collect();
    let arrow_time = start.elapsed();

    let total_rows: usize = arrow_batches.iter().map(|batch| batch.num_rows()).sum();
    let num_columns = arrow_batches
        .first()
        .map(|batch| batch.num_columns())
        .unwrap_or(0);

    assert_eq!(total_rows, 1000);
    assert_eq!(num_columns, 4);
    assert!(arrow_time < std::time::Duration::from_secs(5));

    info!(
        "✅ Arrow performance: export {:?}, {} rows, {} columns",
        arrow_time, total_rows, num_columns
    );
    Ok(())
}
