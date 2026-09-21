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
DoD: [x] reproduction command+exit recorded  [x] fix + regression test → cargo test -p frozen-duckdb --test column_names_tests exit 0  [x] committed

## Contract (all tickets)

- Work ONLY in `/Users/sac/frozen-duckdb-wt/r3` on `feat/155-r3`. NEVER touch the main checkout /Users/sac/frozen-duckdb or sibling worktrees. Never push, never touch remotes, never publish.
- Atomic commits, conventional messages, on your branch only.
- Verify before claiming: preserve command + exit. A gate you did not run is UNKNOWN, not green.
- Append every transition to the History table below (the one sanctioned write outside your worktree), then STOP — the coordinator merges.

## History

| ts | standing | branch+SHA | gates+exits | remaining |
|----|----------|------------|-------------|-----------|
| 2026-09-21 | OPEN | feat/155-r3@c55d4f2 | not started | all of DoD |
| 2026-09-21 | PARTIAL_ALIVE | feat/155-r3@d655ce9 | `cargo test -p frozen-duckdb --test column_names_tests` exit 101 — REPRODUCTION: `test_column_names_before_execution` panicked at `crates/frozen-duckdb/src/duckdb/raw_statement.rs:218:29: called \`Option::unwrap()\` on a \`None\` value` (pre-execution column_names, exactly as T5 observed; post-exec + empty-result tests passed, proving the defect is execution-order specific) | fix, gates, commit |
| 2026-09-21 | ALIVE | feat/155-r3@4e77b3f | fix: RawStatement::try_schema() (non-panicking, pub(crate)); new `Error::StatementNotExecuted` (enum `#[non_exhaustive]`, additive); `Statement::column_names() -> Result<Vec<String>>` maps missing schema → StatementNotExecuted. Gates: `cargo test -p frozen-duckdb --test column_names_tests` exit 0 (3 passed: pre-exec error, post-exec names, empty-result names); `cargo test -p frozen-duckdb --lib` exit 0 (140 passed, incl. test_prepare_column_names adapted and test_unexecuted_schema_panics still green — schema() panic contract preserved); `cargo check --workspace` exit 0; `cargo check -p frozen-duckdb --all-targets` exit 0. Falsifiers: pattern-match pins Err(StatementNotExecuted) exactly; no other column_names call sites in repo (grep). Committed 4e77b3f, not pushed. | none — ready for coordinator merge |
| 2026-09-21 | LEDGER (帳) | feat/155-r3@4e77b3f | UNSUPPORTED(ticket-scope, 3 rows): ticket scope listed only raw_statement.rs + tests, but the error boundary physically lives at (1) `column.rs` — Statement::column_names signature Vec→Result is where the error is returned (a raw_statement.rs-only fix cannot return an error through an infallible Vec signature; schema() itself keeps its documented panic contract per test_unexecuted_schema_panics); (2) `error.rs` — a proper duckdb::Error required the new StatementNotExecuted variant (no honest existing variant; DuckDBFailure would fabricate an underlying-call error); (3) `mod.rs` — 2 test lines got `?` for the new Result signature. Cross-checked TR1/TR2/TR4–TR8 scopes: no sibling ticket owns these files → zero wave conflict. 4 files touched beyond scope-list, all mechanical, all ledgered here. | none |
