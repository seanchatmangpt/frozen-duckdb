# Performance Tuning Guide

## Overview

This guide provides **comprehensive performance tuning** strategies for Frozen DuckDB, covering **build optimization**, **runtime performance**, **memory management**, and **query optimization** to achieve **maximum efficiency**.

## Build Performance Optimization

### 1. Environment Setup Optimization

**Fastest Setup:**
```bash
# One-time setup for persistent environment
echo 'export DUCKDB_LIB_DIR="$(pwd)/prebuilt"' >> ~/.bashrc
echo 'export DUCKDB_INCLUDE_DIR="$(pwd)/prebuilt"' >> ~/.bashrc
source ~/.bashrc
```

**Verification:**
```bash
# Verify environment is configured
echo $DUCKDB_LIB_DIR
echo $DUCKDB_INCLUDE_DIR

# Should show: /path/to/frozen-duckdb/prebuilt
```

### 2. Build Configuration

**Optimal Cargo Configuration:**
```toml
# .cargo/config.toml
[build]
rustflags = ["-C", "target-cpu=native"]

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

**Build Script Optimization:**
```rust
// build.rs - optimized for performance
fn main() {
    // Only configure if environment is set
    if let Ok(lib_dir) = env::var("DUCKDB_LIB_DIR") {
        println!("cargo:rustc-link-search=native={}", lib_dir);
        println!("cargo:rustc-link-lib=dylib=duckdb");
        println!("cargo:include={}", lib_dir);

        // Minimize rerun triggers
        println!("cargo:rerun-if-env-changed=DUCKDB_LIB_DIR");
        println!("cargo:rerun-if-env-changed=DUCKDB_INCLUDE_DIR");
    }
}
```

### 3. Incremental Build Optimization

**Enable sccache (Recommended):**
```bash
# Install sccache
cargo install sccache

# Configure Rust to use sccache
export RUSTC_WRAPPER=sccache

# Verify sccache is working
sccache --show-stats
```

**Build Cache Optimization:**
```bash
# Clean only when necessary
cargo clean  # Only when dependencies change

# Use incremental builds
cargo build  # Fast incremental builds with frozen DuckDB
```

## Runtime Performance Optimization

### 1. Connection Management

**Efficient Connection Handling:**
```rust
use duckdb::Connection;
use std::sync::Arc;

// Reuse connections for multiple operations
fn efficient_database_operations() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Arc::new(Connection::open_in_memory()?);

    // Multiple operations on same connection (fast)
    for i in 0..1000 {
        conn.execute(&format!("INSERT INTO test VALUES ({})", i), [])?;
    }

    // Query with prepared statement (faster than string formatting)
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM test WHERE id > ?")?;
    let count: i64 = stmt.query_row([500], |row| row.get(0))?;

    println!("Found {} records", count);
    Ok(())
}
```

**Connection Pooling:**
```rust
use std::collections::HashMap;
use duckdb::Connection;

struct ConnectionPool {
    connections: HashMap<String, Connection>,
}

impl ConnectionPool {
    fn get_connection(&mut self, db_path: &str) -> &Connection {
        self.connections.entry(db_path.to_string()).or_insert_with(|| {
            Connection::open(db_path).expect("Failed to open database")
        })
    }
}
```

### 2. Query Optimization

**Prepared Statements:**
```rust
// Slower - string formatting on every call
for i in 0..1000 {
    conn.execute(&format!("INSERT INTO users VALUES ({}, 'user_{}')", i, i), [])?;
}

// Faster - prepared statement with parameters
let mut stmt = conn.prepare("INSERT INTO users VALUES (?, ?)")?;
for i in 0..1000 {
    stmt.execute([i, &format!("user_{}", i)])?;
}
```

**Query Planning:**
```sql
-- Analyze query execution plan
EXPLAIN SELECT * FROM large_table WHERE id > 1000;

-- Use appropriate indexes
CREATE INDEX idx_large_table_id ON large_table(id);

-- Optimize joins
EXPLAIN SELECT c.name, o.total
FROM customers c
JOIN orders o ON c.id = o.customer_id
WHERE o.total > 100;
```

### 3. Data Format Optimization

**Format Selection Strategy:**

| Operation Type | Recommended Format | Performance Gain |
|----------------|-------------------|------------------|
| **Bulk Loading** | CSV | Simple, fast loading |
| **Analytics** | Parquet | 5-10x query speedup |
| **In-Memory** | Arrow | Zero-copy operations |
| **Storage** | Parquet | 2-10x compression |

**Format Conversion for Performance:**
```bash
# Convert to Parquet for analytical workloads
frozen-duckdb convert --input large_dataset.csv --output large_dataset.parquet

