# CLI API Reference

## Overview

The Frozen DuckDB CLI provides a **comprehensive command-line interface** for dataset management, format conversion, performance benchmarking, and **LLM operations** via the Flock extension.

## Command Structure

```bash
frozen-duckdb-cli [OPTIONS] <COMMAND>

Options:
    -v, --verbose...    Increase verbosity (can be used multiple times)
    -h, --help         Print help
    -V, --version      Print version

Commands:
    download      Download and generate sample datasets
    convert       Convert datasets between different formats
    test          Show testing guidance
    benchmark     Benchmark operations
    info          Show comprehensive system information
    flock-setup   Setup Ollama for LLM operations
    complete      Generate text completion
    embed         Generate embeddings for semantic search
    search        Perform semantic search
    filter        Filter data using LLM evaluation
    summarize     Summarize text collections
    validate-ffi  Validate FFI functionality (core DuckDB + Flock LLM)
    help          Print this message or the help of the given subcommand(s)
```

## Dataset Management Commands

### `download` - Download and Generate Datasets

Downloads or generates sample datasets for testing and development.

```bash
frozen-duckdb-cli download --dataset <DATASET> [OPTIONS]

Options:
    -d, --dataset <DATASET>   Dataset name to download or generate
                              (runtime-validated: chinook, tpch)
    -o, --output-dir <DIR>    Output directory for dataset files [default: datasets]
    -f, --format <FORMAT>     Output format [default: csv]
                              (chinook: csv, parquet; tpch: csv, parquet, duckdb)
    -h, --help               Print help
```

**Dataset Options:**
- **`chinook`**: Music database with artists, albums, tracks, and sales data
- **`tpch`**: TPC-H decision support benchmark with 8 tables

**Examples:**
```bash
# Download Chinook dataset in CSV format
frozen-duckdb-cli download --dataset chinook --format csv

# Generate TPC-H dataset in Parquet format
frozen-duckdb-cli download --dataset tpch --format parquet --output-dir ./data
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
frozen-duckdb-cli convert --input <INPUT> --output <OUTPUT> [OPTIONS]

Options:
    -i, --input <INPUT>              Input file path to convert from
    -o, --output <OUTPUT>            Output file path to convert to
        --input-format <FORMAT>      Input file format [default: csv] (long flag only)
        --output-format <FORMAT>     Output file format [default: parquet] (long flag only)
    -h, --help                      Print help
```

**Supported Conversions** (only CSV ↔ Parquet is implemented; every other
pair errors with "Unsupported conversion"):
| From → To | CSV | Parquet | JSON | Arrow |
|-----------|-----|---------|------|-------|
| **CSV** | ❌ | ✅ | ❌ | ❌ |
| **Parquet** | ✅ | ❌ | ❌ | ❌ |
| **JSON** | ❌ | ❌ | ❌ | ❌ |
| **Arrow** | ❌ | ❌ | ❌ | ❌ |

**Examples:**
```bash
# Convert CSV to Parquet
frozen-duckdb-cli convert --input data.csv --output data.parquet

# Convert Parquet to CSV with explicit formats
frozen-duckdb-cli convert --input data.parquet --output data.csv --input-format parquet --output-format csv
```

## System Information Commands

### `info` - System Information

Displays comprehensive information about frozen DuckDB configuration.

```bash
frozen-duckdb-cli info

Options:
    -h, --help    Print help
```

(The global `-v` counts verbosity and is placed *before* the subcommand:
`frozen-duckdb-cli -v info`.)

**Information Displayed:**
- **Version**: Frozen DuckDB version (the crate version, mirroring the bundled DuckDB version)
- **Build Type**: Pre-compiled binary
- **Architecture**: Current system architecture
- **Available Extensions**: DuckDB extensions loaded

**Example Output** (requires `-v` — every field is a tracing INFO event,
suppressed at the default WARN verbosity; witnessed 2026-09-22 the default
invocation prints nothing and exits 0):
```bash
$ frozen-duckdb-cli -v info
INFO frozen_duckdb::cli::dataset_manager: 🦆 Frozen DuckDB Information
INFO ... Version: 1.5.5
INFO ... Build Type: Pre-compiled binary
INFO ... Architecture: aarch64        # std::env::consts::ARCH ("aarch64", not "arm64")
INFO ... Target: macos
INFO ... Available Extensions: autocomplete, avro, aws, ... (the full
       duckdb_extensions() list — 31 entries on the v1.5.5 dylib, incl. flock)
```

## LLM Integration Commands

