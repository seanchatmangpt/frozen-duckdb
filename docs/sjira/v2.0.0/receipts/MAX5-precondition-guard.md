# MAX5 — STALE_PRECONDITION_REFUSED two-phase guard (receipt)

Standing: **ALIVE** (court 5/5 witnessed twice; clippy exit 0)

## Subject

- `crates/frozen-duckdb/src/duckdb/precondition.rs` — `PreconditionGuard`:
  observes a table state hash at analysis time; `execute_if_unchanged`
  re-hashes at execution and refuses with the typed
  `STALE_PRECONDITION_REFUSED: expected <hash>, got <hash>` on mutation,
  executes + returns the new hash when clean.
- `crates/frozen-duckdb/tests/precondition_guard.rs` — the court.

## Executed verification

1. Lane's own run (per its final report, before a transient API cut):
   all 5 court tests pass; it was at the clippy step.
2. Independent re-run (lane MAX10, single command across all six
   maximization suites): `precondition_guard` **5 passed, 0 failed**.
3. Clippy (coordinator, `CARGO_TARGET_DIR=target-laneMAX5`):
   `cargo clippy -p frozen-duckdb --tests -- -D warnings` → exit 0.

## Court shape

- analyze → out-of-band mutate → guarded execute → **refused** (witnessed
  typed `STALE_PRECONDITION_REFUSED`)
- analyze → guarded execute clean → **passes**, new hash returned
- two-phase analog: mutation SQL carries the precondition hash; mismatch
  aborts before execution.

Provenance: receipt transcribed post-run by the coordinator from the lane's
final report + MAX10's independent 5/5 re-run + the coordinator's clippy run.
Re-run verbatim:

```bash
CARGO_TARGET_DIR=target-laneMAX5 cargo test -p frozen-duckdb --test precondition_guard
CARGO_TARGET_DIR=target-laneMAX5 cargo clippy -p frozen-duckdb --tests -- -D warnings
```
