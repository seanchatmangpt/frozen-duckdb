# CLI Commands Reference

## Overview

The Frozen DuckDB CLI provides **comprehensive command-line operations** for dataset management, format conversion, LLM operations, and system administration. All commands are designed for **production use** with **clear error messages** and **performance optimization**.

## Command Categories

### Dataset Operations
- **`download`** - Generate sample datasets (Chinook, TPC-H)
- **`convert`** - Convert between data formats

### System Operations
- **`info`** - Display system information and configuration
- **`test`** - Show testing guidance
- **`benchmark`** - Performance benchmarking (coming soon)

### LLM Operations
- **`flock-setup`** - Configure Ollama for LLM operations
- **`complete`** - Generate text completion
- **`embed`** - Generate embeddings for semantic search
- **`search`** - Perform semantic search
- **`filter`** - Filter data using LLM evaluation
- **`summarize`** - Summarize text collections

## Dataset Operations

### `download` Command

Downloads or generates sample datasets for testing and development.

```bash
frozen-duckdb download --dataset <DATASET> [OPTIONS]

Arguments:
    <DATASET>    Dataset name [possible values: chinook, tpch]

Options:
    -o, --output-dir <DIR>    Output directory [default: datasets]
    -f, --format <FORMAT>     Output format [default: csv] [possible values: csv, parquet, duckdb]
    -h, --help               Print help
```

#### Chinook Dataset

**Music Database Schema:**
```sql
-- Artists table
CREATE TABLE artists (
    ArtistId INTEGER PRIMARY KEY,
    Name TEXT NOT NULL
);

-- Albums table
CREATE TABLE albums (
    AlbumId INTEGER PRIMARY KEY,
    Title TEXT NOT NULL,
    ArtistId INTEGER REFERENCES artists(ArtistId)
);

-- Tracks table
CREATE TABLE tracks (
    TrackId INTEGER PRIMARY KEY,
    Name TEXT NOT NULL,
    AlbumId INTEGER REFERENCES albums(AlbumId),
    Composer TEXT,
    Milliseconds INTEGER,
    Bytes INTEGER,
    UnitPrice DECIMAL
);
```

**Usage Examples:**
```bash
# Generate Chinook in CSV format
frozen-duckdb download --dataset chinook --format csv

# Generate in Parquet with custom location
frozen-duckdb download --dataset chinook --format parquet --output-dir ./data

# Generate in DuckDB native format
frozen-duckdb download --dataset chinook --format duckdb
```

#### TPC-H Dataset

**Decision Support Benchmark Schema:**
```sql
-- Customer table
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

-- Orders table
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

-- Lineitem table
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
```

**Usage Examples:**
```bash
# Generate TPC-H in Parquet format (recommended)
frozen-duckdb download --dataset tpch --format parquet

# Generate in CSV format
frozen-duckdb download --dataset tpch --format csv --output-dir ./benchmark

# Generate in DuckDB format for maximum performance
frozen-duckdb download --dataset tpch --format duckdb
```

### `convert` Command

Converts datasets between different file formats for optimal performance and compatibility.

```bash
frozen-duckdb convert --input <INPUT> --output <OUTPUT> [OPTIONS]

Options:
    -i, --input <INPUT>              Input file path
    -o, --output <OUTPUT>            Output file path
    -f, --input-format <FORMAT>      Input format [default: csv] [possible values: csv, parquet, json]
    -t, --output-format <FORMAT>     Output format [default: parquet] [possible values: csv, parquet, json, arrow]
    -h, --help                      Print help
```

**Supported Conversions:**
| Input → Output | CSV | Parquet | JSON | Arrow |
|----------------|-----|---------|------|-------|
| **CSV** | ✅ | ✅ | ✅ | ❌ |
| **Parquet** | ✅ | ✅ | ❌ | ❌ |
| **JSON** | ❌ | ❌ | ✅ | ❌ |
| **Arrow** | ❌ | ❌ | ❌ | ✅ |

