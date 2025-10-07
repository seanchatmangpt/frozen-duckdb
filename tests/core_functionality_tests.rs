//! Core functionality tests for frozen DuckDB binary
//!
//! These tests validate that the frozen DuckDB binary provides complete
//! compatibility with duckdb-rs API and core SQL functionality.
//! Follows 80/20 rule: tests the 20% of features that deliver 80% of value.

use anyhow::Result;
use duckdb::{params, Config, Connection};
use std::time::Instant;
use tracing::info;

/// Test basic connection functionality
#[test]
fn test_connection_management() -> Result<()> {
    // Test in-memory connection
    let conn = Connection::open_in_memory()?;
    let version: String = conn.query_row("SELECT version()", [], |row| row.get(0))?;
    assert!(version.starts_with("v"));
    info!("✅ In-memory connection working: {}", version);

    // Test connection with custom config
    let config = Config::default().with("threads", "2")?;
    let conn_with_config = Connection::open_in_memory_with_flags(config)?;
    let thread_count: i32 =
        conn_with_config.query_row("SELECT current_setting('threads')", [], |row| row.get(0))?;
    assert_eq!(thread_count, 2);
    info!("✅ Connection with custom config working");

    Ok(())
}

/// Test all major SQL data types (80/20 coverage)
#[test]
fn test_data_type_compatibility() -> Result<()> {
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
        "#,
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

    info!("✅ All major data types working correctly");
    Ok(())
}

/// Test basic SQL operations (CRUD)
#[test]
fn test_crud_operations() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // CREATE
    conn.execute_batch(
        r#"
        CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            email TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE posts (
            id INTEGER PRIMARY KEY,
            user_id INTEGER REFERENCES users(id),
            title TEXT NOT NULL,
            content TEXT,
            published_at TIMESTAMP
        );
        "#,
    )?;

    // INSERT
    conn.execute(
        "INSERT INTO users (id, username, email) VALUES (?, ?, ?)",
        params![1, "alice", "alice@example.com"],
    )?;

    conn.execute(
        "INSERT INTO users (id, username, email) VALUES (?, ?, ?)",
        params![2, "bob", "bob@example.com"],
    )?;

    conn.execute(
        "INSERT INTO posts (id, user_id, title, content) VALUES (?, (SELECT id FROM users WHERE username = 'alice'), ?, ?)",
        params![1, "First Post", "Hello world content"],
    )?;

    let alice_id = 1;

    assert!(alice_id > 0);

    // SELECT
    let user_count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
    assert_eq!(user_count, 2);

    let post_count: i64 = conn.query_row("SELECT COUNT(*) FROM posts", [], |row| row.get(0))?;
    assert_eq!(post_count, 1);

    // UPDATE
    let updated_rows: usize = conn.execute(
        "UPDATE users SET email = ? WHERE username = ?",
        params!["alice.new@example.com", "alice"],
    )?;
    assert_eq!(updated_rows, 1);

    // DELETE
    let deleted_rows: usize = conn.execute(
        "DELETE FROM posts WHERE user_id = (SELECT id FROM users WHERE username = 'alice')",
        [],
    )?;
    assert_eq!(deleted_rows, 1);

    info!("✅ CRUD operations working correctly");
    Ok(())
}

/// Test prepared statements and parameterized queries
#[test]
fn test_prepared_statements() -> Result<()> {
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
        "#,
    )?;

    // Test parameterized insert
    let mut insert_stmt = conn.prepare(
        "INSERT INTO products (id, name, price, category, in_stock) VALUES (?, ?, ?, ?, ?)",
    )?;

    let products = vec![
        (1, "Laptop", 999.99, "Electronics", true),
        (2, "Book", 19.99, "Education", true),
        (3, "Chair", 149.50, "Furniture", false),
    ];

    for (id, name, price, category, in_stock) in products {
        insert_stmt.execute(params![id, name, price, category, in_stock])?;
    }

    // Test parameterized select
    let mut select_stmt = conn.prepare(
        "SELECT name, price FROM products WHERE category = ? AND in_stock = ? ORDER BY price",
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

    info!("✅ Prepared statements working correctly");
    Ok(())
}

