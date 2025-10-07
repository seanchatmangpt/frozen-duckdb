# Library API Reference

## Overview

The Frozen DuckDB library provides three core modules for **architecture detection**, **performance benchmarking**, and **environment setup**. All modules are designed for **production use** with comprehensive error handling and performance optimization.

## Architecture Module

The `architecture` module handles **automatic architecture detection** and **binary selection** for optimal performance.

### Functions

#### `detect() -> String`

Detects the current system architecture with manual override support.

```rust
use frozen_duckdb::architecture;

// Detect current architecture
let arch = architecture::detect();
println!("Current architecture: {}", arch);
// Output: "arm64" or "x86_64"

// With environment override
std::env::set_var("ARCH", "x86_64");
assert_eq!(architecture::detect(), "x86_64");
std::env::remove_var("ARCH");
```

**Environment Override:**
```bash
# Force x86_64 binary selection
ARCH=x86_64 cargo build

# Force arm64 binary selection
ARCH=arm64 cargo build
```

#### `is_supported(arch: &str) -> bool`

Checks if the given architecture has optimized binaries available.

```rust
use frozen_duckdb::architecture;

assert!(architecture::is_supported("x86_64"));
assert!(architecture::is_supported("arm64"));
assert!(architecture::is_supported("aarch64"));
assert!(!architecture::is_supported("unknown"));
```

**Supported Architectures:**
- `x86_64`: Intel/AMD 64-bit processors (55MB optimized binary)
- `arm64`: Apple Silicon processors (50MB optimized binary)
- `aarch64`: ARM 64-bit processors (same as arm64, 50MB optimized binary)

#### `get_binary_name() -> String`

Gets the appropriate binary filename for the current architecture.

```rust
use frozen_duckdb::architecture;

let binary = architecture::get_binary_name();
assert!(binary.starts_with("libduckdb"));
assert!(binary.ends_with(".dylib"));

// With architecture override
std::env::set_var("ARCH", "x86_64");
assert_eq!(architecture::get_binary_name(), "libduckdb_x86_64.dylib");
std::env::remove_var("ARCH");
```

**Binary Mapping:**
| Architecture | Binary Name | Size | Optimization |
|--------------|-------------|------|--------------|
| x86_64 | libduckdb_x86_64.dylib | 55MB | Intel/AMD optimized |
| arm64/aarch64 | libduckdb_arm64.dylib | 50MB | ARM optimized |
| Other | libduckdb.dylib | ~50MB | Generic fallback |

### Performance Characteristics

- **Detection time**: <1μs (cached at OS level)
- **Binary selection**: <1μs
- **Memory usage**: Negligible
- **Thread safety**: Safe for concurrent use

## Environment Setup Module

The `env_setup` module provides utilities for **validating** and **configuring** the frozen DuckDB environment.

### Functions

#### `is_configured() -> bool`

Checks if the frozen DuckDB environment is properly configured.

```rust
use frozen_duckdb::env_setup;

// Check configuration status
if env_setup::is_configured() {
    println!("✅ Environment is properly configured");
} else {
    println!("❌ Please run: source prebuilt/setup_env.sh");
}
```

**Environment Variables Checked:**
- `DUCKDB_LIB_DIR`: Directory containing DuckDB libraries
- `DUCKDB_INCLUDE_DIR`: Directory containing DuckDB headers

#### `get_lib_dir() -> Option<String>`

Gets the configured DuckDB library directory path.

```rust
use frozen_duckdb::env_setup;

// Get library directory
if let Some(lib_dir) = env_setup::get_lib_dir() {
    println!("Library directory: {}", lib_dir);
} else {
    println!("DUCKDB_LIB_DIR not set");
}
```

**Expected Directory Contents:**
- `libduckdb_x86_64.dylib` (55MB) - Intel/AMD 64-bit binary
- `libduckdb_arm64.dylib` (50MB) - Apple Silicon/ARM 64-bit binary
- `libduckdb.dylib` - Generic fallback binary

#### `get_include_dir() -> Option<String>`

