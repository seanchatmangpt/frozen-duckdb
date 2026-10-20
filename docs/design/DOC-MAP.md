# DOC-MAP — Canonical Home per Topic

Lane ERRC-E3 (Eliminate/Reduce), 2026-10-19. One canonical owner per
topic; overlapping docs carry pointers, not restatements. Status
vocabulary per repo law (PROPOSED/PARTIAL_ALIVE/ALIVE/BLOCKED/
UNSUPPORTED); this map assigns ownership, not standing.

## Topic → canonical doc

| topic | canonical home | pointers in (reduced to pointer) |
|---|---|---|
| 13-stage court ladder (this repo's stage specs, laws L1-L4, COURT-x2) | `court-ladder-spec.md` | — |
| smalt 11-stage ladder mapping to this repo | `court-ladder-11.md` | — (DISAMBIGUATION: a *different* ladder — smalt sdk/sleep-gate stages mapped onto fd rungs. Same stage count, different content; not a duplicate of `court-ladder-spec.md`) |
| doctor verification battery (TEST-001..025, phases 0-9, boundary law) | `doctor-verification-blueprint.md` | — |
| doctor battery live landing state (what runs green today) | `doctor-implementation-map.md` | — |
| blocked doctor phases + FR-01 unblock landing sequence | `doctor-blocked-phases.md` | — (complementary: per-test unblock preconditions + landing order; blueprint phases 3/5/6 keep only one-line definitional tables) |
| GATE-V-01 latency court | `gate-v01-latency.md` (workload: `latency-bench-plan.md`) | — |
| GATE-V-02 crash-chaos court (AC-2 alias of record) | `gate-v02-crash-chaos.md` | `acceptance-criteria-courts.md` AC-2 |
| GATE-V-03 memory-ceiling court (AC-3 alias, rev2 numbers supersede AC-3's example probe) | `gate-v03-memory-ceiling.md` | `acceptance-criteria-courts.md` AC-3 |
| GATE-V-04 OPFS concurrency / lock race | `gate-v04-opfs-concurrency.md` (browser leg: `chrome-burnin-court.md` Court 3) | — |
| GATE-V-05 host-VFS transfer court | `gate-v05-hostvfs-transfer.md` (contract: `host-range-vfs-contract.md`) | `doctor-blocked-phases.md` TEST-011/012 rows (definitional pointers, not restatements) |
| Chrome burn-in courts 1-4 (crash persistence, memory budget, lock race, version gate) | `chrome-burnin-court.md` | `verification-plan.md` V-W2 |
| BP9 court battery inventory (V-W1..V-G2) | `verification-plan.md` | — |
| FR/NFR/AC traceability, Revision 1 + rev1→rev2 alias map | `requirements-traceability.md` | — (master-traceability is the rev2-per-requirement rollup; both kept — complementary, not duplicate) |
| PRD-FR-nn / SLA-n per-requirement rollup with repo loci + receipts | `master-traceability.md` | — |
| PRD/ARD Revision 2 source requirements | `prd-ard-v2.md` | — |
| Sovereign spec + §5 ladder + §6 economics policy | `sovereign-spec-v26.10.19.md` | — |
| OCEL 2.0 compression method + V-S2/AC-4 ratio court | `ocel2-compression.md` | `acceptance-criteria-courts.md` AC-4 (already delegated by pointer) |
| Fleet hardware classes A-E (doctor stage 12) | `fleet-hardware-classes.md` | — (no overlap with revenue-packs: hardware courts vs product packs; TEST-FLEET-nn vs TEST-REV-nn) |
| Revenue packs 1-4 (TEST-REV-nn, doctor stage 11) | `revenue-packs.md` | — |
| Warehouse-sim testbed design (graph→render→counterfactual ledger) | `warehouse-sim-testbed.md` | — (economics *thesis* lives in `economic-model.md`; testbed is the fixture/court design — complementary) |
| Economics thesis (subsidized silicon, unit economics) | `economic-model.md` | — |

## Dedupe applied this window (lane ERRC-E3)

1. `acceptance-criteria-courts.md` AC-2: witness command, artifact,
   and non-vacuity mutation restated `gate-v02-crash-chaos.md`
   verbatim-class → reduced to pointer + AC-layer standing-flip rule.
   Removes the "both owning docs must stay in sync" burden noted in
   `requirements-traceability.md` §GATE-V-01..05.
2. `acceptance-criteria-courts.md` AC-3: probe/config/mutation
   superseded by gate-v03 rev2 (the count(*) probe is witnessed
   vacuous there) → pointer + correction note + standing-flip rule.
3. `verification-plan.md` V-W2: mechanism restated
   `chrome-burnin-court.md` Court 1 → witness command kept, mechanism
   reduced to pointer; shim inherits Court 1's falsifier.

## Deliberately NOT deduped (complementary, not duplication)

- `court-ladder-11.md` vs `court-ladder-spec.md` — two different
  ladders that share a stage count; disambiguated above.
- `doctor-blocked-phases.md` vs `doctor-verification-blueprint.md`
  phases 3/5/6 — register+landing-sequence vs one-line battery rows.
- `requirements-traceability.md` vs `master-traceability.md` —
  rev1+alias map vs rev2 per-requirement rollup.
- `fleet-hardware-classes.md` vs `revenue-packs.md` — disjoint
  court families.

## See Also

`README.md` (landing-status index) · `doctor-implementation-map.md`
(live TEST status) · `scripts/courts/README.md`
