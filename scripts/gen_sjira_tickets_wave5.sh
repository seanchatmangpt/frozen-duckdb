#!/bin/bash
# gen_sjira_tickets_wave5.sh — wave 5: gap/staleness crawl, findings feed back into packs.
set -euo pipefail
MILESTONE="${1:-v26.9.21}"
DATE="$(date +%Y-%m-%d)"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$REPO_ROOT/docs/sjira/$MILESTONE"
mkdir -p "$OUT"

SPECS=(
"G1|Diataxis renders staleness crawl|/Users/sac/frozen-duckdb-wt/g1|feat/155-g1|docs/{tutorials,how-to,reference,explanation}/**; docs/{index,meta}.md|
Crawl every C2-rendered Diataxis file for claims falsified by current reality (they were rendered before wave-4 landed: no mention of make verify/sync/genesis-check/gates, ggen.toml, GENESIS.md, HANDWRITTEN.md, PUBLISH_RUNBOOK, receipts). Verify each how-to recipe by EXECUTING it where cheap (e.g. documented commands). Fix renders in place — these are consequences of the frontmatter schema that the canonical declarative config cannot re-render (known edge in docs/GENESIS.md), so in-place edits are ledgered UNSUPPORTED in HANDWRITTEN.md + your ticket History. Append pack-feedback rows to docs/sjira/v26.9.21/PACK_FEEDBACK.md (create with a header if absent).
DoD: [ ] every Diataxis file crawled with verdict  [ ] fixes applied + executed-where-possible  [ ] HANDWRITTEN row  [ ] PACK_FEEDBACK rows  [ ] ggen sync run + cargo build green  [ ] committed"

"G2|Root docs + changelog facts crawl|/Users/sac/frozen-duckdb-wt/g2|feat/155-g2|README.md; {QUICKSTART,MIGRATION_GUIDE,CONTEXT,FORWARD_DEPLOYMENT}.md; schema/changelog.ttl; CHANGELOG.md (via re-render only)|
Crawl root docs for staleness vs the post-wave-4 reality (ggen consumer config, make targets, receipts, GENESIS manifest, 110+ commit delta). CHANGELOG LAW: it is pack-rendered from schema/changelog.ttl — to add/fix entries, EDIT THE FACTS in schema/changelog.ttl and run ggen sync run (never hand-edit CHANGELOG.md); wave-4 items (ggen composition, verify chain, receipts, GENESIS) likely deserve a changelog presence — judge and feed the facts. Also verify README claims (platform support, links) still hold.
DoD: [ ] each root doc crawled with verdict  [ ] changelog changes made via facts + re-render only (diff CHANGELOG before/after to prove)  [ ] ggen sync run + cargo build green  [ ] PACK_FEEDBACK rows  [ ] committed"

"G3|docs/ tree staleness crawl|/Users/sac/frozen-duckdb-wt/g3|feat/155-g3|docs/{architecture,guides,api,cli,contributing,testing}/**|
Crawl the wave-1/2-era docs tree for information falsified by waves 3-4: binary-management.md (predates ggen/GENESIS), integration.md flows, api/cli docs (cli commands vs actual frozen-duckdb-cli surface), testing docs (go-ffi-smoketest vs run_ffi_validation.sh's kept-with-header state), contributing docs. Fix in place; where a doc describes removed machinery, update or mark clearly. Verify doc-referenced file paths exist.
DoD: [ ] every docs/ subdir crawled with verdict  [ ] fixes applied  [ ] dead-path grep clean  [ ] ggen sync run + cargo build green  [ ] PACK_FEEDBACK rows  [ ] committed"

"G4|GENESIS + HANDWRITTEN accuracy crawl|/Users/sac/frozen-duckdb-wt/g4|feat/155-g4|docs/GENESIS.md; HANDWRITTEN.md; .gitignore|
Crawl the two ledgers against reality: every GENESIS row — source exists? consequence present? status correct (post-dry-run fixes: Makefile is now rule-owned, C2 renders lineage-pinned)? every HANDWRITTEN row — still true, or paid down (the Makefile row was paid down by the dry-run agent — verify the ledger reflects that)? Missing rows: wave-4/5 artifacts not yet mapped (gates/dry-run-publish*, generated/receipt_contract_matrix.json, scripts/verify-gates.sh, scripts/verify_publish_receipt.py, ADMISSION.md, CI_PROJECTION.md, docs/tutorials et al). .gitignore completeness (.ggen/, .ggen-v2/, target/, generated/?).
DoD: [ ] every row verified with evidence  [ ] missing rows added  [ ] paid-down rows removed  [ ] make genesis-check exit 0  [ ] committed"

"G5|scripts/ crawl: headers + drift|/Users/sac/frozen-duckdb-wt/g5|feat/155-g5|scripts/**; scripts/README*.md|
Crawl scripts/ post-TR6: STATUS headers on kept files (test_ffi_simple.sh, run_ffi_validation.sh, smoke_go*.go, validate_prod_build.sh, validate_frozen_approach.sh) — are their factual claims still right (central-cache model, library resolution)? README_gap_detection.md documents REMOVED gates (TR6 handoff) — reconcile or mark. TR6's BLOCKED triage now in scope: scan_fakes.sh, scan_fakes_core_team.sh, kcura-config{,.example}.yaml — keep/fix/remove with evidence. Execute cheap probes (bash -n, dry runs) where safe.
DoD: [ ] every script crawled with verdict  [ ] fixes/removals reasoned  [ ] bash -n green on touched scripts  [ ] cargo build green  [ ] committed"

"G6|in-code doc-comment crawl|/Users/sac/frozen-duckdb-wt/g6|feat/155-g6|crates/**{Cargo.toml,*.rs} doc comments only|
Crate-by-crate crawl of doc comments for falsified claims: frozen-duckdb lib.rs (the duckdb = 1.4.0 before-state example — TR8 flagged the crates.io truth; fix the example to a real resolvable version and document the encoding law), builder lib.rs (cache-path doc comments vs the normalization code), sys build.rs comments, CLI help strings vs actual commands (frozen-duckdb-cli --help). Doc changes only — no behavior edits; cargo test must stay 301/0.
DoD: [ ] each crate crawled with verdict  [ ] doc fixes applied  [ ] cargo test --workspace 301/0 preserved  [ ] PACK_FEEDBACK rows  [ ] committed"

"G7|workflows + projection docs crawl|/Users/sac/frozen-duckdb-wt/g7|feat/155-g7|.github/workflows/**; docs/sjira/v26.9.21/{CI_PROJECTION,ADMISSION}.md|
Crawl: (1) the four workflows — do they match current reality (pre-seed zips, CMAKE_OSX_ARCHITECTURES, asset names, actionlint clean)? Any drift from TR1/TR2's fixes? (2) CI_PROJECTION.md — are its 5 failed edges still true on ggen 26.8.18? (3) ADMISSION.md — does its path match the now-AUTHORED pack (packs/frozen-duckdb-pack exists on branch pack/frozen-duckdb-capabilities @301ebc42f — read it via git show in ~/ggen-marketplace)? Update docs to reality; workflow changes only if a claim is falsified AND the fix is mechanical (actionlint/yaml-parse after).
DoD: [ ] 4 workflows verified (actionlint or yaml-parse)  [ ] both projection docs updated to reality  [ ] ggen sync run + cargo build green  [ ] committed"

"G8|release docs coherence crawl|/Users/sac/frozen-duckdb-wt/g8|feat/155-g8|docs/sjira/v26.9.21/{PUBLISH_RUNBOOK,MILESTONE,PR_BODY,ISSUE_1_COMMENT,TPUB}.md|
Crawl the release surface for internal contradictions accumulated across 5 waves: PUBLISH_RUNBOOK now has base steps + C4's 6-phase section + C5's provenance gate — are they ONE coherent procedure or three overlapping ones? Do MILESTONE checkboxes match observed reality? Does PR_BODY's test plan still hold at the new head? Does ISSUE_1_COMMENT's language respect the dry-run-publish fence (no claims of published)? Harmonize into one authoritative runbook flow; do not rewrite History tables.
DoD: [ ] contradictions listed then fixed  [ ] runbook reads as one procedure  [ ] fence law respected  [ ] ggen sync run green  [ ] committed"

"G9|pack feedback: author updates|/Users/sac/ggen-marketplace-wt/g9|pack/frozen-duckdb-capabilities|~/ggen-marketplace/packs/frozen-duckdb-pack/** (your worktree branch)|
YOUR WORKTREE: /Users/sac/ggen-marketplace-wt/g9 — create it: cd ~/ggen-marketplace && git worktree add ~/ggen-marketplace-wt/g9 -b pack/frozen-duckdb-capabilities-g9 pack/frozen-duckdb-capabilities (the pack authored by C6 @301ebc42f, unmerged). Crawl the frozen-duckdb repo (/Users/sac/frozen-duckdb — read-only) for NEW laws the pack does not yet express, and feed them into the pack's ontology/gates/templates: candidates from the waves — the ggen consumer-config law ([[ontology.pack]] absent, imports operative, schema exclusivity), Makefile-as-rendered-consequence law, receipt-runtime-state law (.ggen-v2 untracked, secrets), domain.ttl multi-writer hazard (pack-owned fact files as remedy), the cache-normalization + DOCS_RS laws (verify C6's existing facts still match the current build.rs/builder code — falsify, update if drifted). Extend gates to tripwire at least 2 new laws. Update the pack README's law list + bump pack.toml patch version. Validate: SPARQL gates via rdflib, structural check, ggen graph validate — preserve exits.
DoD: [ ] new laws in ontology with evidence anchors  [ ] ≥2 new gates proven to refuse a corrupted fixture  [ ] pack.toml version bumped  [ ] validation exits recorded  [ ] committed"

"G10|cross-cutting consistency crawler|/Users/sac/frozen-duckdb-wt/g10|feat/155-g10|repo-wide READ; fixes only in: docs/**, README.md, HANDWRITTEN.md|
The adversarial whole-repo crawler. Hunt: (1) version chaos — every occurrence of 1.4.0 / 1.5.5 / v1.5.5 / 1.10505 / 1.10505.0 classified (current-pin / historical / crate-encoding / STALE); (2) dead internal links (README + docs/*.md hrefs to files that don't exist, anchor links); (3) session-relative claims in durable files (this session / just fixed / not yet run) that will read as false in a week; (4) TODO/FIXME/XXX/UNSUPPORTED inventory with per-item disposition; (5) orphaned files (nothing references them — feed candidates to docs/GENESIS.md gaps via a report, do not delete). Produce a findings table, fix the safe class (docs/links/labels), ledger the rest.
DoD: [ ] version chaos table complete  [ ] dead links fixed or ledgered  [ ] session-relative claims neutralized in durable docs  [ ] inventory delivered  [ ] ggen sync run + cargo build green  [ ] committed"
)

render_ticket() {
  local id="$1" title="$2" wt="$3" branch="$4" scope="$5" body="$6"
  cat > "$OUT/$id.md" <<EOF
---
id: $id
dcterms:title: "$title"
dcterms:created: "$DATE"
dcterms:isPartOf: "$MILESTONE"
rdf:type: oslc_cm:ChangeRequest, earl:TestRequirement
aps:standing: https://w3id.org/chatman/aps#OPEN
milestone: frozen-duckdb v1.5.5 wave 5 (gap/staleness crawl → pack feedback)
worktree: $wt
branch: $branch
---

# $id — $title

## Mission

Falsify every claim in your slice against current reality. Outdated → fix at the
lawful source (pack facts → re-render for consequences; doc text in place for
prose). Missing → add or ledger. Feed pack-relevant findings into
docs/sjira/v26.9.21/PACK_FEEDBACK.md (append your section; create with header if
absent). 偽: a claim you did not test is UNKNOWN, not true.

## Scope

\`\`\`
$scope
\`\`\`

## Contract (all tickets)

- Base f3120fd: all gates green (ggen sync run, make verify, make genesis-check, 301 tests). Keep them green.
- Work ONLY in \`$wt\` on \`$branch\` (G9: marketplace worktree per ticket). Never push. Atomic conventional commits.
- Rendered consequences are edited at their source + re-rendered via \`ggen sync run\` — never hand-edited (exceptions must be ledgered UNSUPPORTED with the reason).
- Preserve command+exit evidence. Append transitions to the History table below (sanctioned main-checkout write), then STOP — the coordinator merges.

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
  render_ticket "$id" "$title" "$wt" "$branch" "$scope" "$body"
done
echo "rendered G1..G10"
