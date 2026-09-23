#!/bin/bash
# gen_sjira_tickets.sh — render docs/sjira/<milestone> ticket scaffolding.
#
# Tickets are MANUFACTURED (票): the spec table below is the ontology, this
# script is the projection. Edit the spec, re-run, never hand-edit rendered
# tickets (transitions belong in each ticket's History section).
#
# Usage: scripts/gen_sjira_tickets.sh [milestone]   (default v26.9.21)
set -euo pipefail

MILESTONE="${1:-v26.9.21}"
DATE="$(date +%Y-%m-%d)"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$REPO_ROOT/docs/sjira/$MILESTONE"
mkdir -p "$OUT"

# ticket_id|title|worktree|branch|scope|body
SPECS=(
"T2A|Core test suite green|/Users/sac/frozen-duckdb-wt/tests-core|feat/155-tests-core|crates/frozen-duckdb/tests/{frozen_duckdb_tests,dropin_compatibility_tests,core_functionality_tests}.rs; crates/frozen-duckdb/src/duckdb/test_all_types.rs; minor src/duckdb/* compile fixes|
Context: tests/ are integration tests — 'crate::' paths are illegal there; use frozen_duckdb::. test_all_types.rs moved from lib-cfg-test to tests/: fix paths, then run it against DuckDB 1.5.5 and repair the EXCLUDE list / match fallthrough (todo!() panics on unknown columns; 1.5.5 may add GEOMETRY/VARIANT) and any golden drift, empirically driven by failures. dev-dep pretty_assertions is already declared.
DoD: [ ] cargo test -p frozen-duckdb --lib → exit 0  [ ] cargo test -p frozen-duckdb --test frozen_duckdb_tests --test dropin_compatibility_tests --test core_functionality_tests → exit 0 (or documented #[ignore] + UNSUPPORTED row in History)  [ ] all fixes committed atomically on feat/155-tests-core"

"T2B|Extension test suite green/gated|/Users/sac/frozen-duckdb-wt/tests-ext|feat/155-tests-ext|crates/frozen-duckdb/tests/{arrow_tests,parquet_tests,polars_tests,vss_tests,tpch_integration_test,flock_tests}.rs|
Context: fix 'use duckdb::' → 'use frozen_duckdb::'. polars_tests.rs imports a polars crate that is not a dependency → gate whole file with #![cfg(feature = \"polars\")] + UNSUPPORTED(polars dev-dep not carried) note; do NOT add polars. flock_tests.rs needs community extension flock + live Ollama → runtime gate: early-return SKIP unless env FLOCK_TEST=1. tpch/parquet: official 1.5.5 dylib ships parquet statically; tpch needs network INSTALL — attempt real run; if network-blocked, env-gate DUCKDB_NET_TESTS=1 with default-on and note. vss probes soft-skip by design — keep.
DoD: [ ] cargo test -p frozen-duckdb --test arrow_tests --test parquet_tests --test vss_tests --test tpch_integration_test --test flock_tests → exit 0 (gated skips documented in History)  [ ] committed"

"T3|Examples compile + run|/Users/sac/frozen-duckdb-wt/examples|feat/155-examples|crates/frozen-duckdb/examples/{basic_usage,performance_comparison}.rs|
Context: 'use duckdb::' rot. Fix to frozen_duckdb::, build all examples, then EXECUTE basic_usage against the frozen 1.5.5 dylib (execution evidence, not inspection). performance_comparison may be slow — cap with timeout 300.
DoD: [ ] cargo build --workspace --examples → exit 0  [ ] cargo run --example basic_usage → exit 0, output in History  [ ] cargo run --example performance_comparison → exit 0 or BLOCKED with preserved output"

"T4|Docs accuracy final pass|/Users/sac/frozen-duckdb-wt/docs-final|feat/155-docs-final|*.md, docs/**/*.md|
Context: verify every CURRENT-behavior claim against implemented reality: release asset names libduckdb_{arch}.dylib; vendored 1.5.5 headers in frozen-duckdb-builder (offline builds); @rpath propagation (no DYLD_LIBRARY_PATH needed for bins/tests); universal macOS dylib serving arm64+x86_64; local-compile fallback pinned v1.5.5; crate version 1.5.5 mirrors DuckDB 1.5.5. Grep-anchor each claim in code before asserting it. Historical records (CONTEXT.md phases, ADRs) stay untouched.
DoD: [ ] grep -rn '1\\.4\\.0' --include='*.md' → only intentional-historical hits, each with justification in History  [ ] README 'DuckDB 1.5 Support' section matches final reality incl. issue #1 answered  [ ] committed"

"T5|CI parity: clippy/fmt/workflows|/Users/sac/frozen-duckdb-wt/ci-parity|feat/155-ci-parity|.github/workflows/*.yml; targeted allow-attributes under crates/frozen-duckdb/src/duckdb/|
Context: ci.yml gates are fmt --check, clippy --all-targets --all-features -D warnings, build, test, examples. Make those pass locally. Prefer minimal targeted #[allow] at vendored module roots (comment: vendored from duckdb-rs 1.4.0-era; do not churn) over broad lint suppression; cargo fmt the workspace if the diff is mechanical. build-binaries.yml was fixed on feat/155-ci (asset names libduckdb_{arch}.dylib) — re-verify against the new vendored-headers builder reality and ci-simple/test-minimal coherence. yaml-parse every workflow you touch.
DoD: [ ] cargo clippy --workspace --all-targets --all-features -- -D warnings → exit 0 (or documented ci.yml amendment)  [ ] cargo fmt --check → exit 0  [ ] workflows yaml-parse  [ ] committed"

"T6|Scripts repair: fiction + float pins|/Users/sac/frozen-duckdb-wt/scripts-fix|feat/155-scripts-fix|scripts/**.sh|
Context: create_frozen_setup.sh fetches libduckdb_{arch}.dylib assets that never existed upstream (fictional scheme) — rewrite to fetch from this repo's GitHub Releases (the real post-fix scheme) or remove the dead path with an UNSUPPORTED note. build_frozen_duckdb.sh clones duckdb-rs default branch unpinned → pin --branch v1.10505.0 (duckdb-rs crate version encoding DuckDB 1.5.5). Sweep remaining 1.4.x pins (create_frozen_setup.sh has v1.4.1). bash -n everything touched. Do NOT execute build scripts.
DoD: [ ] grep -rn 'v1\\.4' scripts/ → zero live pins  [ ] bash -n on all edited scripts → exit 0  [ ] committed"

"T7|Verification: cache normalization + rpath|/Users/sac/frozen-duckdb-wt/verify|feat/155-verify|NO repo edits; owns mutation of ~/.frozen-duckdb/cache/v1.5.5-arm64 (announce in History before mutating)|
Context: prove the builder's normalization and rpath propagation by execution. (1) Delete ~/.frozen-duckdb/cache/v1.5.5-arm64/duckdb/ and libduckdb.dylib → cargo build -p frozen-duckdb → assert both restored (vendored headers + symlink) and build exit 0. (2) otool -l target/debug/frozen-duckdb-cli | grep -A2 LC_RPATH shows the cache dir. (3) env -i HOME=$HOME PATH=/usr/bin:/bin <abs> frozen-duckdb-cli --help → exit 0 (clean-env dyld proof). (4) python3 ctypes duckdb_library_version() == v1.5.5 post-mutation. (5) cd test-dependency && cargo build → dual-engine attempt, record outcome (UNKNOWN acceptable, preserve output).
DoD: [ ] all five probes executed with command+exit recorded in History  [ ] standing declared"

"T8|Release preflight + comms artifacts|/Users/sac/frozen-duckdb-wt/release|feat/155-release|docs/sjira/v26.9.21/{PR_BODY.md,ISSUE_1_COMMENT.md,MILESTONE.md}; Cargo.toml packaging (read-only)|
Context: (1) cargo publish --dry-run in order: frozen-duckdb-builder, frozen-duckdb-sys, frozen-duckdb — fix nothing; record all three outcomes. Verify vendored headers ship: cargo package -p frozen-duckdb-builder --list | grep vendored-headers. Record package sizes (crates.io limit 10MiB). (2) Render PR_BODY.md — title 'feat: DuckDB 1.5.5 support — pin bump, release asset fix, API layer restoration (closes #1)' + Summary/Changes/Test-plan with [FILL] slots for cross-ticket evidence. (3) Render ISSUE_1_COMMENT.md — warm reply to @dfeyer (opened 2026-03-23: 'Did you plan to release a version with DuckDB 1.5?'): done, crate v1.5.5 tracks upstream v1.5.5, drop-in, automatic prebuilt download (macOS universal arm64+x86_64), vendored headers, honest note that Windows/Linux assets remain roadmap, #{PR_NUMBER} placeholder. (4) MILESTONE.md: mark DoD done-items and list operator cuts (merge, tag v1.5.5 push, release-asset verify, cargo publish, post comment).
DoD: [ ] three dry-runs executed + recorded  [ ] vendored-headers in package list  [ ] PR_BODY.md + ISSUE_1_COMMENT.md + MILESTONE.md rendered  [ ] committed"
)
# Standing column (FDDB-26922-03): each ticket's frontmatter aps:standing is
# rendered from its FINAL History standing; new tickets default to OPEN.
standing_for() {
  case "$1" in
    T2A|T2B|T4|T5|T6|T8|TPUB) echo "ALIVE" ;;
    T3|T7) echo "PARTIAL_ALIVE" ;;
    TR2) echo "UNSUPPORTED" ;;
    *) echo "OPEN" ;;
  esac
}

