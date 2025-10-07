# Coding Standards Guide

## Overview

Frozen DuckDB follows **strict coding standards** to ensure **code quality**, **maintainability**, and **performance**. This guide outlines the **Rust coding standards**, **linting requirements**, and **best practices** required for **production-quality contributions**.

## Rust Language Standards

### 1. Code Style

**Formatting (rustfmt):**
```bash
# Format all code
cargo fmt --all

# Check formatting compliance
cargo fmt --all --check

# Format specific files
cargo fmt -- src/main.rs src/lib.rs
```

**Naming Conventions:**
```rust
// ✅ Good naming
pub fn detect_architecture() -> String
pub fn is_configured() -> bool
pub struct DatasetManager

// ❌ Poor naming
pub fn detectArch() -> String  // CamelCase for functions
pub fn isConfigured() -> bool  // PascalCase for functions
pub struct datasetmanager      // lowercase for types
```

**Module Organization:**
```rust
// lib.rs - clear module exports
pub mod architecture;
pub mod benchmark;
pub mod env_setup;

// Each module file
pub fn public_function() -> Result<()> {
    // Public API
}

fn private_helper() -> Result<()> {
    // Internal implementation
}
```

### 2. Error Handling

**Consistent Error Types:**
```rust
// Use anyhow for flexible error handling
use anyhow::{Context, Result};

pub fn validate_binary() -> Result<()> {
    let lib_dir = get_lib_dir()
        .ok_or_else(|| anyhow::anyhow!("DUCKDB_LIB_DIR not set"))?;

    // ... validation logic ...

    Ok(())
}

// Provide context for errors
pub fn setup_environment() -> Result<()> {
    if !is_configured() {
        return Err(anyhow::anyhow!(
            "Frozen DuckDB not configured. Please run: source prebuilt/setup_env.sh"
        ));
    }

    Ok(())
}
```

**No unwrap() in Library Code:**
```rust
// ✅ Proper error handling
pub fn safe_operation() -> Result<()> {
    let value = risky_operation()
        .context("Failed to perform risky operation")?;

    Ok(())
}

// ❌ Avoid in library code
pub fn unsafe_operation() {
    let value = risky_operation().unwrap(); // Could panic
}
```

### 3. Type Safety

**Strong Typing:**
```rust
// ✅ Clear, typed interfaces
pub fn detect_architecture() -> String {
    // Returns architecture name
}

pub fn is_supported(arch: &str) -> bool {
    // Boolean result
}

pub fn get_binary_name() -> String {
    // Returns binary filename
}

// ❌ Weak typing
pub fn process_input(input: &str) -> String {
    // Unclear what this returns
}
```

**Avoid Global State:**
```rust
// ✅ No global mutable state
pub fn detect_architecture() -> String {
    env::var("ARCH").unwrap_or_else(|_| std::env::consts::ARCH.to_string())
}

// ❌ Global state (avoid when possible)
static mut GLOBAL_ARCH: String = String::new();

pub fn set_global_arch(arch: String) {
    unsafe { GLOBAL_ARCH = arch; } // Unsafe and error-prone
}
```

## Code Quality Standards

### 1. Linting Requirements

**Clippy Configuration:**
```toml
# Cargo.toml
[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }

# Deny problematic patterns
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
```

**Required Lints:**
```bash
# Run comprehensive linting
cargo clippy --all-targets --all-features -- -D warnings

# Check specific lints
cargo clippy -- -W clippy::pedantic
cargo clippy -- -W clippy::nursery

# Check with release build
cargo clippy --release --all-targets -- -D warnings
```

**Acceptable Lint Exceptions:**
```rust
// In specific cases, allow certain patterns
#[allow(clippy::too_many_arguments)]
pub fn complex_function(
    arg1: Type1,
    arg2: Type2,
    // ... many arguments
) -> Result<()> {
    // Implementation with clear justification for many arguments
}
```

### 2. Documentation Standards

