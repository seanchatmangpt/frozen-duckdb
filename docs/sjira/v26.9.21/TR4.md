---
id: TR4
dcterms:title: "docs.rs survival"
dcterms:created: "2026-09-21"
dcterms:isPartOf: "v26.9.21"
rdf:type: oslc_cm:ChangeRequest, earl:TestRequirement
aps:standing: https://w3id.org/chatman/aps#OPEN
milestone: frozen-duckdb v1.5.5 wave 3 (hardening, pre-publish)
worktree: /Users/sac/frozen-duckdb-wt/r4
branch: feat/155-r4
---

# TR4 — docs.rs survival

## Scope (files you may change)

```
crates/frozen-duckdb-sys/build.rs; crates/frozen-duckdb-sys/Cargo.toml
```

Anything outside scope: do not edit — record BLOCKED with the reason instead.

## Work order


Context: docs.rs builds on Linux where no .so asset exists → build.rs links (or compiles for 30+ min) → docs fail. Implement the standard DOCS_RS pattern: detect env DOCS_RS in sys build.rs → generate bindings from the vendored headers (builder's vendored-headers via the normal flow) but SKIP linking (no rustc-link-lib, no rpath) so rustdoc can typecheck without the library. Add [package.metadata.docs.rs] to sys Cargo.toml if useful. Verify: DOCS_RS=1 cargo build -p frozen-duckdb-sys exit 0 in a HOME without any ~/.frozen-duckdb cache (isolate with HOME=/tmp/fd-docs-home), and plain cargo build still links normally afterwards.
DoD: [ ] DOCS_RS=1 isolated build exit 0  [ ] normal build still links (CLI runs)  [ ] committed

## Contract (all tickets)

- Work ONLY in `/Users/sac/frozen-duckdb-wt/r4` on `feat/155-r4`. NEVER touch the main checkout /Users/sac/frozen-duckdb or sibling worktrees. Never push, never touch remotes, never publish.
- Atomic commits, conventional messages, on your branch only.
- Verify before claiming: preserve command + exit. A gate you did not run is UNKNOWN, not green.
- Append every transition to the History table below (the one sanctioned write outside your worktree), then STOP — the coordinator merges.

## History

| ts | standing | branch+SHA | gates+exits | remaining |
|----|----------|------------|-------------|-----------|
| 2026-09-21 | OPEN | feat/155-r4@c55d4f2 | not started | all of DoD |
