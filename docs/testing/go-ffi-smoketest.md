# Go FFI Smoke Test for Frozen DuckDB

> **STATUS (G3, 2026-09-21):** the scripts documented here are kept with
> on-script status headers, not verified green. `run_ffi_validation.sh` and
> `test_ffi_simple.sh` carry a **KEPT-UNCERTAIN / BLOCKED-as-committed**
> header (TR6, see the script tops and `docs/sjira/v26.9.21/TR6.md`): they
> assume a repo-local `prebuilt/` dylib, but the builder caches binaries in
> `~/.frozen-duckdb/cache/v{VER}-{arch}/`, so as committed they would fail at
> the library-resolution step. Repair belongs to the T6 (scripts-fix) lane.
> The maintained FFI validation path is `frozen-duckdb-cli validate-ffi`
> (audited 2026-09-22: the `--skip-llm` flag is accepted but **not yet
> wired** — LLM validation layers run regardless, so Ollama is always
> needed for a fully green run; see `docs/api/cli.md`).

## Overview

The Go FFI smoke test validates that the frozen-duckdb library properly exposes all required FFI functionality, including:

- **Core DuckDB FFI functions**: Database lifecycle, query execution, data types
- **Flock LLM Extensions**: Text completion, embeddings, filtering, fusion functions
- **Architecture Detection**: Binary selection and compatibility validation
- **Error Handling**: Edge cases and failure modes
- **Performance**: Build time and runtime performance validation

## Quick Start

### Prerequisites

1. **Go 1.21+** installed and configured
2. **Frozen DuckDB environment** set up
3. **CGO enabled** for C interop

### Running the Tests

```bash
# Maintained FFI validation (validates the same surface through the CLI)
frozen-duckdb-cli validate-ffi # full; needs Ollama — --skip-llm is
                               # accepted but NOT wired (audited 2026-09-22)

# Kept-with-header scripts (KEPT-UNCERTAIN — see STATUS above; they assume a
# repo-local prebuilt/ dylib that the builder no longer provides, and are NOT
# verified green in this tree):
./scripts/run_ffi_validation.sh   # BLOCKED as committed (TR6)
./scripts/test_ffi_simple.sh      # BLOCKED as committed (TR6)
./scripts/build_go_smoketest.sh   # not run this session
```

## Test Coverage

### Core DuckDB FFI Functions

The smoke test validates these essential DuckDB C API functions:

#### Database Lifecycle
- `duckdb_open()` - Open database connection
- `duckdb_close()` - Close database connection
- `duckdb_connect()` - Create connection to database
- `duckdb_disconnect()` - Close connection

#### Query Execution
- `duckdb_query()` - Execute SQL queries
- `duckdb_destroy_result()` - Clean up query results
- `duckdb_result_error()` - Get error messages

#### Result Inspection
- `duckdb_column_count()` - Get number of columns
- `duckdb_row_count()` - Get number of rows
- `duckdb_column_name()` - Get column names
- `duckdb_column_type()` - Get column data types

#### Value Extraction
- `duckdb_value_varchar()` - Extract string values
- `duckdb_value_int32()` - Extract integer values
- `duckdb_value_double()` - Extract floating-point values
- `duckdb_value_boolean()` - Extract boolean values

### Flock LLM Extensions

The test validates that Flock extension functions are available:

#### LLM Functions
- `llm_complete()` - Text completion and generation
- `llm_embedding()` - Embedding generation for semantic search
- `llm_filter()` - Intelligent data filtering and classification

#### Fusion Functions
- `fusion_rrf()` - Reciprocal rank fusion for search results
- `fusion_combsum()` - CombSUM fusion for score combination

### Architecture Detection

The test validates:
- Current system architecture detection
- Architecture-specific binary selection
- Binary compatibility and loading

### Error Handling

The test validates:
- Invalid SQL query handling
- Missing model/secret error handling
- Library loading error handling
- Memory management and cleanup

## Test Files

### Go Smoke Test Files

- **`scripts/smoke_go_simple.go`** - Main Go smoke test implementation
- **`scripts/duckdb_ffi.h`** - C header with FFI function declarations
- **`scripts/build_go_smoketest.sh`** - Build script for Go smoke test

### Validation Scripts

- **`scripts/run_ffi_validation.sh`** - Comprehensive FFI validation suite
- **`scripts/test_ffi_simple.sh`** - Simple FFI validation (no Go required)

## Test Results

### Expected Output

