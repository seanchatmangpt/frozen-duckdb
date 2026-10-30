# CI-PS-preseed-fix: best-effort pre-seed blocks in CI workflows

Date: 2026-10-19. Lane: CI-PS.
Subject files:
- `.github/workflows/ci.yml`
- `.github/workflows/ci-simple.yml`
- `.github/workflows/test-minimal.yml`

## Scope

Each of the 5 cache/pre-seed blocks across the three workflow files was made
best-effort: a pre-seed failure (e.g. cache miss, artifact 404) must not fail
the workflow step — pre-seeding is an optimization, not a gate.

## Results (lane's final report, session 2026-10-19)

- 5 pre-seed blocks converted to best-effort (continue-on-error / guarded
  curl paths) across ci.yml, ci-simple.yml, test-minimal.yml
- live 404 simulation: pre-seed step exercised against a deliberately
  nonexistent artifact URL — step exited 0 with the 404 logged, workflow green

Simulation command shape (as run in the lane):

```
# pre-seed step against nonexistent artifact -> observed exit 0
```

## Standing

**PASS** — 5/5 blocks best-effort; 404 simulation exit 0.

## Provenance

This receipt was transcribed post-run from the lane's final report (not
written by the lane itself). Verification is re-runnable by pushing a branch
triggering the workflows, or locally replaying the guarded curl block against
a nonexistent URL and asserting exit 0.

Transcribed, re-run to verify: workflow dispatch on a branch (no cheap local equivalent).
