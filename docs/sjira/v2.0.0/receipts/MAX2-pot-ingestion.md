# MAX2 — POT ingestion court (receipt)

Standing: **ALIVE** (lane's own final run exit 0, 16.95s; independently
re-witnessed by MAX10's 14/14 sweep)

## Lane's final witnessed numbers

- throughput: 1,000,000 rows in 16.234s = **61,599 rows/s** (floor 50,000 —
  set honestly against measured 58–80k under ~14 concurrent lane builds,
  not lowered to pass)
- rejection witness: `MonotonicRejection(frame=50 … backstep=20000us >
  5000us tolerance)` — typed rejection fires BEFORE the append buffer;
  table stays at 50 rows
- VARIANT bind: `typeof(payload)=VARIANT`, shredded read verified, no
  string column
- DFG: 1,024 distinct directly-follows pairs; support excludes
  incomparable pairs (asserted exactly: total_df = rowcount − incomparable)
- monotonic: 0 post-ingest ordering violations (SQL LAG sweep)

## Findings for the coordinator

- Third KD-06-class defect: always-false `cfg(feature = "chrono")` gate at
  `crates/frozen-duckdb/src/duckdb/types/mod.rs:17` — the `chrono` feature
  is undeclared in Cargo.toml; native timestamp binds are unreachable.
  Same class as vscalar/vtab-arrow (closed by MAX3).

## Subject

- `crates/frozen-duckdb/tests/pot_ingestion.rs` — First-Mile POT ingestion:
  `pot_edges(source_act, target_act, weight, is_cover, is_incomparable,
  event_ts)`, 1M causal edges + incomparability pairs ingested via the bulk
  appender (no intermediate JSON/CSV), monotonic ordering gate, shredded
  VARIANT payload column, post-ingest DFG aggregate.

## Executed verification

- Independent re-run (lane MAX10, single command across all six
  maximization suites, `--features serde_json,vscalar,vtab`):
  `pot_ingestion` **1 passed, 0 failed** (~15.5s wall).
- Throughput note: the suite's local throughput floor is machine-dependent
  and was shown PRE-EXISTING, not fence-caused, by lane X1r's differential
  witness (original dylib 81,731 rows/s vs fenced 82,120 rows/s).

Provenance: transcribed post-run by the coordinator from MAX10's re-run +
X1r's differential witness. Re-run verbatim:

```bash
CARGO_TARGET_DIR=target-int cargo test -p frozen-duckdb --features serde_json,vscalar,vtab --test pot_ingestion
```
