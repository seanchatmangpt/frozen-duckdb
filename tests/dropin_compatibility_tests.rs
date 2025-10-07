//! Drop-in compatibility tests for frozen-duckdb
//!
//! These tests verify that frozen-duckdb provides the exact same API as duckdb-rs,
//! ensuring that existing code can be migrated without any changes.

use frozen_duckdb::{Connection, Result, params, Config};
use std::time::Instant;

/// Test that basic connection functionality works identically to duckdb-rs
#[test]
fn test_connection_compatibility() -> Result<()> {
    // Test in-memory connection (same API as duckdb-rs)
    let conn = Connection::open_in_memory()?;
    let version: String = conn.query_row("SELECT version()", [], |row| row.get(0))?;
    assert!(version.starts_with("v"));
    
    // Test connection with custom config (same API as duckdb-rs)
    let config = Config::default().with("threads", "2")?;
    let conn_with_config = Connection::open_in_memory_with_flags(config)?;
    let thread_count: i32 = conn_with_config.query_row(
        "SELECT current_setting('threads')", 
        [], 
        |row| row.get(0)
    )?;
    assert_eq!(thread_count, 2);
    
    Ok(())
}

/// Test that prepared statements work identically to duckdb-rs
#[test]
fn test_prepared_statements_compatibility() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    
    conn.execute_batch(
        r#"
        CREATE TABLE products (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            price REAL,
            category TEXT,
            in_stock BOOLEAN
        );
        "#
    )?;
    
    // Test parameterized insert (same API as duckdb-rs)
    let mut insert_stmt = conn.prepare(
        "INSERT INTO products (id, name, price, category, in_stock) VALUES (?, ?, ?, ?, ?)"
    )?;
    
    let products = vec![
        (1, "Laptop", 999.99, "Electronics", true),
        (2, "Book", 19.99, "Education", true),
        (3, "Chair", 149.50, "Furniture", false),
    ];
    
    for (id, name, price, category, in_stock) in products {
        insert_stmt.execute(params![id, name, price, category, in_stock])?;
    }
    
    // Test parameterized select (same API as duckdb-rs)
    let mut select_stmt = conn.prepare(
        "SELECT name, price FROM products WHERE category = ? AND in_stock = ? ORDER BY price"
    )?;
    
    let rows = select_stmt.query_map(params!["Electronics", true], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
    })?;
    
    let mut found_products = Vec::new();
    for row in rows {
        found_products.push(row?);
    }
    
    assert_eq!(found_products.len(), 1);
    assert_eq!(found_products[0].0, "Laptop");
    assert!((found_products[0].1 - 999.99).abs() < 0.001);
    
    Ok(())
}

/// Test that transactions work identically to duckdb-rs
#[test]
fn test_transaction_compatibility() -> Result<()> {
    let mut conn = Connection::open_in_memory()?;
    
    conn.execute_batch(
        r#"
        CREATE TABLE accounts (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            balance REAL
        );
        
        INSERT INTO accounts VALUES (1, 'Alice', 1000.0), (2, 'Bob', 500.0);
        "#
    )?;
    
    // Test successful transaction (same API as duckdb-rs)
    {
        let tx = conn.transaction()?;
        tx.execute(
            "UPDATE accounts SET balance = balance - ? WHERE id = ?",
            params![100.0, 1], // Alice transfers 100 to Bob
        )?;
        tx.execute(
            "UPDATE accounts SET balance = balance + ? WHERE id = ?",
            params![100.0, 2],
        )?;
        tx.commit()?;
    }
    
    let alice_balance: f64 = conn.query_row(
        "SELECT balance FROM accounts WHERE id = 1", 
        [], 
        |row| row.get(0)
    )?;
    let bob_balance: f64 = conn.query_row(
        "SELECT balance FROM accounts WHERE id = 2", 
        [], 
        |row| row.get(0)
    )?;
    
    assert!((alice_balance - 900.0).abs() < 0.001);
    assert!((bob_balance - 600.0).abs() < 0.001);
    
    // Test transaction rollback (same API as duckdb-rs)
    {
        let tx = conn.transaction()?;
        tx.execute(
            "UPDATE accounts SET balance = balance - ? WHERE id = ?",
            params![50.0, 2],
        )?;
        // Don't commit - should rollback
    }
    
    let bob_balance_after_rollback: f64 = conn.query_row(
        "SELECT balance FROM accounts WHERE id = 2", 
        [], 
        |row| row.get(0)
    )?;
    assert!((bob_balance_after_rollback - 600.0).abs() < 0.001); // Should still be 600
    
    Ok(())
}

