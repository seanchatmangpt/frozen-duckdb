# U12 — Panic Hunter Receipt (frozen-duckdb v2.0.0)

Subject: `/Users/sac/frozen-duckdb` @ master `0a205b7` (dirty tree, lane U12 files only).
Battery: `crates/frozen-duckdb/tests/panic_hunter.rs` — 24 cells over the PUBLIC API.

## Run

- BEFORE: `cargo test -p frozen-duckdb --test panic_hunter`
  → `17 passed; 7 failed` — 7 PANIC cells (BLOCKERs).
- AFTER (fixes applied): same command → `24 passed; 0 failed; finished in 0.10s`.
- Regression: `cargo test -p frozen-duckdb --lib` → `140 passed; 0 failed`.
- Gates: `cargo clippy -p frozen-duckdb --all-targets -- -D warnings` → clean;
  `cargo fmt -p frozen-duckdb --check` → clean.

## Panic table (BLOCKERs found → all fixed)

| # | Cell | Public entry | Panic site (before) | Trigger | Fix |
|---|------|--------------|---------------------|---------|-----|
| P1 | `nul_in_execute_batch` | `Connection::execute_batch` | `duckdb/inner_connection.rs:89` `CString::new(sql).unwrap()` | `"SELECT 1;\0SELECT 2;"` | `.unwrap()` → `?` (typed `Error::NulError`) |
| P2 | `nul_in_prepare` / `trailing_nul_sql` | `Connection::prepare` | `inner_connection.rs:100` `CString::new(sql).unwrap()` | `"SELECT \0"` / trailing NUL | `.unwrap()` → `?` |
| P3 | `nul_in_appender_table` / `nul_in_appender_schema` | `Connection::appender` / `appender_to_db` | `inner_connection.rs:180-181` `CString::new(table/schema).unwrap()` | table/schema containing NUL | `.unwrap()` → `?` |
| P4 | `nul_in_config_key` / `nul_in_config_value` | `Config::with` (reached via `Connection::open_in_memory_with_flags`) | `duckdb/config.rs:143-144` `CString::new(key/value).unwrap()` | NUL in config key or value | `.unwrap()` → `?` |

Class: one root cause, five `unwrap()` sites on external input (violates the
crate's own no-unwrap-on-external-input law). `From<NulError> for Error`
already existed, so the conversion is a pure `?` substitution at each site —
no new error variant, no API change. Callers now receive
`Err(Error::NulError(NulError{..}))` instead of a process panic.

## Non-panicking cells (clean — typed Err or correct result)

`emoji_identifiers`, `rtl_identifiers` (RTL table round-trips, count=1),
`deeply_nested_parens` (2000 levels — parsed fine), `empty_sql`,
`empty_prepare`, `empty_param_bind`, `i64_boundaries` (MIN..MAX inserted and
counted), `u64_boundaries` (`u64::MAX` bind accepted), `f64_nan_inf_binds`
(NaN/±Inf stored, count=6), `huge_blob` (8 MiB, `octet_length` verified),
`ten_mb_string_roundtrip` (10 MiB VARCHAR byte-exact),
`varchar_embedded_nul_bind` (embedded NUL survives round-trip byte-exact),
`negative_interval`, `absurd_decimal_precision` (DECIMAL(77,40) → typed Err),
`decimal_38_boundary` (DECIMAL(38,10) OK).

## MINOR notes (non-blocking)

- `u64::MAX` binds succeed into BIGINT via the FFI layer; behavior not
  asserted as a value round-trip (lossy by DuckDB semantics) — registry note
  only, no defect filed.
- `deeply_nested_parens` at 2000 levels parses without error in this build;
  a 100k-level parser-recursion probe was out of lane scope.

## Court law compliance

- Real runs only; no git commands executed (lane law).
- The AFTER flip (7 fail → 0 fail) is the non-vacuity proof: the battery
  demonstrably fails on the pre-fix subject.
- No projections edited (`scripts/`, `Makefile`, `gates/` untouched).
