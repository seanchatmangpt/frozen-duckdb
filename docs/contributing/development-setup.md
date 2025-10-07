# Development Setup Guide

## Overview

This guide provides **step-by-step instructions** for setting up a **development environment** for contributing to Frozen DuckDB, including **required tools**, **environment configuration**, and **development workflow**.

## Prerequisites

### 1. System Requirements

**Hardware Requirements:**
- **RAM**: 16GB+ (8GB minimum for basic development)
- **Storage**: 100GB+ (for repositories, models, and test data)
- **CPU**: 4+ cores (8+ cores recommended for parallel testing)

**Operating System:**
- **macOS**: 12.0+ (Intel and Apple Silicon supported)
- **Linux**: Ubuntu 18.04+, CentOS 7+, or similar
- **Windows**: Windows 10+ with WSL2 (Ubuntu recommended)

### 2. Required Tools

#### Rust Development Tools

**Install Rust:**
```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add to PATH
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

**Install Additional Rust Tools:**
```bash
# Rust formatter and linter
cargo install rustfmt
cargo install clippy

# Development utilities
cargo install cargo-edit  # For managing dependencies
cargo install cargo-watch # For automatic rebuilding
cargo install cargo-profdata # For profiling

# Verify tools
rustfmt --version
cargo clippy --version
```

#### Database Development Tools

**Install DuckDB:**
```bash
# macOS
brew install duckdb

# Linux
# Ubuntu/Debian
sudo apt-get install duckdb

# Or build from source
git clone https://github.com/duckdb/duckdb.git
cd duckdb
make
sudo make install
```

**Verify DuckDB:**
```bash
# Test basic functionality
duckdb -c "SELECT version();"

# Test extensions
duckdb -c "INSTALL parquet; LOAD parquet; SELECT 'parquet loaded' as status;"
```

#### LLM Development Tools

**Install Ollama:**
```bash
# macOS/Linux
curl -fsSL https://ollama.ai/install.sh | sh

# Windows (via WSL)
curl -fsSL https://ollama.ai/install.sh | sh

# Verify installation
ollama --version
```

**Install Required Models:**
```bash
# Text generation model
ollama pull qwen3-coder:30b

# Embedding model
ollama pull qwen3-embedding:8b

# Verify models
ollama list
```

## Repository Setup

### 1. Clone Repository

```bash
# Clone the repository
git clone https://github.com/seanchatmangpt/frozen-duckdb.git
cd frozen-duckdb

# Verify repository structure
ls -la
# Should show: Cargo.toml, src/, prebuilt/, scripts/, etc.
```

### 2. Install Dependencies

```bash
# Install Rust dependencies
cargo build

# Install development dependencies
cargo build --release

# Verify no compilation errors
cargo check
cargo check --all-targets
```

### 3. Set Up Environment

```bash
# Set up frozen DuckDB environment
source prebuilt/setup_env.sh

# Verify environment configuration
echo $DUCKDB_LIB_DIR
echo $DUCKDB_INCLUDE_DIR

# Should show paths to prebuilt directory
```

### 4. Run Tests

```bash
# Run basic tests to verify setup
cargo test --lib

# Run all tests (including integration tests)
cargo test --all

# Run tests multiple times (core team requirement)
cargo test --all && cargo test --all && cargo test --all
```

## Development Environment Configuration

### 1. Editor Setup

**VS Code Configuration:**
```json
// .vscode/settings.json
{
  "rust-analyzer.cargo.extraEnv": {
    "DUCKDB_LIB_DIR": "${workspaceFolder}/prebuilt",
    "DUCKDB_INCLUDE_DIR": "${workspaceFolder}/prebuilt"
  },
  "rust-analyzer.cargo.extraArgs": ["--all-features"],
  "rust-analyzer.check.extraArgs": ["--all-targets"],
  "rust-analyzer.cargo.buildScripts.enable": true,
  "rust-analyzer.procMacro.enable": true,
  "rust-analyzer.experimental.procAttr.enable": true
}
```

**VS Code Extensions:**
- **rust-analyzer**: Rust language support
- **CodeLLDB**: Rust debugging
- **Better TOML**: TOML file support
- **Prettier**: Code formatting

**Vim/Neovim Configuration:**
```vim
" .vimrc or init.vim
" Rust development setup
let g:rustfmt_autosave = 1
let g:rust_clippy_autosave = 1

