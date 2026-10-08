# How to: Using frozen-duckdb

## Prerequisites


- crates/frozen-duckdb-builder/src/lib.rs::ensure_binary (function)

- crates/frozen-duckdb-sys/build.rs::main (function)

- crates/frozen-duckdb-sys/build.rs::write_to_out_dir (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::ArrowArray (struct)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::ArrowSchema (struct)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_add_aggregate_function_to_set (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_add_replacement_scan (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_add_scalar_function_to_set (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_aggregate_function_add_parameter (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_aggregate_function_get_extra_info (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_aggregate_function_set_destructor (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_aggregate_function_set_error (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_aggregate_function_set_extra_info (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_aggregate_function_set_functions (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_aggregate_function_set_name (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_aggregate_function_set_return_type (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_aggregate_function_set_special_handling (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_blob (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_bool (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_data_chunk (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_date (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_default (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_default_to_chunk (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_double (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_float (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_hugeint (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_int16 (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_int32 (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_int64 (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_int8 (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_interval (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_null (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_time (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_timestamp (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_uhugeint (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_uint16 (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_uint32 (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_uint64 (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_uint8 (function)

- crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs::duckdb_append_value (function)


## Steps


1. Use `ensure_binary` from `crates/frozen-duckdb-builder/src/lib.rs`.

2. Use `main` from `crates/frozen-duckdb-sys/build.rs`.

3. Use `write_to_out_dir` from `crates/frozen-duckdb-sys/build.rs`.

4. Use `duckdb_add_aggregate_function_to_set` from `crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs`.

5. Use `duckdb_add_replacement_scan` from `crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs`.

6. Use `duckdb_add_scalar_function_to_set` from `crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs`.

7. Use `duckdb_aggregate_function_add_parameter` from `crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs`.

8. Use `duckdb_aggregate_function_get_extra_info` from `crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs`.

9. Use `duckdb_aggregate_function_set_destructor` from `crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs`.

10. Use `duckdb_aggregate_function_set_error` from `crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs`.

11. Use `duckdb_aggregate_function_set_extra_info` from `crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs`.

12. Use `duckdb_aggregate_function_set_functions` from `crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs`.


## Verified snippet

<!-- The snippet slot carries code copied from the extracted code surface -->
<!-- (doc:Claim rows whose doc:attribute is "snippet"), never agent prose. -->

```rust
// crates/frozen-duckdb-builder/src/lib.rs :: ensure_binary
ensure_binary()
```

<!-- AGENT-COMMENTARY-BEGIN -->
<!-- The ONLY region an agent may write into. Bounds: <= 12 lines,    -->
<!-- <= 100 chars/line, no new code facts (any new symbol mentioned   -->
<!-- must exist in queries/ast_extract.rq output; the doc_quality     -->
<!-- court fails Phi_halluc > 0.001 otherwise). No tables, no         -->
<!-- signatures, no parameters, no error lists — AGENT-FORBIDDEN      -->
<!-- everywhere.                                                      -->
<!-- AGENT-COMMENTARY-END -->
