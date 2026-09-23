---
id: T2B
dcterms:title: "Extension test suite green/gated"
dcterms:created: "2026-09-21"
dcterms:isPartOf: "v26.9.21"
rdf:type: oslc_cm:ChangeRequest, earl:TestRequirement
aps:standing: https://w3id.org/chatman/aps#ALIVE
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
| 2026-09-21 | BUILD_BROKEN (tests only; lib/bin green) | feat/155-tests-ext@f048742 | `cargo test -p frozen-duckdb --test arrow_tests --test parquet_tests --test polars_tests --test vss_tests --test tpch_integration_test --test flock_tests --no-run` → exit 1: all six targets E0432/E0433 `use of unresolved module or unlinked crate duckdb` | fix imports, gates |
| 2026-09-21 | PARTIAL_ALIVE | feat/155-tests-ext@worktree (pre-commit) | all six edited (`use frozen_duckdb::`, arrow via `frozen_duckdb::duckdb::arrow`); `--no-run` → exit 0. First real run: arrow 6/6 ok, flock 11/11 ok (SKIP gate), parquet 6/6 ok, vss deferred, **tpch_integration_test SIGSEGV (signal 11, cargo exit 101)** under default parallel harness | isolate tpch segfault |
| 2026-09-21 | PARTIAL_ALIVE | feat/155-tests-ext@worktree (pre-commit) | tpch falsification: each of 6 tests green with `--test-threads=1` and individually (exit 0 each); full binary single-threaded → 6/6 ok exit 0. Diagnosis: tpch extension `dbgen` carries process-global state, concurrent `CALL dbgen` across connections races → SIGSEGV. Fix: file-local `Mutex` serializing the six tests (all remain real runs) | re-run gate |
| 2026-09-21 | ALIVE | feat/155-tests-ext@1062719 | DoD gate `cargo test -p frozen-duckdb --test arrow_tests --test parquet_tests --test vss_tests --test tpch_integration_test --test flock_tests` → exit 0, **3 consecutive runs** + 1 at HEAD: arrow 6/6, parquet 6/6, vss 8/8, tpch 6/6 (real INSTALL/LOAD/dbgen/queries), flock 11/11 (SKIP-gated, verified 11 `SKIP:` prints). polars: `--test polars_tests` → 0 tests, exit 0 (compile-gated). Commits: ece47f0 fix(tests) rename; 6a75ecc test(polars) gate; 8711138 test(flock) FLOCK_TEST gate; 1062719 fix(tests) tpch dbgen mutex | none — ready for coordinator merge |
| 2026-09-21 | UNSUPPORTED ledger | feat/155-tests-ext@1062719 | UNSUPPORTED(frozen-duckdb extension test suites, polars coverage): ported crate carries no polars feature/dev-dep; `polars_tests.rs` compile-gated behind `feature = "polars"` (UNSUPPORTED note in-file). Intended owner: future pack/fact adding a polars feature to the vendored API layer. Hand-written lines on 産面 this ticket: test-file edits only, all inside ticket scope; operator wrote 0 | — |
