# USER-FINDINGS-U2 — path-dep consumer seat (lane U2)

- **Lane**: U2, repo `/Users/sac/frozen-duckdb` (read-only plus the two owned
  files: this receipt and `docs/tutorials/quickstart-native.md`). No git
  commands run. Scratch: `/tmp/u2/`.
- **Date**: 2026-10-19
- **Standing**: KD-15 **ALIVE as fixed** — re-confirmed from a fresh user
  seat, not reproduced as a defect. Quickstart tutorial updated to match.

## 1. First build, fresh out-of-tree path-dep crate (warm cache)

Scratch crate at `/tmp/u2/probe`, manifest:

```toml
[package]
name = "u2-probe"
version = "0.1.0"
edition = "2021"

[dependencies]
frozen-duckdb = { path = "/Users/sac/frozen-duckdb/crates/frozen-duckdb" }
```

Real build output (exact):

```text
$ /usr/bin/time -p cargo build
   Compiling frozen-duckdb-builder v2.0.0-dev (/Users/sac/frozen-duckdb/crates/frozen-duckdb-builder)
   Compiling frozen-duckdb-sys v2.0.0-dev (/Users/sac/frozen-duckdb/crates/frozen-duckdb-sys)
warning: frozen-duckdb-sys@2.0.0-dev: Using prebuilt DuckDB binary: /Users/sac/.frozen-duckdb/cache/v2.0.0-dev-arm64/libduckdb_arm64.dylib
   Compiling u2-probe v0.1.0 (/private/tmp/u2/probe)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3m 19s
real 199.35
user 160.92
sys 19.01
```

- **First build: 199.4 s (3m19s)**, dep-tree dominated. The engine dylib came
  from the frozen cache (builder warning line above); the entire wait is the
  Rust dep tree compiling fresh in the scratch target dir, not DuckDB.
- **Incremental rebuild: 2.03 s real** (`touch src/main.rs && cargo build`,
  `/usr/bin/time -p`: `real 2.03 / user 0.73 / sys 0.93`).

## 2. Quickstart snippet verbatim

Program source identical to the `docs/tutorials/quickstart-native.md`
listing (`Connection::open_in_memory`, `execute_batch` CREATE+INSERT,
`prepare` + `query_row` on the `?` param, print `User: {name}`). Result:

```text
$ ./target/debug/u2-probe
User: Alice
exit=0
$ cargo run -q
User: Alice
exit=0
```

Zero compile errors on the verbatim snippet.

## 3. KD-15 (rpath/dyld) — not reproduced; ADV9 fix confirmed from the user seat

```text
$ otool -l target/debug/u2-probe | grep -A2 LC_RPATH
  (no LC_RPATH entries)
$ otool -L target/debug/u2-probe | grep duckdb
	/Users/sac/.frozen-duckdb/cache/v2.0.0-dev-arm64/libduckdb_arm64.dylib (compatibility version 0.0.0, current version 0.0.0)
```

The consumer links the engine by **absolute cache path** with zero
LC_RPATH entries — the ADV9 install-ID stamp
(`docs/sjira/v2.0.0-dev/receipts/ADV9-rpath.md`) working end to end.
Bare run, no DYLD env, no install_name_tool: `exit=0`.

## 4. Three real queries + error-quality probe

Scratch crate `/tmp/u2/queries` (same path dep, standalone manifest), same
shape as `/tmp/u2/queries/src/main.rs` on disk. Program flow: open
in-memory → `CREATE TABLE t (a INTEGER, b TEXT)` → INSERT 3 rows → SELECT
ordered → `SELECT count(*)` → bad query `SELECT * FROM no_such_tbl`.

Real run output (exact):

```text
$ cargo run -q
Q1 CREATE: ok
Q2 SELECT: [(1, "x"), (2, "y"), (3, "z")]
Q3 COUNT: 3
BAD QUERY ERR: Catalog Error: Table with name no_such_tbl does not exist!
Did you mean "pg_sequence"?
LINE 1: SELECT * FROM no_such_tbl
                      ^^^^^^^^^^^
exit=0
```

**Error quality: good.** The error names the missing table, offers a fuzzy
"Did you mean" candidate, and echoes the SQL with a caret. Note: the
candidate `pg_sequence` is a DuckDB-internal catalog entry rather than the
nearest user table — upstream DuckDB behavior, not this crate.

## quickstart-native.md fix (the one owned doc edit)

The tutorial's caveat section claimed KD-15 was "known-open" and implied the
`install_name_tool -add_rpath` workaround is required. Stale: ADV9's
install-ID stamp fixed it, and this lane witnessed the fix working from a
fresh consumer seat (section 3). Replaced the "known-open" caveat with a
"fixed, verified from a fresh consumer seat" linkage note that keeps the
legacy workaround for pre-fix caches.

- Old heading: `## Caveat: first-run dyld error for external consumers (known-open)`
- New heading: `## Linkage note: first-run dyld error (KD-15) — fixed, verified from a fresh consumer seat`
- Also updated the section: real otool output from this lane embedded.

## Replay

```bash
cd /tmp/u2/probe && cargo run -q        # User: Alice, exit=0
cd /tmp/u2/queries && cargo run -q      # Q1/Q2/Q3 + BAD QUERY ERR block
otool -L /tmp/u2/probe/target/debug/u2-probe | grep duckdb   # absolute cache path
```

## Standing summary

| Claim | Standing | Evidence |
|---|---|---|
| Fresh path-dep consumer builds against warm cache | ALIVE | 199.4 s first build, real output above |
| Verbatim quickstart snippet compiles and runs | ALIVE | `User: Alice`, exit=0, zero compile errors |
| KD-15 fixed for path-dep consumers on this machine | ALIVE | absolute-path LC_LOAD_DYLIB, bare run exit=0 |
| KD-15 on Linux / caches persisted pre-fix | UNVERIFIED | macOS-only stamp; self-heal claimed by ADV9, not re-witnessed here |
| Query API + error-message quality | ALIVE | Q1–Q3 pass; Catalog Error names the miss |
