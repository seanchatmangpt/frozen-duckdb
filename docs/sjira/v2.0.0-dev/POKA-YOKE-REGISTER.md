# Poka-Yoke Register — v2.0.0-dev campaign (lane LSS3)

Mistake-proofing inventory: each device names the mistake it prevents and the
real incident (or lane receipt) that motivated it. A device with no named
incident is prose, not a guard.

## Landed devices

| # | Device | Surface | Mistake prevented | Motivating incident |
|---|--------|---------|-------------------|---------------------|
| P1 | ENGINE_VERSION marker + size pin | `crates/frozen-duckdb-builder` cache (`~/.frozen-duckdb/cache/`) | Stale/mismatched engine reused after version flip (builder links against a cached lib whose ENGINE_VERSION disagrees with the crate version 2.0.0-dev) | The 1.5.5 -> 2.0.0-dev flip (commit e3b6eda): cache keyed only by version would have served the old engine; pin forces re-acquire |
| P2 | Fail-closed doctor | `scripts/doctor.sh`, `scripts/doctor/` | Environment drift read as healthy (a probe that errors is read as "not applicable" and the run continues green) | BINARIES-FAILURE-ANALYSIS.md — binaries pipeline failures that surfaced only at download/CI time |
| P3 | SHIM courts (BASELINE→BEFORE→SHIM→AFTER→RESTORE, sha256-exact restore) | `scripts/courts/` | Vacuous gates — a gate that cannot fail is admitted as proof | Smalt shim-court pattern; campaign court index `docs/sjira/v2.0.0-dev/receipts/COURT-INDEX.md` |
| P4 | ggen byte-identical sync (genesis-check) | `make gates` / `ggen sync run` | Hand-edits to generated surfaces (`scripts/verify-gates.sh`, `Makefile`) silently diverging from the schema (`schema/verify.ttl`) | Generated-surface drift found during the docs campaign (commit 79e803e genesis 58/58) |
| P5 | Atomic cache writes | `crates/frozen-duckdb-builder` acquire step | Concurrent lanes observing a half-written cached lib and linking garbage | 10+-lane fan-out over one checkout (target-lane* history in `docs/sjira/v2.0.0-dev/_LANES.md`) |
| P6 | BUILDER_VERSION cross-check | `.github/workflows/*` + `scripts/fastfollow_flip.sh` | Crate version and builder binary pinned to different versions across 5 workflows | ci lockstep commit d307ae4 — matrices drifted before the cross-check existed |
| P7 | Probe-first scripts | `scripts/doctor/`, download/acquire path | Full local DuckDB compile from source when a prebuilt exists — hours wasted per lane | README known open item: GH Release v1.5.5 assets missing -> every lane compiled locally before probe-first landing |

## Gaps (prose-only, unguarded)

- **Lane-lease cleanup law**: "the coordinator deletes every lane's
  `CARGO_TARGET_DIR=target-lane<N>` at integration, BEFORE the final commit"
  (`~/.claude/rules/same-checkout-fanout.md`) is currently prose plus a cleanup
  lane's receipt — nothing mechanical rejects a commit while leases are live.
  14 `target-lane*/` dirs exist on this tree right now (A1, A3, CG1, ERRC, R1,
  R2, R3, T1, T3, T4, T5, T6, V2, V3).
- **Absolute-path leak class (AD8)**: commits embedding `/Users/sac` paths.
  Not yet guarded; candidate for a later device.

## Implemented this lane (LSS3): P8 lane-lease pre-commit guard

- **Surface**: `scripts/check_lane_leases.sh` + `scripts/git-hooks/pre-commit`
  (manual install by operator).
- **Mistake prevented**: an integration commit that captures lane build roots
  (the 30GB orphaned-lease class; ~2GB+ per `target-lane*` dir here).
- **Mechanism**: fail-closed — nonzero exit if any `target-lane*` dir exists
  under repo root, with the per-dir size printed so the operator sees the lease
  cost, not just a refusal. Support `--check` mode for non-commit use.
- **Falsifier (how the trip is proven non-vacuous)**:
  `scripts/courts/shim_court_lane_lease.sh` — BASELINE (hash dir listing) ->
  SHIM (create `target-laneTEST` marked SHIM-COURT) -> court runs
  `check_lane_leases.sh`, expects exit 0 (no leases) BEFORE, nonzero AFTER
  the shim dir is minted, RESTORE (dir removed, listing byte-compare
  fail-closed).

## Install (operator action, not run in-lane)

```bash
cp scripts/git-hooks/pre-commit .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit
```

## Banned atoms

This register models no publish. `published`, `crates-io-uploaded`,
`release-complete` are not invoked; standing is `dry-run-verified` class only.
