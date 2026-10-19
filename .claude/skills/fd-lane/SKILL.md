---
name: fd-lane
description: Join or coordinate a concurrent lane fan-out on the frozen-duckdb checkout — lane leases, CARGO_TARGET_DIR build roots, compile-freeze SLA (cargo), integration. Use when running or joining a multi-lane wave.
---

# frozen-duckdb lane protocol

One canonical checkout (/Users/sac/frozen-duckdb); agents are lanes with
disjoint file ownership (docs/sjira/v<version>/_LANES.md). Coordinator owns
all git transitions and commits per lane. Lanes never run git
state-changing commands.

## Lane setup

1. Claim your lane in `docs/sjira/v<version>/_LANES.md`; never touch
   another lane's files. Shared seams: record resolutions in the wave's
   RESOLUTIONS.md (or _LANES.md appendix), don't edit another lane's
   region.
2. Build isolation: `CARGO_TARGET_DIR=target-lane<N>` on every cargo
   invocation (build, test, clippy, bench). This is per-lane disk, not a
   repo copy — no worktrees, no shadow clones.
   ```sh
   CARGO_TARGET_DIR=target-lane6 cargo build
   CARGO_TARGET_DIR=target-lane6 cargo test
   ```
3. Disk is the scarce resource: each lane target dir duplicates
   dependencies. Kill unused lane roots promptly; never leave
   `target-lane*` in the tree at integration.

## In-flight laws

- No `git stash` — it sweeps other lanes' in-flight edits (checkout-global
  transition, not lane-local). Snapshot a file via
  `git show HEAD:path > /tmp/...` and swap back when needed.
- Compile-freeze SLA: an edit that breaks the shared `cargo build` (e.g.
  Cargo.toml/Cargo.lock or shared `src/` changes) is fixed or reverted by
  its owner within ~10 min; past that the owner reverts. Any lane may
  apply a minimal unblock fix to another lane's non-compiling file,
  disclosed in BOTH lanes' receipts.
- Census vs compile-abort: `cargo test` exit 1 with zero tests executed is
  a compile abort — report the broken file, not a failure count.
- Read-only verification lanes run real commands (`cargo build`,
  `cargo test`) and report actual output as receipts; no PASS claims
  without output.

## Integration (coordinator)

1. Delete every `target-lane*` directory BEFORE the final commit (a
   fan-out leaving new lane target dirs behind is an incomplete
   integration). Check `.gitignore` covers `target/` and `target-lane*/`.
2. Serialize: verify ladder (`cargo build` → `cargo test` → targeted
   integration tests) → per-lane atomic commits — `git commit -F <file>`,
   never `-m` (inline `-m` has caused silent corruption).
3. Wrap: per-lane receipts (exact diff, commands, exits, SHA) → unresolved
   findings → coordinator merge only on explicit request.
