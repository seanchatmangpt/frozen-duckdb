---
id: TR2
dcterms:title: "Release rehearsal + build-binaries hardening"
dcterms:created: "2026-09-21"
dcterms:isPartOf: "v26.9.21"
rdf:type: oslc_cm:ChangeRequest, earl:TestRequirement
aps:standing: https://w3id.org/chatman/aps#OPEN
milestone: frozen-duckdb v1.5.5 wave 3 (hardening, pre-publish)
worktree: /Users/sac/frozen-duckdb-wt/r2
branch: feat/155-r2
---

# TR2 — Release rehearsal + build-binaries hardening

## Scope (files you may change)

```
.github/workflows/build-binaries.yml
```

Anything outside scope: do not edit — record BLOCKED with the reason instead.

## Work order


Context: the v1.5.5 tag run has never executed — rehearse it before the operator cuts. (1) HARDEN the workflow: matrix builds x86_64 on an arm64 runner — the local-compile fallback needs CMAKE_OSX_ARCHITECTURES to label the slice correctly; set it per matrix arch in the workflow AND honor an env override in builder (coordinated: TR7 owns builder lib.rs — set the env here, TR7 makes builder read it; note this dependency in History). (2) REHEARSE the ensure_binary cache-miss path locally in isolation: HOME=/tmp/fd-rehearsal-home cargo build of a scratch project depending on the builder — exercises download-404 → local compile of DuckDB v1.5.5. That is a 15-45 min full source build: run it with a generous timeout; if it exceeds 40 min, record partial evidence (cmake configured + make progressing) and stand PARTIAL. Verify the produced artifact lands at the path/name the workflow's find step expects.
DoD: [ ] workflow hardened + yaml-parse green  [ ] rehearsal executed with command+exit (or PARTIAL with preserved progress evidence)  [ ] committed

## Contract (all tickets)

- Work ONLY in `/Users/sac/frozen-duckdb-wt/r2` on `feat/155-r2`. NEVER touch the main checkout /Users/sac/frozen-duckdb or sibling worktrees. Never push, never touch remotes, never publish.
- Atomic commits, conventional messages, on your branch only.
- Verify before claiming: preserve command + exit. A gate you did not run is UNKNOWN, not green.
- Append every transition to the History table below (the one sanctioned write outside your worktree), then STOP — the coordinator merges.

## History

| ts | standing | branch+SHA | gates+exits | remaining |
|----|----------|------------|-------------|-----------|
| 2026-09-21 | OPEN | feat/155-r2@c55d4f2 | not started | all of DoD |
