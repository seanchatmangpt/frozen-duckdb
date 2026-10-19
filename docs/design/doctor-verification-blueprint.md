# Doctor Verification Blueprint

- **Lane**: D1 (write; this file only; NO git)
- **Status**: source-of-truth capture of the operator's Verification Blueprint
  & Chicago-Style Test Battery: 25 tests, phases 0-9, target command
  `just doctor`.
- **Live mapping**: per-test implementation status is tracked by D11 in
  `docs/design/doctor-implementation-map.md` — authoritative for what runs
  green today vs FR-01-blocked vs implemented-elsewhere. This file defines the
  battery; that file defines its landing state.
- **Standing**: PARTIAL_ALIVE — the battery's executable substrate (phase
  scripts, aggregate runner, mapped test files) exists on disk; blocked and
  pending rows are loud, not silent.

## Boundary law

`just doctor` (via `scripts/doctor.sh`, fail-closed aggregate) is the
battery's single entry point. Contract:

- **exit 0** — clean: every phase exited 0; runner prints `ALL 10 PHASES
  VERIFIED CLEAN`.
- **exit 1** — breached: any phase FAIL, loud SKIP, or NOT-IMPLEMENTED forces
  aggregate exit 1. A skip is loud (`[SKIP] <reason>`; phase-level exit 2) but
  is never a pass — the aggregate collapses it to a breach.
- **No vacuous skips.** Missing toolchain → `[SKIP] cargo not on PATH` (loud).
  Missing test file → `PHASE NOT-IMPLEMENTED` (phase exit 3, loud). Both
  breach the boundary at the aggregate. A court that cannot flip is FAIL, not
  PASS.
- Phase exit taxonomy: `0` pass · `2` loud skip · `3` NOT-IMPLEMENTED · any
  other nonzero = fail. Every nonzero collapses to aggregate exit 1.

## Phase pipeline

Target command `just doctor` → `scripts/doctor.sh`:

```text
just doctor ─ scripts/doctor.sh ── fail-closed aggregate, exit 0 clean / 1 breached
 ├─ 00_preflight      TEST-001..002     header attestation, strip contract
 ├─ 01_storage_wal    TEST-003..006     page format, alignment, WAL drain
 ├─ 02_types_variant  TEST-007..009     value/type round-trip, blob literal
 ├─ 03_host_vfs       TEST-010..012     host-VFS transfer (FR-01-blocked)
 ├─ 04_memory_guard   TEST-013..014     OOM typed abort, leak audit
 ├─ 05_chrome         TEST-015..016     Chrome burn-in (FR-01-blocked)
 ├─ 06_ocel           TEST-017..019     compression, rehydration, digest
 ├─ 07_performance    TEST-020..022     throughput, sustained RSS, latency
 ├─ 08_concurrency    TEST-023          flock file-lock contract
 └─ 09_court_ladder   TEST-005,024,025  crash chaos, court battery, boundary
```

Nonzero at any phase stops nothing but breaches everything: the aggregate
continues through all 10 phases collecting verdicts, then exits 1 on any
breach. Phases 03/05 exit 3 NOT-IMPLEMENTED (loud divergence, never a fake
pass).

## Status vocabulary (per-test `Status` field)

- `runnable-native-today` — mapped artifact exists and runs green in this repo
  today under its phase script.
- `FR-01-blocked` — needs the `duckdb_wasm` artifact + version gate (FR-01,
  `docs/design/requirements-traceability.md`); phase scripts 03/05 exit 3
  NOT-IMPLEMENTED until FR-01 lands.
- `implemented-elsewhere` — the law is already carried by a live artifact
  outside the phase's own test file (named per row).
- `pending` — test file or assert not yet authored; surfaced loudly (phase
  exit 3, or the gate runs without the row).

## Phase 0 — Preflight <a id="phase-0"></a>

Script: `scripts/doctor/00_preflight.sh`. Subsystem: build-surface integrity.
Checks are inline and real; `set -euo pipefail`; exit = number of breaches.

**TEST-001 — header hash attestation** — runnable-native-today

- Subsystem: build surface (vendored headers).
- Objective: attest the vendored DuckDB headers have not silently changed.
- Mechanism: `shasum -a 256` over
  `crates/frozen-duckdb-builder/vendored-headers/duckdb.h` (inline in the
  phase script; also `scripts/doctor/checks/attest_pin.sh`).
- State assertions: 64-hex digest printed and well-formed; missing header or
  malformed digest exits nonzero.

**TEST-002 — strip contract** — runnable-native-today

