# Dataset Operations Guide

## Overview

Dataset operations in Frozen DuckDB provide **comprehensive data management** capabilities including **sample dataset generation**, **format conversion**, and **TPC-H benchmark data** for testing, development, and performance evaluation.

## Sample Dataset Generation

### Chinook Music Database

The **Chinook dataset** is a comprehensive sample database that simulates a digital music store with realistic data relationships.

#### Database Schema

```sql
-- Core tables with relationships
CREATE TABLE artists (
    ArtistId INTEGER PRIMARY KEY,
    Name TEXT NOT NULL
);

CREATE TABLE albums (
    AlbumId INTEGER PRIMARY KEY,
    Title TEXT NOT NULL,
    ArtistId INTEGER REFERENCES artists(ArtistId)
);

CREATE TABLE tracks (
    TrackId INTEGER PRIMARY KEY,
    Name TEXT NOT NULL,
    AlbumId INTEGER REFERENCES albums(AlbumId),
    MediaTypeId INTEGER,
    GenreId INTEGER,
    Composer TEXT,
    Milliseconds INTEGER,
    Bytes INTEGER,
    UnitPrice DECIMAL(10,2)
);

CREATE TABLE genres (
    GenreId INTEGER PRIMARY KEY,
    Name TEXT NOT NULL
);

CREATE TABLE media_types (
    MediaTypeId INTEGER PRIMARY KEY,
    Name TEXT NOT NULL
);
```

#### Data Characteristics

| Table | Records | Size | Description |
|-------|---------|------|-------------|
| **artists** | 275 | ~10KB | Music artists and bands |
| **albums** | 347 | ~15KB | Album information |
| **tracks** | 3,503 | ~200KB | Individual songs |
| **genres** | 25 | ~1KB | Music categories |
| **media_types** | 5 | ~1KB | Audio format types |

#### Generation Commands

```bash
# CSV format (human-readable)
frozen-duckdb download --dataset chinook --format csv

# Parquet format (analytics-optimized)
frozen-duckdb download --dataset chinook --format parquet --output-dir ./analytics

# DuckDB native format (maximum performance)
frozen-duckdb download --dataset chinook --format duckdb
```

**Generated File Structure:**
```
datasets/
├── chinook.csv       # CSV format - human readable
├── chinook.parquet   # Parquet format - columnar storage
└── chinook.duckdb    # DuckDB format - native performance
```

#### Usage Examples

```sql
-- Query artists by genre
SELECT a.Name as Artist, g.Name as Genre, COUNT(t.TrackId) as Tracks
FROM artists a
JOIN albums al ON a.ArtistId = al.ArtistId
JOIN tracks t ON al.AlbumId = t.AlbumId
JOIN genres g ON t.GenreId = g.GenreId
WHERE g.Name = 'Rock'
GROUP BY a.Name, g.Name
ORDER BY Tracks DESC
LIMIT 10;

-- Find most expensive tracks
SELECT Name, Composer, UnitPrice
FROM tracks
WHERE UnitPrice > 0.99
ORDER BY UnitPrice DESC;

-- Analyze sales by media type
SELECT mt.Name as MediaType, SUM(t.UnitPrice) as TotalSales
FROM tracks t
JOIN media_types mt ON t.MediaTypeId = mt.MediaTypeId
GROUP BY mt.Name
ORDER BY TotalSales DESC;
```

### TPC-H Decision Support Benchmark

The **TPC-H dataset** is an industry-standard benchmark for decision support systems that simulates complex business operations.

#### Schema Overview

