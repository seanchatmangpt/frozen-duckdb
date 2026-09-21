---
id: T2A
dcterms:title: "Core test suite green"
dcterms:created: "2026-09-21"
dcterms:isPartOf: "v26.9.21"
rdf:type: oslc_cm:ChangeRequest, earl:TestRequirement
aps:standing: https://w3id.org/chatman/aps#OPEN
milestone: frozen-duckdb v1.5.5 (DuckDB 1.5.5 support, closes seanchatmangpt/frozen-duckdb#1)
worktree: /Users/sac/frozen-duckdb-wt/tests-core
branch: feat/155-tests-core
---

# T2A — Core test suite green

## Scope (files you may change)

```
crates/frozen-duckdb/tests/{frozen_duckdb_tests,dropin_compatibility_tests,core_functionality_tests}.rs; crates/frozen-duckdb/src/duckdb/test_all_types.rs; minor src/duckdb/* compile fixes
```

Anything outside scope: do not edit — record BLOCKED with the reason instead.

## Work order


Context: tests/ are integration tests — 'crate::' paths are illegal there; use frozen_duckdb::. test_all_types.rs moved from lib-cfg-test to tests/: fix paths, then run it against DuckDB 1.5.5 and repair the EXCLUDE list / match fallthrough (todo!() panics on unknown columns; 1.5.5 may add GEOMETRY/VARIANT) and any golden drift, empirically driven by failures. dev-dep pretty_assertions is already declared.
DoD: [ ] cargo test -p frozen-duckdb --lib → exit 0  [ ] cargo test -p frozen-duckdb --test frozen_duckdb_tests --test dropin_compatibility_tests --test core_functionality_tests → exit 0 (or documented #[ignore] + UNSUPPORTED row in History)  [ ] all fixes committed atomically on feat/155-tests-core

## Contract (all tickets)

- Work ONLY in `/Users/sac/frozen-duckdb-wt/tests-core` on `feat/155-tests-core`. NEVER touch the main checkout /Users/sac/frozen-duckdb or sibling worktrees. Never push, never touch remotes.
- Atomic commits, conventional messages (`test:`, `fix:`, `docs:`, `ci:`, `build:`), on your branch only.
- 流: read this ticket → 階 (reuse what exists) → edit → verify (run the DoD gates) → 証 (append History) → stop.
- Standing vocabulary: ALIVE | BLOCKED | BUILD_BROKEN | PARTIAL_ALIVE | REFUSED_* | UNKNOWN | UNSUPPORTED. Preserve every command + exit code. inspection ≠ execution: a gate you did not run is not green.
- Shared resources: ~/.frozen-duckdb/cache (read-mostly; T7 owns its v1.5.5-arm64 mutation), crates.io registry, network. Coordinate through the ticket, never by touching sibling trees.
- Update the History table on every transition, then STOP — the coordinator merges.

## History

| ts | standing | branch+SHA | gates+exits | remaining |
|----|----------|------------|-------------|-----------|
| 2026-09-21 | OPEN | feat/155-tests-core@c8e5788 | not started | all of DoD |
