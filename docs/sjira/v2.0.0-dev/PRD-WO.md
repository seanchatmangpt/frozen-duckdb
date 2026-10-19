# PRD-WO — frozen-duckdb — v2.0.0-dev

| field | value |
|---|---|
| repo | `/Users/sac/frozen-duckdb` (canonical checkout) |
| work order | v2.0.0-dev PRD: requirements graph, PRD/ARD docs, and requirements-shapes gate design surface for the 2.0 upgrade campaign |
| lane | P14 (writer), PRD lane partition P1–P18 |
| standing | **PROPOSED** — no PRD deliverable has an executed court |
| date opened | 2026-10-19 |

## Scope

PRD lanes P1–P18 write docs, graph, and gates artifacts only. No lane in this
partition touches `crates/`, `scripts/`, `.github/`, or any code surface, and no
lane may assert a code ALIVE claim — ALIVE requires executed verification on the
real subject, which is out of scope for requirements-design artifacts. One
canonical checkout, no worktrees. Git ownership per `docs/sjira/v2.0.0-dev/_LANES.md`
as amended for the PRD partition: lanes never run git state-changing commands; the
coordinator owns all transitions and commits. This document is lane P14's write
surface within `docs/sjira/v2.0.0-dev/`; no other file is modified by this lane.

## Deliverables (file ownership per _LANES.md, PRD partition)

| deliverable | owner lane |
|---|---|
| `docs/design/prd-ard-v2.md` | P1 |
| `docs/design/requirements-traceability.md` | P2 |
| `docs/design/boundary-abi-contract.md` | P3 |
| `docs/design/vfs-journal-design.md` | P4 |
| `docs/design/latency-bench-plan.md` | P5 |
| `docs/design/binary-size-budget.md` | P6 |
| `docs/design/compression-proof-protocol.md` | P7 |
| `docs/design/panic-confinement-protocol.md` | P8 |
| `docs/design/variant-shredding-design.md` | P9 |
| `docs/design/view-expansion-design.md` | P10 |
| `docs/design/persona-journey-map.md` | P11 |
| `docs/design/sync-digest-design.md` | P12 |
| `docs/design/ingest-throughput-plan.md` | P13 |
| `docs/sjira/v2.0.0-dev/PRD-WO.md` | P14 (this document) |
| `schema/requirements.ttl` | P15 |
| `gates/blueprint/requirements-shapes.ttl` | P16 |
| P17 | reserve (audit/verifier intake for requirements courts) |
| P18 | reserve (standings-flip gate-runner intake; runs `scripts/check_sjira_standing.sh` before any flip) |

Exact lane-to-file assignment is resolved by `_LANES.md`; P14 owns only this PRD
work order. Shared seams: `schema/requirements.ttl` is the single requirements
graph surface — `gates/blueprint/requirements-shapes.ttl` constrains it, and
docs lanes cite it rather than duplicating it; projections are generated, never
hand-edited.

## Falsifier

The work order fails if any of:

1. **Promotion gate.** Any FR or NFR is promoted PROPOSED→ALIVE without its mapped
   court having run real against the actual subject and produced both a real
   BEFORE-green receipt and an AFTER-flip receipt (command, exit code, actual
   output). A promotion citing only the AFTER run — or a BEFORE receipt that is
   planned/inspected rather than executed — is SUSPECT.
2. **Zero code ALIVE claims.** Any PRD deliverable asserts code behavior (ALIVE)
   for a surface no lane has executed. PRD prose may state PROPOSED or
   PARTIAL_ALIVE only with a cited executed receipt.
3. **No-sleep doctrine.** Any verification plan or court that sequences
   coordination via fixed `sleep(N)` instead of bounded polls on real observables
   fails review.
4. **No-mock doctrine.** Any court built on `mockall`/stub collaborators faking an
   owned, locally-runnable DuckDB surface fails review — real subprocess, real
   engine, Chicago-style. A hand-written real interface implementation is not a
   mock.
5. **NFR-04 dependency edge.** NFR-04 is FR-01-dependent: no NFR-04 promotion may
   precede the FR-01 court's AFTER-flip receipt, and an NFR-04 ALIVE claim with a
   missing or stale FR-01 receipt fails. The edge is recorded here and must be
   mirrored in `schema/requirements.ttl` (`requirements:dependsOn` FR-01) and in
   `docs/design/requirements-traceability.md`.

## Standing vocabulary

- **PROPOSED** (current, per-deliverable and whole order): specified on paper, no
  executed court.
- **PARTIAL_ALIVE**: some PRD deliverables landed AND at least one mapped
  requirements court has run real with receipted BEFORE-green + AFTER-flip output;
  remaining deliverables still PROPOSED.
- **ALIVE** requires: every deliverable on disk, every FR/NFR's mapped court
  executed with real command + exit + output on both sides of the flip (BEFORE
  green, AFTER flip), falsifiers 3–5 clean (no sleep, no mock, NFR-04 after
  FR-01), zero unresolved auditor findings.

## Execution boundary

Nothing in this document asserts execution. Commands, exits, and digests belong in
lane receipts under `receipts/`, not here. This work order records scope,
deliverable ownership, falsifier, and standing vocabulary only. A PRD that has
merely been written is PROPOSED, never ALIVE.
