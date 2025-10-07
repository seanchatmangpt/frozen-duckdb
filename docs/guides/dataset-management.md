# Dataset Management Guide

## Overview

The Frozen DuckDB CLI provides **comprehensive dataset management** capabilities including **sample data generation**, **format conversion**, and **TPC-H benchmark datasets** for testing and development.

## Sample Dataset Generation

### Chinook Database

The **Chinook dataset** is a sample music database that contains information about artists, albums, tracks, and sales.

#### Generate Chinook Dataset

```bash
# Generate in CSV format (default)
frozen-duckdb download --dataset chinook --format csv

# Generate in Parquet format (recommended for analytics)
frozen-duckdb download --dataset chinook --format parquet --output-dir ./data

# Generate in DuckDB native format (fastest queries)
frozen-duckdb download --dataset chinook --format duckdb
```

**Generated Files:**
```
datasets/
├── chinook.csv      # CSV format (human-readable)
├── chinook.parquet  # Parquet format (columnar, compressed)
└── chinook.duckdb   # DuckDB native format (fastest)
```

#### Chinook Schema

| Table | Description | Rows | Size |
|-------|-------------|------|------|
| **artists** | Music artists | ~275 | ~10KB |
| **albums** | Albums by artists | ~347 | ~15KB |
| **tracks** | Individual songs | ~3,503 | ~200KB |
| **genres** | Music genres | ~25 | ~1KB |
| **media_types** | Media format types | ~5 | ~1KB |
| **playlists** | Music playlists | ~14 | ~5KB |
| **playlist_tracks** | Playlist contents | ~8,717 | ~100KB |

### TPC-H Benchmark Dataset

The **TPC-H dataset** is an industry-standard benchmark for decision support systems that simulates a business environment with customers, suppliers, parts, and orders.

#### Generate TPC-H Dataset

```bash
# Generate small dataset (SF 0.01) - recommended for testing
frozen-duckdb download --dataset tpch --format parquet

# Generate in specific location
frozen-duckdb download --dataset tpch --format csv --output-dir ./benchmark_data

# Generate in DuckDB native format for maximum performance
frozen-duckdb download --dataset tpch --format duckdb
```

**Scale Factors Available:**
- **SF 0.01** (Tiny): ~19,000 rows across 8 tables - perfect for testing
- **SF 0.1** (Small): ~190,000 rows - good for development
- **SF 1.0** (Standard): ~1.9M rows - for performance testing

#### TPC-H Schema

| Table | Description | SF 0.01 Rows | Purpose |
|-------|-------------|--------------|---------|
| **customer** | Customer information | ~1,500 | Customer analytics |
| **lineitem** | Order line items | ~6,000 | Order processing |
| **nation** | Country information | ~25 | Geographic data |
| **orders** | Customer orders | ~1,500 | Order management |
| **part** | Parts catalog | ~2,000 | Inventory management |
| **partsupp** | Part-supplier relationships | ~8,000 | Supply chain |
| **region** | Geographic regions | ~5 | Regional analysis |
| **supplier** | Supplier information | ~100 | Supplier management |

**TPC-H Queries Supported:**
- **Complex joins** across multiple tables
- **Aggregation operations** with grouping
- **Subquery performance** testing
- **Date range filtering** and time-series analysis

## Format Conversion

### Supported Formats

| Format | Description | Use Case | Performance |
|--------|-------------|----------|-------------|
| **CSV** | Comma-separated values | Human-readable, universal | Good for small data |
| **Parquet** | Columnar storage | Analytics, compression | Best for large data |
| **JSON** | JavaScript Object Notation | Web APIs, structured data | Good for complex data |
| **Arrow** | Apache Arrow format | High-performance interchange | Best for in-memory analytics |
| **DuckDB** | Native DuckDB format | Maximum query performance | Best for DuckDB workflows |

### Conversion Examples

