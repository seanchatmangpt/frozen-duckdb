# Court Ladder — 11 Stages (Sovereign Spec §5)

Lane X13 · frozen-duckdb · 2026-10-19.

The ladder is the ordered admission sequence a change must survive before it
is landed. Stages 2-11 are SMALT-repo courts (smalt's `sdk_coverage.py`,
`shim_court/lib.sh`, `sleep-gate` machinery). This repo satisfies each
stage's *intent* with its own rungs: `scripts/verify-gates.sh`,
`scripts/courts/*`, and the no-sleep doctrine in CLAUDE.md.

Status vocabulary: ALIVE (real implementation, real output on disk),
DESIGNED-ONLY (spec exists, no executable court today).

## The ladder

| # | Stage | Owner | frozen-duckdb mapping | Standing |
|---|---|---|---|---|
| 1 | doctor | smalt | `scripts/verify-gates.sh` preflight probes | PARTIAL_ALIVE |
| 2 | mix test residual | smalt | `cargo test --workspace` ladder | DESIGNED-ONLY |
| 3 | sdk-ts-court | smalt | Go FFI smoketest (analog surface) | DESIGNED-ONLY |
| 4 | sdk-py-court | smalt | drop-in duckdb-rs API-compat claim | DESIGNED-ONLY |
| 5 | tf-smoke | smalt | `run_ffi_validation.sh`, Go smoke | DESIGNED-ONLY |
| 6 | bench-check | smalt | GATE-V01 + `bench_gate.sh` | DESIGNED-ONLY |
| 7 | mutation-court | smalt | GATE-V fail-closed runner pattern | DESIGNED-ONLY |
| 8 | census | smalt | none (workspace ladder only) | DESIGNED-ONLY |
| 9 | invariant-gates | smalt | GATE-V02/V04, storage-format-v2 | DESIGNED-ONLY |
| 10 | naming-sweep | smalt | `spec_sync_check.sh`, `make genesis-check` | DESIGNED-ONLY |
| 11 | sleep-gate-sdk | smalt | no-sleep doctrine, no gate | DESIGNED-ONLY |

Per-stage detail (purpose, full mapping):

1. **doctor** — toolchain/environment probes gate all later stages.
   fd: `scripts/verify-gates.sh` probes + vendored-headers offline
   bindgen check. PARTIAL_ALIVE (2/5 green, BLOCKED run witnessed).
2. **mix test residual** — residual suite must be counted and zero-fail.
   fd: `cargo test -p frozen-duckdb` / `cargo test --workspace`
   (CLAUDE.md test ladder). No counted-residual gate executable.
3. **sdk-ts-court** — TS SDK parity via coverage compiler + KD-nn register.
   fd: Go FFI smoketest (`scripts/build_go_smoketest.sh`,
   `docs/testing/go-ffi-smoketest.md`) is the analog secondary surface.
4. **sdk-py-court** — Python SDK parity, same register discipline.
   fd: no Python surface; intent covered by the main crate's
   drop-in duckdb-rs replacement claim.
5. **tf-smoke** — end-to-end smoke through the real entrypoint.
   fd: `scripts/smoke_go.go`, `scripts/run_ffi_validation.sh`,
   `scripts/test_ffi_simple.sh`. Scripts exist; no court wrapper.
6. **bench-check** — performance fence; regression blocks admission.
   fd: GATE-V01 latency court + `scripts/bench_gate.sh`,
   `scripts/run_bench_suite.sh`, `scripts/courts/
   ingest-throughput-plan.md`. gate_v01.sh is a NOT-IMPLEMENTED stub.
7. **mutation-court** — every mutation must flip a real outcome
   (shim-court). fd: the GATE-V aggregate runner's fail-closed verdict
   table is the pattern; no mutation court executable yet.
8. **census** — enforce >=80% test-volume reduction, loud skips.
   fd: none; workspace ladder is the only volume accounting.
9. **invariant-gates** — CAS / ETag / WORM invariants under real load.
   fd: GATE-V04 (OPFS concurrency), GATE-V02 (crash chaos),
   `scripts/courts/storage-format-v2.md`, `sync-digest-design.md`.
10. **naming-sweep** — surface naming drift (API/doc/registry).
    fd: `scripts/spec_sync_check.sh` + `make genesis-check`
    (generated-surface drift check).
11. **sleep-gate-sdk** — no `sleep(N)` coordination; poll observables.
    fd: no-sleep doctrine (CLAUDE.md / global rule). No mechanical
    gate in this repo yet.

## Honest standing summary

Real, executable, receipted today in frozen-duckdb:

- Stage 1 intent: `scripts/verify-gates.sh` — executable, writes
  `scripts/verify-evidence.ttl`, but the witnessed run is 2/5 green
  (BLOCKED): build-workspace exit 0, test-validation exit 0;
  clippy-deny-warnings exit 101, fmt-check exit 1, test-workspace
  exit 101. Standing PARTIAL_ALIVE, not ALIVE.
- Court aggregate runner `scripts/courts/run_all.sh` is executable and
  fail-closed, but every cell is NOT-IMPLEMENTED.

Witnessed verify-gates output (2026-10-19, tail):

```
verify-gates: 2/5 gates green -> scripts/verify-evidence.ttl
  build-workspace: exit 0
  clippy-deny-warnings: exit 101
  fmt-check: exit 1
  test-validation: exit 0
  test-workspace: exit 101
STANDING: BLOCKED (red gate above; evidence: scripts/verify-evidence.ttl)
```

The red gates (fmt, clippy, workspace tests) are pre-existing on a tree
with many in-flight lane edits; this doc lane introduced no code changes.

Witnessed output (2026-10-19):

```
$ bash scripts/courts/run_all.sh; echo "exit=$?"
=== GATE-V verdict table ===   [rows verbatim; DETAIL column elided]
COURT      VERDICT          DETAIL
gate_v01   NOT-IMPLEMENTED  GATE-V01 court not implemented
gate_v02   NOT-IMPLEMENTED  GATE-V02 court not implemented
gate_v03   NOT-IMPLEMENTED  GATE-V03 court not implemented
gate_v04   NOT-IMPLEMENTED  GATE-V04 court not implemented
gate_v05   NOT-IMPLEMENTED  GATE-V05 court not implemented
GATE-V: FAIL (fail-closed: NOT-IMPLEMENTED and FAIL both block)
exit=1
```

(Each DETAIL cell in the real output also names its design doc:
`docs/design/gate-v0N-*.md`.)

The fail-closed exit-1 on a fully-empty ladder is the correct behavior per
`scripts/courts/run_all.sh`: NOT-IMPLEMENTED is never rendered as a green
cell. Everything in stages 2-11 is DESIGNED-ONLY for this repo.

## Closing the gap (order)

1. Implement gate_v01..gate_v05 bodies (stages 5/6/9 intent land at once).
2. Add a residual-test-count gate over `cargo test --workspace` (stage 2).
3. Mechanical no-sleep sweep over `crates/**` and `scripts/**` (stage 11).
4. Naming/surface drift: fold `spec_sync_check.sh` into verify-gates
   (stage 10).
5. Census and mutation courts last — they require survivors from 1-4.

## See Also

- `scripts/courts/run_all.sh` — fail-closed aggregate runner (executable)
- `scripts/courts/gate-v0*.md` — GATE-V designs (stages 5/6/9)
- `docs/design/verification-plan.md`, `docs/design/requirements-traceability.md`
- smalt repo: `scripts/sdk_coverage.py`, `scripts/shim_court/lib.sh`,
  `just sleep-gate` — canonical stages 3/4/7/11 machinery
