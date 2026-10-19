# Extension ABI Hub: In-WASM Function Registration

Status: PROPOSED (design doc; nothing here is implemented).
Lane: W-EXT4. Related: `docs/design/wasm-opfs-architecture.md`.

## Summary

The stable DuckDB C extension API becomes the runtime extension
mechanism for the WASM build: the JS host registers scalar and table
functions into a live engine at runtime through function-pointer hooks.
No new C++ compile step; the engine stays a frozen artifact and
extension capability is injected through the ABI it already exports.

## Existing instantiation (ALIVE)

The Rust wrapper already implements this exact pattern natively.
The table-function registration family
(`crates/frozen-duckdb/src/duckdb/vtab/function.rs`):

- `duckdb_create_table_function()` — 12-call construction
  family entry (`TableFunctionBuilder::new`, line ~307).
- `set_name`, `add_parameter`, `add_named_parameter`,
  `add_result_column`, `set_function`, `set_bind`, `set_init`,
  `set_local_init`, `supports_pushdown`.
- `duckdb_register_table_function(con, ptr)`
  (`crates/frozen-duckdb/src/duckdb/vtab/mod.rs:162`).

The scalar-function set family
(`crates/frozen-duckdb/src/duckdb/vscalar/function.rs`):

- `duckdb_register_scalar_function_set(con, ptr)`
  (`vscalar/function.rs:29`), built via `new(name)`,
  `add_function`, `add_parameter`, `add_variadic_parameter`,
  `set_return_type`, `set_function`, `set_extra_info`.

The wrapper IS the existing instantiation of the hub pattern; the
WASM/host-callback form is the proposed extension of it.

## Proposed extension (host-callback over the boundary)

The JS host implements the hook side: it supplies callback shims
whose addresses are handed to the same registration family, so a
host-supplied table function behaves like any native one.

### C sketch (PROPOSED, from operator blueprint)

```c
/* PROPOSED — not compiled anywhere in this repo. */
/* Host registers a table function over the boundary. */
duckdb_table_function tf = duckdb_create_table_function();
duckdb_table_function_set_name(tf, "actor_stream_scan");
duckdb_table_function_add_parameter(tf, DUCKDB_TYPE(DUCKDB_TYPE_UBIGINT));
duckdb_table_function_add_result_column(tf, "seq", DUCKDB_TYPE(DUCKDB_TYPE_UBIGINT));
duckdb_table_function_add_result_column(tf, "payload", DUCKDB_TYPE(DUCKDB_TYPE_VARCHAR));
duckdb_table_function_set_bind(tf, host_bind_shim);
duckdb_table_function_set_init(tf, host_init_shim);
duckdb_table_function_set_function(tf, host_scan_shim);
duckdb_register_table_function(con, tf);
```

The `host_*_shim` symbols are trampolines: on the WASM side they
marshal bind/init/scan state to JS imports, stream rows into
data chunks, and surface host exceptions as `duckdb_table_function_set_error`
/ `duckdb_function_set_error` typed errors.

## Host-actor-stream concept

Host streams become SQL tables. A JS host sequence source
(actor events, log lines, OPFS-backed feeds) is registered as
`actor_stream_scan`-style table function; SQL then joins, filters,
and aggregates over live host data with the engine's own executor,
no ingestion copy into an OPFS database file first.

## Falsifiers

1. **Real scan**: a registered host table function must serve a
   real scan through the engine — e.g.
   `SELECT count(*) FROM actor_stream_scan(42)` returns the host's
   row count through real bind/init/scan execution, not a stub.
2. **Typed catalog error**: querying an unregistered name, e.g.
   `SELECT * FROM no_such_function(1)`, must return a typed catalog
   error (function not found), not a panic, silent empty relation,
   or fallback.
3. **Re-audit gate**: because the registration family is the
   highest-fragility FFI surface (the 12-call construction family;
   audit fact from the frozen-duckdb FFI audit), any WASM reshaping
   of these entry points (new shim ABI, changed marshaling) is
   treated as a change to that surface and gets re-audited before
   landing.