#### CSV to Parquet (Recommended for Analytics)

```bash
# Convert CSV to Parquet for better performance
frozen-duckdb convert --input data.csv --output data.parquet

# With explicit format specification
frozen-duckdb convert \
  --input customer_data.csv \
  --output customer_data.parquet \
  --input-format csv \
  --output-format parquet
```

**Benefits of Parquet:**
- **Columnar storage**: Only read needed columns
- **Compression**: Typically 2-10x smaller than CSV
- **Performance**: 5-10x faster queries for analytical workloads
- **Schema evolution**: Supports adding/removing columns

#### Parquet to CSV (For Human Analysis)

```bash
# Convert Parquet back to CSV
frozen-duckdb convert --input data.parquet --output data.csv

# Include headers in output
frozen-duckdb convert \
  --input analytics_data.parquet \
  --output report.csv \
  --input-format parquet \
  --output-format csv
```

#### Batch Conversion

```bash
#!/bin/bash
# convert_all_csv_to_parquet.sh

# Find all CSV files and convert to Parquet
for file in *.csv; do
    parquet_file="${file%.csv}.parquet"
    echo "Converting $file to $parquet_file"
    frozen-duckdb convert --input "$file" --output "$parquet_file"
done

echo "✅ Converted $(ls *.csv | wc -l) files to Parquet"
```

## Advanced Dataset Operations

### Custom Dataset Creation

```sql
-- Create custom dataset using DuckDB
CREATE TABLE custom_data AS
SELECT
    id,
    name,
    value * 1.1 as adjusted_value,
    CASE
        WHEN value > 100 THEN 'high'
        WHEN value > 50 THEN 'medium'
        ELSE 'low'
    END as category
FROM source_data;

-- Export in multiple formats
COPY custom_data TO 'custom.csv' (FORMAT CSV, HEADER);
COPY custom_data TO 'custom.parquet' (FORMAT PARQUET);
```

### Dataset Validation

```bash
#!/bin/bash
# validate_dataset.sh

DATASET_DIR="datasets"
DATASET_NAME="tpch"

echo "🔍 Validating $DATASET_NAME dataset..."

# Check file existence
if [[ ! -d "$DATASET_DIR" ]]; then
    echo "❌ Dataset directory not found: $DATASET_DIR"
    exit 1
fi

# Validate file sizes
echo "📊 File sizes:"
ls -lah "$DATASET_DIR"/${DATASET_NAME}.*

# Test data integrity
echo "🧪 Testing data integrity..."
duckdb -c "
SELECT
    'customer' as table_name, COUNT(*) as rows FROM '$DATASET_DIR/customer.parquet'
UNION ALL
SELECT 'orders', COUNT(*) FROM '$DATASET_DIR/orders.parquet'
UNION ALL
SELECT 'lineitem', COUNT(*) FROM '$DATASET_DIR/lineitem.parquet'
ORDER BY table_name;
"

echo "✅ Dataset validation complete"
```

### Dataset Performance Benchmarking

```bash
#!/bin/bash
# benchmark_dataset.sh

echo "🏁 Benchmarking dataset performance..."

# Test query performance on different formats
echo "📊 Query performance comparison:"

# CSV format
time duckdb -c "
SELECT COUNT(*) FROM 'datasets/tpch/customer.csv';
SELECT AVG(c_acctbal) FROM 'datasets/tpch/customer.csv';
" 2>&1 | grep real

# Parquet format
time duckdb -c "
SELECT COUNT(*) FROM 'datasets/tpch/customer.parquet';
SELECT AVG(c_acctbal) FROM 'datasets/tpch/customer.parquet';
" 2>&1 | grep real

# DuckDB native format
time duckdb -c "
SELECT COUNT(*) FROM 'datasets/tpch/customer.duckdb';
SELECT AVG(c_acctbal) FROM 'datasets/tpch/customer.duckdb';
" 2>&1 | grep real
```

