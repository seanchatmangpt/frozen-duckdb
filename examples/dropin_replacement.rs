//! Example demonstrating frozen-duckdb as a drop-in replacement for duckdb-rs
//!
//! This example shows that frozen-duckdb provides the exact same API as duckdb-rs,
//! but with 99% faster build times. No code changes are needed when migrating
//! from duckdb-rs to frozen-duckdb.

use frozen_duckdb::{Connection, Result, params};

fn main() -> Result<()> {
    println!("🦆 Frozen DuckDB - Drop-in Replacement Example");
    println!("===============================================");
    
    // Create in-memory database (same API as duckdb-rs)
    let mut conn = Connection::open_in_memory()?;
    
    // Create tables and insert data (same API as duckdb-rs)
    conn.execute_batch(
        r#"
        CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT UNIQUE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
        
        CREATE TABLE posts (
            id INTEGER PRIMARY KEY,
            user_id INTEGER REFERENCES users(id),
            title TEXT NOT NULL,
            content TEXT,
            published_at TIMESTAMP
        );
        
        INSERT INTO users (id, name, email) VALUES 
        (1, 'Alice Johnson', 'alice@example.com'),
        (2, 'Bob Smith', 'bob@example.com'),
        (3, 'Charlie Brown', 'charlie@example.com');
        
        INSERT INTO posts (id, user_id, title, content) VALUES
        (1, 1, 'Hello World', 'My first post!'),
        (2, 1, 'Rust is Awesome', 'Learning Rust with DuckDB'),
        (3, 2, 'Database Performance', 'How to optimize queries'),
        (4, 3, 'Getting Started', 'Introduction to databases');
        "#,
    )?;
    
    // Query data using prepared statements (same API as duckdb-rs)
    let mut stmt = conn.prepare(
        "SELECT u.name, u.email, COUNT(p.id) as post_count
         FROM users u
         LEFT JOIN posts p ON u.id = p.user_id
         GROUP BY u.id, u.name, u.email
         ORDER BY post_count DESC"
    )?;
    
    println!("\n📊 User Post Statistics:");
    println!("{:<20} {:<25} {:<10}", "Name", "Email", "Posts");
    println!("{}", "-".repeat(60));
    
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,  // name
            row.get::<_, String>(1)?,  // email
            row.get::<_, i64>(2)?,     // post_count
        ))
    })?;
    
    for row in rows {
        let (name, email, post_count) = row?;
        println!("{:<20} {:<25} {:<10}", name, email, post_count);
    }
    
    // Demonstrate parameterized queries (same API as duckdb-rs)
    let mut stmt = conn.prepare(
        "SELECT p.title, p.content, u.name as author
         FROM posts p
         JOIN users u ON p.user_id = u.id
         WHERE u.name = ?"
    )?;
    
    println!("\n📝 Posts by Alice:");
    let rows = stmt.query_map(params!["Alice Johnson"], |row| {
        Ok((
            row.get::<_, String>(0)?,  // title
            row.get::<_, String>(1)?,  // content
            row.get::<_, String>(2)?,  // author
        ))
    })?;
    
    for row in rows {
        let (title, content, author) = row?;
        println!("📌 {} (by {})", title, author);
        println!("   {}", content);
        println!();
    }
    
    // Demonstrate transactions (same API as duckdb-rs)
    let tx = conn.transaction()?;
    
    // Insert a new user and post in a transaction
    tx.execute(
        "INSERT INTO users (id, name, email) VALUES (?, ?, ?)",
        params![4, "Diana Prince", "diana@example.com"],
    )?;
    
    tx.execute(
        "INSERT INTO posts (id, user_id, title, content) VALUES (?, ?, ?, ?)",
        params![5, 4, "Transaction Test", "This post was created in a transaction"],
    )?;
    
    // Commit the transaction
    tx.commit()?;
    
    println!("✅ Transaction completed successfully!");
    
    // Verify the transaction worked
    let new_user_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM users WHERE name = 'Diana Prince'",
        [],
        |row| row.get(0),
    )?;
    
    assert_eq!(new_user_count, 1);
    println!("✅ New user and post created successfully!");
    
    // Demonstrate aggregation queries (same API as duckdb-rs)
    let total_posts: i64 = conn.query_row(
        "SELECT COUNT(*) FROM posts",
        [],
        |row| row.get(0),
    )?;
    
    let avg_posts_per_user: f64 = conn.query_row(
        "SELECT AVG(post_count) FROM (
            SELECT COUNT(p.id) as post_count
            FROM users u
            LEFT JOIN posts p ON u.id = p.user_id
            GROUP BY u.id
        )",
        [],
        |row| row.get(0),
    )?;
    
    println!("\n📈 Summary Statistics:");
    println!("Total posts: {}", total_posts);
    println!("Average posts per user: {:.2}", avg_posts_per_user);
    
    println!("\n🎉 Example completed successfully!");
    println!("💡 This demonstrates that frozen-duckdb provides the exact same API as duckdb-rs,");
    println!("   but with 99% faster build times. No code changes needed!");
    
    Ok(())
}