**Usage Examples:**
```bash
# Convert CSV to Parquet (recommended for analytics)
frozen-duckdb convert --input customer_data.csv --output customer_data.parquet

# Convert Parquet to CSV for human analysis
frozen-duckdb convert --input analytics.parquet --output report.csv

# Batch conversion script
for file in *.csv; do
    frozen-duckdb convert --input "$file" --output "${file%.csv}.parquet"
done
```

## System Operations

### `info` Command

Displays comprehensive information about the Frozen DuckDB configuration and capabilities.

```bash
frozen-duckdb info [OPTIONS]

Options:
    -v, --verbose    Show detailed information
    -h, --help      Print help
```

**Information Categories:**
- **Version Information**: Frozen DuckDB version and build type
- **Architecture Details**: System architecture and binary selection
- **Extension Status**: Available DuckDB extensions
- **Environment Validation**: Configuration status
- **Performance Metrics**: Build time and resource usage

**Example Output:**
```bash
🦆 Frozen DuckDB Information
  Version: 0.1.0
  Build Type: Pre-compiled binary
  Architecture: arm64
  Target: darwin
  Available Extensions: parquet, tpch, flock
  Library Directory: /Users/sac/dev/frozen-duckdb/prebuilt
  Binary Size: 50MB (libduckdb_arm64.dylib)
```

**Verbose Output:**
```bash
# With -v flag
frozen-duckdb info -v

🦆 Frozen DuckDB Information
  Version: 0.1.0
  Build Type: Pre-compiled binary
  Architecture: arm64
  Target: darwin
  Available Extensions: parquet, tpch, flock, arrow, json
  Library Directory: /Users/sac/dev/frozen-duckdb/prebuilt
  Include Directory: /Users/sac/dev/frozen-duckdb/prebuilt
  Binary Size: 50MB (libduckdb_arm64.dylib)
  Environment Status: ✅ Configured
  Performance: Build time <10s, Memory <200MB
```

### `test` Command

Shows guidance for running the comprehensive test suite.

```bash
frozen-duckdb test

# Output:
🧪 Tests have been moved to the test suite
   Run tests with: cargo test
   Run specific tests with: cargo test <test_name>
   Run all tests with: cargo test --all
```

**Test Categories:**
- **Unit Tests**: Library functionality (architecture, benchmark, env_setup)
- **Integration Tests**: End-to-end functionality
- **Performance Tests**: Build time and runtime performance
- **LLM Tests**: Flock extension functionality

**Running Tests:**
```bash
# Run all tests (recommended for CI/CD)
cargo test --all

# Run specific test categories
cargo test --test core_functionality_tests
cargo test --test flock_tests

# Run with verbose output for debugging
cargo test -- --nocapture

# Run multiple times to check for flaky tests (core team requirement)
cargo test --all && cargo test --all && cargo test --all
```

### `benchmark` Command

Performance benchmarking for various DuckDB operations (feature coming soon).

```bash
frozen-duckdb benchmark [OPTIONS]

Options:
    -o, --operation <OPERATION>    Operation type [default: query] [possible values: query, insert, export]
    -n, --iterations <INT>         Number of iterations [default: 1000]
    -s, --size <SIZE>              Dataset size [default: medium] [possible values: small, medium, large]
    -h, --help                    Print help
```

**Planned Features:**
- **Query Performance**: SELECT operation benchmarking
- **Insert Performance**: Data loading speed measurement
- **Export Performance**: Data export efficiency testing
- **LLM Performance**: Text generation and embedding speed

## LLM Operations

### `flock-setup` Command

Configures Ollama integration for LLM operations via the Flock extension.

```bash
frozen-duckdb flock-setup [OPTIONS]

Options:
    -u, --ollama-url <URL>    Ollama server URL [default: http://localhost:11434]
    -s, --skip-verification   Skip model verification
    -h, --help               Print help
```

**Setup Process:**
1. **Install Flock Extension**: `INSTALL flock FROM community; LOAD flock;`
2. **Create Ollama Secret**: `CREATE SECRET ollama_secret (TYPE OLLAMA, API_URL 'http://localhost:11434')`
3. **Create Models**: `CREATE MODEL('coder', 'qwen3-coder:30b', 'ollama')`
4. **Verify Setup**: Test basic LLM operations

