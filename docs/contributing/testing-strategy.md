# Testing Strategy Guide

## Overview

Frozen DuckDB follows a **comprehensive testing strategy** that ensures **reliability**, **performance**, and **correctness** across all components. This guide outlines the **testing philosophy**, **test categories**, and **validation procedures** required for **production-quality code**.

## Testing Philosophy

### Core Team Requirements

**Multiple Test Runs:**
```bash
# Core team requirement: Run tests 3+ times to catch flaky behavior
cargo test --all && cargo test --all && cargo test --all
```

**Test Consistency:**
```bash
# Run with different configurations
cargo test --release --all
ARCH=x86_64 source prebuilt/setup_env.sh && cargo test --all
ARCH=arm64 source prebuilt/setup_env.sh && cargo test --all
```

**Performance Validation:**
```bash
# Ensure performance targets are met
cargo test --test performance_tests

# Validate build time requirements
time cargo build --release  # Should be <10s
```

## Test Categories

### 1. Unit Tests

**Purpose**: Test individual functions and modules in isolation

**Location**: `src/` directory, alongside implementation code

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_architecture() {
        let arch = detect();
        assert!(!arch.is_empty());
        assert!(matches!(arch.as_str(), "x86_64" | "arm64" | "aarch64"));
    }

    #[test]
    fn test_binary_validation() {
        // Test with valid binary path
        std::env::set_var("DUCKDB_LIB_DIR", "/valid/path");
        assert!(validate_binary().is_ok());

        // Test with invalid binary path
        std::env::set_var("DUCKDB_LIB_DIR", "/invalid/path");
        assert!(validate_binary().is_err());
        std::env::remove_var("DUCKDB_LIB_DIR");
    }
}
```

### 2. Integration Tests

**Purpose**: Test component interactions and end-to-end functionality

**Location**: `tests/` directory

**Example Structure:**
```
tests/
├── core_functionality_tests.rs    # Core DuckDB operations
├── arrow_tests.rs                # Arrow integration
├── parquet_tests.rs              # Parquet integration
├── flock_tests.rs                # LLM/Flock extension
└── performance_tests.rs          # Performance validation
```

### 3. Property Tests

**Purpose**: Test properties and invariants using generated test data

**Tools**: `proptest` crate for property-based testing

**Example:**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_architecture_detection_consistency(arch in "x86_64|arm64|aarch64") {
        std::env::set_var("ARCH", arch);
        let detected = detect();
        assert_eq!(detected, arch);
        std::env::remove_var("ARCH");
    }

    #[test]
    fn test_binary_name_generation(input_arch in "x86_64|arm64|aarch64|unknown") {
        std::env::set_var("ARCH", input_arch);
        let binary_name = get_binary_name();
        assert!(binary_name.starts_with("libduckdb"));
        assert!(binary_name.ends_with(".dylib"));
        std::env::remove_var("ARCH");
    }
}
```

### 4. Performance Tests

**Purpose**: Validate performance requirements and detect regressions

**Example:**
```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_build_time_requirements() {
        let duration = benchmark::measure_build_time(|| {
            // Simulate build operation
            std::thread::sleep(Duration::from_millis(10));
            Ok(())
        });

        // Validate performance requirement
        assert!(
            duration.as_secs() < 10,
            "Build time exceeded 10s requirement: {:?}",
            duration
        );
    }

    #[test]
    fn test_query_performance() {
        let conn = Connection::open_in_memory().unwrap();

        // Setup test data
        conn.execute("CREATE TABLE test (id INTEGER, data TEXT)", []).unwrap();
        for i in 0..1000 {
            conn.execute(
                "INSERT INTO test VALUES (?, ?)",
                [i, &format!("data_{}", i)],
            ).unwrap();
        }

        // Measure query performance
        let start = std::time::Instant::now();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM test", [], |row| row.get(0)).unwrap();
        let duration = start.elapsed();

        assert_eq!(count, 1000);
        assert!(
            duration.as_millis() < 100,
            "Query too slow: {:?}",
            duration
        );
    }
}
```