" Set environment variables for Rust
let $DUCKDB_LIB_DIR = expand('%:p:h') . '/prebuilt'
let $DUCKDB_INCLUDE_DIR = expand('%:p:h') . '/prebuilt'
```

### 2. Shell Configuration

**Persistent Environment Setup:**
```bash
# Add to ~/.bashrc or ~/.zshrc
export DUCKDB_LIB_DIR="$(pwd)/prebuilt"
export DUCKDB_INCLUDE_DIR="$(pwd)/prebuilt"

# Rust development tools
export PATH="$HOME/.cargo/bin:$PATH"
export RUST_BACKTRACE=1
export RUST_LOG=debug

# Ollama configuration
export OLLAMA_HOST=127.0.0.1:11434
```

**Development Aliases:**
```bash
# Add to ~/.bashrc or ~/.zshrc
alias ct="cargo test"
alias cta="cargo test --all"
alias cb="cargo build"
alias cbr="cargo build --release"
alias cc="cargo check"
alias ccl="cargo clippy"
alias cfo="cargo fmt"

# Frozen DuckDB specific aliases
alias fdb-setup="source prebuilt/setup_env.sh"
alias fdb-info="cargo run -- info"
alias fdb-test="cargo test --all && cargo test --all && cargo test --all"
```

### 3. Git Configuration

**Git Hooks Setup:**
```bash
# Install pre-commit hooks (if available)
# cp scripts/pre-commit .git/hooks/pre-commit
# chmod +x .git/hooks/pre-commit

# Configure git for development
git config core.editor "code --wait"
git config user.name "Your Name"
git config user.email "your.email@example.com"
```

## Development Workflow

### 1. Code Development

**Start Development Session:**
```bash
# Set up environment
source prebuilt/setup_env.sh

# Start development with auto-reload
cargo watch -x check -x test

# Or run tests on file changes
cargo watch -x "test --all"
```

**Code Style and Formatting:**
```bash
# Format code
cargo fmt

# Check style
cargo fmt --check

# Lint code
cargo clippy

# Check with all features
cargo clippy --all-targets --all-features -- -D warnings
```

### 2. Testing Workflow

**Run Test Suite:**
```bash
# Run all tests (core team requirement: 3+ times)
cargo test --all && cargo test --all && cargo test --all

# Run specific test categories
cargo test architecture
cargo test env_setup
cargo test benchmark
cargo test core_functionality_tests
cargo test flock_tests

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_detect_architecture
```

**Test Data Setup:**
```bash
# Generate test datasets
cargo run -- download --dataset chinook --format parquet --output-dir test_data
cargo run -- download --dataset tpch --format parquet --output-dir test_data

# Verify test data
ls -la test_data/
```

### 3. LLM Development

**Ollama Development Setup:**
```bash
# Start Ollama for development
ollama serve

# In another terminal, set up frozen DuckDB
source prebuilt/setup_env.sh

# Test LLM functionality
cargo run -- complete --prompt "Hello, how are you?"

# Test embedding generation
cargo run -- embed --text "machine learning"
```

**Development Model Selection:**
```bash
# Use smaller models for faster development
CREATE MODEL('dev_coder', 'qwen3-coder:7b', 'ollama');

# Use full models for testing
CREATE MODEL('test_coder', 'qwen3-coder:30b', 'ollama');
```

## Debugging and Profiling

### 1. Build Debugging

**Verbose Build Output:**
```bash
# Debug build issues
RUST_LOG=debug cargo build

# Show compilation commands
cargo build -v

# Check dependencies
cargo tree
```

**Common Build Issues:**
```bash
# Missing dependencies
cargo fetch

# Outdated lock file
rm Cargo.lock && cargo build

# Compilation cache issues
cargo clean && cargo build
```

### 2. Runtime Debugging

**Application Debugging:**
```bash
# Debug with backtrace
RUST_BACKTRACE=1 cargo run -- complete --prompt "test"

# Debug with logging
RUST_LOG=debug cargo run -- info

# Profile memory usage
cargo profdata --bin frozen-duckdb
```

**LLM Debugging:**
```bash
# Check Ollama server status
curl -s http://localhost:11434/api/version

# Monitor server logs
tail -f ~/.ollama/logs/server.log