- Subsystem: build surface (release binary).
- Objective: a release binary strips without dropping required symbols.
- Mechanism: structural contract over the build-script surface —
  `grep -qE 'strip|postprocess'` on `scripts/build_frozen_duckdb.sh`.
- State assertions: contract marker present; missing script or absent marker
  exits nonzero.

## Phase 1 — Storage/WAL <a id="phase-1"></a>

Script: `scripts/doctor/01_storage_wal.sh`. Subsystem: on-disk storage
contract. The phase probes engine availability with a real `cargo build -p
frozen-duckdb` before running tests (PENDING-REBUILD is loud, exit 3).

**TEST-003 — checkpoint round-trip** — runnable-native-today

- Subsystem: storage.
- Objective: the DB file on disk is queryable after CHECKPOINT (close/reopen
  round-trip).
- Mechanism: `tests/doctor_storage_tests.rs` via
  `cargo test -p frozen-duckdb --test doctor_storage_tests`.
- State assertions: reopen the checkpointed file, query rows back, typed
  compare against what was written.

**TEST-004 — 4 KiB page alignment** — runnable-native-today

- Subsystem: storage (page format).
- Objective: after CHECKPOINT the DB file size is a multiple of 4096
  (DuckDB's page size).
- Mechanism: `tests/doctor_storage_tests.rs` — checkpoint, then assert
  `size % 4096 == 0` over the real on-disk file.
- State assertions: file size multiple of 4096 post-checkpoint.

**TEST-006 — WAL drain** — runnable-native-today

- Subsystem: storage (WAL lifecycle).
- Objective: the WAL is bounded — grows on uncheckpointed writes, drains on
  checkpoint.
- Mechanism: `tests/doctor_storage_tests.rs` (TEST-006): bulk insert, stat
  the `.wal`.
- State assertions: `.wal` non-empty after bulk insert without checkpoint;
  deleted-or-zero-length after CHECKPOINT.

## Phase 2 — Types/variant <a id="phase-2"></a>

Script: `scripts/doctor/02_types_variant.sh`. Subsystem: value/type fidelity.

**TEST-007 — value/type round-trip** — runnable-native-today

- Subsystem: types.
- Objective: values round-trip losslessly (NULL, empty containers, boundary
  numerics).
- Mechanism: `tests/variant_tests.rs` via
  `cargo test -p frozen-duckdb --test variant_tests` (runner shared with
  TEST-008/009).
- State assertions: round-tripped value equals original — typed compare, not
  a summary.

**TEST-008 — serde_json interop** — runnable-native-today

- Subsystem: types (serde_json).
- Objective: JSON in → typed value → JSON out preserves value and type tag.
- Mechanism: `tests/variant_tests.rs` (serde_json interop tests).
- State assertions: round-tripped JSON is value- and type-faithful.

**TEST-009 — blob literal** — implemented-elsewhere

- Subsystem: types (blob literal parse/emit).
- Objective: blob literals round-trip byte-exactly (incl. empty blob, binary
  payloads).
- Mechanism: the fixed blob-literal handling in
  `src/duckdb/types/mod.rs` (`Blob` variant; `test_blob` /
  `test_empty_blob` at types/mod.rs:221/233) plus the variant suite.
- State assertions: blob in → literal → parse back byte-exact; empty blob
  preserved.
- Note: the fix already landed in `types/mod.rs`; the law is carried by
  `tests/variant_tests.rs`.

## Phase 3 — Host-VFS <a id="phase-3"></a>

Script: `scripts/doctor/03_host_vfs.sh`. Subsystem: host-VFS transfer across
the wasm boundary. **FR-01-blocked**: the phase exits 3 NOT-IMPLEMENTED until
FR-01 (the `duckdb_wasm` artifact + version gate) lands. Contract reference:
`docs/design/host-range-vfs-contract.md`.

| id | objective | state assertions (definitional, pre-implementation) | status |
|---|---|---|---|
| TEST-010 | byte-exact read-through of a seeded host file | transferred bytes equal host bytes (digest compare) | FR-01-blocked |
| TEST-011 | write-back replays to host bytes | journal replay reproduces host-side bytes exactly | FR-01-blocked |
| TEST-012 | large-blob transfer under bounded memory | transfer completes; peak memory under the declared bound | FR-01-blocked |

## Phase 4 — Memory guard <a id="phase-4"></a>

Script: `scripts/doctor/04_memory_guard.sh`. Subsystem: memory behavior under
pressure.

**TEST-013 — OOM graceful abort** — implemented-elsewhere

- Subsystem: memory guard.
- Objective: at a 256 MB cap the engine fails typed, not by killing the
  process; state survives.
- Mechanism: `tests/oom_graceful_abort.rs`
  (`oom_at_256mb_is_typed_and_state_survives`).
- State assertions: typed error at the 256 MB cap; connection state queryable
  after the typed failure. Companion `same_probe_completes_at_4gb_cap` (same
  file) proves the failure is cap-specific, not a query regression.
- Note: already carried by `tests/oom_graceful_abort.rs`; runs in Phase 4
  today.

**TEST-014 — leak audit** — runnable-native-today

- Subsystem: memory guard.
- Objective: repeated analytical work does not leak resident memory.
- Mechanism: `tests/doctor_leak_audit.rs` — 20k iterations over a temp view
  on an in-memory DB; RSS delta within documented tolerance (64 KiB real-leak
  floor + 2 MiB macOS `ps` sampling headroom).
- State assertions: RSS delta within tolerance after 20k iterations;
  PENDING-REBUILD reports PENDING, never a faked pass.

## Phase 5 — Chrome burn-in <a id="phase-5"></a>

Script: `scripts/doctor/05_chrome.sh`. Subsystem: real Chrome via CDP over
the wasm/OPFS substrate. **FR-01-blocked**: the phase exits 3
NOT-IMPLEMENTED until FR-01 lands. Pattern reference:
`docs/design/chrome-burnin-court.md`.

| id | objective | state assertions (definitional, pre-implementation) | status |
|---|---|---|---|
| TEST-015 | persistence survives a real forced crash (`Page.crash` via CDP) | post-relaunch checksums equal pre-crash digests after WAL replay | FR-01-blocked |
| TEST-016 | bounded-duration burn-in soak with bounded memory | RSS/JS-heap samples stay under budget (bounded polls, no sleeps) | FR-01-blocked |

## Phase 6 — OCEL <a id="phase-6"></a>

Script: `scripts/doctor/06_ocel.sh`. Subsystem: event-log subsystem.
`tests/ocel_tests.rs` is expected per D6 but not yet authored — the phase
exits 3 loudly until it exists.

| id | objective | state assertions (definitional, pre-implementation) | status |
|---|---|---|---|
| TEST-017 | compression ratio at/above floor on a reference corpus | per-file compressed/original ratio ≥ floor; sub-floor exits nonzero | pending (test file missing) |
| TEST-018 | segment-vector rehydration equals the raw log | rehydrated log equals raw log field-by-field (structural diff, never summary-only) | pending (test file missing) |
| TEST-019 | sync-digest round-trip is stable | same log → same digest; any mutation → different digest | pending (test file missing) |

Reference designs: `docs/design/ocel2-compression.md`,
`docs/design/sync-digest-design.md`.

## Phase 7 — Performance <a id="phase-7"></a>

Script: `scripts/doctor/07_performance.sh`. Subsystem: throughput and latency
bounds (NFR-01: p95 < 15 ms @ 100k rows).

**TEST-020 — sustained ingest RSS** — pending

- Subsystem: performance (memory during load).
- Objective: RSS stays bounded during sustained ingest at gate throughput.
- Mechanism: companion assert alongside `tests/gate_ingest_throughput.rs`.
- State assertions: peak RSS under the declared bound across the sustained
  ingest leg.
- Note: companion assert not yet authored; the throughput gate itself
  (TEST-022) already runs.

**TEST-021 — latency gate** — implemented-elsewhere

- Subsystem: performance (query latency).
- Objective: p95 < 15 ms @ 100k rows (NFR-01).
- Mechanism: `tests/gate_v01_latency.rs` (`gate_v01_p95_under_15ms`), run by
  the Phase 7 script.
- State assertions: measured p95 under 15 ms over the seeded 100k-row table;
  machine specs + subject binding printed into the receipt.
- Note: already carried by `tests/gate_v01_latency.rs`; runs in Phase 7
  today.

**TEST-022 — ingest throughput gate** — implemented-elsewhere

- Subsystem: performance (ingest).
- Objective: lower-bound throughput gate over seeded 6-column batch ingest.
- Mechanism: `tests/gate_ingest_throughput.rs`
  (`gate_ingest_throughput_native`), run by the Phase 7 script.
- State assertions: sustained + segment legs each clear their per-leg
  rows/sec bound (asserted in-test).
- Note: already carried by `tests/gate_ingest_throughput.rs`; runs in Phase 7
  today.

## Phase 8 — Concurrency <a id="phase-8"></a>

Script: `scripts/doctor/08_concurrency.sh`. Subsystem: file-lock contract.

**TEST-023 — flock contract** — runnable-native-today

- Subsystem: concurrency (file locking).
- Objective: two writers cannot hold the DB lock simultaneously.
- Mechanism: `tests/flock_tests.rs`, run by the Phase 8 script.
- State assertions: second opener gets a typed lock error; after release, a
  new opener succeeds.

## Phase 9 — Court ladder <a id="phase-9"></a>

Script: `scripts/doctor/09_court_ladder.sh`. Subsystem: gate courts +
aggregate boundary.

**TEST-005 — crash chaos court** — implemented-elsewhere

- Subsystem: courts (crash chaos).
- Objective: recovery after a real forced crash mid-write.
- Mechanism: `scripts/courts/gate_v02.sh` (crash-chaos court), executed by
  `scripts/courts/run_all.sh` in Phase 9.
- State assertions: recovered DB matches pre-crash digests; the court must
  flip under its non-vacuity shim — a court that cannot flip is FAIL, not
  PASS.
- Note: the law already lives in `scripts/courts/gate_v02.sh`; Phase 9 runs
  it via the ladder.

**TEST-024 — court battery verdict table** — runnable-native-today

- Subsystem: courts (battery).
- Objective: the full gate court battery runs fail-closed with real exits.
- Mechanism: `scripts/courts/run_all.sh` (gate_v01..v05), Phase 9.
- State assertions: per-court PASS/FAIL/SKIP/CORRUPTED verdicts with real
  exit codes; any FAIL or CORRUPTED breaches the boundary.

**TEST-025 — aggregate boundary** — runnable-native-today

- Subsystem: courts (aggregate).
- Objective: the `just doctor` exit code IS the boundary.
- Mechanism: `scripts/doctor.sh` — 10 phases, `any_fail` collapse.
- State assertions: exit 0 only when all 10 phases pass; SKIP,
  NOT-IMPLEMENTED, and FAIL all collapse to exit 1; per-phase verdict lines
  (`[PASS]` / `[SKIP]` / `[NOT-IMPLEMENTED]` / `[FAIL]`) name each phase.

## Test index

| id | phase | one-line law | status |
|---|---|---|---|
| TEST-001 | 0 | header hash attestation | runnable-native-today |
| TEST-002 | 0 | strip contract | runnable-native-today |
| TEST-003 | 1 | checkpoint round-trip | runnable-native-today |
| TEST-004 | 1 | 4 KiB page alignment | runnable-native-today |
| TEST-005 | 9 | crash chaos court | implemented-elsewhere |
| TEST-006 | 1 | WAL drain | runnable-native-today |
| TEST-007 | 2 | value/type round-trip | runnable-native-today |
| TEST-008 | 2 | serde_json interop | runnable-native-today |
| TEST-009 | 2 | blob literal | implemented-elsewhere |
| TEST-010 | 3 | host-VFS read-through | FR-01-blocked |
| TEST-011 | 3 | host-VFS write-back | FR-01-blocked |
| TEST-012 | 3 | host-VFS large-blob | FR-01-blocked |
| TEST-013 | 4 | OOM typed abort at 256 MB | implemented-elsewhere |
| TEST-014 | 4 | leak audit (RSS delta) | runnable-native-today |
| TEST-015 | 5 | OPFS crash persistence | FR-01-blocked |
| TEST-016 | 5 | burn-in soak, bounded memory | FR-01-blocked |
| TEST-017 | 6 | compression floor | pending (test file missing) |
| TEST-018 | 6 | rehydration equivalence | pending (test file missing) |
| TEST-019 | 6 | sync-digest round-trip | pending (test file missing) |
| TEST-020 | 7 | sustained ingest RSS | pending (assert not authored) |
| TEST-021 | 7 | p95 latency < 15 ms | implemented-elsewhere |
| TEST-022 | 7 | ingest throughput gate | implemented-elsewhere |
| TEST-023 | 8 | flock mutual exclusion | runnable-native-today |
| TEST-024 | 9 | court battery verdict table | runnable-native-today |
| TEST-025 | 9 | aggregate boundary exit code | runnable-native-today |

## Receipt

Authored by lane D1, 2026-10-19. Every mechanism path above was verified to
exist on disk at authoring time: `scripts/doctor/00..09*.sh`,
`scripts/doctor.sh`, `scripts/courts/{gate_v02,run_all}.sh`,
`crates/frozen-duckdb/tests/{doctor_storage_tests,variant_tests,
oom_graceful_abort,doctor_leak_audit,gate_v01_latency,
gate_ingest_throughput,flock_tests}.rs`, and the blob literal in
`crates/frozen-duckdb/src/duckdb/types/mod.rs` (`test_blob`,
`test_empty_blob`). The battery runs via `just doctor`; its live landing
state is D11's `docs/design/doctor-implementation-map.md`.