### 5. LLM Integration Tests

**Purpose**: Test Flock extension and Ollama integration

**Requirements**: Ollama server running with required models

**Example:**
```rust
#[cfg(test)]
mod flock_tests {
    use super::*;

    #[test]
    fn test_flock_extension_loading() {
        let conn = Connection::open_in_memory().unwrap();

        // Install and load Flock extension
        conn.execute_batch("INSTALL flock FROM community; LOAD flock;").unwrap();

        // Verify extension loaded
        let extension: String = conn.query_row(
            "SELECT extension_name FROM duckdb_extensions() WHERE extension_name = 'flock'",
            [],
            |row| row.get(0),
        ).unwrap();

        assert_eq!(extension, "flock");
    }

    #[test]
    fn test_llm_completion() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("INSTALL flock FROM community; LOAD flock;").unwrap();

        // Create models
        conn.execute("CREATE MODEL('test_coder', 'qwen3-coder:30b', 'ollama')", []).unwrap();
        conn.execute("CREATE PROMPT('test_prompt', 'Complete: {{text}}')", []).unwrap();

        // Test completion
        let result: String = conn.query_row(
            "SELECT llm_complete({'model_name': 'test_coder'}, {'prompt_name': 'test_prompt', 'context_columns': [{'data': 'Hello'}]})",
            [],
            |row| row.get(0),
        ).unwrap();

        assert!(!result.is_empty());
        assert!(result.to_lowercase().contains("hello"));
    }
}
```

## Test Data Management

### 1. Test Dataset Generation

**Automated Test Data Setup:**
```rust
// In test setup code
fn setup_test_datasets() -> Result<(), Box<dyn std::error::Error>> {
    // Generate Chinook dataset
    let dataset_manager = DatasetManager::new()?;
    dataset_manager.download_chinook("test_datasets", "parquet")?;

    // Generate TPC-H dataset
    dataset_manager.download_tpch("test_datasets", "parquet")?;

    Ok(())
}

#[test]
fn test_with_real_data() {
    setup_test_datasets().expect("Failed to setup test data");

    // Test with real data
    let conn = Connection::open("test_datasets/tpch.duckdb").unwrap();
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM customer", [], |row| row.get(0)).unwrap();
    assert!(count > 0);
}
```

**Test Data Validation:**
```sql
-- Validate test data integrity
SELECT
    'customer' as table_name,
    COUNT(*) as row_count,
    COUNT(DISTINCT c_custkey) as unique_customers,
    MIN(c_acctbal) as min_balance,
    MAX(c_acctbal) as max_balance
FROM customer

UNION ALL

SELECT
    'orders' as table_name,
    COUNT(*) as row_count,
    COUNT(DISTINCT o_orderkey) as unique_orders,
    MIN(o_totalprice) as min_total,
    MAX(o_totalprice) as max_total
FROM orders

ORDER BY table_name;
```

### 2. Test Fixtures

**Reusable Test Components:**
```rust
// test_fixtures.rs
use duckdb::Connection;

pub struct TestDatabase {
    conn: Connection,
}

impl TestDatabase {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open_in_memory()?;
        Ok(Self { conn })
    }

    pub fn create_test_table(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute(
            "CREATE TABLE test (id INTEGER, name TEXT, value DECIMAL)",
            [],
        )?;
        Ok(())
    }

    pub fn insert_test_data(&self, count: i32) -> Result<(), Box<dyn std::error::Error>> {
        for i in 0..count {
            self.conn.execute(
                "INSERT INTO test VALUES (?, ?, ?)",
                [i, &format!("item_{}", i), (i as f64) * 1.5],
            )?;
        }
        Ok(())
    }

    pub fn get_connection(&self) -> &Connection {
        &self.conn
    }
}

// Usage in tests
#[test]
fn test_data_operations() {
    let db = TestDatabase::new().unwrap();
    db.create_test_table().unwrap();
    db.insert_test_data(100).unwrap();

    let count: i64 = db.get_connection()
        .query_row("SELECT COUNT(*) FROM test", [], |row| row.get(0))
        .unwrap();

    assert_eq!(count, 100);
}
```

