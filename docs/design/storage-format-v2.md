# Storage Format v2 Design Notes (OPFS Persistence Layer)

Status: DESIGN — nothing in this document has been executed this wave.
All runtime claims marked UNVERIFIED until the burn-in court (BP2) runs.

## Ground truth from the compiled dylib

Source: `strings` over
`~/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb.dylib`
(arm64 dev build cached by frozen-duckdb-builder). Extracted 2026-10-19.
These strings are in the shipped binary; they are compile-time ground
truth for what the dev runtime gates, not runtime-proven behavior.

- "Nested schemas are only supported for storage versions v2.0.0 and
  higher."
- "Use an in-memory database, or ATTACH with (STORAGE_VERSION 'v2.0.0')"
- "Use an in-memory database, or ATTACH with (STORAGE_VERSION v2.0.0)"
  (unquoted-variant string also present)
- "Unsupported row group size %llu - row group sizes >= 122_880 are only
  supported with STORAGE_VERSION '1.2.0' or above."
- "Explicit provided STORAGE_VERSION ("%s") and ENCRYPTION_KEY
  (storage >= v1.4.0) quoted-variant + unquoted-variant strings
- "storage_version" (setting name string)

Interpretation (UNVERIFIED as runtime behavior):

- The dev runtime gates nested-schema storage behind storage version
  v2.0.0; a DB attached without an explicit v2.0.0 storage version
  cannot use nested schemas.
- The OPFS layer must pass `STORAGE_VERSION 'v2.0.0'` at ATTACH/OPEN
  for any database that uses nested schemas, and must NOT combine it
  with an explicit ENCRYPTION_KEY (the dylib declares them
  incompatible).
- Row groups >= 122,880 require storage >= 1.2.0; v2.0.0 satisfies
  this.

## Storage-version gating design

- Open path: `Config::with("storage_version", "v2.0.0")` via the
  crate's existing key/value passthrough
  (`crates/frozen-duckdb/src/duckdb/config.rs`, `Config::with`).
  UNVERIFIED: the exact accepted key spelling and value format
  ("v2.0.0" vs "2.0.0") have not been probed at runtime this wave.
- Fallback: if the key is refused at open, fall back to default
  storage version and reject nested-schema usage at the layer API
  (typed error, loud not quiet).
- The dylib strings mention ATTACH-time `STORAGE_VERSION` in the
  ATTACH clause form; whether the config-key form is accepted for the
  main database (vs ATTACH-only) is UNVERIFIED.

## Migration path for legacy DBs

- DuckDB's documented mechanism: `EXPORT DATABASE` (writes load.sql
  plus Parquet/CSV files) then `IMPORT DATABASE` into a new database
  created with the target storage version. Both names confirmed
  present in the dylib strings ("EXPORT DATABASE", "IMPORT DATABASE:
  load.sql may only contain COPY statements, found: %s").
- Planned flow (UNVERIFIED, not executed):
  1. Detect legacy storage version on open (UNVERIFIED: exact
     detection signal — error-on-open vs metadata read — is
     undetermined).
  Legacy open path: open the file read-only with the default
  storage version, run `EXPORT DATABASE TO '<tmp>'`, close, create
  the new file with `STORAGE_VERSION 'v2.0.0'`, `IMPORT DATABASE
  FROM '<tmp>'`, atomic-rename into place.
  2. Failure at any step leaves the original file untouched (the
     EXPORT/IMPORT runs against a temp dir, never in-place).
  3. Rollout: no in-place upgrade; a failed migration retries from
     the original file.
- UNVERIFIED: EXPORT/IMPORT over a wasm/OPFS filesystem abstraction
  has not been executed; the migration runner itself does not exist
  yet.

## Crash-consistency contract (for BP2 burn-in court)

The OPFS syncaccesshandle-based persistence layer must guarantee:

- C1. A committed transaction is durable across a hard kill
  (browser tab crash / `process.kill`) iff its WAL flush completed
  before the kill.
- C1a. A killed mid-transaction state recovers to the last committed
  state, with no partial rows from the killed transaction.
- C2. A failed migration (EXPORT/IMPORT flow above) leaves the
  original DB file byte-identical to pre-migration (sha256 compare).
- C3. Repeated open/close cycles over OPFS (100+ iterations) produce
  the same logical content as a single session (no drift).
C1/C1a/C3: to be verified by the BP2 burn-in court over real OPFS in
Chrome; C2 verified by sha256 before/after on the DB file.
Every item in this section is UNVERIFIED until BP2 receipts exist.

## See Also

- `docs/design/chrome-burnin-court.md`
- `docs/design/wasm-opfs-architecture.md`
- `crates/frozen-duckdb/src/duckdb/config.rs` (`Config::with`)
