# Test Dependency Crate

This crate demonstrates the issue where DuckDB gets compiled every time when using `frozen-duckdb` as a dependency.

## Purpose

This test crate simulates the real-world scenario where:
1. A project depends on `frozen-duckdb`
2. The dependent crate uses DuckDB functionality
3. Without proper environment setup, DuckDB compilation occurs in dependent crates

## Usage

### Build without frozen-duckdb environment
```bash
cd ../
cargo build  # This should trigger DuckDB compilation
```

### Build with frozen-duckdb environment
```bash
cd ../
source prebuilt/setup_env.sh
cargo build  # This should use pre-compiled binaries
```

### Run the test application
```bash
cargo run
```

## Expected Behavior

- **Without setup**: DuckDB compilation occurs (slow)
- **With setup**: Uses pre-compiled binaries (fast)

## Testing

```bash
cargo test
```

This will show whether the environment variables are properly propagated to dependent crates.