## Test Execution Strategy

### 1. Local Testing

**Development Testing:**
```bash
# Quick unit tests during development
cargo test --lib

# Test specific modules
cargo test architecture
cargo test benchmark

# Test with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_detect_architecture
```

**Integration Testing:**
```bash
# Test all components together
cargo test --all

# Test with release build
cargo test --release --all

# Test with different architectures
ARCH=x86_64 source prebuilt/setup_env.sh && cargo test --all
ARCH=arm64 source prebuilt/setup_env.sh && cargo test --all
```

### 2. CI/CD Testing

**Automated Testing Pipeline:**
```yaml
# .github/workflows/test.yml
- name: Run tests multiple times
  run: |
    for i in {1..3}; do
      echo "Test run $i of 3"
      cargo test --all
    done

- name: Test with different configurations
  run: |
    cargo test --release --all
    ARCH=x86_64 source prebuilt/setup_env.sh && cargo test --all
    ARCH=arm64 source prebuilt/setup_env.sh && cargo test --all

- name: Performance validation
  run: |
    cargo test --test performance_tests
    # Validate build time requirements
```

### 3. Property-Based Testing

**Comprehensive Input Testing:**
```rust
proptest! {
    #[test]
    fn test_binary_validation_with_various_paths(
        lib_dir in "(/[a-zA-Z0-9/_-]+)",
        include_dir in "(/[a-zA-Z0-9/_-]+)"
    ) {
        std::env::set_var("DUCKDB_LIB_DIR", lib_dir);
        std::env::set_var("DUCKDB_INCLUDE_DIR", include_dir);

        // Test should handle various path formats gracefully
        let _ = validate_binary(); // Should not panic

        std::env::remove_var("DUCKDB_LIB_DIR");
        std::env::remove_var("DUCKDB_INCLUDE_DIR");
    }

    #[test]
    fn test_architecture_detection_robustness(input in ".*") {
        std::env::set_var("ARCH", input);

        // Should handle any input gracefully
        let arch = detect();
        assert!(!arch.is_empty());

        std::env::remove_var("ARCH");
    }
}
```

## Test Data Generation

### 1. Automated Test Data

**Test Data Generation Script:**
```bash
#!/bin/bash
# generate_test_data.sh

echo "🧪 Generating test datasets..."

# Generate Chinook dataset
frozen-duckdb download --dataset chinook --format parquet --output-dir test_data

# Generate TPC-H dataset
frozen-duckdb download --dataset tpch --format parquet --output-dir test_data

# Generate custom test data
duckdb test_data/custom.duckdb -c "
CREATE TABLE custom_test AS
SELECT
    id,
    'item_' || id as name,
    id * 1.5 as value,
    CASE WHEN id % 2 = 0 THEN 'even' ELSE 'odd' END as category
FROM generate_series(1, 1000) as id;
"

echo "✅ Test data generation complete"
ls -la test_data/
```

### 2. Test Data Validation

**Data Quality Checks:**
```sql
-- Validate generated datasets
CREATE TABLE data_quality_report AS

-- Chinook validation
SELECT
    'chinook' as dataset,
    'artists' as table_name,
    COUNT(*) as row_count,
    COUNT(DISTINCT ArtistId) as unique_ids,
    MIN(LENGTH(Name)) as min_name_length,
    MAX(LENGTH(Name)) as max_name_length
FROM 'test_data/chinook.parquet'

UNION ALL

-- TPC-H validation
SELECT
    'tpch' as dataset,
    'customer' as table_name,
    COUNT(*) as row_count,
    COUNT(DISTINCT c_custkey) as unique_ids,
    MIN(c_acctbal) as min_balance,
    MAX(c_acctbal) as max_balance
FROM 'test_data/customer.parquet'

ORDER BY dataset, table_name;
```

