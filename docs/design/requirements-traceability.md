# Requirements Traceability Matrix

Maps every requirement (FR-01..05, NFR-01..04) and acceptance criterion
(AC-1..4) to its owning design doc, verification court, current standing,
and blocking gap. Requirements sourced from the task brief; no
`docs/design/prd-ard-v2.md` exists on disk (checked 2026-10-19) — if it
lands later, this matrix must be re-derived against it.

Standing vocabulary: PROPOSED (specified, no court receipt on any subject),
PARTIAL_ALIVE (native reality already proves part of the requirement),
ALIVE (court receipt on exact subject), BLOCKED, UNSUPPORTED.

## Functional Requirements

| id | requirement | owning design doc | verification court | standing | blocking gap |
|---|---|---|---|---|---|
| FR-01 | WASM compile: `duckdb_wasm` artifact + version gate | `docs/design/wasm-opfs-architecture.md` ("Build matrix"; both toolchain paths UNVERIFIED in-source) | V-W1 `make wasm-build` + `node scripts/wasm/wasm_smoke.mjs` | PROPOSED | `scripts/wasm/` does not exist; no emcmake build ever attempted against the vendored headers |
| FR-02 | OPFS persistence: `Page.crash` + reload + WAL replay | `docs/design/wasm-opfs-architecture.md` ("OPFS block-I/O layer design") | V-W2 `scripts/wasm/opfs_crash_court.mjs`; BP2 Court 1 (mid-transaction crash, BLAKE3 checksums) | PROPOSED | no `opfs/` module and no court script on disk; flush-at-WAL-commit mapping unimplemented |
| FR-02-quota | typed `QuotaExceededError` → out-of-disk error on RPC surface | `docs/design/wasm-opfs-architecture.md` (quota falsifier 3) | BP2 Court 3 (OPFS lock race + QuotaManager) | PROPOSED | unimplemented; per-origin quota sizes in current Chrome UNVERIFIED |
| FR-03 | VARIANT type | `crates/frozen-duckdb/tests/variant_tests.rs` (de facto spec); referenced by `docs/design/dmadv-charter.md` | V-G2 native battery: `cargo test -p frozen-duckdb --test variant_tests` | PARTIAL_ALIVE | native: 5 passed / 0 failed, real run this session (exit 0); WASM-side VARIANT (same tests under wasm32) has no court — that portion PROPOSED |
| FR-04 | vectorized view expansion | `docs/design/rehydration-sql.md`; `docs/design/storage-format-v2.md` | V-S1 `cargo test -p frozen-duckdb --test ocel_rehydration_equivalence` | PROPOSED | test file does not exist; segment-vector rehydration not implemented (V-S1 is UNSUPPORTED today, flips only with the implementation) |
| FR-05 | memory cap (256MB wasm linear / ~192MB `memory_limit`) + graceful abort | `docs/design/wasm-opfs-architecture.md` (256MB cap section); `docs/design/memory-bounds.md` | V-W3 `scripts/wasm/memory_budget_court.mjs`; BP2 Court 2 (CDP `Performance.getMetrics` peak poll) | PARTIAL_ALIVE | native plumbing exists: generic `Config::with(key, value)` at `crates/frozen-duckdb/src/duckdb/config.rs:129` accepts `memory_limit` — but no test exercises it and no wasm memory court exists; worker OOM behavior is marked UNVERIFIED in the design doc itself |

## Non-Functional Requirements

| id | requirement | owning design doc | verification court | standing | blocking gap |
|---|---|---|---|---|---|
| NFR-01 | p95 < 15ms @ 100k rows | `docs/design/memory-bounds.md`; `docs/design/ocel2-compression.md` | none — GAP in `docs/design/verification-plan.md` (no latency court among V-W1..V-G2) | PROPOSED | no benchmark harness, no baseline measurement, no court defined |
| NFR-02 | ≤25MB stripped wasm artifact size | `docs/design/wasm-opfs-architecture.md` | none — V-W1 covers the artifact's existence and version, not its size | PROPOSED | no artifact exists to measure; size gate not specified in verification-plan.md |
| NFR-03 | ≥90% compression on OCEL corpus | `docs/design/ocel2-compression.md` | V-S2 `python3 scripts/compression_ratio_court.py --corpus test-fixtures/ocel-reference-corpus/ --floor 0.90` | PROPOSED | court never run; reference corpus fixture not confirmed on disk |
| NFR-04 | zero-panic trap boundary | `docs/design/wasm-opfs-architecture.md` (worker containment, marked UNVERIFIED in-source) | BP2 Courts 2/3 + `-sWASM_EH=1` flag (itself UNVERIFIED) | PROPOSED | no trap-boundary test exists; wasm_eh platform string unproven |

## Acceptance Criteria

| id | requirement | owning design doc | verification court | standing | blocking gap |
|---|---|---|---|---|---|
| AC-1 | dual-target parity (native vs WASM results identical) | none — GAP: no design doc owns parity | none — GAP: no parity court in verification-plan.md | PROPOSED | both owner and court missing; flagged for verification-plan owner |
| AC-2 | crash-chaos recovery | `docs/design/wasm-opfs-architecture.md` falsifier 2; `docs/design/chrome-burnin-court.md` Court 1 | V-W2 / BP2 Court 1 | PROPOSED | court scripts absent from `scripts/` |
| AC-3 | OOM graceful abort | `docs/design/wasm-opfs-architecture.md`; `docs/design/memory-bounds.md` | V-W3 / BP2 Court 2 (incl. 10x-batch vacuity falsifier) | PROPOSED | `memory_budget_court.mjs` does not exist; worker-OOM containment UNVERIFIED |
| AC-4 | 90% compression proof on 1M events | `docs/design/ocel2-compression.md` | V-S2 (floor 0.90) | PROPOSED | court never run; 1M-event corpus not built |

