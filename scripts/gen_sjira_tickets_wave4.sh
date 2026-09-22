#!/bin/bash
# gen_sjira_tickets_wave4.sh — wave 4: marketplace capability composition tickets.
set -euo pipefail
MILESTONE="${1:-v26.9.21}"
DATE="$(date +%Y-%m-%d)"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$REPO_ROOT/docs/sjira/$MILESTONE"
mkdir -p "$OUT"

SPECS=(
"C2|readme-diataxis-pack composition|/Users/sac/frozen-duckdb-wt/c2|feat/155-c2|ggen.toml; schema/; docs/**; README.md (read-mostly)|
Pack: packs/readme-diataxis-pack (projects README facts into Diataxis quadrants: tutorials/getting-started.md, how-to/*, reference/*, explanation/*, docs/index.md, docs/meta.md). 階 first: read the pack's README.md + pack.toml + templates/ + gates/ in ~/ggen-marketplace. Declare it via [[ontology.pack]] in ggen.toml, seed consumer facts from the existing README, ggen sync run, reconcile rendered docs with existing docs/ (existing files that duplicate a quadrant get superseded or cross-linked, not silently deleted). Record failed edges where the pack's assumptions don't fit this repo.
DoD: [ ] pack declared + sync run exit 0  [ ] Diataxis set rendered or failed-edge ledgered  [ ] cargo gate still green  [ ] committed"

"C3|repo-reconciliation-pack composition|/Users/sac/frozen-duckdb-wt/c3|feat/155-c3|ggen.toml; schema/; docs/sjira/** (rendered); gates/|
Pack: packs/repo-reconciliation-pack (as-found/as-desired/as-built/as-operated 照合; all_deltas_are_classified, blocking_delta_prevents_promotion). Its pack.toml declares deps (as_found/load_path/intervention/temporary_works/mermaid packs) to declare in the consumer [ontology.pack] table. Apply it to the v1.5.5 milestone: as-found = master@1c84384 state, as-desired = PR #3 + sjira tickets, as-built = feat/duckdb-1.5.5@HEAD, deltas classified (none blocking). Render its gates/templates per its ontology.
DoD: [ ] pack + deps declared  [ ] reconciliation artifacts rendered  [ ] delta classification recorded with evidence  [ ] sync exit 0  [ ] committed"

"C4|dry-run-publish-pack composition|/Users/sac/frozen-duckdb-wt/c4|feat/155-c4|ggen.toml; schema/; docs/sjira/v26.9.21/PUBLISH_RUNBOOK.md; gates/|
Pack: packs/dry-run-publish-pack — the release DoD as a 6-phase PDDL8 domain (scope, generate, verify, manufacture, cleanroom, receipt) with a DRY-RUN-OVERCLAIM FENCE (no atom named published/crates-io-uploaded; terminal goal is dry-run-verified). Compose it onto this repo: align PUBLISH_RUNBOOK.md phases to the pack's 6 gates, render its fixtures/shapes as repo-side evidence templates, keep the fence law visible. This makes the operator's publish dry-run a repeatable modeled gate, not a one-off.
DoD: [ ] pack declared  [ ] 6-phase alignment rendered/reconciled with the runbook  [ ] fence law present in the rendered artifact  [ ] sync exit 0  [ ] committed"

"C5|receipt + supply-chain evidence|/Users/sac/frozen-duckdb-wt/c5|feat/155-c5|ggen.toml; schema/; scripts/; docs/sjira/v26.9.21/PUBLISH_RUNBOOK.md|
Packs: packs/supply-chain-evidence-pack and/or packs/receipt-provenance-unification-pack (choose by fit; REUSE the better, COMPOSE if both). Goal: `ggen receipt verify` and evidence provenance wired into this repo's release path — the TPUB pre-flight receipts and the wave History become verifiable provenance, not prose. Render per the chosen pack's ontology; wire a receipt-verify step into PUBLISH_RUNBOOK.md's gate sequence.
DoD: [ ] pack declared  [ ] receipt capability rendered + verified once against a real receipt  [ ] runbook gate step added  [ ] sync exit 0  [ ] committed"

"C7|castle-changelog-release-pack|/Users/sac/frozen-duckdb-wt/c7|feat/155-c7|ggen.toml; schema/; CHANGELOG.md|
Pack: packs/castle-changelog-release-pack. Assess fit against the hand-maintained CHANGELOG.md (a hand-written surface = 帳 debt). If the pack renders changelogs from facts (git history/tags/ontology), project v1.5.5's entry and reconcile; if it doesn't fit (wrong domain, needs unavailable inputs), that is a FAILED EDGE to record in detail — failed(edge) ≠ failed(G), silent pruning forbidden. Either way the outcome is a rendered artifact or a ledgered edge, never a silent skip.
DoD: [ ] pack assessed with evidence  [ ] rendered or failed-edge ledgered  [ ] sync exit 0  [ ] committed"

"C8|verify-gate chain|/Users/sac/frozen-duckdb-wt/c8|feat/155-c8|ggen.toml; schema/; scripts/; Makefile (new)|
Packs: packs/ggen-verify-pack and packs/evidence-standing-pack (assess, pick fit). Goal: one repo-side verify capability that chains the existing gates (cargo fmt/clippy/build/test, test-validation harness, ggen sync run, receipt verify) into a single declared, rendered target (Makefile verify) so the full gate battery is one command and its standing vocabulary (ALIVE/BLOCKED/…) is rendered from pack facts, not ad-hoc.
DoD: [ ] pack declared/assessed  [ ] verify target rendered and EXECUTED once end-to-end exit 0  [ ] sync exit 0  [ ] committed"

"C9|admission + failed-edge ledger|/Users/sac/frozen-duckdb-wt/c9|feat/155-c9|ggen.toml; schema/; docs/sjira/v26.9.21/ADMISSION.md (new)|
Packs: packs/marketplace-governance-pack + packs/pack-compatibility-pack (+ pack-maturity-pack for the maturity ladder). Produce docs/sjira/v26.9.21/ADMISSION.md: the exact admission path for a new pack authored for this repo (parallel agent authors packs/frozen-duckdb-pack in the marketplace worktree pack/frozen-duckdb-capabilities), the qualification/compatibility checks it must pass, and the FAILED-EDGE LEDGER: capabilities this repo needed that NO marketplace pack expresses (candidates from wave history: macOS universal dylib staging, DOCS_RS no-link builds, duckdb asset-name law, rpath propagation) — each recorded as failed(edge) with intended owner, feeding the pack author.
DoD: [ ] admission path documented with pack citations  [ ] failed-edge ledger complete (≥4 rows)  [ ] sync exit 0  [ ] committed"

"C10|integration: manifest + Makefile + CI hook|/Users/sac/frozen-duckdb-wt/c10|feat/155-c10|ggen.toml; schema/; Makefile (new, coordinate); docs/GENESIS.md (new)|
Capstone: the 器 reconciliation manifest. Produce docs/GENESIS.md (or rendered equivalent): a table mapping every generated/consequence file in this repo ← owning pack + template + rule (and files that are hand-written ← owning HANDWRITTEN.md row), so upstream pack changes are detectable by reconciliation, not orphaned. Add Makefile targets: make sync (ggen sync run), make verify (coordinate with C8's target — if both land, the later merge wins and reconciles; keep your target named verify only if C8 hasn't claimed it in YOUR branch's view — otherwise name it gates). Do not enable CI auto-sync yet — propose the CI hook as a documented step (dry-run/check mode only).
DoD: [ ] manifest covers every consequence file  [ ] Makefile targets rendered/declared  [ ] sync exit 0  [ ] committed"

"C11|CI projection assessment|/Users/sac/frozen-duckdb-wt/c11|feat/155-c11|ggen.toml; schema/; .github/workflows/ (only if a pack projects them); docs/sjira/v26.9.21/CI_PROJECTION.md (new)|
Packs: packs/github-actions-pack and packs/cargo-cicd-pack (assess both). Context: this repo's CI (ci.yml, ci-simple.yml, test-minimal.yml, build-binaries.yml) was hand-repaired in wave 3 (TR1/TR2) — that is 帳 debt. Assess whether either pack can project these workflows from facts: if yes, project and reconcile (careful: the wave-3 fixes encode hard-won laws — cache pre-seeding, CMAKE_OSX_ARCHITECTURES, asset names — a projection that loses them is a regression; diff ruthlessly); if no, record the failed edge with specifics. Output: CI_PROJECTION.md verdict + rendered workflows or ledgered edges.
DoD: [ ] both packs assessed with citations  [ ] projection or failed-edge ledger  [ ] yaml-parse any touched workflow  [ ] sync exit 0  [ ] committed"
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
milestone: frozen-duckdb v1.5.5 wave 4 (marketplace capability composition)
worktree: $wt
branch: $branch
---

# $id — $title

## 階 law (search ladder, before any write)

REUSE the pack as-is → COMPOSE packs → EXTEND a pack family → INVENT only as an
admitted pack. Record every failed edge in your ticket History: failed(edge) ≠
failed(G); silent pruning forbidden.

## Scope

\`\`\`
$scope
\`\`\`

Anything outside scope: do not edit — ledger it instead.

## Work order

$body

## Contract (all tickets)

- ggen consumer bootstrap ALREADY landed (ggen.toml + schema/domain.ttl, sync parses green). Declare packs via [[ontology.pack]]; never hand-edit a rendered consequence.
- Work ONLY in \`$wt\` on \`$branch\` (\`$wt\` paths are absolute above; the marketplace lives at ~/ggen-marketplace, READ-ONLY for you unless your ticket says otherwise). Never push. Atomic conventional commits.
- \`ggen sync run\` must exit 0 after your changes; \`cargo build --workspace\` must stay green; every render/gate EXECUTED once with command+exit preserved (証).
- Append transitions to the History table below (sanctioned main-checkout write), then STOP — the coordinator merges.

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
echo "rendered: C2 C3 C4 C5 C7 C8 C9 C10 C11"