```sql
-- Customer relationship management
CREATE TABLE customer (
    c_custkey INTEGER PRIMARY KEY,
    c_name TEXT,
    c_address TEXT,
    c_nationkey INTEGER,
    c_phone TEXT,
    c_acctbal DECIMAL,
    c_mktsegment TEXT,
    c_comment TEXT
);

-- Order management
CREATE TABLE orders (
    o_orderkey INTEGER PRIMARY KEY,
    o_custkey INTEGER REFERENCES customer(c_custkey),
    o_orderstatus TEXT,
    o_totalprice DECIMAL,
    o_orderdate DATE,
    o_orderpriority TEXT,
    o_clerk TEXT,
    o_shippriority INTEGER,
    o_comment TEXT
);

-- Order line items
CREATE TABLE lineitem (
    l_orderkey INTEGER REFERENCES orders(o_orderkey),
    l_partkey INTEGER,
    l_suppkey INTEGER,
    l_linenumber INTEGER,
    l_quantity DECIMAL,
    l_extendedprice DECIMAL,
    l_discount DECIMAL,
    l_tax DECIMAL,
    l_returnflag TEXT,
    l_linestatus TEXT,
    l_shipdate DATE,
    l_commitdate DATE,
    l_receiptdate DATE,
    l_shipinstruct TEXT,
    l_shipmode TEXT,
    l_comment TEXT,
    PRIMARY KEY (l_orderkey, l_linenumber)
);

-- Supplier and parts data
CREATE TABLE supplier (s_suppkey INTEGER PRIMARY KEY, s_name TEXT, s_address TEXT, s_nationkey INTEGER, s_phone TEXT, s_acctbal DECIMAL, s_comment TEXT);
CREATE TABLE part (p_partkey INTEGER PRIMARY KEY, p_name TEXT, p_mfgr TEXT, p_brand TEXT, p_type TEXT, p_size INTEGER, p_container TEXT, p_retailprice DECIMAL, p_comment TEXT);
CREATE TABLE partsupp (ps_partkey INTEGER REFERENCES part(p_partkey), ps_suppkey INTEGER REFERENCES supplier(s_suppkey), ps_availqty INTEGER, ps_supplycost DECIMAL, ps_comment TEXT, PRIMARY KEY (ps_partkey, ps_suppkey));
CREATE TABLE nation (n_nationkey INTEGER PRIMARY KEY, n_name TEXT, n_regionkey INTEGER, n_comment TEXT);
CREATE TABLE region (r_regionkey INTEGER PRIMARY KEY, r_name TEXT, r_comment TEXT);
```

#### Scale Factors

| Scale Factor | Tables | Rows | Storage | Generation Time |
|-------------|--------|------|---------|-----------------|
| **SF 0.01** | 8 | ~19,000 | 1-5MB | <10 seconds |
| **SF 0.1** | 8 | ~190,000 | 10-50MB | <30 seconds |
| **SF 1.0** | 8 | ~1.9M | 100-500MB | 1-2 minutes |

#### Generation Commands

```bash
# Tiny dataset for testing (recommended)
frozen-duckdb download --dataset tpch --format parquet

# Small dataset for development
frozen-duckdb download --dataset tpch --format parquet --output-dir ./dev_data

# Native DuckDB format for maximum performance
frozen-duckdb download --dataset tpch --format duckdb
```

#### TPC-H Query Examples

**Query 1: Pricing Summary Report**
```sql
SELECT
    l_returnflag,
    l_linestatus,
    SUM(l_quantity) as sum_qty,
    SUM(l_extendedprice) as sum_base_price,
    SUM(l_extendedprice * (1 - l_discount)) as sum_disc_price,
    SUM(l_extendedprice * (1 - l_discount) * (1 + l_tax)) as sum_charge,
    AVG(l_quantity) as avg_qty,
    AVG(l_extendedprice) as avg_price,
    AVG(l_discount) as avg_disc,
    COUNT(*) as count_order
FROM lineitem
WHERE l_shipdate <= DATE '1998-09-02'
GROUP BY l_returnflag, l_linestatus
ORDER BY l_returnflag, l_linestatus;
```