## Court coverage GAPs

`docs/design/verification-plan.md` defines V-W1..V-G2 (build, persistence,
memory budget, rehydration, compression, graph, native gates). Three
requirement classes have no assigned court:

- **NFR-01** p95 latency — needs a V-L1 latency court.
- **NFR-02** artifact size — needs a V-W1b size gate.
- **AC-1** dual-target parity — needs a V-P1 parity court.

These are flagged for the verification-plan owner to add before any of the
affected rows can move past PROPOSED.

## Receipt: FR-03 native VARIANT run (this session)

```
$ cargo test -p frozen-duckdb --test variant_tests
test variant_cast_literal_roundtrip ... ok
test variant_roundtrip_varchar_bind_and_read ... ok
test result: 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

5 passed, not 6: the file contains 6 `fn` items, one is a non-test helper;
5 are `#[test]`. The brief's "6/6" was inaccurate; 5/5 is the honest count.
PARTIAL_ALIVE is recorded on the native portion only; the wasm-side portion
of FR-03 stays PROPOSED until a wasm artifact exists to compile the same
tests against.

## Revision 2 mapping (PRD-FR / SLA / GATE-V)

Revision 2 renames the requirement space: functional requirements become
`PRD-FR-nn`, the latency/size non-functionals become `SLA-n`, and the
verification courts are renumbered `GATE-V-nn`. This section maps the
Revision-1 ids above onto the Revision-2 ids so both namings stay
resolvable. Standing vocabulary and honesty rules are unchanged: PROPOSED
means no court receipt on any subject; native-runnable flags carried in
Revision 1 are preserved here, not upgraded by the rename.

### Alias table (Revision 1 → Revision 2)

| Revision 1 | Revision 2 | Note |
|---|---|---|
| FR-01 (wasm compile) | PRD-FR-01 / PRD-FR-04 | splits: compile gate → 01; artifact surface → 04 |
| FR-02 (OPFS persistence) | PRD-FR-04 | equivalent; crash/reload/WAL scope carried intact |
| FR-03 (VARIANT) | PRD-FR-03 | equivalent; `variant_tests` 5/5 receipt only; wasm side PROPOSED |
| FR-04 (view expansion) | — (no PRD-FR) | absorbed into OCEL layer; V-S1 court coverage retained |
| FR-05 (memory cap) | PRD-FR-05 | equivalent; 256MB linear / ~192MB `memory_limit` carried |
| NFR-01 (p95 < 15ms) | SLA-1 | rename only; court gap unchanged |
| NFR-02 (≤25MB artifact) | SLA-2 | rev2 adds `<10MB` wire clause atop the 25MB bound |
| NFR-03 (≥90% compression) | retained, no renumber | under compression proof; no SLA row minted |
| NFR-04 (zero-panic boundary) | PRD-FR-05 + GATE-V-03 | via memory/probe gates, not standalone |

Revision-1 FR-01 detail: the old single wasm-compile FR split in rev2;
compile/version gate maps to PRD-FR-01, the OPFS-adjacent artifact
surface to PRD-FR-04. FR-04's rehydration equivalence remains
court-covered (V-S1) even though it loses its PRD row.

### GATE-V-01..05 → owning courts

| gate | owning court doc | note |
|---|---|---|
| GATE-V-01 | `docs/design/latency-bench-plan.md` | rev2 workload: filter, multi-agg, rolling avg |
| GATE-V-02 | `chrome-burnin-court.md` court 1; AC-2 row below | crash-chaos; dual ownership |
| GATE-V-03 | `acceptance-criteria-courts.md` AC-3 probe | rev2: 2GB probe replaces 256MB ceiling |
| GATE-V-04 | `chrome-burnin-court.md` BP2 court 3 | lock-race/quota; Web Locks addition noted |
| GATE-V-05 | NEW — `docs/design/host-call-vfs.md` | VFS falsifier → gate, ≤5MB transfer budget |

GATE-V-01 reconciliation: where the earlier workload list in the latency
plan differs from the rev2 three-query list (point filter, multi-level
aggregation, windowed rolling average), the rev2 list supersedes it.
GATE-V-03: adopt rev2 numbers (2GB probe) wherever the two conflict.
GATE-V-02: second owner is AC-2 in `docs/design/acceptance-criteria-
courts.md`; both owning docs must stay in sync on the crash-recovery
falsifier. GATE-V-04: the rev2 Web Locks addition is noted; existing
falsifiers unchanged. GATE-V-05: previously had no gate row at all.

### Status honesty

All rows above remain at their Revision-1 standing: PROPOSED unless a real
receipt exists (FR-03/PRD-FR-03 native portion stays PARTIAL_ALIVE on the
strength of the 5-passed `variant_tests` run receipted earlier in this
file). The rename itself flips nothing; gates flip only on court receipts
against exact subjects.

## See Also

`docs/design/verification-plan.md` · `docs/design/chrome-burnin-court.md` ·
`docs/design/wasm-opfs-architecture.md` · `docs/design/ocel2-compression.md`
