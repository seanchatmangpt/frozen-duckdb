# DEFINITION-OF-DONE — frozen-duckdb — v2.0.0-dev

Campaign DoD in the MILESTONE.md spirit. Every item carries: requirement,
owner (owning artifact/lane), standing, evidence pointer. Standing
vocabulary: ALIVE / PARTIAL_ALIVE / PROPOSED / BLOCKED (plus IN-FLIGHT for
running work, REQUIRED for mandatory open integration steps).

**Engine-identity rule**: the rebuilt dylib
`~/.frozen-duckdb/cache/v2.0.0-dev-arm64/libduckdb.dylib` self-reports
`v2.0.0-dev88595` (sha 8a22ef19, thin arm64, F10 rebuild + test-validation
green). Engine-identity probes accept that ID
(`scripts/courts/crash_chaos_native.sh:165`); the pre-rebuild cache-poison
VOID rule is discharged — all ALIVE standings below are witnessed
post-rebuild unless explicitly marked.

## Definition of Done checklist

- [x] **DOD-1 Engine bump** — owner F10 — ALIVE
  - Requirement: version flip to v2.0.0-dev88595, sha 8a22ef19, thin
    arm64; test-validation green on the rebuilt dylib.
  - Evidence: F10 rebuild receipt (test-validation green);
    engine-identity probe whitelist
    `scripts/courts/crash_chaos_native.sh:165`. Supersedes the earlier
    "receipt pending / self-report flip open" state.
- [x] **DOD-2 VARIANT surface** — owner X6 — ALIVE (native)
  - Requirement: variant 5/5, serde 6/6, lib 140/140, native.
  - Evidence: X6 receipt (variant 5/5 + serde 6/6 + lib 140/140 green).
    Was PARTIAL_ALIVE pre-rebuild witness; re-witnessed post-rebuild.
- [x] **DOD-3 Storage court 2/2 (WAL)** — owners D4/F5 — ALIVE
  - Evidence: F5 receipt; doctor stage `scripts/doctor/02_storage_wal.sh`
    2/2 green (H4 stage-matrix read).
- [x] **DOD-4 Types court 3/3** — owners D5+F4 — ALIVE
  - Evidence: D5+F4 receipts; stage `scripts/doctor/03_types_variant.sh`
    3/3 green (H4 stage-matrix read).
- [x] **DOD-5 Leak audit +0-window** — owner D6 — ALIVE
  - Evidence: D6 receipt green plus the coordinator clippy fix; stage
    `scripts/doctor/05_memory_guard.sh`.
- [x] **DOD-6 Concurrency court 2/2 incl. vacuity catch** — owner D7 —
  ALIVE
  - Evidence: D7 receipt; stage `scripts/doctor/09_concurrency.sh`.
- [x] **DOD-7 OCEL** — owners D8/F3 — ALIVE
  - Requirement: 94.93% @100k, 97.01% @1M, SHIM flip witnessed
    post-rebuild.
  - Evidence: D8/F3 receipts; SHIM flip re-witnessed on the rebuilt
    dylib. Was PARTIAL_ALIVE pending the SHIM re-run.
- [x] **DOD-8 Latency gate non-vacuous: 12.3ms** — owners F1+X3 — ALIVE
  - Evidence: F1 receipt; X3 native court
    `crates/frozen-duckdb/tests/gate_v01_latency.rs`. Stage
    `scripts/doctor/08_perf_sla.sh` non-vacuous PASS (H4 read).
- [x] **DOD-9 Throughput: 5-9x headroom** — owners F2/X8 — ALIVE
  - Evidence: F2 receipt; X8 native court
    `crates/frozen-duckdb/tests/gate_ingest_throughput.rs`.
- [x] **DOD-10 OOM graceful abort, both legs** — owner X9 — ALIVE
  - Evidence: X9 receipt; court
    `crates/frozen-duckdb/tests/oom_graceful_abort.rs` now run and
    receipted. Was PARTIAL_ALIVE (run not yet receipted).
- [ ] **DOD-11 11-stage ladder live: 6/11 PASS, 3 loud-NI, 2 real-FAIL** —
  owner H2 — PARTIAL_ALIVE
  - Evidence: `scripts/doctor.sh` stage banners; H2 receipt. The
    final-state 6/11 with loud blocks is the dispatched end state, not a
    shortfall — see DOD-12/16/17 for the blocked legs.