# Test basic connectivity
curl -s http://localhost:11434/api/generate \
  -H "Content-Type: application/json" \
  -d '{"model": "qwen3-coder:30b", "prompt": "test"}'
```

### 3. Test Debugging

**Test Failure Investigation:**
```bash
# Run failing test with backtrace
RUST_BACKTRACE=1 cargo test failing_test_name

# Run with verbose output
cargo test failing_test_name -- --nocapture

# Debug specific test
cargo test failing_test_name -- --exact

# Run tests in gdb for debugging
rust-gdb --args cargo test failing_test_name
```

## Performance Development

### 1. Performance Monitoring

**Build Performance Tracking:**
```bash
#!/bin/bash
# monitor_build_performance.sh

echo "Build performance test started at $(date)"

# Measure build time
time cargo build --release

# Measure test time
time cargo test --all

# Check binary size
ls -lh target/release/frozen-duckdb

echo "Performance test completed at $(date)"
```

**Runtime Performance Profiling:**
```bash
# Profile application performance
cargo profdata --bin frozen-duckdb

# Memory profiling
valgrind --tool=massif cargo run -- complete --prompt "test"

# CPU profiling
perf record cargo run -- complete --prompt "test"
perf report
```

### 2. Benchmarking Development

**Custom Benchmarking:**
```rust
// Add to your development tests
#[cfg(test)]
mod performance_tests {
    use super::*;
    use frozen_duckdb::benchmark;

    #[test]
    fn benchmark_my_feature() {
        let duration = benchmark::measure_build_time(|| {
            // Your feature implementation
            Ok(())
        });

        // Assert performance meets requirements
        assert!(duration.as_millis() < 100, "Feature too slow: {:?}", duration);
    }
}
```

**Performance Regression Detection:**
```bash
#!/bin/bash
# detect_performance_regression.sh

# Record current performance
BUILD_TIME=$(time cargo build 2>&1 | grep real | awk '{print $2}')
TEST_TIME=$(time cargo test --quiet 2>&1 | grep real | awk '{print $2}')

# Check against baseline (if exists)
if [[ -f .performance_baseline ]]; then
    source .performance_baseline

    # Alert on significant regressions (>10% slowdown)
    if (( $(echo "$BUILD_TIME > $BASELINE_BUILD * 1.1" | bc -l) )); then
        echo "⚠️  Build performance regression detected!"
        exit 1
    fi
else
    # Create baseline
    echo "BASELINE_BUILD=$BUILD_TIME" > .performance_baseline
    echo "BASELINE_TEST=$TEST_TIME" >> .performance_baseline
fi

echo "✅ Performance within acceptable limits"
```

## Development Tools Setup

### 1. Database Development

**DuckDB CLI Setup:**
```bash
# Install DuckDB CLI for manual testing
# Already installed via package manager

# Create development database
duckdb dev.duckdb -c "
CREATE TABLE test_data AS SELECT 1 as id, 'test' as value;
SELECT * FROM test_data;
"
```

**Database Development Workflow:**
```sql
-- Test DuckDB functionality
SELECT version();

-- Test extensions
INSTALL parquet FROM community;
LOAD parquet;

-- Test custom functions
SELECT 'Hello ' || 'World' as greeting;
```

### 2. LLM Development

**Ollama Development Commands:**
```bash
# List available models
ollama list

# Test model directly
ollama run qwen3-coder:30b "Explain recursion in programming"

# Test embedding model
curl -s http://localhost:11434/api/embeddings \
  -H "Content-Type: application/json" \
  -d '{"model": "qwen3-embedding:8b", "prompt": "machine learning"}' \
  | jq '.embedding | length'

# Monitor model performance
ollama show qwen3-coder:30b
```

**Development Model Management:**
```bash
# Create development model
ollama create dev-coder -f ./Devfile

# Example Devfile for development
cat > Devfile << 'EOF'
FROM qwen3-coder:7b
PARAMETER temperature 0.8
PARAMETER top_p 0.9
SYSTEM "You are a development assistant helping with coding tasks."
EOF

# Use development model
CREATE MODEL('dev_coder', 'dev-coder', 'ollama');
```

## Code Quality Tools

### 1. Linting and Formatting

**Automated Code Quality:**
```bash
# Format all code
cargo fmt --all

# Check formatting
cargo fmt --all --check

# Lint with Clippy
cargo clippy --all-targets -- -D warnings