**Module Documentation:**
```rust
//! # Architecture Detection Utilities
//!
//! This module provides utilities for detecting the current system architecture
//! and selecting the appropriate DuckDB binary for the platform.
//!
//! ## Supported Architectures
//!
//! - **x86_64**: Intel/AMD 64-bit processors
//! - **arm64/aarch64**: Apple Silicon and ARM 64-bit processors
//!
//! ## Usage Examples
//!
//! ```rust
//! use frozen_duckdb::architecture;
//!
//! let arch = architecture::detect();
//! println!("Current architecture: {}", arch);
//! ```
```

**Function Documentation:**
```rust
/// Detects the current system architecture with manual override support.
///
/// This function first checks for the `ARCH` environment variable to allow
/// manual override of the detected architecture. If not set, it falls back
/// to the system's actual architecture using `std::env::consts::ARCH`.
///
/// # Returns
///
/// A string representing the architecture (e.g., "x86_64", "arm64", "aarch64").
///
/// # Examples
///
/// ```rust
/// use frozen_duckdb::architecture;
///
/// let arch = architecture::detect();
/// assert!(!arch.is_empty());
/// ```
pub fn detect() -> String {
    env::var("ARCH").unwrap_or_else(|_| std::env::consts::ARCH.to_string())
}
```

**Error Documentation:**
```rust
/// Validates that the frozen DuckDB binary exists and is accessible.
///
/// # Returns
///
/// `Ok(())` if at least one DuckDB binary is found and accessible,
/// `Err` with a descriptive error message if no binaries are found.
///
/// # Errors
///
/// This function will return an error if:
/// - `DUCKDB_LIB_DIR` environment variable is not set
/// - The library directory does not exist
/// - No DuckDB binaries are found in the directory
pub fn validate_binary() -> Result<()> {
    // Implementation
}
```

### 3. Testing Standards

**Test Documentation:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Test basic architecture detection functionality
    #[test]
    fn test_detect_architecture() {
        let arch = detect();
        assert!(!arch.is_empty());
        assert!(matches!(arch.as_str(), "x86_64" | "arm64" | "aarch64"));
    }

    /// Test error handling for invalid binary paths
    #[test]
    fn test_binary_validation_with_invalid_path() {
        std::env::set_var("DUCKDB_LIB_DIR", "/nonexistent/path");
        assert!(validate_binary().is_err());
        std::env::remove_var("DUCKDB_LIB_DIR");
    }
}
```

## Performance Standards

### 1. Performance Requirements

**Build Performance SLO:**
- **First build**: ≤10 seconds
- **Incremental build**: ≤1 second
- **Release build**: ≤10 seconds

**Runtime Performance SLO:**
- **Query operations**: ≤100ms for typical queries
- **LLM operations**: ≤5 seconds for typical requests
- **Memory usage**: ≤200MB for typical operations

**Performance Testing:**
```rust
#[test]
fn test_performance_requirements() {
    let duration = benchmark::measure_build_time(|| {
        // Your operation here
        Ok(())
    });

    // Validate against SLO
    assert!(
        duration.as_secs() < 10,
        "Operation exceeded performance requirement: {:?}",
        duration
    );
}
```

### 2. Memory Management

**Efficient Memory Usage:**
```rust
// ✅ Efficient memory handling
pub fn process_data_efficiently(data: &[u8]) -> Result<Vec<String>> {
    // Process in streaming fashion
    let mut results = Vec::new();

    for chunk in data.chunks(1024) {
        let processed = process_chunk(chunk)?;
        results.push(processed);
    }

    Ok(results)
}

// ❌ Memory inefficient
pub fn process_data_inefficiently(data: &[u8]) -> Result<Vec<String>> {
    // Load everything into memory at once
    let all_data = String::from_utf8(data.to_vec())?; // Large allocation
    // ... process all_data ...
}
```

**Memory Profiling:**
```bash
# Profile memory usage
cargo install cargo-profdata
cargo profdata --bin frozen-duckdb

# Check for memory leaks
valgrind --tool=memcheck cargo test --test memory_tests
```

## Security Standards

### 1. Safe Code Practices

**No Unsafe Code in Library:**
```rust
// ✅ Safe FFI handling
use std::os::raw::c_char;

extern "C" {
    fn duckdb_open(path: *const c_char) -> *mut DuckDBConnection;
}

// Safe wrapper with proper error handling
pub fn safe_duckdb_open(path: &str) -> Result<Connection> {
    // ... safe implementation with error handling ...
}

// ❌ Unsafe code (avoid in library code)
pub fn unsafe_duckdb_open(path: &str) -> Connection {
    unsafe { duckdb_open(path.as_ptr() as *const c_char) } // Unsafe
}
```

