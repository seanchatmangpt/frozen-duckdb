# r2d2 Connection Pooling

H1 matches filename, H2 sections. `DuckdbConnectionManager` implements
`r2d2::ManageConnection` for `Connection` handles, so a web server or worker
pool can share a small set of DuckDB connections across threads.

## Creating a manager

- `DuckdbConnectionManager::file` — open a pooled manager over a database file
  (same semantics as `Connection::open`).
- `file_with_flags` — same, but takes a `Config` for page sizes, memory limit,
  access mode, etc.
- `DuckdbConnectionManager::memory` — pooled in-memory database.
- `memory_with_flags` — pooled in-memory database with a `Config`.

## Pool usage

Wrap the manager in `r2d2::Pool::new(manager)`, then `pool.get()` hands out
`PooledConnection` wrappers. Every `get` reuses an existing connection; the
manager serializes access with an internal `Arc<Mutex<Connection>>`, so
concurrent checkouts queue instead of corrupting state.

## Registering extensions into the pool

Because the manager owns one shared `Connection`, extension registration
happens once on that connection:

- `register_table_function` — register a table function into the pooled
  connection.
- `register_scalar_function` — register a `VScalar` scalar function into the
  pooled connection (see
  [scalar-functions](scalar-functions.md)).

## When not to pool

DuckDB allows a single read/write process per database file. If you need
multi-process access, use `DuckdbConnectionManager::file` with
`AccessMode.READ_ONLY` via `file_with_flags`, or give each process its own
copy of the file.
