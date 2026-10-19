# Doctor Blocked-Phase Register

- **Lane**: D9 (write; `docs/design/doctor-blocked-phases.md` only; NO git).
- **Companion to**: `docs/design/doctor-verification-blueprint.md` (battery
  definitions), `docs/design/doctor-implementation-map.md` (D11, live status).
- **Scope**: the `just doctor` phases that are blocked by design — Phase 3
  (host-VFS, TEST-011/012) and Phase 5 (chrome OPFS, TEST-015/016/017) —
  plus CI-tier extensions (TEST-003 wasm size audit, TEST-014 ASan tier).
- **Blocking requirement**: FR-01 — the `duckdb_wasm` artifact + version
  gate (`docs/design/requirements-traceability.md`). No wasm artifact exists
  today; phase scripts `scripts/doctor/03_host_vfs.sh` and
  `scripts/doctor/05_chrome.sh` exit 3 NOT-IMPLEMENTED by design. Fail-closed:
  never a green, never a silent skip.

## Boundary law (restated for blocked phases)

- Phase exit 3 = loud divergence, collapses to aggregate exit 1 in
  `scripts/doctor.sh`. A blocked phase is a breach, not a pass.
- No court in this register has run. Every mechanism below is design until a
  real receipt lands on the exact subject. Standing of every row: BLOCKED.

## Phase 3 — Host-VFS (blocked)

Script: `scripts/doctor/03_host_vfs.sh` (exit 3 NOT-IMPLEMENTED).
Contract: `docs/design/host-range-vfs-contract.md`.

### TEST-011 — host-VFS write / journal replay

| field | value |
|---|---|
| objective | write-back replays to host bytes exactly |
| unblock precondition | (1) real emsdk installed on PATH;
  `scripts/build_wasm_duckdb.sh` produces `duckdb.wasm` (FR-01).
  (2) host-call VFS write path implemented per
  `docs/design/host-call-vfs.md`. |
| witnessing court | GATE-V-05 spec (`docs/design/gate-v05-hostvfs-transfer.md`)
  — host-side transfer counter; write-back leg asserts journal replay
  reproduces host bytes byte-exactly (digest compare). |
| non-vacuity mutation | corrupt one replayed host byte post-write; the court
  must FAIL on the digest mismatch. A court that still passes under the
  corruption is VACUOUS and exits 1 as such. |
| standing | BLOCKED |

### TEST-012 — host-VFS bulk transfer under bounded memory

| field | value |
|---|---|
| objective | large-blob transfer completes with peak memory under the
  declared bound |
| unblock precondition | same FR-01 chain as TEST-011 (emsdk →
  `duckdb.wasm`), plus the host-call VFS bulk/read path and a declared
  peak-memory bound. |
| witnessing court | GATE-V-05 spec — `total_bytes_transferred` counter over
  one query session; bulk leg additionally asserts the peak-memory ceiling
  from the per-call log (`per_call: [(offset, len)]`). |
| non-vacuity mutation | inflate one range-copy length in the host counter;
  the budget court must FAIL (counter exceeds bound or log disagrees with
  copied bytes). A court that accepts inflated counts is VACUOUS. |
| standing | BLOCKED |

Shared precondition for both: until `scripts/build_wasm_duckdb.sh` yields a
real `duckdb.wasm`, no leg of GATE-V-05 beyond the native-host accounting leg
(PARTIAL_ALIVE at best) can run.

## Phase 5 — Chrome OPFS burn-in (blocked)

Script: `scripts/doctor/05_chrome.sh` (exit 3 NOT-IMPLEMENTED).
Pattern: `docs/design/chrome-burnin-court.md`.

### TEST-015 — OPFS persistence survives `Page.crash`

| field | value |
|---|---|
| objective | post-relaunch OPFS checksums equal pre-crash digests after WAL
  replay |
| unblock precondition | FR-01 wasm artifact (emsdk → `duckdb.wasm`); CDP
  harness driving `chrome-headless-shell` with a real OPFS profile dir per
  `docs/design/chrome-burnin-court.md`. |
| witnessing court | chrome-burnin-court Court 1 (forced `Page.crash`
  mid-transaction; checksum equality is the only admissible post-state). |
| non-vacuity mutation | skip the WAL replay step after relaunch; the court
  must FAIL on the checksum comparison. A relaunch court that passes without
  replay is VACUOUS. |
| standing | BLOCKED |

### TEST-016 — burn-in soak, bounded memory

| field | value |
|---|---|
| objective | bounded-duration real workload; RSS/JS-heap samples under
  budget via bounded polls (no-sleep doctrine) |
| unblock precondition | FR-01 wasm artifact; CDP harness; workload + poll
  config per `chrome-burnin-court.md`. |
| witnessing court | chrome-burnin-court soak court: RSS/heap under budget at
  every bounded poll over the soak window. |
| non-vacuity mutation | inject a deliberate per-iteration heap retention
  (leak shim) into the workload; the soak court must FAIL its budget before
  the window ends. A soak that tolerates the leak shim is VACUOUS. |
| standing | BLOCKED |

### TEST-017 — chrome OPFS lock race

| field | value |
|---|---|
| objective | OPFS sync access handle lock conflict is typed, never a hang or
  silent corruption |
| unblock precondition | FR-01 wasm artifact; CDP harness; two workers
  contending one OPFS file. |
| witnessing court | chrome-burnin-court lock-race court (concurrent open of
  one OPFS handle; assert typed failure and uncorrupted post-state). |
| non-vacuity mutation | remove the typed-lock guard; the court must FAIL
  (hang detected by deadline poll, or corrupted post-state). |
| standing | BLOCKED |
| note | `doctor-verification-blueprint.md` currently numbers TEST-017 in
  Phase 6 (OCEL compression, pending). The chrome lock-race row here follows
  the dispatch numbering; the D11 implementation map owns the authoritative
  id reconciliation. |

## CI-tier extensions (not blocked — scheduled extensions of live tests)

### TEST-003 (wasm size audit) — CI tier

| field | value |
|---|---|
| objective | `duckdb.wasm` size stays within the binary-size budget |
| mechanism | post-build size gate over the FR-01 artifact, budgeted per
  `docs/design/binary-size-budget.md`; runs in CI on every wasm build. |
| status | CI-tier extension — requires FR-01 to have an artifact to measure;
  recorded here so the budget gate is not lost when FR-01 lands. |
| standing | BLOCKED (inherits FR-01; no artifact to audit) |

### TEST-014 full ASan tier — CI tier

| field | value |
|---|---|
| objective | full leak/UB audit under AddressSanitizer, beyond the RSS-delta
  leak audit that runs in Phase 4 today |
| mechanism | `cargo test --workspace` with
  `-Zsanitizer=address` in a nightly CI job; ASan reports are hard failures. |
| status | CI-tier extension of the runnable-native-today TEST-014; not
  blocked by FR-01, scheduled as a CI job addition. |
| standing | PARTIAL_ALIVE (RSS tier live; ASan tier scheduled) |

## Landing sequence (when FR-01 unblocks)

1. emsdk installed; `scripts/build_wasm_duckdb.sh` produces `duckdb.wasm`
   with a pinned, attested version (FR-01 closes).
2. Host-call VFS implemented (`docs/design/host-call-vfs.md`); Phase 3
   scripts swap exit-3 stubs for real courts; TEST-011/012 witness.
3. CDP harness + chrome-burnin-court land; Phase 5 unblocks; TEST-015/016/017
   witness.
4. TEST-003 wasm size audit gates CI on the artifact; TEST-014 ASan tier
   lands as a nightly CI job.
