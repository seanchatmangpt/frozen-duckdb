# KAIZEN — doctor runtime (LSS4)

Lane LSS4. Subject: `scripts/doctor/` phase scripts, measured 2026-10-19 on
master@79e803e (working tree).

## Quick win landed

`DOCTOR_TARGET_DIR` (default `$ROOT/doctor-target/`) exported as
`CARGO_TARGET_DIR` in the 6 cargo-invoking phase scripts
(02, 03, 05, 07, 08, 09). One build paid across all doctor phases instead of
per-stage rebuild/link against the developer `target/`.

## Measured before/after (stages 02 + 03 + 05, sequential, isolated)

| stage | before (default `target/`, warm) | after (shared `doctor-target/`, warm) |
|---|---|---|
| 02_storage_wal | 216.13s | 1.40s |
| 03_types_variant | 4.14s | 0.59s |
| 05_memory_guard | 28.10s | 31.56s |
| **total** | **248.4s** | **33.6s** (7.4x) |

One-time cold cost of `doctor-target/`: 31.3s (stage 03 warm-up run, full
workspace build in the fresh dir). Exit codes 0; receipts in
`/tmp/after_*.log` names real test output ("2 passed", "1 passed").

Note: stage 05 variance (28s vs 31.6s) is test-body runtime (memory-guard
workload), not build time; it is build-dominated in neither case.

## Remaining kaizen backlog

1. **`--release` for the perf stage (08_perf_sla)** — latency/throughput SLA
   tests measure debug-build numbers today; run them `--release` so the SLA
   is asserted on the shipping profile. Requires an opt-in env
   (`DOCTOR_PERF_RELEASE=1`) so doctor stays fast by default.
2. **Test-parallelism within stages** — stages iterate test binaries serially
   in a `for` loop; pass all `--test` names in one `cargo test` invocation
   per stage (one link, parallel binaries) or run stage loops concurrently
   where tests are independent.
3. **Full-battery re-measure** — numbers above cover stages 02/03/05; the
   win compounds across 07/08/09 which now share the same dir. Time
   `scripts/doctor.sh` end-to-end before/after as the standing metric.
4. **doctor-target hygiene** — `doctor-target/` is a lane build root
   (fan-out cleanup law): add to coordinator integration cleanup and
   `.gitignore`.
5. **Cold-build amortization** — optionally seed `doctor-target/` from the
   developer `target/` (hardlink or `cargo` cache reuse) to remove the
   one-time 31s cold cost in CI-less local runs.

## Falsifier

Re-run the 3-stage timing with `DOCTOR_TARGET_DIR` unset in one arm:
if the shared-dir arm is not faster warm-vs-warm, the win is vacuous.