**Input Validation:**
```rust
// ✅ Proper input validation
pub fn validate_architecture(arch: &str) -> Result<String> {
    match arch {
        "x86_64" | "arm64" | "aarch64" => Ok(arch.to_string()),
        _ => Err(anyhow::anyhow!("Unsupported architecture: {}", arch)),
    }
}

// ❌ No input validation
pub fn process_architecture(arch: &str) -> String {
    // Could receive invalid input
    arch.to_string()
}
```

### 2. Error Safety

**Panic-Free Code:**
```rust
// ✅ Panic-free error handling
pub fn safe_operation() -> Result<()> {
    let result = risky_operation()
        .context("Failed to perform risky operation")?;

    Ok(())
}

// ❌ Panic-prone code
pub fn unsafe_operation() -> Result<()> {
    let result = risky_operation().unwrap(); // Could panic
    Ok(())
}
```

## Code Organization Standards

### 1. Module Structure

**Clear Module Boundaries:**
```rust
// src/lib.rs
pub mod architecture;
pub mod benchmark;
pub mod env_setup;

// Each module should be focused and cohesive
// architecture.rs - only architecture-related functionality
// benchmark.rs - only performance measurement
// env_setup.rs - only environment validation
```

**Single Responsibility Principle:**
```rust
// ✅ Single responsibility
pub mod architecture {
    pub fn detect() -> String { /* architecture detection only */ }
    pub fn is_supported(arch: &str) -> bool { /* support checking only */ }
    pub fn get_binary_name() -> String { /* binary selection only */ }
}

// ❌ Multiple responsibilities
pub mod utils {
    pub fn detect() -> String { /* mixed with other utilities */ }
    pub fn format_time() -> String { /* unrelated functionality */ }
    pub fn validate_email() -> bool { /* completely different domain */ }
}
```

### 2. Function Design

**Small, Focused Functions:**
```rust
// ✅ Small, focused functions
pub fn detect_architecture() -> String {
    env::var("ARCH").unwrap_or_else(|_| std::env::consts::ARCH.to_string())
}

pub fn is_architecture_supported(arch: &str) -> bool {
    matches!(arch, "x86_64" | "arm64" | "aarch64")
}

pub fn get_binary_for_architecture(arch: &str) -> String {
    match arch {
        "x86_64" => "libduckdb_x86_64.dylib".to_string(),
        "arm64" | "aarch64" => "libduckdb_arm64.dylib".to_string(),
        _ => "libduckdb.dylib".to_string(),
    }
}

// ❌ Large, complex function
pub fn process_everything() -> Result<()> {
    // 100+ lines of mixed concerns
}
```

**Function Length Limits:**
- **Functions**: ≤80 lines (extract helpers for longer functions)
- **Modules**: ≤500 lines (split into submodules if larger)
- **Tests**: ≤50 lines per test function

### 3. Type Design

**Clear Type Definitions:**
```rust
// ✅ Clear, well-defined types
#[derive(Debug, Clone)]
pub struct ArchitectureInfo {
    pub name: String,
    pub is_supported: bool,
    pub binary_name: String,
}

pub enum Architecture {
    X86_64,
    Arm64,
    AArch64,
    Unknown(String),
}

// ❌ Unclear types
pub struct Config {
    pub data: HashMap<String, Value>, // Too generic
}
```

## Testing Standards

### 1. Test Coverage Requirements

**Coverage Targets:**
- **Core library functions**: >95% coverage
- **Error paths**: >90% coverage
- **Edge cases**: >85% coverage
- **Integration points**: >80% coverage

**Coverage Measurement:**
```bash
# Generate coverage report
cargo install cargo-llvm-cov
cargo llvm-cov --all --lcov --output-path coverage.lcov

# View coverage in browser
cargo llvm-cov --all --html

# Check coverage thresholds
cargo llvm-cov --all --text | grep -A 10 "Functions\|Lines\|Branches"
```

### 2. Test Quality Standards