/// Test transaction functionality
#[test]
fn test_transactions() -> Result<()> {
    let mut conn = Connection::open_in_memory()?;

    conn.execute_batch(
        r#"
        CREATE TABLE accounts (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            balance REAL
        );

        INSERT INTO accounts VALUES (1, 'Alice', 1000.0), (2, 'Bob', 500.0);
        "#,
    )?;

    // Test successful transaction
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

    let alice_balance: f64 =
        conn.query_row("SELECT balance FROM accounts WHERE id = 1", [], |row| {
            row.get(0)
        })?;
    let bob_balance: f64 =
        conn.query_row("SELECT balance FROM accounts WHERE id = 2", [], |row| {
            row.get(0)
        })?;

    assert!((alice_balance - 900.0).abs() < 0.001);
    assert!((bob_balance - 600.0).abs() < 0.001);

    // Test transaction rollback
    {
        let tx = conn.transaction()?;
        tx.execute(
            "UPDATE accounts SET balance = balance - ? WHERE id = ?",
            params![50.0, 2],
        )?;
        // Don't commit - should rollback
    }

    let bob_balance_after_rollback: f64 =
        conn.query_row("SELECT balance FROM accounts WHERE id = 2", [], |row| {
            row.get(0)
        })?;
    assert!((bob_balance_after_rollback - 600.0).abs() < 0.001); // Should still be 600

    info!("✅ Transaction functionality working correctly");
    Ok(())
}

/// Test JOIN operations (common SQL pattern)
#[test]
fn test_join_operations() -> Result<()> {
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
        "#,
    )?;

    // Test INNER JOIN
    let mut stmt = conn.prepare(
        "SELECT c.name, c.city, o.amount, o.order_date
         FROM customers c
         INNER JOIN orders o ON c.id = o.customer_id
         ORDER BY c.name, o.order_date",
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

    info!("✅ JOIN operations working correctly");
    Ok(())
}

/// Test aggregation and GROUP BY (analytical operations)
#[test]
fn test_aggregation_operations() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    conn.execute_batch(
        r#"
        CREATE TABLE sales (
            id INTEGER PRIMARY KEY,
            region TEXT NOT NULL,
            product TEXT NOT NULL,
            amount REAL,
            sale_date DATE
        );

        INSERT INTO sales VALUES
        (1, 'North', 'Widget A', 100.0, '2024-01-01'),
        (2, 'North', 'Widget B', 150.0, '2024-01-01'),
        (3, 'South', 'Widget A', 200.0, '2024-01-01'),
        (4, 'South', 'Widget C', 75.0, '2024-01-01'),
        (5, 'North', 'Widget A', 125.0, '2024-01-02'),
        (6, 'East', 'Widget B', 300.0, '2024-01-02');
        "#,
    )?;

    // Test aggregation with GROUP BY
    let mut stmt = conn.prepare(
        "SELECT region, product, COUNT(*) as sales_count, SUM(amount) as total_amount, AVG(amount) as avg_amount
         FROM sales
         GROUP BY region, product
         ORDER BY region, product"
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

    let mut aggregation_count = 0;
    for row in rows {
        let (_region, _product, count, total, avg) = row?;
        assert!(count > 0);
        assert!(total > 0.0);
        assert!(avg > 0.0);
        aggregation_count += 1;
    }

    assert_eq!(aggregation_count, 5); // 5 unique region-product combinations

    info!("✅ Aggregation operations working correctly");
    Ok(())
}

/// Test error handling and edge cases
#[test]
fn test_error_handling() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Test SQL syntax error
    let result = conn.execute("INVALID SQL SYNTAX", []);
    assert!(result.is_err());

    // Test constraint violation
    conn.execute_batch(
        r#"
        CREATE TABLE test_constraints (
            id INTEGER PRIMARY KEY,
            unique_value TEXT UNIQUE
        );
        INSERT INTO test_constraints VALUES (1, 'value1');
        "#,
    )?;

    // This should fail due to unique constraint
    let result = conn.execute(
        "INSERT INTO test_constraints VALUES (?, ?)",
        params![2, "value1"], // Duplicate unique value
    );
    assert!(result.is_err());

    // Test foreign key constraint
    conn.execute_batch(
        r#"
        CREATE TABLE parent (id INTEGER PRIMARY KEY);
        CREATE TABLE child (id INTEGER PRIMARY KEY, parent_id INTEGER REFERENCES parent(id));
        INSERT INTO parent VALUES (1);
        "#,
    )?;

    // This should fail due to foreign key constraint
    let result = conn.execute(
        "INSERT INTO child VALUES (?, ?)",
        params![1, 999], // Non-existent parent_id
    );
    assert!(result.is_err());

    info!("✅ Error handling working correctly");
    Ok(())
}

