# Lane partition — DuckDB 2.0 upgrade wave (2026-10-19)

Canonical checkout: /Users/sac/frozen-duckdb. One wave, 12 writer + 4 verifier + 4 auditor
lanes, disjoint file ownership. Coordinator owns all git transitions and commits. Lanes
never run git state-changing commands.

| lane | type | owns (write) |
|---|---|---|
| W1 | writer | crates/frozen-duckdb-sys/ (upstream pin: bindgen source SHA b6013f7aa2f36caabb9466ce1791667ea8a280bb, sys Cargo.toml) |
| W2 | writer | crates/frozen-duckdb-builder/vendored-headers/ (duckdb.h, duckdb.hpp refresh) |
| W3 | writer | crates/frozen-duckdb-builder/ snapshot/cache versioning (cache dir label v2.0.0-dev) |
| W4 | writer | crates/frozen-duckdb/src/duckdb/core/logical_type.rs (new type codes 40/41/42) |
| W5 | writer | crates/frozen-duckdb/src/duckdb/ struct audit (hugeint/interval/list_entry/string_t layouts) |
| W6 | writer | crates/frozen-duckdb/src/duckdb/config.rs (config surface; deferred options noted) |
| W7 | writer | crates/frozen-duckdb/src/ types surface, path-scoped per A9 finding 1: `types/`, `row.rs`, `vtab/arrow.rs` (public API against 2.0 headers) |
| W8 | writer | test-dependency/, test-validation/ manifests only (Cargo.toml; upstream-compat feature) |
| W9 | writer | .github/workflows/ (build-binaries.yml, ci.yml, ci-simple.yml, test-minimal.yml) |
| W10 | writer | ggen.toml + generated schema surface (Makefile via `make sync`, never hand-edited) |
| W11 | writer | docs/sjira/v2.0.0-dev/ (this window: _LANES.md, UPGRADE-WO.md, CAMPAIGN-RECEIPT.md, receipts/) |
| W12 | writer | scripts/ + README.md (build/download scripts, dev-prerelease docs) |
| V1 | verify | read-only; run real commands, report receipts (headers/layout probe) |
| V2 | verify | read-only; run real commands, report receipts (builder pin / duckdb_library_version) |
| V3 | verify | read-only; run real commands, report receipts (cache-poison falsifier) |
| V4 | verify | read-only; run real commands, report receipts (full verify gate + workspace tests) |
| A1–A4 | audit | read-only; findings with file:line + real output |

Shared seams: `ggen.toml` and the generated Makefile are wholly W10 (no other lane edits
them). W8 owns manifests only — no `src/` under test-dependency/test-validation. No
co-owned files; any cross-lane need routes through the coordinator.

## Wave 3: BP1-BP16 (blueprint)

Each BP lane owns exactly one blueprint document under `docs/design/` per the dispatch
manifest, plus the shared blueprint surface partitioned below. Coordinator repaired
coordinator-side where a BP lane refused; no BP lane edited another BP lane's doc.

| lane | type | owns (write) |
|---|---|---|
| BP1 | writer | its assigned docs/design/ blueprint doc + BLUEPRINT-WO.md + design/README.md |
| BP2-BP15 | writer | each lane's own assigned docs/design/ blueprint doc (one per lane, disjoint) |
| BP16 | writer | its assigned docs/design/ blueprint doc + schema/blueprint.ttl |
| BP16b | writer | gates/blueprint/blueprint-shapes.ttl + LICENSE-ENTERPRISE-DRAFT.md (BP16 split) |

## Wave 4: P1-P18 (PRD)

PRD work order: PRD-WO.md; README append is shared PRD-window surface owned by P18.
Note: `prd-ard-v2.md` was authored by the coordinator after P1's honest refusal
(coordinator repair disclosed here; P1 retains nominal ownership).

| lane | type | owns (write) |
|---|---|---|
| P1 | writer | prd-ard-v2.md (coordinator-authored after P1 refusal) + PRD-WO.md |
| P2 | writer | requirements-traceability.md |
| P3 | writer | boundary-abi-contract.md |
| P4 | writer | vfs-journal-design.md |
| P5 | writer | latency-bench-plan.md |
| P6 | writer | binary-size-budget.md |
| P7 | writer | compression-proof-protocol.md |
| P8 | writer | panic-confinement-protocol.md |
| P9 | writer | variant-shredding-design.md |
| P10 | writer | view-expansion-design.md |
| P11 | writer | persona-journey-map.md |
| P12 | writer | sync-digest-design.md |
| P13 | writer | ingest-throughput-plan.md |
| P14 | writer | acceptance-criteria-courts.md (flagged the _LANES.md gap this wave closes) |
| P15 | writer | schema/requirements.ttl |
| P16 | writer | gates/blueprint/requirements-shapes.ttl |
| P17 | writer | README.md append (PRD section only) |
| P18 | writer | cross-wave reconciliation reserve (no standing file; coordinator-routed) |

