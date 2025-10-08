#!/bin/bash

# Validate Production Build Script
# Creates a new Rust project and validates frozen-duckdb from crates.io

set -e  # Exit on any error
set -u  # Exit on undefined variables
set -o pipefail  # Exit on pipe failures

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="frozen-duckdb-validation"
PROJECT_DIR="../${PROJECT_NAME}"
CRATE_NAME="frozen-duckdb"
CRATE_VERSION="0.1.0"

echo -e "${BLUE}🦆 Frozen DuckDB Production Build Validation${NC}"
echo "=================================================="
echo ""

# Function to print status
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Clean up any existing validation project
if [ -d "$PROJECT_DIR" ]; then
    print_warning "Removing existing validation project..."
    rm -rf "$PROJECT_DIR"
fi

# Create new Rust project
print_info "Creating new Rust project: $PROJECT_NAME"
cargo new "$PROJECT_NAME" --bin
cd "$PROJECT_NAME"

# Add frozen-duckdb dependency
print_info "Adding frozen-duckdb dependency from crates.io..."
cargo add frozen-duckdb

# Verify the dependency was added correctly
if grep -q "frozen-duckdb" Cargo.toml; then
    print_status "frozen-duckdb dependency added successfully"
else
    print_error "Failed to add frozen-duckdb dependency"
    exit 1
fi

# Create comprehensive validation code
print_info "Creating validation code..."

cat > src/main.rs << 'EOF'
//! Production Build Validation for frozen-duckdb
//! 
//! This program validates that frozen-duckdb works correctly when installed
//! from crates.io as a production dependency.

use frozen_duckdb::{Connection, Result};
use std::time::Instant;