**Query 3: Shipping Priority**
```sql
SELECT
    l_orderkey,
    SUM(l_extendedprice * (1 - l_discount)) as revenue,
    o_orderdate,
    o_shippriority
FROM customer, orders, lineitem
WHERE c_mktsegment = 'BUILDING'
    AND c_custkey = o_custkey
    AND l_orderkey = o_orderkey
    AND o_orderdate < DATE '1995-03-15'
    AND l_shipdate > DATE '1995-03-15'
GROUP BY l_orderkey, o_orderdate, o_shippriority
ORDER BY revenue DESC, o_orderdate
LIMIT 10;
```

**Query 6: Forecast Revenue Change**
```sql
SELECT
    SUM(l_extendedprice * l_discount) as revenue
FROM lineitem
WHERE l_shipdate >= DATE '1994-01-01'
    AND l_shipdate < DATE '1995-01-01'
    AND l_discount BETWEEN 0.05 AND 0.07
    AND l_quantity < 24;
```

## Format Conversion Operations

### Supported Formats

| Format | Description | Use Case | Performance |
|--------|-------------|----------|-------------|
| **CSV** | Comma-separated values | Human-readable, universal | Good for small data |
| **Parquet** | Columnar storage | Analytics, compression | Best for large data |
| **JSON** | JavaScript Object Notation | Web APIs, structured | Good for complex data |
| **Arrow** | Apache Arrow format | High-performance interchange | Best for in-memory |
| **DuckDB** | Native DuckDB format | Maximum query performance | Best for DuckDB workflows |

### Conversion Performance

| Conversion | Time | Size Reduction | Query Speedup |
|------------|------|---------------|---------------|
| **CSV → Parquet** | <5s | 50-90% smaller | 5-10x faster |
| **Parquet → CSV** | <3s | N/A | Human readable |
| **CSV → DuckDB** | <2s | 30-50% smaller | 10-20x faster |
| **Parquet → DuckDB** | <1s | Minimal | Maximum performance |

### Conversion Examples

#### CSV to Parquet (Recommended for Analytics)

```bash
# Convert single file
frozen-duckdb convert --input customer_data.csv --output customer_data.parquet

# Convert with explicit format specification
frozen-duckdb convert \
  --input sales_data.csv \
  --output sales_data.parquet \
  --input-format csv \
  --output-format parquet

# Batch conversion
for file in *.csv; do
    frozen-duckdb convert --input "$file" --output "${file%.csv}.parquet"
done
```

#### Parquet to DuckDB (Maximum Performance)

```bash
# Convert for maximum query performance
frozen-duckdb convert --input analytics.parquet --output analytics.duckdb

# Use in DuckDB for best performance
duckdb analytics.duckdb -c "
SELECT COUNT(*) FROM customer WHERE c_acctbal > 1000;
"
```

#### Batch Processing

```bash
#!/bin/bash
# convert_all_to_parquet.sh

echo "🔄 Converting all CSV files to Parquet..."

for file in datasets/*.csv; do
    parquet_file="${file%.csv}.parquet"
    echo "Converting $file → $parquet_file"

    frozen-duckdb convert --input "$file" --output "$parquet_file"

    # Verify conversion
    if [[ -f "$parquet_file" ]]; then
        echo "✅ Converted $(basename "$file")"
    else
        echo "❌ Failed to convert $(basename "$file")"
    fi
done

echo "🎉 Conversion complete!"
```

## Advanced Dataset Operations

### Custom Dataset Creation

