//! Basic usage example for frozen DuckDB binary
//!
//! This example demonstrates how to use the frozen DuckDB binary
//! for fast builds without compilation overhead.

use anyhow::Result;
use duckdb::Connection;
use tracing::info;

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🚀 Starting frozen DuckDB example");

    // Create an in-memory database connection
    let conn = Connection::open_in_memory()?;
    info!("✅ Connected to DuckDB");

    // Create a simple table
    conn.execute("CREATE TABLE users (id INTEGER, name TEXT, email TEXT)", [])?;
    info!("✅ Created users table");

    // Insert some sample data
    conn.execute(
        "INSERT INTO users VALUES (1, 'Alice', 'alice@example.com'), (2, 'Bob', 'bob@example.com')",
        [],
    )?;
    info!("✅ Inserted sample data");

    // Query the data
    let mut stmt = conn.prepare("SELECT id, name, email FROM users ORDER BY id")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i32>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;

    info!("📊 Query results:");
    for row in rows {
        let (id, name, email) = row?;
        println!("  ID: {}, Name: {}, Email: {}", id, name, email);
    }

    // Demonstrate some DuckDB features
    let result: i64 = conn.query_row(
        "SELECT COUNT(*) FROM users WHERE name LIKE 'A%'",
        [],
        |row| row.get(0),
    )?;
    info!("🔍 Users with names starting with 'A': {}", result);

    // Show performance metrics
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
    }
    let duration = start.elapsed();
    info!(
        "⚡ Executed 1000 queries in {:?} ({:.2} queries/sec)",
        duration,
        1000.0 / duration.as_secs_f64()
    );

    info!("🎉 Frozen DuckDB example completed successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() -> Result<()> {
        let conn = Connection::open_in_memory()?;

        // Test table creation
        conn.execute("CREATE TABLE test (id INTEGER, value TEXT)", [])?;

        // Test data insertion
        conn.execute("INSERT INTO test VALUES (1, 'hello'), (2, 'world')", [])?;

        // Test data retrieval
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM test", [], |row| row.get(0))?;
        assert_eq!(count, 2);

        Ok(())
    }

    #[test]
    fn test_performance() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        conn.execute("CREATE TABLE perf_test (id INTEGER, data TEXT)", [])?;

        // Insert test data
        for i in 0..1000 {
            conn.execute(
                "INSERT INTO perf_test VALUES (?, ?)",
                [i, &format!("data_{}", i)],
            )?;
        }

        // Measure query performance
        let start = std::time::Instant::now();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM perf_test", [], |row| row.get(0))?;
        let duration = start.elapsed();

        assert_eq!(count, 1000);
        assert!(
            duration.as_millis() < 100,
            "Query took too long: {:?}",
            duration
        );

        Ok(())
    }
}
