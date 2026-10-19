# README — Blueprint Design Index (BP16)

Index of the frozen-duckdb 2.0 blueprint layer (v2.0.0-dev campaign).
Status vocabulary per repo law: all items PROPOSED — no court has flipped
for any item yet (no lane receipt claims otherwise as of 2026-10-19).

## Design docs

| file | one-line | status |
|---|---|---|
| wasm-opfs-architecture.md | DuckDB-as-WASM Worker, persisting via OPFS sync handles | PROPOSED |
| chrome-burnin-court.md | Burn-in court over real chrome-headless-shell via CDP | PROPOSED |
| ocel2-compression.md | OCEL 2.0 segment-vector compression over DuckDB | PROPOSED |
| economic-model.md | Economics of the in-browser distribution (DESIGN, not measured) | PROPOSED |
| dmadv-charter.md | DMADV charter for the v2.0.0-dev campaign (lane BP6) | PROPOSED |
| storage-format-v2.md | Storage format v2 design notes, OPFS persistence layer | PROPOSED |
| memory-bounds.md | <=256MB in-browser memory cap design, CDP RSS-polling court | PROPOSED |
| verification-plan.md | Court battery (BP9); every court flips before ALIVE claims | PROPOSED |
| blueprint-ci.md | Nightly browser burn-in workflow + blueprint CI extension design | PROPOSED |
| soc-valuation-model.md | SoC valuation model for the blueprint layer | PROPOSED |
| rehydration-sql.md | Lossless OCEL 2.0 segment-vector rehydration SQL design | PROPOSED |

## Root-level blueprint artifacts

| file | one-line | status |
| schema/blueprint.ttl | Blueprint ontology (lane BP7); all layers bp:status PROPOSED | PROPOSED |
| gates/blueprint/blueprint-shapes.ttl | SHACL shapes for the blueprint graph (BP13) | PROPOSED |
| LICENSE-ENTERPRISE-DRAFT.md | Enterprise license draft; needs counsel review | PROPOSED |

## See Also

- docs/sjira/v2.0.0-dev/CAMPAIGN-RECEIPT.md

## Revision 2 artifacts (R-wave, 2026-10-19)

Statuses read from disk this session (file existence + section grep),
not from any lane's claim. Vocabulary: LANDED = file/section on disk;
MISSING = no file and no section found.

| artifact | one-line | status |
|---|---|---|
| prd-ard-v2.md | "Revision 2 - Dual-Target Release & Host-VFS" section, line 198+ | LANDED |
| requirements-traceability.md | rev2 PRD-FR/SLA/GATE-V alias + mapping tables (line 70+) | LANDED |
| gate-v01-latency.md | latency court spec, rev2 three-query workload | LANDED |
| gate-v02-crash-chaos.md | crash/chaos court spec | LANDED |
| gate-v03 spec | docs/design/gate-v03-memory-ceiling.md — PARTIAL_ALIVE (real native OOM witness) | LANDED |
| gate-v04-opfs-concurrency.md | OPFS concurrency court spec | LANDED |
| gate-v05-hostvfs-transfer.md | host-VFS wire-transfer court spec | LANDED |
| host-range-vfs-contract | host-range:// VFS contract; prd-ard-v2 rev2 | LANDED (section) |
| freeze-moat | docs/design/freeze-moat.md — design doc | LANDED |
| receipt-sealing | docs/design/receipt-sealing.md — design doc | LANDED |
| scripts/courts/ runner | gate_v01.sh .. gate_v05.sh runners present (6 lines each) | LANDED |
| schema/requirements.ttl prd2 individuals | prd2: skos:Concept individuals, prd: scheme | LANDED |
| prd2 SHACL shapes | gates/blueprint/prd2-shapes.ttl — coverage record + flip receipt | LANDED |

## Sovereign spec wave (X-lanes, 2026-10-19)

| artifact | one-line | status |
|---|---|---|
| sovereign-spec-v26.10.19.md | sovereign spec wave landing doc (405 lines) | LANDED |
| master-traceability.md | master traceability rollup | IN-FLIGHT |
| gate-v01-latency.rs court | tests/gate_v01_latency.rs, frozen-duckdb crate (128 ln) | LANDED |
| crash_chaos_native.sh court | native crash/chaos court script | IN-FLIGHT |
| ocel-sovereign-schema-validation.md | OCEL sovereign schema validation doc | IN-FLIGHT |
| gate_ingest_throughput.rs | tests/gate_ingest_throughput.rs, fd crate (159 ln) | LANDED |
| oom_graceful_abort.rs | crates/frozen-duckdb/tests/oom_graceful_abort.rs (96 lines) | LANDED |
| smalt-interception-subsystem.md | smalt interception subsystem design (100 lines) | LANDED |
| ex4pm-conformance-subsystem.md | ex4pm conformance subsystem design (133 lines) | LANDED |
| court-ladder-11.md | 11-rung court ladder doc | IN-FLIGHT |
| wasm-abi-surface.md | WASM ABI surface doc | IN-FLIGHT |
| LICENSE-ENTERPRISE-DRAFT rev2 | LICENSE-ENTERPRISE-DRAFT.md rev2 (153 lines) | LANDED |