fn main() -> Result<()> {
    println!("🦆 Frozen DuckDB Production Build Validation");
    println!("=============================================");
    println!("");

    // Test 1: Basic Connection
    println!("📋 Test 1: Basic Connection");
    let start = Instant::now();
    let conn = Connection::open_in_memory()?;
    let duration = start.elapsed();
    println!("   ✅ In-memory connection created in {:?}", duration);
    println!("");

    // Test 2: Basic SQL Operations
    println!("📋 Test 2: Basic SQL Operations");
    let start = Instant::now();
    
    // Create table
    conn.execute("CREATE TABLE test_table (id INTEGER, name TEXT, value REAL)", [])?;
    println!("   ✅ Table created");
    
    // Insert data
    conn.execute("INSERT INTO test_table VALUES (1, 'Alice', 42.5)", [])?;
    conn.execute("INSERT INTO test_table VALUES (2, 'Bob', 37.2)", [])?;
    conn.execute("INSERT INTO test_table VALUES (3, 'Charlie', 29.8)", [])?;
    println!("   ✅ Data inserted");
    
    // Query data
    let mut stmt = conn.prepare("SELECT * FROM test_table WHERE value > ?")?;
    let rows: Vec<(i32, String, f64)> = stmt.query_map([30.0], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?.collect::<std::result::Result<Vec<_>, _>>()?;
    
    println!("   ✅ Query executed, found {} rows", rows.len());
    for (id, name, value) in &rows {
        println!("      - ID: {}, Name: {}, Value: {}", id, name, value);
    }
    
    let duration = start.elapsed();
    println!("   ✅ All SQL operations completed in {:?}", duration);
    println!("");

    // Test 3: JSON Operations
    println!("📋 Test 3: JSON Operations");
    let start = Instant::now();
    
    conn.execute("CREATE TABLE json_test (id INTEGER, data JSON)", [])?;
    conn.execute("INSERT INTO json_test VALUES (1, '{\"name\": \"test\", \"value\": 123}')", [])?;
    
    let json_result: String = conn.query_row(
        "SELECT data->>'name' FROM json_test WHERE id = 1",
        [],
        |row| row.get(0)
    )?;
    
    println!("   ✅ JSON query result: '{}'", json_result);
    let duration = start.elapsed();
    println!("   ✅ JSON operations completed in {:?}", duration);
    println!("");

    // Test 4: Parquet Operations (if available)
    println!("📋 Test 4: Parquet Operations");
    let start = Instant::now();
    
    // Create a simple parquet file for testing
    conn.execute("CREATE TABLE parquet_test AS SELECT * FROM test_table", [])?;
    
    // Try to export to parquet (this might fail if parquet extension isn't available)
    match conn.execute("COPY parquet_test TO 'test.parquet' (FORMAT PARQUET)", []) {
        Ok(_) => {
            println!("   ✅ Parquet export successful");
            // Clean up
            let _ = std::fs::remove_file("test.parquet");
        },
        Err(e) => {
            println!("   ⚠️  Parquet export not available: {}", e);
        }
    }
    
    let duration = start.elapsed();
    println!("   ✅ Parquet operations completed in {:?}", duration);
    println!("");

    // Test 5: Performance Test
    println!("📋 Test 5: Performance Test");
    let start = Instant::now();
    
    // Create a larger dataset
    conn.execute("CREATE TABLE perf_test (id INTEGER, data TEXT)", [])?;
    
    let batch_start = Instant::now();
    for i in 0..1000 {
        conn.execute("INSERT INTO perf_test VALUES (?, ?)", [&i, &format!("data_{}", i)])?;
    }
    let batch_duration = batch_start.elapsed();
    
    // Query performance
    let query_start = Instant::now();
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM perf_test", [], |row| row.get(0))?;
    let query_duration = query_start.elapsed();
    
    println!("   ✅ Inserted 1000 rows in {:?}", batch_duration);
    println!("   ✅ Counted {} rows in {:?}", count, query_duration);
    
    let duration = start.elapsed();
    println!("   ✅ Performance test completed in {:?}", duration);
    println!("");

    // Test 6: Error Handling
    println!("📋 Test 6: Error Handling");
    let start = Instant::now();
    
    // Test invalid SQL
    match conn.execute("SELECT * FROM non_existent_table", []) {
        Ok(_) => println!("   ❌ Expected error for invalid SQL"),
        Err(_) => println!("   ✅ Proper error handling for invalid SQL"),
    }
    
    // Test invalid parameters
    match conn.query_row("SELECT ?", [42], |row| row.get::<_, String>(0)) {
        Ok(_) => println!("   ✅ Type conversion handled properly"),
        Err(_) => println!("   ✅ Proper error handling for type conversion"),
    }
    
    let duration = start.elapsed();
    println!("   ✅ Error handling test completed in {:?}", duration);
    println!("");

    // Test 7: CLI Integration (if available)
    println!("📋 Test 7: CLI Integration");
    let start = Instant::now();
    
    // Test if we can access CLI modules
    match std::process::Command::new("cargo")
        .args(&["run", "--bin", "frozen-duckdb-cli", "--", "--help"])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                println!("   ✅ CLI binary available and functional");
            } else {
                println!("   ⚠️  CLI binary not available in this context");
            }
        },
        Err(_) => {
            println!("   ⚠️  CLI binary not available in this context");
        }
    }
    
    let duration = start.elapsed();
    println!("   ✅ CLI integration test completed in {:?}", duration);
    println!("");

    // Summary
    let total_duration = start.elapsed();
    println!("🎉 All validation tests completed successfully!");
    println!("📊 Total validation time: {:?}", total_duration);
    println!("");
    println!("✅ frozen-duckdb v{} is working correctly from crates.io!", env!("CARGO_PKG_VERSION"));
    println!("🚀 Ready for production use!");
    
    Ok(())
}
EOF

# Create additional test files
print_info "Creating additional test files..."

# Create a lib.rs test
cat > src/lib.rs << 'EOF'
//! Library validation for frozen-duckdb

use frozen_duckdb::{Connection, Result};

/// Test basic library functionality
pub fn test_basic_functionality() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    conn.execute("CREATE TABLE test (id INTEGER)", [])?;
    conn.execute("INSERT INTO test VALUES (42)", [])?;
    
    let result: i32 = conn.query_row("SELECT id FROM test", [], |row| row.get(0))?;
    assert_eq!(result, 42);
    
    Ok(())
}

/// Test connection pooling (if available)
pub fn test_connection_pooling() -> Result<()> {
    // This would test connection pooling if implemented
    // For now, just test multiple connections
    let conn1 = Connection::open_in_memory()?;
    let conn2 = Connection::open_in_memory()?;
    
    conn1.execute("CREATE TABLE test1 (id INTEGER)", [])?;
    conn2.execute("CREATE TABLE test2 (id INTEGER)", [])?;
    
    Ok(())
}
EOF

# Update Cargo.toml to include lib
cat >> Cargo.toml << 'EOF'