render_ticket() {
  local id="$1" title="$2" wt="$3" branch="$4" scope="$5" body="$6" standing="${7:-OPEN}"
  cat > "$OUT/$id.md" <<EOF
---
id: $id
dcterms:title: "$title"
dcterms:created: "$DATE"
dcterms:isPartOf: "$MILESTONE"
rdf:type: oslc_cm:ChangeRequest, earl:TestRequirement
aps:standing: $(printf 'https://w3id.org/chatman/aps#%s' "$standing")
milestone: frozen-duckdb v1.5.5 (DuckDB 1.5.5 support, closes seanchatmangpt/frozen-duckdb#1)
worktree: $wt
branch: $branch
---

# $id — $title

## Scope (files you may change)

\`\`\`
$scope
\`\`\`

Anything outside scope: do not edit — record BLOCKED with the reason instead.

## Work order

$body

## Contract (all tickets)

- Work ONLY in \`$wt\` on \`$branch\`. NEVER touch the main checkout /Users/sac/frozen-duckdb or sibling worktrees. Never push, never touch remotes.
- Atomic commits, conventional messages (\`test:\`, \`fix:\`, \`docs:\`, \`ci:\`, \`build:\`), on your branch only.
- 流: read this ticket → 階 (reuse what exists) → edit → verify (run the DoD gates) → 証 (append History) → stop.
- Standing vocabulary: ALIVE | BLOCKED | BUILD_BROKEN | PARTIAL_ALIVE | REFUSED_* | UNKNOWN | UNSUPPORTED. Preserve every command + exit code. inspection ≠ execution: a gate you did not run is not green.
- Shared resources: ~/.frozen-duckdb/cache (read-mostly; T7 owns its v1.5.5-arm64 mutation), crates.io registry, network. Coordinate through the ticket, never by touching sibling trees.
- Update the History table on every transition, then STOP — the coordinator merges.

## History

| ts | standing | branch+SHA | gates+exits | remaining |
|----|----------|------------|-------------|-----------|
| $DATE | OPEN | $branch@$(cd "$REPO_ROOT" && git rev-parse --short feat/duckdb-1.5.5) | not started | all of DoD |
EOF
}

# MILESTONE.md
cat > "$OUT/MILESTONE.md" <<EOF
---
dcterms:title: "frozen-duckdb v1.5.5 — Definition of Done"
dcterms:created: "$DATE"
rdf:type: prov:Activity
aps:standing: $(printf 'https://w3id.org/chatman/aps#OPEN')
---

# MILESTONE $MILESTONE — frozen-duckdb v1.5.5 (DuckDB 1.5.5, closes issue #1)

## Done (coordinator, this session)

- [x] DuckDB v1.5.5 universal dylib + headers staged in ~/.frozen-duckdb/cache/v1.5.5-{arm64,x86_64}; ctypes + CLI verified v1.5.5
- [x] C API 1.4→1.5 diff: 0 removals / 0 renames / 0 layout changes; all 459 referenced symbols present (bindgen SAFE)
- [x] Pins bumped to 1.5.5: builder, scripts, CI defaults, docs, workspace version (waves feat/155-{crates,docs,scripts,ci}, merged)
- [x] CI release-asset naming fixed: assets now match ensure_binary() download URLs
- [x] Builder: vendored 1.5.5 headers, duckdb/ layout, libduckdb.dylib link name, cache normalization on every acquisition path
- [x] sys: bindgen double-include fixed; links=duckdb + DEP metadata
- [x] Vendored duckdb-rs API layer restored to compilation (mod wiring, crate:: paths, arrow 56 / hashlink 0.10 / strum 0.27 / fallible-* / rust_decimal / cast)
- [x] Runtime @rpath propagation via DEP_DUCKDB build script; CLI bin executes against 1.5.5 (dyld clean)
- [x] Workspace cargo build → exit 0

## Open (this wave — tickets T2A..T8)

$(printf '%s\n' "${SPECS[@]}" | cut -d'|' -f1,2 | sed 's/^/- [ ] /' | sed 's/|/ — /')

## Operator cuts (権 — agent-attempted only with fresh operator authority)

- [ ] Merge feat/duckdb-1.5.5 → master (coordinator opens draft PR; operator merges)
- [ ] git tag v1.5.5 && git push origin v1.5.5 → build-binaries.yml publishes libduckdb_{arm64,x86_64}.dylib
- [ ] Verify release assets + clean-machine download path (checklist in PR_BODY.md)
- [ ] cargo publish (builder → sys → frozen-duckdb), after version keys verified by T8 dry-runs
- [ ] Post ISSUE_1_COMMENT.md on issue #1 after publish
EOF

# _RUNBOOK.md — canonical dispatch form
cat > "$OUT/_RUNBOOK.md" <<EOF
# RUNBOOK — $MILESTONE dispatch contract (_RUNBOOK.md = canonical dispatch prompt form)

## Dispatch form (per agent)

1. Ticket path (this directory, e.g. docs/sjira/$MILESTONE/T2A.md)
2. Worktree path (inside the ticket; worktree pre-created by coordinator on the ticket's branch)
3. Nothing else. Session state lives in the ticket's History table, never in conversation.

## Laws in force

- 流 loop: parse → orient (this runbook + ticket) → 階 (reuse/compose/extend before inventing) → edit → verify → 証 → stop.
- 帳: any hand-written line outside a generator/rendered path on 産面 needs an UNSUPPORTED row in the ticket History.
- 器: generated/consequence files are edited at their source, never as projections.
- 並: one agent per worktree; shared mutable trees are serialized by the coordinator, never by agents.
- 証: ALIVE requires observed execution this session against the exact subject. Preserve command + exit. No acceptance mocks.
- 偽: attempt your own falsifiers before claiming DoD; a gate you skipped is UNKNOWN, not green.
- 権: no push, no publish, no tag, no release, no issue/PR mutation. Coordinator merges; operator cuts remain in MILESTONE.md.
- 延: any operating knowledge an agent uses must land in the repo (ticket, comment, or gate), not stay in the session.

## Coordinator merge order (serialized, --no-ff, mix of gates before each)

T5 (ci-parity) → T6 (scripts-fix) → T4 (docs-final) → T2A → T2B → T3 (examples) → T7 (verify) → T8 (release) — order may be re-sequenced by conflict evidence; conflicts resolve toward the branch whose scope owns the file (scope table in each ticket).
EOF

# render tickets
for spec in "${SPECS[@]}"; do
  id="${spec%%|*}";        rest="${spec#*|}"
  title="${rest%%|*}";     rest="${rest#*|}"
  wt="${rest%%|*}";        rest="${rest#*|}"
  branch="${rest%%|*}";    rest="${rest#*|}"
  scope="${rest%%|*}";     body="${rest#*|}"
  standing="$(standing_for "$id")"
  render_ticket "$id" "$title" "$wt" "$branch" "$scope" "$body" "$standing"
done

echo "rendered $(ls "$OUT" | wc -l | tr -d ' ') files into $OUT:"
ls "$OUT"
