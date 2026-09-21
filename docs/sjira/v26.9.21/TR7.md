---
id: TR7
dcterms:title: "Linux + multi-arch builder story"
dcterms:created: "2026-09-21"
dcterms:isPartOf: "v26.9.21"
rdf:type: oslc_cm:ChangeRequest, earl:TestRequirement
aps:standing: https://w3id.org/chatman/aps#OPEN
milestone: frozen-duckdb v1.5.5 wave 3 (hardening, pre-publish)
worktree: /Users/sac/frozen-duckdb-wt/r7
branch: feat/155-r7
---

# TR7 — Linux + multi-arch builder story

## Scope (files you may change)

```
crates/frozen-duckdb-builder/src/lib.rs; prebuilt/setup_env.sh; scripts/download_duckdb_binaries.sh
```

Anything outside scope: do not edit — record BLOCKED with the reason instead.

## Work order


Context: builder is macOS-only by construction (downloads .dylib names; get_binary_path uses target_os extension so Linux wants libduckdb_{arch}.so — but the download URL still fetches .dylib). Make the next release multi-platform: (1) builder: platform-aware asset name (.dylib on macOS, .so on Linux) in the download URL + get_binary_path already handles extension — align them; honor CMAKE_OSX_ARCHITECTURES env in compile_duckdb_locally (TR2's workflow sets it per matrix arch); keep Windows on the documented local-compile fallback (no .dll work). (2) setup_env.sh: real Linux branch (LD_LIBRARY_PATH + .so symlink names). (3) download_duckdb_binaries.sh: align linux naming. NOTE: release assets for .so do not exist until the NEXT tag — document that in the builder comments and ticket History (the v1.5.5 tag ships macOS assets per current workflow; TR2 coordinates the workflow side).
DoD: [ ] platform-aware naming unit-consistent (review + cargo test -p frozen-duckdb-builder exit 0)  [ ] bash -n setup_env.sh + download script  [ ] committed

## Contract (all tickets)

- Work ONLY in `/Users/sac/frozen-duckdb-wt/r7` on `feat/155-r7`. NEVER touch the main checkout /Users/sac/frozen-duckdb or sibling worktrees. Never push, never touch remotes, never publish.
- Atomic commits, conventional messages, on your branch only.
- Verify before claiming: preserve command + exit. A gate you did not run is UNKNOWN, not green.
- Append every transition to the History table below (the one sanctioned write outside your worktree), then STOP — the coordinator merges.

## History

| ts | standing | branch+SHA | gates+exits | remaining |
|----|----------|------------|-------------|-----------|
| 2026-09-21 | OPEN | feat/155-r7@c55d4f2 | not started | all of DoD |
