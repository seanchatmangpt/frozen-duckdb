# SP1-profile-matrix: typed build profile matrix in frozen-duckdb-builder

Date: 2026-10-19. Lane: SP1.
Subject files:
- `crates/frozen-duckdb-builder/src/profile.rs` (new)
- `crates/frozen-duckdb-builder/src/lib.rs` (`pub mod profile;` wired)
- inline `#[cfg(test)]` suite in `profile.rs` (12 tests)

## Scope

Build profiles (feature/flag matrix for the builder's acquisition paths) are
carried as typed data with a `ProfileError` instead of string parsing at
call sites. The module is wired into the crate root as `pub mod profile;`
so downstream lanes (MAX1 poison court among them) consume the same typed
surface.

## Results (lane's final report, session 2026-10-19)

```
cargo test -p frozen-duckdb-builder --lib profile
```

- 12 profile tests passed
- full builder suite verified on the shared target: 59/59 passing

## Standing

**PASS** — 12/12 profile tests, 59/59 builder suite on shared target.

## Provenance

This receipt was transcribed post-run from the lane's final report (not
written by the lane itself). Verification commands re-runnable verbatim:

```
cargo test -p frozen-duckdb-builder --lib profile
cargo test -p frozen-duckdb-builder
```

Transcribed, re-run to verify: `cargo test -p frozen-duckdb-builder --lib profile`
