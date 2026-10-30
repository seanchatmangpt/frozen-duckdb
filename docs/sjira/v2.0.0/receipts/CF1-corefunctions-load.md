# CF1-corefunctions-load: rev_process_mining LOAD core_functions fix

Date: 2026-10-19. Lane: CF1.
Subject file: `crates/frozen-duckdb/tests/test_rev_process_mining.rs` (modified)

## Scope

The rev process-mining suite failed at the `LOAD core_functions` step (the
core_functions extension was not resolvable in the test environment). Fix:
the LOAD path in the test was corrected so the suite sets up its extension
surface properly, and the rest of the suite (including the DFG tests from
lane MAX9) stays green.

## Results (lane's final report, session 2026-10-19)

```
cargo test -p frozen-duckdb --test test_rev_process_mining
```

- 1 passed / 1 ignored (the previously-LOAD-failing path now runs; ignored
  case remains intentionally skipped)
- `cargo test -p frozen-duckdb --test dfg_cut_detection` re-run after the fix: still green (1 passed)

## Standing

**PASS** — LOAD fix landed, dfg lane unaffected.

## Provenance

This receipt was transcribed post-run from the lane's final report (not
written by the lane itself). Verification commands re-runnable verbatim:

```
cargo test -p frozen-duckdb --test test_rev_process_mining
cargo test -p frozen-duckdb --test dfg_cut_detection
```

Transcribed, re-run to verify: `cargo test -p frozen-duckdb --test test_rev_process_mining`
