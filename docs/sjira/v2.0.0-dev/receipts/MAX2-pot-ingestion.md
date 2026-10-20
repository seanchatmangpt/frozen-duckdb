# MAX2 — POT ingestion court (receipt)

Standing: **ALIVE** (independently re-witnessed; lane's own receipt file
pending — this file records the witnessed state)

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