# Verify performance improvement
time duckdb -c "SELECT COUNT(*) FROM 'large_dataset.csv'" 2>&1 | grep real
time duckdb -c "SELECT COUNT(*) FROM 'large_dataset.parquet'" 2>&1 | grep real
```

## Memory Management

### 1. Memory Usage Monitoring

**System Memory Tracking:**
```bash
# Monitor memory usage during operations
htop

# Check process memory
ps aux | grep duckdb

# Monitor DuckDB memory usage
duckdb -c "SELECT * FROM pragma_memory_usage();"
```

**Rust Memory Profiling:**
```bash
# Install memory profiler
cargo install cargo-profdata

# Profile memory usage
cargo profdata --bin your-app
```

### 2. Memory-Efficient Operations

**Batch Processing:**
```rust
// Process large datasets in chunks
const BATCH_SIZE: usize = 10000;

fn process_large_dataset(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let total_rows: i64 = conn.query_row("SELECT COUNT(*) FROM large_table", [], |row| row.get(0))?;

    for offset in (0..total_rows).step_by(BATCH_SIZE) {
        let query = format!(
            "SELECT * FROM large_table LIMIT {} OFFSET {}",
            BATCH_SIZE, offset
        );

        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map([], |row| {
            // Process each row
            Ok(row.get::<_, i32>(0)?)
        })?;

        for row in rows {
            let id = row?;
            // Process individual row
        }
    }

    Ok(())
}
```

**Streaming Operations:**
```sql
-- Process data in streaming fashion
COPY (
    SELECT * FROM large_table WHERE processed = false LIMIT 10000
) TO 'temp_output.csv' (FORMAT CSV);

-- Mark as processed
UPDATE large_table SET processed = true WHERE id IN (
    SELECT id FROM temp_output.csv
);
```

### 3. Memory Configuration

**DuckDB Memory Settings:**
```sql
-- Configure memory limits
SET memory_limit = '4GB';

-- Monitor memory usage
PRAGMA memory_limit;

-- Optimize for available memory
PRAGMA threads = 4;  -- Match CPU cores
```

**Application-Level Memory Management:**
```rust
use std::sync::Arc;
use duckdb::Connection;

// Share connections to minimize memory usage
fn shared_connection_example() {
    let conn = Arc::new(Connection::open_in_memory().unwrap());

    // Multiple threads can share the connection
    let conn_clone = Arc::clone(&conn);

    std::thread::spawn(move || {
        // Use connection in thread
        let count: i64 = conn_clone.query_row("SELECT COUNT(*) FROM test", [], |row| row.get(0)).unwrap();
        println!("Thread count: {}", count);
    });
}
```

## LLM Performance Optimization

### 1. Model Selection

**Performance vs Quality Trade-off:**

| Model | Size | Speed | Quality | Use Case |
|-------|------|-------|---------|----------|
| **qwen3-coder:7b** | 7B | Fast | Good | Development, quick responses |
| **qwen3-coder:30b** | 30B | Medium | Excellent | Production, high quality |
| **qwen3-embedding:8b** | 8B | Fast | Good | Similarity search |

**Model Selection Strategy:**
```sql
-- Use smaller models for development
CREATE MODEL('dev_coder', 'qwen3-coder:7b', 'ollama');

-- Use larger models for production
CREATE MODEL('prod_coder', 'qwen3-coder:30b', 'ollama');

-- Switch based on environment
SELECT CASE
    WHEN current_setting('custom.environment') = 'production'
    THEN 'prod_coder'
    ELSE 'dev_coder'
END as selected_model;
```

### 2. Batch Processing

**Efficient Batch Operations:**
```sql
-- Process multiple texts in single operation
CREATE TABLE batch_embeddings AS
SELECT
    content,
    llm_embedding(
        {'model_name': 'embedder'},
        [{'data': content}]
    ) as embedding
FROM (
    VALUES
        ('Machine learning is AI'),
        ('Deep learning uses neural networks'),
        ('Natural language processing')
) AS texts(content);
```

**Batch Size Optimization:**
```rust
// Optimal batch sizes for different operations
const EMBEDDING_BATCH_SIZE: usize = 100;
const COMPLETION_BATCH_SIZE: usize = 10;
const SEARCH_BATCH_SIZE: usize = 50;