**Usage Examples:**
```bash
# Standard setup with local Ollama
frozen-duckdb flock-setup

# Setup with remote Ollama server
frozen-duckdb flock-setup --ollama-url http://192.168.1.100:11434

# Quick setup without verification
frozen-duckdb flock-setup --skip-verification
```

**Verification Steps:**
```bash
# Check Ollama server status
curl -s http://localhost:11434/api/version

# Verify models are available
curl -s http://localhost:11434/api/tags | grep qwen3-coder

# Test LLM functionality
frozen-duckdb complete --prompt "Hello, how are you?"
```

### `complete` Command

Generates text completion using LLM models.

```bash
frozen-duckdb complete [OPTIONS]

Options:
    -p, --prompt <PROMPT>      Text to complete
    -i, --input <FILE>         Read prompt from file
    -o, --output <FILE>        Write response to file
    -m, --model <MODEL>        Model to use [default: coder] [possible values: coder, embedder]
    -h, --help                Print help
```

**Input Methods:**
- **Direct prompt**: `--prompt "Explain recursion in programming"`
- **File input**: `--input prompt.txt`
- **Interactive**: No arguments (reads from stdin)

**Usage Examples:**
```bash
# Complete text directly
frozen-duckdb complete --prompt "Explain recursion in programming"

# Read from file and save to file
frozen-duckdb complete --input my_prompt.txt --output response.txt

# Interactive mode
echo "Write a haiku about databases" | frozen-duckdb complete

# Use specific model
frozen-duckdb complete --prompt "Debug this code" --model coder
```

**Output Examples:**
```bash
# Simple completion
$ frozen-duckdb complete --prompt "The Rust programming language"
The Rust programming language is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety featuring...

# Code completion
$ frozen-duckdb complete --prompt "fn fibonacci(n: u32) -> u32 {"
fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
```

### `embed` Command

Generates embeddings for semantic search and similarity operations.

```bash
frozen-duckdb embed [OPTIONS]

Options:
    -t, --text <TEXT>         Text to generate embeddings for
    -i, --input <FILE>        Read texts from file (one per line)
    -o, --output <FILE>       Write embeddings to file as JSON
    -m, --model <MODEL>       Model to use [default: embedder] [possible values: coder, embedder]
    -n, --normalize          Normalize embeddings
    -h, --help               Print help
```

**Input Formats:**
- **Single text**: `--text "Python programming language"`
- **Multiple texts**: `--input documents.txt` (one text per line)
- **Batch processing**: Both options combined

**Usage Examples:**
```bash
# Generate embedding for single text
frozen-duckdb embed --text "machine learning"

# Process multiple texts from file
frozen-duckdb embed --input documents.txt --output embeddings.json

# Generate normalized embeddings
frozen-duckdb embed --text "artificial intelligence" --normalize

# Batch processing with output
frozen-duckdb embed --input texts.txt --output vectors.json --normalize
```

**Output Format:**
```json
[
  {
    "text": "machine learning",
    "embedding": [0.123456, 0.789012, ...],
    "dimensions": 1024,
    "normalized": true
  }
]
```

### `search` Command

Performs semantic search using embeddings and similarity matching.

```bash
frozen-duckdb search [OPTIONS]

Options:
    -q, --query <QUERY>       Search query
    -c, --corpus <FILE>       Corpus file for search
    -t, --threshold <FLOAT>   Similarity threshold [default: 0.7]
    -l, --limit <INT>         Maximum results [default: 10]
    -f, --format <FORMAT>     Output format [default: text] [possible values: text, json]
    -h, --help               Print help
```

**Search Algorithm:**
1. **Embed query**: Generate embedding for search query
2. **Compare embeddings**: Calculate similarity with corpus embeddings
3. **Rank results**: Sort by similarity score (cosine similarity)
4. **Filter results**: Apply threshold and limit
5. **Format output**: Return results in requested format

**Usage Examples:**
```bash
# Basic semantic search
frozen-duckdb search --query "machine learning" --corpus documents.txt

# Search with custom threshold and limit
frozen-duckdb search --query "database optimization" --corpus papers.txt --threshold 0.8 --limit 5

# JSON output for programmatic processing
frozen-duckdb search --query "rust programming" --corpus code.txt --format json

# Search in generated embeddings
frozen-duckdb search --query "neural networks" --corpus embeddings.json --threshold 0.75
```

