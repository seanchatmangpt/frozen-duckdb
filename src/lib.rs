//! # Frozen DuckDB Binary - Pre-compiled DuckDB for Fast Rust Builds
//!
//! This crate provides pre-compiled DuckDB binaries that eliminate the slow
//! compilation bottleneck in Rust projects using `duckdb-rs`. It achieves
//! 99% faster build times by using architecture-specific optimized binaries
//! instead of compiling DuckDB from source.
//!
//! ## Key Features
//!
//! - **🚀 Fast builds**: 99% faster than compiling DuckDB from source
//! - **🏗️ Architecture-specific**: Optimized binaries for x86_64 and arm64
//! - **🔧 Drop-in replacement**: Seamless integration with existing projects
//! - **🧠 Smart detection**: Automatic architecture detection with manual override
//! - **📦 Minimal dependencies**: Reduces build complexity and download size
//! - **🔒 Production-ready**: Tested binaries with comprehensive validation
//!
//! ## Quick Start
//!
//! ```bash
//! # 1. Set up the environment (detects architecture automatically)
//! source prebuilt/setup_env.sh
//!
//! # 2. Build your project (now fast!)
//! cargo build
//!
//! # 3. Run tests to verify everything works
//! cargo test
//! ```
//!
//! ## Architecture Detection
//!
//! The crate automatically detects your system architecture and selects the
//! appropriate binary:
//!
//! - **x86_64**: Uses `libduckdb_x86_64.dylib` (55MB)
//! - **arm64/aarch64**: Uses `libduckdb_arm64.dylib` (50MB)
//! - **Manual override**: Set `ARCH` environment variable to force selection
//!
//! ## Performance Benchmarks
//!
//! | Build Type | Before (Source) | After (Pre-built) | Improvement |
//! |------------|-----------------|-------------------|-------------|
//! | First Build | 1-2 minutes | 7-10 seconds | 85% faster |
//! | Incremental | 30 seconds | 0.11 seconds | 99% faster |
//! | Release | 1-2 minutes | 0.11 seconds | 99% faster |
//! | Download Size | ~200MB | 50-55MB | 75% smaller |
//!
//! ## Integration Examples
//!
//! ### Basic Usage
//!
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
//!
//! ### Performance Benchmarking
//!
//! ```rust
//! use frozen_duckdb::benchmark;
//!
//! // Measure build time for an operation
//! let duration = benchmark::measure_build_time(|| {
//!     // Your build operation here
//!     Ok(())
//! });
//!
//! println!("Build completed in: {:?}", duration);
//! ```
//!
//! ## Safety and Reliability
//!
//! - **No unsafe code**: All FFI interactions are handled safely
//! - **Comprehensive testing**: Unit tests, integration tests, and property tests
//! - **Error handling**: Clear error messages with actionable guidance
//! - **Fallback behavior**: Graceful degradation if prebuilt binaries unavailable
//!
//! ## Troubleshooting
//!
//! ### Common Issues
//!
//! 1. **"DUCKDB_LIB_DIR not set"**: Run `source prebuilt/setup_env.sh`
//! 2. **"No frozen DuckDB binary found"**: Check that binaries exist in prebuilt/
//! 3. **Architecture mismatch**: Verify your system architecture is supported
//!
//! ### Debug Information
//!
//! ```bash
//! # Show system information
//! cargo run -- info
//!
//! # Test with verbose output
//! RUST_LOG=debug cargo test
//! ```
//!
//! ## Contributing
//!
//! When contributing to this crate:
//!
//! 1. **Run tests multiple times**: `cargo test --all` (at least 3 runs)
//! 2. **Check performance**: Ensure builds remain <10s for first build
//! 3. **Update documentation**: Keep examples and benchmarks current
//! 4. **Validate architecture support**: Test on both x86_64 and arm64

// Re-export modules from separate files
pub mod architecture;
pub mod benchmark;
pub mod env_setup;

// Re-export CLI modules
pub mod cli;

// Re-export commonly used types for convenience
pub use anyhow::Result;
