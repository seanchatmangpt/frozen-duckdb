---
id: TR1
dcterms:title: "PR CI triage + workflow fixes"
dcterms:created: "2026-09-21"
dcterms:isPartOf: "v26.9.21"
rdf:type: oslc_cm:ChangeRequest, earl:TestRequirement
aps:standing: https://w3id.org/chatman/aps#OPEN
milestone: frozen-duckdb v1.5.5 wave 3 (hardening, pre-publish)
worktree: /Users/sac/frozen-duckdb-wt/r1
branch: feat/155-r1
---

# TR1 — PR CI triage + workflow fixes

## Scope (files you may change)

```
.github/workflows/{ci.yml,ci-simple.yml,test-minimal.yml}
```

Anything outside scope: do not edit — record BLOCKED with the reason instead.

## Work order


Context: draft PR #3 (feat/duckdb-1.5.5) triggers ci.yml on ubuntu+macOS. Check its live status: gh api repos/seanchatmangpt/frozen-duckdb/actions/runs?branch=feat/duckdb-1.5.5 and .../commits/<sha>/check-runs (gh 2.54: use gh api, NOT gh pr checks -R). Diagnose any red run from its logs (gh api .../actions/runs/<id>/logs?job_id=... or jobs listing). Likely hazards: ubuntu jobs have no prebuilt .so (builder falls back to a long local compile — may time out), setup_env.sh assumes macOS dylib names.
DoD: [ ] PR #3 check status recorded (run ids + conclusions)  [ ] red causes fixed within workflow scope or explicitly documented as builder-owned with the fix specified  [ ] yaml-parse all touched workflows  [ ] committed

## Contract (all tickets)

- Work ONLY in `/Users/sac/frozen-duckdb-wt/r1` on `feat/155-r1`. NEVER touch the main checkout /Users/sac/frozen-duckdb or sibling worktrees. Never push, never touch remotes, never publish.
- Atomic commits, conventional messages, on your branch only.
- Verify before claiming: preserve command + exit. A gate you did not run is UNKNOWN, not green.
- Append every transition to the History table below (the one sanctioned write outside your worktree), then STOP — the coordinator merges.

## History

| ts | standing | branch+SHA | gates+exits | remaining |
|----|----------|------------|-------------|-----------|
| 2026-09-21 | OPEN | feat/155-r1@c55d4f2 | not started | all of DoD |