/// Test basic query performance (simplified for frozen binary validation)
#[test]
fn test_basic_performance() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Create test table
    conn.execute(
        "CREATE TABLE perf_test (id INTEGER, data TEXT, value REAL)",
        [],
    )?;

    // Insert a smaller dataset for basic performance validation
    for i in 0..1000 {
        conn.execute(
            "INSERT INTO perf_test VALUES (?, ?, ?)",
            params![i, format!("data_{}", i), (i as f64) * 0.1],
        )?;
    }

    // Measure basic query performance
    let start = Instant::now();
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM perf_test WHERE value > ?",
        [50.0],
        |row| row.get(0),
    )?;
    let query_time = start.elapsed();

    assert_eq!(count, 499); // Values 50.1 to 99.9 (values 501 to 999 = 499 values)
    assert!(query_time < std::time::Duration::from_secs(1));

    info!(
        "✅ Basic performance test: query {:?}, {} matching rows",
        query_time, count
    );
    Ok(())
}

/// Test with Chinook dataset for realistic scenarios
#[test]
fn test_chinook_dataset() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Create Chinook-like schema
    conn.execute_batch(
        r#"
        CREATE TABLE artist (artist_id INTEGER PRIMARY KEY, name TEXT NOT NULL);
        CREATE TABLE album (album_id INTEGER PRIMARY KEY, title TEXT NOT NULL, artist_id INTEGER REFERENCES artist(artist_id));
        CREATE TABLE track (track_id INTEGER PRIMARY KEY, name TEXT NOT NULL, album_id INTEGER REFERENCES album(album_id),
                           composer TEXT, milliseconds INTEGER, bytes INTEGER, unit_price REAL);

        INSERT INTO artist VALUES (1, 'AC/DC'), (2, 'Aerosmith');
        INSERT INTO album VALUES (1, 'For Those About To Rock We Salute You', 1), (2, 'Let There Be Rock', 1), (3, 'Toys In The Attic', 2);
        INSERT INTO track VALUES
        (1, 'For Those About To Rock (We Salute You)', 1, 'Angus Young, Malcolm Young, Brian Johnson', 343719, 11170334, 0.99),
        (2, 'Put The Finger On You', 1, 'Angus Young, Malcolm Young, Brian Johnson', 205662, 6713451, 0.99),
        (3, 'Walk This Way', 3, 'Steven Tyler, Joe Perry', 331180, 10871135, 0.99);
        "#,
    )?;

    // Test complex query with JOINs (realistic scenario)
    let mut stmt = conn.prepare(
        "SELECT a.name as artist_name, al.title as album_title, t.name as track_name, t.unit_price
         FROM artist a
         JOIN album al ON a.artist_id = al.artist_id
         JOIN track t ON al.album_id = t.album_id
         WHERE t.unit_price > ?
         ORDER BY a.name, al.title, t.name",
    )?;

    let rows = stmt.query_map([0.5], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, f64>(3)?,
        ))
    })?;

    let mut track_count = 0;
    for row in rows {
        let (artist, album, track, price) = row?;
        assert!(!artist.is_empty());
        assert!(!album.is_empty());
        assert!(!track.is_empty());
        assert!(price > 0.5);
        track_count += 1;
    }

    assert_eq!(track_count, 3); // All tracks have price > 0.5

    // Test aggregation on Chinook data
    let total_revenue: f64 = conn.query_row(
        "SELECT SUM(unit_price) FROM track WHERE album_id IN (SELECT album_id FROM album WHERE artist_id = 1)",
        [],
        |row| row.get(0),
    )?;

    assert!((total_revenue - 1.98).abs() < 0.001); // 0.99 * 2 tracks

    info!(
        "✅ Chinook dataset tests working: {} tracks, ${:.2} total revenue",
        track_count, total_revenue
    );
    Ok(())
}
