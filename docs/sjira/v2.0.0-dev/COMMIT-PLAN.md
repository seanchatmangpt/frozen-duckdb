# COMMIT-PLAN — v2.0.0-dev integration (prepared by lane Z3)

Census basis: `git status --porcelain` (read-only) on 2026-10-19.
Coordinator alone runs git. Commit messages pre-drafted in
`/tmp/z3_commit_msg_{a..f}.txt`; use `git commit -F <file>` (repo law —
never inline `-m`).

## Pre-flight (coordinator, before commit 1)

1. `.gitignore` additions land in commit (f), but must exist BEFORE any
   `git add -A`. Add to `.gitignore`:
   ```
   target-lane*/
   target-integration/
   warehouse-sim/node_modules/
   warehouse-sim/.next/
   libnull.rlib
   ```
   Simplest: commit (f) FIRST so the ignore rules protect every later add.
   (Note: `target/` is already ignored; `target-lane*/` is not — 24 lane
   build-root dirs are currently untracked-and-unignored.)
2. `libnull.rlib` — stray build artifact at repo root, NOT ignored today.
   Do not commit; ignored by (f).
3. `Cargo.lock` is gitignored in this repo (`.gitignore` matches it) even
   though the workspace has binaries. Coordinator decision point: keep it
   ignored (status quo) or un-ignore and commit. Default plan: leave ignored.
4. `docs/sjira/v2.0.0-dev/receipts/VERIFIER-RECEIPTS-LOG.md` is gitignored
   (`VERIFIER-RECEIPTS-LOG.md` rule). Receipt dir will commit as effectively
   empty unless the coordinator un-ignores that filename for the window.
   Default plan: leave as-is (log is runtime state).

## Ordered commit sequence

Each step: `git add <paths> && git commit -F /tmp/z3_commit_msg_X.txt`.

| # | msg file | scope | paths |
|---|---|---|---|
| 1 | `/tmp/z3_commit_msg_f.txt` | chore: gitignore + justfile + LICENSE draft + release-op agent | `.gitignore`, `justfile`, `LICENSE-ENTERPRISE-DRAFT.md`, `.claude/agents/release-op.md` |
| 2 | `/tmp/z3_commit_msg_a.txt` | feat!: 2.0.0-dev bump + VARIANT + doctor/gate tests | `Cargo.toml`, `crates/frozen-duckdb/`, `crates/frozen-duckdb-sys/`, `crates/frozen-duckdb-builder/`, `test-dependency/`, `test-validation/` |
| 3 | `/tmp/z3_commit_msg_b.txt` | chore(build): prebuilt + scripts + doctor entry | `prebuilt/`, `scripts/build_*.sh`, `scripts/create_*.sh`, `scripts/download_duckdb_binaries.sh`, `scripts/precompile_deps.sh`, `scripts/setup_*.sh`, `scripts/use_prebuilt_duckdb.sh`, `scripts/doctor.sh`, `scripts/doctor/` |
| 4 | `/tmp/z3_commit_msg_c.txt` | ci: workflows | `.github/workflows/{ci,ci-simple,test-minimal,build-binaries}.yml` |
| 5 | `/tmp/z3_commit_msg_d.txt` | chore(ggen): schemas + regenerated projections | `ggen.toml`, `schema/*.ttl`, `gates/`, `Makefile`, `.doc-surface.toml`, `scripts/verify-gates.sh`, `scripts/verify-evidence.ttl`, `CHANGELOG.md`, `docs/GENESIS.md` |
| 6 | `/tmp/z3_commit_msg_e.txt` | docs: design corpus + window + warehouse-sim source | `docs/design/`, `docs/sjira/v2.0.0-dev/` (minus gitignored receipts log), `warehouse-sim/` (minus ignored node_modules/.next), `README.md`, `QUICKSTART.md`, `MIGRATION_GUIDE.md`, `TROUBLESHOOTING.md`, `CONTEXT.md`, `CLAUDE.md`, `docs/api docs/architecture docs/cli docs/contributing docs/explanation docs/guides docs/how-to docs/meta.md docs/reference docs/testing` |

## MUST NOT be committed

- `target-lane*/` (24 dirs: A15,A18,A23,A30,BP12,D4-D8,F1-F10,V1,V2,V2a,V3,
  X3,X6,X8,X9,X16) — lane build roots; coordinator deletes at integration
  (fan-out cleanup law) BEFORE the final commit, then the gitignore rules in
  commit (f) keep them ignored.
- `target-integration/` — integration build root; delete at integration.
- `warehouse-sim/node_modules/`, `warehouse-sim/.next/` — deps/build output.
- `libnull.rlib` — stray artifact.
- `.ggen/`, `.ggen-v2/` — ggen runtime state incl. signing keypair (secrets
  law); already gitignored, never add.
- `Cargo.lock` — currently gitignored; leave ignored per default plan.
- `/tmp/z3_commit_msg_*.txt` — outside the repo; nothing to do.
- `docs/sjira/v2.0.0-dev/receipts/VERIFIER-RECEIPTS-LOG.md` — gitignored
  runtime log, leave ignored.
- NO `git push` in this plan; merge-only repo law, no rebase/force-push.

## Post-sequence checks (coordinator)

1. `git status --porcelain` must be empty (or only ignored files).
2. `make verify-gates` green after commit 5 (regenerated projections match
   sources; GENESIS check passes).
3. `cargo test --workspace` green (gate on commit 2).
4. Coordinator deletes `target-lane*/` + `target-integration/` before the
   final integration commit.

## DECISION (lane GAP7, 2026-10-19): Cargo.lock policy — commit both (supersedes "leave ignored" above)

Z3 gap closed. The workspace contains binaries (test-validation bin, repo
doctor tooling), so cargo's library-only Cargo.lock exception does not apply:
lockfiles are required for reproducible builds of consumer-facing artifacts.

- `Cargo.lock` (root) and `test-dependency/Cargo.lock` are both COMMITTED.
  Rationale: both ship bins; committing both gives consumers bit-reproducible
  dependency resolution end to end. `.gitignore` now negates both
  (`!Cargo.lock`, `!test-dependency/Cargo.lock`).
- Freshness receipt: `cargo metadata --format-version 1` exit 0; root
  Cargo.lock sha256 `0998510911111ec8d49ac35ee717e7a0265df00e17fdf4b0de7e200addd11720`
  unchanged by the refresh (already current).
- `.cargo-laneT1/` added to `.gitignore` (T1's lane-local CARGO_HOME lease).
- Commit group (a) gains `Cargo.lock` + `test-dependency/Cargo.lock`; the
  `.gitignore` edit rides the gitignore/rules commit it was already in.
- Coordinator MUST-NOT list updated: the `Cargo.lock — leave ignored` bullet
  is superseded by this decision.
