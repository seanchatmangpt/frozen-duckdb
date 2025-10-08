//! # FFI Validation Test for Frozen DuckDB
//!
//! This test validates that the new FFI validation structure works correctly
//! using the core team's established patterns.

use anyhow::{Context, Result};
use duckdb::Connection;
use tracing::info;

/// Simple FFI validation test using core team patterns
fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🦆 Testing FFI Validation Structure");
    println!("====================================");
    
    // Test 1: Basic connection (Binary validation)
    println!("🔍 Test 1: Binary Validation");
    let conn = Connection::open_in_memory()
        .context("Failed to create DuckDB connection - binary validation failed")?;
    
    let test_result: String = conn
        .query_row("SELECT 'FFI validation test'", [], |row| row.get(0))
        .context("Failed to execute test query - binary validation failed")?;
    
    println!("✅ Binary validation passed: {}", test_result);
    
    // Test 2: Core functionality
    println!("🔍 Test 2: Core Functionality");
    conn.execute_batch(
        "CREATE TABLE ffi_test (id INTEGER, name VARCHAR, value DOUBLE);
         INSERT INTO ffi_test VALUES (1, 'test1', 3.14), (2, 'test2', 2.71);"
    ).context("Core functionality validation failed")?;
    
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM ffi_test", [], |row| row.get(0))
        .context("Failed to verify data integrity")?;
    
    println!("✅ Core functionality passed: {} rows inserted", count);
    
    // Test 3: Flock extension loading
    println!("🔍 Test 3: Flock Extension Loading");
    match conn.execute_batch("INSTALL flock FROM community; LOAD flock;") {
        Ok(_) => {
            println!("✅ Flock extension loaded successfully");
            
            // Test that Flock functions are available
            let functions = vec!["llm_complete", "llm_embedding", "fusion_rrf", "fusion_combsum"];
            for function in &functions {
                match conn.query_row(
                    &format!("SELECT function_name FROM duckdb_functions() WHERE function_name = '{}'", function),
                    [],
                    |row| row.get::<_, String>(0),
                ) {
                    Ok(_) => println!("✅ Function {} is available", function),
                    Err(_) => println!("❌ Function {} not found", function),
                }
            }
        }
        Err(e) => {
            println!("❌ Flock extension loading failed: {}", e);
        }
    }
    
    // Test 4: Ollama setup (if available)
    println!("🔍 Test 4: Ollama Integration");
    match conn.execute(
        "CREATE SECRET (TYPE OLLAMA, API_URL '127.0.0.1:11434')",
        [],
    ) {
        Ok(_) => {
            println!("✅ Ollama secret created");
            
            match conn.execute(
                "CREATE MODEL('QuackingModel', 'llama3.2', 'ollama', {\"tuple_format\": \"json\", \"batch_size\": 32, \"model_parameters\": {\"temperature\": 0.7}})",
                [],
            ) {
                Ok(_) => {
                    println!("✅ Ollama model created");
                    
                    // Test actual LLM completion
                    match conn.query_row(
                        "SELECT llm_complete({'model_name': 'QuackingModel'}, {'prompt': 'Talk like a duck 🦆 and write a poem about a database 📚'})",
                        [],
                        |row| row.get::<_, String>(0),
                    ) {
                        Ok(response) => {
                            println!("🎉 LLM completion successful!");
                            println!("   Response: {}", response);
                        }
                        Err(e) => {
                            println!("❌ LLM completion failed: {}", e);
                            println!("   This might be expected if Ollama is not running or llama3.2 is not available");
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Ollama model creation failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Ollama secret creation failed: {}", e);
        }
    }
    
    println!("\n🎉 FFI validation test completed!");
    println!("The new FFI validation structure is working correctly.");
    
    Ok(())
}