**Property-Based Testing:**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_architecture_detection_properties(arch_input in "x86_64|arm64|aarch64|unknown") {
        std::env::set_var("ARCH", arch_input);

        let detected = detect();

        // Properties that should always hold
        prop_assert!(!detected.is_empty());
        prop_assert!(detected.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));

        std::env::remove_var("ARCH");
    }
}
```

**Comprehensive Error Testing:**
```rust
#[test]
fn test_all_error_conditions() {
    // Test missing environment
    std::env::remove_var("DUCKDB_LIB_DIR");
    assert!(!is_configured());

    // Test invalid paths
    std::env::set_var("DUCKDB_LIB_DIR", "/nonexistent/path");
    assert!(validate_binary().is_err());

    // Test permission issues
    std::env::set_var("DUCKDB_LIB_DIR", "/root/private");
    assert!(validate_binary().is_err());

    // Cleanup
    std::env::remove_var("DUCKDB_LIB_DIR");
}
```

## Documentation Standards

### 1. API Documentation

**Complete Function Documentation:**
```rust
/// Measures the execution time of a build operation with high precision.
///
/// This function provides a simple way to measure how long a build operation
/// takes to complete. It uses `std::time::Instant` for high-resolution timing
/// and measures the time even if the operation fails.
///
/// # Arguments
///
/// * `operation` - A closure that performs the build operation to measure
///
/// # Returns
///
/// A `std::time::Duration` representing the time taken to execute the operation.
///
/// # Examples
///
/// ```rust
/// use frozen_duckdb::benchmark;
/// use std::time::Duration;
///
/// let duration = benchmark::measure_build_time(|| {
///     std::thread::sleep(Duration::from_millis(100));
///     Ok(())
/// });
///
/// assert!(duration >= Duration::from_millis(100));
/// ```
///
/// # Performance Characteristics
///
/// - **Timing precision**: Microsecond-level accuracy on most platforms
/// - **Overhead**: <1μs per measurement
/// - **Memory usage**: No allocations during timing
/// - **Thread safety**: Safe for concurrent use
pub fn measure_build_time<F>(operation: F) -> std::time::Duration
where
    F: FnOnce() -> Result<()>,
{
    // Implementation
}
```

### 2. Module Documentation

**Comprehensive Module Docs:**
```rust
//! # Architecture Detection Utilities
//!
//! This module provides utilities for detecting the current system architecture
//! and selecting the appropriate DuckDB binary for the platform. It supports
//! automatic detection with manual override capabilities for testing and
//! cross-compilation scenarios.
//!
//! ## Supported Architectures
//!
//! - **x86_64**: Intel/AMD 64-bit processors
//! - **arm64/aarch64**: Apple Silicon and ARM 64-bit processors
//! - **Fallback**: Generic binary for unsupported architectures
//!
//! ## Usage Examples
//!
//! ### Basic Usage
//!
//! ```rust
//! use frozen_duckdb::architecture;
//!
//! // Detect current architecture
//! let arch = architecture::detect();
//! println!("Current architecture: {}", arch);
//!
//! // Check if architecture is supported
//! if architecture::is_supported(&arch) {
//!     println!("✅ Architecture is supported");
//! } else {
//!     println!("⚠️  Architecture may not be optimized");
//! }
//! ```
//!
//! ### Environment Override
//!
//! ```bash
//! # Force x86_64 binary selection
//! ARCH=x86_64 cargo build
//!
//! # Force arm64 binary selection
//! ARCH=arm64 cargo build
//! ```
//!
//! ## Performance Considerations
//!
//! - Architecture detection is performed once at startup
//! - Binary selection is cached for the duration of the process
//! - Manual override adds minimal overhead (<1ms)
//! - Unsupported architectures fall back to generic binary
```

### 3. Code Examples

**Working Examples:**
```rust
//! ```rust
//! use frozen_duckdb::{architecture, env_setup};
//!
//! // Check if environment is properly configured
//! if env_setup::is_configured() {
//!     println!("✅ Frozen DuckDB is ready!");
//!     println!("Architecture: {}", architecture::detect());
//! } else {
//!     println!("❌ Please run: source prebuilt/setup_env.sh");
//! }
//! ```
```

**Examples Must:**
- ✅ **Compile and run** without errors
- ✅ **Demonstrate real usage** patterns
- ✅ **Include expected output** when relevant
- ✅ **Be minimal but complete** (no unnecessary complexity)

## Performance Standards

### 1. Algorithmic Complexity

**Document Performance Characteristics:**
```rust
/// Gets the appropriate binary filename for the current architecture.
///
/// This function selects the correct DuckDB binary based on the detected
/// architecture. It returns architecture-specific binaries for supported
/// platforms and falls back to a generic binary for unsupported ones.
///
/// # Performance Impact
///
/// Using architecture-specific binaries provides:
/// - **x86_64**: Up to 15% better performance on Intel/AMD processors
/// - **arm64**: Up to 20% better performance on Apple Silicon
/// - **Generic**: Baseline performance, works everywhere
///
/// # Examples
///
/// ```rust
/// use frozen_duckdb::architecture;
///
/// let binary = architecture::get_binary_name();
/// assert!(binary.starts_with("libduckdb"));
/// ```
pub fn get_binary_name() -> String {
    // Implementation
}
```

### 2. Memory Usage Standards

**Memory Efficiency:**
```rust
// ✅ Memory efficient
pub fn process_data_streaming(data: &[u8]) -> Result<Vec<String>> {
    let mut results = Vec::new();

    // Process in chunks to limit memory usage
    for chunk in data.chunks(4096) {
        let processed = process_chunk(chunk)?;
        results.push(processed);
    }

    Ok(results)
}