## Integration with Applications

### Rust Integration

```rust
use duckdb::Connection;
use frozen_duckdb::cli::{DatasetManager, Commands};

fn setup_test_data() -> Result<(), Box<dyn std::error::Error>> {
    let dataset_manager = DatasetManager::new()?;

    // Generate TPC-H test data
    dataset_manager.download_tpch("test_datasets", "parquet")?;

    // Use in application
    let conn = Connection::open("test_datasets/tpch.duckdb")?;

    // Run analytical queries
    let result: f64 = conn.query_row(
        "SELECT AVG(c_acctbal) FROM customer WHERE c_nationkey = 1",
        [],
        |row| row.get(0),
    )?;

    println!("Average account balance: {:.2}", result);
    Ok(())
}
```

### Python Integration

```python
import subprocess
import duckdb

def setup_datasets():
    # Generate test data using CLI
    subprocess.run([
        "frozen-duckdb", "download",
        "--dataset", "tpch",
        "--format", "parquet",
        "--output-dir", "datasets"
    ], check=True)

    # Use in Python
    con = duckdb.connect("datasets/tpch.duckdb")

    # Run analytical queries
    result = con.execute("""
        SELECT
            n_name,
            COUNT(*) as customer_count,
            AVG(c_acctbal) as avg_balance
        FROM customer c
        JOIN nation n ON c.c_nationkey = n.n_nationkey
        GROUP BY n_name
        ORDER BY customer_count DESC
    """).fetchall()

    print("Customer analysis by nation:")
    for row in result:
        print(f"  {row[0]}: {row[1]} customers, avg balance ${row[2]:.2".2f"
```

### Node.js Integration

```javascript
const { spawn } = require('child_process');
const DuckDB = require('duckdb');

function setupDatasets() {
    // Generate test data
    return new Promise((resolve, reject) => {
        const child = spawn('frozen-duckdb', [
            'download',
            '--dataset', 'chinook',
            '--format', 'parquet',
            '--output-dir', 'datasets'
        ]);

        child.on('close', (code) => {
            if (code === 0) {
                resolve();
            } else {
                reject(new Error(`Dataset generation failed with code ${code}`));
            }
        });
    });
}

// Use generated data
async function analyzeData() {
    await setupDatasets();

    const db = new DuckDB.Database('datasets/chinook.duckdb');

    return new Promise((resolve, reject) => {
        db.all(`
            SELECT
                g.Name as genre,
                COUNT(t.TrackId) as track_count,
                AVG(t.UnitPrice) as avg_price
            FROM tracks t
            JOIN genres g ON t.GenreId = g.GenreId
            GROUP BY g.Name
            ORDER BY track_count DESC
            LIMIT 10
        `, (err, rows) => {
            if (err) {
                reject(err);
            } else {
                console.log('Top genres by track count:');
                rows.forEach(row => {
                    console.log(`  ${row.genre}: ${row.track_count} tracks, avg $${row.avg_price}`);
                });
                resolve(rows);
            }
        });
    });
}
```

## Performance Optimization

### Format Selection Strategy

| Use Case | Recommended Format | Why |
|----------|-------------------|-----|
| **Development/Testing** | CSV | Human-readable, easy to edit |
| **Analytics Workloads** | Parquet | Columnar storage, compression |
| **Production Queries** | DuckDB | Maximum query performance |
| **Data Exchange** | Parquet | Standard format, good compression |
| **Web APIs** | JSON | Universal support, structured |

### Storage Optimization

```bash
# Compare file sizes
echo "📊 File size comparison:"
ls -lah datasets/tpch.*

# CSV vs Parquet compression
echo "CSV size: $(du -sh datasets/tpch/customer.csv | cut -f1)"
echo "Parquet size: $(du -sh datasets/tpch/customer.parquet | cut -f1)"

# Query performance comparison
echo "🔍 Query performance:"
time duckdb -c "SELECT COUNT(*) FROM 'datasets/tpch/customer.csv'" 2>&1 | grep real
time duckdb -c "SELECT COUNT(*) FROM 'datasets/tpch/customer.parquet'" 2>&1 | grep real
```