## Performance Testing

### 1. Build Performance Tests

**Build Time Validation:**
```rust
#[test]
fn test_build_time_requirements() {
    let duration = benchmark::measure_build_time(|| {
        // Simulate build operation
        std::thread::sleep(Duration::from_millis(50));
        Ok(())
    });

    // Validate against SLO
    assert!(
        duration.as_secs() < 10,
        "Build time exceeded 10s requirement: {:?}",
        duration
    );
}

#[test]
fn test_incremental_build_performance() {
    // Test that incremental builds are fast
    let first_build = benchmark::measure_build_time(|| Ok(()));
    let second_build = benchmark::measure_build_time(|| Ok(()));

    // Incremental should be much faster
    assert!(
        second_build < first_build,
        "Incremental build should be faster: {:?} vs {:?}",
        second_build, first_build
    );
}
```

### 2. Runtime Performance Tests

**Query Performance Validation:**
```rust
#[test]
fn test_query_performance_requirements() {
    let conn = Connection::open_in_memory().unwrap();

    // Setup test data
    conn.execute("CREATE TABLE perf_test (id INTEGER, data TEXT)", []).unwrap();
    for i in 0..10000 {
        conn.execute(
            "INSERT INTO perf_test VALUES (?, ?)",
            [i, &format!("data_{}", i)],
        ).unwrap();
    }

    // Test query performance
    let start = Instant::now();
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM perf_test WHERE id > 5000", [], |row| row.get(0)).unwrap();
    let duration = start.elapsed();

    assert_eq!(count, 5000);
    assert!(
        duration.as_millis() < 100,
        "Query performance requirement not met: {:?}",
        duration
    );
}

#[test]
fn test_llm_performance_requirements() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("INSTALL flock FROM community; LOAD flock;").unwrap();

    // Test completion performance
    let start = Instant::now();
    let result: String = conn.query_row(
        "SELECT llm_complete({'model_name': 'test_coder'}, {'prompt_name': 'test_prompt', 'context_columns': [{'data': 'test'}]})",
        [],
        |row| row.get(0),
    ).unwrap();
    let duration = start.elapsed();

    assert!(!result.is_empty());
    assert!(
        duration.as_secs() < 10,
        "LLM operation too slow: {:?}",
        duration
    );
}
```

## Test Organization

### 1. Test Module Structure

**Library Tests:**
```
src/
├── lib.rs
│   └── tests for library entry point
├── architecture.rs
│   └── mod tests for architecture detection
├── benchmark.rs
│   └── mod tests for performance measurement
└── env_setup.rs
    └── mod tests for environment validation
```

**Integration Tests:**
```
tests/
├── core_functionality_tests.rs    # Core DuckDB operations
├── arrow_tests.rs                # Arrow integration
├── parquet_tests.rs              # Parquet integration
├── polars_tests.rs               # Polars integration
├── flock_tests.rs                # LLM/Flock extension
├── vss_tests.rs                  # Vector similarity search
└── tpch_integration_test.rs      # TPC-H benchmark tests
```

### 2. Test Naming Conventions

**Unit Tests:**
```rust
#[test]
fn test_function_name() {
    // Test specific function
}

#[test]
fn test_error_condition() {
    // Test error handling
}

#[test]
fn test_edge_case() {
    // Test boundary conditions
}
```

**Integration Tests:**
```rust
#[test]
fn test_component_integration() {
    // Test multiple components working together
}

#[test]
fn test_end_to_end_workflow() {
    // Test complete user workflows
}

#[test]
fn test_performance_integration() {
    // Test performance across integrated components
}
```

## Test Coverage Requirements

### 1. Code Coverage

**Coverage Targets:**
- **Core library**: >90% coverage
- **CLI functionality**: >85% coverage
- **LLM integration**: >80% coverage (considering external dependencies)
- **Error paths**: >95% coverage

