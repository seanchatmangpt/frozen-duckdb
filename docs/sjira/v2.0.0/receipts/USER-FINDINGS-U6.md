# USER-FINDINGS-U6 — 1.5.5-user walkthrough of MIGRATION_GUIDE.md (lane U6)

Date: 2026-10-19. Subject: /Users/sac/frozen-duckdb @ master (2.0.0).
Method: fresh scratch crate /tmp/u6/mig (path dep on
`/Users/sac/frozen-duckdb/crates/frozen-duckdb`), every guide snippet executed
verbatim against the 2.0.0 engine, plus behavioral probes.

## Snippet execution results (all green)

- Step 2/Example 1: `Connection::open_in_memory` + `execute_batch` CREATE/INSERT
  — OK, no import change needed (`use frozen_duckdb::{Connection, Result}`).
- Example 3: `prepare` + `params!["electronics"]` + `query_map` returning
  `(String, f64)` — OK; real output `row: ("electronics", 9.99)`.
- Example 2 (file db): `Connection::open("/tmp/u6/data.db")` — OK; file banner
  reads `v2.0.0`.
- `SELECT version()` -> `v2.0.0` (NOT 1.5.5, NOT `1.10505`).

## Findings

| id | severity | finding | evidence |
|---|---|---|---|
| U6-1 | BLOCKER | Guide Step 1 says `cargo add frozen-duckdb`; crates.io `frozen-duckdb` is an unrelated 0.1.0 and 2.0.0 is unpublished — verbatim command installs the wrong crate. | `curl crates.io/api/v1/crates/frozen-duckdb` -> max 0.1.0; guide's own prerelease note |
| U6-2 | MAJOR | Troubleshooting claims `frozen-duckdb = "2.0.0"` appears in Cargo.toml; a 1.5.5 user pinning via registry line cannot have that (unpublished); engine version string `v2.0.0` never appears anywhere in the guide. | scratch run output |
| U6-3 | MAJOR | BLOB round-trip behavior change undocumented. 1.x returned text rendering `x0102`; 2.0.0 returns raw bytes. Witnessed in-repo: `crates/frozen-duckdb/src/duckdb/types/mod.rs:330` (`assert_eq!(Value::Blob(vec![1,2]), ...)`) and reproduced in scratch: `blob bytes: [1, 2]`. | run output + source witness |
| U6-4 | MAJOR | Extensions: `INSTALL flock` -> `HTTP Error: Failed to download extension "flock" ... (HTTP 404)`; `INSTALL tpch` OK but `LOAD tpch` fails `signature is either missing or invalid (allow_unsigned_extensions)`; `tpch(1)` helper table function gone (ADV4). Guide's "All features ... Parquet, JSON, Arrow integration" never warns. | scratch run output; receipts/ADV4-extensions.md |
| U6-5 | MINOR | "All features included by default" false: `serde_json` is an opt-in feature (Cargo.toml [features], PB10-F2), gating serde_json::Value FromSql/ToSql incl. VARIANT round-trip. | crates/frozen-duckdb/Cargo.toml:74-82 |
| U6-6 | MINOR | Feature Flags "Before" comment conflates crate version with engine version: `1.10505.0` is the duckdb-rs crate version, not "bundling DuckDB 1.5.5". | guide line 185 pre-edit |
| U7-7 | MINOR | Rollback plan ignores on-disk storage: db files written by 2.0.0 carry the `v2.0.0` banner (strings(1) on /tmp/u6/data.db) and are not guaranteed to reopen on 1.x. | scratch probe |
| U6-8 | MINOR (positive) | VARIANT DDL `CREATE TABLE t(v VARIANT)` is now ACCEPTED (was 1.x-refused era); raw VARIANT projects `Struct(metadata BLOB, value BLOB)` — matches variant_tests.rs:114. | scratch run + variant_tests.rs |

## Guide fixes landed (MIGRATION_GUIDE.md)

1. Step 1: loud warning that `cargo add frozen-duckdb` resolves to the wrong
   0.1.0 crate; path/git pin instructions instead (U6-1, U6-2).
2. Troubleshooting version string: pin form + engine `v2.0.0` note (U6-2).
3. Feature Flags: `serde_json` opt-in correction + new "Behavior changes"
   subsection: blob round-trip, extension 404s/unsigned tpch, VARIANT DDL
   (U6-3, U6-4, U6-5, U6-8).
4. Before-line comment: crate-vs-engine version distinction (U6-6).
5. Rollback plan: storage banner caveat, export before rollback (U7-7).

## Standing

- Guide snippet execution: PASS (8/8 snippets/probes executed, 0 stubs).
- Guide correctness at receipt time: fixed the 5 top breaks above; remaining
  marketing claims ("99% faster", "Production ready") are outside lane scope.
- No git commands run. Lane files touched: MIGRATION_GUIDE.md (edits above),
  this receipt, /tmp/u6 scratch.
