# Scalar Functions (vscalar)

The `vscalar` feature exposes DuckDB's scalar UDF interface. Implement the
`VScalar` trait and register the function on a `Connection` or on the pool's
`DuckdbConnectionManager`.

## Signatures

`ScalarFunctionSignature` describes one signature:

- `exact` — fixed parameter list (`Vec<LogicalTypeHandle>`) plus a return
  type.
- `variadic` — one repeated parameter type plus a return type.

## Parameters and state

`ScalarParams` is the runtime parameter bundle passed to `VScalar::call`.
`get_scalar_extra_info` retrieves per-function extra info; `set_error` fails
the call with a DuckDB error.

## Registration

- `register_scalar_function` — register one `VScalar` implementation under a
  SQL name.
- `register_scalar_function_with_state` — same, with extra state available
  via `get_scalar_extra_info`.
- `register_scalar_function_set` — register a whole scalar function set
  (multiple signatures under one SQL name).

## Example

```rust,no_run
# use frozen_duckdb::{Connection, duckdb::vscalar::VScalar};
use duckdb::core::{LogicalTypeHandle, ScalarFunctionSignature, ScalarParams, VScalar};
# struct AddOne;
# impl VScalar for AddOne {
#     fn sig() -> Vec<ScalarFunctionSignature> { vec![] }
#     fn call(ctx, input, output) -> ::duckdb::Result<()> { unimplemented!() }
# }
# fn demo(conn: &Connection) -> duckdb::Result<()> {
conn.register_scalar_function::<AddOne>("add_one")?;
# Ok(())
# }
```