- [ ] **DOD-12 Doctor + court-x2 wiring + fleet battery** — owners
  H2/H3/E4/T1-T5 — ALIVE
  - Requirement: bin-audit verdict propagates through the sys-audit
    stage; crash-chaos court wired; fleet hardware battery executable.
  - Evidence: `scripts/doctor/01_sys_audit.sh`;
    `scripts/courts/crash_chaos_native.sh` (E2; X4 defect withdrawn to
    the fixture layer per the R5 CORRECTION RECORD — Z2 court green on
    3 legs, GATE-V-02 native leg PARTIAL_ALIVE green); stage 12 fleet
    run EXIT=3 with every executable leg PASS (A/B-proxy/C/D-chaos
    2-real-kill-9 + no-op-kill flip DETECTED/D-compression/E-duckdb) in
    CAMPAIGN-RECEIPT.md T6 CORRECTION.
- [ ] **DOD-13 ggen projections + value-audit courts** — owners V4/V5 —
  PARTIAL_ALIVE
  - Requirement: projections byte-identical; genesis 58/58; receipt
    chain GREEN; value-audit courts (V2-V5) at their landed standings.
  - Evidence: `make verify` / `make gates` output; V2 DOCS_RS PASS 89/89
    and V2a 22/22 in `receipts/VERIFIER-RECEIPTS-LOG.md`; V3/V4/V5
    standings as landed in the lane receipts.
- [ ] **DOD-14 Blueprint / PRD / ontology layers** — owners BP lanes/R15 —
  PROPOSED
  - Evidence: `docs/design/README.md` (BP16 index);
    `schema/blueprint.ttl`; `schema/requirements.ttl`; GATE-V runners
    `scripts/courts/run_all.sh` all five NOT-IMPLEMENTED, EXIT=1
    (fail-closed, R15 intake). No court flipped; no layer claims ALIVE.
- [ ] **DOD-15 Known-open: 21 mangled exports (upstream)** — owner V-lane —
  BLOCKED (upstream)
  - Evidence: V receipt.
- [ ] **DOD-16 Known-open: wasm/OPFS FR-01/FR-02 + KD holding-areas** —
  stages 04_host_vfs / 06_browser_opfs — PROPOSED
  - No wasm artifact receipt exists (only `scripts/build_wasm_duckdb.sh`
    and `docs/design/wasm-abi-surface.md`); FR-01 stays unflipped. The
    GAP6 KNOWN_DIVERGENCES register is the KD holding-area for these
    divergences. Also remaining PROPOSED: wasm scorecard leg, cgroup
    leg (Linux-root-gated rc=2, per T6), BEAM leg (BLOCKED-OTHER-REPO,
    smalt), formal-ε-DP.
- [ ] **DOD-17 Warehouse-sim + economics court** — owners G7/S-lanes/Z7 —
  PARTIAL_ALIVE
  - Evidence: economics PASS per Z7 receipt; G7 sovereign probe leg
    landed (`warehouse-sim/economics/sovereign_probe/`, egress 0 /
    invoice $0.00 by construction); sim-leg runner for the three
    fail-closed assertions not on disk (CAMPAIGN-RECEIPT.md G7). S-wave
    layers (S1-S6) IN-FLIGHT; G2 render self-healed (byte-identical
    double-sync green); G6 receipt-chain court FAIL-closed at
    `warehouse-sim/economics/court.ts:117` by design until S5's minimal
    fix. Cost constants remain ASSUMPTION-UNVERIFIED.
- [ ] **DOD-18 Known-open: crates.io hold decision** — owner release-op —
  PROPOSED
  - Evidence: operator decision pending; `gates/dry-run-publish/`.
- [x] **DOD-19 Cache-poison incident re-witness** — owner coordinator —
  ALIVE (discharged)
  - Evidence: rebuilt dylib self-reports `v2.0.0-dev88595` (F10); court
    battery re-witnessed post-rebuild (DOD-2..10). The pre-rebuild VOID
    rule is discharged.
- [ ] **DOD-20 Final-integration reconciliation pass** — owner coordinator
  — REQUIRED (open)
  - Requirement: re-read every co-touched surface on disk, reconcile
    against lane receipts, only then commit (rogue-writer incident class,
    witnessed 4x+1x+1 cache event).
  - Evidence: H4 stage-matrix finding; this DOD refresh is an input to
    that pass, not a substitute for it.

## Gate rule

DONE only when every item is ALIVE with a receipt newer than the rebuilt
dylib mtime, or explicitly BLOCKED/PROPOSED with a named authority. An item
claiming ALIVE with no receipt one-liner is SUSPECT.