### Memory-Efficient Processing

```sql
-- Process large datasets in chunks
CREATE TEMP TABLE chunk AS
SELECT * FROM large_dataset LIMIT 100000;

-- Process chunk
-- ... your processing logic ...

-- Clear chunk and load next
DELETE FROM chunk;
INSERT INTO chunk SELECT * FROM large_dataset LIMIT 100000 OFFSET 100000;
```

## Dataset Maintenance

### Backup and Archival

```bash
#!/bin/bash
# backup_datasets.sh

BACKUP_DIR="dataset_backups/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"

echo "💾 Creating dataset backup in $BACKUP_DIR"

# Backup all dataset files
cp -r datasets/* "$BACKUP_DIR/"

# Create compressed archive
tar -czf "dataset_backup_$(date +%Y%m%d).tar.gz" -C "$BACKUP_DIR" .

echo "✅ Backup complete"
```

### Cleanup and Optimization

```bash
#!/bin/bash
# cleanup_datasets.sh

echo "🧹 Cleaning up dataset files..."

# Remove temporary files
find datasets -name "*.tmp" -delete
find datasets -name "*.log" -delete

# Remove duplicate formats (keep Parquet only)
find datasets -name "*.csv" -exec rm {} \;
find datasets -name "*.json" -exec rm {} \;

# Compress old datasets
find datasets -name "*.parquet" -mtime +30 -exec gzip {} \;

echo "✅ Cleanup complete"
```

### Dataset Versioning

```bash
#!/bin/bash
# version_datasets.sh

VERSION=$(date +%Y%m%d)
DATASET_DIR="datasets"

echo "📦 Creating versioned dataset archive..."

# Create versioned copy
VERSION_DIR="${DATASET_DIR}_v${VERSION}"
cp -r "$DATASET_DIR" "$VERSION_DIR"

# Create archive
tar -czf "datasets_v${VERSION}.tar.gz" "$VERSION_DIR"

# Update current symlink
rm -f "${DATASET_DIR}_current"
ln -sf "$VERSION_DIR" "${DATASET_DIR}_current"

echo "✅ Created datasets version $VERSION"
```

## Troubleshooting Dataset Issues

### Common Issues

#### 1. Generation Fails

**Error:** `Dataset generation failed`

**Solutions:**
```bash
# Check available disk space
df -h

# Check permissions
ls -la datasets/

# Try different output directory
frozen-duckdb download --dataset chinook --output-dir /tmp/test

# Check DuckDB connection
duckdb -c "SELECT 1;"
```

#### 2. Format Conversion Issues

**Error:** `Conversion failed`

**Solutions:**
```bash
# Check input file exists and is readable
ls -la input_file.csv

# Check output directory permissions
mkdir -p output_dir && touch output_dir/test

# Try explicit format specification
frozen-duckdb convert --input file.csv --output file.parquet --input-format csv --output-format parquet

# Check for special characters in file paths
ls -la "file with spaces.csv"
```

#### 3. Memory Issues with Large Datasets

**Error:** `Out of memory`

**Solutions:**
```bash
# Monitor memory usage
htop

# Use smaller scale factor
frozen-duckdb download --dataset tpch --format parquet  # Uses SF 0.01

# Process in chunks
# Split large files before conversion
split -l 100000 large_file.csv chunk_
for chunk in chunk_*; do
    frozen-duckdb convert --input "$chunk" --output "${chunk}.parquet"
done
```

### Performance Issues

#### Query Performance Problems

**Problem:** Queries running slowly on generated data

