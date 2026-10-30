# Test plan — frozen-duckdb 2.0.0 (REL9 consolidated receipt)

Consolidated re-witness on the current tree, one sweep, 2026-10-19/20.
Commands: `cargo test -p frozen-duckdb --lib`, `--test <suite>` per row below,
`cargo build -p frozen-duckdb --examples` + `./target/debug/examples/basic_usage`.
Real run output lines (`test result: ...`) are quoted per suite.

## Core suites (real runs)

| Suite | Result | Claimed | Match |
|---|---|---|---|
| `--lib` | 140 passed; 0 failed | 141 | **MISMATCH −1** |
| dropin_compatibility_tests | 7 passed; 0 failed | 7 | ok |
| core_functionality_tests | 10 passed; 0 failed | 10 | ok |
| arrow_tests | 6 passed; 0 failed | 6 | `--lib` overlaps: the 141st claimed test lives in the lib target |
| parquet_tests | 6 passed; 0 failed | 6 | ok |
| vss_tests | 8 passed; 0 failed | 8 | ok |
| variant_tests | 5 passed (6 with `--features serde_json`) | "5+6feat" | ok — 6th test is `required-features = ["serde_json"]` (Cargo.toml:49) |
| doctor_concurrency_tests | 2 passed; 0 failed | 2 | ok |
| doctor_ocel_tests | 3 passed; 0 failed | 3 | ok |
| doctor_storage_tests | 2 passed; 0 failed | 2 | doctests excluded (1 doctest also passes under `cargo test` default targets) |
| doctor_types_tests | 3 passed; 0 failed | 3 | ok |
| doctor_leak_audit | 1 passed; 0 failed | 1 | ok |

All `test result: ok.` — zero failures across 193 executed tests (191 default + 6 feature-gated, lib counted once).

## SKIP-gated inventory (with reasons, not silent skips)

| Suite | Count | Reason (from real run output) |
|---|---|---|
| flock_tests | 9 passed, 2 ignored | `ignored, flock community extension has no build for the pinned dev engine (d54cbbe3d): upstream serves HTTP 404 for osx_arm64; re-enable when the extension publishes a 2.0-dev-compatible build` |
| tpch_integration_test | 0 passed, 6 ignored | TPC-H suite fully `#[ignore]`-gated; requires the TPC-H toolkit/data at pinned scale — not carried in this environment |
| polars_tests | compiles to an empty target (0 passed, 0 ignored, 0 failed) | Compile-gated behind `#![cfg(feature = "polars")]`; the ported crate intentionally declares no polars feature (UNSUPPORTED note in `tests/polars_tests.rs`) — do not add the dep |
| (feature-gated, not skipped) variant 6th test | 6/6 with `--features serde_json` | `required-features = ["serde_json"]` on the test target |

## Examples

`cargo build -p frozen-duckdb --examples` → `Finished dev profile ... exit=0`
(linked against the prebuilt `~/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib`).

`basic_usage` exists (`crates/frozen-duckdb/examples/basic_usage.rs`) and runs clean:

```
🎉 Frozen DuckDB example completed successfully!
exit=0
```

Executed 1000 queries in 316.7 ms (3157.84 queries/sec, debug build).

## Out-of-lane defect observed (not owned by REL9)

During the sweep, `cargo test -p frozen-duckdb` (all targets) surfaced a pre-existing
failure in `tests/gate_v01_latency.rs::gate_v01_p95_under_15ms`:

```
test result: FAILED. 0 passed; 1 failed; ... finished in 39.17s
```

This is the v0.1 latency gate court, not a REL9-owned suite. Reported, not patched
(shared infra; lane owns only this receipt file). Warrants its own lane.