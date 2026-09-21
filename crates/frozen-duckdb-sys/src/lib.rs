#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(deref_nullptr)]
#![allow(improper_ctypes)]

#[allow(clippy::all)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindgen.rs"));
}
#[allow(clippy::all)]
pub use bindings::*;

mod string;
pub use string::*;

pub const DuckDBError: duckdb_state = duckdb_state_DuckDBError;
pub const DuckDBSuccess: duckdb_state = duckdb_state_DuckDBSuccess;

pub use self::error::*;
mod error;