**Output Formats:**

**Text Format (Default):**
```bash
🔍 Found 3 similar documents:
  1. "Machine learning algorithms use neural networks" (similarity: 0.892)
  2. "Deep learning is a subset of machine learning" (similarity: 0.856)
  3. "AI systems learn from data patterns" (similarity: 0.743)
```

**JSON Format:**
```json
[
  {
    "document": "Machine learning algorithms use neural networks",
    "similarity_score": 0.892
  },
  {
    "document": "Deep learning is a subset of machine learning",
    "similarity_score": 0.856
  }
]
```

### `filter` Command

Filters data using LLM evaluation and criteria matching.

```bash
frozen-duckdb filter [OPTIONS]

Options:
    -c, --criteria <CRITERIA>    Filtering criteria
    -p, --prompt <PROMPT>        Custom evaluation prompt
    -i, --input <FILE>           Input file to filter (one item per line)
    -o, --output <FILE>          Output file for results
    -m, --model <MODEL>          Model to use [default: coder] [possible values: coder, embedder]
    -h, --help                  Print help
```

**Filtering Strategies:**
- **Criteria-based**: `--criteria "Is this about technology?"`
- **Custom prompt**: `--prompt "Answer yes or no: {{text}}"`
- **Positive only**: Only show items that match criteria

**Usage Examples:**
```bash
# Filter technology-related items
frozen-duckdb filter --criteria "Is this about technology?" --input items.txt

# Custom evaluation prompt
frozen-duckdb filter --prompt "Is this a programming language? Answer yes or no: {{text}}" --input languages.txt

# Save filtered results
frozen-duckdb filter --criteria "Is this positive?" --input reviews.txt --output positive_reviews.txt

# Show only matching items
frozen-duckdb filter --criteria "Contains 'machine learning'?" --input articles.txt
```

**Output Examples:**

**Default Format:**
```bash
📊 Filter results:
✅ MATCH: "Machine learning is transforming healthcare"
❌ NO MATCH: "The weather is nice today"
✅ MATCH: "AI algorithms improve efficiency"
```

**File Output:**
```text
# positive_reviews.txt
✅ "This product exceeded my expectations"
✅ "Excellent quality and fast shipping"
❌ "Average product, nothing special"
✅ "Highly recommended for developers"
```

### `summarize` Command

Summarizes collections of text using LLM capabilities.

```bash
frozen-duckdb summarize [OPTIONS]

Options:
    -i, --input <FILE>       Input file or directory
    -o, --output <FILE>      Output file for summary
    -s, --strategy <STRATEGY> Summarization strategy [default: concise] [possible values: concise, detailed, bullet]
    -l, --max-length <INT>   Maximum summary length in words [default: 200]
    -m, --model <MODEL>      Model to use [default: coder] [possible values: coder, embedder]
    -h, --help              Print help
```

**Input Types:**
- **Single file**: `--input document.txt` (one text per line)
- **Directory**: `--input documents/` (reads all .txt files)
- **Multiple files**: Processes all text files in directory

**Summarization Strategies:**
- **`concise`**: Brief, to-the-point summary (default)
- **`detailed`**: Comprehensive summary with key points
- **`bullet`**: Bullet-point format for easy scanning

**Usage Examples:**
```bash
# Summarize single document
frozen-duckdb summarize --input article.txt --strategy concise

# Summarize multiple documents in directory
frozen-duckdb summarize --input research_papers/ --output summary.txt --strategy detailed

# Bullet-point summary with custom length
frozen-duckdb summarize --input meeting_notes.txt --strategy bullet --max-length 100

# Save summary to file
frozen-duckdb summarize --input documents.txt --output summary.md --strategy detailed --max-length 300
```

**Output Examples:**

**Concise Strategy:**
```text
# summary.txt
The research examines machine learning applications in healthcare, focusing on diagnostic accuracy improvements through neural networks. Key findings show 95% accuracy in medical image analysis, with recommendations for clinical implementation including data privacy considerations and model validation protocols.
```

