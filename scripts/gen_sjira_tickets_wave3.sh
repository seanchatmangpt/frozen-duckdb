#!/bin/bash
# gen_sjira_tickets_wave3.sh — render wave-3 hardening tickets into docs/sjira/<milestone>.
# Same law as gen_sjira_tickets.sh: spec table is the ontology, this renders. Re-run after edits.
set -euo pipefail

MILESTONE="${1:-v26.9.21}"
DATE="$(date +%Y-%m-%d)"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$REPO_ROOT/docs/sjira/$MILESTONE"
mkdir -p "$OUT"

SPECS=(
"TR1|PR CI triage + workflow fixes|/Users/sac/frozen-duckdb-wt/r1|feat/155-r1|.github/workflows/{ci.yml,ci-simple.yml,test-minimal.yml}|
Context: draft PR #3 (feat/duckdb-1.5.5) triggers ci.yml on ubuntu+macOS. Check its live status: gh api repos/seanchatmangpt/frozen-duckdb/actions/runs?branch=feat/duckdb-1.5.5 and .../commits/<sha>/check-runs (gh 2.54: use gh api, NOT gh pr checks -R). Diagnose any red run from its logs (gh api .../actions/runs/<id>/logs?job_id=... or jobs listing). Likely hazards: ubuntu jobs have no prebuilt .so (builder falls back to a long local compile — may time out), setup_env.sh assumes macOS dylib names.
DoD: [ ] PR #3 check status recorded (run ids + conclusions)  [ ] red causes fixed within workflow scope or explicitly documented as builder-owned with the fix specified  [ ] yaml-parse all touched workflows  [ ] committed"

"TR2|Release rehearsal + build-binaries hardening|/Users/sac/frozen-duckdb-wt/r2|feat/155-r2|.github/workflows/build-binaries.yml|
Context: the v1.5.5 tag run has never executed — rehearse it before the operator cuts. (1) HARDEN the workflow: matrix builds x86_64 on an arm64 runner — the local-compile fallback needs CMAKE_OSX_ARCHITECTURES to label the slice correctly; set it per matrix arch in the workflow AND honor an env override in builder (coordinated: TR7 owns builder lib.rs — set the env here, TR7 makes builder read it; note this dependency in History). (2) REHEARSE the ensure_binary cache-miss path locally in isolation: HOME=/tmp/fd-rehearsal-home cargo build of a scratch project depending on the builder — exercises download-404 → local compile of DuckDB v1.5.5. That is a 15-45 min full source build: run it with a generous timeout; if it exceeds 40 min, record partial evidence (cmake configured + make progressing) and stand PARTIAL. Verify the produced artifact lands at the path/name the workflow's find step expects.
DoD: [ ] workflow hardened + yaml-parse green  [ ] rehearsal executed with command+exit (or PARTIAL with preserved progress evidence)  [ ] committed"

"TR3|column_names() pre-execution panic|/Users/sac/frozen-duckdb-wt/r3|feat/155-r3|crates/frozen-duckdb/src/duckdb/raw_statement.rs; crates/frozen-duckdb/tests/column_names_tests.rs (new)|
Context: T5 observed raw_statement.rs:218 schema unwrap panics when column_names() is called before statement execution. Reproduce first (falsify before fixing), then fix (proper error, not panic), then add a regression test file covering: pre-execution column_names, post-execution column_names, empty-result column_names.
DoD: [ ] reproduction command+exit recorded  [ ] fix + regression test → cargo test -p frozen-duckdb --test column_names_tests exit 0  [ ] committed"

"TR4|docs.rs survival|/Users/sac/frozen-duckdb-wt/r4|feat/155-r4|crates/frozen-duckdb-sys/build.rs; crates/frozen-duckdb-sys/Cargo.toml|
Context: docs.rs builds on Linux where no .so asset exists → build.rs links (or compiles for 30+ min) → docs fail. Implement the standard DOCS_RS pattern: detect env DOCS_RS in sys build.rs → generate bindings from the vendored headers (builder's vendored-headers via the normal flow) but SKIP linking (no rustc-link-lib, no rpath) so rustdoc can typecheck without the library. Add [package.metadata.docs.rs] to sys Cargo.toml if useful. Verify: DOCS_RS=1 cargo build -p frozen-duckdb-sys exit 0 in a HOME without any ~/.frozen-duckdb cache (isolate with HOME=/tmp/fd-docs-home), and plain cargo build still links normally afterwards.
DoD: [ ] DOCS_RS=1 isolated build exit 0  [ ] normal build still links (CLI runs)  [ ] committed"

"TR5|test-validation made real|/Users/sac/frozen-duckdb-wt/r5|feat/155-r5|test-validation/**|
Context: A6 audit: test-validation/src/main.rs prints ❌/✅ but never asserts — it validates nothing (its cache-path pins were bumped but the harness is decorative). Make it a real gate: assert binary exists at builder's expected path (read VERSION from the builder crate, not a local hardcode), assert duckdb_library_version() via the FFI equals v1.5.5-derived expectation, assert header layout (duckdb/duckdb.h present), exit nonzero on any failure. Keep it a workspace member; run it: cargo run -p test-validation → exit 0.
DoD: [ ] harness asserts, no decorative prints  [ ] cargo run -p test-validation exit 0  [ ] committed"

"TR6|Repo hygiene + 帳 ledger|/Users/sac/frozen-duckdb-wt/r6|feat/155-r6|root rot files; scripts/{smoke*,fake_guard.sh,demo*}; HANDWRITTEN.md (new)|
Context: kcura-era rot at root (A6 audit): demo-dependency-issue.sh, test_architecture.rs, test_ffi_validation.rs, datasets/, test_datasets/, scripts/ smoke_go/smoke_node/smoke_py/fake_guard.sh family and ci_gate.sh/ci_gates.sh referencing non-existent crates/kcura-* paths. Triage each: still-useful → fix and keep; dead → git rm with the removal reasoned in the commit message; uncertain → keep + mark with a header comment. Then create HANDWRITTEN.md (帳): table path | semantic element | missing capability | intended owner pack | date — seed it with this wave's known agent-authored-by-hand rows (tickets T2A..TR6 non-generated repairs; the repo has no ggen pack projection).
DoD: [ ] every triaged file has a keep/fix/remove decision with evidence  [ ] repo still builds: cargo build --workspace exit 0  [ ] HANDWRITTEN.md exists with seeded rows  [ ] committed"

"TR7|Linux + multi-arch builder story|/Users/sac/frozen-duckdb-wt/r7|feat/155-r7|crates/frozen-duckdb-builder/src/lib.rs; prebuilt/setup_env.sh; scripts/download_duckdb_binaries.sh|
Context: builder is macOS-only by construction (downloads .dylib names; get_binary_path uses target_os extension so Linux wants libduckdb_{arch}.so — but the download URL still fetches .dylib). Make the next release multi-platform: (1) builder: platform-aware asset name (.dylib on macOS, .so on Linux) in the download URL + get_binary_path already handles extension — align them; honor CMAKE_OSX_ARCHITECTURES env in compile_duckdb_locally (TR2's workflow sets it per matrix arch); keep Windows on the documented local-compile fallback (no .dll work). (2) setup_env.sh: real Linux branch (LD_LIBRARY_PATH + .so symlink names). (3) download_duckdb_binaries.sh: align linux naming. NOTE: release assets for .so do not exist until the NEXT tag — document that in the builder comments and ticket History (the v1.5.5 tag ships macOS assets per current workflow; TR2 coordinates the workflow side).
DoD: [ ] platform-aware naming unit-consistent (review + cargo test -p frozen-duckdb-builder exit 0)  [ ] bash -n setup_env.sh + download script  [ ] committed"

"TR8|Packaging polish + final consistency|/Users/sac/frozen-duckdb-wt/r8|feat/155-r8|crates/frozen-duckdb/Cargo.toml; crates/frozen-duckdb-builder/Cargo.toml; README.md; CHANGELOG.md|
Context: pre-publish polish. (1) Crate metadata: builder + main crate get description/readme/keywords sanity (sys is TR4's; do not touch it); ensure readme files referenced exist. (2) README: badges/links point at real URLs (repo-relative docs links must resolve on crates.io — use absolute GitHub URLs where needed); mention Linux/multi-arch honestly (TR7 may land after you — write 'macOS prebuilt assets; Linux via local compile (prebuilt .so planned)' unless TR7 evidence says otherwise at your run time). (3) CHANGELOG: add entries for the wave-3 hardening under [1.5.5] Unreleased-fixes subsection. (4) Final grep: no stale 1.4.0-as-current, no 1.10505 vs 1.5.5 inconsistencies in user-facing strings.
DoD: [ ] metadata complete (cargo package -p frozen-duckdb --list spot-check)  [ ] README claims grep-anchored  [ ] CHANGELOG updated  [ ] committed"
)
# Standing column (FDDB-26922-03): each ticket's frontmatter aps:standing is
# rendered from its FINAL History standing; new tickets default to OPEN.
standing_for() {
  case "$1" in
    TR3|TR4|TR5|TR6|TR7|TR8) echo "ALIVE" ;;
    TR1) echo "PARTIAL_ALIVE" ;;
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
aps:standing: https://w3id.org/chatman/aps#${standing}
milestone: frozen-duckdb v1.5.5 wave 3 (hardening, pre-publish)
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

- Work ONLY in \`$wt\` on \`$branch\`. NEVER touch the main checkout /Users/sac/frozen-duckdb or sibling worktrees. Never push, never touch remotes, never publish.
- Atomic commits, conventional messages, on your branch only.
- Verify before claiming: preserve command + exit. A gate you did not run is UNKNOWN, not green.
- Append every transition to the History table below (the one sanctioned write outside your worktree), then STOP — the coordinator merges.

## History

| ts | standing | branch+SHA | gates+exits | remaining |
|----|----------|------------|-------------|-----------|
| $DATE | OPEN | $branch@$(cd "$REPO_ROOT" && git rev-parse --short feat/duckdb-1.5.5) | not started | all of DoD |
EOF
}

for spec in "${SPECS[@]}"; do
  id="${spec%%|*}";        rest="${spec#*|}"
  title="${rest%%|*}";     rest="${rest#*|}"
  wt="${rest%%|*}";        rest="${rest#*|}"
  branch="${rest%%|*}";    rest="${rest#*|}"
  scope="${rest%%|*}";     body="${rest#*|}"
  standing="$(standing_for "$id")"
  render_ticket "$id" "$title" "$wt" "$branch" "$scope" "$body" "$standing"
done

echo "wave-3 tickets rendered:"
ls "$OUT" | grep "^T-R"
