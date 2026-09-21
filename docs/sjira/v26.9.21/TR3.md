---
id: TR3
dcterms:title: "column_names() pre-execution panic"
dcterms:created: "2026-09-21"
dcterms:isPartOf: "v26.9.21"
rdf:type: oslc_cm:ChangeRequest, earl:TestRequirement
aps:standing: https://w3id.org/chatman/aps#OPEN
milestone: frozen-duckdb v1.5.5 wave 3 (hardening, pre-publish)
worktree: /Users/sac/frozen-duckdb-wt/r3
branch: feat/155-r3
---

# TR3 — column_names() pre-execution panic

## Scope (files you may change)

```
crates/frozen-duckdb/src/duckdb/raw_statement.rs; crates/frozen-duckdb/tests/column_names_tests.rs (new)
```

Anything outside scope: do not edit — record BLOCKED with the reason instead.

## Work order


Context: T5 observed raw_statement.rs:218 schema unwrap panics when column_names() is called before statement execution. Reproduce first (falsify before fixing), then fix (proper error, not panic), then add a regression test file covering: pre-execution column_names, post-execution column_names, empty-result column_names.
DoD: [ ] reproduction command+exit recorded  [ ] fix + regression test → cargo test -p frozen-duckdb --test column_names_tests exit 0  [ ] committed

## Contract (all tickets)

- Work ONLY in `/Users/sac/frozen-duckdb-wt/r3` on `feat/155-r3`. NEVER touch the main checkout /Users/sac/frozen-duckdb or sibling worktrees. Never push, never touch remotes, never publish.
- Atomic commits, conventional messages, on your branch only.
- Verify before claiming: preserve command + exit. A gate you did not run is UNKNOWN, not green.
- Append every transition to the History table below (the one sanctioned write outside your worktree), then STOP — the coordinator merges.

## History

| ts | standing | branch+SHA | gates+exits | remaining |
|----|----------|------------|-------------|-----------|
| 2026-09-21 | OPEN | feat/155-r3@c55d4f2 | not started | all of DoD |