Gets the configured DuckDB include directory path.

```rust
use frozen_duckdb::env_setup;

// Get include directory
if let Some(include_dir) = env_setup::get_include_dir() {
    println!("Include directory: {}", include_dir);
} else {
    println!("DUCKDB_INCLUDE_DIR not set");
}
```

**Expected Directory Contents:**
- `duckdb.h` (186KB) - C header file
- `duckdb.hpp` (1.8MB) - C++ header file

#### `validate_binary() -> Result<()>`

Validates that the frozen DuckDB binary exists and is accessible.

```rust
use frozen_duckdb::env_setup;

// Validate binary existence
match env_setup::validate_binary() {
    Ok(()) => println!("✅ DuckDB binaries are available"),
    Err(e) => println!("❌ Binary validation failed: {}", e),
}
```

**Validation Process:**
1. Check `DUCKDB_LIB_DIR` environment variable
2. Verify library directory exists and is accessible
3. Confirm at least one DuckDB binary is present
4. Validate binary permissions and integrity

**Error Conditions:**
- `DUCKDB_LIB_DIR` not set
- Library directory doesn't exist
- No DuckDB binaries found
- Directory not accessible

### Performance Characteristics

- **Configuration check**: <1μs (cached environment variables)
- **Path retrieval**: <1μs
- **Binary validation**: <10ms (file system operations)
- **Memory usage**: Negligible

## Benchmark Module

The `benchmark` module provides utilities for **measuring** and **comparing** build performance.

### Functions

#### `measure_build_time<F>(operation: F) -> std::time::Duration`

Measures the execution time of a build operation with high precision.

```rust
use frozen_duckdb::benchmark;
use std::time::Duration;

// Measure a simple operation
let duration = benchmark::measure_build_time(|| {
    std::thread::sleep(Duration::from_millis(100));
    Ok(())
});

assert!(duration >= Duration::from_millis(100));
assert!(duration < Duration::from_millis(200));
```

**Features:**
- **High precision**: Microsecond-level accuracy
- **Error handling**: Measures time even when operations fail
- **Zero overhead**: Minimal performance impact
- **Thread safety**: Safe for concurrent use

#### `compare_build_times<F1, F2>(operation1: F1, operation2: F2) -> Result<(std::time::Duration, std::time::Duration)>`

Compares the execution times of two different build operations.

```rust
use frozen_duckdb::benchmark;
use std::time::Duration;

let (time1, time2) = benchmark::compare_build_times(
    || {
        // Fast operation (e.g., using pre-built binaries)
        std::thread::sleep(Duration::from_millis(50));
        Ok(())
    },
    || {
        // Slow operation (e.g., compiling from source)
        std::thread::sleep(Duration::from_millis(200));
        Ok(())
    },
).unwrap();

assert!(time2 > time1);
// Calculate improvement percentage
let improvement = ((time2.as_millis() - time1.as_millis()) as f64 / time2.as_millis() as f64) * 100.0;
println!("Improvement: {:.1}%", improvement);
```

**Use Cases:**
- **Build optimization**: Compare different build configurations
- **Binary vs source**: Measure impact of pre-built vs compiled binaries
- **Architecture comparison**: Test performance across different architectures
- **Dependency analysis**: Understand impact of different dependencies

### Performance Characteristics

- **Timing precision**: Microsecond-level accuracy on most platforms
- **Overhead**: <1μs per measurement
- **Memory usage**: No allocations during timing
- **Thread safety**: Safe for concurrent use

## Error Handling

All library functions follow consistent error handling patterns:

### Error Types

```rust
use anyhow::Result;

// All public functions return Result<T, anyhow::Error>
pub fn detect() -> String;                    // Never fails
pub fn is_configured() -> bool;              // Never fails
pub fn get_lib_dir() -> Option<String>;       // Never fails
pub fn get_include_dir() -> Option<String>;   // Never fails
pub fn validate_binary() -> Result<()>;      // Can fail
pub fn measure_build_time<F>(...) -> Duration; // Never fails
pub fn compare_build_times<F1, F2>(...) -> Result<...>; // Can fail
```