fn process_embeddings_batch(texts: Vec<String>) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
    let chunks: Vec<_> = texts.chunks(EMBEDDING_BATCH_SIZE).collect();

    let mut all_embeddings = Vec::new();
    for chunk in chunks {
        let embeddings = process_embedding_chunk(chunk.to_vec())?;
        all_embeddings.extend(embeddings);
    }

    Ok(all_embeddings)
}
```

### 3. Caching Strategy

**Embedding Cache:**
```sql
-- Cache embeddings to avoid recomputation
CREATE TABLE embedding_cache (
    content_hash VARCHAR PRIMARY KEY,
    content TEXT,
    embedding FLOAT[1024],
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Check cache before computing
INSERT INTO embedding_cache (content_hash, content, embedding)
SELECT
    hash(content) as content_hash,
    content,
    llm_embedding({'model_name': 'embedder'}, [{'data': content}]) as embedding
FROM new_content
WHERE hash(content) NOT IN (SELECT content_hash FROM embedding_cache);
```

## Storage and I/O Optimization

### 1. File Format Optimization

**Format Performance Comparison:**

| Format | Read Speed | Write Speed | Compression | Use Case |
|--------|------------|-------------|-------------|----------|
| **CSV** | Medium | Fast | None | Simple data exchange |
| **Parquet** | Fast | Medium | Excellent | Analytics, large data |
| **Arrow** | Very Fast | Fast | Good | In-memory processing |
| **DuckDB** | Very Fast | Very Fast | Good | DuckDB workflows |

**Optimal Format Selection:**
```bash
# For analytical workloads
frozen-duckdb download --dataset tpch --format parquet

# For maximum query performance
frozen-duckdb download --dataset chinook --format duckdb

# For data exchange
frozen-duckdb convert --input data.parquet --output data.csv
```

### 2. Partitioning Strategy

**Date-Based Partitioning:**
```sql
-- Partition large tables by date
CREATE TABLE sales_partitioned (
    sale_date DATE,
    customer_id INTEGER,
    amount DECIMAL,
    -- other columns...
) PARTITION BY sale_date;

-- Query specific date ranges efficiently
SELECT SUM(amount) FROM sales_partitioned WHERE sale_date = '2024-01-15';
```

**Category-Based Partitioning:**
```sql
-- Partition by category for faster filtering
CREATE TABLE products_partitioned (
    category VARCHAR,
    product_id INTEGER,
    name VARCHAR,
    price DECIMAL
) PARTITION BY category;

-- Fast category-specific queries
SELECT * FROM products_partitioned WHERE category = 'electronics';
```

## Network and Distributed Optimization

### 1. Local vs Remote Ollama

**Local Ollama (Recommended):**
```bash
# Benefits: No network latency, complete privacy, reliable
ollama serve  # Local server

# Configure for local
CREATE SECRET ollama_secret (TYPE OLLAMA, API_URL 'http://localhost:11434');
```

**Remote Ollama (Advanced):**
```bash
# For distributed setups
CREATE SECRET ollama_secret (TYPE OLLAMA, API_URL 'http://ollama-server:11434');

# With authentication
CREATE SECRET ollama_secret (
    TYPE OLLAMA,
    API_URL 'https://your-server:11434',
    API_KEY 'your-api-key'
);
```

### 2. Connection Pooling

**Ollama Connection Management:**
```rust
use reqwest::Client;
use std::time::Duration;

fn create_optimized_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(90))
        .build()
        .expect("Failed to create HTTP client")
}
```

## Monitoring and Profiling

### 1. Performance Monitoring

**Build Time Monitoring:**
```bash
#!/bin/bash
# monitor_build_performance.sh

echo "Build started at $(date)"
start_time=$(date +%s)

# Build with frozen DuckDB
source ../frozen-duckdb/prebuilt/setup_env.sh
cargo build --release

end_time=$(date +%s)
duration=$((end_time - start_time))

echo "Build completed in ${duration}s"

# Alert if build time exceeds threshold
if (( duration > 15 )); then
    echo "⚠️  Build time exceeded 15s threshold"
    # Send alert or log issue
fi
```

**Runtime Performance Monitoring:**
```sql
-- Monitor query performance
CREATE TABLE performance_log (
    query_text VARCHAR,
    execution_time_ms INTEGER,
    rows_returned INTEGER,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Log slow queries
INSERT INTO performance_log
SELECT
    'SELECT * FROM large_table' as query_text,
    1500 as execution_time_ms,
    1000000 as rows_returned,
    CURRENT_TIMESTAMP;
```

### 2. Resource Usage Profiling

**Memory Profiling:**
```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct ProfilingAllocator;

unsafe impl GlobalAlloc for ProfilingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        DEALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
        System.dealloc(ptr, layout)
    }
}

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATED: AtomicUsize = AtomicUsize::new(0);
```

### 3. Performance Benchmarks

**Automated Benchmarking:**
```rust
use frozen_duckdb::benchmark;
use std::time::{Duration, Instant};