```sql
-- Create derived dataset with transformations
CREATE TABLE customer_analytics AS
SELECT
    c.c_custkey,
    c.c_name,
    c.c_acctbal,
    n.n_name as nation,
    r.r_name as region,
    CASE
        WHEN c.c_acctbal > 5000 THEN 'high_value'
        WHEN c.c_acctbal > 0 THEN 'regular'
        ELSE 'negative'
    END as customer_segment,
    -- Calculate order statistics
    (SELECT COUNT(*) FROM orders o WHERE o.o_custkey = c.c_custkey) as order_count,
    (SELECT SUM(o_totalprice) FROM orders o WHERE o.o_custkey = c.c_custkey) as total_spent
FROM customer c
JOIN nation n ON c.c_nationkey = n.n_nationkey
JOIN region r ON n.n_regionkey = r.r_regionkey;

-- Export in multiple formats
COPY customer_analytics TO 'customer_analytics.csv' (FORMAT CSV, HEADER);
COPY customer_analytics TO 'customer_analytics.parquet' (FORMAT PARQUET);
```

### Dataset Validation

```sql
-- Validate data integrity
SELECT
    'customer' as table_name,
    COUNT(*) as row_count,
    COUNT(DISTINCT c_custkey) as unique_keys,
    MIN(c_acctbal) as min_balance,
    MAX(c_acctbal) as max_balance,
    AVG(c_acctbal) as avg_balance
FROM customer

UNION ALL

SELECT
    'orders' as table_name,
    COUNT(*) as row_count,
    COUNT(DISTINCT o_orderkey) as unique_keys,
    MIN(o_totalprice) as min_total,
    MAX(o_totalprice) as max_total,
    AVG(o_totalprice) as avg_total
FROM orders

ORDER BY table_name;
```

### Performance Benchmarking

```sql
-- Compare query performance across formats
CREATE TABLE performance_results AS

-- CSV performance
SELECT 'CSV' as format, 'count' as operation,
       (SELECT COUNT(*) FROM 'datasets/customer.csv') as result,
       'customer' as table_name

UNION ALL

SELECT 'CSV' as format, 'aggregate' as operation,
       (SELECT AVG(c_acctbal) FROM 'datasets/customer.csv') as result,
       'customer' as table_name

UNION ALL

-- Parquet performance
SELECT 'Parquet' as format, 'count' as operation,
       (SELECT COUNT(*) FROM 'datasets/customer.parquet') as result,
       'customer' as table_name

UNION ALL

SELECT 'Parquet' as format, 'aggregate' as operation,
       (SELECT AVG(c_acctbal) FROM 'datasets/customer.parquet') as result,
       'customer' as table_name;
```

## Integration Examples

### Rust Application Integration

```rust
use duckdb::Connection;
use std::process::Command;

fn setup_test_environment() -> Result<(), Box<dyn std::error::Error>> {
    // Generate test datasets
    let output = Command::new("frozen-duckdb")
        .args(&["download", "--dataset", "tpch", "--format", "parquet"])
        .output()?;

    if !output.status.success() {
        return Err(format!("Dataset generation failed: {}",
                          String::from_utf8_lossy(&output.stderr)).into());
    }

    // Connect to generated data
    let conn = Connection::open("datasets/tpch.duckdb")?;

    // Run analytical queries
    let result: f64 = conn.query_row(
        "SELECT AVG(c_acctbal) FROM customer WHERE c_nationkey = 1",
        [],
        |row| row.get(0),
    )?;

    println!("Average account balance for nation 1: {:.2}", result);
    Ok(())
}
```

### Python Data Pipeline

```python
import subprocess
import duckdb
import pandas as pd

def create_analytics_pipeline():
    # Generate source data
    subprocess.run([
        "frozen-duckdb", "download",
        "--dataset", "chinook",
        "--format", "parquet",
        "--output-dir", "source_data"
    ], check=True)

    # Load data for analysis
    con = duckdb.connect("source_data/chinook.duckdb")

    # Perform analysis
    df = con.execute("""
        SELECT
            g.Name as genre,
            COUNT(t.TrackId) as track_count,
            AVG(t.UnitPrice) as avg_price,
            SUM(t.UnitPrice) as total_revenue
        FROM tracks t
        JOIN genres g ON t.GenreId = g.GenreId
        GROUP BY g.Name
        ORDER BY total_revenue DESC
    """).fetchdf()

    # Export results
    df.to_csv("genre_analysis.csv", index=False)
    df.to_parquet("genre_analysis.parquet")

    return df

# Run pipeline
results = create_analytics_pipeline()
print(f"Analyzed {len(results)} genres")
```

