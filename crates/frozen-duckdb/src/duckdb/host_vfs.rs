//! KD-06 first slice (LOCAL): host-range VFS contract, local filesystem slice.
//!
//! Contract: `docs/design/host-range-vfs-contract.md` (ARD §3.2.A).
//! This slice proves the open/read/size callback mechanism against a
//! local file (`std::fs`) via the table function
//! `read_local_range(file, offset, length)`. The HTTP host-callback
//! delegation (real `host-range://` scheme) remains KD-06-open.
//!
//! Semantics (contract + local-slice reading):
//! - unreadable file -> typed `InvalidInput` bind error, no silent-zero row
//! - `offset < 0` or `length <= 0` -> typed `InvalidInput`
//! - offset past EOF -> empty result (0 rows), contract "EOF -> empty"
//! - range crossing EOF -> clamped partial range (remaining bytes)
//! - output `(data BLOB, offset BIGINT, length BIGINT)`; rows carry
//!   exact bytes of up-to-CHUNK emission.

use crate::duckdb::core::{DataChunkHandle, Inserter, LogicalTypeHandle, LogicalTypeId};
use crate::duckdb::vtab::{BindInfo, InitInfo, TableFunctionInfo, VTab};
use crate::duckdb::Result;
use std::fs::File;
use std::os::unix::fs::FileExt;
use std::sync::atomic::{AtomicU64, Ordering};

/// Closed-set status vocabulary (contract status table) mapped to typed
/// errors — no numeric-only channel, no silent-zero return.
#[derive(Debug)]
pub enum HostRangeStatus {
    /// Bad parameters or unreadable file.
    InvalidInput(String),
    /// Short read inside a clamped range (file shrank under us).
    OutOfRange(String),
}

impl std::fmt::Display for HostRangeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostRangeStatus::InvalidInput(m) => write!(f, "InvalidInput: {m}"),
            HostRangeStatus::OutOfRange(m) => write!(f, "OutOfRange: {m}"),
        }
    }
}

impl std::error::Error for HostRangeStatus {}

/// Bind data = the open/read/size callback analog: open resolves the
/// path, size from `metadata()`, read fills exactly the clamped range.
pub struct ReadLocalRangeBindData {
    data: Vec<u8>,
    resolved_offset: u64,
    /// True when offset >= file size (EOF -> empty result).
    at_eof: bool,
}

/// Init data: scan progress across func invocations.
pub struct ReadLocalRangeInitData {
    emitted: AtomicU64,
}

/// Emission slices in bytes per row.
const CHUNK: u64 = 2048;

fn param_i64(
    v: &crate::duckdb::vtab::Value,
    name: &str,
) -> std::result::Result<i64, Box<dyn std::error::Error>> {
    Ok(v.to_int64())
}

impl VTab for ReadLocalRangeVTab {
    type InitData = ReadLocalRangeInitData;
    type BindData = ReadLocalRangeBindData;

    fn bind(bind: &BindInfo) -> Result<Self::BindData, Box<dyn std::error::Error>> {
        if bind.get_parameter_count() != 3 {
            return Err(Box::new(HostRangeStatus::InvalidInput(format!(
                "read_local_range expects 3 parameters, got {}",
                bind.get_parameter_count()
            ))));
        }
        let path = bind.get_parameter(0).to_string();
        let offset = param_i64(&bind.get_parameter(1), "offset")?;
        let length = param_i64(&bind.get_parameter(2), "length")?;

        if offset < 0 {
            return Err(Box::new(HostRangeStatus::InvalidInput(format!(
                "offset must be >= 0, got {offset}"
            ))));
        }
        if length <= 0 {
            return Err(Box::new(HostRangeStatus::InvalidInput(format!(
                "length must be > 0, got {length}"
            ))));
        }
        let (offset, length) = (offset as u64, length as u64);

        // open callback analog: resolve the path (no wire I/O).
        let f = File::open(&path).map_err(|e| {
            Box::new(HostRangeStatus::InvalidInput(format!("open '{path}' failed: {e}")))
                as Box<dyn std::error::Error>
        })?;
        // size callback analog: content length via metadata, resolved
        // once per handle.
        let size = f.metadata()?.len();

        // EOF -> empty: zero-row scan.
        if offset >= size {
            return Ok(ReadLocalRangeBindData {
                data: Vec::new(),
                resolved_offset: offset,
                at_eof: true,
            });
        }

        // Clamp at EOF (local-slice semantics for range crossing EOF).
        let end = size.min(offset.saturating_add(length));
        let read_len = (end - offset) as usize;

        // read callback analog: exactly read_len bytes at offset; a
        // short read is retried, 0 bytes escalates typed OutOfRange.
        let mut data = vec![0u8; read_len];
        let mut filled = 0usize;
        while filled < read_len {
            let n = f.read_at(&mut data[filled..], offset + filled as u64)?;
            if n == 0 {
                return Err(Box::new(HostRangeStatus::OutOfRange(format!(
                    "short read: 0 bytes at absolute offset {}",
                    offset + filled as u64
                ))));
            }
            filled += n;
        }

        Ok(ReadLocalRangeBindData {
            data,
            resolved_offset: offset,
            at_eof: false,
        })
    }

    fn init(_init: &InitInfo) -> Result<Self::InitData, Box<dyn std::error::Error>> {
        Ok(ReadLocalRangeInitData {
            emitted: AtomicU64::new(0),
        })
    }

    fn func(
        func: &TableFunctionInfo<Self>,
        output: &mut DataChunkHandle,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let init_data = func.get_init_data();
        let bind_data = func.get_bind_data();
        let total_rows = bind_data.data.len().div_ceil(CHUNK as usize) as u64;

        // Emit the next slice of rows; DuckDB re-invokes until we
        // return a 0-length chunk.
        let start_row = init_data.emitted.load(Ordering::Relaxed);
        if start_row >= total_rows {
            output.set_len(0);
            return Ok(());
        }
        let capacity = output_flat_capacity();
        let end_row = total_rows.min(start_row + capacity as u64);
        let mut n = 0u64;
        for row in start_row..end_row {
            let s = (row * CHUNK) as usize;
            let e = bind_data.data.len().min(s + CHUNK as usize);
            output.flat_vector(0).insert(n as usize, &bind_data.data[s..e]);
            output.flat_vector(1).insert(n as usize, (bind_data.resolved_offset + s as u64) as i64);
            output.flat_vector(2).insert(n as usize, (e - s) as i64);
            n += 1;
        }
        init_data.emitted.store(end_row, Ordering::Relaxed);
        output.set_len(n as usize);
        Ok(())
    }
}

/// Rows per func invocation (DuckDB STANDARD_VECTOR_SIZE conservative).
fn output_flat_capacity() -> usize {
    2048
}

/// Registers `read_local_range` on the connection. Idempotent per
/// connection; re-registration with a different implementation under
/// the same name is the engine's `InvalidInput` path (contract).
pub fn register(conn: &crate::duckdb::Connection) -> crate::duckdb::Result<()> {
    conn.register_table_function::<ReadLocalRangeVTab>("read_local_range")
}
