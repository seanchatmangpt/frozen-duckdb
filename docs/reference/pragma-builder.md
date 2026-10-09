# Pragma SQL Builder

`Sql` in `crate::duckdb::pragma` is the escaped SQL builder used by every
`Connection` pragma method. It accumulates a PRAGMA statement piece by piece
with injection-safe push methods, so pragma values never interpolate raw
strings into the statement.

## Building a statement

- `Sql::new` — start an empty statement.
- `push_pragma` — emit the `PRAGMA` keyword plus schema qualification.
- `push_keyword` — push a validated PRAGMA keyword.
- `push_schema_name` — push a `DatabaseName` (main/temp) qualification.
- `push_identifier` — push an escaped identifier.
- `push_value` — push an arbitrary `ToSql` value.
- `push_string_literal` / `push_int` / `push_real` — typed literal pushes.
- `push_space`, `push_dot`, `push_equal_sign`, `open_brace`, `close_brace` —
  punctuation helpers.

## Rendering

`as_str` renders the accumulated statement.

## Connection methods

The `Connection` methods that use this builder:

- `pragma` — run a PRAGMA without rows.
- `pragma_query` — run a PRAGMA and iterate rows.
- `pragma_query_value` — run a PRAGMA and read a single value.
- `pragma_update` — set a pragma setting.
- `pragma_update_and_check` — set a pragma and check the returned row.
