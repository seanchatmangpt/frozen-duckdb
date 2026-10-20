# SV1 — Doctor / Stage-Script Claims Audit (2026-10-19)

Lane SV1, repo /Users/sac/frozen-duckdb. Scope: doctor/stage-script claims in
`_LANES.md`, `DEFINITION-OF-DONE.md`, `CAMPAIGN-RECEIPT.md`,
`ADV16-INTEGRATION.md`, plus the named auxiliary scripts (fastfollow_flip,
publish_chain, extension_watch, verify_release_assets, wasm_size_audit,
quickstart_demo, cdp_smoke, cleanroom, dual_pass, check_lane_leases). Method:
grep the four docs for each claimed artifact, then `ls -x` existence,
executability, and read each script's actual exit contract vs the doc claim.
No git state commands; no fixes applied (report-only lane).

## Claims table

| CLAIMED | EXISTS | EXEC | MATCHES-DESCRIPTION | DISCREPANCY |
|---|---|---|---|---|
| `scripts/doctor.sh` aggregate, fail-closed, "ALL 13 PHASES" (CAMPAIGN L350 "stages 11/12 now wired") | Y | Y | Y | Naming drift only: docs call it the "11-stage ladder" (_LANES Wave 10, CAMPAIGN L141); PHASES array is 13 entries 00..12. Banner says 13. Historical naming; no functional conflict. |
| 00_preflight .. 12_fleet_hardware (13 stage scripts) | Y (all 13) | Y (all 13) | Y | none |
| 01_sys_audit propagates `checks/bin_audit.sh` verdict (court-x2, CAMPAIGN L147) | Y | Y | Y | Script reads "run checks/bin_audit.sh and propagate its verdict"; missing check exits 3 NOT-IMPLEMENTED. bin_audit.sh present + executable. |
| 02_storage_wal "2/2 green" (DOD-4) | Y | Y | Y* | Script execs ONE cargo test invocation (`exec cargo test … --test "$name"`); the "2/2" figure is not derivable from the stage script itself (it is whatever the test file contains). Not contradicted, just not verifiable from the stage alone. |
| 03_types_variant "3/3 green" (DOD-3) | Y | Y | Y* | Same shape as 02 — single `cargo test` exec; per-leg count lives in the test file, not the stage. |
| 05_memory_guard (DOD-6) | Y | Y | Y | exists/executable; contract per stage matrix. |
| 08_perf_sla "non-vacuous PASS" (DOD-8) | Y | Y | Y | `exit 2` on missing artifact (loud), runs gate_v01_latency + gate_ingest_throughput as real `cargo test --release`, `exit "$fail"`. Contract matches "real cargo tests" claim. |
| 09_concurrency (DOD-7) | Y | Y | Y | — |
| 10_verify_economics "stage exits 3 loudly" (CAMPAIGN L157) | Y | Y | Y | Line 14 `exit 3` on missing court script; exit contract comment 0/1/2/3 confirmed. |
| 11_revenue_packs "fail-closed, EXIT=1 four legs FAIL; NOT wired into doctor.sh" (CAMPAIGN L199-213) | Y | Y | Y (historical note now stale) | Script itself still says "NOT wired into scripts/doctor.sh" in its header comment, but `11_revenue_packs` IS in doctor.sh PHASES (line 31) — the stage file's own wiring comment is stale vs the aggregate. |
| 12_fleet_hardware "EXIT=1 fail-closed, EXIT=3 propagates NOT-IMPLEMENTED" (CAMPAIGN L236-274) | Y | Y | Y | Header contract 0/1/2/3, per-leg NOT-IMPLEMENTED-LOUD on rc=0-with-0-tests, rc=2/3 propagation, final FAIL/NOT-IMPLEMENTED exits — all present in file. |
| dual_pass.sh (E4) | Y | Y | Y | TEST-025 court over two back-to-back doctor runs; exit 0 only all assertions; [SKIP] loud. Matches E4 claim. |
| check_lane_leases.sh (P8, poka-yoke) | Y | Y | Y | Rejects commit while `target-lane*/` exists; exit 1 fail-closed. Matches. |
| extension_watch.sh | Y | Y | Y (existence claim in ADV16 L28-68 wiring list) | — |
| verify_release_assets.sh | Y | Y | Y | 0/1/2 exit contract, "empty release state exits 2 loudly", real downloads + Mach-O inspection. |
| wasm_size_audit.sh | Y | Y | Y | SLA raw<=25MiB / gzip<=10MiB, fail-closed [SKIP] on missing tools, exit 2 on missing artifact. |
| publish_chain.sh — DRYRUN=1 default / PUBLISH_CONFIRM=yes real mode (PUBLISH-DECISION L58) | Y | Y | Y | `DRYRUN="${DRYRUN:-1}"`, real mode requires `PUBLISH_CONFIRM=yes` AND `DRYRUN=0` else `die` (REFUSED exit 1). Matches. |
| fastfollow_flip.sh `OLD NEW NEW_SHA` fail-closed version flip (PUBLISH-DECISION L77, PUBLISH-READINESS L68/109, STANDARD-WORK-RELEASE L40-44, POKA-YOKE-REGISTER P6) | **N** | — | — | **MISSING from scripts/`. Claimed in 4 window docs (2 outside the primary four: PUBLISH-READINESS, STANDARD-WORK-RELEASE, POKA-YOKE-REGISTER) as the runbook step C1.1 and P6 control surface. FINDING.** |
| quickstart_demo.sh (LSS10) | Y | Y | Y | Real cargo project scaffold, QUICKSTART-RECEIPT lines, rpath probe. |
| cdp_smoke.mjs (ADV16 L60: "7643 → 7800 bytes") | Y | N (no +x bit) | Y | Byte claim exact: `wc -c` = 7800. Exec bit absent — run via `node`, cosmetic only. SHIM-COURT marker + real browser teardown present. |
| cleanroom/registry_consumer.sh | Y | Y | Y | Loud FAIL/SKIP exits 1/2, receipt JSON, offline-vendor retry with divergence record. |
| `scripts/build_wasm_mvp.sh` (ADV16 L68 wiring list) | Y | — | Y | — |

## Findings (report-only, not fixed)

1. **`scripts/fastfollow_flip.sh` MISSING** — referenced as an executable
   runbook step (PUBLISH-DECISION step, PUBLISH-READINESS row 5
   "OPERATOR: `scripts/fastfollow_flip.sh 2.0.0-dev <NEW> <40-hex upstream SHA>`",
   STANDARD-WORK-RELEASE C1.1, POKA-YOKE P6) in four window docs; no such file
   anywhere under `scripts/`. Any release runbook execution following those docs
   fails at C1.1. Severity: release-blocking gap in the documented path.
2. **Stale wiring comment in `scripts/doctor/11_revenue_packs.sh`** — header
   still states "this stage is NOT wired into scripts/doctor.sh", but the stage
   IS in the doctor.sh PHASES array. Doc claim ("now wired", CAMPAIGN L333) is
   the current truth; the script's self-description is the stale side.
3. **"11-stage ladder" vs 13-phase PHASES naming drift** — _LANES Wave 10 and
   CAMPAIGN L141 formalize 11 stages 00..10; doctor.sh now carries 13 phases
   (banner "ALL 13 PHASES VERIFIED CLEAN"). Docs elsewhere record the 11/12
   wiring, so this is naming residue, not a contradiction, but counts cited as
   "6/11" in DOD-11/CAMPAIGN are against the 11-stage frame, not the current
   13-phase aggregate.
4. **Minor**: `scripts/doctor/cdp_smoke.mjs` lacks the executable bit (run via
   `node`; all sibling court scripts are +x). Cosmetic.

## Standing

- Claimed artifacts audited: 24
- EXISTS+EXEC+MATCHES: 20 clean, 1 exact-byte match (cdp_smoke 7800 B)
- Discrepancies: 4 (1 missing artifact — fastfollow_flip.sh; 1 stale in-file
  comment; 1 naming drift; 1 cosmetic exec bit)
- Court shape: read-only claims audit (no shim, no gate flip); every cell above
  grounded in a disk read or grep hit at 2026-10-19.
