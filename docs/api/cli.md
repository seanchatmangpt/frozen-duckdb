# CLI API Reference

## Overview

The Frozen DuckDB CLI provides a **comprehensive command-line interface** for dataset management, format conversion, performance benchmarking, and **LLM operations** via the Flock extension.

## Command Structure

```bash
frozen-duckdb [OPTIONS] <COMMAND>

Options:
    -v, --verbose...    Increase verbosity (can be used multiple times)
    -h, --help         Print help
    -V, --version      Print version

Commands:
    download     Download and generate sample datasets
    convert      Convert datasets between different formats
    info         Show comprehensive system information
    flock-setup  Setup Ollama for LLM operations
    complete     Generate text completion
    embed        Generate embeddings for semantic search
    search       Perform semantic search
    filter       Filter data using LLM evaluation
    summarize    Summarize text collections
    test         Show testing guidance
    benchmark    Benchmark operations (coming soon)
```

## Dataset Management Commands

### `download` - Download and Generate Datasets

Downloads or generates sample datasets for testing and development.

```bash
frozen-duckdb download --dataset <DATASET> --format <FORMAT> [OPTIONS]

Arguments:
    <DATASET>    Dataset name to download or generate [possible values: chinook, tpch]

Options:
    -o, --output-dir <DIR>    Output directory for dataset files [default: datasets]
    -f, --format <FORMAT>     Output format [default: csv] [possible values: csv, parquet, duckdb]
    -h, --help               Print help
```

**Dataset Options:**
- **`chinook`**: Music database with artists, albums, tracks, and sales data
- **`tpch`**: TPC-H decision support benchmark with 8 tables

**Examples:**
```bash
# Download Chinook dataset in CSV format
frozen-duckdb download --dataset chinook --format csv

# Generate TPC-H dataset in Parquet format
frozen-duckdb download --dataset tpch --format parquet --output-dir ./data
```

**TPC-H Dataset Contents:**
| Table | Description | Rows (SF 0.01) | Size |
|-------|-------------|----------------|------|
| customer | Customer information | ~1,500 | 100KB-1MB |
| lineitem | Order line items | ~6,000 | 500KB-2MB |
| nation | Country information | ~25 | 1KB |
| orders | Customer orders | ~1,500 | 100KB-500KB |
| part | Parts catalog | ~2,000 | 200KB-1MB |
| partsupp | Part-supplier relationships | ~8,000 | 300KB-1MB |
| region | Geographic regions | ~5 | 1KB |
| supplier | Supplier information | ~100 | 10KB-50KB |

### `convert` - Format Conversion

Converts datasets between different file formats.

```bash
frozen-duckdb convert --input <INPUT> --output <OUTPUT> [OPTIONS]

Options:
    -i, --input <INPUT>              Input file path to convert from
    -o, --output <OUTPUT>            Output file path to convert to
    -f, --input-format <FORMAT>      Input file format [default: csv] [possible values: csv, parquet, json]
    -t, --output-format <FORMAT>     Output file format [default: parquet] [possible values: csv, parquet, json, arrow]
    -h, --help                      Print help
```

**Supported Conversions:**
| From → To | CSV | Parquet | JSON | Arrow |
|-----------|-----|---------|------|-------|
| **CSV** | ✅ | ✅ | ✅ | ❌ |
| **Parquet** | ✅ | ✅ | ❌ | ❌ |
| **JSON** | ❌ | ❌ | ✅ | ❌ |
| **Arrow** | ❌ | ❌ | ❌ | ✅ |

**Examples:**
```bash
# Convert CSV to Parquet
frozen-duckdb convert --input data.csv --output data.parquet

# Convert Parquet to CSV with explicit formats
frozen-duckdb convert --input data.parquet --output data.csv --input-format parquet --output-format csv
```

## System Information Commands

### `info` - System Information

Displays comprehensive information about frozen DuckDB configuration.

```bash
frozen-duckdb info [OPTIONS]

Options:
    -v, --verbose    Show detailed information
    -h, --help      Print help
```

**Information Displayed:**
- **Version**: Frozen DuckDB version
- **Build Type**: Pre-compiled binary
- **Architecture**: Current system architecture
- **Available Extensions**: DuckDB extensions loaded
- **Environment Status**: Configuration validation

**Example Output:**
```bash
🦆 Frozen DuckDB Information
  Version: 0.1.0
  Build Type: Pre-compiled binary
  Architecture: arm64
  Target: darwin
  Available Extensions: parquet, tpch, flock
```

## LLM Integration Commands

### `flock-setup` - Ollama Configuration

Sets up Ollama integration for LLM operations via Flock extension.

```bash
frozen-duckdb flock-setup [OPTIONS]

Options:
    -u, --ollama-url <URL>    Ollama server URL [default: http://localhost:11434]
    -s, --skip-verification   Skip model verification
    -h, --help               Print help
```

