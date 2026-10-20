# DG3 dispatch receipt — v2.0.0-dev tag dispatch

Standing: **BLOCKED[DIRTY_TREE]** — no tag created, no push, no workflow run.

## Contract violated

The DG3 task contract requires `git status --porcelain | wc -l` == 0 before tagging
("if not, STOP and report — don't tag a dirty tree"). Measured:

```
$ git -C /Users/sac/frozen-duckdb status --porcelain | wc -l
15
```

## Subject at gate time

- Repo: /Users/sac/frozen-duckdb, branch master
- HEAD: `cbeb249 chore: remove u2-probe stub workspace member (U4 disclosed unblock; U2 landed)`
- Workspace version: 2.0.0-dev (`Cargo.toml:27` — confirmed)
- Builder guard: `crates/frozen-duckdb-builder` uses `version.workspace = true`, so it
  inherits 2.0.0-dev; workflow release job consumes `github.ref_name` directly
  (`.github/workflows/build-binaries.yml:341`), so a `v2.0.0-dev` tag ref would satisfy
  GH2's guard once the tree is clean.
- Workflow trigger: `tags:` filter present at `.github/workflows/build-binaries.yml:5`.

## Exact dirty state (15 entries, verbatim `git status --porcelain`)

```
 M Cargo.lock
 M MIGRATION_GUIDE.md
 M crates/frozen-duckdb/src/duckdb/appender/mod.rs
 M crates/frozen-duckdb/src/duckdb/types/serde_json.rs
 M crates/frozen-duckdb/tests/panic_hunter.rs
 M docs/tutorials/quickstart-native.md
 M scripts/build_linux_prebuilt.sh
 M scripts/cleanroom/linux_link_test.sh
 M scripts/doctor/opfs_persistence_court.mjs
 M warehouse-sim/economics/.evidence/latest
?? deny.toml
?? docs/sjira/v2.0.0-dev/receipts/USER-FINDINGS-U12.md
?? docs/sjira/v2.0.0-dev/receipts/USER-FINDINGS-U3.md
?? docs/sjira/v2.0.0-dev/receipts/USER-FINDINGS-U4.md
?? docs/sjira/v2.0.0-dev/receipts/USER-FINDINGS-U6.md
?? u2-probe/
```

## What was NOT done

- No `git tag v2.0.0-dev`; tag does not exist locally (never ran).
- No `git push origin v2.0.0-dev`; no build-binaries run triggered.
- No `gh run watch`, no `gh release view`, no asset verification.
- No destructive git: nothing staged, reset, checked out, or discarded. The dirty state
  is untouched and belongs to other lanes / the user.

## Unblock path

Commit (or have the coordinator commit) the 15 in-flight entries, then re-run DG3
preconditions. Note `deny.toml` and `u2-probe/` are untracked and may be intentionally
uncommitted; the coordinator should classify them before the tag.
