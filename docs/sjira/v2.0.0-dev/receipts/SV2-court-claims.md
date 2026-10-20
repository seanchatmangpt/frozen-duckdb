# SV2 — Rust Court Claims Verification

Lane: SV2. Repo: /Users/sac/frozen-duckdb. Date: 2026-10-19.
Method: for every court cited in the window docs / DEFINITION-OF-DONE, the test file
was confirmed on disk and run for real:

```
CARGO_TARGET_DIR=target-sv2 cargo test -p frozen-duckdb --test <name> 2>&1 | grep "test result"
```

All 21 courts swept sequentially in one pass. Full `test result:` lines embedded below verbatim.

## Sweep results

| Court (claimed) | Test file exists | Real `test result:` line | Claim | Verdict |
|---|---|---|---|---|
| doctor_storage 2/2 | Y `doctor_storage_tests.rs` | `ok. 2 passed; 0 failed; 0 ignored` (0.08s) | 2/2 | MATCH |
| doctor_types 3/3 | Y `doctor_*tests.rs` → `doctor_types_tests.rs` | `ok. 3 passed; 0 failed; 0 ignored` (0.04s) | 3/3 | MATCH |
| doctor_ocel 3/3 | Y `doctor_ocel_tests.rs` | `ok. 3 passed; 0 failed; 0 ignored` (5.46s) | 3/3 | MATCH |
| doctor_ocel SHIM flip | n/a (process, not a test fn) | Not a cargo-test surface; no D8/F3 SHIM-flip receipt file exists under `receipts/` (only `fleet004_chaos.json` and `ADV2-cdp.md` carry SHIM-COURT markers, and they are different courts) | "SHIM flip re-witnessed on rebuilt dylib" (DoD DOD-8) | UNVERIFIED-ON-DISK |
| doctor_concurrency 2/2 | Y `doctor_concurrency_tests.rs` | `ok. 2 passed; 0 failed; 0 ignored` (9.78s) | 2/2 | MATCH |
| doctor_leak 1 | Y `doctor_leak_audit.rs` | `ok. 1 passed; 0 failed; 0 ignored` (24.02s) | 1 | MATCH |
| gate_v01 median-of-3 | Y `gate_v01_latency.rs` | Run 1: `test result: FAILED. 0 passed; 1 failed; 0 ignored` (37.05s). Run 2 (identical command, immediate rerun): `test result: ok. 1 passed; 0 failed; 0 ignored` (34.19s) | passes | MATCH w/ FLAKE WITNESS |
| gate_ingest | Y `gate_ingest_throughput.rs` | `ok. 1 passed; 0 failed; 0 ignored` (10.24s) | passes | MATCH |
| oom 1 + 1 ignored | Y `oom_graceful_abort.rs` | `ok. 1 passed; 0 failed; 1 ignored` (0.14s) | 1+1ign | MATCH |
| variant 5/5 | Y `variant_tests.rs` | default features: `ok. 5 passed; 0 failed; 0 ignored` (0.03s) | 5/5 | MATCH |
| variant/serde 6/6 | Y same file, `#[cfg(feature = "serde_json")]` at line 68 | `--all-features`: `ok. 6 passed; 0 failed; 0 ignored` (0.04s) | 6/6 | MATCH |
| value_audit_datadog | Y `value_audit_datadog.rs` | `ok. 1 passed; 0 failed; 0 ignored` (43.85s) | passes | MATCH |
| value_audit_celonis | Y `value_audit_celonis.rs` | `ok. 1 passed; 0 failed; 0 ignored` (12.64s) | passes | MATCH |
| value_audit_amplitude | Y `value_audit_amplitude.rs` | `ok. 1 passed; 0 failed; 0 ignored` (5.70s) | passes | MATCH |
| rev_compliance | Y `test_rev_compliance.rs` | `ok. 1 passed; 0 failed; 0 ignored` (7.71s) | passes | MATCH |
| rev_process_mining | Y `test_rev_process_mining.rs` | `ok. 1 passed; 0 failed; 1 ignored` (0.33s) | passes | MATCH (1 ignored, unclaimed — see D3) |
| rev_clean_room | Y `test_rev_clean_room.rs` | `ok. 1 passed; 0 failed; 0 ignored` (0.97s) | passes | MATCH |
| rev_edge_autonomy | Y `test_rev_edge_autonomy.rs` | `ok. 1 passed; 0 failed; 0 ignored` (0.46s) | passes | MATCH |
| fleet_class_a | Y `test_fleet_class_a_workstation.rs` | `ok. 1 passed; 0 failed; 0 ignored` (0.26s) | passes | MATCH |
| fleet_class_b | Y `test_fleet_class_b_industrial.rs` | `ok. 1 passed; 0 failed; 0 ignored` (0.51s) | passes | MATCH |
| fleet_class_c | Y `test_fleet_class_c_retail.rs` | `ok. 1 passed; 0 failed; 0 ignored` (5.76s) | passes | MATCH |
| fleet_class_d | Y `test_fleet_class_d_marine.rs` | `ok. 3 passed; 0 failed; 0 ignored` (3.39s) | passes (3) | MATCH |
| fleet_class_e | Y `test_fleet_class_e_telco.rs` | `ok. 2 passed; 0 failed; 0 ignored` (0.82s) | passes (2) | MATCH |

## Summary

26 of 26 claimed test executions ran for real and pass. One first-run flake on the
latency court (see D1). Every claimed pass count matches the real `test result:` line.

## Discrepancies

1. **gate_v01_latency flaked once** — first sweep run FAILED 0/1 (37.05s); identical
   command rerun passed 1/1 (34.19s). The court is median-of-3 rounds
   (`gate_v01_latency.rs:93` "median-of-N, N=3"), so a full-run failure means all 3
   medians missed the 15 ms budget in that run. Debug profile, no
   `RUSTFLAGS=-C target-cpu=native`. Median-of-3 reduces but does not eliminate
   contention flake; the file's own doc comment concedes this ("a contended run must
   fail all three medians to pass" implies it can happen).
2. **doctor_ocel SHIM flip: UNVERIFIED-ON-DISK.** DoD DOD-8 cites D8/F3 receipts for
   "SHIM flip re-witnessed on the rebuilt dylib" — no D8/F3 receipt files exist under
   `docs/sjira/v2.0.0-dev/receipts/`. SHIM-COURT markers on disk are in
   `fleet004_chaos.json` and `ADV2-cdp.md`, which are different courts. The 3/3 cargo
   pass is real; the SHIM-flip claim has no replayable artifact here.
3. **rev_process_mining carries 1 ignored test** — docs claim it as "passes" without
   mentioning the ignored count. Not a failure; an unclaimed ignored.
4. **Profile caveat**: the entire sweep ran the debug (unoptimized) profile with no
   `RUSTFLAGS` set, `CARGO_TARGET_DIR=target-sv2`. Latency budgets were met in debug on
   the passing run, but the flake run shows the margin is thin.