### ETL Workflow

```bash
#!/bin/bash
# etl_workflow.sh

echo "🚀 Starting ETL workflow..."

# 1. Extract: Generate source data
echo "📥 Extracting data..."
frozen-duckdb download --dataset tpch --format csv --output-dir ./extracted

# 2. Transform: Clean and convert data
echo "🔄 Transforming data..."
frozen-duckdb convert --input ./extracted/customer.csv --output ./transformed/customer.parquet

# 3. Load: Import into target system
echo "📤 Loading data..."
duckdb target.duckdb -c "
CREATE TABLE customers AS SELECT * FROM './transformed/customer.parquet';
CREATE INDEX idx_customer_balance ON customers(c_acctbal);
"

# 4. Validate: Check data quality
echo "✅ Validating data..."
duckdb target.duckdb -c "
SELECT COUNT(*) as total_customers FROM customers;
SELECT AVG(c_acctbal) as avg_balance FROM customers;
"

echo "🎉 ETL workflow complete!"
```

## Performance Optimization

### Format Selection Strategy

**For Development and Testing:**
```bash
# Use CSV for easy inspection and editing
frozen-duckdb download --dataset chinook --format csv
```

**For Analytics Workloads:**
```bash
# Use Parquet for columnar storage and compression
frozen-duckdb download --dataset tpch --format parquet
```

**For Production Queries:**
```bash
# Use DuckDB native format for maximum performance
frozen-duckdb download --dataset tpch --format duckdb
```

### Batch Processing Optimization

**Large Dataset Handling:**
```sql
-- Process large datasets in chunks to manage memory
CREATE TEMP TABLE chunk AS
SELECT * FROM large_dataset
WHERE processed = false
ORDER BY id
LIMIT 50000;

-- Process current chunk
UPDATE large_dataset
SET processed = true
WHERE id IN (SELECT id FROM chunk);

-- Clear chunk for next batch
DELETE FROM chunk;
```

**Parallel Processing:**
```rust
use rayon::prelude::*;
use std::sync::Arc;

fn parallel_data_processing(conn: Arc<Connection>) -> Result<(), Box<dyn std::error::Error>> {
    let data_chunks: Vec<Vec<String>> = chunk_data_into_batches(data, 1000);

    data_chunks.par_iter().for_each(|chunk| {
        // Process each chunk in parallel
        process_data_chunk(&conn, chunk).unwrap();
    });

    Ok(())
}
```

## Monitoring and Maintenance

### Dataset Health Monitoring

```sql
-- Monitor dataset statistics
CREATE TABLE dataset_health AS
SELECT
    'customer' as table_name,
    COUNT(*) as row_count,
    COUNT(DISTINCT c_custkey) as unique_customers,
    MIN(c_acctbal) as min_balance,
    MAX(c_acctbal) as max_balance,
    AVG(c_acctbal) as avg_balance,
    CURRENT_TIMESTAMP as check_time
FROM customer

UNION ALL

SELECT
    'orders' as table_name,
    COUNT(*) as row_count,
    COUNT(DISTINCT o_orderkey) as unique_orders,
    MIN(o_totalprice) as min_order,
    MAX(o_totalprice) as max_order,
    AVG(o_totalprice) as avg_order,
    CURRENT_TIMESTAMP as check_time
FROM orders;
```

### Storage Management

**File Size Monitoring:**
```bash
#!/bin/bash
# monitor_dataset_sizes.sh

echo "📊 Dataset storage usage:"
echo "========================="

# Show file sizes
ls -lah datasets/* | grep -E "\.(csv|parquet|duckdb)$"

# Calculate total size
TOTAL_SIZE=$(du -sh datasets/ | cut -f1)
echo "Total dataset size: $TOTAL_SIZE"

# Check for old/large files
find datasets/ -type f -mtime +30 -exec ls -lh {} \;
```