**Solutions:**
```sql
-- Check table statistics
PRAGMA table_info(customer);

-- Analyze table for query optimization
ANALYZE customer;

-- Check query execution plan
EXPLAIN SELECT * FROM customer WHERE c_acctbal > 1000;
```

#### Storage Space Issues

**Problem:** Dataset files taking too much space

**Solutions:**
```bash
# Use Parquet format (better compression)
frozen-duckdb download --dataset tpch --format parquet

# Remove unnecessary formats
rm datasets/*.csv datasets/*.json

# Compress old datasets
gzip datasets/*.parquet
```

## Best Practices

### 1. Format Selection

- **Use Parquet** for analytical workloads (better performance and compression)
- **Use CSV** for human-readable data and simple imports
- **Use DuckDB native** for maximum query performance in DuckDB workflows
- **Use JSON** for structured data exchange and web APIs

### 2. Dataset Organization

```
datasets/
├── raw/              # Original/source data
├── processed/        # Cleaned and transformed data
├── analytics/        # Aggregated data for analysis
└── archives/         # Compressed historical data
```

### 3. Performance Optimization

- **Generate appropriate scale factors** for your testing needs
- **Use columnar formats** (Parquet) for analytical queries
- **Partition large datasets** by date or category
- **Index frequently queried columns** for better performance

### 4. Data Quality

- **Validate generated data** for completeness and correctness
- **Check data types** match your application requirements
- **Verify referential integrity** in relational datasets
- **Test queries** on sample data before full processing

## Integration Examples

### ETL Pipeline

```bash
#!/bin/bash
# etl_pipeline.sh

echo "🚀 Starting ETL pipeline..."

# Extract: Generate source data
frozen-duckdb download --dataset tpch --format csv --output-dir ./extracted

# Transform: Convert and clean data
frozen-duckdb convert --input ./extracted/customer.csv --output ./transformed/customer.parquet

# Load: Import into target system
duckdb target.duckdb -c "
CREATE TABLE customers AS SELECT * FROM './transformed/customer.parquet';
"

echo "✅ ETL pipeline complete"
```

### Testing Data Setup

```rust
// test_fixtures.rs
use std::process::Command;

pub fn setup_test_datasets() -> Result<(), Box<dyn std::error::Error>> {
    // Generate test data
    let output = Command::new("frozen-duckdb")
        .args(&["download", "--dataset", "chinook", "--format", "parquet"])
        .output()?;

    if !output.status.success() {
        return Err(format!("Dataset generation failed: {}",
                          String::from_utf8_lossy(&output.stderr)).into());
    }

    // Verify data exists
    if !std::path::Path::new("datasets/chinook.parquet").exists() {
        return Err("Test dataset not created".into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_real_data() {
        setup_test_datasets().expect("Failed to setup test data");

        // Your tests using real data
        let conn = duckdb::Connection::open("datasets/chinook.duckdb").unwrap();
        // ... test code ...
    }
}
```

## Summary

Dataset management with Frozen DuckDB provides **comprehensive capabilities** for **generating**, **converting**, and **managing** test and benchmark data. The CLI offers **industry-standard datasets** like TPC-H and Chinook with **multiple format options** optimized for different use cases.

**Key Capabilities:**
- **Sample data generation**: Chinook music database and TPC-H benchmark
- **Format conversion**: CSV ↔ Parquet ↔ JSON ↔ Arrow ↔ DuckDB
- **Performance optimization**: Choose optimal formats for your workload
- **Integration ready**: Works seamlessly with Rust, Python, Node.js applications

**Performance Benefits:**
- **Fast generation**: <10 seconds for typical datasets
- **Efficient storage**: Parquet compression saves 2-10x space
- **Query performance**: 5-10x faster queries with optimized formats
- **Memory efficiency**: Columnar storage reduces memory usage

**Next Steps:**
1. Generate sample data for your testing needs
2. Convert to optimal formats for your workload
3. Integrate with your application workflows
4. Set up automated dataset management for CI/CD
