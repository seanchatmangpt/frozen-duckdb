# frozen-duckdb migration plan v26.11.16

Standing: UNVERIFIED

## Identity
- Path: /Users/sac/frozen-duckdb
- Remote: https://github.com/seanchatmangpt/frozen-duckdb.git (upstream origin/master)
- Branch: master@87f1daa; last commit 2026-10-29; 273 commits; 2209 MB
- Languages/build: Rust workspace (Cargo.toml, crates/, c_api/), justfile, Makefile,
  ggen.toml/.ggen/.ggen-v2, deny.toml. CLAUDE.md yes. Published on crates.io (v2.0.4).
- Nested repos: submodule vendors/duckdb-rs (NOT initialised: "-f4fe688a", dir 0 B)

## Observed state
- Dirty 0 / untracked 0. ahead 0 / behind 0. Clean master.
- STASH COUNT 140 (census). Branch preserve/v26.9.22/stash-0 exists as a stash-preservation
  branch ("WIP on feat/duckdb-1.5.5").
- Unpushed local branches (2): feat/duckdb-1.5.5 (no upstream),
  release/v26.9.22 (tracks origin/master, ahead 3 / behind 93).
  preserve/v26.9.22/stash-0 also has no upstream (3 local-only refs total).
- Tracked branches: docs/doc-hdit-scaffold, fix/tr7-arch-detection.
- Size 2.2 GB but no target/ at root; .cache 4.7 MB, attic/generated/c_api tiny. The bulk
  is probably .git objects or a target/ under crates/* — run `du -sh .git crates/*/target`
  before copying. Engine cache lives in ~/.frozen-duckdb/cache/ (outside repo).
- No .env/db at root. docs/ has sjira/ (not docs/jira).

## What it is
Drop-in replacement for the duckdb Rust crate: prebuilt DuckDB engine library cached after
first build so downstream builds link instead of compiling C++. Builder sub-crate pattern
(frozen-duckdb, frozen-duckdb-builder, frozen-duckdb-sys).

## Classification
ACTIVE (census agrees; v2.0.4 released 2026-10-29).

## Migration risk
MEDIUM. Working tree clean and pushed, but 140 stashes and 3 local-only refs
(feat/duckdb-1.5.5, release/v26.9.22 +3, preserve/v26.9.22/stash-0) would be lost on
re-clone. ~/.frozen-duckdb/cache/ is rebuildable.

## Recommended action
COMMIT-PUSH
1. `cd /Users/sac/frozen-duckdb && git push origin feat/duckdb-1.5.5 preserve/v26.9.22/stash-0`
2. `git push origin release/v26.9.22:release/v26.9.22` (its upstream is wrongly master).
3. DECISION: 140 stashes — bundle them rather than triage:
   `git bundle create /Volumes/archive/frozen-duckdb-all.bundle --all` then
   `git stash list > /Volumes/archive/frozen-duckdb-stashes.txt` (stash refs are not in
   --all; to keep them: `for i in $(seq 0 139); do git branch stash-$i stash@{$i}; done`
   before bundling, or accept loss).
4. `du -sh .git crates/*/target target 2>/dev/null`; `rm -rf target crates/*/target`.
5. Clone fresh on new machine; `git submodule update --init vendors/duckdb-rs` if needed.

## First commands on the new machine
`cargo check --workspace` then `just verify` / `make verify` (README: "make verify 6/6").

## See Also
- /Users/sac/frozen-duckdb/docs/sjira/ (previous campaign docs; no docs/jira dir)
- /Users/sac/docs/jira/v26.11.16/README.md
