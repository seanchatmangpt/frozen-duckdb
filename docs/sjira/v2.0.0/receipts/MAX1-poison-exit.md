# MAX1-poison-exit: CacheAdmission typed verdicts + poison_state_refuse exit contract

Date: 2026-10-19. Lane: MAX1.
Subject files:
- `crates/frozen-duckdb-builder/tests/poison_state_exit.rs` (court, new)
- `crates/frozen-duckdb-builder/src/lib.rs` (CacheAdmission typed verdict path)

## Scope

CacheAdmission emits typed verdicts (not stringly stderr guessing), and the
poison-state path is held to a hard exit contract: when the builder detects a
poisoned cache state it must exit 1 with a typed stderr line. The court
`crates/frozen-duckdb-builder/tests/poison_state_exit.rs` pins 4 scenarios,
each asserting process exit code 1 plus a typed (non-empty, classified)
stderr marker — a silent zero-exit on poison is a court failure.

## Results (lane's final report, session 2026-10-19)

- 46 lib tests passed
- 1 court file passed (`poison_state_exit.rs`, 4 scenarios, all exit 1 + typed stderr)
- 12 profile tests passed (see SP1-profile-matrix.md for the profile detail)

Court command:

```
cargo test -p frozen-duckdb-builder --test poison_state_exit
```

## Standing

**PASS** — court green on the shared target at time of run.

## Provenance

This receipt was transcribed post-run from the lane's final report (not
written by the lane itself). Verification commands are re-runnable verbatim:

```
cargo test -p frozen-duckdb-builder
cargo test -p frozen-duckdb-builder --test poison_state_exit
```

Transcribed, re-run to verify: `cargo test -p frozen-duckdb-builder --test poison_state_exit`