**Bullet Strategy:**
```text
# summary.txt
- Machine learning shows 95% accuracy in medical diagnostics
- Neural networks excel at medical image analysis
- Key challenges: data privacy and model validation
- Recommendations: clinical trials and regulatory approval
- Future directions: real-time diagnostics and personalized medicine
```

**Detailed Strategy:**
```text
# summary.txt
## Machine Learning in Healthcare: A Comprehensive Analysis

### Diagnostic Accuracy
The study demonstrates significant improvements in diagnostic accuracy using machine learning algorithms, particularly neural networks for medical image analysis. The research reports a 95% accuracy rate across multiple medical imaging modalities.

### Technical Implementation
- **Model Architecture**: Convolutional neural networks with transfer learning
- **Training Data**: 10,000+ annotated medical images across 5 specialties
- **Validation**: 5-fold cross-validation with external test set

### Clinical Applications
- **Radiology**: Chest X-ray analysis for pneumonia detection
- **Pathology**: Tissue sample classification for cancer diagnosis
- **Dermatology**: Skin lesion analysis for melanoma screening

### Challenges and Recommendations
- **Data Privacy**: HIPAA compliance and patient data protection
- **Model Validation**: Prospective clinical trials required
- **Regulatory Approval**: FDA clearance for clinical use
- **Implementation**: Integration with existing healthcare systems

### Future Directions
- **Real-time Diagnostics**: Point-of-care AI systems
- **Personalized Medicine**: Patient-specific treatment recommendations
- **Multi-modal Analysis**: Combining imaging with genomic data
```

## Error Handling and Exit Codes

### Exit Codes

| Code | Description | Example Usage |
|------|-------------|---------------|
| **0** | Success | Operation completed successfully |
| **1** | General error | Invalid arguments, file not found |
| **2** | Environment error | DUCKDB_LIB_DIR not set |
| **3** | Binary validation | No DuckDB binary found |
| **4** | Flock extension | Extension not available |

### Error Messages

**Environment Errors:**
```bash
❌ DUCKDB_LIB_DIR not set
   Please run: source prebuilt/setup_env.sh

❌ No frozen DuckDB binary found in /path/to/lib
   Check that binaries exist in prebuilt/
```

**LLM Errors:**
```bash
❌ Flock extension not available
   Run 'frozen-duckdb flock-setup' first

❌ Model not found
   Check if Ollama models are properly configured
```

**File Errors:**
```bash
❌ Failed to read input file 'missing.txt': No such file or directory
   Check that the file exists and is readable

❌ Failed to write to output file 'readonly.txt': Permission denied
   Check file permissions and try again
```

## Performance Characteristics

### Command Performance

| Command | Typical Time | Memory Usage | Notes |
|---------|--------------|--------------|-------|
| **download** | <10s | <100MB | Depends on dataset size |
| **convert** | <5s | <50MB | Varies with file size |
| **info** | <1s | <10MB | System information only |
| **complete** | 2-5s | <200MB | Depends on prompt length |
| **embed** | 1-3s | <150MB | Depends on text length |
| **search** | 0.5-2s | <100MB | Depends on corpus size |
| **filter** | 1-4s | <150MB | Depends on item count |
| **summarize** | 3-10s | <200MB | Depends on text volume |

### Optimization Tips

1. **Use appropriate formats**: Parquet for large datasets, CSV for small data
2. **Batch operations**: Process multiple files together when possible
3. **Memory management**: Monitor usage for large operations
4. **Local Ollama**: Use local server for best performance and privacy

## Integration Examples

### Shell Scripts

```bash
#!/bin/bash
# dataset_pipeline.sh

# Generate test data
frozen-duckdb download --dataset tpch --format parquet --output-dir ./data

# Convert for optimal performance
frozen-duckdb convert --input ./data/customer.csv --output ./data/customer.parquet

# Generate embeddings for search
frozen-duckdb embed --input ./data/documents.txt --output ./data/embeddings.json

echo "✅ Dataset pipeline complete"
```

### CI/CD Integration