# Check with all features
cargo clippy --all-targets --all-features -- -D warnings
```

**Pre-commit Hooks:**
```bash
# Install pre-commit (if using)
pip install pre-commit
pre-commit install

# Manual hook execution
pre-commit run --all-files
```

### 2. Testing Strategy

**Core Team Testing Requirements:**
```bash
# Run tests multiple times to catch flaky behavior
for i in {1..3}; do
    echo "Test run $i of 3"
    cargo test --all
done

# Run with different configurations
cargo test --release --all
ARCH=x86_64 source prebuilt/setup_env.sh && cargo test --all
ARCH=arm64 source prebuilt/setup_env.sh && cargo test --all

# Run property tests
cargo test --test proptest_tests

# Run with insta snapshots
cargo test --test snapshot_tests
```

**Test Categories:**
```bash
# Unit tests (fast, focused)
cargo test --lib

# Integration tests (comprehensive, slower)
cargo test --test core_functionality_tests
cargo test --test arrow_tests
cargo test --test parquet_tests

# LLM tests (require Ollama)
cargo test --test flock_tests

# Performance tests
cargo test --test benchmark_tests
```

### 3. Documentation Testing

**Documentation Validation:**
```bash
# Build documentation
cargo doc --all-features

# Check for broken links
cargo doc --all-features --document-private-items

# Test code examples in documentation
cargo test --doc
```

## Development Best Practices

### 1. Code Organization

**Module Structure:**
```
src/
├── lib.rs              # Library entry point
├── main.rs             # CLI application
├── architecture.rs     # Architecture detection
├── benchmark.rs        # Performance measurement
├── env_setup.rs        # Environment validation
└── cli/
    ├── mod.rs          # CLI module organization
    ├── commands.rs     # Command definitions
    ├── dataset_manager.rs # Dataset operations
    └── flock_manager.rs   # LLM operations
```

**Import Organization:**
```rust
// Group imports logically
use anyhow::{Context, Result};
use clap::Parser;
use tracing::{error, info, warn};

// Standard library
use std::env;
use std::path::Path;

// External crates
use duckdb::Connection;
use serde_json;

// Local modules
use crate::architecture;
use crate::benchmark;
use crate::env_setup;
```

### 2. Error Handling

**Consistent Error Patterns:**
```rust
// Use anyhow for flexible error handling
pub fn validate_binary() -> Result<()> {
    let lib_dir = get_lib_dir()
        .ok_or_else(|| anyhow::anyhow!("DUCKDB_LIB_DIR not set"))?;

    // ... validation logic ...

    Ok(())
}

// Provide actionable error messages
pub fn setup_environment() -> Result<()> {
    if !is_configured() {
        return Err(anyhow::anyhow!(
            "Frozen DuckDB not configured. Please run: source prebuilt/setup_env.sh"
        ));
    }

    Ok(())
}
```

### 3. Testing Standards

**Test Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_basic_functionality() {
        // Test core functionality
        assert!(is_configured());
    }

    #[test]
    fn test_error_conditions() {
        // Test error handling
        let result = validate_binary_with_invalid_path();
        assert!(result.is_err());
    }

    proptest! {
        #[test]
        fn test_architecture_detection(arch in "x86_64|arm64|aarch64") {
            // Property-based testing
            std::env::set_var("ARCH", arch);
            assert_eq!(detect(), arch);
            std::env::remove_var("ARCH");
        }
    }
}
```

## Troubleshooting Development Issues

### 1. Compilation Issues

**Missing Dependencies:**
```bash
# Update dependencies
cargo update

# Check for missing crates
cargo check

# Install missing system dependencies
# macOS
brew install openssl@3

# Linux
sudo apt-get install libssl-dev pkg-config
```

**Build Cache Issues:**
```bash
# Clean build cache
cargo clean

# Clean and rebuild
cargo clean && cargo build

# Check target directory permissions
ls -la target/
```

### 2. Test Issues

**Test Data Setup:**
```bash
# Generate test data
cargo run -- download --dataset chinook --format parquet --output-dir test_data

# Verify test data exists
ls -la test_data/

# Check data integrity
duckdb test_data/chinook.duckdb -c "SELECT COUNT(*) FROM tracks;"
```

**Flaky Test Investigation:**
```bash
# Run test multiple times to identify flakiness
for i in {1..5}; do
    echo "Run $i:"
    cargo test specific_test
done

# Run with different configurations
RUST_LOG=debug cargo test specific_test
cargo test --release specific_test
```

