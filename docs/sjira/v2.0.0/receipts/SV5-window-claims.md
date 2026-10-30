# SV5 — final window coherence audit (2026-10-19)

Lane SV5, read-only; owns this file only. Method: every claim below was
re-read from disk this pass, not recalled. Commands run with real exit
codes; sample sizes noted per check.

## 1. _LANES.md wave tables — deliverables on disk

Sampled 13 wave tables (base W1-W12, Waves 3-14), 2-4 distinctive files
each (58 paths). Results:

| wave | sampled deliverables | result |
|---|---|---|
| base W1-W12 | sys Cargo.toml, vendored duckdb.h, logical_type.rs, config.rs, workflows | ALL OK |
| 3 (BP) | schema/blueprint.ttl, gates/blueprint/*-shapes.ttl, LICENSE-ENTERPRISE-DRAFT.md, docs/design/README.md | ALL OK |
| 4 (P) | docs/design/prd-ard-v2.md (NOT repo root), requirements-traceability.md, boundary-abi-contract.md | ALL OK |
| 5 (W-EXT) | extension-matrix-8020.md, host-call-vfs.md, build_wasm_duckdb.sh, extension-abi-hub.md | ALL OK |
| 6 (R1-R16) | intake files cited live in CAMPAIGN residues | OK |
| 7 (X) | gate_v01_latency.rs, gate_ingest_throughput.rs, oom_graceful_abort.rs, crash_chaos_native.sh (scripts/courts/), sovereign-spec, wasm-abi-surface.md | ALL OK |
| 8 (D/E) | doctor.sh, 01_sys_audit.sh, dual_pass.sh (scripts/doctor/), attest_pin (scripts/doctor/checks/attest_pin.sh), crash_chaos_native.sh (scripts/courts/) | ALL OK |
| 9 (G) | warehouse-sim/schema/warehouse_sim.ttl, ggen.toml, economics/court.ts, sovereign_probe/ | ALL OK |
| 10 (H) | stage renames verified: 00_preflight..10_verify_economics all on disk, implementation-map.md | ALL OK |
| 11 (S) | routes_contract.test.ts, warehouse_clients.ts, journeys.test.ts, disruption scorecard | ALL OK |
| 12 (R) | test_rev_{compliance,process_mining,clean_room,edge_autonomy}.rs, 11_revenue_packs.sh, revenue-packs.md | ALL OK |
| 13 (T) | cgroup_low_ram_runner.sh, fleet_d_chaos.sh, test_fleet_class_{a..d}*.rs, 12_fleet_hardware.sh | ALL OK |
| 14 (GAP) | receipts/ dir populated (PB*, ADV*, AD*, court receipts) | ALL OK |

Zero misses. Path drift found is benign (prd-ard-v2.md lives under
docs/design/, not repo root; dual_pass/attest_pin/crash-chaos under
scripts/doctor{,/checks}/ and scripts/courts/), not lane failures.

## 2. DEFINITION-OF-DONE re-verification (6 heaviest spot-checks)

- DOD-1 (ALIVE): dylib exists, thin arm64 Mach-O; `strings … | grep -m1
  v2.0.0` -> `v2.0.0`. CAVEAT: the binary in the slot was
  REBUILT after the final reconciliation read — slot file mtime is now
  Oct 19 16:15, 65,900,736 bytes (reconciliation witnessed 13:40 /
  117,005,184 bytes, sha prefix `acb8bc49ebae74f5`). Current sha256 =
  `d4cc9336f3738a16f7c7fd44e44c2fa8aba760dfe75c9f2e3ddcd0b2e3eecd19`.
  Self-report holds, so DOD-1 stays ALIVE; the `8a22ef19` /
  `acb8bc49…` sha citations are STALE against the 16:15 rebuild.
- DOD-11 (PARTIAL_ALIVE): stages 00..12 on disk (13 files incl. 11/12);
  ladder standing unchanged from receipts.
- DOD-13 (PARTIAL_ALIVE): CONFIRMED open. `scripts/verify-evidence.ttl`
  (mtime 17:37) still records `ver:exitCode 101` (test-workspace) and
  `exitCode 1` (fmt-check) — the last battery receipt is a FAILING run,
  exactly ADV16 blocker B3. Manifest parse now clean (`cargo metadata`
  exit 0; `serde_json` dep now `optional = true`), so ADV16 B1 is FIXED
  but B3's regenerate-after-green step has NOT run.
- DOD-14 (PROPOSED): all blueprint files exist; re-ran
  `bash scripts/courts/run_all.sh` -> all five GATE-V courts
  NOT-IMPLEMENTED, "GATE-V: FAIL (fail-closed)", consistent with
  "no court flipped; no layer claims ALIVE".
- DOD-15 (BLOCKED upstream): KD-01 open in KNOWN_DIVERGENCES.md.
- DOD-19 (ALIVE/discharged): consistent with the F10 strings probe.

## 3. CAMPAIGN-RECEIPT vs KNOWN_DIVERGENCES

- CONTRADICTION (stale count): CAMPAIGN basis says "KDs open: 11 of 14
  (KD-03/04/14 CLOSED)". KNOWN_DIVERGENCES.md carries a KD-15 row
  (downstream consumer rpath divergence, open). True current state:
  12 open of 15 (KD-03/04/14 closed). UPGRADE-WO.md and
  PUBLISH-DECISION.md repeat the same stale 11-of-14 figure. The
  campaign's own Part-2 receipts (PB4, ACT1 cleanroom court, commit
  fc2bbec) are what minted KD-15, so this is an unpropagated update,
  not a fake.
- KD verdicts themselves: all 15 rows re-read; closed/open labels
  internally consistent with cited receipts (KD-05 PARTIAL_ALIVE with
  real /tmp receipts; KD-12 OBSERVED not ALIVE — honest).
- Engine-identity fields: DOD/CAMPAIGN cite sha `8a22ef19` (F10) and
  reconciliation cites WITNESSED `acb8bc49ebae74f5`; neither matches
  the current slot binary (d4cc9336…, rebuilt 16:15). Disclosed as
  dual-citation in the receipt but now three-way stale.

## 4. Cross-doc contradiction table

| # | surfaces | contradiction | severity |
|---|---|---|---|
| C1 | CAMPAIGN / UPGRADE-WO / PUBLISH-DECISION vs KNOWN_DIVERGENCES | "11 of 14 KDs open" vs 15 rows, 12 open (KD-15 uncounted) | MEDIUM — count drift, verdicts agree |
| C2 | DOD-1/F10/CAMPAIGN vs cache slot | engine sha cited `8a22ef19` / witnessed `acb8bc49…`; slot now hashes `d4cc9336…` (rebuilt 16:15) while self-report `v2.0.0` unchanged | MEDIUM — standing holds, identity receipt stale |
| C3 | ADV16 B3 vs verify-evidence.ttl | B3 said do-not-commit the failing battery receipt; ttl STILL records exitCode 101/1 at 17:37 | OPEN BLOCKER, unresolved |
| C4 | ADV16 B1 vs manifests | B1 compile freeze; manifests now parse (`cargo metadata` exit 0, serde_json optional=true) | RESOLVED — ADV16 not updated to say so |
| C5 | PUBLISH-DECISION Option-A/B | prerelease-acceptance claim vs FIN6 "unpublishable" — disclosed as DISPUTED on disk, live dry-run receipt still absent | DISCLOSED, not a defect |
| C6 | DOD-20 REQUIRED | still open (final integration reconciliation + commit not yet run; "Campaign commits" table empty) | CONSISTENT open item |

## 5. Final honest window standing (independently derived)

PARTIAL_ALIVE — same as the campaign's own claim, derived
independently:

- ALIVE: engine flip (self-report v2.0.0 witnessed this pass);
  court battery artifacts all on disk; doctor ladder 00..12 executable;
  S-wave/revenue/fleet/sim surfaces landed with mixed WITNESSED /
  DISPATCH-CITED receipts (disclosed, not faked).
- Open: 12 of 15 KDs; blueprint PROPOSED (GATE-V fail-closed re-run
  witnessed this pass); DOD-13 battery red (B3 unregenerated); DOD-18
  hold (operator); DOD-20 integration pass REQUIRED. Blocks loud and
  named.
- No overclaim found: standing fields are internally consistent apart
  from C1/C2 citation staleness. Window standing = PARTIAL_ALIVE.