**Cleanup Strategy:**
```bash
# Remove temporary files
find datasets/ -name "*.tmp" -delete

# Archive old datasets
for file in datasets/*.parquet; do
    if [[ $(stat -f "%Sm" -t "%Y%m%d" "$file") -lt $(date -v-30d +%Y%m%d) ]]; then
        gzip "$file"
    fi
done

# Remove duplicate formats (keep Parquet only)
find datasets/ -name "*.csv" -exec rm {} \;
```

## Troubleshooting

### Common Issues

#### 1. Generation Fails

**Error:** `Dataset generation failed`

**Troubleshooting:**
```bash
# Check available disk space
df -h

# Check permissions
ls -la datasets/

# Try different output format
frozen-duckdb download --dataset chinook --format csv

# Check DuckDB installation
duckdb -c "SELECT version();"
```

#### 2. Format Conversion Issues

**Error:** `Conversion failed`

**Troubleshooting:**
```bash
# Verify input file exists
ls -la input_file.csv

# Check output directory permissions
mkdir -p output_dir && touch output_dir/test

# Try explicit format specification
frozen-duckdb convert --input file.csv --output file.parquet --input-format csv --output-format parquet

# Check for special characters
ls -la "file with spaces.csv"
```

#### 3. Performance Issues

**Error:** Queries running slowly

**Troubleshooting:**
```sql
-- Check query execution plan
EXPLAIN ANALYZE SELECT * FROM large_table WHERE condition;

-- Create appropriate indexes
CREATE INDEX idx_condition ON large_table(condition);

-- Use more efficient formats
# Convert to Parquet for better performance
frozen-duckdb convert --input large_table.csv --output large_table.parquet
```

## Best Practices

### 1. Format Selection

- **CSV**: Human-readable data, simple imports, small datasets
- **Parquet**: Analytics workloads, large datasets, compression
- **DuckDB**: Maximum query performance, DuckDB workflows
- **JSON**: Structured data exchange, web APIs

### 2. Storage Organization

```
datasets/
├── raw/              # Original data files
├── processed/        # Cleaned and transformed data
├── analytics/        # Aggregated data for analysis
├── benchmarks/       # Performance test datasets
└── archives/         # Compressed historical data
```

### 3. Performance Optimization

- **Generate appropriate scale factors** for testing needs
- **Use columnar formats** for analytical queries
- **Partition large datasets** by date or category
- **Index frequently queried columns**

### 4. Data Quality

- **Validate generated data** for completeness
- **Check referential integrity** in relational datasets
- **Test queries** on sample data before full processing
- **Document data sources** and generation parameters

## Summary

Dataset operations with Frozen DuckDB provide **comprehensive data management** capabilities with **industry-standard datasets**, **multiple format options**, and **performance optimization**. The system supports **real-world testing scenarios** with **realistic data relationships** and **scalable performance**.

**Key Features:**
- **Industry-standard datasets**: Chinook music database and TPC-H benchmark
- **Multiple format support**: CSV, Parquet, JSON, Arrow, DuckDB native
- **Performance optimization**: Format-specific optimizations for different use cases
- **Integration ready**: Works with Rust, Python, Node.js, and shell scripts

**Performance Benefits:**
- **Fast generation**: <10 seconds for typical datasets
- **Efficient storage**: 50-90% size reduction with Parquet
- **Query performance**: 5-20x faster queries with optimized formats
- **Memory efficiency**: Columnar storage reduces memory usage

**Use Cases:**
- **Development testing**: Quick sample data for application testing
- **Performance benchmarking**: Industry-standard TPC-H benchmarks
- **Analytics prototyping**: Realistic data for analytical queries
- **ETL pipeline testing**: Complex relational data for pipeline validation