**Coverage Measurement:**
```bash
# Install coverage tools
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov --all --lcov --output-path coverage.lcov

# View coverage report
cargo llvm-cov --all --html

# Check coverage thresholds
cargo llvm-cov --all --text | grep -E "(Total|Functions|Lines|Branches)"
```

### 2. Test Scenarios

**Core Functionality:**
- [ ] Architecture detection on all supported platforms
- [ ] Environment validation with various configurations
- [ ] Binary validation with missing/corrupted files
- [ ] Performance measurement accuracy
- [ ] Error handling for all failure modes

**Integration Scenarios:**
- [ ] End-to-end dataset generation and usage
- [ ] Format conversion between all supported types
- [ ] LLM operations with various models and prompts
- [ ] Cross-component interactions
- [ ] Performance across different architectures

**Edge Cases:**
- [ ] Empty inputs and outputs
- [ ] Very large datasets
- [ ] Network failures and timeouts
- [ ] Disk space limitations
- [ ] Permission restrictions

## Flaky Test Detection

### 1. Multiple Run Strategy

**Core Team Requirement:**
```bash
# Run tests multiple times to detect flakiness
for run in {1..3}; do
    echo "=== Test Run $run ==="
    cargo test --all
    echo "=== End Run $run ==="
done
```

**Automated Flaky Detection:**
```bash
#!/bin/bash
# detect_flaky_tests.sh

TEST_RESULTS=()

# Run tests multiple times
for i in {1..5}; do
    echo "Test run $i of 5"
    if cargo test --all --quiet; then
        TEST_RESULTS+=("PASS")
    else
        TEST_RESULTS+=("FAIL")
    fi
done

# Analyze results for flakiness
PASS_COUNT=$(echo "${TEST_RESULTS[@]}" | tr ' ' '\n' | grep -c "PASS")
FAIL_COUNT=$(echo "${TEST_RESULTS[@]}" | tr ' ' '\n' | grep -c "FAIL")

echo "Results: $PASS_COUNT passes, $FAIL_COUNT failures out of 5 runs"

if (( FAIL_COUNT > 0 && PASS_COUNT > 0 )); then
    echo "⚠️  Potential flaky tests detected!"
    echo "   Consider investigating test reliability"
fi
```

### 2. Test Stability Metrics

**Test Execution Tracking:**
```sql
-- Track test execution stability
CREATE TABLE test_execution_log (
    test_name TEXT,
    execution_time_ms INTEGER,
    result TEXT, -- 'PASS' or 'FAIL'
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    run_id INTEGER
);

-- Log test execution
INSERT INTO test_execution_log (test_name, execution_time_ms, result, run_id)
SELECT
    'test_detect_architecture' as test_name,
    5 as execution_time_ms,
    'PASS' as result,
    1 as run_id;
```

## Error Testing Strategy

### 1. Error Condition Coverage

**Comprehensive Error Testing:**
```rust
#[test]
fn test_error_conditions() {
    // Test missing environment variables
    std::env::remove_var("DUCKDB_LIB_DIR");
    assert!(!is_configured());

    // Test invalid binary paths
    std::env::set_var("DUCKDB_LIB_DIR", "/nonexistent/path");
    assert!(validate_binary().is_err());
    std::env::remove_var("DUCKDB_LIB_DIR");

    // Test network failures (LLM operations)
    // Mock network failure and verify graceful handling

    // Test malformed inputs
    let result = validate_binary_with_malformed_input();
    assert!(result.is_err());
}
```

### 2. Error Message Quality

**Actionable Error Messages:**
```rust
#[test]
fn test_error_message_quality() {
    // Test that error messages are actionable
    std::env::set_var("DUCKDB_LIB_DIR", "/nonexistent");
    let result = validate_binary();
    std::env::remove_var("DUCKDB_LIB_DIR");

    assert!(result.is_err());
    let error_msg = format!("{}", result.unwrap_err());

    // Error message should be actionable
    assert!(error_msg.contains("DUCKDB_LIB_DIR"));
    assert!(error_msg.contains("setup_env.sh"));
    assert!(error_msg.to_lowercase().contains("run"));
}
```