### 3. LLM Development Issues

**Ollama Connection Issues:**
```bash
# Check Ollama server status
curl -s http://localhost:11434/api/version

# Check server process
ps aux | grep ollama

# Restart server if needed
killall ollama
ollama serve
```

**Model Loading Issues:**
```bash
# Check model status
ollama list

# Remove and re-pull problematic model
ollama rm qwen3-coder:30b
ollama pull qwen3-coder:30b

# Check disk space
df -h ~/.ollama/
```

## Performance Development

### 1. Development Performance

**Fast Development Cycle:**
```bash
# Quick check during development
cargo check

# Fast test during development
cargo test --lib

# Full test for validation
cargo test --all
```

**Incremental Development:**
```bash
# Use watch mode for automatic rebuilding
cargo watch -x check

# Test on file changes
cargo watch -x "test --all"

# Format on save (if using rustfmt)
cargo watch -x fmt
```

### 2. Performance Profiling

**Memory Profiling:**
```bash
# Install memory profiler
cargo install cargo-profdata

# Profile memory usage
cargo profdata --bin frozen-duckdb

# Analyze memory allocations
# Open generated flame graph in browser
```

**CPU Profiling:**
```bash
# Install CPU profiler
cargo install cargo-profdata

# Profile CPU usage
cargo profdata --bin frozen-duckdb

# Generate performance report
perf record cargo run -- complete --prompt "test"
perf report
```

## Continuous Integration Setup

### 1. Local CI Simulation

**Pre-commit Testing:**
```bash
#!/bin/bash
# pre_commit_tests.sh

echo "🧪 Running pre-commit tests..."

# Format check
cargo fmt --check || { echo "❌ Code formatting issues"; exit 1; }

# Lint check
cargo clippy --all-targets -- -D warnings || { echo "❌ Linting issues"; exit 1; }

# Test check
cargo test --all || { echo "❌ Tests failed"; exit 1; }

echo "✅ All pre-commit checks passed"
```

### 2. GitHub Actions Setup

**Basic CI Configuration:**
```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [macos-latest, ubuntu-latest]
        rust: [stable]

    steps:
    - uses: actions/checkout@v3

    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: ${{ matrix.rust }}
        override: true

    - name: Setup frozen DuckDB
      run: |
        source prebuilt/setup_env.sh
        echo "DUCKDB_LIB_DIR=$DUCKDB_LIB_DIR" >> $GITHUB_ENV
        echo "DUCKDB_INCLUDE_DIR=$DUCKDB_INCLUDE_DIR" >> $GITHUB_ENV

    - name: Build
      run: cargo build --all-targets

    - name: Test
      run: |
        cargo test --all
        cargo test --all  # Run twice for consistency
        cargo test --all  # Run three times (core team requirement)

    - name: Check formatting
      run: cargo fmt --all --check

    - name: Lint
      run: cargo clippy --all-targets -- -D warnings
```

## Summary

Setting up a development environment for Frozen DuckDB requires **careful configuration** of **Rust tools**, **DuckDB**, **Ollama**, and **development workflows**. The setup supports **fast development cycles**, **comprehensive testing**, and **performance optimization**.

**Key Setup Components:**
- **Rust toolchain**: Stable Rust with development tools
- **DuckDB**: Database engine with extension support
- **Ollama**: Local LLM server with required models
- **Frozen DuckDB**: Pre-compiled binaries for fast builds
- **Development tools**: Formatters, linters, and debuggers

**Development Workflow:**
- **Environment setup**: Persistent configuration for development
- **Code development**: Fast cycles with watch mode and hot reload
- **Testing strategy**: Multiple runs to catch flaky behavior
- **Performance monitoring**: Track build and runtime performance

**Quality Assurance:**
- **Code formatting**: Consistent style with rustfmt
- **Linting**: Comprehensive checks with clippy
- **Testing**: Unit, integration, and property-based tests
- **Performance validation**: Ensure SLO requirements are met

**Next Steps:**
1. Complete the [Testing Strategy Guide](./testing-strategy.md) for testing best practices
2. Review the [Coding Standards Guide](./coding-standards.md) for code quality
3. Study the [Architecture Decisions](./architecture-decisions.md) for design rationale
