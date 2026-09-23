---
id: TR1
dcterms:title: "PR CI triage + workflow fixes"
dcterms:created: "2026-09-21"
dcterms:isPartOf: "v26.9.21"
rdf:type: oslc_cm:ChangeRequest, earl:TestRequirement
aps:standing: https://w3id.org/chatman/aps#PARTIAL_ALIVE
milestone: frozen-duckdb v1.5.5 wave 3 (hardening, pre-publish)
worktree: /Users/sac/frozen-duckdb-wt/r1
branch: feat/155-r1
---

# TR1 — PR CI triage + workflow fixes

## Scope (files you may change)

```
.github/workflows/{ci.yml,ci-simple.yml,test-minimal.yml}
```

Anything outside scope: do not edit — record BLOCKED with the reason instead.

## Work order


Context: draft PR #3 (feat/duckdb-1.5.5) triggers ci.yml on ubuntu+macOS. Check its live status: gh api repos/seanchatmangpt/frozen-duckdb/actions/runs?branch=feat/duckdb-1.5.5 and .../commits/<sha>/check-runs (gh 2.54: use gh api, NOT gh pr checks -R). Diagnose any red run from its logs (gh api .../actions/runs/<id>/logs?job_id=... or jobs listing). Likely hazards: ubuntu jobs have no prebuilt .so (builder falls back to a long local compile — may time out), setup_env.sh assumes macOS dylib names.
DoD: [ ] PR #3 check status recorded (run ids + conclusions)  [ ] red causes fixed within workflow scope or explicitly documented as builder-owned with the fix specified  [ ] yaml-parse all touched workflows  [ ] committed

## Contract (all tickets)

- Work ONLY in `/Users/sac/frozen-duckdb-wt/r1` on `feat/155-r1`. NEVER touch the main checkout /Users/sac/frozen-duckdb or sibling worktrees. Never push, never touch remotes, never publish.
- Atomic commits, conventional messages, on your branch only.
- Verify before claiming: preserve command + exit. A gate you did not run is UNKNOWN, not green.
- Append every transition to the History table below (the one sanctioned write outside your worktree), then STOP — the coordinator merges.

## History

| ts | standing | branch+SHA | gates+exits | remaining |
|----|----------|------------|-------------|-----------|
| 2026-09-21 | OPEN | feat/155-r1@c55d4f2 | not started | all of DoD |
| 2026-09-21 | PARTIAL_ALIVE | feat/155-r1@57a2e01 | gh api triage PR#3 (3 workflows, 2 heads, 9 runs): see evidence block below; yaml-parse 3/3 OK (pyyaml); bash -n preseed OK; preseed executed live, macOS+linux branches, scratch HOMEs → builder-native cache layout reproduced; cargo fmt --all -- --check exit 0; cargo clippy --all-targets --all-features -- -D warnings exit 0; cargo build exit 0; cargo clean -p frozen-duckdb-sys + rebuild vs scratch preseeded cache exit 0 (warning path proves cache hit); cargo run --example basic_usage exit 0 (1000 real queries) | CI verdict on pushed head (coordinator pushes; agent never pushes); beta clippy -D warnings never observed passing (beta legs died at fmt first) = UNKNOWN; macos-latest tests/examples on a real GH runner unobserved (local arm64 execution verified); performance-job wall time on 2-core release build unobserved |

### TR1 evidence — PR #3 live check status (recorded 2026-09-21T22:2x–22:4xZ, repo seanchatmangpt/frozen-duckdb, all `pull_request`, head_branch feat/duckdb-1.5.5)

| workflow | run id | head | conclusion |
|---|---|---|---|
| CI | 35662742597 | d655ce9 | failure (Build Examples 106541521252 fail; Performance Tests 106541521482 fail; Test Suite macos/beta 106541521521 fail at Check formatting; ubuntu/beta 106541521529 fail at Check formatting; ubuntu/stable 106541521449 + macos/stable 106541521717 cancelled by fail-fast) |
| CI Simple (for act testing) | 35662742583 | d655ce9 | failure (Test Suite 106541520975 fail at Run clippy — sys build.rs panic; fmt passed on stable) |
| Test Minimal | 35662742564 | d655ce9 | failure (Basic Test 106541520966 fail at Build project — sys build.rs panic) |
| CI | 35662364875 | c55d4f2 | failure (ubuntu/beta fail at Check formatting: "'cargo-fmt' is not installed for the toolchain 'beta'"; Performance fail, Build Examples fail — sys build.rs panic; 3 legs cancelled) |
| CI Simple / Test Minimal | 35662365037 / 35662364941 | c55d4f2 | failure (same causes) |
| CI / CI Simple / Test Minimal | 35662331302 / 35662331242 / 35662331241 | feb5d88 | failure (same causes) |

Root causes (all logs pulled via `gh api .../actions/jobs/<id>/logs`): (1) beta toolchain lacks rustfmt/clippy (dtolnay action installs neither by default) → fmt step red → fail-fast cancelled the rest; (2) `frozen-duckdb-sys` build.rs:26 panic "Failed to get frozen DuckDB binary: Failed to compile DuckDB locally … Could not find built library" on every ubuntu job: prebuilt/ ships no binaries, builder's release download 404s (repo has zero releases), source-compile fallback ignores cmake/make exit codes and searches stale build paths.

### Builder-owned defects — fix specification for TR7/TR2 (all outside TR1 scope; workflows now route around 1–3)

1. `crates/frozen-duckdb-builder/src/lib.rs` `compile_duckdb_locally`: git clone, cmake, make all use `.output()` and never check `.status.success()` — failures surface later as the misleading "Failed to find built library". Check exit codes; carry stdout/stderr into the error.
2. Same function: `find_built_library` search paths are stale for DuckDB ≥1.x cmake output (must include `build/duckdb/libduckdb.*`); `make -j4` full DuckDB build is 30–90 min on GH runners.
3. Same crate: `check_prebuilt_binary` + `download_from_github_release` hardcode `.dylib` — linux never resolves. Must use `.so` + linux assets on linux (upstream `libduckdb-linux-amd64.zip` exists, SONAME `libduckdb.so` — verified by download 2026-09-21); and `ensure_link_name` creates only `libduckdb.dylib`, which the linux linker never searches for `-lduckdb` — even a successful linux compile would fail at link time. Create `libduckdb.so` on linux.
4. `prebuilt/setup_env.sh`: (a) when sourced, `$0` is the caller's script — in GH Actions `run:` blocks `dirname "$(realpath "$0")"` resolves to the runner step-script dir; observed run 35662364875 job 106540322739 printing `Library: /home/runner/work/_temp`. Use `${BASH_SOURCE[0]}`. (b) Hardcoded `.dylib` names on all OSes; needs a linux `.so` branch. (c) Its `DUCKDB_LIB_DIR`/`DUCKDB_INCLUDE_DIR` exports are consumed by nothing — the builder reads neither env var; ci.yml/ci-simple.yml no longer source it (test-minimal.yml keeps it as an exists-and-executes smoke check only).
5. Observation (not a failure cause): post-job cleanup warns `fatal: No url found for submodule path 'vendors/duckdb-rs' in .gitmodules` on every job — repo carries a gitlink without a .gitmodules entry; owner: repo maintenance, not CI.

### What the operator did NOT have to write

All 164 inserted / 17 deleted workflow lines + this evidence block; zero app/hand-written 産面 lines. 比 for this ticket: 100% of delivered lines are tool-manufactured or workflow-scope edits committed on feat/155-r1@57a2e01; ledger deltas: none (no UNSUPPORTED rows needed — all changes inside declared TR1 scope).