## Performance Testing

### 1. Build Performance Tests

**Build Time Validation:**
```rust
#[test]
fn test_build_performance_slo() {
    // Test first build performance
    let first_build = benchmark::measure_build_time(|| {
        // Simulate first build (heavier)
        std::thread::sleep(Duration::from_secs(8));
        Ok(())
    });

    assert!(
        first_build.as_secs() < 10,
        "First build exceeded 10s SLO: {:?}",
        first_build
    );

    // Test incremental build performance
    let incremental_build = benchmark::measure_build_time(|| {
        // Simulate incremental build (lighter)
        std::thread::sleep(Duration::from_millis(100));
        Ok(())
    });

    assert!(
        incremental_build.as_millis() < 1000,
        "Incremental build exceeded 1s SLO: {:?}",
        incremental_build
    );
}
```

### 2. Runtime Performance Tests

**Query Performance:**
```rust
#[test]
fn test_query_performance_requirements() {
    let conn = Connection::open_in_memory().unwrap();

    // Setup large dataset
    conn.execute("CREATE TABLE large_test (id INTEGER, data TEXT)", []).unwrap();
    for i in 0..100000 {
        conn.execute(
            "INSERT INTO large_test VALUES (?, ?)",
            [i, &format!("data_{}", i)],
        ).unwrap();
    }

    // Test complex query performance
    let start = Instant::now();
    let result: i64 = conn.query_row(
        "SELECT COUNT(*) FROM large_test WHERE id > 50000 AND LENGTH(data) > 8",
        [],
        |row| row.get(0),
    ).unwrap();
    let duration = start.elapsed();

    assert_eq!(result, 25000);
    assert!(
        duration.as_millis() < 200,
        "Query performance requirement not met: {:?}",
        duration
    );
}
```

## LLM Testing Strategy

### 1. Flock Extension Testing

**Extension Loading Tests:**
```rust
#[test]
fn test_flock_extension_loading() {
    let conn = Connection::open_in_memory().unwrap();

    // Test extension installation
    conn.execute_batch("INSTALL flock FROM community; LOAD flock;").unwrap();

    // Verify extension loaded
    let extension: String = conn.query_row(
        "SELECT extension_name FROM duckdb_extensions() WHERE extension_name = 'flock'",
        [],
        |row| row.get(0),
    ).unwrap();

    assert_eq!(extension, "flock");
}

#[test]
fn test_ollama_secret_creation() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("INSTALL flock FROM community; LOAD flock;").unwrap();

    // Test secret creation
    conn.execute(
        "CREATE SECRET ollama_secret (TYPE OLLAMA, API_URL 'http://localhost:11434')",
        [],
    ).unwrap();

    // Verify secret exists
    let secret_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM duckdb_secrets() WHERE secret_name = 'ollama_secret'",
        [],
        |row| row.get(0),
    ).unwrap();

    assert_eq!(secret_count, 1);
}
```

### 2. Model Management Tests

**Model Creation Tests:**
```rust
#[test]
fn test_model_creation() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("INSTALL flock FROM community; LOAD flock;").unwrap();

    // Create test models
    conn.execute("CREATE MODEL('test_coder', 'qwen3-coder:30b', 'ollama')", []).unwrap();
    conn.execute("CREATE MODEL('test_embedder', 'qwen3-embedding:8b', 'ollama')", []).unwrap();

    // Verify models exist
    let model_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM duckdb_models()",
        [],
        |row| row.get(0),
    ).unwrap();

    assert_eq!(model_count, 2);
}
```

### 3. LLM Operation Tests

