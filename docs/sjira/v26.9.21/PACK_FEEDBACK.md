# PACK_FEEDBACK.md — wave-5 crawl findings destined for pack facts

Per the wave-5 mission: every G ticket appends one section below with pack-relevant
findings (failed edges, missing capabilities, dual-owner hazards, law candidates).
Landing these in `~/ggen-marketplace` packs is G9's worktree job; this file is the
consumer-side evidence ledger. Append-only; one `##` section per ticket.

## G4 — GENESIS + HANDWRITTEN accuracy crawl (2026-09-21, feat/155-g4)

1. **CHANGELOG dual-owner hazard (paydown required).** `CHANGELOG.md` is
   pack-rendered (rule `changelog`, facts in `schema/changelog.ttl`, commit
   a09ce8a), but the superseded owner `scripts/generate-changelog.sh`
   (conventional-changelog-cli) is STILL invoked by `scripts/release_checklist.sh`
   (lines 54-55). A release run would clobber the pack render with angular-preset
   output. Remedy candidates: re-point release_checklist.sh at `ggen sync run`, or
   a release-checklist pack gate that refuses `generate-changelog.sh` invocations
   on a pack-rendered changelog. Owner family: release/checklist tooling (G5
   triage is the natural consumer).
2. **ggen sync writes signing keys into the consumer repo cwd.** `ggen sync run`
   materializes `.ggen/keys/signing.key` (PRIVATE) + `.ggen/keys/verifying.key`
   in the project root; the wave-4 integration merge (30b14fc) accidentally
   committed them (C4's History had recorded them as left-untracked). G4 untracked
   the pair and added `.ggen/` + `.ggen-v2/` gitignore rows. Pack/ggen remedy:
   key material belongs in a state dir OUTSIDE the repo tree, or behind an
   explicit config path — the current default makes every consumer one careless
   `git add .` away from committing a private key. Owner family:
   receipt-provenance / ggen-verify-pack.
3. **Declarative-vs-frontmatter schema convergence is the highest-leverage ggen
   fix.** The canonical declarative config cannot express multi-SELECT templates
   (FM-CONFIG-003), so the 15 C2 Diataxis renders are stranded: present, tracked,
   existence-checked as `gap` rows with `lineage:7f604e5` sources, but not
   re-renderable. Until convergence lands, every edit to those renders is 帳 debt
   (UNSUPPORTED in-place, G1). One schema unifying both expression forms collapses
   this entire debt class.
4. **genesis-check has no first-class `lineage:` source form.** Lineage-pinned
   gap rows skip source checks by design; an ancestry check
   (`git merge-base --is-ancestor <sha> HEAD`) inside the parser would make them
   self-verifying instead of crawl-verified. Owner family: repo-reconciliation
   pack (the genesis-check template lives in ggen.toml rule
   render-makefile-verify).
5. **Consumer-instance-data pack contract works — replicate it.**
   castle-changelog-release-pack ships zero individuals and the consumer authors
   `schema/changelog.ttl` facts wired through `[ontology].imports`. This is the
   clean answer to the known domain.ttl multi-writer hazard (C4/C5 unions were
   hand-merged; ledger row still open): pack-owned fact files per concern, one
   render each, no hand merges.
6. **Contract-in-ontology + thin executor pattern is pack-worthy.** C5's
   `scripts/verify_publish_receipt.py` is a hand-authored executor whose field
   contract lives entirely in the ontology + rendered census
   (`generated/receipt_contract_matrix.json`). A provenance-gate pack could render
   the executor per consumer, removing the hand script (natural next 帳 paydown
   row: HANDWRITTEN candidate once a pack expresses it).

Falsifier note: every finding above was observed against feat/155-g4@HEAD
(genesis-check checked=57 gaps=23; release_checklist grep; git ls-files for the
keypair; git log -S for rule provenance). Not fed forward as speculation.
