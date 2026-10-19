# How To: Run the Chicago Doctor

Problem-oriented guide: run the full verification battery, read the
verdict lines, and interpret the exit-code contract.

`just doctor` runs `scripts/doctor.sh`: 13 phase scripts (00..12) under
`scripts/doctor/`, aggregated **fail-closed** — exit 0 only if every
phase exits 0.

See Also: `docs/design/doctor-implementation-map.md`,
`docs/design/doctor-verification-blueprint.md`,
`docs/design/doctor-dual-pass-court.md`

## Run it

```bash
just doctor        # full 13-phase battery
just doctor-quick  # phase 00 preflight only (fast prerequisite check)
just court-x2      # dual-pass idempotence court (TEST-025)
just courts        # anti-vacuity shim courts (BASELINE/BEFORE/SHIM/AFTER/RESTORE)
```

Prerequisite: the frozen-duckdb engine must be built (`cargo build`);
the doctor loads the built dylib. Running pre-rebuild yields honest
phase-1 failures, not faked passes.

## The stages

| Phase | Stage role | Grounded in |
|---|---|---|
| 00_preflight | probes + sys-audit handoff | `scripts/doctor/00_preflight.sh` |
| 01_sys_audit | binary/symbol audit | `checks/bin_audit.sh` (wired) |
| 02_storage_wal | storage/WAL | `doctor_storage_tests.rs` |
| 03_types_variant | types/variant | `doctor_types_tests.rs` |
| 04_host_vfs | host VFS transfer | BLOCKED FR-01 (KD-06) |
| 05_memory_guard | memory ceiling/OOM | `doctor_leak_audit` + `oom_graceful_abort` |
| 06_browser_opfs | browser/OPFS | BLOCKED FR-01 (KD-05) |
| 07_ocel_compress | OCEL compression court | `doctor_ocel_tests.rs` |
| 08_perf_sla | perf SLA fence | `gate_v01_latency.rs` et al. |
| 09_concurrency | concurrency | `doctor_concurrency_tests.rs` |
| 10_verify_economics | economics court | loud exit-3 until court runner lands |
| 11_revenue_packs | REV pack courts | `scripts/value_audit/` |
| 12_fleet_hardware | fleet class courts | `test_fleet_class_*.rs` |

## Exit-code contract (per phase)

| Exit | Verdict line | Meaning |
|---|---|---|
| 0 | `[PASS] <phase>` | verified clean |
| 2 | `[SKIP] <phase>` | loud skip (e.g. cargo not on PATH); fails the aggregate |
| 3 | `[NOT-IMPLEMENTED] <phase>` | loud divergence, never a fake pass; fails the aggregate |
| other | `[FAIL] <phase> (exit N)` | real failure; fails the aggregate |

Note: any non-zero phase exit (including SKIP and NOT-IMPLEMENTED) sets
`any_fail=1`; the aggregate then exits 1. There is no silent pass path.

## Reading the verdict

Tail of a run:

```
== KNOWN DIVERGENCES ==
total: 14  open: 12  closed: 2
KD-03 KD-04 ... KD-14
DOCTOR VERDICT: FAIL-CLOSED — one or more phases not clean
```

- The KNOWN DIVERGENCES block is parsed live from `KNOWN_DIVERGENCES.md`
  (rows starting `| KD-`; `CLOSED —` counts as closed). It is
  informational; it does not change the exit contract.
- `DOCTOR VERDICT: FAIL-CLOSED ...` + exit 1 is the CORRECT state on
  darwin today: stages 04/06 are FR-01-blocked (no wasm artifact,
  KD-05) and 11/12 carry platform-gated loud divergences by design.
  The aggregate stays red until those KD rows close — that standing is
  honest, not a defect.
- `ALL 13 PHASES VERIFIED CLEAN` + exit 0 is the only green.

## The idempotence court (just court-x2)

`scripts/doctor/dual_pass.sh` runs two back-to-back doctor runs at one
HEAD and asserts A1-A5 (idempotence: same phase standings, evidence TTL
regenerated, no `target-lane*/` residue, receipt validity via
`scripts/verify_publish_receipt.py`). Exit 0 only if every assertion
holds; any breach exits 1 naming the assertion. Logs land at
`/tmp/doctor-pass1.log` and `/tmp/doctor-pass2.log`. It is an
observation court — no shim, read-only over git.

## Stage receipts

Each stage's real test output (cargo `test result:` lines, measured
ratios, exits) is the receipt. Do not trust a summary that lacks the
embedded command output; rerun the stage script directly, e.g.
`bash scripts/doctor/07_ocel_compress.sh`, and cite what printed.

See Also: `KNOWN_DIVERGENCES.md`,
`docs/how-to/how-to-memory-caps.md` (stage 05 detail),
`docs/how-to/how-to-ocel2-compression.md` (stage 07 detail)
