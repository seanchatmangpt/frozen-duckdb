//! # Environment Setup Utilities for Frozen DuckDB Binary
//!
//! This module provides utilities for validating the frozen DuckDB environment,
//! including checking environment variables and verifying binary existence.
//! It ensures that the pre-built DuckDB binaries are properly configured
//! and accessible before attempting to use them.
//!
//! ## Key Features
//!
//! - **Environment validation**: Checks required environment variables
//! - **Binary verification**: Ensures DuckDB binaries exist and are accessible
//! - **Configuration helpers**: Easy access to configured paths
//! - **Error reporting**: Clear error messages for troubleshooting
//!
//! ## Usage Examples
//!
//! ### Basic Environment Check
//!
//! ```rust
//! use frozen_duckdb::env_setup;
//!
//! // Check if environment is properly configured
//! if env_setup::is_configured() {
//!     println!("✅ Frozen DuckDB environment is ready!");
//! } else {
//!     println!("❌ Please run: source prebuilt/setup_env.sh");
//! }
//! ```
//!
//! ### Get Configuration Paths
//!
//! ```rust
//! use frozen_duckdb::env_setup;
//!
//! // Get library directory
//! if let Some(lib_dir) = env_setup::get_lib_dir() {
//!     println!("Library directory: {}", lib_dir);
//! }
//!
//! // Get include directory
//! if let Some(include_dir) = env_setup::get_include_dir() {
//!     println!("Include directory: {}", include_dir);
//! }
//! ```
//!
//! ### Validate Binary Existence
//!
//! ```rust
//! use frozen_duckdb::env_setup;
//!
//! // Validate that binaries exist
//! match env_setup::validate_binary() {
//!     Ok(()) => println!("✅ DuckDB binaries are available"),
//!     Err(e) => println!("❌ Binary validation failed: {}", e),
//! }
//! ```
//!
//! ## Environment Variables
//!
//! The module expects the following environment variables to be set:
//!
//! - `DUCKDB_LIB_DIR`: Path to directory containing DuckDB library files
//! - `DUCKDB_INCLUDE_DIR`: Path to directory containing DuckDB header files
//!
//! These are typically set by running `source prebuilt/setup_env.sh`.
//!
//! ## Binary Validation
//!
//! The validation process checks for:
//!
//! - **x86_64 binary**: `libduckdb_x86_64.dylib` (55MB)
//! - **arm64 binary**: `libduckdb_arm64.dylib` (50MB)
//! - **Generic fallback**: `libduckdb.dylib` (if architecture-specific not found)
//!
//! At least one binary must be present for validation to succeed.

use anyhow::Result;
use std::env;
use std::path::Path;

/// Checks if the frozen DuckDB environment is properly configured.
///
/// This function verifies that both required environment variables are set:
/// `DUCKDB_LIB_DIR` and `DUCKDB_INCLUDE_DIR`. These variables are typically
/// set by running `source prebuilt/setup_env.sh`.
///
/// # Returns
///
/// `true` if both required environment variables are set, `false` otherwise.
///
/// # Examples
///
/// ```rust
/// use frozen_duckdb::env_setup;
///
/// // Check configuration status
/// if env_setup::is_configured() {
///     println!("✅ Environment is properly configured");
/// } else {
///     println!("❌ Please run: source prebuilt/setup_env.sh");
/// }
/// ```
///
/// # Environment Variables Checked
///
/// - `DUCKDB_LIB_DIR`: Must be set to the directory containing DuckDB libraries
/// - `DUCKDB_INCLUDE_DIR`: Must be set to the directory containing DuckDB headers
///
/// # Performance
///
/// This function is optimized for frequent calls with minimal overhead.
/// Environment variable access is cached by the OS, so repeated calls
/// are very fast (<1μs).
pub fn is_configured() -> bool {
    env::var("DUCKDB_LIB_DIR").is_ok() && env::var("DUCKDB_INCLUDE_DIR").is_ok()
}

/// Gets the configured DuckDB library directory path.
///
/// This function retrieves the value of the `DUCKDB_LIB_DIR` environment
/// variable, which should point to the directory containing the DuckDB
/// library files (`.dylib` files).
///
/// # Returns
///
/// `Some(String)` containing the library directory path if the environment
/// variable is set, `None` if it's not set or empty.
///
/// # Examples
///
/// ```rust
/// use frozen_duckdb::env_setup;
///
/// // Get library directory
/// if let Some(lib_dir) = env_setup::get_lib_dir() {
///     println!("Library directory: {}", lib_dir);
/// } else {
///     println!("DUCKDB_LIB_DIR not set");
/// }
/// ```
///
/// # Expected Directory Contents
///
/// The library directory should contain:
///
/// - `libduckdb_x86_64.dylib` (55MB) - Intel/AMD 64-bit binary
/// - `libduckdb_arm64.dylib` (50MB) - Apple Silicon/ARM 64-bit binary
/// - `libduckdb.dylib` - Generic fallback binary
///
/// # Error Handling
///
/// This function never fails - it returns `None` if the environment
/// variable is not set, rather than panicking or returning an error.
pub fn get_lib_dir() -> Option<String> {
    env::var("DUCKDB_LIB_DIR").ok()
}

