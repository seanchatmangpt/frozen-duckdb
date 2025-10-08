//! # CLI Module for Frozen DuckDB
//!
//! This module contains the command-line interface implementation,
//! organized into logical sub-modules for better maintainability.

pub mod commands;
pub mod dataset_manager;
pub mod flock_manager;

pub use commands::*;
pub use dataset_manager::*;
pub use flock_manager::*;