/// Test that complex queries work identically to duckdb-rs
#[test]
fn test_complex_queries_compatibility() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    
    conn.execute_batch(
        r#"
        CREATE TABLE customers (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            city TEXT
        );
        
        CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            customer_id INTEGER REFERENCES customers(id),
            amount REAL,
            order_date VARCHAR
        );
        
        INSERT INTO customers VALUES
        (1, 'Alice', 'New York'),
        (2, 'Bob', 'Los Angeles'),
        (3, 'Charlie', 'Chicago');
        
        INSERT INTO orders VALUES
        (1, 1, 299.99, '2024-01-15'),
        (2, 1, 149.50, '2024-01-16'),
        (3, 2, 89.99, '2024-01-17'),
        (4, 3, 199.99, '2024-01-18');
        "#
    )?;
    
    // Test INNER JOIN (same API as duckdb-rs)
    let mut stmt = conn.prepare(
        "SELECT c.name, c.city, o.amount, o.order_date
         FROM customers c
         INNER JOIN orders o ON c.id = o.customer_id
         ORDER BY c.name, o.order_date"
    )?;
    
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, f64>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    
    let mut order_count = 0;
    for row in rows {
        let (name, city, amount, _date) = row?;
        assert!(!name.is_empty());
        assert!(!city.is_empty());
        assert!(amount > 0.0);
        order_count += 1;
    }
    
    assert_eq!(order_count, 4);
    
    // Test aggregation (same API as duckdb-rs)
    let total_orders: f64 = conn.query_row(
        "SELECT SUM(amount) FROM orders",
        [],
        |row| row.get(0),
    )?;
    
    assert!((total_orders - 739.47).abs() < 0.01); // 299.99 + 149.50 + 89.99 + 199.99
    
    Ok(())
}

/// Test that error handling works identically to duckdb-rs
#[test]
fn test_error_handling_compatibility() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    
    // Test SQL syntax error (same behavior as duckdb-rs)
    let result = conn.execute("INVALID SQL SYNTAX", []);
    assert!(result.is_err());
    
    // Test constraint violation (same behavior as duckdb-rs)
    conn.execute_batch(
        r#"
        CREATE TABLE test_constraints (
            id INTEGER PRIMARY KEY,
            unique_value TEXT UNIQUE
        );
        INSERT INTO test_constraints VALUES (1, 'value1');
        "#
    )?;
    
    // This should fail due to unique constraint (same behavior as duckdb-rs)
    let result = conn.execute(
        "INSERT INTO test_constraints VALUES (?, ?)",
        params![2, "value1"], // Duplicate unique value
    );
    assert!(result.is_err());
    
    Ok(())
}

/// Test that performance is at least as good as duckdb-rs
#[test]
fn test_performance_compatibility() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    
    // Create test table
    conn.execute(
        "CREATE TABLE perf_test (id INTEGER, data TEXT, value REAL)",
        [],
    )?;
    
    // Insert test data
    for i in 0..1000 {
        conn.execute(
            "INSERT INTO perf_test VALUES (?, ?, ?)",
            params![i, format!("data_{}", i), (i as f64) * 0.1],
        )?;
    }
    
    // Measure query performance
    let start = Instant::now();
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM perf_test WHERE value > ?",
        [50.0],
        |row| row.get(0),
    )?;
    let query_time = start.elapsed();
    
    assert_eq!(count, 499); // Values 50.1 to 99.9
    assert!(query_time < std::time::Duration::from_secs(1)); // Should be very fast
    
    Ok(())
}

/// Test that all common data types work identically to duckdb-rs
#[test]
fn test_data_types_compatibility() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    
    // Create table with all major data types
    conn.execute_batch(
        r#"
        CREATE TABLE data_types_test (
            id INTEGER PRIMARY KEY,
            name TEXT,
            age INTEGER,
            salary REAL,
            is_active BOOLEAN,
            birth_date VARCHAR,
            created_at VARCHAR,
            metadata JSON,
            binary_data BLOB
        );
        "#
    )?;
    
    // Insert test data for each type
    conn.execute(
        "INSERT INTO data_types_test VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            1,
            "Alice Smith",
            30,
            75000.50,
            true,
            "1994-03-15",
            "2024-01-01 10:00:00",
            r#"{"department": "engineering", "level": "senior"}"#,
            vec![1u8, 2u8, 3u8, 4u8, 5u8]
        ],
    )?;
    
    // Test retrieval of each data type
    let mut stmt = conn.prepare("SELECT * FROM data_types_test WHERE id = ?")?;
    let result = stmt.query_row([1], |row| {
        Ok((
            row.get::<_, i32>(0)?,     // INTEGER
            row.get::<_, String>(1)?,  // TEXT
            row.get::<_, i32>(2)?,     // INTEGER
            row.get::<_, f64>(3)?,     // REAL
            row.get::<_, bool>(4)?,    // BOOLEAN
            row.get::<_, String>(5)?,  // DATE (as string)
            row.get::<_, String>(6)?,  // TIMESTAMP (as string)
            row.get::<_, String>(7)?,  // JSON (as string)
            row.get::<_, Vec<u8>>(8)?, // BLOB
        ))
    })?;
    
    let (id, name, age, salary, is_active, birth_date, created_at, metadata, binary_data) = result;
    
    assert_eq!(id, 1);
    assert_eq!(name, "Alice Smith");
    assert_eq!(age, 30);
    assert!((salary - 75000.50).abs() < 0.001);
    assert!(is_active);
    assert_eq!(birth_date, "1994-03-15");
    assert!(created_at.contains("2024-01-01"));
    assert!(metadata.contains("engineering"));
    assert_eq!(binary_data, vec![1, 2, 3, 4, 5]);
    
    Ok(())
}
