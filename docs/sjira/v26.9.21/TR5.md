---
id: TR5
dcterms:title: "test-validation made real"
dcterms:created: "2026-09-21"
dcterms:isPartOf: "v26.9.21"
rdf:type: oslc_cm:ChangeRequest, earl:TestRequirement
aps:standing: https://w3id.org/chatman/aps#OPEN
milestone: frozen-duckdb v1.5.5 wave 3 (hardening, pre-publish)
worktree: /Users/sac/frozen-duckdb-wt/r5
branch: feat/155-r5
---

# TR5 — test-validation made real

## Scope (files you may change)

```
test-validation/**
```

Anything outside scope: do not edit — record BLOCKED with the reason instead.

## Work order


Context: A6 audit: test-validation/src/main.rs prints ❌/✅ but never asserts — it validates nothing (its cache-path pins were bumped but the harness is decorative). Make it a real gate: assert binary exists at builder's expected path (read VERSION from the builder crate, not a local hardcode), assert duckdb_library_version() via the FFI equals v1.5.5-derived expectation, assert header layout (duckdb/duckdb.h present), exit nonzero on any failure. Keep it a workspace member; run it: cargo run -p test-validation → exit 0.
DoD: [ ] harness asserts, no decorative prints  [ ] cargo run -p test-validation exit 0  [ ] committed

## Contract (all tickets)

- Work ONLY in `/Users/sac/frozen-duckdb-wt/r5` on `feat/155-r5`. NEVER touch the main checkout /Users/sac/frozen-duckdb or sibling worktrees. Never push, never touch remotes, never publish.
- Atomic commits, conventional messages, on your branch only.
- Verify before claiming: preserve command + exit. A gate you did not run is UNKNOWN, not green.
- Append every transition to the History table below (the one sanctioned write outside your worktree), then STOP — the coordinator merges.

## History

| ts | standing | branch+SHA | gates+exits | remaining |
|----|----------|------------|-------------|-----------|
| 2026-09-21 | OPEN | feat/155-r5@c55d4f2 | not started | all of DoD |
| 2026-09-21 | ALIVE | feat/155-r5@770f88c | `cargo run -p test-validation` exit 0 (7 gates held: builder path, binary presence, builder-derived version v1.5.5, layout rule, header duckdb/duckdb.h, dylib FFI duckdb_library_version()=v1.5.5, arch agreement); failure paths proven: `HOME=/dev/null ./target/debug/test-validation` exit 1 ("Failed to create cache directory"), `env -u HOME ./target/debug/test-validation` exit 1 ("HOME environment variable not set") | none — DoD complete; coordinator merges |

DoD: [x] harness asserts, no decorative prints — version DERIVED from builder (`ensure_binary()` path `v{VERSION}-{arch}` parsed; builder crate untouched, scope held to `test-validation/**`) [x] cargo run -p test-validation exit 0 [x] committed (770f88c)
