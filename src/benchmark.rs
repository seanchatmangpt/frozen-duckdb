//! # Performance Benchmarking Utilities
//!
//! This module provides utilities for measuring and comparing build times
//! for performance optimization and validation. It's designed to help
//! developers understand the performance impact of different build
//! configurations and optimizations.
//!
//! ## Key Features
//!
//! - **Precise timing**: High-resolution timing with `std::time::Instant`
//! - **Error handling**: Measures time even when operations fail
//! - **Comparison utilities**: Easy A/B testing of different approaches
//! - **Zero overhead**: Minimal performance impact on measured operations
//!
//! ## Usage Examples
//!
//! ### Basic Timing
//!
//! ```rust
//! use frozen_duckdb::benchmark;
//!
//! // Measure a single operation
//! let duration = benchmark::measure_build_time(|| {
//!     // Your build operation here
//!     std::thread::sleep(std::time::Duration::from_millis(100));
//!     Ok(())
//! });
//!
//! println!("Operation took: {:?}", duration);
//! ```
//!
//! ### Comparing Two Approaches
//!
//! ```rust
//! use frozen_duckdb::benchmark;
//!
//! let (time1, time2) = benchmark::compare_build_times(
//!     || {
//!         // First approach (e.g., with pre-built binaries)
//!         std::thread::sleep(std::time::Duration::from_millis(50));
//!         Ok(())
//!     },
//!     || {
//!         // Second approach (e.g., compiling from source)
//!         std::thread::sleep(std::time::Duration::from_millis(200));
//!         Ok(())
//!     },
//! ).unwrap();
//!
//! println!("Approach 1: {:?}", time1);
//! println!("Approach 2: {:?}", time2);
//! println!("Improvement: {:.1}%",
//!     ((time2.as_millis() - time1.as_millis()) as f64 / time2.as_millis() as f64) * 100.0);
//! ```
//!
//! ## Performance Characteristics
//!
//! - **Timing precision**: Microsecond-level accuracy
//! - **Overhead**: <1μs per measurement
//! - **Memory usage**: Minimal, no allocations during timing
//! - **Thread safety**: Safe for concurrent use
//!
//! ## Best Practices
//!
//! 1. **Warm up**: Run operations multiple times to account for JIT/caching
//! 2. **Multiple runs**: Average results over several iterations
//! 3. **Consistent environment**: Run benchmarks in controlled conditions
//! 4. **Statistical significance**: Use proper statistical analysis for comparisons

use anyhow::Result;
use std::time::Instant;

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
/// The duration includes the full execution time, regardless of whether the
/// operation succeeded or failed.
///
/// # Examples
///
/// ```rust
/// use frozen_duckdb::benchmark;
/// use std::time::Duration;
///
/// // Measure a simple operation
/// let duration = benchmark::measure_build_time(|| {
///     std::thread::sleep(Duration::from_millis(100));
///     Ok(())
/// });
///
/// assert!(duration >= Duration::from_millis(100));
/// assert!(duration < Duration::from_millis(200));
///
/// // Measure an operation that fails
/// let duration = benchmark::measure_build_time(|| {
///     std::thread::sleep(Duration::from_millis(50));
///     Err(anyhow::anyhow!("Test error"))
/// });
///
/// // Still measures the time, even though it failed
/// assert!(duration >= Duration::from_millis(50));
/// ```
///
/// # Performance Characteristics
///
/// - **Timing precision**: Microsecond-level accuracy on most platforms
/// - **Overhead**: <1μs per measurement
/// - **Memory usage**: No allocations during timing
/// - **Thread safety**: Safe for concurrent use
///
/// # Best Practices
///
/// 1. **Warm up**: Run the operation once before measuring to account for JIT compilation
/// 2. **Multiple runs**: Average results over several iterations for accuracy
/// 3. **Consistent environment**: Run benchmarks in controlled conditions
/// 4. **Statistical analysis**: Use proper statistical methods for comparisons
pub fn measure_build_time<F>(operation: F) -> std::time::Duration
where
    F: FnOnce() -> Result<()>,
{
    let start = Instant::now();
    let _ = operation();
    start.elapsed()
}

