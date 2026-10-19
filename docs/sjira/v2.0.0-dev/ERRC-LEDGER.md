# ERRC-LEDGER — frozen-duckdb v2.0.0-dev finish sweep + campaign

| field | value |
|---|---|
| repo | `/Users/sac/frozen-duckdb` |
| lane | ERRC-C6 (Create) |
| date | 2026-10-19 |
| window | `docs/sjira/v2.0.0-dev/` |
| standing | **PARTIAL_ALIVE** (ledger of record; per-quadrant standings below) |

Basis: every path below was verified on disk this pass (2026-10-19).
Receipts cited as DISPATCH-CITED per `CAMPAIGN-RECEIPT.md` "Final
reconciliation" method; WITNESSED items name the exact on-disk artifact.
One in-flight defect is recorded honestly (builder test E0382, Raised row 5).

## Eliminated

| item | lane | evidence path |
|---|---|---|
| Vestigial bench/ticket scripts (6 files: | W12 | `git status` shows `D scripts/bench_gate.sh`, |
| `bench_gate.sh`, `gen_sjira_tickets.sh`, | | `D scripts/gen_sjira_tickets.sh`, |
| `run_bench_suite.sh`) | | `D scripts/gen_sjira_tickets_wave{3,4,5}.sh`, |
| | | `D scripts/run_bench_suite.sh` (working tree, |
| | | coordinator commits) |
| Dead bindgen snapshots | W1 | `git status`: `D` |
| | | `crates/frozen-duckdb-sys/src/` |
| | | `bindgen_bundled_version.rs` and `.../` |
| | | `bindgen_bundled_version_loadable.rs` |
| Broken ggen template (FM-GEN-008, | G2 | `warehouse-sim/templates/` repaired; incident |
| `snowflake_submit_route.ts.tera`, | | + loud-SKIP court run recorded in |
| missing `r.endpointLabel`) replaced | | `docs/sjira/v2.0.0-dev/CAMPAIGN-RECEIPT.md` |
| | | (G7 section, "G2's render self-healed") |
| KCura branding sweep | W12 | PARTIAL: removed from current-window surfaces; |
| | | residual echo strings remain at |
| | | `scripts/download_duckdb_binaries.sh:140-141` |
| | | and `scripts/verify_semantic_coverage.sh:38` |
| | | (grepped this pass) — flagged for coordinator |
| Stale size echoes | W12 | NOT LOCATED this pass: no on-disk artifact |
| | | naming the removed echoes was found; recorded |
| | | as DISPATCH-CITED-UNVERIFIED, do not cite |
| | | as WITNESSED |

## Reduced

| item | lane | evidence path |
|---|---|---|
| Prose standings -> pointer/register discipline | W11 | `KNOWN_DIVERGENCES.md` (repo root, 14 rows;
| (standing claims reduced to KD rows with | | each row carries surface/divergence/evidence/ |
| close conditions) | | close condition; KD-03/04/14 CLOSED) |
| Cache-write race surface reduced: concurrent | GAP9 | `KNOWN_DIVERGENCES.md` KD-14 (CLOSED: |
| `ensure_binary` writers -> atomic temp+rename | | temp+rename across cache-persist sites); |
| discipline | | `crates/frozen-duckdb-builder/src/lib.rs` |
| | | (atomic marker write, lines ~514-537) |
| Doctor surface reduced to one stage matrix | D2/E1 | `scripts/doctor.sh` (aggregates 13 stages |
| | | 00-12, fail-closed) + `scripts/doctor/` |

## Raised

