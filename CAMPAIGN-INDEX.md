# CAMPAIGN-INDEX — frozen-duckdb v2.0.0-dev

One file a stranger reads to find everything. Status per
`docs/sjira/v2.0.0-dev/DEFINITION-OF-DONE.md` vocabulary:
ALIVE / PARTIAL_ALIVE / PROPOSED / BLOCKED.

## Engine & Courts

Rust test files (run: `cargo test -p frozen-duckdb --test <name>`).

| Path | Status |
|---|---|
| `crates/frozen-duckdb/tests/gate_v01_latency.rs` | ALIVE (gate v01 court) |
| `crates/frozen-duckdb/tests/gate_ingest_throughput.rs` | ALIVE (ingest gate) |
| `crates/frozen-duckdb/tests/oom_graceful_abort.rs` | ALIVE (memory ceiling) |
| `crates/frozen-duckdb/tests/doctor_storage_tests.rs` | ALIVE (WAL/storage) |
| `crates/frozen-duckdb/tests/doctor_types_tests.rs` | ALIVE (variant/types) |
| `crates/frozen-duckdb/tests/doctor_ocel_tests.rs` | ALIVE (OCEL compress) |
| `crates/frozen-duckdb/tests/doctor_concurrency_tests.rs` | ALIVE (concurrency) |
| `crates/frozen-duckdb/tests/doctor_leak_audit.rs` | ALIVE (leak audit) |
| `crates/frozen-duckdb/tests/variant_tests.rs` | ALIVE (variant 5/5) |
| `crates/frozen-duckdb/tests/value_audit_amplitude.rs` | ALIVE (value court) |
| `crates/frozen-duckdb/tests/value_audit_celonis.rs` | ALIVE (value court) |
| `crates/frozen-duckdb/tests/value_audit_datadog.rs` | ALIVE (value court) |
| `crates/frozen-duckdb/tests/test_fleet_class_a_workstation.rs` | ALIVE (fleet A) |
| `crates/frozen-duckdb/tests/test_fleet_class_b_industrial.rs` | ALIVE (fleet B) |
| `crates/frozen-duckdb/tests/test_fleet_class_c_retail.rs` | ALIVE (fleet C) |
| `crates/frozen-duckdb/tests/test_fleet_class_d_marine.rs` | ALIVE (fleet D) |
| `crates/frozen-duckdb/tests/test_fleet_class_e_telco.rs` | ALIVE (fleet E) |
| `crates/frozen-duckdb/tests/tpch_integration_test.rs` | ALIVE (TPCH) |
| `crates/frozen-duckdb/tests/dropin_compatibility_tests.rs` | ALIVE (drop-in) |
| `crates/frozen-duckdb/tests/flock_tests.rs` | ALIVE (flock) |

Shell courts — `scripts/courts/` (run: `bash scripts/courts/run_all.sh`).

| Path | Status |
|---|---|
| `scripts/courts/gate_v01.sh` .. `gate_v05.sh` | ALIVE (gates v01-v05) |
| `scripts/courts/crash_chaos_native.sh` | ALIVE (engine-identity probe) |
| `scripts/courts/fleet_d_chaos.sh` | ALIVE (fleet D chaos) |
| `scripts/courts/run_all.sh` | ALIVE (court runner) |

Doctor stages — `scripts/doctor/` (run: `bash scripts/doctor.sh`).

| Path | Status |
|---|---|
| `scripts/doctor/00_preflight.sh` | ALIVE |
| `scripts/doctor/01_sys_audit.sh` | ALIVE |
| `scripts/doctor/02_storage_wal.sh` | ALIVE (DOD-3) |
| `scripts/doctor/03_types_variant.sh` | ALIVE (DOD-2) |
| `scripts/doctor/04_host_vfs.sh` | PARTIAL_ALIVE |
| `scripts/doctor/05_memory_guard.sh` | ALIVE |
| `scripts/doctor/06_browser_opfs.sh` | PARTIAL_ALIVE |
| `scripts/doctor/07_ocel_compress.sh` | ALIVE |
| `scripts/doctor/08_perf_sla.sh` | ALIVE |
| `scripts/doctor/09_concurrency.sh` | ALIVE |
| `scripts/doctor/09_court_ladder.sh` | ALIVE |
| `scripts/doctor/10_verify_economics.sh` | ALIVE |
| `scripts/doctor/11_revenue_packs.sh` | PROPOSED |
| `scripts/doctor/12_fleet_hardware.sh` | ALIVE |
| `scripts/doctor/dual_pass.sh` | ALIVE (just court-x2) |

## Design

One-liners in `docs/design/` — status PROPOSED unless a court above proves
the design (then ALIVE). See `docs/design/README.md` for the catalog.

Core specs: `sovereign-spec-v26.10.19.md` (PROPOSED, master spec),
`prd-ard-v2.md` (PROPOSED), `dmadv-charter.md` (ALIVE, campaign charter),
`master-traceability.md` (PROPOSED, traceability hub),
`requirements-traceability.md` (PROPOSED), `verification-plan.md` (PROPOSED),
`acceptance-criteria-courts.md` (ALIVE, court derivation).

Gates: `gate-v01-latency.md`, `gate-v02-crash-chaos.md`,
`gate-v03-memory-ceiling.md`, `gate-v04-opfs-concurrency.md`,
`gate-v05-hostvfs-transfer.md` (each ALIVE — court shipped in
`scripts/courts/`).

Court ladder / doctor: `court-ladder-11.md`, `court-ladder-spec.md` (ALIVE),
`doctor-verification-blueprint.md`, `doctor-implementation-map.md`,
`doctor-dual-pass-court.md` (ALIVE, `scripts/doctor/dual_pass.sh`),
`doctor-blocked-phases.md` (PARTIAL_ALIVE).