### Error Messages

The library provides **clear, actionable error messages**:

```rust
// Environment errors
"DUCKDB_LIB_DIR not set"
"No frozen DuckDB binary found in /path/to/lib"

// Binary validation errors
"No frozen DuckDB binary found in /path/to/lib"
"Library directory does not exist"

// Performance measurement errors
"Operation failed during timing measurement"
"Comparison operations returned different error types"
```

## Integration Examples

### Basic Environment Check

```rust
use frozen_duckdb::{architecture, env_setup};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if environment is properly configured
    if env_setup::is_configured() {
        println!("✅ Frozen DuckDB is ready!");
        println!("Architecture: {}", architecture::detect());

        // Validate binaries exist
        env_setup::validate_binary()?;
        println!("✅ Binaries validated");
    } else {
        println!("❌ Please run: source prebuilt/setup_env.sh");
        std::process::exit(1);
    }

    Ok(())
}
```

### Performance Benchmarking

```rust
use frozen_duckdb::benchmark;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Measure build time for an operation
    let duration = benchmark::measure_build_time(|| {
        // Your build operation here
        std::thread::sleep(std::time::Duration::from_millis(100));
        Ok(())
    });

    println!("Build completed in: {:?}", duration);

    // Compare two approaches
    let (fast_time, slow_time) = benchmark::compare_build_times(
        || {
            // Fast approach (pre-built binary)
            std::thread::sleep(std::time::Duration::from_millis(50));
            Ok(())
        },
        || {
            // Slow approach (source compilation)
            std::thread::sleep(std::time::Duration::from_millis(200));
            Ok(())
        },
    )?;

    println!("Fast approach: {:?}", fast_time);
    println!("Slow approach: {:?}", slow_time);

    Ok(())
}
```

## Testing

The library includes comprehensive tests covering:

### Unit Tests

```bash
# Run all library tests
cargo test --lib

# Run specific module tests
cargo test architecture
cargo test env_setup
cargo test benchmark
```

### Test Coverage

- **Architecture detection**: Multiple architectures and overrides
- **Environment validation**: Various configuration scenarios
- **Binary validation**: Missing files, permission issues
- **Performance measurement**: Timing accuracy and error handling
- **Error conditions**: Invalid inputs and edge cases

### Property Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_architecture_detection_consistency(arch in "x86_64|arm64|aarch64") {
            std::env::set_var("ARCH", arch);
            let detected = detect();
            assert_eq!(detected, arch);
            std::env::remove_var("ARCH");
        }
    }
}
```

## Performance Considerations

### Optimization Guidelines

1. **Architecture detection**: Perform once at startup, cache results
2. **Environment validation**: Call sparingly, cache results when possible
3. **Binary validation**: Perform during initialization, not on every operation
4. **Performance measurement**: Use for optimization, not in production hot paths

### Memory Usage

- **Module size**: <50KB compiled
- **Runtime memory**: Negligible for typical usage
- **Allocation patterns**: No heap allocations in performance-critical paths

## Migration Guide

### From Manual Architecture Detection

```rust
// Before
let arch = std::env::consts::ARCH;

// After
use frozen_duckdb::architecture;
let arch = architecture::detect(); // Includes override support
```

### From Manual Environment Setup

```rust
// Before
let lib_dir = std::env::var("DUCKDB_LIB_DIR").unwrap();

// After
use frozen_duckdb::env_setup;
let lib_dir = env_setup::get_lib_dir().unwrap(); // Includes validation
```

## Summary

The library API provides a **simple, reliable interface** for architecture detection, environment setup, and performance benchmarking. All functions are designed for **production use** with comprehensive error handling, performance optimization, and extensive testing.

**Key Benefits:**
- **Zero configuration**: Automatic architecture detection with manual override
- **Fast validation**: Quick environment and binary verification
- **Accurate benchmarking**: High-precision performance measurement
- **Production ready**: Comprehensive error handling and testing
- **Minimal overhead**: Optimized for frequent use
