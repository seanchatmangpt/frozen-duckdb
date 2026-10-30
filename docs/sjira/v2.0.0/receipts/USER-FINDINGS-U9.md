# USER-FINDINGS-U9 — Error-message reader (v2.0.0)

Lane U9. Subject: `crates/frozen-duckdb` @ master `0a205b7` (dirty tree; lane edits limited to
`src/duckdb/error.rs`). Triggers run from a path-dep consumer at `/tmp/u9/trigger` against the
real engine (`libduckdb_arm64.dylib` from `~/.frozen-duckdb/cache/v2.0.0-arm64`).

- Before capture: full harness run, exit 0, output quoted in the grade table.
- After capture: same harness rebuilt against the fixed crate, exit 0, quoted per fix.
- Regression: `cargo test -p frozen-duckdb --test variant_tests` → `5 passed; 0 failed`
  (the one test asserting on `Invalid column type` uses `contains`, so appended hints are safe).
- `cargo clippy -p frozen-duckdb -- -D warnings` → clean.
- `cargo fmt -p frozen-duckdb -- --check` → clean.

## Grade table (all 8 triggers, BEFORE state)

| # | Trigger | Grade | Exact message user sees (before) |
|---|---|---|---|
| 1 | Bad SQL syntax | **A** | `Parser Error: syntax error at or near "SELEC"` + `LINE 1: SELEC 1` + caret |
| 2 | Nonexistent table | **A** | `Catalog Error: Table with name nope_table does not exist!` + `Did you mean "pg_tables"?` + caret |
| 3 | Locked db (2nd connection) | **A** | `IO Error: Could not set lock on file "…/l.db": Conflicting lock is held in …/u9-trigger (PID 83741) by user sac. See also https://duckdb.org/docs/current/connect/concurrency` |
| 4 | OOM (`SET memory_limit='50MB'` + big materialization) | **A** | `Out of Memory Error: failed to allocate data of size 16.0 MiB (32.0 MiB/47.6 MiB used)` + 3-line `Possible solutions` list + tuning docs link |
| 6 | Type mismatch in bind | **A** | `Binder Error: Could not choose a best candidate function for "+(STRING_LITERAL, INTEGER_LITERAL)". In order to select one, please add explicit type casts.` + candidate function list |
| 8 | Extension not loaded | **A** | `IO Error: Extension "…/definitely_not_an_ext.duckdb_extension" not found.` + `Candidate extensions: "inet", "fts", "excel", "delta", "sqlite_scanner"` |
| 5 | Missing column | **C** | `Invalid column name: no_such_col` |
| 7 | Invalid cast (wrong Rust type for result) | **C** | `Invalid column type Int at index: 0, name: 42` |
| 9 | Invalid column index (positional `row.get(9)` past end) | **C** | `Invalid column index: 5` (string read from the crate's `Display` arm; this exact trigger was NOT in the harness — see `[SKIP]` below) |

Engine messages (1, 2, 3, 4, 6, 8) come from the precompiled libduckdb binary — already grade A
and not editable as crate message strings; left untouched by design (message-strings-only lane).

## The 3 fixed (worst crate-side messages → A)

All three are `Display` edits in
`/Users/sac/frozen-duckdb/crates/frozen-duckdb/src/duckdb/error.rs` — message strings only,
no variant, payload, or behavior changes.

### Fix 1 — `InvalidColumnType` (trigger 7, C → A)

- Before: `Invalid column type Int at index: 0, name: 42`
- After: ``Invalid column type Int at index: 0, name: 42 (requested a different Rust type; fetch as Int or cast in SQL, e.g. `SELECT 42::VARCHAR`)``

### Fix 2 — `InvalidColumnName` (trigger 5, B → A)

- Before: `Invalid column name: no_such_col`
- After: ``Invalid column name: no_such_col (column lookup is by exact, case-sensitive name — list valid names with `Statement::column_names`)``

### Fix 3 — `InvalidColumnIndex` (trigger 9, C → A)

- Before: `Invalid column index: {i}` (from `Display` arm)
- After: ``Invalid column index: {i} (the query returned fewer columns; indices are 0-based — check `Statement::column_count`)``

## [SKIP] register (loud, not silent)

- `[SKIP] locked-db-from-same-process`: two in-process `Connection::open` calls to the same file
  do NOT conflict (DuckDB dedups in-process instances), so the lock trigger requires a second
  OS process; the harness spawns a child `hold` mode to capture trigger 3's message.
- `[SKIP] invalid-column-index trigger`: the harness covers name-based get (trigger 5) but not
  positional out-of-range get, so Fix 3's before/after pair is reconstructed from the Display
  arm + post-fix clippy/tests, not a captured engine rerun. The other two fixes have real
  before/after reruns quoted above.
- `[SKIP] OOM sizing`: the first OOM attempt (range × range cross product) exceeded the
  120 s harness timeout instead of erroring; the trigger was re-shaped as
  `length(list(i))` materialization under `memory_limit='50MB'`, which errors deterministically.

## Regression proof (after fixes)

- `cargo test -p frozen-duckdb --test variant_tests`: `5 passed; 0 failed; 0 ignored`
- `cargo clippy -p frozen-duckdb -- -D warnings`: clean
- `cargo fmt -p frozen-duckdb -- --check`: clean

Harness kept at `/tmp/u9/trigger` for replay by the coordinator.
