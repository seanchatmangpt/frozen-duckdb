//! Tests for Parquet integration with DuckDB
//!
//! These tests validate Parquet functionality using the Chinook dataset
//! in Parquet format for comprehensive testing.

use anyhow::Result;
use duckdb::Connection;
use std::fs;
use std::time::Instant;
use tempfile::NamedTempFile;
use tracing::info;

/// Test Parquet extension loading and basic functionality
#[test]
fn test_parquet_extension_loaded() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Check if Parquet extension is available
    let result: String = conn.query_row(
        "SELECT extension_name FROM duckdb_extensions() WHERE extension_name = 'parquet'",
        [],
        |row| row.get(0),
    )?;

    assert_eq!(result, "parquet");
    info!("✅ Parquet extension is loaded and available");
    Ok(())
}

/// Test Parquet file creation with Chinook-like dataset
#[test]
fn test_parquet_file_creation() -> Result<()> {
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

    // Create temporary Parquet file
    let temp_file = NamedTempFile::new()?;
    let parquet_path = temp_file.path().to_str().unwrap();

    // Export to Parquet
    conn.execute(
        &format!("COPY track TO '{}' (FORMAT PARQUET)", parquet_path),
        [],
    )?;

    // Verify file was created
    assert!(fs::metadata(parquet_path).is_ok());
    let file_size = fs::metadata(parquet_path)?.len();
    assert!(file_size > 0);

    info!("✅ Parquet file created successfully: {} bytes", file_size);
    Ok(())
}

/// Test Parquet file reading with Chinook-like dataset
#[test]
fn test_parquet_file_reading() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Create test data and export to Parquet
    conn.execute_batch(
        r#"
        CREATE TABLE track (track_id INTEGER, name TEXT, composer TEXT, milliseconds INTEGER, unit_price REAL);

        INSERT INTO track VALUES
        (1, 'For Those About To Rock (We Salute You)', 'Angus Young, Malcolm Young, Brian Johnson', 343719, 0.99),
        (2, 'Put The Finger On You', 'Angus Young, Malcolm Young, Brian Johnson', 205662, 0.99),
        (3, 'Walk This Way', 'Steven Tyler, Joe Perry', 331180, 0.99);
        "#,
    )?;

    let temp_file = NamedTempFile::new()?;
    let parquet_path = temp_file.path().to_str().unwrap();

    conn.execute(
        &format!("COPY track TO '{}' (FORMAT PARQUET)", parquet_path),
        [],
    )?;

    // Read back from Parquet file
    let count: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM read_parquet('{}')", parquet_path),
        [],
        |row| row.get(0),
    )?;

    assert_eq!(count, 3);

    // Verify data integrity
    let sum: f64 = conn.query_row(
        &format!(
            "SELECT SUM(unit_price) FROM read_parquet('{}')",
            parquet_path
        ),
        [],
        |row| row.get(0),
    )?;

    assert!((sum - 2.97).abs() < 0.001); // 0.99 * 3 = 2.97

    info!(
        "✅ Parquet file reading working: {} rows, sum = {}",
        count, sum
    );
    Ok(())
}

/// Test Parquet performance with larger dataset
#[test]
fn test_parquet_performance() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Create larger dataset for performance testing
    conn.execute(
        "CREATE TABLE parquet_perf_test (id INTEGER, name TEXT, value REAL, category TEXT)",
        [],
    )?;

    // Insert 1000 rows
    for i in 0..1000 {
        conn.execute(
            "INSERT INTO parquet_perf_test VALUES (?, ?, ?, ?)",
            duckdb::params![
                i,
                format!("user_{}", i),
                (i as f64) * 1.5,
                format!("cat_{}", i % 10)
            ],
        )?;
    }

    let temp_file = NamedTempFile::new()?;
    let parquet_path = temp_file.path().to_str().unwrap();

    // Measure Parquet export performance
    let start = Instant::now();
    conn.execute(
        &format!(
            "COPY parquet_perf_test TO '{}' (FORMAT PARQUET)",
            parquet_path
        ),
        [],
    )?;
    let export_time = start.elapsed();

    // Measure Parquet import performance
    let start = Instant::now();
    let count: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM read_parquet('{}')", parquet_path),
        [],
        |row| row.get(0),
    )?;
    let import_time = start.elapsed();

    assert_eq!(count, 1000);
    assert!(export_time < std::time::Duration::from_secs(5));
    assert!(import_time < std::time::Duration::from_secs(2));

    info!(
        "✅ Parquet performance: export {:?}, import {:?}, {} rows",
        export_time, import_time, count
    );
    Ok(())
}

/// Test Parquet with JSON data integration
#[test]
fn test_parquet_json_integration() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Create dataset with JSON metadata
    conn.execute(
        "CREATE TABLE parquet_json_test (
            id INTEGER,
            name TEXT,
            value REAL,
            metadata JSON,
            created_at TIMESTAMP
        )",
        [],
    )?;

    // Insert test data
    for i in 0..100 {
        conn.execute(
            "INSERT INTO parquet_json_test VALUES (?, ?, ?, ?, ?)",
            duckdb::params![
                i,
                format!("user_{}", i),
                (i as f64) * 1.5,
                format!("{{\"category\": \"cat_{}\", \"score\": {}}}", i % 5, i * 2),
                format!("2024-01-{:02} 10:00:00", (i % 28) + 1),
            ],
        )?;
    }

    // Export to Parquet
    let temp_file = NamedTempFile::new()?;
    let parquet_path = temp_file.path().to_str().unwrap();

    conn.execute(
        &format!(
            "COPY parquet_json_test TO '{}' (FORMAT PARQUET)",
            parquet_path
        ),
        [],
    )?;

    // Read back from Parquet and filter by JSON
    let count: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM read_parquet('{}') WHERE json_extract_string(metadata, '$.category') = 'cat_0'", parquet_path),
        [],
        |row| row.get(0),
    )?;

    assert_eq!(count, 20); // 100 rows / 5 categories = 20 each

    info!(
        "✅ Parquet-JSON integration working: {} matching rows",
        count
    );
    Ok(())
}

/// Test Parquet with complex analytical queries
#[test]
fn test_parquet_analytics() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Create complex dataset
    conn.execute(
        "CREATE TABLE complex_parquet_analytics (
            user_id INTEGER,
            event_type TEXT,
            value REAL,
            timestamp TIMESTAMP,
            metadata JSON
        )",
        [],
    )?;

    // Insert complex data
    for i in 0..200 {
        let event_type = match i % 3 {
            0 => "click",
            1 => "view",
            _ => "purchase",
        };
        conn.execute(
            "INSERT INTO complex_parquet_analytics VALUES (?, ?, ?, ?, ?)",
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

    let temp_file = NamedTempFile::new()?;
    let parquet_path = temp_file.path().to_str().unwrap();

    conn.execute(
        &format!(
            "COPY complex_parquet_analytics TO '{}' (FORMAT PARQUET)",
            parquet_path
        ),
        [],
    )?;

    // Complex analytical query on Parquet data
    let result: f64 = conn.query_row(
        &format!(
            "SELECT AVG(value) FROM read_parquet('{}') WHERE event_type = 'click' AND json_extract_string(metadata, '$.session_id') LIKE 'sess_%'",
            parquet_path
        ),
        [],
        |row| row.get(0),
    )?;

    assert!(result > 0.0);

    info!(
        "✅ Parquet complex analytics working: avg value = {}",
        result
    );
    Ok(())
}
