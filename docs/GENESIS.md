# GENESIS.md — the 器 reconciliation manifest

Every generated/consequence file in this repo is mapped to its owning source, so an
upstream change tells you exactly which files must re-render. **Reconciliation happens
by this manifest, never by orphaned projections**: if a source below changes, its
consequence rows are stale until re-rendered, and a consequence whose source has
vanished is an orphan (a defect, not a leftover).

- 源 hierarchy for pack-rendered files: `~/ggen-marketplace/marketplace.toml` →
  `packs/<name>/pack.toml` → `ontology.ttl` → `templates/` → `gates/` → this repo.
  Consumers declare packs in `ggen.toml` under `[[ontology.pack]]`; capacity rules
  under `[[generation.rules]]`. Never hand-edit a rendered consequence.
- Repo-tooling consequences are re-rendered by re-running the owning generator
  (listed per row). Hand-written lines on 産面 exist only through a `HANDWRITTEN.md`
  row (帳 ledger; shrinks monotonically per milestone).
- Run `make genesis-check` (or `make gates`) to verify every row: each source still
  exists, each consequence is present, each upstream pin still matches. The check
  parses the machine table below (rows whose first cell is a backticked path).

## Row vocabulary

- consequence — the projection file (or family glob; a family row names its anchor
  file and counts the family in the rule column). Rows prefixed `ext:` are
  consequences outside this repo (checked as documentation, not existence).
- source — where the bytes come from. Forms: a repo path (must exist), a glob
  (must match), `upstream:repo@ref` (ref must still be the recorded pin:
  a 40-hex ref is checked against the `vendors/duckdb-rs` gitlink, a version ref
  against the pin in `schema/domain.ttl`), `HANDWRITTEN.md` (the named ledger row
  must still exist and still name the path), `hand:` (authored in place under a
  sanctioned law), `unknown:` (provenance not yet established), `lineage:<sha>`
  (bytes rendered at a pinned commit whose owning rule is absent from the
  canonical config; valid only on `gap` rows — genesis-check skips source checks
  there; the commit must remain an ancestor of HEAD, checked by crawl — parser
  support for an ancestry check is recorded pack feedback, not yet built).
- status — `active` (checked), `pending` (source not yet in this branch's tree;
  the row flips to active when the source lands — if the source exists while the
  row is still pending, genesis-check FAILS so the flip cannot be forgotten),
  `gap` (reconciliation debt: a file on 産面 whose bytes lack an operative
  in-tree generator — hand-written, unknown-provenance, or rendered by a rule
  not in the canonical config; listed here instead of being silently pruned).

## Manifest (machine-readable — parsed by `make genesis-check`)

