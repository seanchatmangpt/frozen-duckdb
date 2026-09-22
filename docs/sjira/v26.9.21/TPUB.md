---
id: TPUB
dcterms:title: "Dry-run publish pre-flight (final head)"
dcterms:created: "2026-09-21"
rdf:type: oslc_cm:ChangeRequest, earl:TestRequirement
aps:standing: https://w3id.org/chatman/aps#OPEN
milestone: frozen-duckdb v1.5.5 — pre-publish verification
worktree: /Users/sac/frozen-duckdb-wt/publish
branch: feat/155-publish
---

# TPUB — Dry-run publish pre-flight

## Scope

Read-only on the repo except: docs/sjira/v26.9.21/{PUBLISH_RUNBOOK.md (new), TPUB.md (History)} in your worktree.
No manifest edits, no real publish, no push.

## Work order

1. crates.io name/ownership pre-check: `curl -s https://crates.io/api/v1/crates/frozen-duckdb` (and -sys, -builder). If a crate exists, record owners + versions; if 404, record name-free. Publish is impossible if names are taken by others — this decides the release path.
2. `cargo publish --dry-run --allow-dirty -p frozen-duckdb-builder` → expected PASS.
3. `-p frozen-duckdb-sys`, `-p frozen-duckdb`: expected ordering failures (path+version deps resolve from registry). For each, run the T8-style replica-build compensation (extract `cargo package` output dir, build from the packaged tree) and record exits.
4. `cargo package --list` audits for all three: vendored-headers present (builder), readme present, no stray large files; record .crate sizes vs the 10 MiB cap.
5. docs.rs rehearsal: `HOME=<isolated> DOCS_RS=1 cargo build -p frozen-duckdb-sys` per the TR4 pattern → exit 0.
6. Render PUBLISH_RUNBOOK.md: the exact operator sequence (merge PR #3 → tag v1.5.5 push → CI asset verify via gh api → publish builder, index-wait, sys, main → issue comment with ISSUE_1_COMMENT.md), including failure handling at each step and the crates.io name-check findings from (1).

## History

| ts | standing | branch+SHA | gates+exits | remaining |
|----|----------|------------|-------------|-----------|
| 2026-09-21 | OPEN | feat/155-publish@<head> | not started | all of DoD |
| 2026-09-21 | ALIVE | feat/155-publish@4c41d83 | (1) NAME-CHECK GO/GO/GO: `curl https://crates.io/api/v1/crates/{frozen-duckdb,frozen-duckdb-sys,frozen-duckdb-builder}` → 200/404/404; sparse index `index.crates.io/fr/oz/*` → 200/404/404; frozen-duckdb exists at **0.1.0 only** (published 2025-10-07, sole owner **seanchatmangpt** = this repo owner via `/owners`, repository URL matches, not yanked, 2,332 downloads, dep `duckdb ^1.4.0` = old scheme) → 0.1.0→1.5.5 is a legal same-owner version jump, no takeover needed; -sys/-builder name-free. (2) `cargo publish --dry-run --allow-dirty -p frozen-duckdb-builder` exit 0 (full verification build from unpacked `target/package/frozen-duckdb-builder-1.5.5`, upload aborted by dry-run). (3) `-p frozen-duckdb-sys` exit 101 and `-p frozen-duckdb` exit 101 (`no matching package named frozen-duckdb-{builder,sys} found` — ordering artifact, unchanged from T8); NEW: `cargo package --allow-dirty --no-verify` for sys/main ALSO exits 101 — packaging itself is index-blocked (resolution precedes packaging), so sys/main `.crate` files are not producible pre-publication; T8 replica compensation re-executed on final head: exact `cargo package --list` file sets + publish-rewritten manifests (inheritance flattened, path dep → version-only) + the real extracted builder tree, joined by `[patch.crates-io]` harness at /tmp/fdup-tpub/replica → `cargo build` exit 0, explicit 3-crate build exit 0, replica `frozen-duckdb-cli --help` exit 0. (4) AUDITS: `cargo package --list` ×3 exit 0 — `vendored-headers/{duckdb.h,duckdb.hpp}` in builder list, `README.md` in all three (sys/main resolve from workspace root via `readme.workspace = true`), no stray large files (largest: duckdb.hpp 2.0 MiB, duckdb.h 244 KiB, bundled bindgen 496+209 KiB); sizes vs 10 MiB cap: builder `.crate` **426.9 KiB observed** (437,096 B; cargo reported 8 files, 2.3 MiB unpacked), sys 748 KiB / main 944 KiB unpacked (computed from packaged file sets — `.crate` not producible, see (3)); all ≪ cap. (5) DOCS-RS REHEARSAL: `HOME=/tmp/fd-docs-home-tpub DOCS_RS=1 RUSTUP_HOME=/Users/sac/.rustup CARGO_HOME=/Users/sac/.cargo cargo build -p frozen-duckdb-sys` exit 0 — build script logged "docs.rs build: bindings generated from vendored headers … no library linked", isolated HOME left completely empty (0 `.frozen-duckdb` entries); falsifier: forced build-script rerun in normal mode re-engages the prebuilt-binary/linking path, exit 0 (DOCS_RS↔normal switch clean on final head). (6) `docs/sjira/v26.9.21/PUBLISH_RUNBOOK.md` rendered in worktree (operator cut sequence: merge PR #3 → tag v1.5.5+push → gh-api release-asset verify `libduckdb_{arm64,x86_64}.dylib` → publish builder → index-wait poll loop → sys → main → issue-1 comment with #{PR_NUMBER}→#3 → docs.rs observation → clean-machine smoke; failure handling per step incl. 409 semantics, index lag, missing-asset HOLD, docs.rs fix-forward/no-yank doctrine; name-check findings embedded), committed 4c41d83. No manifest edits needed — zero manifest defects surfaced. | none for TPUB; coordinator: merge feat/155-publish → operator cuts per PUBLISH_RUNBOOK.md |