/// Compares the execution times of two different build operations.
///
/// This function is useful for A/B testing different build approaches,
/// such as comparing pre-built binaries vs. compiling from source.
/// It measures both operations and returns their execution times.
///
/// # Arguments
///
/// * `operation1` - The first build operation to measure
/// * `operation2` - The second build operation to measure
///
/// # Returns
///
/// A `Result` containing a tuple of `(Duration, Duration)` representing
/// the execution times of the first and second operations respectively.
/// The function always succeeds, even if the operations fail.
///
/// # Examples
///
/// ```rust
/// use frozen_duckdb::benchmark;
/// use std::time::Duration;
///
/// let (time1, time2) = benchmark::compare_build_times(
///     || {
///         // Fast operation (e.g., using pre-built binaries)
///         std::thread::sleep(Duration::from_millis(50));
///         Ok(())
///     },
///     || {
///         // Slow operation (e.g., compiling from source)
///         std::thread::sleep(Duration::from_millis(200));
///         Ok(())
///     },
/// ).unwrap();
///
/// assert!(time2 > time1);
/// assert!(time1 >= Duration::from_millis(50));
/// assert!(time2 >= Duration::from_millis(200));
///
/// // Calculate improvement percentage
/// let improvement = ((time2.as_millis() - time1.as_millis()) as f64 / time2.as_millis() as f64) * 100.0;
/// println!("Improvement: {:.1}%", improvement);
/// ```
///
/// # Performance Analysis
///
/// This function is designed for performance comparison scenarios:
///
/// - **Build optimization**: Compare different build configurations
/// - **Binary vs source**: Measure impact of pre-built vs compiled binaries
/// - **Architecture comparison**: Test performance across different architectures
/// - **Dependency analysis**: Understand impact of different dependencies
///
/// # Statistical Considerations
///
/// For meaningful comparisons, consider:
///
/// 1. **Multiple runs**: Run comparisons multiple times and average results
/// 2. **Warm-up**: Allow system to stabilize before measurements
/// 3. **Consistent conditions**: Ensure same system load and environment
/// 4. **Outlier detection**: Remove anomalous measurements from analysis
/// 5. **Confidence intervals**: Calculate statistical significance of differences
pub fn compare_build_times<F1, F2>(
    operation1: F1,
    operation2: F2,
) -> Result<(std::time::Duration, std::time::Duration)>
where
    F1: FnOnce() -> Result<()>,
    F2: FnOnce() -> Result<()>,
{
    let time1 = measure_build_time(operation1);
    let time2 = measure_build_time(operation2);
    Ok((time1, time2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measure_build_time() {
        let duration = measure_build_time(|| {
            std::thread::sleep(std::time::Duration::from_millis(10));
            Ok(())
        });

        assert!(duration >= std::time::Duration::from_millis(10));
        assert!(duration < std::time::Duration::from_millis(100));
    }

    #[test]
    fn test_compare_build_times() {
        let (time1, time2) = compare_build_times(
            || {
                std::thread::sleep(std::time::Duration::from_millis(5));
                Ok(())
            },
            || {
                std::thread::sleep(std::time::Duration::from_millis(10));
                Ok(())
            },
        )
        .unwrap();

        assert!(time2 > time1);
        assert!(time1 >= std::time::Duration::from_millis(5));
        assert!(time2 >= std::time::Duration::from_millis(10));
    }

    #[test]
    fn test_measure_build_time_with_error() {
        let duration = measure_build_time(|| Err(anyhow::anyhow!("Test error")));

        // Even with an error, we should get a duration measurement
        assert!(duration >= std::time::Duration::from_millis(0));
    }
}