**Setup Process:**
1. **Install Flock extension**: `INSTALL flock FROM community; LOAD flock;`
2. **Create Ollama secret**: `CREATE SECRET ollama_secret (TYPE OLLAMA, API_URL 'http://localhost:11434')`
3. **Create models**: `CREATE MODEL('coder', 'qwen3-coder:30b', 'ollama')` and `CREATE MODEL('embedder', 'qwen3-embedding:8b', 'ollama')`
4. **Verify setup**: Test basic LLM operations

**Required Models:**
- **qwen3-coder:30b**: Text generation and completion (30.5B parameters)
- **qwen3-embedding:8b**: Embedding generation (7.6B parameters)

**Example:**
```bash
# Setup with default local Ollama
frozen-duckdb flock-setup

# Setup with custom Ollama URL
frozen-duckdb flock-setup --ollama-url http://192.168.1.100:11434

# Setup without verification (faster)
frozen-duckdb flock-setup --skip-verification
```

### `complete` - Text Completion

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

**Usage Modes:**
1. **Direct prompt**: `--prompt "Explain recursion in programming"`
2. **File input**: `--input prompt.txt`
3. **Interactive**: No arguments (reads from stdin)

**Examples:**
```bash
# Complete text directly
frozen-duckdb complete --prompt "Explain recursion in programming"

# Read prompt from file
frozen-duckdb complete --input my_prompt.txt --output response.txt

# Interactive mode
echo "Write a haiku about databases" | frozen-duckdb complete
```

### `embed` - Embedding Generation

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
- **File input**: `--input texts.txt` (one text per line)
- **Multiple texts**: Both options can be combined

**Output Format:**
```json
[
  {
    "text": "Python programming language",
    "embedding": [0.123, 0.456, ...],
    "dimensions": 1024
  }
]
```

**Examples:**
```bash
# Generate embedding for single text
frozen-duckdb embed --text "machine learning"

# Generate embeddings for multiple texts from file
frozen-duckdb embed --input documents.txt --output embeddings.json

# Generate normalized embeddings
frozen-duckdb embed --text "artificial intelligence" --normalize
```

### `search` - Semantic Search

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

**Search Process:**
1. **Embed query**: Generate embedding for search query
2. **Compare embeddings**: Calculate similarity with corpus embeddings
3. **Rank results**: Sort by similarity score
4. **Filter results**: Apply threshold and limit
5. **Format output**: Return results in requested format

**Examples:**
```bash
# Basic semantic search
frozen-duckdb search --query "machine learning" --corpus documents.txt

# Search with custom threshold and limit
frozen-duckdb search --query "database optimization" --corpus papers.txt --threshold 0.8 --limit 5

# JSON output format
frozen-duckdb search --query "rust programming" --corpus code.txt --format json
```

### `filter` - LLM-based Filtering

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

**Filtering Modes:**
1. **Criteria-based**: `--criteria "Is this about technology?"`
2. **Custom prompt**: `--prompt "Answer yes or no: {{text}}"`
3. **Positive only**: Only show items that match criteria

**Examples:**
```bash
# Filter technology-related items
frozen-duckdb filter --criteria "Is this about technology?" --input items.txt

# Custom evaluation prompt
frozen-duckdb filter --prompt "Is this a programming language? Answer yes or no: {{text}}" --input languages.txt

# Save results to file
frozen-duckdb filter --criteria "Is this positive?" --input reviews.txt --output positive_reviews.txt
```

### `summarize` - Text Summarization

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

**Examples:**
```bash
# Summarize single document
frozen-duckdb summarize --input article.txt --strategy concise

# Summarize multiple documents in directory
frozen-duckdb summarize --input papers/ --output summary.txt --strategy detailed

# Bullet-point summary with custom length
frozen-duckdb summarize --input notes.txt --strategy bullet --max-length 100
```

## Utility Commands

### `test` - Testing Guidance

Shows information about running the test suite.

```bash
frozen-duckdb test

# Output:
🧪 Tests have been moved to the test suite
   Run tests with: cargo test
   Run specific tests with: cargo test <test_name>
   Run all tests with: cargo test --all
```

### `benchmark` - Performance Benchmarking

Runs performance benchmarks (feature coming soon).

```bash
frozen-duckdb benchmark [OPTIONS]

Options:
    -o, --operation <OPERATION>    Operation type to benchmark [default: query] [possible values: query, insert, export]
    -n, --iterations <INT>         Number of iterations [default: 1000]
    -s, --size <SIZE>              Dataset size [default: medium] [possible values: small, medium, large]
    -h, --help                    Print help
```

## Error Handling

The CLI provides **clear error messages** and **consistent exit codes**:

### Exit Codes

| Code | Description | Example |
|------|-------------|---------|
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

### Startup Performance

- **CLI startup**: <100ms
- **Command parsing**: <10ms
- **Environment validation**: <50ms
- **Extension loading**: <200ms (for Flock operations)

### Operation Performance

| Operation | Typical Time | Memory Usage |
|-----------|--------------|--------------|
| **Dataset generation** | <10s (small) | <100MB |
| **Format conversion** | <1s (typical files) | <50MB |
| **Text completion** | <5s (typical requests) | <200MB |
| **Embedding generation** | <3s (single text) | <150MB |
| **Semantic search** | <2s (small corpus) | <100MB |
| **Text summarization** | <10s (multiple documents) | <200MB |

