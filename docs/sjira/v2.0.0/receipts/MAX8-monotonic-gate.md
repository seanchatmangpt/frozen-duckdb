# MAX8-monotonic-gate: monotonic ingress CHECK(prev_ts) gate

Date: 2026-10-19. Lane: MAX8.
Subject file: `crates/frozen-duckdb/tests/monotonic_ingress.rs` (new)

## Scope

Monotonic ingress law: out-of-order timestamps are refused at the database
boundary, not in application code. A CHECK constraint on `prev_ts` gates
every insert; a late-arriving (non-monotonic) row fails the constraint.

## Results (lane's final report, session 2026-10-19)

```
cargo test -p frozen-duckdb --test monotonic_ingress
```

- 2 passed (accepts strictly-increasing ingress; refuses out-of-order row via CHECK violation)

Non-vacuity: shim flip performed — the CHECK `prev_ts` gate was removed and
the out-of-order refusal test flipped to failure, proving the gate (not an
incidental error path) carries the law.

Overhead: Δ -5/-6 ms vs ungated baseline on the measured ingress run (gate
cost within noise; no regression).

## Standing

**PASS** — 2/2 green, gate proven non-vacuous by shim flip.

## Provenance

This receipt was transcribed post-run from the lane's final report (not
written by the lane itself). Verification command re-runnable verbatim:

```
cargo test -p frozen-duckdb --test monotonic_ingress
```

Fresh re-run output (transcription lane):

```
running 2 tests
test shim_court_gate_is_load_bearing ... ok
test monotonic_ingress_gate ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

Re-run to verify: exit 0, 2/2 (fresh output above).
