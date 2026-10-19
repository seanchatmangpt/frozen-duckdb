---
name: fd-court
description: Run and interpret the frozen-duckdb verification ladder (cargo build/test/clippy/fmt, test-validation, make verify-gates, make verify) and act on failures. Use when validating changes before landing or diagnosing a gate failure.
---

# frozen-duckdb court ladder

Choose the cheapest rung that answers the question; escalate only on failure.

1. `cargo build --workspace` — compile preflight. Build errors block every later rung; fix before anything else.
2. `cargo test -p frozen-duckdb` — crate tests (unit + integration under `crates/frozen-duckdb/tests/`: arrow, cli_surface, column_names, core_functionality, dropin_compatibility, flock, frozen_duckdb, parquet, polars, vss, tpch). Safe concurrently with lanes. Single test: `cargo test -p frozen-duckdb <name>`.
3. `cargo run -p test-validation` — semantic/domain validation suite (its own crate).
4. `cargo clippy --workspace --all-targets --all-features -- -D warnings` — lint gate; warnings are hard failures.
5. `cargo fmt --all -- --check` — formatting gate; zero-diff check, cheap to run any time.
6. `make verify-gates` — rendered gate battery (`scripts/verify-gates.sh`): runs build, clippy, fmt, test-validation, test-workspace in order, records real exit codes as `ver:Check` facts in `scripts/verify-evidence.ttl`, exits 1 with a red standing (BUILD_BROKEN/BLOCKED) on any failure. Heavy — run after fixes, not as a probe.
7. `make verify` — full admission: verify-gates + `ggen sync run` + `ggen receipt verify` (receipt chain). Release/landing rung.

Note: `make verify-gates` runs `cargo test --workspace` (all three crates), heavier than rung 2.

## On failure

- Classify first: BUILD_BROKEN (build/test compile), BLOCKED (clippy/fmt), or a real test failure. Classify, form ONE new hypothesis, rerun the failing boundary rung first (the cheapest rung that contains the failure), then expand upward.
- Never rerun an unchanged failing command without a new hypothesis.
- verify-gates exits 1 with `STANDING: BUILD_BROKEN|BLOCKED` — read which gate printed the nonzero exit in its output, fix, and rerun that rung, not the whole battery.
- verify-gates writes `scripts/verify-evidence.ttl` on every run (even red) — the evidence file is a receipt, not a pass signal; check the exit code and the printed `N/TOTAL gates green` line.
- clippy with `-D warnings` fails on warnings that plain `cargo build` passes — fix the lint, don't downgate.
- After a fix, rerun the boundary rung first, then expand upward to verify-gates.

## Final gate

A change is landed only after the rung covering its blast area is green with real output (exit 0 + printed gate summary or evidence file). No output = not landed.