## Wave 5: W-EXT1-4 (80/20 + Host-VFS)

| lane | type | owns (write) |
|---|---|---|
| W-EXT1 | writer | extension-matrix-8020.md |
| W-EXT2 | writer | host-call-vfs.md |
| W-EXT3 | writer | build_wasm_duckdb.sh |
| W-EXT4 | writer | extension-abi-hub.md |

## Wave 6: R1-R16 (Revision-2 intake)

R lanes own exactly their Revision-2 intake files as dispatched — one file set per lane,
disjoint, mirroring the W1-W12 wave surfaces they revise.

| lane | type | owns (write) |
|---|---|---|
| R1-R16 | writer | each lane's own Revision-2 intake files per dispatch (disjoint, no co-owned) |

## Wave 7: X1-X20 (sovereign spec v26.10.19 intake + Chicago validation)

Each X lane owns exactly its dispatched file(s); disjoint, no co-owned. X16/X17 are
read-only BEFORE-receipt lanes. X20 owns this partition.

| lane | type | owns (write) |
|---|---|---|
| X1 | writer | sovereign-spec capture (v26.10.19 intake doc) |
| X2 | writer | master-traceability |
| X3 | writer | gate_v01_latency.rs |
| X4 | writer | crash_chaos_native.sh |
| X5 | writer | ocel-sovereign-schema-validation.md |
| X6 | verify | read-only; re-run the read-only suite, report receipts |
| X7 | verify | read-only; binary re-measure, report receipts |
| X8 | writer | gate_ingest_throughput.rs |
| X9 | writer | oom_graceful_abort.rs |
| X10 | writer | smalt capture |
| X11 | writer | ex4pm capture |
| X12 | writer | license rev2 |
| X13 | writer | court-ladder-11.md |
| X14 | writer | wasm-abi-surface.md |
| X16 | verify | read-only; BEFORE receipt |
| X17 | verify | read-only; BEFORE receipt |
| X18 | writer | index + WO |
| X19 | writer | residues |
| X20 | writer | docs/sjira/v2.0.0-dev/_LANES.md (this partition) |

Shared seams: none — append-only docs plus new test/court files, all disjoint.

## Wave 8: D1-D11 + E1-E6 (doctor battery finish)

Each lane owns exactly its dispatched file(s); disjoint, no co-owned. E5 is read-only.

| lane | type | owns (write) |
|---|---|---|
| D1 | writer | blueprint doc (doctor battery) |
| D2 | writer | doctor.sh + phase scripts + justfile |
| D3 | writer | attest_pin |
| D4 | writer | storage tests |
| D5 | writer | types tests |
| D6 | writer | leak audit |
| D7 | writer | concurrency tests |
| D8 | writer | ocel tests |
| D9 | writer | blocked-phases doc |
| D10 | writer | implementation map |
| D11 | writer | dual-pass spec |
| E1 | writer | phase wiring |
| E2 | writer | crash_chaos_native.sh |
| E3 | writer | bin_audit |
| E4 | writer | dual_pass.sh |
| E5 | verify | read-only; end-to-end validate, report receipts |
| E6 | writer | docs/sjira/v2.0.0-dev/_LANES.md + CAMPAIGN-RECEIPT.md (bookkeeping) |

Shared seams: none — doctor/phase scripts are D2, wiring is E1, courts and
tests land as new disjoint files.

## Wave 9: G1-G7 (warehouse-sim testbed)

Each lane owns exactly its dispatched file(s); disjoint, no co-owned. Source
of truth for the sim is `warehouse-sim/schema/warehouse_sim.ttl` — rendered
surfaces are never hand-edited. All sim cost constants are
ASSUMPTION-UNVERIFIED counterfactual parameters.