## Verbosity Levels

The CLI supports **multiple verbosity levels** for debugging:

```bash
# Default (WARN and above)
frozen-duckdb info

# Info level (-v)
frozen-duckdb -v info

# Debug level (-vv)
frozen-duckdb -vv download --dataset chinook

# Trace level (-vvv)
frozen-duckdb -vvv complete --prompt "test"
```

**Log Levels:**
- **No flag**: WARN level and above
- **`-v`**: INFO level and above
- **`-vv`**: DEBUG level and above
- **`-vvv`**: TRACE level and above (most verbose)

## Integration Examples

### Basic Usage Script

```bash
#!/bin/bash
# setup_frozen_duckdb.sh

# Set up environment
source prebuilt/setup_env.sh

# Verify setup
frozen-duckdb info

# Generate sample data
frozen-duckdb download --dataset chinook --format csv

# Convert to Parquet for better performance
frozen-duckdb convert --input datasets/chinook.csv --output datasets/chinook.parquet

echo "✅ Frozen DuckDB setup complete!"
```

### CI/CD Integration

```yaml
# .github/workflows/test.yml
- name: Setup frozen DuckDB
  run: |
    source frozen-duckdb/prebuilt/setup_env.sh
    echo "DUCKDB_LIB_DIR=$DUCKDB_LIB_DIR" >> $GITHUB_ENV
    echo "DUCKDB_INCLUDE_DIR=$DUCKDB_INCLUDE_DIR" >> $GITHUB_ENV

- name: Run tests
  run: cargo test --all

- name: Generate test data
  run: frozen-duckdb download --dataset tpch --format parquet --output-dir test_data
```

### LLM Pipeline Script

```bash
#!/bin/bash
# llm_pipeline.sh

# Setup Ollama and Flock
frozen-duckdb flock-setup

# Generate embeddings for documents
frozen-duckdb embed --input documents.txt --output embeddings.json

# Search for relevant content
frozen-duckdb search --query "$1" --corpus documents.txt --format json

# Generate summary if requested
if [[ "$2" == "summary" ]]; then
    frozen-duckdb summarize --input documents.txt --output summary.txt
fi
```

## Troubleshooting

### Common Issues

#### 1. Environment Not Configured
```bash
❌ DUCKDB_LIB_DIR not set

# Solution:
source prebuilt/setup_env.sh
```

#### 2. Binary Not Found
```bash
❌ No frozen DuckDB binary found

# Solution:
ls -la prebuilt/libduckdb*
# Check that binaries exist
```

#### 3. Flock Extension Issues
```bash
❌ Flock extension not available

# Solution:
frozen-duckdb flock-setup
# Then verify Ollama is running
```

#### 4. Model Not Available
```bash
❌ Model not found

# Solution:
# Check Ollama models
curl -s http://localhost:11434/api/tags | grep qwen3-coder
# Pull missing models
ollama pull qwen3-coder:30b
ollama pull qwen3-embedding:8b
```

### Debug Information

```bash
# Show system information
frozen-duckdb info

# Show available extensions
frozen-duckdb -v info

# Test with maximum verbosity
frozen-duckdb -vvv complete --prompt "test"
```

## Performance Tuning

### Optimization Tips

1. **Use appropriate formats**: Parquet for analytical workloads, CSV for human-readable data
2. **Batch operations**: Process multiple files together when possible
3. **Memory management**: Monitor memory usage for large datasets
4. **Network optimization**: Use local Ollama for faster LLM operations

### Resource Requirements

| Component | Minimum | Recommended | Notes |
|-----------|---------|-------------|-------|
| **RAM** | 4GB | 16GB | For LLM operations |
| **Storage** | 100MB | 1GB | Including datasets |
| **Network** | N/A | Fast local | For Ollama communication |
| **CPU** | 2 cores | 4+ cores | For parallel operations |

## Security Considerations

### Data Privacy

- **Local processing**: All operations happen locally by default
- **No data transmission**: LLM operations use local Ollama instance
- **File permissions**: Respect existing file system permissions
- **Temporary files**: Cleaned up automatically

### Network Security

- **Local Ollama only**: Designed for localhost:11434 by default
- **No external APIs**: All LLM operations through local Ollama
- **Configurable URLs**: Can specify custom Ollama endpoints
- **Network isolation**: No internet connectivity required

## Summary

The CLI API provides a **comprehensive, user-friendly interface** for all Frozen DuckDB operations, from basic dataset management to advanced LLM capabilities. The design emphasizes **ease of use**, **performance**, and **reliability** while maintaining **complete compatibility** with existing workflows.

**Key Features:**
- **Intuitive commands**: Clear, consistent command structure
- **Comprehensive options**: Rich configuration for all use cases
- **Error handling**: Clear messages and actionable guidance
- **Performance optimization**: Fast operations with minimal overhead
- **LLM integration**: Seamless Flock extension support
- **Production ready**: Robust error handling and validation