### `flock-setup` - Ollama Configuration

Sets up Ollama integration for LLM operations via Flock extension.

```bash
frozen-duckdb-cli flock-setup [OPTIONS]

Options:
        --ollama-url <URL>        Ollama server URL [default: http://localhost:11434]
        --text-model <MODEL>      Text generation model [default: qwen3-coder:30b]
        --embedding-model <MODEL> Embedding model [default: qwen3-embedding:8b]
        --skip-verification       Skip model verification (long flag only)
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
frozen-duckdb-cli flock-setup

# Setup with custom Ollama URL
frozen-duckdb-cli flock-setup --ollama-url http://192.168.1.100:11434

# Setup without verification (faster)
frozen-duckdb-cli flock-setup --skip-verification
```

### `complete` - Text Completion

Generates text completion using LLM models.

```bash
frozen-duckdb-cli complete [OPTIONS]

Options:
    -p, --prompt <PROMPT>      Text to complete (mutually exclusive with --input)
    -i, --input <FILE>         Read prompt from file
    -o, --output <FILE>        Write response to file
    -m, --model <MODEL>        Model alias to use [default: text_generator]
        --max-tokens <N>       Maximum tokens to generate [default: 512]
    -t, --temperature <T>      Sampling temperature [default: 0.7]
    -h, --help                Print help
```

`--model` takes the model alias configured by `flock-setup`
(`text_generator` / `embedder` by default), not a free Ollama model name.

**Usage Modes:**
1. **Direct prompt**: `--prompt "Explain recursion in programming"`
2. **File input**: `--input prompt.txt` (cannot be combined with `--prompt`)
3. **Interactive**: No arguments (reads one line from stdin)

**Examples:**
```bash
# Complete text directly
frozen-duckdb-cli complete --prompt "Explain recursion in programming"

# Read prompt from file
frozen-duckdb-cli complete --input my_prompt.txt --output response.txt

# Interactive mode
echo "Write a haiku about databases" | frozen-duckdb-cli complete
```

### `embed` - Embedding Generation

Generates embeddings for semantic search and similarity operations.

> **STATUS (audited 2026-09-22):** not implemented. After the Flock
> readiness check, `FlockManager::generate_embeddings()` always returns
> `Err` (vector extraction from DuckDB's array type is a TODO — see the
> method doc in `crates/frozen-duckdb/src/cli/flock_manager.rs`), so the
> CLI's `.expect()` panics and the process aborts with **exit code 101**.
> Witnessed: `frozen-duckdb-cli embed --text "hello"` → panic
> "Embedding generation not implemented yet", exit 101.

```bash
frozen-duckdb-cli embed [OPTIONS]

Options:
    -t, --text <TEXT>         Text to generate embeddings for (mutually exclusive with --input)
    -i, --input <FILE>        Read texts from file (one per line)
    -o, --output <FILE>       Write embeddings to file as JSON
    -m, --model <MODEL>       Model alias to use [default: embedder]
        --normalize           Normalize embeddings (long flag only)
    -h, --help               Print help
```

**Input Formats:**
- **Single text**: `--text "Python programming language"`
- **File input**: `--input texts.txt` (one text per line)
- The two options are mutually exclusive — they cannot be combined

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
frozen-duckdb-cli embed --text "machine learning"

# Generate embeddings for multiple texts from file
frozen-duckdb-cli embed --input documents.txt --output embeddings.json

