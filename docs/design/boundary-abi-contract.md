# boundary-abi-contract

Status: DRAFT (P3, PRD lane). Native side = grounded in repo reality; WASM side =
PROPOSED and UNVERIFIED throughout.

## 1. Purpose and scope

This contract defines the single host-to-engine boundary for frozen-duckdb: the only
permitted crossing points between a host process (or host WASM runtime) and the DuckDB
engine. It exists so that (a) the native FFI today and (b) a future WASM engine export
surface describe ONE contract, not two divergent ones
([[sdk-parity-register]]: probe receipts are the register of truth; the boundary court
below is the gate).

Two instantiations are covered:

- **Native (ALIVE, receipted by the crate itself):** the `duckdb_*` C API consumed via
  `crates/frozen-duckdb-sys` and wrapped by
  `crates/frozen-duckdb/src/duckdb/`. This is the existing, compiling,
  test-covered surface: ~95 distinct `duckdb_*` symbols are referenced across
  `src/duckdb/` (connection lifecycle, prepared statements, bind/append primitives,
  Arrow result extraction, config, appender, interrupt).
- **WASM export surface (PROPOSED, UNVERIFIED):** `init_engine`,
  `execute_vectorized_query`, `ingest_batch`, `close_engine`. These names appear in the
  campaign planning only; there is no WASM engine crate, no export, and no court in
  this repo yet. Every claim about them in this document is UNVERIFIED until a
  compiled WASM artifact exports them and a boundary court faults each one.

## 2. ABI rules (pure unboxed C ABI)

The boundary carries primitives, pointers, and lengths only. No ownership transfer
across the boundary; no Rust- or C++-mangled names; no panics across the boundary.

- **Types permitted:** fixed-width integers (`i32`/`u32`/`i64`/`u64`/`usize`-as-`u64`),
  booleans as `u8`, raw pointers (`const T*`, `T*`), and explicit lengths
  (`len: u64` bytes, always adjacent to the pointer they qualify).
- **Types banned at the boundary:** `Result`, `Option`, `Vec`, `String`, slices as
  fat pointers, `Box<dyn Error>`, C++ classes/references, exceptions, unwinding.
  Rust-side wrappers convert; they never leak. (The native instantiation already
  honors this: `crates/frozen-duckdb-sys` declares `duckdb_state` as
  `DuckDBSuccess`/`DuckDBError` integers; `error.rs` converts `ffi::Error` into the
  typed `Error` enum on the Rust side of the line.)
- **Strings:** `(const char* ptr, u64 len)` pairs, UTF-8, not required NUL-terminated
  when `len` is supplied; if a NUL-terminated legacy symbol is called, the wrapper
  performs the termination inside the engine side, never in the host.
- **Memory transfer: zero-copy shared buffer.** Column data crosses as Arrow C
  structures (`ArrowArray` + `ArrowSchema`, matching the existing
  `duckdb_arrow` / `duckdb_arrow_schema` / `duckdb_destroy_arrow` path in
  `arrow_batch.rs` / `polars_dataframe.rs`): the host allocates, the engine fills in
  place, buffers are released by the side that owns the allocator via the paired
  `_destroy`/`_free` symbol. The engine never copies a column buffer to hand it to the
  host and the host never copies to hand data to the engine (batch ingest follows the
  same rule). Release protocol: the producer fills, the consumer signals release via
  the ABI function; no implicit lifetimes, no GC.
- **Panic/exception containment:** the engine side catches unwind at each export and
  converts it to the most specific status code below (ExecutionHalt for
  user-interrupt-like aborts, ExecutionHalt-or-narrower per court), never propagating
  a panic into the host.
- **Lifecycle pairing:** every acquire symbol has exactly one destroy symbol
  (`duckdb_open_ext`/`duckdb_close`,
  `duckdb_extract_statements`/`duckdb_destroy_extracted`,
  `duckdb_destroy_prepare`, `duckdb_destroy_result`,
  `duckdb_destroy_arrow`, `duckdb_destroy_data_chunk`,
  `duckdb_create_config`/`duckdb_destroy_config`, `duckdb_free` for engine-allocated
  strings). Hosts must not free engine-allocated memory with a foreign allocator and
  vice versa.

## 3. Status codes (structured)

Every boundary call returns exactly one `u32` status code. Error detail is NOT carried
in the return; it is fetched from a thread-safe error buffer (Section 4).