| lane | type | owns (write) |
|---|---|---|
| G1 | writer | warehouse-sim/schema/warehouse_sim.ttl (providers, states, cold-start, ledger) |
| G2 | writer | warehouse-sim/ggen.toml + rendered app/api/** routes + lib/ledger.ts |
| G3 | writer | determinism court (byte-identical ledger replay; drift = FAIL) |
| G4 | writer | economics court (three assertions, fail-closed) |
| G5 | writer | warehouse-sim/README.md |
| G6 | writer | docs/design/warehouse-sim-testbed.md |
| G7 | writer | _LANES.md Wave 9 partition + CAMPAIGN-RECEIPT.md (residues, append-only) |

Shared seams: none — the graph (G1) feeds ggen.toml (G2) by dependency order
G1 -> G2 -> G3/G4; docs lanes G5-G7 are disjoint from code lanes.

## Wave 10: H1-H4 (stage-matrix reconciliation)

Each lane owns exactly its dispatched file(s); disjoint, no co-owned. The
stage matrix is the renamed doctor ladder: 11 stages 00..10 under
`scripts/doctor/`, aggregated by `scripts/doctor.sh`. The D/E waves
renamed the phase scripts to the stage matrix — old phase-N filenames
(`01_storage_wal`, `02_types_variant`, `03_host_vfs`, `04_memory_guard`,
`05_chrome`, `06_ocel`, `07_performance`, `08_concurrency`,
`09_court_ladder`) became new stage filenames (`01_sys_audit`,
`02_storage_wal`, `03_types_variant`, `04_host_vfs`, `05_memory_guard`,
`06_browser_opfs`, `07_ocel_compress`, `08_perf_sla`, `09_concurrency`,
`10_verify_economics`); full old-to-new table in
`docs/design/doctor-implementation-map.md` (Wave 10 reconciliation
section). `sys-audit` resolves to `scripts/doctor/checks/bin_audit.sh`.

| lane | type | owns (write) |
|---|---|---|
| H1 | writer | doctor.sh 11-stage aggregate + stage renames (scripts/doctor/) |
| H2 | writer | court-x2 wiring (sys-audit propagation + crash-chaos court) |
| H3 | writer | stage 10 economics-court wiring (10_verify_economics.sh + G5 runner seam) |
| H4 | writer | implementation-map reconciliation + _LANES Wave 10 + CAMPAIGN-RECEIPT residues |

Shared seams: H3's `10_verify_economics.sh` depends on the G5 economics
runner at `scripts/warehouse-sim/economics/run_court.sh` (IN-FLIGHT);
H4 is docs-only and disjoint from H1-H3 code surfaces.

## Wave 11: S1-S7 (server/client S-wave, SaaS-disruption contrast)

Each lane owns exactly its dispatched file(s); disjoint, no co-owned.
Purpose: validate the SaaS-disruption contrast end-to-end — real HTTP
sim, SDK-fidelity clients, analyst journeys, chaos/concurrency, the
disruption scorecard, and admin observability. All sim cost constants
remain ASSUMPTION-UNVERIFIED. Lanes never run git state-changing
commands; the coordinator owns all git transitions.

| lane | type | owns (write) |
|---|---|---|
| S1 | writer | route contracts (`warehouse-sim/tests/routes_contract.test.ts`) |
| S2 | writer | SDK-fidelity clients (`warehouse-sim/client/warehouse_clients.ts` + `tests/clients.test.ts`) |
| S3 | writer | analyst journeys (`warehouse-sim/tests/journeys.test.ts` + `tests/helpers/`) |
| S4 | writer | chaos + concurrency probes against the live sim |
| S5 | writer | disruption scorecard (+ minimal authorized fix at `warehouse-sim/economics/court.ts:117` FAIL-closed site) |
| S6 | writer | admin observability surface |
| S7 | writer | bookkeeping: `_LANES.md` Wave 11 + `CAMPAIGN-RECEIPT.md` residues + `docs/design/warehouse-sim-testbed.md` testing-matrix append |

Shared seams: S4/S3 execute against the routes S1 contracts and the
clients S2 ships; execution is read-only against those files — disputes
route through the coordinator. S5's economics-court fix is limited to
the named line; anything wider is coordinator-routed.

## Wave 12: R1-R5 (revenue packs)

Each lane owns exactly its dispatched file(s); disjoint, no co-owned.
Purpose: the four revenue-pack courts (TEST-REV-001..004) against the
real v2.0.0-dev engine, their doctor stage, and the pack spec doc. All
price bands are ASSUMPTION-UNVERIFIED operator claims. Lanes never run
git state-changing commands; the coordinator owns all git transitions,
including wiring stage 11 into `scripts/doctor.sh` after the lanes land
(an admitted transition — no lane edits the aggregate).

| lane | type | owns (write) |
|---|---|---|
| R1 | writer | `crates/frozen-duckdb/tests/test_rev_compliance.rs` (REV-001 trial-balance compliance court) |
| R2 | writer | `crates/frozen-duckdb/tests/test_rev_process_mining.rs` (REV-002 process-mining court, native leg + loud FR-01/FR-02 browser-leg skip) |
| R3 | writer | `crates/frozen-duckdb/tests/test_rev_clean_room.rs` (REV-003 clean-room DP-join court) |
| R4 | writer | `crates/frozen-duckdb/tests/test_rev_edge_autonomy.rs` (REV-004 edge-autonomy court) |
| R5 | writer | `scripts/doctor/11_revenue_packs.sh` (fail-closed stage) + `docs/design/revenue-packs.md` + `_LANES.md` Wave 12 + `CAMPAIGN-RECEIPT.md` residues (bookkeep) |

Shared seams: R5's stage 11 executes the four test files R1-R4 own —
execution is read-only against those files; compile breaks in R1-R4
surfaces are reported fail-closed by the stage, never patched by R5
beyond the compile-freeze SLA in the standing fan-out rules.

## Wave 13: T1-T6 (fleet hardware classes)

Each lane owns exactly its dispatched file(s); disjoint, no co-owned.
Purpose: the five fleet hardware-class courts (TEST-FLEET-001..005) and
their fail-closed doctor stage 12. Spec bounds are operator claims,
ASSUMPTION-UNVERIFIED until a passing stage run receipts them. Lanes
never run git state-changing commands; the coordinator owns all git
transitions, including wiring stage 12 into `scripts/doctor.sh` (an
admitted transition after the lanes land). Physical-reality column and
class matrix: `docs/design/fleet-hardware-classes.md` (T6).

| lane | type | owns (write) |
|---|---|---|
| T1 | writer | `scripts/doctor/cgroup_low_ram_runner.sh` (Class B real gate, Linux+root; darwin exits 2 loud) |
| T2 | writer | `scripts/courts/fleet_d_chaos.sh` + `crates/frozen-duckdb/tests/test_fleet_class_d_marine.rs` (Class D chaos + compression) |
| T3 | writer | `crates/frozen-duckdb/tests/test_fleet_class_a_workstation.rs` (Class A native; wasm/OPFS FR-01 BLOCKED loud) |
| T4 | writer | `crates/frozen-duckdb/tests/test_fleet_class_b_industrial.rs` (Class B memory-envelope PROXY leg, PARTIAL_ALIVE ceiling) |
| T5 | writer | `crates/frozen-duckdb/tests/test_fleet_class_c_retail.rs` (Class C retail ledger, native on this host) |
| T6 | writer | `scripts/doctor/12_fleet_hardware.sh` (fail-closed stage) + `docs/design/fleet-hardware-classes.md` + `_LANES.md` Wave 13 + `CAMPAIGN-RECEIPT.md` residues (bookkeep) |

Shared seams: T6's stage 12 executes T1-T5's files — execution is
read-only against them; compile breaks in those in-flight surfaces are
reported fail-closed by the stage and owned by the file's lane under
the compile-freeze SLA, never patched by T6. T2's chaos court builds a
fixture from `/tmp/t2writer` (outside the checkout).

## Wave 14: GAP1-GAP16 (final reconciliation)

Each lane owns exactly its dispatched file(s); disjoint, no co-owned.
Purpose: close the court battery on the flipped v2.0.0-dev engine
(F10) and reconcile every residual IN-FLIGHT entry in
`CAMPAIGN-RECEIPT.md` against disk. Lanes never run git state-changing
commands; the coordinator owns all git transitions.

| lane | type | owns (write) |
|---|---|---|
| GAP1 | writer | court battery closure receipts in `receipts/` (storage/types/leak) |
| GAP2 | writer | OCEL + throughput/latency/oom court closure receipts (same dir) |
| GAP3 | writer | crash-chaos (Z2) closure + X4-defect withdrawal record append |
| GAP4 | writer | revenue-pack finals (REV-001..004) receipts append |
| GAP5 | writer | fleet hardware class finals (A-E) receipts append |
| GAP6 | writer | warehouse-sim finals (courts/scorecard/admin/clients/chaos/journeys) append |
| GAP7 | writer | doctor ladder aggregate run + loud-block record |
| GAP8 | writer | incident record final state (cache poison, re-witness, registry repair) |
| GAP9 | writer | doctor implementation-map final reconciliation append |
| GAP10 | writer | requirements-traceability final standing append |
| GAP11 | writer | sovereign-spec standing append |
| GAP12 | writer | dual-pass spec final state append |
| GAP13 | writer | `CAMPAIGN-RECEIPT.md` final reconciliation append + `_LANES.md` Wave 14 |
| GAP14 | writer | F10 engine-flip receipt (v2.0.0-dev88595, sha 8a22ef19) |
| GAP15 | writer | latency/throughput/oom receipt cross-check vs F10 identity |
| GAP16 | writer | FINAL.md draft + handoff pointer to next window |

Shared seams: GAP14/GAP15 both read the flipped dylib identity —
reads are disjoint from writes; GAP13's append cites the other GAP
lanes' landed receipts and does not own their files.