// ❌ Memory inefficient
pub fn process_data_at_once(data: &[u8]) -> Result<Vec<String>> {
    // Load entire dataset into memory
    let all_data = String::from_utf8(data.to_vec())?; // Large allocation
    // ... process all_data ...
}
```

## Safety Standards

### 1. Safe Code Practices

**FFI Safety:**
```rust
// ✅ Safe FFI usage
use std::ffi::CString;
use std::os::raw::c_char;

extern "C" {
    fn duckdb_open(path: *const c_char) -> *mut DuckDBConnection;
}

pub fn safe_open_database(path: &str) -> Result<Connection> {
    let c_path = CString::new(path)
        .context("Path contains null bytes")?;

    // Safe FFI call with proper error handling
    let conn_ptr = unsafe { duckdb_open(c_path.as_ptr()) };
    if conn_ptr.is_null() {
        return Err(anyhow::anyhow!("Failed to open database"));
    }

    Ok(Connection { ptr: conn_ptr })
}

// ❌ Unsafe FFI usage
pub fn unsafe_open_database(path: &str) -> Connection {
    let conn_ptr = unsafe { duckdb_open(path.as_ptr() as *const c_char) };
    Connection { ptr: conn_ptr } // Could be null pointer
}
```

**Panic Safety:**
```rust
// ✅ Panic-safe code
pub fn robust_operation() -> Result<()> {
    let result = operation_that_might_fail()
        .context("Operation failed with context")?;

    Ok(())
}

// ❌ Panic-prone code
pub fn fragile_operation() -> Result<()> {
    let result = operation_that_might_fail().unwrap(); // Could panic
    Ok(())
}
```

## Code Review Standards

### 1. Review Checklist

**Code Quality:**
- [ ] Code compiles without warnings (`cargo clippy -- -D warnings`)
- [ ] Code is properly formatted (`cargo fmt --check`)
- [ ] All tests pass (`cargo test --all`)
- [ ] Documentation is complete and accurate
- [ ] Error handling is comprehensive

**Performance:**
- [ ] Performance requirements are documented
- [ ] Memory usage is reasonable for the operation
- [ ] No unnecessary allocations in hot paths
- [ ] Performance tests validate requirements

**Security:**
- [ ] No unsafe code without justification
- [ ] Input validation is comprehensive
- [ ] Error messages don't leak sensitive information
- [ ] No potential for panics in error paths

**Testing:**
- [ ] Unit tests cover all public functions
- [ ] Integration tests validate component interactions
- [ ] Property tests validate invariants
- [ ] Error conditions are thoroughly tested

### 2. Pull Request Requirements

**PR Template:**
```markdown
## What changed and why

[Link to issue or describe the change]

## Which performance targets might be impacted?

- [ ] Build time (first build ≤10s, incremental ≤1s)
- [ ] Runtime performance (queries ≤100ms, LLM ≤5s)
- [ ] Memory usage (≤200MB for typical operations)
- [ ] Storage requirements

## New/updated tests added?

