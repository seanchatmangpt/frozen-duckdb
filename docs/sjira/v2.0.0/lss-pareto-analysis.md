# LSS Pareto Analysis — v2.0.0 campaign defects (lane LSS1, DMAIC Analyze)

| field | value |
|---|---|
| repo | `/Users/sac/frozen-duckdb` |
| window | `docs/sjira/v2.0.0/` |
| date | 2026-10-19 |
| method | Pareto over named findings; every count cites a named finding |
| sources | `CAMPAIGN-RECEIPT.md` (residues + incident record), |
| | `ERRC-LEDGER.md`, `KNOWN_DIVERGENCES.md` (14 rows), |
| | coordinator dispatch (lane disclosures not otherwise |
| | materialized in window docs, marked `DISPATCH-NAMED`) |
| standing | ANALYSIS (no standing claim; Analyze-phase artifact) |

## 1. Finding inventory (42 findings, each traceable)

### Family A — lane-coordination (shared-tree write/stale-state) — 11

| # | finding | count | evidence |
|---|---|---|---|
| A1 | Cache poison: v1.5.5 bytes into v2.0.0 slot | 1 | CAMPAIGN-RECEIPT incident record |
| A2 | Rogue-writer clobbers of F3 file (4x) | 4 | CAMPAIGN-RECEIPT rogue-writer incident class |
| A3 | Rogue-writer clobber of G1's ontology (1x) | 1 | CAMPAIGN-RECEIPT "1x on another" |
| A4 | Wrong-tree staging | 1 | DISPATCH-NAMED (coordinator lane disclosure) |
| A5 | Links collision | 1 | DISPATCH-NAMED |
| A6 | fmt hunk (fmt --check drift mid-wave) | 1 | DISPATCH-NAMED |
| A7 | Stale `09_court_ladder.sh` not in PHASES | 1 | CAMPAIGN-RECEIPT H4 "Stale remnant" |
| A8 | Post-13:22 receipts VOID for engine identity | 1 | CAMPAIGN-RECEIPT cache-poison clause |
| A9 | Tracked build outputs `target-gap8/` fingerprints | 1 | session `git status` (LSS1 observe) |
| A10 | Wrong-tree staging: receipts modified in place | 1 | `git status`: `M val001.json/meta` |
| A11 | Stray nested window dirs inside `receipts/` | 1 | `ls receipts/` this pass |
|    | (`v2.0.0`, `v26.10.8`, `v26.9.21`, `v26.9.22`) | | |

### Family B — lane-authored code/fixture defects (lanes' own artifacts) — 9

| # | finding | count | evidence |
|---|---|---|---|
| B1-B3 | E2 fixture bugs x3; false X4 "WAL-replay | 3 | CAMPAIGN-RECEIPT CORRECTION RECORD |
|    | atomicity defect", withdrawn | | E2 receipt itemization |
| B4 | E0382 moved-value builder test, lib.rs:1240 | 1 | ERRC-LEDGER Raised row 5, witnessed 2x |
| B5 | Syntax error test_fleet_class_b_industrial.rs:138 | 1 | CAMPAIGN-RECEIPT T6 first snapshot |
| B6 | Compile break `test_fleet_class_d_marine.rs:213` | 1 | CAMPAIGN-RECEIPT T6 first snapshot |
| B7 | T6 filter bug: name-filter matched 0 fns | 1 | CAMPAIGN-RECEIPT T6 CORRECTION |
| B8 | expect-panic lint/compile defect | 1 | DISPATCH-NAMED |
| B9 | unreachable arm (match arm flagged) | 1 | DISPATCH-NAMED |

### Family C — version-drift (2.0.0 pin surface divergence) — 7

| # | finding | count | evidence |
|---|---|---|---|
| C1 | KD-01: 21 mangled `__Z*` exports, C-ABI contract breached | 1 | KNOWN_DIVERGENCES KD-01 |
| C2 | KD-02: flock extension 404 at pin b6013f7 | 2 tests | KNOWN_DIVERGENCES KD-02 |
| C3 | KD-07: REV-001 wall 4.23-6.05s vs <2.5s | 1 | KD-07; value_audit rev001.json |
| C4 | KD-09: cgroup leg needs Linux+root; darwin rc=2 | 1 | KD-09; T6 final EXIT=3 aggregate |
| C5 | KD-10: Class E actor court lives in smalt (other repo) | 1 | KD-10 |
| C6 | KD-11: browser/OPFS courts FR-01/FR-02-blocked | 1 | KD-11; stages 04/06 exit-3 |
| C7 | Stale `v2.1.0-dev89402` self-report in cache slot | 1 | CAMPAIGN-RECEIPT open residues |

(KD-05, KD-08 counted under Family E / F below; KD-03/04/14 CLOSED rows
are counted in Family D where they name rendered-surface defects;
KD-06 host-VFS is an unimplemented-by-design surface, not a wave
defect — excluded, not double-counted.)

### Family D — template-projection (ggen render) — 7

| # | finding | count | evidence |
|---|---|---|---|
| D1 | KD-03: poll routes static QUEUED, state dead | 1 | KNOWN_DIVERGENCES KD-03 CLOSED |
| D2 | KD-04 routes exported helpers, Next index-signature violation | 1 | KD-04 CLOSED |
| D3 | FM-GEN-008 missing `r.endpointLabel`, rc=1 | 1 | CAMPAIGN-RECEIPT G7, ggen verbatim |
| D4 | Render TDZ errors (2 distinct sites) | 2 | DISPATCH-NAMED |
| D5 | slice fail-open | 1 | DISPATCH-NAMED |
| D6 | serve-on-demote | 1 | DISPATCH-NAMED |

