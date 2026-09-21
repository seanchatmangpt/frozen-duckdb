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