| consequence | source | status | rule | owner |
|---|---|---|---|---|
| `ggen.toml` | `hand:` | active | SOURCE root (consumer config). Edited in place; declares packs and generation rules; bootstrap commit 304a958. Never rendered. | C10 records; bootstrap by wave-4 coordinator |
| `schema/domain.ttl` | `hand:` | active | SOURCE root (consumer ontology seed). Edited in place; carries the DuckDB version pin checked by upstream rows. | C10 records; bootstrap by wave-4 coordinator |
| `docs/sjira/v26.9.21/T[0-9]*.md` | `scripts/gen_sjira_tickets.sh` | active | Edit the SPECS table in the script, re-run it; rendered tickets are consequences. Ticket History sections are the sanctioned append-only exception (票 law). | wave-2 ticket wave |
| `docs/sjira/v26.9.21/TR*.md` | `scripts/gen_sjira_tickets_wave3.sh` | active | Same rule as the T row: SPECS table is the ontology, script is the projection. | wave-3 hardening |
| `docs/sjira/v26.9.21/C[0-9]*.md` | `scripts/gen_sjira_tickets_wave4.sh` | active | Renderer + outputs landed with the wave-4 merge (commit 400fba1); pending row flipped to active by the wave-4 dry run (2026-09-21) — genesis-check correctly refused sync/gates until this flip. Edit the SPECS table in the script, re-run it; ticket History sections are the sanctioned append-only exception (票 law). | wave-4 marketplace capability |
| `CHANGELOG.md` | `ggen.toml` | active | Rendered by rule `changelog` (castle-changelog-release-pack contract): consumer facts in schema/changelog.ttl render via templates/CHANGELOG.md.tmpl — edit the facts, re-run `ggen sync run`, never the render. Superseded owner scripts/generate-changelog.sh is STILL invoked by scripts/release_checklist.sh (lines 54-55) — live dual-owner hazard; paydown recorded in PACK_FEEDBACK.md (G4). | changelog composition (commit a09ce8a); re-pointed by wave-5 G4 |
| `ext:GitHub-Releases/libduckdb_{arch}.dylib` | `.github/workflows/build-binaries.yml` | active | Out-of-repo consequence: tag push renders release assets via the workflow matrix. Workflow itself is a hand-maintained SOURCE (TR2-hardened). | TR2 |
| `crates/frozen-duckdb-builder/vendored-headers/duckdb.h` | `upstream:duckdb/duckdb@v1.5.5` | active | From the official DuckDB v1.5.5 release zip (file is upstream-autogenerated by DuckDB scripts/generate_c_api.py); staged by the builder's cache normalization (crates/frozen-duckdb-builder/src/lib.rs, VENDORED_HEADERS_DIR). Re-stage both headers on any DuckDB bump. | wave-2 cache normalization |
| `crates/frozen-duckdb-builder/vendored-headers/duckdb.hpp` | `upstream:duckdb/duckdb@v1.5.5` | active | Same rule as duckdb.h above — the pair stages together. | wave-2 cache normalization |
| `prebuilt/duckdb.h` | `upstream:duckdb/duckdb@v1.5.5` | active | Same upstream artifact, staged for the manual prebuilt workflow (see prebuilt/README.md). Re-stage on DuckDB bump together with vendored-headers. | prebuilt workflow |
| `prebuilt/duckdb.hpp` | `upstream:duckdb/duckdb@v1.5.5` | active | Same rule as prebuilt/duckdb.h — the pair stages together. | prebuilt workflow |
| `crates/frozen-duckdb/src/duckdb/mod.rs` | `upstream:duckdb/duckdb-rs@f4fe688a8718dd1645f41f4eef336d5c8dd0738f` | active | Family row: 41 files under crates/frozen-duckdb/src/duckdb/ (incl. appender/, core/, types/), vendored from duckdb-rs (1.4.0-era) once, then repaired in place — repairs go through HANDWRITTEN.md rows, re-sync is a deliberate upstream bump of the gitlink below. | vendored API layer |
| `vendors/duckdb-rs` | `upstream:duckdb/duckdb-rs@f4fe688a8718dd1645f41f4eef336d5c8dd0738f` | active | The gitlink itself (mode 160000, uninitialized submodule) — the recorded upstream reference for the vendored API layer. | vendored API layer |
| `crates/frozen-duckdb/src/duckdb/raw_statement.rs` | `HANDWRITTEN.md` | active | Vendored file with an in-place repair (column_names pre-execution error); lineage row above, repair row in the ledger. | HANDWRITTEN row 1 (TR3) |
| `crates/frozen-duckdb/tests/column_names_tests.rs` | `HANDWRITTEN.md` | active | Wholly hand-written regression-test file for the repair above. | HANDWRITTEN row 1 (TR3) |
| `crates/frozen-duckdb-sys/build.rs` | `HANDWRITTEN.md` | active | DOCS_RS pattern (bindings from vendored headers, skip link/rpath on docs.rs). | HANDWRITTEN row 2 (TR4) |
| `crates/frozen-duckdb-builder` | `HANDWRITTEN.md` | active | Ledger token is the crate dir; semantic element = acquisition paths in src/lib.rs (cache normalization law). | HANDWRITTEN row 3 (wave-2) |
| `prebuilt/setup_env.sh` | `HANDWRITTEN.md` | active | install-name compat symlinks (neutral libduckdb.dylib chain) + TR7 Linux branch. | HANDWRITTEN row 4 (T6/TR7) |
| `scripts/create_frozen_setup.sh` | `HANDWRITTEN.md` | active | Fetch-scheme repair: real v1.5.5 release-asset URLs. | HANDWRITTEN row 5 (T6) |
| `scripts/build_frozen_duckdb.sh` | `HANDWRITTEN.md` | active | Source-build pin: clone --branch v1.10505.0. | HANDWRITTEN row 6 (T6) |
| `scripts/build_static_duckdb.sh` | `HANDWRITTEN.md` | active | Static twin pinned to the same family float. | HANDWRITTEN row 6 (T6) |
| `test-dependency/Cargo.toml` | `HANDWRITTEN.md` | active | Standalone workspace table + crates.io pin duckdb = "1.10505". | HANDWRITTEN row 7 (T7 P5) |
| `scripts/test_ffi_simple.sh` | `HANDWRITTEN.md` | active | TR6 STATUS header (kept-uncertain marker). | HANDWRITTEN row 8 (TR6) |
| `scripts/run_ffi_validation.sh` | `HANDWRITTEN.md` | active | Same TR6 STATUS-header family as test_ffi_simple.sh. | HANDWRITTEN row 8 (TR6) |
| `scripts/smoke_go.go` | `HANDWRITTEN.md` | active | Same TR6 STATUS-header family. | HANDWRITTEN row 8 (TR6) |
| `scripts/smoke_go_simple.go` | `HANDWRITTEN.md` | active | Same TR6 STATUS-header family. | HANDWRITTEN row 8 (TR6) |
| `scripts/validate_prod_build.sh` | `HANDWRITTEN.md` | active | Same TR6 STATUS-header family. | HANDWRITTEN row 8 (TR6) |
| `scripts/validate_frozen_approach.sh` | `HANDWRITTEN.md` | active | Same TR6 STATUS-header family. | HANDWRITTEN row 8 (TR6) |
| `scripts/lib/config.sh` | `HANDWRITTEN.md` | active | Same TR6 STATUS-header family (lib/ coupling noted in ledger). | HANDWRITTEN row 8 (TR6) |
| `docs/GENESIS.md` | `hand:` | active | This manifest; SOURCE (authored under C10). genesis-check parses the table above — keep first-cell discipline (backticked path only in manifest data rows). | C10 |
| `Makefile` | `ggen.toml` | active | Rendered consequence of rule `render-makefile-verify`: C8 verify targets from schema/verify.ttl facts + C10 sync/genesis-check/gates block (C10 bytes moved into the owning rule template during the wave-4 dry run; the HANDWRITTEN.md multi-rule-union row was paid down, and the merge's truncated block head restored). Re-render: `ggen sync run`. | C8+C10; union repaired by wave-4 dry run |
| `docs/sjira/v26.9.21/TPUB.md` | `hand:` | active | Hand-authored dry-run publish ticket (commit 7a271c7; no renderer covers it — a failed edge for the wave-4 renderer, recorded there). | TPUB wave |
| `docs/sjira/v26.9.21/PUBLISH_RUNBOOK.md` | `hand:` | active | Hand-rendered operator cut sequence with TPUB pre-flight evidence (commit 4c41d83). | TPUB wave |
| `docs/sjira/v26.9.21/PR_BODY.md` | `hand:` | active | T8 deliverable, agent-authored in place (ticket-scoped; 票 session history is the sanctioned medium). | T8 |
| `docs/sjira/v26.9.21/ISSUE_1_COMMENT.md` | `hand:` | active | T8 deliverable, agent-authored in place. | T8 |
| `docs/sjira/v26.9.21/MILESTONE.md` | `hand:` | active | T8 deliverable: DoD ticks + operator cuts. | T8 |
| `docs/sjira/v26.9.21/_RUNBOOK.md` | `hand:` | active | Dispatch contract (canonical dispatch prompt form); edited only by the coordinator. | coordinator |
| `scripts/duckdb_ffi.h` | `hand:` | gap | Pre-wave Go-smoketest FFI header, authored by hand with NO HANDWRITTEN row. Reconciliation debt: pay down by adding a ledger row or a generator. | C10 gap record |
| `scripts/lib/intelligent_cache.sh` | `unknown:` | gap | Provenance not established (pre-wave); referenced by TR6 lib/ coupling note. | C10 gap record |
| `scripts/lib/logging.sh` | `unknown:` | gap | Provenance not established (pre-wave). | C10 gap record |
| `scripts/lib/self_healing.sh` | `unknown:` | gap | Provenance not established (pre-wave). | C10 gap record |
| `scripts/scan_fakes.sh` | `hand:` | gap | kcura-era file outside TR6 scope table; recorded BLOCKED in TR6 History; remaining lib/ consumer. | HANDWRITTEN drift note |
| `scripts/scan_fakes_core_team.sh` | `hand:` | gap | Same kcura-era triage debt as scan_fakes.sh. | HANDWRITTEN drift note |
| `scripts/kcura-config.yaml` | `hand:` | gap | kcura-era config, triage pending (TR6 BLOCKED). | HANDWRITTEN drift note |
| `scripts/kcura-config.example.yaml` | `hand:` | gap | kcura-era config template, triage pending (TR6 BLOCKED). | HANDWRITTEN drift note |
| `schema/verify.ttl` | `hand:` | active | SOURCE root (C8 gate facts: ver:RequiredCheck battery + es:redStanding bindings; imported by ggen.toml; owns the scripts/verify-gates.sh and Makefile renders). | C8 |
| `schema/verify-gates.rq` | `hand:` | active | SOURCE root (SPARQL query driving both verify renders: render-verify-gates + render-makefile-verify). | C8 |
| `schema/changelog.ttl` | `hand:` | active | SOURCE root (consumer rel:Release / rel:ChangeEntry facts; castle-changelog-release-pack contract: pack ships zero individuals, consumer authors its own; owns the CHANGELOG.md render). | changelog composition (commit a09ce8a) |
| `schema/reconciliation-vocab.ttl` | `hand:` | active | SOURCE root (ret: retrofit vocabulary queried by the four C3 reconciliation rules). | C3 |
| `templates/CHANGELOG.md.tmpl` | `hand:` | active | SOURCE template for rule changelog. | changelog composition |
| `templates/receipt_contract_matrix.json.tmpl` | `hand:` | active | SOURCE template for rule receipt-contract-census. | C5 |
| `scripts/verify-gates.sh` | `ggen.toml` | active | Rendered by rule render-verify-gates (schema/verify.ttl facts + schema/verify-gates.rq). Runs the required-check battery, records real exit codes into scripts/verify-evidence.ttl, exits nonzero on any red gate. | C8 |
| `scripts/verify-evidence.ttl` | `scripts/verify-gates.sh` | active | Runtime evidence record (記): re-written by every verify-gates run with observed exit codes; committed as 証 (pattern: commit f3120fd). | C8 |
| `gates/dry-run-publish-gates.md` | `ggen.toml` | active | Rendered by rule dry-run-publish-gate-matrix (dry-run-publish-pack): 6-phase gate matrix + DRY-RUN-OVERCLAIM fence. | C4 |
| `gates/dry-run-publish/dry-run-publish-domain.ttl` | `ggen.toml` | active | Rendered by rule dry-run-publish-domain: PDDL8 STRIPS fragments + merged problem; the publish-fence tripwire lives in the owning rule, not in this render. | C4 |
| `gates/dry-run-publish/dry-run-publish-shapes.ttl` | `ggen.toml` | active | Rendered by rule dry-run-publish-shapes: SHACL shapes for a filled dry-run evidence graph. | C4 |
| `generated/receipt_contract_matrix.json` | `ggen.toml` | active | Rendered by rule receipt-contract-census (rp:FieldBinding facts, receipt-provenance-unification-pack); consumed by scripts/verify_publish_receipt.py — never hand-edit the matrix. | C5 |
| `scripts/verify_publish_receipt.py` | `hand:` | active | C5 provenance-gate executor: wires `ggen receipt verify` + `ggen receipt history` + the rendered census into the PUBLISH_RUNBOOK gate sequence; executor only — the field contract lives in schema/domain.ttl. | C5 |
| `docs/sjira/v26.9.21/G[0-9]*.md` | `scripts/gen_sjira_tickets_wave5.sh` | active | Same rule as the T/TR/C rows: the SPECS table in the script is the ontology; ticket History sections are the sanctioned append-only exception (票 law). | wave-5 crawl wave |
| `docs/sjira/v26.9.21/ADMISSION.md` | `hand:` | active | C9 deliverable: exact admission path for frozen-duckdb-pack + failed-edge ledger. | C9 |
| `docs/sjira/v26.9.21/CI_PROJECTION.md` | `hand:` | active | C11 deliverable: CI projection assessment — 5 ledgered failed edges; projection NOT viable this wave. | C11 |
| `docs/sjira/v26.9.21/PACK_FEEDBACK.md` | `hand:` | active | Wave-5 pack-feedback ledger: one appended section per G ticket (ticket-Mission-sanctioned append surface; created by G4). | wave-5 crawl wave |
| `docs/sjira/v26.9.21/reconciliation/差異受領台帳.md` | `ggen.toml` | active | Rendered by rule c3-reconciliation-ledger (ret: facts). | C3 |
| `docs/sjira/v26.9.21/reconciliation/状態照合.mmd` | `ggen.toml` | active | Rendered by rule c3-state-reconciliation-mmd (mermaid-pack flowchart). | C3 |
| `docs/sjira/v26.9.21/reconciliation/再認証判定.md` | `ggen.toml` | active | Rendered by rule c3-promotion-judgment. | C3 |
| `docs/sjira/v26.9.21/reconciliation/recertification-receipt.json` | `ggen.toml` | active | Rendered by rule c3-recertification-receipt (tcps-recertification-v1 receipts). | C3 |
| `docs/index.md` | `lineage:7f604e5` | gap | C2 Diataxis render (readme-diataxis-pack, 97 rdx: facts, frontmatter schema). Rule + facts absent from the canonical declarative union — re-render blocked on ggen schema convergence. In-place edits only through a HANDWRITTEN.md row (wave-5 G1). | C2 (merged via cae1a4d); lineage-pinned by wave-5 G4 |
| `docs/meta.md` | `lineage:7f604e5` | gap | Same lineage as docs/index.md. | C2 (merged via cae1a4d); lineage-pinned by wave-5 G4 |
| `docs/tutorials/getting-started.md` | `lineage:7f604e5` | gap | Same lineage as docs/index.md. | C2 (merged via cae1a4d); lineage-pinned by wave-5 G4 |
| `docs/how-to/configure-env.md` | `lineage:7f604e5` | gap | Same lineage as docs/index.md. | C2 (merged via cae1a4d); lineage-pinned by wave-5 G4 |
| `docs/how-to/deploy-test-and-use.md` | `lineage:7f604e5` | gap | Same lineage as docs/index.md. | C2 (merged via cae1a4d); lineage-pinned by wave-5 G4 |
| `docs/how-to/deploy.md` | `lineage:7f604e5` | gap | Same lineage as docs/index.md. | C2 (merged via cae1a4d); lineage-pinned by wave-5 G4 |
| `docs/how-to/run-tests.md` | `lineage:7f604e5` | gap | Same lineage as docs/index.md. | C2 (merged via cae1a4d); lineage-pinned by wave-5 G4 |
| `docs/how-to/usage-examples.md` | `lineage:7f604e5` | gap | Same lineage as docs/index.md. | C2 (merged via cae1a4d); lineage-pinned by wave-5 G4 |
| `docs/reference/api.md` | `lineage:7f604e5` | gap | Same lineage as docs/index.md. | C2 (merged via cae1a4d); lineage-pinned by wave-5 G4 |
| `docs/reference/environment-variables.md` | `lineage:7f604e5` | gap | Same lineage as docs/index.md. | C2 (merged via cae1a4d); lineage-pinned by wave-5 G4 |
| `docs/reference/functions.md` | `lineage:7f604e5` | gap | Same lineage as docs/index.md. | C2 (merged via cae1a4d); lineage-pinned by wave-5 G4 |
| `docs/reference/tech-stack.md` | `lineage:7f604e5` | gap | Same lineage as docs/index.md. | C2 (merged via cae1a4d); lineage-pinned by wave-5 G4 |
| `docs/explanation/faq.md` | `lineage:7f604e5` | gap | Same lineage as docs/index.md. | C2 (merged via cae1a4d); lineage-pinned by wave-5 G4 |
| `docs/explanation/lessons-and-optimizations.md` | `lineage:7f604e5` | gap | Same lineage as docs/index.md. | C2 (merged via cae1a4d); lineage-pinned by wave-5 G4 |
| `docs/explanation/related.md` | `lineage:7f604e5` | gap | Same lineage as docs/index.md. | C2 (merged via cae1a4d); lineage-pinned by wave-5 G4 |

## Known edge — C2 Diataxis renders (lineage-pinned, wave-5 record)

The `readme-diataxis-pack` composition (ticket C2, commit 7f604e5 on `feat/155-c2`)
rendered 15 Diataxis docs (`docs/tutorials/getting-started.md`, `docs/how-to/*`,
`docs/reference/*`, `docs/explanation/*`, `docs/index.md`, `docs/meta.md`) from 97
`rdx:` fact lines — but under ggen's **frontmatter schema** (`[packs.<name>]`
declaration), because the canonical **declarative schema cannot express the pack's
multi-SELECT templates** (failed edge ledgered in C2.md History and the HANDWRITTEN.md
`ggen.toml` row). The wave-4 canonical `ggen.toml` union kept the declarative form
(C3/C4/C5 rules), so the C2 rule is absent from the canonical config. The rendered
tree itself is PRESENT on this branch (merged via cae1a4d inside PR #3) and is
carried as 15 `gap` rows with `lineage:7f604e5` sources in the manifest above —
recorded reconciliation debt, not a silent prune (the pre-wave-5 wording "absent
from this branch" was falsified by G4's crawl). Consequence: those 15 docs are
existence-checked but not re-renderable here until the ggen schema convergence
lands; in-place edits are sanctioned only through HANDWRITTEN.md rows (wave-5
ticket G1 owns that crawl). `docs/{api,architecture,cli,contributing,guides,llm,
testing}/` remain the pre-wave hand-authored tree (sources, not C2 consequences).
When convergence lands, re-declare the pack, re-seed the `rdx:` facts (7f604e5 is
the lineage pin), re-render, and flip these rows to active `ggen.toml` rows in the
same change.

## Sources that carry no row (edited in place)

Everything not listed above is a SOURCE, not a consequence: application code under
`crates/` (except the vendored family and ledger rows), `docs/**` prose, root
`*.md`, `.cargo/config.toml`, `config.toml`, `.github/workflows/*.yml`
(hand-maintained; TR1/TR2 own them), and the remaining `scripts/` tooling. When a
pack lands in `[[ontology.pack]]`, its rendered consequences get NEW rows here —
a rendered file must never masquerade as a source.

## Reconciliation procedure (when a source changes)

1. Edit the source at its owner (pack ontology, generator spec table, upstream
   bump), never the projection.
2. Re-run the owning generator: `ggen sync run` for pack rules; the named script
   for repo-tooling rows; re-stage upstream artifacts on version bumps.
3. `make gates` (sync + genesis-check) must exit 0.
4. Commit source + re-rendered consequences atomically; update this manifest in
   the same commit whenever rows change.

## CI hook (PROPOSED — documented, NOT enabled)

Per C10 scope, auto-sync stays off. The proposed check-mode step for a future
`.github/workflows` job (operator cut required to enable):

```yaml
  genesis:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: ggen sync dry-run (no writes)
        run: ggen sync run --dry-run --format quiet
      - name: reconciliation manifest check
        run: make genesis-check
```

`--dry-run` resolves and renders without writing, so CI reports drift (a stale
projection or a vanished source) without mutating the checkout. Enabling this is
an operator decision recorded in MILESTONE.md, not an agent action.
