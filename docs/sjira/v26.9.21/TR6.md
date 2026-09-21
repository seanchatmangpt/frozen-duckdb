---
id: TR6
dcterms:title: "Repo hygiene + 帳 ledger"
dcterms:created: "2026-09-21"
dcterms:isPartOf: "v26.9.21"
rdf:type: oslc_cm:ChangeRequest, earl:TestRequirement
aps:standing: https://w3id.org/chatman/aps#OPEN
milestone: frozen-duckdb v1.5.5 wave 3 (hardening, pre-publish)
worktree: /Users/sac/frozen-duckdb-wt/r6
branch: feat/155-r6
---

# TR6 — Repo hygiene + 帳 ledger

## Scope (files you may change)

```
root rot files; scripts/{smoke*,fake_guard.sh,demo*}; HANDWRITTEN.md (new)
```

Anything outside scope: do not edit — record BLOCKED with the reason instead.

## Work order


Context: kcura-era rot at root (A6 audit): demo-dependency-issue.sh, test_architecture.rs, test_ffi_validation.rs, datasets/, test_datasets/, scripts/ smoke_go/smoke_node/smoke_py/fake_guard.sh family and ci_gate.sh/ci_gates.sh referencing non-existent crates/kcura-* paths. Triage each: still-useful → fix and keep; dead → git rm with the removal reasoned in the commit message; uncertain → keep + mark with a header comment. Then create HANDWRITTEN.md (帳): table path | semantic element | missing capability | intended owner pack | date — seed it with this wave's known agent-authored-by-hand rows (tickets T2A..TR6 non-generated repairs; the repo has no ggen pack projection).
DoD: [ ] every triaged file has a keep/fix/remove decision with evidence  [ ] repo still builds: cargo build --workspace exit 0  [ ] HANDWRITTEN.md exists with seeded rows  [ ] committed

## Contract (all tickets)

- Work ONLY in `/Users/sac/frozen-duckdb-wt/r6` on `feat/155-r6`. NEVER touch the main checkout /Users/sac/frozen-duckdb or sibling worktrees. Never push, never touch remotes, never publish.
- Atomic commits, conventional messages, on your branch only.
- Verify before claiming: preserve command + exit. A gate you did not run is UNKNOWN, not green.
- Append every transition to the History table below (the one sanctioned write outside your worktree), then STOP — the coordinator merges.

## History

| ts | standing | branch+SHA | gates+exits | remaining |
|----|----------|------------|-------------|-----------|
| 2026-09-21 | OPEN | feat/155-r6@c55d4f2 | not started | all of DoD |