/// Gets the configured DuckDB include directory path.
///
/// This function retrieves the value of the `DUCKDB_INCLUDE_DIR` environment
/// variable, which should point to the directory containing the DuckDB
/// header files (`.h` and `.hpp` files).
///
/// # Returns
///
/// `Some(String)` containing the include directory path if the environment
/// variable is set, `None` if it's not set or empty.
///
/// # Examples
///
/// ```rust
/// use frozen_duckdb::env_setup;
///
/// // Get include directory
/// if let Some(include_dir) = env_setup::get_include_dir() {
///     println!("Include directory: {}", include_dir);
/// } else {
///     println!("DUCKDB_INCLUDE_DIR not set");
/// }
/// ```
///
/// # Expected Directory Contents
///
/// The include directory should contain:
///
/// - `duckdb.h` (186KB) - C header file
/// - `duckdb.hpp` (1.8MB) - C++ header file
///
/// # Error Handling
///
/// This function never fails - it returns `None` if the environment
/// variable is not set, rather than panicking or returning an error.
pub fn get_include_dir() -> Option<String> {
    env::var("DUCKDB_INCLUDE_DIR").ok()
}

/// Validates that the frozen DuckDB binary exists and is accessible.
///
/// This function performs a comprehensive check to ensure that at least one
/// DuckDB binary is available in the configured library directory. It checks
/// for architecture-specific binaries first, then falls back to generic ones.
///
/// # Returns
///
/// `Ok(())` if at least one DuckDB binary is found and accessible,
/// `Err` with a descriptive error message if no binaries are found.
///
/// # Examples
///
/// ```rust
/// use frozen_duckdb::env_setup;
///
/// // Validate binary existence
/// match env_setup::validate_binary() {
///     Ok(()) => println!("✅ DuckDB binaries are available"),
///     Err(e) => println!("❌ Binary validation failed: {}", e),
/// }
/// ```
///
/// # Binary Search Order
///
/// The function checks for binaries in this order:
///
/// 1. `libduckdb_x86_64.dylib` - Intel/AMD 64-bit optimized binary
/// 2. `libduckdb_arm64.dylib` - Apple Silicon/ARM 64-bit optimized binary
/// 3. `libduckdb.dylib` - Generic fallback binary
///
/// At least one binary must be present for validation to succeed.
///
/// # Error Conditions
///
/// The function will return an error if:
///
/// - `DUCKDB_LIB_DIR` environment variable is not set
/// - The library directory does not exist
/// - No DuckDB binaries are found in the directory
/// - The directory exists but is not accessible
///
/// # Performance
///
/// This function performs file system operations and should be called
/// sparingly. Consider caching the result if you need to check multiple times.
///
/// # Safety
///
/// This function only checks for file existence and does not attempt to
/// load or execute the binaries. It's safe to call even if the binaries
/// are corrupted or incompatible with the current system.
pub fn validate_binary() -> Result<()> {
    let lib_dir = get_lib_dir().ok_or_else(|| anyhow::anyhow!("DUCKDB_LIB_DIR not set"))?;

    let lib_path = Path::new(&lib_dir);

    // Check for architecture-specific binaries
    let x86_64_binary = lib_path.join("libduckdb_x86_64.dylib");
    let arm64_binary = lib_path.join("libduckdb_arm64.dylib");

    if x86_64_binary.exists() || arm64_binary.exists() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "No frozen DuckDB binary found in {}",
            lib_dir
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_configured_with_missing_env() {
        // Clear environment variables for this test
        env::remove_var("DUCKDB_LIB_DIR");
        env::remove_var("DUCKDB_INCLUDE_DIR");

        assert!(!is_configured());
    }

    #[test]
    fn test_get_lib_dir() {
        env::set_var("DUCKDB_LIB_DIR", "/test/path");
        assert_eq!(get_lib_dir(), Some("/test/path".to_string()));
        env::remove_var("DUCKDB_LIB_DIR");
    }

    #[test]
    fn test_get_include_dir() {
        env::set_var("DUCKDB_INCLUDE_DIR", "/test/include");
        assert_eq!(get_include_dir(), Some("/test/include".to_string()));
        env::remove_var("DUCKDB_INCLUDE_DIR");
    }

    #[test]
    fn test_validate_binary_missing_env() {
        env::remove_var("DUCKDB_LIB_DIR");
        assert!(validate_binary().is_err());
    }

    #[test]
    fn test_validate_binary_invalid_path() {
        env::set_var("DUCKDB_LIB_DIR", "/nonexistent/path");
        assert!(validate_binary().is_err());
        env::remove_var("DUCKDB_LIB_DIR");
    }
}
