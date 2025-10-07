//! Test crate to demonstrate frozen-duckdb dependency behavior
//!
//! This crate depends on frozen-duckdb and uses DuckDB functionality.
//! It simulates the issue where DuckDB gets compiled every time
//! when using frozen-duckdb as a dependency.

use anyhow::Result;
use duckdb::{Connection, Result as DuckResult};
use tracing::{info, warn};

fn main() -> Result<()> {
    // Initialize tracing for better logging
    tracing_subscriber::fmt::init();

    info!("🚀 Starting test-dependency application...");

    // Check if frozen-duckdb environment is configured
    match (
        std::env::var("DUCKDB_LIB_DIR"),
        std::env::var("DUCKDB_INCLUDE_DIR")
    ) {
        (Ok(lib_dir), Ok(include_dir)) => {
            info!("✅ Frozen DuckDB environment configured:");
            info!("   Library: {}", lib_dir);
            info!("   Headers: {}", include_dir);
            info!("   This means DuckDB should use pre-compiled binaries!");
        }
        _ => {
            warn!("❌ Frozen DuckDB environment not configured");
            warn!("   DUCKDB_LIB_DIR: {:?}", std::env::var("DUCKDB_LIB_DIR"));
            warn!("   DUCKDB_INCLUDE_DIR: {:?}", std::env::var("DUCKDB_INCLUDE_DIR"));
            warn!("   This means DuckDB compilation may occur");
        }
    }

    // Test DuckDB connection and basic operations
    test_duckdb_operations()?;

    info!("✅ Test completed successfully!");
    Ok(())
}

/// Test basic DuckDB operations to trigger any compilation issues
fn test_duckdb_operations() -> Result<()> {
    info!("🔍 Testing DuckDB operations...");

    // Create in-memory database
    let conn = Connection::open_in_memory()?;

    // Test basic operations that would trigger compilation if DuckDB isn't pre-compiled
    conn.execute_batch(
        r#"
        CREATE TABLE test_table (
            id INTEGER PRIMARY KEY,
            name TEXT,
            value REAL
        );

        INSERT INTO test_table VALUES
        (1, 'test1', 100.5),
        (2, 'test2', 200.7),
        (3, 'test3', 300.9);
        "#,
    )?;

    // Test query operations
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM test_table", [], |row| row.get(0))?;
    assert_eq!(count, 3);

    let names: Vec<String> = conn.prepare("SELECT name FROM test_table ORDER BY name")?
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;

    assert_eq!(names, vec!["test1", "test2", "test3"]);

    info!("✅ DuckDB operations completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duckdb_integration() -> Result<()> {
        test_duckdb_operations()
    }
}
