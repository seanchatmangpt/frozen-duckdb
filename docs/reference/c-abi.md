# c-abi

Information-oriented reference for frozen-duckdb's host-to-engine C ABI.
Canonical law: `docs/design/boundary-abi-contract.md`; WASM projection:
`docs/design/wasm-abi-surface.md` (references below are pointers, not
restatements of law).

## The two instantiations

The boundary contract has exactly two instantiations of one contract:

- **WASM export surface (`fdb_*` family)** — PROPOSED / UNVERIFIED
  until FR-01 (a compiled WASM artifact + boundary court). The five
  export signatures, the linear memory map, and the host-VFS callback
  alignment live in `docs/design/wasm-abi-surface.md`, which is the
  canonical text for them; this doc does not restate them.
- **Native instantiation** — ALIVE. The engine is consumed through the
  `duckdb_*` C API: `crates/frozen-duckdb-sys` declares the raw
  symbols, `crates/frozen-duckdb/src/duckdb/` wraps them. ~95 distinct
  `duckdb_*` symbols are referenced across `src/duckdb/`
  (connection lifecycle, prepared statements, bind/append primitives,
  Arrow result extraction, config, appender, interrupt). This is the
  compiling, test-covered reality; the WASM surface is not.

Cross-reference: `boundary-abi-contract.md` (§1 instantiations, §2 ABI
rules, §7 native grounding).

## Boundary rules (summary; contract is canonical)

- Primitives, raw pointers, adjacent lengths only. No `Result`,
  `Option`, `Vec`, `String`, fat slices, C++ classes, exceptions, or
  unwinding across the line.
- Strings are UTF-8 `(const ptr, u64 len)` pairs, not required
  NUL-terminated; legacy NUL-terminated symbols get terminated on the
  engine side, never by the host.
- Zero-copy memory transfer via Arrow C structures (`ArrowArray` +
  `ArrowSchema`) on the `duckdb_arrow` / `duckdb_arrow_schema` /
  `duckdb_destroy_arrow` path.
- Every acquire has exactly one destroy (`duckdb_open_ext` /
  `duckdb_close`, `duckdb_destroy_prepare`, `duckdb_destroy_result`,
  `duckdb_destroy_arrow`, `duckdb_create_config` /
  `duckdb_destroy_config`, `duckdb_free` for engine-allocated
  strings). Hosts never free engine memory with a foreign allocator.
- Panics are caught at each export and converted to a status code
  (most specifically `ExecutionHalt`); nothing unwinds into the host.
- Error detail is never packed into the return; it is read from the
  per-handle error buffer (`duckdb_prepare_error`,
  `duckdb_appender_error`, `duckdb_extract_statements_error`).

## Status codes

Every boundary call returns exactly one `u32` from a closed set — the
same set in both instantiations (boundary contract §3, wasm surface §4):

| code | name | meaning |
|---|---|---|
| 0 | `Success` | call completed; out-params valid |
| 1 | `InvalidInput` | bad ptr/len, non-UTF-8, bad SQL, wrong params/type |
| 2 | `OutOfMemory` | engine allocation failed (incl. memory-limit breach) |
| 3 | `DiskExhausted` | persistent write failed for space/quota |
| 4 | `ExecutionHalt` | aborted: interrupt, timeout, or contained panic |

Closed set; `u32::MAX` reserved as `Unknown` — any other value is a
contract violation finding, not a status.

Native mapping (boundary contract §3): `duckdb_state` is
`DuckDBSuccess`/`DuckDBError`; `error.rs` converts the FFI error into
the typed Rust `Error` on the Rust side of the line; interrupt via
`duckdb_interrupt`, timeouts, and contained panics map to
`ExecutionHalt`.

## See Also

`docs/design/boundary-abi-contract.md` ·
`docs/design/wasm-abi-surface.md` ·
`docs/design/host-range-vfs-contract.md` · `api.md`