| code | name | meaning |
|---|---|---|
| 0 | `Success` | call completed; out-params valid |
| 1 | `InvalidInput` | malformed input: bad ptr/len, non-UTF-8, bad SQL, wrong params/type |
| 2 | `OutOfMemory` | allocation inside the engine failed |
| 3 | `DiskExhausted` | persistent storage write failed for lack of space/quota |
| 4 | `ExecutionHalt` | execution aborted: interrupt, timeout, or contained panic |

The code set is closed. `u32::MAX` is reserved as `Unknown` so a corrupted return can
be distinguished from a real code; `Unknown` is always a boundary-contract violation
finding, never an expected result.

## 4. Error buffer

- Each engine instance exposes a thread-safe error-buffer fetch: a single ABI call
  taking the handle and returning `(const char* ptr, u64 len)` valid until the next
  call on the same handle. Native instantiation: the existing per-handle error
  accessors (`duckdb_prepare_error`, `duckdb_appender_error`,
  `duckdb_extract_statements_error`) generalize this pattern per object; the contract
  requires ONE uniform `error_fetch(handle) -> (ptr, len)` per engine handle.
- Thread safety: the buffer is per-handle and internally synchronized; concurrent
  calls on different handles never contend; a fetch concurrent with a faulting call
  returns either the empty string or the completed message, never a torn one.
- Detail is best-effort: the status code is the contract; the message is diagnostic.
  An absent message with a non-`Success` code is lawful.

## 5. Native error mapping (grounded)

The typed enum in `crates/frozen-duckdb/src/duckdb/error.rs` is the native
instantiation's error vocabulary. Mapping to status codes (many-to-one; the
status code is lossy by design, detail stays in the error buffer):

| `Error` variant (error.rs) | status code |
|---|---|
| `DuckDBFailure` | disk -> `DiskExhausted`; alloc -> `OutOfMemory`; else `ExecutionHalt` |
| `FromSqlConversionFailure`, `InvalidColumnType`, `ArrowTypeToDuckdbType` | `InvalidInput` |
| `IntegralValueOutOfRange`, `InvalidParameterCount`, `InvalidParameterName`, | `InvalidInput` |
| `InvalidColumnIndex`, `InvalidColumnName`, `MultipleStatement`, `InvalidQuery`, | `InvalidInput` |
| `Utf8Error`, `NulError`, `InvalidPath` | `InvalidInput` |
| `ToSqlConversionFailure`, `AppendError` | `InvalidInput` |
| `ExecuteReturnedResults`, `QueryReturnedNoRows`, `QueryReturnedMoreThanOneRow`, | `InvalidInput` |
| `StatementChangedRows`, `StatementNotExecuted` | `InvalidInput` (caller-shape) |
| interrupt via `duckdb_interrupt`, timeouts, contained panic | `ExecutionHalt` |

The mapping is implemented on the Rust side of the boundary (wrapper layer), so the
native crate's public Rust API keeps its typed `Error` while the raw C surface carries
only `u32` codes.

## 6. Falsifier: the boundary court

A boundary claim is not landed until the court proves every status code reachable by
a REAL faulting input against a REAL engine instance (Chicago school; a code never
observed is a code that does not exist):

- `InvalidInput`: fault, e.g. a non-UTF-8 SQL byte string, a param-count mismatch
  (`InvalidParameterCount` path), a null/short buffer with nonzero len.
- `OutOfMemory`: fault via a forced allocation failure (huge allocation input, e.g.
  an Arrow buffer with a length beyond available memory) — must observe `OutOfMemory`
  or, if the substrate converts it, name the conversion explicitly in the court
  receipt.
- `DiskExhausted`: fault via a database file on a full/quota'd filesystem (tmpfs with
  a size cap) and a write-forcing statement.
- `ExecutionHalt`: fault via `duckdb_interrupt` (native, existing symbol) mid-query,
  or a contained panic injection on the WASM side.
- `Success`: the negative control — the same input minus the fault returns `Success`.

Court properties: exit 1 on any unreachable code or any observed `Unknown`; real
subprocess/engine, no mocks; each case receipts the input, the returned code, and the
error-buffer content. All four WASM export names above are UNVERIFIED until this court
runs against a compiled WASM artifact.

## 7. Standing summary

| claim | standing |
|---|---|
| native FFI = unboxed C ABI, zero-copy Arrow, lifecycle pairing | ALIVE (compiling; in-tree) |
| status codes + error buffer (native) | PARTIAL_ALIVE (no uniform `error_fetch`) |
| native error mapping table | PARTIAL_ALIVE (specified here; wrapper not yet implemented) |
| boundary court run | UNKNOWN (never run) |
| WASM exports init_engine / execute_vectorized_query / | UNVERIFIED (PROPOSED only) |
| ingest_batch / close_engine | UNVERIFIED (PROPOSED only) |
