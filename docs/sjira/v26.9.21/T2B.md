---
id: T2B
dcterms:title: "Extension test suite green/gated"
dcterms:created: "2026-09-21"
dcterms:isPartOf: "v26.9.21"
rdf:type: oslc_cm:ChangeRequest, earl:TestRequirement
aps:standing: https://w3id.org/chatman/aps#OPEN
milestone: frozen-duckdb v1.5.5 (DuckDB 1.5.5 support, closes seanchatmangpt/frozen-duckdb#1)
worktree: /Users/sac/frozen-duckdb-wt/tests-ext
branch: feat/155-tests-ext
---

# T2B — Extension test suite green/gated

## Scope (files you may change)

```
crates/frozen-duckdb/tests/{arrow_tests,parquet_tests,polars_tests,vss_tests,tpch_integration_test,flock_tests}.rs
```

Anything outside scope: do not edit — record BLOCKED with the reason instead.

## Work order


Context: fix 'use duckdb::' → 'use frozen_duckdb::'. polars_tests.rs imports a polars crate that is not a dependency → gate whole file with #![cfg(feature = "polars")] + UNSUPPORTED(polars dev-dep not carried) note; do NOT add polars. flock_tests.rs needs community extension flock + live Ollama → runtime gate: early-return SKIP unless env FLOCK_TEST=1. tpch/parquet: official 1.5.5 dylib ships parquet statically; tpch needs network INSTALL — attempt real run; if network-blocked, env-gate DUCKDB_NET_TESTS=1 with default-on and note. vss probes soft-skip by design — keep.
DoD: [ ] cargo test -p frozen-duckdb --test arrow_tests --test parquet_tests --test vss_tests --test tpch_integration_test --test flock_tests → exit 0 (gated skips documented in History)  [ ] committed

## Contract (all tickets)

- Work ONLY in `/Users/sac/frozen-duckdb-wt/tests-ext` on `feat/155-tests-ext`. NEVER touch the main checkout /Users/sac/frozen-duckdb or sibling worktrees. Never push, never touch remotes.
- Atomic commits, conventional messages (`test:`, `fix:`, `docs:`, `ci:`, `build:`), on your branch only.
- 流: read this ticket → 階 (reuse what exists) → edit → verify (run the DoD gates) → 証 (append History) → stop.
- Standing vocabulary: ALIVE | BLOCKED | BUILD_BROKEN | PARTIAL_ALIVE | REFUSED_* | UNKNOWN | UNSUPPORTED. Preserve every command + exit code. inspection ≠ execution: a gate you did not run is not green.
- Shared resources: ~/.frozen-duckdb/cache (read-mostly; T7 owns its v1.5.5-arm64 mutation), crates.io registry, network. Coordinate through the ticket, never by touching sibling trees.
- Update the History table on every transition, then STOP — the coordinator merges.

## History

| ts | standing | branch+SHA | gates+exits | remaining |
|----|----------|------------|-------------|-----------|
| 2026-09-21 | OPEN | feat/155-tests-ext@c8e5788 | not started | all of DoD |
