# Doctor Implementation Map (TEST-001..025)

- **Lane**: D10 (write; this file only; NO git)
- **Status**: per-test implementation mapping for the 25-test doctor battery.
  Verified against disk 2026-10-19 in this canonical checkout. Complements
  `docs/design/doctor-verification-blueprint.md` (battery definition, D1).
  Note: `docs/design/doctor-blocked-phases.md` does NOT exist on disk at
  authoring time (searched; absent) — the NOT-IMPLEMENTED rationale below is
  inferred from phase-script stubs, not that doc.
- **Status legend**:
  - IMPLEMENTED — artifact on disk; invocation path identified.
  - PARTIAL — native partial coverage only.
  - IN-FLIGHT — owning lane may still be writing.
  - NOT-IMPLEMENTED — no artifact; phase script stub or exits 3 loudly.

## Phase script index (verified on disk)

| Phase | Script | Invocation (as read from the script) |
|---|---|---|
| 00 | `scripts/doctor/00_preflight.sh` | inline TEST-001/002 checks |
| 01 | `scripts/doctor/01_storage_wal.sh` | `cargo test --test frozen_duckdb_tests` |
| 02 | `scripts/doctor/02_types_variant.sh` | `cargo test --test variant_tests` |
| 03 | `scripts/doctor/03_host_vfs.sh` | none — 4-line stub |
| 04 | `scripts/doctor/04_memory_guard.sh` | `cargo test --test oom_graceful_abort` |
| 05 | `scripts/doctor/05_chrome.sh` | none — 4-line stub |
| 06 | `scripts/doctor/06_ocel.sh` | `cargo test --test ocel_tests` (file absent → exit 3) |
| 07 | `scripts/doctor/07_performance.sh` | loop: gate_v01_latency + gate_ingest_throughput |
| 08 | `scripts/doctor/08_concurrency.sh` | `cargo test --test flock_tests` |
| 09 | `scripts/doctor/09_court_ladder.sh` | `scripts/courts/run_all.sh` (gate_v01..v05) |

## TEST mapping

Integration test paths are relative to `crates/frozen-duckdb/tests/`.
Format per test: status / implementing artifact (verified on disk) /
invoking doctor phase script.

- **TEST-001** — IMPLEMENTED.
  Artifact: `scripts/doctor/checks/attest_pin.sh` (D3); header hash
  attestation also inline in the phase script.
  Invoke: `scripts/doctor/00_preflight.sh`.
- **TEST-002** — IMPLEMENTED.
  Artifact: inline strip-contract check in `00_preflight.sh`
  (release binary symbol contract).
  Invoke: `scripts/doctor/00_preflight.sh`.
- **TEST-004** — IMPLEMENTED (wiring IN-FLIGHT).
  Artifact: `doctor_storage_tests.rs` (D4), on disk.
  Invoke: `01_storage_wal.sh` — currently invokes `frozen_duckdb_tests`,
  not the owner file (gap 1 below).
- **TEST-005** — IN-FLIGHT (artifact absent).
  Artifact: `scripts/courts/crash_chaos_native.sh` (X4) NOT on disk;
  `scripts/courts/` holds only `gate_v01..v05.sh` + `run_all.sh`.
  Invoke: `09_court_ladder.sh` via `courts/run_all.sh`, once present.
- **TEST-006** — IMPLEMENTED (wiring IN-FLIGHT).
  Artifact: `doctor_storage_tests.rs` (D4), on disk.
  Invoke: same as TEST-004.
- **TEST-007** — PARTIAL (native partial per work order).
  Artifact: `variant_tests.rs`.
  Invoke: `02_types_variant.sh`.
- **TEST-008** — IMPLEMENTED.
  Artifact: `doctor_types_tests.rs` (D5), on disk.
  Invoke: `02_types_variant.sh` invokes `variant_tests` only; the owner
  file is direct-run today (`cargo test --test doctor_types_tests`).
- **TEST-009** — PARTIAL.
  Artifact: blob literal coverage in
  `crates/frozen-duckdb/src/duckdb/types/mod.rs`
  (`test_blob`, `test_empty_blob`, lines 221/233).
  Invoke: `02_types_variant.sh` (lib unit tests, not a `tests/` file).
- **TEST-010** — IMPLEMENTED.
  Artifact: `doctor_types_tests.rs` (D5), on disk.
  Invoke: same as TEST-008.
- **TEST-011** — NOT-IMPLEMENTED.
  Phase 03 (`03_host_vfs.sh`) is a 4-line stub with no invoke.
- **TEST-012** — NOT-IMPLEMENTED.
  Phase 03 stub, as TEST-011.
- **TEST-013** — IMPLEMENTED.
  Artifact: `oom_graceful_abort.rs` (X9), on disk.
  Invoke: `04_memory_guard.sh`.
- **TEST-014** — IMPLEMENTED (wiring IN-FLIGHT).
  Artifact: `doctor_leak_audit.rs` (D6), on disk; no phase script
  invokes it yet.
- **TEST-015** — NOT-IMPLEMENTED.
  Phase 05 (`05_chrome.sh`) is a 4-line stub with no invoke.