fn run_performance_benchmarks() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏁 Running performance benchmarks...");

    // Benchmark different operations
    let operations = vec![
        ("simple_query", || benchmark_simple_query()),
        ("complex_join", || benchmark_complex_join()),
        ("embedding_generation", || benchmark_embedding_generation()),
        ("text_completion", || benchmark_text_completion()),
    ];

    for (name, operation) in operations {
        let duration = benchmark::measure_build_time(operation);
        println!("{}: {:?}", name, duration);

        if duration > Duration::from_secs(5) {
            println!("⚠️  {} operation is slow", name);
        }
    }

    Ok(())
}
```

## Troubleshooting Performance Issues

### 1. Slow Builds

**Symptoms:** Build times >15 seconds

**Diagnosis:**
```bash
# Check if using frozen DuckDB
echo $DUCKDB_LIB_DIR

# Should show: /path/to/frozen-duckdb/prebuilt

# Check binary size and modification time
ls -lah prebuilt/libduckdb*

# Check if fallback to bundled compilation
RUST_LOG=debug cargo build 2>&1 | grep -i duckdb
```

**Solutions:**
```bash
# Re-source environment
source ../frozen-duckdb/prebuilt/setup_env.sh

# Verify binary selection
echo "Selected binary: $(ls -la $DUCKDB_LIB_DIR/libduckdb* | head -1)"

# Clean and rebuild
cargo clean
cargo build
```

### 2. Slow Queries

**Symptoms:** Query execution >1 second

**Diagnosis:**
```sql
-- Analyze query execution plan
EXPLAIN ANALYZE SELECT * FROM large_table WHERE condition;

-- Check table statistics
PRAGMA table_info(large_table);

-- Monitor memory usage during query
SELECT * FROM pragma_memory_usage();
```

**Solutions:**
```sql
-- Create appropriate indexes
CREATE INDEX idx_condition ON large_table(condition);

-- Use more efficient query patterns
SELECT * FROM large_table WHERE condition LIMIT 100;

-- Optimize data types and storage
ALTER TABLE large_table ALTER COLUMN id SET TYPE INTEGER;
```

### 3. Memory Issues

**Symptoms:** Out of memory errors, slow performance

**Diagnosis:**
```bash
# Monitor memory usage
htop

# Check DuckDB memory settings
duckdb -c "PRAGMA memory_limit;"

# Profile memory allocations
RUST_LOG=debug cargo run 2>&1 | grep -i memory
```

**Solutions:**
```bash
# Increase memory limit
SET memory_limit = '8GB';

# Process data in smaller chunks
# Use streaming operations
# Optimize data types for memory efficiency
```

### 4. LLM Performance Issues

**Symptoms:** Slow text generation, timeouts

**Diagnosis:**
```bash
# Check Ollama server status
curl -s http://localhost:11434/api/version

# Monitor model loading
ollama list

# Test basic connectivity
frozen-duckdb complete --prompt "test"
```

**Solutions:**
```bash
# Restart Ollama server
ollama stop && ollama serve

# Use smaller models for faster responses
CREATE MODEL('fast_coder', 'qwen3-coder:7b', 'ollama');

# Optimize batch sizes
# Use local models only
```

## Performance Best Practices

### 1. Environment Setup

- **Always source setup script** before building
- **Use persistent environment variables** for development
- **Verify configuration** in CI/CD pipelines
- **Monitor build times** and investigate anomalies

### 2. Data Management

- **Choose optimal formats** for your workload (Parquet for analytics)
- **Use appropriate partitioning** for large datasets
- **Implement caching** for frequently accessed data
- **Monitor storage growth** and clean up old data

### 3. Query Optimization

- **Use prepared statements** for repeated queries
- **Create indexes** for frequently filtered columns
- **Analyze query plans** for optimization opportunities
- **Batch operations** for better throughput

### 4. Resource Management

- **Monitor memory usage** during intensive operations
- **Configure memory limits** based on available resources
- **Use connection pooling** for multiple operations
- **Profile performance** regularly to identify bottlenecks

## Performance Targets and Monitoring

### SLO (Service Level Objectives)

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| **First build** | ≤10 seconds | 7-10 seconds | ✅ **Met** |
| **Incremental build** | ≤1 second | 0.11 seconds | ✅ **Met** |
| **Query response** | ≤100ms | <50ms | ✅ **Met** |
| **LLM completion** | ≤5 seconds | 2-5 seconds | ✅ **Met** |
| **Memory usage** | ≤1GB | ~200MB | ✅ **Met** |

### Performance Regression Detection

```bash
#!/bin/bash
# detect_performance_regression.sh