```
🦆 Frozen DuckDB FFI Smoke Test
============================================================
✅ PASS Library Version (2.1ms)
   DuckDB Version: 1.5.5
✅ PASS Architecture Detection (0.5ms)
   Detected Architecture: arm64
✅ PASS Database Lifecycle (15.2ms)
✅ PASS Basic Queries (8.7ms)
✅ PASS Error Handling (3.1ms)
✅ PASS Flock Extension Loading (45.3ms)
✅ PASS Flock LLM Functions (12.8ms)

============================================================
🦆 Frozen DuckDB FFI Smoke Test Results
============================================================
Total Tests: 7
Passed: 7
Failed: 0
Success Rate: 100.0%
Total Duration: 87.7ms
🎉 ALL TESTS PASSED - FFI is fully functional!
```

### Performance Targets

- **Library loading**: < 50ms
- **Database connection**: < 20ms
- **Query execution**: < 10ms for simple queries
- **Extension loading**: < 100ms
- **Total test suite**: < 200ms

## Architecture Support

### Supported Architectures

- **x86_64/amd64**: Intel/AMD 64-bit processors
- **arm64/aarch64**: Apple Silicon and ARM 64-bit processors

### Binary Selection

The test validates that the correct architecture-specific binary is selected:

```bash
# Test with specific architecture
ARCH=x86_64 ./scripts/run_ffi_validation.sh
ARCH=arm64 ./scripts/run_ffi_validation.sh
```

## Integration with CI/CD

### GitHub Actions

Add to your workflow (the shell scripts above are BLOCKED as committed —
use the CLI entry point until T6 repairs them):

```yaml
- name: FFI Validation
  # --skip-llm is accepted but NOT wired (audited 2026-09-22); LLM layers
  # run regardless, so Ollama must be reachable for a green run
  run: ./target/debug/frozen-duckdb-cli validate-ffi --skip-llm
```

### Local Development

```bash
# Run before committing (no environment sourcing needed)
cargo build --workspace
./target/debug/frozen-duckdb-cli validate-ffi   # --skip-llm accepted, not wired (2026-09-22)
```

## Troubleshooting

### Common Issues

#### Go Not Found
```bash
# Install Go via asdf
asdf plugin-add golang
asdf install golang 1.22.5
asdf local golang 1.22.5
```

#### CGO Issues
```bash
# Ensure CGO is enabled
export CGO_ENABLED=1
export CGO_CFLAGS="-I$DUCKDB_INCLUDE_DIR"
export CGO_LDFLAGS="-L$DUCKDB_LIB_DIR -lduckdb"
```

#### Library Not Found
```bash
# The builder caches binaries here (no DUCKDB_LIB_DIR involved)
ls -la ~/.frozen-duckdb/cache/v1.5.5-*/
# For the legacy CGO/Go path only, prebuilt/setup_env.sh can still be sourced
# manually — it is not part of the cargo build path
```

#### Flock Extension Issues
```bash
# Check if Ollama is running (for LLM functions)
curl -s http://localhost:11434/api/version

# Install required models
ollama pull qwen3-coder:30b
ollama pull qwen3-embedding:8b
```

### Debug Mode

Run with verbose output:

```bash
# Enable debug logging
export RUST_LOG=debug
./scripts/run_ffi_validation.sh
```

## Performance Benchmarks

### Build Time Comparison

| Method | First Build | Incremental Build |
|--------|-------------|-------------------|
| Source Compilation | 5-10 minutes | 30-60 seconds |
| Frozen Binary | 10-30 seconds | 1-5 seconds |
| **Improvement** | **99% faster** | **95% faster** |

### Runtime Performance

| Operation | Time | Throughput |
|-----------|------|------------|
| Library Loading | < 50ms | - |
| Database Connection | < 20ms | - |
| Simple Query | < 10ms | 100+ queries/sec |
| Extension Loading | < 100ms | - |

## Contributing

### Adding New Tests

1. Add test function to `scripts/smoke_go_simple.go`
2. Register test in `main()` function
3. Update this documentation
4. Run validation: `./scripts/test_ffi_simple.sh`

### Test Requirements

- **Fast execution**: Each test should complete in < 100ms
- **Deterministic**: Tests should produce consistent results
- **Comprehensive**: Cover all major FFI functionality
- **Clear errors**: Provide actionable error messages

## Related Documentation

(The first two links below used to point at `architecture-detection.md` and
`flock-integration.md`, which do not exist in this tree — dead paths recorded
by G3. Current homes:)
- Architecture overview: [../architecture/overview.md](../architecture/overview.md)
- Binary management: [../architecture/binary-management.md](../architecture/binary-management.md)
- Performance Tuning: [../guides/performance-tuning.md](../guides/performance-tuning.md)
- Troubleshooting: [../../TROUBLESHOOTING.md](../../TROUBLESHOOTING.md)
