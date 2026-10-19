# KNOWN_DIVERGENCES

Registered divergences between claimed and witnessed behavior for the
v2.0.0-dev campaign (2026-10-19). Pattern: smalt KD register — every row
carries surface, divergence, evidence receipt, and close condition.
A row with no evidence path is invalid; a closed row without a
post-close receipt is invalid.

| id | surface | divergence | evidence | close condition |
|---|---|---|---|---|
| KD-01 | native dylib symbol table | 21 C++-mangled `__Z*` exports present in the pinned dev dylib (C-ABI-only contract breached; upstream visibility flags) | scripts/doctor/checks/bin_audit.sh FAIL receipt (Z7/F7 runs); stage 00/01 red | upstream fences exports at v2.0.0, or an export-filter wrapper ships |
| KD-02 | community extensions | `flock` extension has no build for the dev pin (HTTP 404 at community-extensions.duckdb.org/b6013f7aa2) | flock_tests 2 tests `#[ignore]`ed with the 404 verbatim | upstream publishes a 2.0-dev-compatible flock build; un-ignore + rerun |
| KD-03 | warehouse-sim poll routes | rendered GET handlers returned static QUEUED (state machine dead code server-side) | G5/S4/S3 independent findings; journeys J1 stalled | CLOSED — POLL re-rendered stateful routes; journeys 4/4 settle SUCCEEDED over real HTTP |
| KD-04 | warehouse-sim route modules | generated routes exported helpers, violating Next route index-signature (.next/types errors) | S4 finding; .next/types output | CLOSED — POLL/GAP3 moved helpers to lib via templates; tsc scoped-clean |
| KD-05 | wasm artifact | no `frozen_duckdb.wasm` exists (FR-01); all wasm/OPFS legs BLOCKED | scripts/build_wasm_duckdb.sh exit 2 (broken brew emscripten); GAP1 attempt pending | real emsdk build produces duckdb.wasm passing the version gate, or upstream v2.0.0 release assets |
| KD-06 | host-VFS | `host-range://` VFS unimplemented (design only) | docs/design/host-range-vfs-contract.md; GATE-V-05 spec UNVERIFIED | host-call VFS implemented + GATE-V-05 ≤5MB court green |
| KD-07 | REV-001 latency clause | 10M-row compliance wall 4.23–6.05s vs <2.5s spec clause (measured under concurrent lane load) | /tmp/value_audit/rev001.json UNMET-BY-MEASURED | GAP8 idle-machine release re-measure; close if <2.5s, else clause stays hardware-honest |
| KD-08 | GATE-V-01 leak clause | allocator-stats C API absent on pinned engine surface; leak clause UNSUPPORTED-BY-CURRENT-SURFACE | gate_v01_latency.rs receipt (X3/F1); D6 deceleration-law proxy covers memory | upstream exposes allocator stats; else the D6 RSS-deceleration proxy remains the memory-leak witness |
| KD-09 | Class B hardware court | cgroup leg requires Linux + root; darwin proxy is PARTIAL ceiling | cgroup_low_ram_runner.sh exit 2 loud on darwin; T4 proxy 80.5MiB PASS | Linux+root host runs the real cgroup court; or DOCKER1's container leg witnesses the kernel limit |
| KD-10 | Class E actor leg | BEAM `:atomics`/`:ets` scheduler court lives in smalt (other repo) | test_fleet_class_e_telco module doc; BLOCKED-OTHER-REPO | smalt lands the actor-leg court; cross-repo receipt |
| KD-11 | browser substrate courts | chrome-headless-shell OPFS/CDP courts (TEST-015..017, GATE-V-02 CDP leg, GATE-V-04) FR-01/FR-02-blocked | doctor stages 04/06 exit-3 loud receipts | KD-05 closes first, then the CDP harness lands (BP2 spec) |
| KD-12 | anomaly detection precision | exact-set-match false: 148 noise-tail extra detections beyond the 10,000 injected (recall 1.0000) | /tmp/value_audit/rev004.json `exact_match: false`, false-positive ceiling assert | spec decision: tighten QUALIFY threshold or accept recall-only contract (recorded OBSERVED, not ALIVE) |
| KD-13 | cargo registry substrate | ~/.cargo/registry/src extractions corrupted by an over-aggressive cleanup actor (multiple lanes hit E0583/empty manifests); repaired in place by V3/T5/R3/GAP16 | lane receipts (V3, T5, R3, T1) | GAP16 full default-CARGO_HOME build green; substrate owner audits the cleanup actor |
| KD-14 | shared cache write race | concurrent ensure_binary writers could observe partial dylib (mid-write reads, "stale cached binary does not match its label") | X3/T2/F6 incident receipts | CLOSED — GAP9 atomic temp+rename discipline across all cache-persist sites; 23/23 builder tests |

See Also: docs/sjira/v2.0.0-dev/DEFINITION-OF-DONE.md