```yaml
# .github/workflows/test.yml
- name: Setup test environment
  run: |
    source frozen-duckdb/prebuilt/setup_env.sh
    frozen-duckdb download --dataset chinook --format parquet --output-dir test_data

- name: Run tests
  run: cargo test --all
```

### Automation Scripts

```python
#!/usr/bin/env python3
# automate_llm_tasks.py

import subprocess
import json

def run_llm_command(cmd):
    """Run frozen-duckdb command and return result"""
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    return result.stdout.strip()

# Generate embeddings for documents
print("Generating embeddings...")
run_llm_command("frozen-duckdb embed --input documents.txt --output embeddings.json")

# Search for relevant content
print("Searching for 'machine learning'...")
search_results = run_llm_command(
    "frozen-duckdb search --query 'machine learning' --corpus documents.txt --format json"
)

# Parse and use results
results = json.loads(search_results)
print(f"Found {len(results)} relevant documents")

# Generate summary
print("Generating summary...")
summary = run_llm_command(
    "frozen-duckdb summarize --input documents.txt --strategy concise"
)
print("Summary:", summary)
```

## Troubleshooting Common Issues

### 1. Command Not Found

**Error:** `frozen-duckdb: command not found`

**Solutions:**
```bash
# Build the CLI first
cargo build --release

# Use full path
./target/release/frozen-duckdb --help

# Add to PATH
export PATH="$PWD/target/release:$PATH"
```

### 2. Environment Not Configured

**Error:** `DUCKDB_LIB_DIR not set`

**Solutions:**
```bash
# Source the setup script
source ../frozen-duckdb/prebuilt/setup_env.sh

# Set environment manually
export DUCKDB_LIB_DIR="/path/to/frozen-duckdb/prebuilt"
export DUCKDB_INCLUDE_DIR="/path/to/frozen-duckdb/prebuilt"
```

### 3. LLM Operations Failing

**Error:** `Flock extension not available`

**Solutions:**
```bash
# Setup Ollama integration
frozen-duckdb flock-setup

# Check Ollama server
curl -s http://localhost:11434/api/version

# Verify models are loaded
ollama list
```

### 4. File Permission Issues

**Error:** `Permission denied`

**Solutions:**
```bash
# Check file permissions
ls -la input_file.txt

# Fix permissions
chmod 644 input_file.txt
chmod 755 output_directory/

# Check disk space
df -h
```

## Best Practices

### 1. Command Organization

- **Use consistent output directories** for related operations
- **Document complex command sequences** in scripts
- **Validate inputs and outputs** before processing
- **Use appropriate formats** for intended use cases

### 2. Performance Optimization

- **Process data in appropriate batch sizes** for your system
- **Use local Ollama** for best performance and privacy
- **Monitor resource usage** during intensive operations
- **Cache frequently used results** to avoid recomputation

### 3. Error Handling

- **Check exit codes** in scripts and automation
- **Provide meaningful error messages** for user guidance
- **Implement retry logic** for transient failures
- **Log operations** for debugging and monitoring

### 4. Integration

- **Test commands** in isolation before integrating into workflows
- **Handle environment setup** in automation scripts
- **Document command usage** for team members
- **Monitor performance** and optimize as needed

## Summary

The CLI commands provide a **comprehensive toolkit** for **dataset management**, **format conversion**, **LLM operations**, and **system administration**. Each command is designed for **production use** with **clear error handling**, **performance optimization**, and **extensive documentation**.

**Key Command Categories:**
- **Dataset Operations**: Generate and convert test/benchmark data
- **System Operations**: Information display and testing guidance
- **LLM Operations**: Text completion, embeddings, search, filtering, summarization

**Performance Highlights:**
- **Dataset generation**: <10 seconds for typical datasets
- **Format conversion**: <5 seconds for typical files
- **LLM operations**: 1-5 seconds for typical requests
- **Memory efficient**: <200MB for most operations

**Integration Ready:**
- **Shell scripts**: Easy automation and pipelines
- **CI/CD systems**: GitHub Actions, GitLab CI, Docker
- **Programming languages**: Rust, Python, Node.js integration
- **Error handling**: Clear messages and actionable guidance