- **TEST-016** — NOT-IMPLEMENTED. No artifact, no phase wiring found.
- **TEST-017** — NOT-IMPLEMENTED. No artifact, no phase wiring found.
- **TEST-018** — IN-FLIGHT.
  Artifact: `doctor_ocel_tests.rs` (D8) NOT on disk; the phase's
  expected `ocel_tests.rs` is also absent.
  Invoke: `06_ocel.sh` → exits 3 NOT-IMPLEMENTED loudly today.
- **TEST-019** — IN-FLIGHT. Same as TEST-018.
- **TEST-020** — IN-FLIGHT. Same as TEST-018.
- **TEST-021** — IMPLEMENTED.
  Artifact: `gate_v01_latency.rs` (X3), on disk.
  Invoke: `07_performance.sh`.
- **TEST-022** — IMPLEMENTED.
  Artifact: `gate_ingest_throughput.rs` (X8), on disk.
  Invoke: `07_performance.sh`.
- **TEST-023** — IMPLEMENTED (wiring IN-FLIGHT).
  Artifact: `doctor_concurrency_tests.rs` (D7), on disk.
  Invoke: `08_concurrency.sh` — currently invokes `flock_tests`
  (gap 1 below).
- **TEST-024** — IMPLEMENTED (wiring IN-FLIGHT).
  Artifact: `doctor_concurrency_tests.rs` (D7), on disk.
  Invoke: same as TEST-023.
- **TEST-025** — NOT-IMPLEMENTED. No artifact, no phase wiring found.

## Wiring gaps for the coordinator

1. `01_storage_wal.sh` and `08_concurrency.sh` invoke legacy files
   (`frozen_duckdb_tests`, `flock_tests`) instead of the D4/D7 owner
   files (`doctor_storage_tests`, `doctor_concurrency_tests`).
2. `02_types_variant.sh` does not invoke `doctor_types_tests`.
3. `doctor_leak_audit.rs` (TEST-014) has no invoking phase script.
4. Phase 06's expected filename (`ocel_tests.rs`) does not match the
   D8 owner filename (`doctor_ocel_tests.rs`); both absent today.
5. `scripts/courts/crash_chaos_native.sh` (TEST-005, X4) absent.
6. `docs/design/doctor-blocked-phases.md` absent; referenced rationale
   doc must land or the NOT-IMPLEMENTED rows stay inference-based.

## Stage-to-artifact reconciliation (Wave 10, lane H4, 2026-10-19)

The 11-stage ladder (operator spec; `docs/design/court-ladder-11.md`) is
now formalized as the doctor stage matrix: 11 stages 00..10, one script
per stage under `scripts/doctor/`, aggregated fail-closed by
`scripts/doctor.sh` (11 phase banners 00..10). Old phase-N filenames map
to new stage filenames as follows.

| old phase-N | new stage file | stage role | standing on disk (2026-10-19) |
|---|---|---|---|
| 00_preflight | 00_preflight.sh | probes + sys audit handoff | executable; bin_audit wired |
| (none) | 01_sys_audit.sh | sys audit | = `checks/bin_audit.sh` |
| 01_storage_wal | 02_storage_wal.sh | storage/WAL | 2/2 green |
| 02_types_variant | 03_types_variant.sh | types/variant | 3/3 green |
| 03_host_vfs | 04_host_vfs.sh | host VFS transfer | BLOCKED FR-01 |
| 04_memory_guard | 05_memory_guard.sh | memory ceiling/OOM | PARTIAL_ALIVE |
| 05_chrome | 06_browser_opfs.sh | browser/OPFS | BLOCKED FR-01 |
| 06_ocel | 07_ocel_compress.sh | OCEL/compression | 94.93% ALIVE |
| 07_performance | 08_perf_sla.sh | perf SLA fence | non-vacuous PASS |
| 08_concurrency | 09_concurrency.sh | concurrency | wired to D7 owner file |
| 09_court_ladder | 10_verify_economics.sh | economics court | IN-FLIGHT G5 |

Notes:

- Stage 01 sys-audit resolves to
  `scripts/doctor/checks/bin_audit.sh` (E3 artifact);
  `01_sys_audit.sh` only propagates its verdict (E-wire, lane H1/H2).
- Stage 11 (ladder stage `sleep-gate-sdk` in `court-ladder-11.md`) is
  discharged here as the stage-11 economics court: the doctor aggregate
  ends at `10_verify_economics.sh`, which runs the warehouse-sim
  economics court at `scripts/warehouse-sim/economics/run_court.sh`.
  That runner is NOT on disk at reconciliation time (exit 3 loud
  NOT-IMPLEMENTED) — economics standing is IN-FLIGHT (G5), never a
  faked economics pass.
- Stale remnant: `scripts/doctor/09_court_ladder.sh` survives alongside
  `09_concurrency.sh`; it is not in the `doctor.sh` PHASES array and
  should be retired or rewired by the coordinator at integration.
- Standings above are as-found-on-disk at reconciliation read time
  (2026-10-19), cited from the stage scripts and their owner test
  files, not from lane claims. Host-VFS and browser/OPFS remain BLOCKED
  on FR-01 (engine identity / cache slot), consistent with the E-wave
  quarantine finding in CAMPAIGN-RECEIPT.md.
