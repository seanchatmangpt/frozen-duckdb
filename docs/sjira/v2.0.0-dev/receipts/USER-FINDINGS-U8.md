# USER-FINDINGS-U8 — bulk-load power user: appender API UX at 10M rows

Lane U8. Repo: /Users/sac/frozen-duckdb. Date: 2026-10-19.
Subject: crates/frozen-duckdb @ master (post 0a205b7 tree) + one papercut fix.
Probe source: /tmp/u8/probe (path-dep on crates/frozen-duckdb, real engine,
prebuilt libduckdb_arm64.dylib from ~/.frozen-duckdb/cache).

## 1. Throughput baseline (vs X8's 12.16M/s)

10M rows (BIGINT, DOUBLE, VARCHAR) via `append_row(params![...])` per row:

- Run A (pre-fix build): **11.16M rows/s**, elapsed 0.90 s
- Run B (post-fix build): **10.19M rows/s**, elapsed 0.98 s

Confirms X8's 12.16M/s within run-to-run noise (same order; this probe
carries a VARCHAR column and per-row `params!` boxing, X8's packed path
was tighter). 10M rows land in ~1 s. No fix-induced regression: both runs
within noise band.

## 2. Error-path UX table (what the user actually SEES)

| Scenario | Before fix | After fix |
|---|---|---|
| Wrong column count (1 val into 2-col table) | `DuckDBFailure(..., "Call to EndRow before all columns have been appended to!")` | unchanged (end_row path; message OK, lacks column names/counts) |
| Type mismatch (`"abc"` into INTEGER) | `Err(AppendError)` — unit variant, Display "Append error", **zero bits** | `DuckDBFailure(..., Some("Could not convert string 'abc' to INT32"))` |
| NULL into NOT NULL column | append_row → `Ok(())`; violation surfaces **only at flush**: `"Failed to append: NOT NULL constraint failed: t4.a"` | unchanged |
| FK violation | surfaces at flush with full engine message | unchanged |

**Structural finding**: per-column append failures returned the message-less
`Error::AppendError` even though the engine held a precise message
(`duckdb_appender_error`); constraint violations (NOT NULL, FK) are buffered
and only surface at `flush()` — a 10M-row load fails at second 900, not row 1
of the bad pattern... actually it fails at the first flush (default chunk
flush), so failure latency is bounded by chunk size, not the whole load.

## 3. Partial-failure semantics — appender is POISONED, silently

`result_from_duckdb_appender` (crates/frozen-duckdb/src/duckdb/error.rs:254-272)
destroys the appender handle in its error path (line ~267:
`ffi::duckdb_appender_destroy(appender)`). So **any** failed `append_row`
end_row / `flush()` nulls the handle inside the still-alive Rust `Appender`.

Observed on real runs:
- flush#1 fails (FK violation) with a good message
- next `append_row` on the same object → Err with message `"appender is null"`
- `flush()` again → `"appender is null"`

Not UB (destroy nulls the pointer, Drop guards `is_null`), but the appender is
a poisoned object wearing a usable API: no `is_poisoned()` flag, no typed
error, and the error surfaces as "appender is null" — which reads like a
library bug, not a lifecycle state. **Follow-up needed**: either make the
error path stop destroying (let Drop own the lifetime) or poison explicitly
with a typed `Error::AppenderClosed`. NOT fixed in this lane (one-fix budget).

## 4. Memory: 10M-row batch RSS

`/usr/bin/time -l` maximum resident set size: **383 MB** (~38 B/row for
BIGINT+DOUBLE+9-byte VARCHAR), peak footprint 358 MB. Bounded — the appender
flushes in chunks; RSS does not scale with total rows.

## Fix receipt (one papercut)

File: crates/frozen-duckdb/src/duckdb/appender/mod.rs
- `bind_parameter`: failed per-column append now returns
  `Error::DuckDBFailure(ffi::Error::new(ffi::DuckDBError), appender_error_message(...))`
  instead of bare `Error::AppendError`.
- New `#[cold] fn appender_error_message` reads `duckdb_appender_error`
  WITHOUT destroying the handle (the shared helper destroys on read).
- New regression test `test_append_type_mismatch_has_real_message`.

Verification (real commands this session):
- `cargo test -p frozen-duckdb --lib duckdb::appender -- --nocapture`
  → **7 passed; 0 failed** (witness: "SHIM-COURT lane U8: per-row message =
  \"Could not convert string 'abc' to INT32\"")
- `cargo clippy -p frozen-duckdb --all-features -- -D warnings` → clean
- `rustfmt --check` on appender/mod.rs → clean (pre-existing fmt drift in
  types/serde_json.rs is NOT this lane's file, untouched)

## Follow-ups registered

1. Poisoned-appender lifecycle (error.rs:267 destroy-on-error) — typed
   `AppenderClosed` or stop destroying; error message "appender is null"
   currently misleads.
2. Wrong-column-count message names no columns; adding expected/actual
   counts needs appender-side column tracking (not cheap from the C API).
3. NOT NULL/FK surface only at flush — documented above; consider
   end_row-time constraint probes for small-batch users.