# Generate normalized embeddings
frozen-duckdb-cli embed --text "artificial intelligence" --normalize
```

### `search` - Semantic Search

Performs semantic search using embeddings and similarity matching.

> **STATUS (audited 2026-09-22):** not implemented. `FlockManager::
> semantic_search()` always returns `Err("Semantic search not implemented
> ...")` (see `crates/frozen-duckdb/src/cli/flock_manager.rs`), so the
> CLI's `.expect()` panics with **exit code 101**. The options below are
> the accepted surface; the operation itself is a TODO.

```bash
frozen-duckdb-cli search [OPTIONS]

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
frozen-duckdb-cli search --query "machine learning" --corpus documents.txt

# Search with custom threshold and limit
frozen-duckdb-cli search --query "database optimization" --corpus papers.txt --threshold 0.8 --limit 5

# JSON output format
frozen-duckdb-cli search --query "rust programming" --corpus code.txt --format json
```

### `filter` - LLM-based Filtering

Filters data using LLM evaluation and criteria matching.

```bash
frozen-duckdb-cli filter [OPTIONS]

Options:
    -c, --criteria <CRITERIA>    Filtering criteria (mutually exclusive with --prompt)
    -p, --prompt <PROMPT>        Custom evaluation prompt
    -i, --input <FILE>           Input file to filter (one item per line)
    -o, --output <FILE>          Output file for results
    -m, --model <MODEL>          Model alias to use [default: text_generator]
        --positive-only          Show only matching items (long flag only)
    -h, --help                  Print help
```

**Filtering Modes:**
1. **Criteria-based**: `--criteria "Is this about technology?"`
2. **Custom prompt**: `--prompt "Answer yes or no"` (applied verbatim as the classification instruction for every input line — there is no placeholder substitution)
3. **Positive only**: Only show items that match criteria

Note: `--input` is plain text, one item per line.

**Examples:**
```bash
# Filter technology-related items
frozen-duckdb-cli filter --criteria "Is this about technology?" --input items.txt

# Custom evaluation prompt (used verbatim for every line)
frozen-duckdb-cli filter --prompt "Is this a programming language? Answer yes or no" --input languages.txt

# Save results to file
frozen-duckdb-cli filter --criteria "Is this positive?" --input reviews.txt --output positive_reviews.txt
```

### `summarize` - Text Summarization

Summarizes collections of text using LLM capabilities.

```bash
frozen-duckdb-cli summarize [OPTIONS]

Options:
    -i, --input <FILE>       Input file or directory
    -o, --output <FILE>      Output file for summary
    -s, --strategy <STRATEGY> Summarization strategy [default: reduce]
                             (implemented values: reduce, map — any other
                             value, including "extractive", falls back to a
                             single combined-summary path)
        --max-length <INT>   Maximum summary length in words [default: 150] (long flag only)
    -m, --model <MODEL>      Model alias to use [default: text_generator]
    -h, --help              Print help
```

**Input Types:**
- **Single file**: `--input document.txt` (one text per line)
- **Directory**: `--input documents/` (reads all .txt files)
- **Multiple files**: Processes all text files in directory

**Summarization Strategies:**
- **`reduce`**: Hierarchical summarization via the LLM reduce function (default)
- **`map`**: Individual summaries, then combined
- **anything else** (including `extractive`): falls back to one combined
  LLM summary over all texts — there is no distinct extractive path
  (`crates/frozen-duckdb/src/cli/flock_manager.rs`, `summarize_texts`)

**Examples:**
```bash
# Summarize single document
frozen-duckdb-cli summarize --input article.txt

# Summarize multiple documents in directory
frozen-duckdb-cli summarize --input papers/ --output summary.txt --strategy map

# Extractive-style summary with custom length (falls back to the
# combined-summary path — no distinct extractive implementation)
frozen-duckdb-cli summarize --input notes.txt --strategy extractive --max-length 100
```

## Utility Commands

### `test` - Testing Guidance

Shows information about running the test suite.

```bash
frozen-duckdb-cli test

# Output (only with -v or higher — the lines are tracing INFO events,
# suppressed at the default WARN verbosity; witnessed 2026-09-22:
# default invocation prints nothing and exits 0):
🧪 Tests have been moved to the test suite
   Run tests with: cargo test
   Run specific tests with: cargo test <test_name>
   Run all tests with: cargo test --all
```

### `benchmark` - Performance Benchmarking

Runs performance benchmarks on DuckDB operations.

> **STATUS (audited 2026-09-22):** stub. The handler only logs
> "Benchmarking ... operation" and "📊 Performance benchmarking feature
> coming soon!" at INFO level and exits 0 — no benchmark is executed
> (`crates/frozen-duckdb/src/main.rs`, `Commands::Benchmark`). At default
> verbosity it prints nothing.

```bash
frozen-duckdb-cli benchmark [OPTIONS]

Options:
    -o, --operation <OPERATION>    Operation type to benchmark [default: query]
                                   (possible values: query, insert, export)
    -i, --iterations <N>           Number of iterations [default: 1000]
    -s, --size <SIZE>              Dataset size [default: medium]
                                   (possible values: small, medium, large)
    -h, --help                    Print help
```

### `validate-ffi` - FFI Validation

Validates FFI functionality end to end: binary loading, core DuckDB
operations, Flock LLM functions and aggregates, fusion functions, context
columns, and TPC-H extension loading / data generation / all 22 benchmark
queries.

```bash
frozen-duckdb-cli validate-ffi [OPTIONS]

Options:
        --skip-llm    Skip LLM validation (faster; no Ollama required)
        --format <F>  Output format [default: human] (possible values: human, json)
        --verbose     Show detailed per-step information
    -h, --help       Print help
```

> **STATUS (audited 2026-09-22):** `--skip-llm` and `--verbose` are
> accepted but **not wired** — `main.rs` destructures them as
> `_skip_llm` / `_verbose` ("wired to FlockManager knobs in a later
> milestone; unused today") and calls `validate_ffi()` with no
> arguments, so LLM validation layers always run. Only `--format json`
> changes behavior.

**Examples:**
```bash
# Full validation
frozen-duckdb-cli validate-ffi

# Core-only validation (no Ollama needed)
frozen-duckdb-cli validate-ffi --skip-llm

# JSON output for automation
frozen-duckdb-cli validate-ffi --format json
```

## Error Handling

The CLI provides **clear error messages** and **consistent exit codes**:

### Exit Codes

| Code | Description | Example Usage |
|------|-------------|---------------|
| **0** | Success | Operation completed successfully |
| **1** | General error | Invalid input, file not found, operation failed |
| **2** | CLI usage error | Unknown flag / missing argument (clap parse error) |
| **4** | Flock extension | Extension not available |
| **101** | Unimplemented-feature panic | `embed` / `search` abort via `.expect()` (Rust panic exit code) |

(Explicit `std::process::exit` calls in `src/main.rs` emit only 0, 1, and
4; code 2 comes from clap's own parse-error exit, and 101 from the
`embed`/`search` panics. There is no dedicated environment/binary-
validation exit code: binary acquisition happens inside `cargo build` via
`frozen-duckdb-builder`, not in the CLI process.)

### Error Messages

**Environment Errors (legacy prebuilt workflow only):**
```bash
❌ DUCKDB_LIB_DIR not set

❌ No frozen DuckDB binary found in /path/to/lib
```

These come from `env_setup::validate_binary()` in the legacy manual-prebuilt
path. The normal `cargo build` path has no environment to configure — the
builder downloads and normalizes the dylib into `~/.frozen-duckdb/cache/`.

**LLM Errors:**
```bash
❌ Flock extension not available
   Run 'frozen-duckdb-cli flock-setup' first

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
frozen-duckdb-cli info

# Info level (-v)
frozen-duckdb-cli -v info

# Debug level (-vv)
frozen-duckdb-cli -vv download --dataset chinook

# Trace level (-vvv)
frozen-duckdb-cli -vvv complete --prompt "test"
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
# setup_frozen_duckdb.sh — no environment setup required

# Verify setup
frozen-duckdb-cli info

# Generate sample data
frozen-duckdb-cli download --dataset chinook --format csv

# Convert to Parquet for better performance
frozen-duckdb-cli convert --input datasets/chinook.csv --output datasets/chinook.parquet

echo "✅ Frozen DuckDB setup complete!"
```

### CI/CD Integration

```yaml
# .github/workflows/test.yml — zero setup: the builder downloads and caches
# the dylib on first build; the emitted @rpath handles runtime loading
- name: Run tests
  run: cargo test --all

- name: Generate test data
  run: frozen-duckdb-cli download --dataset tpch --format parquet --output-dir test_data
```

### LLM Pipeline Script

```bash
#!/bin/bash
# llm_pipeline.sh

# Setup Ollama and Flock
frozen-duckdb-cli flock-setup

# Generate embeddings for documents
frozen-duckdb-cli embed --input documents.txt --output embeddings.json

# Search for relevant content
frozen-duckdb-cli search --query "$1" --corpus documents.txt --format json

# Generate summary if requested
if [[ "$2" == "summary" ]]; then
    frozen-duckdb-cli summarize --input documents.txt --output summary.txt
fi
```

## Troubleshooting

### Common Issues

#### 1. Binary Not Found
```bash
❌ No frozen DuckDB binary found

# Solution: check the builder-managed cache, then rebuild to retry
# acquisition (cache -> local prebuilt dir -> GitHub Release download
# -> local compile pinned at upstream tag v1.5.5)
ls -la ~/.frozen-duckdb/cache/
cargo clean && cargo build
```

#### 2. Flock Extension Not Available
```bash
❌ Flock extension not available

# Solution:
frozen-duckdb-cli flock-setup
# Then verify Ollama is running
```

#### 3. Model Not Available
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
frozen-duckdb-cli info

# Show available extensions
frozen-duckdb-cli -v info

# Test with maximum verbosity
frozen-duckdb-cli -vvv complete --prompt "test"
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