- [ ] Unit tests for new functions
- [ ] Integration tests for new features
- [ ] Performance tests for SLO validation
- [ ] Property tests for edge cases

## Documentation updated?

- [ ] README.md updated if public API changed
- [ ] API documentation updated for new functions
- [ ] Examples updated to reflect changes
- [ ] Troubleshooting guide updated if needed

## Architecture decisions explained?

- [ ] New modules follow established patterns
- [ ] Breaking changes justified and documented
- [ ] Performance implications analyzed
- [ ] Security implications considered
```

## Continuous Integration Standards

### 1. CI Pipeline Requirements

**Required CI Checks:**
```yaml
# .github/workflows/ci.yml
- name: Format check
  run: cargo fmt --all --check

- name: Lint check
  run: cargo clippy --all-targets -- -D warnings

- name: Test execution
  run: |
    cargo test --all
    cargo test --all  # Run twice for consistency
    cargo test --all  # Run three times (core team requirement)

- name: Performance validation
  run: cargo test --test performance_tests

- name: Documentation build
  run: cargo doc --all-features
```

### 2. Quality Gates

**Automated Quality Gates:**
```bash
#!/bin/bash
# quality_gate.sh

echo "🛡️ Running quality gate checks..."

# Format check
if ! cargo fmt --all --check; then
    echo "❌ Code formatting issues"
    exit 1
fi

# Lint check
if ! cargo clippy --all-targets -- -D warnings; then
    echo "❌ Linting issues"
    exit 1
fi

# Test execution
for i in {1..3}; do
    if ! cargo test --all; then
        echo "❌ Tests failed on run $i"
        exit 1
    fi
done

# Performance validation
if ! cargo test --test performance_tests; then
    echo "❌ Performance requirements not met"
    exit 1
fi

echo "✅ All quality gates passed"
```

## Maintenance Standards

### 1. Code Refactoring

**Refactoring Guidelines:**
- **Extract functions** for code longer than 80 lines
- **Split modules** larger than 500 lines
- **Remove dead code** that hasn't been used in 6+ months
- **Update dependencies** regularly for security and performance

**Refactoring Process:**
```bash
# Before refactoring
cargo test --all  # Ensure tests pass

# Make changes incrementally
cargo check  # Quick validation during refactoring

# After refactoring
cargo test --all  # Full validation
cargo clippy --all-targets -- -D warnings  # Style validation
```

### 2. Dependency Management

**Dependency Standards:**
```toml
# Use specific versions for reproducibility
[dependencies]
anyhow = "1.0.75"  # Specific version
serde = { version = "1.0", features = ["derive"] }  # Feature specification

# Avoid wildcard versions in production
# tokio = "*"  # ❌ Avoid
tokio = "1.35"  # ✅ Specific version
```

**Security Updates:**
```bash
# Check for security vulnerabilities
cargo audit

# Update dependencies regularly
cargo update

# Check for outdated dependencies
cargo outdated
```

## Summary

Frozen DuckDB's coding standards ensure **high-quality, maintainable, and performant code** through **comprehensive linting**, **thorough testing**, **clear documentation**, and **performance validation**. All contributions must meet these standards for **production-quality results**.

**Key Standards:**
- **Rust style**: rustfmt compliance with clear naming conventions
- **Error handling**: anyhow-based error handling with actionable messages
- **Type safety**: Strong typing with comprehensive input validation
- **Performance**: SLO validation with performance regression detection
- **Security**: Safe code practices with panic-free error handling
- **Testing**: Multiple test runs with property-based and performance testing

**Quality Assurance:**
- **Linting**: Comprehensive clippy checks with deny-level warnings
- **Testing**: 95%+ coverage with flaky test detection
- **Documentation**: Complete API docs with working examples
- **Performance**: SLO validation with automated regression detection

**Development Workflow:**
- **Code review**: Comprehensive checklist for all changes
- **CI/CD**: Automated quality gates for all PRs
- **Maintenance**: Regular refactoring and dependency updates
- **Performance monitoring**: Continuous validation of SLO requirements

**Next Steps:**
1. Review your code against these standards before submitting PRs
2. Run the full test suite multiple times to catch flaky behavior
3. Ensure all documentation examples compile and run correctly
4. Study the [Architecture Decisions](./architecture-decisions.md) for design rationale