ABI / engine: `boundary-abi-contract.md`, `extension-abi-hub.md`,
`wasm-abi-surface.md`, `wasm-opfs-architecture.md` (PROPOSED),
`freeze-moat.md` (ALIVE, campaign thesis), `storage-format-v2.md` (ALIVE).

Subsystems: `variant-shredding-design.md` (ALIVE), `ocel2-compression.md`,
`rehydration-sql.md`, `sync-digest-design.md`, `vfs-journal-design.md`,
`host-call-vfs.md`, `host-range-vfs-contract.md`, `view-expansion-design.md`,
`compression-proof-protocol.md`, `panic-confinement-protocol.md`,
`receipt-sealing.md`, `ocel-sovereign-schema-validation.md`,
`smalt-interception-subsystem.md`, `ex4pm-conformance-subsystem.md`
(all PROPOSED).

Value / economics: `economic-model.md`, `soc-valuation-model.md`,
`revenue-packs.md`, `disruption-scorecard.md`, `persona-journey-map.md`,
`ingest-throughput-plan.md`, `latency-bench-plan.md`,
`binary-size-budget.md`, `extension-matrix-8020.md`,
`fleet-hardware-classes.md` (ALIVE, fleet A-E tests),
`blueprint-ci.md`, `chrome-burnin-court.md` (PARTIAL_ALIVE),
`warehouse-sim-testbed.md` (ALIVE, see Warehouse Sim below).

## Ontology & Gates

| Path | Status |
|---|---|
| `schema/blueprint.ttl` | ALIVE |
| `schema/requirements.ttl` | ALIVE |
| `schema/domain.ttl` | ALIVE |
| `schema/verify.ttl` + `verify-gates.rq` | ALIVE (gate queries) |
| `schema/changelog.ttl` | ALIVE |
| `schema/reconciliation-vocab.ttl` | ALIVE |
| `gates/blueprint/blueprint-shapes.ttl` | ALIVE (SHACL) |
| `gates/blueprint/prd2-shapes.ttl` | ALIVE (SHACL) |
| `gates/blueprint/requirements-shapes.ttl` | ALIVE (SHACL) |
| `gates/dry-run-publish/` | ALIVE (publish gate) |
| `KNOWN_DIVERGENCES.md` | ALIVE (SDK-parity register) |
| `make gates` / `make verify` | ALIVE (entry commands) |

## Warehouse Sim

TypeScript testbed in `warehouse-sim/` (run: see `warehouse-sim/README.md`).

| Path | Status |
|---|---|
| `warehouse-sim/lib/sim_engine.ts` (+ `.test.ts`) | ALIVE (sim core) |
| `warehouse-sim/lib/ledger.ts`, `statement_state.ts` | ALIVE |
| `warehouse-sim/lib/route_helpers.ts` | ALIVE |
| `warehouse-sim/tests/` (9 test files) | ALIVE (chaos, journeys, etc.) |
| `warehouse-sim/courts/receipt_chain.sh` | ALIVE |
| `warehouse-sim/courts/determinism.sh` | ALIVE |
| `warehouse-sim/courts/receipt.json` | ALIVE (court output) |
| `warehouse-sim/economics/` (court.ts, scorecard.ts, value_ledger.ts,
val003_snowflake.ts, rate_cards.ts, run_court.sh, run_scorecard.sh) | ALIVE |
| `warehouse-sim/schema/warehouse_sim.ttl` | ALIVE |
| `warehouse-sim/app/api/` | PARTIAL_ALIVE (Next.js surface) |
| `warehouse-sim/client/`, `templates/` | PARTIAL_ALIVE |

## Window (docs/sjira/v2.0.0-dev)

| Path | Status |
|---|---|
| `docs/sjira/v2.0.0-dev/DEFINITION-OF-DONE.md` | ALIVE (DoD hub) |
| `docs/sjira/v2.0.0-dev/PRD-WO.md` | ALIVE |
| `docs/sjira/v2.0.0-dev/BLUEPRINT-WO.md` | ALIVE |
| `docs/sjira/v2.0.0-dev/UPGRADE-WO.md` | ALIVE |
| `docs/sjira/v2.0.0-dev/_LANES.md` | ALIVE (lane partition) |
| `docs/sjira/v2.0.0-dev/CAMPAIGN-RECEIPT.md` | ALIVE |
| `docs/sjira/v2.0.0-dev/COMMIT-PLAN.md` | ALIVE |
| `docs/sjira/v2.0.0-dev/receipts/` (fleet001-004 + metas) | ALIVE |

## Value Audit

| Path | Status |
|---|---|
| `scripts/value_audit.sh` | ALIVE (entry; `just value-audit`) |
| `warehouse-sim/economics/render_value_audit.ts` | ALIVE (CFO render) |
| `crates/frozen-duckdb/tests/value_audit_{amplitude,celonis,datadog}.rs` |
ALIVE (three competitor courts) |
| `justfile` recipe `value-audit` | ALIVE |

## Legal & Economics

| Path | Status |
|---|---|
| `LICENSE` | ALIVE (base license) |
| `LICENSE-ENTERPRISE-DRAFT.md` | PROPOSED (draft) |
| `docs/design/economic-model.md` | PROPOSED |
| `docs/design/soc-valuation-model.md` | PROPOSED |
| `docs/design/revenue-packs.md` | PROPOSED |
| `docs/design/disruption-scorecard.md` | PROPOSED |
| `scripts/economics/verify_economics.sh` | ALIVE (economics court) |