| item | lane | evidence path |
|---|---|---|
| Doctor ladder 11 -> 13 stages (00-12: revenue | R5/T6 | `scripts/doctor/11_revenue_packs.sh`, |
| + fleet stages added and wired | | `scripts/doctor/12_fleet_hardware.sh`, |
| | | both wired into `scripts/doctor.sh`) |
| KD register surfaced into doctor output: | Z7/F7 | `scripts/doctor/checks/bin_audit.sh` feeds |
| stage 00/01 propagate the mangled-export | | KD-01 verdict; `scripts/doctor.sh` stages |
| KD-01 row (bin audit FAIL -> stage red) | | 00/01 red per `KNOWN_DIVERGENCES.md` KD-01 |
| `release_checklist.sh` realigned to the | W12 | `scripts/release_checklist.sh` (on disk, |
| 2.0.0-dev pin/flow | | grep-verified this pass) |
| Builder test battery grown to the atomic-cache | GAP9 | `KNOWN_DIVERGENCES.md` KD-14 cites 23/23 |
| suite (22 -> 23-24 tests incl. | | builder tests — DISPATCH-CITED; this lane's |
| `ENGINE_VERSION` round-trip) | | re-run at ledger-write time hit IN-FLIGHT |
| | | E0382 at `crates/frozen-duckdb-builder/src/` |
| | | `lib.rs:1240` (`borrow of moved value: |
| | | expected`, witnessed twice via |
| | | `CARGO_TARGET_DIR=/tmp/errc-t cargo test -p` |
| | | `frozen-duckdb-builder`); coordinator must |
| | | re-witness 23/23 post-fix before citing PASS |

## Created

| item | lane | evidence path |
|---|---|---|
| Value-audit battery + CFO renderer | V6 | `scripts/value_audit.sh` (orchestrator, |
| | | header names the CFO render), |
| | | `warehouse-sim/economics/render_value_audit.ts`, |
| | | receipts `docs/sjira/v2.0.0-dev/receipts/` |
| | | `val001-004.{json,meta}` |
| Revenue packs | R1-R5 | `docs/design/revenue-packs.md`, |
| | | `scripts/doctor/11_revenue_packs.sh`, |
| | | `crates/frozen-duckdb/tests/test_rev_*.rs` |
| | | (4 files), receipts `rev001-004.{json,meta}` |
| Fleet hardware classes A-E | T1-T6 | `docs/design/fleet-hardware-classes.md`, |
| | | `scripts/doctor/12_fleet_hardware.sh`, |
| | | `crates/frozen-duckdb/tests/test_fleet_class_*.rs`, |
| | | receipts `fleet001-005{,_proxy,_chaos}.json` |
| Warehouse-sim testbed | G2/G7 | `warehouse-sim/` (engine lib, courts with |
| | | `courts/receipt.json`, economics evidence |
| | | `economics/.evidence/run-*/receipt.json`), |
| | | `docs/design/warehouse-sim-testbed.md` |
| Admin observability surface | S-lane | `warehouse-sim/app/api/admin/` (on disk) |
| KD register (14 rows, 3 CLOSED) | W11 | `KNOWN_DIVERGENCES.md` |
| CAMPAIGN-INDEX | W11 | `CAMPAIGN-INDEX.md` (repo root) |
| Atomic cache writes + `ENGINE_VERSION` | GAP9/W3 | `crates/frozen-duckdb-builder/src/lib.rs` |
| marker | | (marker read/write fns, lines ~514-537) |
| Receipt files (verifier/court outputs) | V-lanes | `docs/sjira/v2.0.0-dev/receipts/` |
| | | (17 json + 17 meta + log) |

## Falsifiers (per quadrant — what would make the quadrant vacuous)

| quadrant | falsifier |
|---|---|
| Eliminated | A cited-deleted file still resolves on disk (git status no |
| | longer shows `D`), or a deleted script remains referenced by a |
| | live caller (Makefile/justfile/workflow) — the sweep was |
| | cosmetic, not an elimination. |
| Reduced | A reduced claim can still be made in prose without a register |
| | row: any ALIVE/PASS standing asserted in a window doc with no |
| | `KNOWN_DIVERGENCES.md` row or receipt path behind it, or a |
| | cache write that persists a dylib without the temp+rename |
| | discipline (KD-14 reopen). |
| Raised | The raised surface regresses silently: doctor.sh runs with a |
| | stage missing from the PHASES array without exit != 0 |
| | (fail-closed broken), or the 23/23 builder PASS is cited |
| | without a post-E0382-fix re-run receipt. |
| Created | A created artifact cannot be run or cannot refuse: |
| | `bash scripts/value_audit.sh`, `bash scripts/doctor.sh`, or |
| | `bash scripts/courts/run_all.sh` exits 0 while a leg is |
| | unimplemented (rc=0-with-0-tests class), or a receipt json in |
| | `receipts/` names no command/exit/output. |
