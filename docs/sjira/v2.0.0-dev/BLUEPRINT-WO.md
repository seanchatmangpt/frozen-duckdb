# BLUEPRINT-WO — frozen-duckdb — v2.0.0-dev

| field | value |
|---|---|
| repo | `/Users/sac/frozen-duckdb` (canonical checkout) |
| work order | v2.0.0-dev blueprint: docs, gates, and graph design surface for the 2.0 upgrade campaign |
| lane | BP11 (writer), per `_LANES.md` blueprint lane partition |
| standing | **PROPOSED** — IN_PROGRESS vocabulary does not apply until blueprint lanes execute |
| date opened | 2026-10-19 |

## Scope

Blueprint lanes BP1–BP16 write docs, gates, and graph artifacts only. No lane in this
partition touches `crates/`, `scripts/`, or any code surface, and no lane may assert a
code ALIVE claim — ALIVE requires executed verification on the real subject, which is
out of scope for design artifacts. One canonical checkout, no worktrees. Git
ownership per `docs/sjira/v2.0.0-dev/_LANES.md`: lanes never run git state-changing
commands; the coordinator owns all transitions and commits. This document is lane
BP11's write surface within `docs/sjira/v2.0.0-dev/`; no other file is modified by
this lane.

## Deliverables (file ownership per _LANES.md)

| deliverable | owner lane |
|---|---|
| `docs/design/architecture-2.0.md` | BP lane |
| `docs/design/migration-guide-2.0.md` | BP lane |
| `docs/design/builder-cache-v2.md` | BP lane |
| `docs/design/ffi-surface-2.0.md` | BP lane |
| `docs/design/config-surface-2.0.md` | BP lane |
| `docs/design/extension-strategy.md` | BP lane |
| `docs/design/verification-plan.md` | BP lane |
| `LICENSE-ENTERPRISE-DRAFT.md` | BP lane |
| `schema/blueprint.ttl` | BP lane |
| `docs/design/prd-ard-v2.md` | P lanes |
| `docs/design/requirements-traceability.md` | P lanes |
| `docs/design/boundary-abi-contract.md` | P lanes |
| `docs/design/vfs-journal-design.md` | P lanes |
| `docs/design/latency-bench-plan.md` | P lanes |
| `docs/design/binary-size-budget.md` | P lanes |
| `docs/design/compression-proof-protocol.md` | P lanes |
| `docs/design/panic-confinement-protocol.md` | P lanes |
| `docs/design/variant-shredding-design.md` | P lanes |
| `docs/design/view-expansion-design.md` | P lanes |
| `docs/design/persona-journey-map.md` | P lanes |
| `docs/design/sync-digest-design.md` | P lanes |
| `docs/design/ingest-throughput-plan.md` | P lanes |
| `schema/requirements.ttl` | P lanes |
| `gates/blueprint/requirements-shapes.ttl` | P lanes |
| `docs/design/gate-v01-latency.md` | R14 |
| `docs/design/gate-v02-crash-chaos.md` | R14 |
| `docs/design/gate-v03-2gb-probe.md` | R14 |
| `docs/design/gate-v04-opfs-concurrency.md` | R14 |
| `docs/design/gate-v05-hostvfs-transfer.md` | R14 |
| `scripts/courts/gate_v01.sh` .. `gate_v05.sh` | R14 |
| `schema/requirements.ttl` prd2 individuals | R14 |
| `docs/design/prd-ard-v2.md` rev2 section | R14 |
| `docs/design/requirements-traceability.md` rev2 map | R14 |
| `docs/design/host-range-vfs-contract.md` | R14 |
| `docs/design/freeze-moat.md` | R14 |
| `docs/design/receipt-sealing.md` | R14 |

X-lane sovereign spec wave addendum (2026-10-19):

| deliverable | owner lane |
|---|---|
| `docs/design/sovereign-spec-v26.10.19.md` | X lanes |
| `docs/design/master-traceability.md` | X lanes |
| `tests/gate_v01_latency.rs` court (frozen-duckdb crate) | X lanes |
| `scripts/crash_chaos_native.sh` court | X lanes |
| `docs/design/ocel-sovereign-schema-validation.md` | X lanes |
| `tests/gate_ingest_throughput.rs` (frozen-duckdb crate) | X lanes |
| `tests/oom_graceful_abort.rs` (frozen-duckdb crate) | X lanes |
| `docs/design/smalt-interception-subsystem.md` | X lanes |
| `docs/design/ex4pm-conformance-subsystem.md` | X lanes |
| `docs/design/court-ladder-11.md` | X lanes |
| `docs/design/wasm-abi-surface.md` | X lanes |
| `LICENSE-ENTERPRISE-DRAFT.md` rev2 | X lanes |

Exact lane-to-file assignment is resolved by `_LANES.md`; BP11 owns only this
blueprint work order. Shared seams: `schema/blueprint.ttl` is the single graph
surface — consumers project from it, never hand-edit projections.

## Falsifier

The work order fails if any of:

1. Any blueprint claim is promoted PROPOSED→ALIVE without its
   `docs/design/verification-plan.md` court having run real against the actual
   subject — command, exit code, actual output. A promotion without a run receipt is
   SUSPECT.
2. Any blueprint deliverable asserts code behavior (ALIVE) for a surface no lane has
   executed. Blueprint prose may state PROPOSED or PARTIAL_ALIVE only with a cited
   executed receipt.
3. No-sleep doctrine applies: any verification plan that sequences coordination via
   fixed `sleep(N)` instead of bounded polls on real observables fails review.
   No-mock doctrine applies: any court built on `mockall`/stub collaborators faking an
   owned, locally-runnable DuckDB surface fails review — real subprocess/real engine,
   Chicago-style.
4. `LICENSE-ENTERPRISE-DRAFT.md` language describing a "self-executing tripwire"
   remains **UNKNOWN-PENDING-COUNSEL**: any blueprint text asserting that mechanism
   works, is enforceable, or is ALIVE fails. The mechanism is prose-only until counsel
   rules.

## Standing vocabulary

- **PROPOSED** (current, per-deliverable and whole order): designed on paper, no
  executed court.
- **PARTIAL_ALIVE**: some deliverables landed AND at least one verification-plan
  court has run real with receipted output; remaining deliverables still PROPOSED.
- **ALIVE** requires: every deliverable on disk, every claim's verification-plan
  court executed with real command + exit + output, falsifiers 3 and 4 clean
  (no sleep/mock coordination, no tripwire ALIVE claim), zero unresolved auditor
  findings.

## Execution boundary

Nothing in this document asserts execution. Commands, exits, and digests belong in
lane receipts under `receipts/`, not here. This work order records scope,
deliverable ownership, falsifier, and standing vocabulary only. A blueprint that has
merely been written is PROPOSED, never ALIVE.
