---
id: TR8
dcterms:title: "Packaging polish + final consistency"
dcterms:created: "2026-09-21"
dcterms:isPartOf: "v26.9.21"
rdf:type: oslc_cm:ChangeRequest, earl:TestRequirement
aps:standing: https://w3id.org/chatman/aps#OPEN
milestone: frozen-duckdb v1.5.5 wave 3 (hardening, pre-publish)
worktree: /Users/sac/frozen-duckdb-wt/r8
branch: feat/155-r8
---

# TR8 — Packaging polish + final consistency

## Scope (files you may change)

```
crates/frozen-duckdb/Cargo.toml; crates/frozen-duckdb-builder/Cargo.toml; README.md; CHANGELOG.md
```

Anything outside scope: do not edit — record BLOCKED with the reason instead.

## Work order


Context: pre-publish polish. (1) Crate metadata: builder + main crate get description/readme/keywords sanity (sys is TR4's; do not touch it); ensure readme files referenced exist. (2) README: badges/links point at real URLs (repo-relative docs links must resolve on crates.io — use absolute GitHub URLs where needed); mention Linux/multi-arch honestly (TR7 may land after you — write 'macOS prebuilt assets; Linux via local compile (prebuilt .so planned)' unless TR7 evidence says otherwise at your run time). (3) CHANGELOG: add entries for the wave-3 hardening under [1.5.5] Unreleased-fixes subsection. (4) Final grep: no stale 1.4.0-as-current, no 1.10505 vs 1.5.5 inconsistencies in user-facing strings.
DoD: [ ] metadata complete (cargo package -p frozen-duckdb --list spot-check)  [ ] README claims grep-anchored  [ ] CHANGELOG updated  [ ] committed

## Contract (all tickets)

- Work ONLY in `/Users/sac/frozen-duckdb-wt/r8` on `feat/155-r8`. NEVER touch the main checkout /Users/sac/frozen-duckdb or sibling worktrees. Never push, never touch remotes, never publish.
- Atomic commits, conventional messages, on your branch only.
- Verify before claiming: preserve command + exit. A gate you did not run is UNKNOWN, not green.
- Append every transition to the History table below (the one sanctioned write outside your worktree), then STOP — the coordinator merges.

## History

| ts | standing | branch+SHA | gates+exits | remaining |
|----|----------|------------|-------------|-----------|
| 2026-09-21 | OPEN | feat/155-r8@c55d4f2 | not started | all of DoD |
