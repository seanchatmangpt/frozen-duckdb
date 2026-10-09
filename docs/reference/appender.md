# Appender API

The `Appender` struct provides a fast write path for bulk inserts, bypassing
the SQL planner for `INSERT` statements.

## Creating an appender

`Connection::appender` and `Connection::appender_to_db` construct an
`Appender` bound to a table (optionally `schema.table`).

## Writing rows

- `append_row` — append one row from individual column values.
- `append_rows` — append many rows from an iterator of row tuples in a single
  call; this is the bulk path and is faster than a loop of `append_row`.
- `flush` — push buffered rows to the table. `Appender` also flushes when
  dropped (an error at drop time is swallowed; call `flush` explicitly when
  you need to observe the error).

## Example

```rust,no_run
# use frozen_duckdb::{Connection, Appender};
# fn demo(conn: &Connection) -> duckdb::Result<()> {
let mut appender = conn.appender("trades")?;
appender.append_rows(&[
    ("AAPL", 1, 229.5),
    ("MSFT", 2, 415.3),
])?;
appender.flush()?;
# Ok(())
# }
```