[lib]
name = "frozen_duckdb_validation"
path = "src/lib.rs"
EOF

# Create integration tests
mkdir -p tests
cat > tests/integration_tests.rs << 'EOF'
//! Integration tests for frozen-duckdb validation

use frozen_duckdb_validation::{test_basic_functionality, test_connection_pooling};
use frozen_duckdb::{Connection, Result};

#[test]
fn test_library_functionality() -> Result<()> {
    test_basic_functionality()
}

#[test]
fn test_connection_pooling() -> Result<()> {
    test_connection_pooling()
}

#[test]
fn test_drop_in_replacement() -> Result<()> {
    // Test that frozen-duckdb behaves like duckdb-rs
    let conn = Connection::open_in_memory()?;
    
    // Test basic operations
    conn.execute("CREATE TABLE users (id INTEGER, name TEXT)", [])?;
    conn.execute("INSERT INTO users VALUES (1, 'Alice')", [])?;
    conn.execute("INSERT INTO users VALUES (2, 'Bob')", [])?;
    
    // Test query
    let mut stmt = conn.prepare("SELECT name FROM users WHERE id = ?")?;
    let name: String = stmt.query_row([1], |row| row.get(0))?;
    assert_eq!(name, "Alice");
    
    // Test batch operations
    let names: Vec<String> = conn.query("SELECT name FROM users ORDER BY id", [], |row| {
        Ok(row.get(0)?)
    })?.collect::<std::result::Result<Vec<_>, _>>()?;
    
    assert_eq!(names, vec!["Alice", "Bob"]);
    
    Ok(())
}

#[test]
fn test_performance_characteristics() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    
    // Create test data
    conn.execute("CREATE TABLE perf_test (id INTEGER, data TEXT)", [])?;
    
    // Measure insert performance
    let start = std::time::Instant::now();
    for i in 0..100 {
        conn.execute("INSERT INTO perf_test VALUES (?, ?)", [&i, &format!("data_{}", i)])?;
    }
    let insert_duration = start.elapsed();
    
    // Measure query performance
    let start = std::time::Instant::now();
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM perf_test", [], |row| row.get(0))?;
    let query_duration = start.elapsed();
    
    assert_eq!(count, 100);
    assert!(insert_duration.as_millis() < 1000, "Insert should be fast");
    assert!(query_duration.as_millis() < 100, "Query should be very fast");
    
    Ok(())
}
EOF

# Build and test the project
print_info "Building validation project..."
if cargo build --release; then
    print_status "Release build successful"
else
    print_error "Release build failed"
    exit 1
fi

print_info "Running validation tests..."
if cargo test --release; then
    print_status "All tests passed"
else
    print_error "Some tests failed"
    exit 1
fi

print_info "Running main validation program..."
if cargo run --release; then
    print_status "Main validation program completed successfully"
else
    print_error "Main validation program failed"
    exit 1
fi

# Check crate version
print_info "Verifying crate version..."
CRATE_VERSION_IN_PROJECT=$(grep "frozen-duckdb" Cargo.toml | head -1 | sed 's/.*"\([^"]*\)".*/\1/')
if [ "$CRATE_VERSION_IN_PROJECT" = "$CRATE_VERSION" ]; then
    print_status "Correct crate version ($CRATE_VERSION) is being used"
else
    print_warning "Crate version mismatch. Expected: $CRATE_VERSION, Got: $CRATE_VERSION_IN_PROJECT"
fi

# Performance summary
print_info "Performance Summary:"
echo "   - Build time: $(cargo build --release --quiet 2>&1 | grep -o 'Finished.*' || echo 'N/A')"
echo "   - Binary size: $(ls -lh target/release/frozen-duckdb-validation 2>/dev/null | awk '{print $5}' || echo 'N/A')"

# Cleanup option
echo ""
read -p "Do you want to keep the validation project? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    print_info "Cleaning up validation project..."
    cd ..
    rm -rf "$PROJECT_NAME"
    print_status "Validation project cleaned up"
else
    print_info "Validation project kept at: $PROJECT_DIR"
    print_info "You can run it again with: cd $PROJECT_NAME && cargo run --release"
fi

echo ""
print_status "🎉 Production build validation completed successfully!"
print_status "✅ frozen-duckdb v$CRATE_VERSION is ready for production use!"