### Family E — environment/substrate — 5

| # | finding | count | evidence |
|---|---|---|---|
| E1 | KD-13 cargo registry corruption, 4 lanes hit | 4 | KD-13; V3/T5/R3/T1 receipts |
| E2 | KD-05 wasm: broken brew emscripten, no .wasm | 1 | KD-05; build_wasm_duckdb.sh exit 2 |

### Family F — API-discovery (2.0.0 API surface) — 3

| # | finding | count | evidence |
|---|---|---|---|
| F1 | E0277 AppenderParams x6 (rev_compliance:81) | 6 errors | CAMPAIGN-RECEIPT R5 stage run |
| F2 | E0277 AppenderParams tuple (rev_edge:131) | 1 error | CAMPAIGN-RECEIPT R5 stage run |
| F3 | KD-08 allocator-stats C API absent on pin | 1 | KD-08 |

Count note: F1/F2 are 7 compile errors across 2 finding-sites; counted
as 2 finding-sites (7 error instances) in the table, 2 in the pareto
curve to avoid error-count inflation.

## 2. Pareto curve (finding-sites, N=42)

Normalization: TDZ counted as 2 sites; F1/F2 error instances (7) kept
out of the curve to avoid inflation (2 sites counted). Curve N:
A11 + B9 + C7 + D7 + E5 + F3 = 42.

| rank | family | sites | % of 42 | cumulative % |
|---|---|---|---|---|
| 1 | A lane-coordination | 11 | 26.2% | 26.2% |
| 2 | B lane-authored code/fixture | 9 | 21.4% | 47.6% |
| 3 | C version-drift | 7 | 16.7% | 64.3% |
| 4 | D template-projection | 7 | 16.7% | 81.0% |
| 5 | E environment | 5 | 11.9% | 92.9% |
| 6 | F API-discovery | 3 | 7.1% | 100% |

## 3. Vital few

Strict 80/20 does not hold at family granularity (6 families is too
coarse for a clean 80/20 cut). The honest reading of the curve:

- Top 2 families (A+B = 20/42 = 47.6%) are lane-sourced defects —
  coordination plus lane-authored artifacts — the single biggest
  channel; neither the engine nor upstream is the top source.
- **Top 4 of 6 families = 34/42 = 81.0%** — the 80% line falls at
  family 4 (template-projection joins the vital few).
- The dispatch expectation (~20% of families -> ~80%) is met at
  class level: the concurrent-writer class inside A (A1-A6 = 9 of
  42 = 21.4% of all findings from one cause) is the one class that
  is individually ~20% of the whole. That class is the true vital
  few target.

## 4. Targeted system fixes per family

| family | share | targeted system fix | status of fix |
|---|---|---|---|
| A lane-coordination | 26.2% | Coordinator-owned git + per-file single-writer leases, |
| | | marker/atomic/class laws (KD-14 atomic temp+rename landed), |
| | | final-integration reconciliation pass before any commit | partially landed |
| | | (REQUIRED clause now in CAMPAIGN-RECEIPT), file mtime+sha | |
| | | double-read before receipt cite | |
| B lane-authored code/fixture | 21.4% | Census-vs-compile-abort rule (stage 11/12 | landed |
|    | refuse rc=101 as pass); fixture courts carry | | partial |
|    | positive control so a broken fixture fails loud | | |
| C version-drift | 16.7% | KD register rows with close conditions (landed, 14 rows), | landed |
| | | pin a 2.0.0 API-surface probe in stage 00/01 (KD-01 | partial |
| | | already propagated into doctor), upstream pin bump watch | |
| D template-projection | 16.7% | Graph-level template contracts: every `.tera` variable | partial |
| | | declared in the graph (FM-GEN-008 class), double-sync | |
| | | byte-identical gate (landed for warehouse-sim), route | |
| | | index-signature + statefulness asserted in the graph not | |
| | | the rendered file | |
| E environment | 11.9% | Substrate-owner audit of the cleanup actor (KD-13 close | open |
| | | condition), GAP16 default-CARGO_HOME build gate | |
| F API-discovery | 7.1% | One pinned API-surface compatibility note per pin bump; | open |
| | | compile-abort census already prevents silent pass-through | partial |

## 5. Falsifier

- A cited finding above that does not resolve to its named evidence
  (e.g. KD row absent, receipt section absent) invalidates the count.
- The pareto conclusion is refuted if a re-count from the same sources
  moves any family across the 80% line boundary (i.e. if A+B+C+D sum
  falls below 80% or rises above it with the same sources).
- If a next wave's defects do not concentrate in the same top classes
  (concurrent-writer class, version-drift pins), the family taxonomy
  is wrong, not the fleet.

## See Also

- `CAMPAIGN-RECEIPT.md` (incident record, residues)
- `ERRC-LEDGER.md` (Raised row 5 E0382)
- `KNOWN_DIVERGENCES.md` (14-row KD register)
- `~/.claude/rules/same-checkout-fanout.md` (compile-freeze SLA,
  census-vs-compile-abort, lock fd inheritance)
