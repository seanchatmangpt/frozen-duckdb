# MAX9-dfg-cut: DFG cut detection over 1M edges

Date: 2026-10-19. Lane: MAX9.
Subject file: `crates/frozen-duckdb/tests/dfg_cut_detection.rs` (new)

## Scope

Directly-follows-graph (DFG) cut detection executed in SQL over the process
mining edge table. The test compares the SQL cut result against an exact
oracle computed independently in the test — oracle equality, not a
self-constructed fixture.

## Results (lane's final report, session 2026-10-19)

```
cargo test -p frozen-duckdb --test dfg_cut_detection
```

- 1 passed
- scale: 1,000,000 edges
- latency: 4.968 ms (median of 5 runs)
- oracle: exact match

## Standing

**PASS** — oracle-exact on 1M edges at ~5 ms.

## Provenance

This receipt was transcribed post-run from the lane's final report (not
written by the lane itself). Verification command re-runnable verbatim:

```
cargo test -p frozen-duckdb --test dfg_cut_detection
```

Fresh re-run output (transcription lane):

```
running 1 test
test dfg_cut_detection_court ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s
```

Re-run to verify: exit 0, 1/1 (fresh output above).