**Text Completion Tests:**
```rust
#[test]
fn test_llm_completion() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("INSTALL flock FROM community; LOAD flock;").unwrap();

    // Setup minimal test environment
    conn.execute("CREATE MODEL('test_coder', 'qwen3-coder:7b', 'ollama')", []).unwrap();
    conn.execute("CREATE PROMPT('test_prompt', 'Complete: {{text}}')", []).unwrap();

    // Test completion
    let result: String = conn.query_row(
        "SELECT llm_complete({'model_name': 'test_coder'}, {'prompt_name': 'test_prompt', 'context_columns': [{'data': 'Hello'}]})",
        [],
        |row| row.get(0),
    ).unwrap();

    // Basic validation
    assert!(!result.is_empty());
    assert!(result.to_lowercase().contains("hello"));
}

#[test]
fn test_embedding_generation() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("INSTALL flock FROM community; LOAD flock;").unwrap();

    // Create embedding model
    conn.execute("CREATE MODEL('test_embedder', 'qwen3-embedding:8b', 'ollama')", []).unwrap();

    // Test embedding generation
    let embedding: Vec<f32> = conn.query_row(
        "SELECT llm_embedding({'model_name': 'test_embedder'}, [{'data': 'test text'}])",
        [],
        |row| {
            let embedding_str: String = row.get(0).unwrap();
            serde_json::from_str(&embedding_str).unwrap_or_default()
        },
    ).unwrap();

    // Validate embedding properties
    assert_eq!(embedding.len(), 1024);
    assert!(embedding.iter().all(|&x| x.is_finite()));
}
```

## Test Maintenance

### 1. Test Data Updates

**Regular Test Data Refresh:**
```bash
#!/bin/bash
# refresh_test_data.sh

echo "🔄 Refreshing test datasets..."

# Update Chinook dataset
frozen-duckdb download --dataset chinook --format parquet --output-dir test_data

# Update TPC-H dataset
frozen-duckdb download --dataset tpch --format parquet --output-dir test_data

# Validate updated data
duckdb test_data/chinook.duckdb -c "
SELECT 'Chinook' as dataset, COUNT(*) as tracks FROM tracks;
"

duckdb test_data/tpch.duckdb -c "
SELECT 'TPC-H' as dataset, COUNT(*) as customers FROM customer;
"

echo "✅ Test data refresh complete"
```

### 2. Test Performance Monitoring

**Performance Regression Detection:**
```sql
-- Track test performance over time
CREATE TABLE test_performance_history (
    test_name TEXT,
    execution_time_ms INTEGER,
    memory_usage_mb INTEGER,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Log test performance
INSERT INTO test_performance_history (test_name, execution_time_ms, memory_usage_mb)
SELECT
    'test_large_dataset_query' as test_name,
    150 as execution_time_ms,
    200 as memory_usage_mb;
```

## Summary

The testing strategy for Frozen DuckDB ensures **comprehensive validation** of **functionality**, **performance**, and **reliability** through **multiple test categories**, **property-based testing**, and **performance validation**. The strategy follows **core team requirements** for **multiple test runs** and **performance SLO validation**.

**Key Testing Components:**
- **Unit tests**: Individual function and module testing
- **Integration tests**: End-to-end component interaction testing
- **Property tests**: Comprehensive input validation with generated data
- **Performance tests**: SLO validation and regression detection
- **LLM tests**: Flock extension and Ollama integration testing

**Testing Best Practices:**
- **Multiple runs**: Catch flaky behavior through repeated execution
- **Configuration testing**: Validate across different environments and architectures
- **Performance validation**: Ensure SLO requirements are consistently met
- **Error coverage**: Comprehensive testing of all failure modes

**Quality Assurance:**
- **High coverage targets**: >90% for core functionality
- **Flaky test detection**: Automated identification of unreliable tests
- **Performance monitoring**: Track and prevent performance regressions
- **Error message quality**: Actionable guidance for troubleshooting

**Next Steps:**
1. Implement the testing patterns described in this guide
2. Set up automated flaky test detection in CI/CD
3. Establish performance baselines for regression detection
4. Review the [Coding Standards Guide](./coding-standards.md) for code quality requirements