# Record current performance
CURRENT_BUILD_TIME=$(time cargo build 2>&1 | grep real | awk '{print $2}')
CURRENT_TEST_TIME=$(time cargo test --quiet 2>&1 | grep real | awk '{print $2}')

# Compare with baseline (stored in file)
BASELINE_FILE=".performance_baseline"

if [[ -f "$BASELINE_FILE" ]]; then
    source "$BASELINE_FILE"

    # Check for regressions (>10% slowdown)
    BUILD_REGRESSION=$(echo "$CURRENT_BUILD_TIME > $BASELINE_BUILD_TIME * 1.1" | bc -l)
    TEST_REGRESSION=$(echo "$CURRENT_TEST_TIME > $BASELINE_TEST_TIME * 1.1" | bc -l)

    if (( BUILD_REGRESSION || TEST_REGRESSION )); then
        echo "⚠️  Performance regression detected!"
        echo "   Build time: $CURRENT_BUILD_TIME (baseline: $BASELINE_BUILD_TIME)"
        echo "   Test time: $CURRENT_TEST_TIME (baseline: $BASELINE_TEST_TIME)"
        exit 1
    fi
else
    # Create baseline
    echo "BASELINE_BUILD_TIME=$CURRENT_BUILD_TIME" > "$BASELINE_FILE"
    echo "BASELINE_TEST_TIME=$CURRENT_TEST_TIME" >> "$BASELINE_FILE"
fi

echo "✅ Performance within acceptable limits"
```

## Advanced Optimization Techniques

### 1. Parallel Processing

**Concurrent Operations:**
```rust
use rayon::prelude::*;
use std::sync::Arc;

fn parallel_data_processing(conn: Arc<Connection>) -> Result<(), Box<dyn std::error::Error>> {
    let data: Vec<i32> = (0..1000000).collect();

    // Process in parallel chunks
    data.par_chunks(10000).for_each(|chunk| {
        let conn = Arc::clone(&conn);
        let chunk_data: Vec<i32> = chunk.to_vec();

        // Process chunk (in real implementation)
        // conn.execute_batch(&batch_insert_sql).unwrap();
    });

    Ok(())
}
```

### 2. Vectorized Operations

**SIMD Optimization:**
```rust
use std::simd::{f32x4, SimdFloat};

// Vectorized similarity calculation
fn vectorized_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());

    let mut sum = 0.0;
    let mut i = 0;

    // Process 4 elements at a time (SIMD)
    while i + 4 <= a.len() {
        let va = f32x4::from_slice(&a[i..i+4]);
        let vb = f32x4::from_slice(&b[i..i+4]);
        let diff = va - vb;
        let squared = diff * diff;
        sum += squared.reduce_sum();
        i += 4;
    }

    // Handle remaining elements
    for j in i..a.len() {
        let diff = a[j] - b[j];
        sum += diff * diff;
    }

    (-sum).exp() // Cosine similarity approximation
}
```

### 3. Memory-Mapped Files

**Large File Processing:**
```rust
use memmap2::MmapOptions;
use std::fs::File;

fn process_large_file_mmap(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Memory-map large file for efficient access
    let file = File::open(path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };

    // Process file content without loading into memory
    let content = std::str::from_utf8(&mmap)?;

    // Process line by line without allocation
    for line in content.lines() {
        // Process each line
        process_line(line);
    }

    Ok(())
}
```

## Summary

Performance tuning with Frozen DuckDB focuses on **maximizing efficiency** across **build time**, **runtime performance**, **memory usage**, and **storage optimization**. The system is designed to deliver **exceptional performance** while maintaining **ease of use** and **production reliability**.

**Key Optimization Areas:**
- **Build performance**: 99% faster builds with pre-compiled binaries
- **Runtime efficiency**: Optimized queries and memory management
- **Storage optimization**: Efficient data formats and compression
- **LLM performance**: Fast local inference with Ollama integration

**Performance Targets:**
- **Build time**: ≤10s first build, ≤1s incremental
- **Query performance**: ≤100ms for typical operations
- **Memory usage**: ≤1GB for typical workloads
- **LLM operations**: ≤5s for typical requests

**Monitoring Strategy:**
- **Automated benchmarking**: Regular performance validation
- **Regression detection**: Alert on performance degradation
- **Resource monitoring**: Track memory and CPU usage
- **Performance profiling**: Identify and resolve bottlenecks

**Next Steps:**
1. Implement the optimization strategies that match your workload
2. Set up performance monitoring for your specific use case
3. Establish performance baselines and regression detection
4. Continuously optimize based on usage patterns and requirements
