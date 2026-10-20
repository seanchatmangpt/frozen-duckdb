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

DuckDB allows a single process per database file — full stop, readers
included. The file lock is exclusive: while any process holds the file open
(read/write OR `AccessMode::ReadOnly`), every other process's
`Connection::open` fails IMMEDIATELY (~4 ms; no blocking, no timeout) with:

```text
IO Error: Could not set lock on file "<db>": Conflicting lock is held in
"<holder>" (PID <n>) ... See also
https://duckdb.org/docs/current/connect/concurrency
```

Witnessed against the frozen 2.0.0-dev engine with two real OS processes
(receipt: `docs/sjira/v2.0.0-dev/receipts/USER-FINDINGS-U10.md`): a
READ_ONLY open is rejected while a writer holds the file, two READ_ONLY
processes with no writer open fine concurrently, `kill -9` mid-transaction
recovers cleanly on next open (WAL replay, uncommitted work lost, no stale
lock), and concurrent reopen churn succeeds exactly when the opener wins the
lock.

The only multi-process mode is multiple readers with NO writer: open every
process with `AccessMode::ReadOnly` via `file_with_flags`. For read/write
multi-process workloads, either give each process its own copy of the file,
or have non-holders treat the lock error above as a retryable signal and
poll until the holder exits.
