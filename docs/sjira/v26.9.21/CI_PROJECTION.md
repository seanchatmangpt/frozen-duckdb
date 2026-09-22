# C11 — CI Projection Assessment (v26.9.21, wave 4)

Ticket: `docs/sjira/v26.9.21/C11.md` · Branch: `feat/155-c11` · Worktree: `/Users/sac/frozen-duckdb-wt/c11`
Subject: can `packs/github-actions-pack` and/or `packs/cargo-cicd-pack` project this repo's
`.github/workflows/{ci,ci-simple,test-minimal,build-binaries}.yml` from facts, preserving every
wave-3 law (TR1/TR2 hand-repairs — 帳 debt)?

## Verdict

| Pack | Capability family | Verdict |
|------|-------------------|---------|
| `github-actions-pack` v0.3.2 | MATCH — schema-only `gha:` vocabulary + real workflow renderer (`workflow.yml.tmpl`, frontmatter `to:/sparql:/for_each:`) + 5 executable refusal gates | **Projection NOT viable this wave.** Five ledgered failed edges (FE-1..FE-5). No workflow touched — wave-3 surface preserved byte-for-byte, zero regression. |
| `cargo-cicd-pack` v0.1.0 | MISMATCH — name-topology near-fit only | **No projection capability.** Renders Rust catalog/dispatch/proof code for the `cargo-cicd` CLI (crates.io `cargo-cicd`), not CI pipelines. |

比例 truthfully stated: 0% of this surface manufactured this wave. The 帳 debt remains 508
hand-written workflow lines; this document is the ledgered paydown plan (per 帳 law, growing
debt requires a paydown plan in the same change — the plan is the "Paydown path" section).

## Wave-3 laws inventoried (the preservation bar)

Every item below is load-bearing and cited in the workflow comments themselves:

- **ci.yml**: `fail-fast: false` (one red leg must not cancel others — observed 2026-09-21);
  matrix `{ubuntu,macos} × {stable,beta}`; `timeout-minutes: 45` (kills the ~1h silent
  source-compile fallback); `components: rustfmt, clippy` on dtolnay/rust-toolchain (beta legs
  lack them); DuckDB cache pre-seed (~35-line run block: upstream release zip download, per-OS
  asset selection `libduckdb-osx-universal.zip` / `libduckdb-linux-{amd64,arm64}.zip`,
  `libduckdb_<arch>.<ext>` naming + plain-name symlink law, `set -euo pipefail`); 3 jobs total.
- **ci-simple.yml**: same pre-seed; single job; "for act testing".
- **test-minimal.yml**: env checks, `prebuilt/` + `setup_env.sh` assertions, pre-seed, build.
- **build-binaries.yml**: tag-push `v*` + `workflow_dispatch` inputs; macos matrix
  `{x86_64, arm64}`; **`CMAKE_OSX_ARCHITECTURES=${{ matrix.arch }}`** (arm64 runner mislabels
  x86_64 legs otherwise); brew cmake; temp-project mega-library build via
  `frozen-duckdb-builder`; **staged asset name `libduckdb_{arch}.dylib` matching the builder's
  download URL**; artifact upload; release job (`softprops/action-gh-release@v2`,
  `GITHUB_TOKEN` env) gated `if: startsWith(github.ref, 'refs/tags/')`.

## Evidence (証) — every gate executed this session, command + exit preserved

Probe: transient `gha:` instance facts (a representative mini-workflow carrying the wave-3 edge
cases: `onBlock` with branch filters, `workflowEnv`, `jobExtra` with strategy/fail-fast, mutable
then pinned action refs, multi-line `with:` values, multiline run block) wired to the pack, then
**fully reverted** — tree clean at `304a958`.

| # | Command | Exit | Result |
|---|---------|------|--------|
| 1 | `ggen sync run` (bootstrapped config) | 0 | baseline, `"packs": {}` |
| 2 | `ggen.toml` + `[[ontology.pack]]` (per ticket/bootstrap contract) | 1 | `[FM-CONFIG-003]` unknown field `pack` — **`[[ontology.pack]]` does not exist in ggen 26.8.18** (`[ontology]` accepts only `source/imports/base_iri/prefixes/standard_only`) |
| 3 | `ggen.toml` + `[packs]` (the pack example's own wiring) | 1 | `[FM-CONFIG-101]` schema ambiguity: `frontmatter:packs_table_shaped` conflicts with `declarative:{sync,generation,project_version}` markers |
| 4 | `[[generation.rules]]` `template = { pack = ... }` — 3 shapes (`"name/path"`, `name=`, nested table) | 1 each | `[FM-CONFIG-003]` "data did not match any variant of untagged enum `TemplateSource`" — **no pack-sourced template variant exists in 26.8.18** (declarative path dead; pack.toml itself documents this sourcing as broken `[FM-GEN-007]` in 26.8.8) |
| 5 | frontmatter-schema consumer (`[project]+[ontology]+[templates]+[packs]`, the pack example's exact shape) | 1 | **Pack discovered; gates execute against the UNION graph.** `[FM-PACK-013]` gate `020_security.rq` REFUSED: 2 rows — mutable third-party action refs `actions/checkout@v4`, `actions/cache@v3` |
| 6 | same, with real 40-hex pins (resolved this session: `git ls-remote` → checkout `11d5960a…` # v4, cache `6f8efc29…` # v3) | 0 | rendered `.github/workflows/probe-projection.yml`; all 5 pack gates ran (010/020/030/040/050 closures in receipt) |
| 7 | re-render with corrected `onBlock`/`jobExtra` authoring (falsifier: initial corruption was my authoring, not the template's) | 0 | `onBlock`, `workflowEnv`, job name/runner/timeout, `jobExtra` strategy/fail-fast/matrix, step `uses`/`run` all render **correctly** |
| 8 | `python3 -c "yaml.safe_load(...)"` on the render | 1 | **`YAML-PARSE: FAIL`** — line 36, the multi-line `with:` block |
| 9 | `actionlint .github/workflows/probe-projection.yml` | 1 | `36:10: could not parse as YAML` — independent confirmation |
| 10 | `git checkout -- ggen.toml schema/domain.ttl && rm -rf .ggen-v2 .ggen ggen.lock templates .github/workflows/probe-projection.yml` | 0 | tree clean at `304a958` |

## Failed edges (帳 ledger — `failed(edge) ≠ failed(G)`; silent pruning forbidden)

**FE-1 — consumer wiring (ggen 26.8.18 × this repo's declarative bootstrap).**
The bootstrapped `ggen.toml` uses the declarative schema (`[project].version`, `[generation]`,
`[sync]`, `[rdf]`). Under that schema, pack templates are unreachable: `[[ontology.pack]]` does
not exist (evidence #2), `[packs]` is ambiguity-barred (evidence #3), `TemplateSource` has no
pack variant (evidence #4). The pack's designed wiring is the frontmatter schema (evidence #5) —
adopting it means **replacing** the bootstrap config, contradicting the ticket contract
("declare via `[[ontology.pack]]`"). This is a ggen/marketplace-side convergence gap, not
fixable in this repo alone.

**FE-2 — gate 020 vs the as-built action-ref surface.**
The pack executes its gates at sync against the union graph and **refuses** the exact refs wave
3 validated: `actions/checkout@v4`, `dtolnay/rust-toolchain@stable`, `actions/cache@v3`,
`actions/upload-artifact@v4`, `softprops/action-gh-release@v2` (evidence #5; probe carried 2 of
the 5). A gate-compliant projection must pin all five to 40-hex SHAs. That changes the precise
surface wave 3 hand-validated on real runs, and the change cannot be verified this session (no
GitHub-hosted run is executable from here — `inspection ≠ execution`). A projection that cannot
be proven equivalent is a regression risk under the ticket's own bar, so it does not land.

**FE-3 — multi-line `with:` values are structurally inexpressible (hard blocker).**
`workflow.yml.tmpl` renders every `gha:withParam` as a single line (`{{ kv }}` at fixed indent;
values arrive via `GROUP_CONCAT` string join). Block scalars are therefore impossible:
`actions/cache`'s `path: |` (3 paths), `softprops/action-gh-release`'s `body: |` and `files: |`
all render as bare sibling lines + an empty block scalar → **hard YAML error**, confirmed by two
independent parsers (evidence #8, #9). I attempted the workaround forms (separate `withParam`
rows per continuation line): they emit at the same indent as the key and misparse. No
authoring-side fix exists; the fix is upstream (a `gha:withBlock` property + `reindent` rendering
in the template — an EXTEND-rung marketplace change).

**FE-4 — unconditional top-level `permissions:` block.**
The template always renders `permissions: <ceiling>`. None of the four workflows declares one
(they run on the default token). Rendering `contents: read` silently narrows scope;
`build-binaries`' release job in fact needs `contents: write`, which gate 020 branch (2) refuses
unless modeled via `gha:performsOperation → gha:requiresPermission` derivation. Either choice
produces a security surface ≠ wave 3, on the same untestable-this-session basis as FE-2.

**FE-5 — `with:`/`env:` ordering is not authoring order (minor, mechanism note).**
`GROUP_CONCAT(DISTINCT ...)` yields implementation-defined order (observed reversed + sorted).
Semantically tolerable for GitHub Actions maps, but it proves values pass through string
concatenation — the same mechanism that makes FE-3 unfixable authoring-side.

**Bootstrap observation (for a future wave, in-scope file, not changed here).**
`[generation] rules = []` + the comment "append `[[generation.rules]]` entries" collide: TOML
forbids `rules = []` and `[[generation.rules]]` for the same key (`[FM-CONFIG-103]` duplicate
key, observed). The first wave that lands a real rule must delete `rules = []` in the same
commit.

## What the pack does render correctly (credited — the real ladder rung)

Under the frontmatter wiring with gate-clean facts: workflow `name`, `on:` via `onBlock`,
`env:`, `concurrency:`, `jobs:` with key/`runs-on`/`timeout-minutes`/`needs`/`if`/
`jobExtra` (strategy, fail-fast, matrix — exact YAML), ordered steps (`stepOrder`) with
`name`/`id`/`if`/`uses`/single-line `with`/`env`/`run:` (bash) — plus fail-closed gate
execution at sync, a per-file provenance header, and refusal output naming the exact offending
subject. The capability family is right; the gaps are specific and upstream-sized.

## Paydown path (帳 paydown plan for the 508 lines)

1. Marketplace (EXTEND `github-actions-pack`): add `gha:withBlock` (reindented block-scalar
   `with:` values — closes FE-3/FE-5) and render `permissions:` only when declared or
   `job-level-only` (closes FE-4).
2. ggen (tooling): declarative-schema pack wiring, or bless `[packs]` under the declarative
   schema (closes FE-1).
3. Operator decision (権): SHA-pinning policy for the five action refs — only after 1+2, and
   with a real runner run as evidence (closes FE-2).
4. Then: author `gha:` facts for the four workflows, render, diff against wave-3 files
   semantically, gate with `actionlint` + a real triggered run before deleting the hand-written
   originals.

## cargo-cicd-pack — failed edge

`failed(edge_cargo_cicd)`: the name invites a CI/CD reading; the pack is none. Its ontology is
the audited command surface of the `cargo-cicd` CLI (cnv: noun/verb reuse, 52 rows / 21 nouns,
`cc:sourceFile` citations; known disclosed gap: `src/nouns/lsp.rs` unaudited). Its five
templates render `src/cargo_cicd_{catalog,dispatch,dispatch_proof}.rs`, a catalog proof, mod
wiring, and a reference doc — zero YAML, zero `.github/workflows/` capability, no workflow
vocabulary. frozen-duckdb has no `cargo-cicd` dependency, so nothing is composable here either
(fan-out of one query against `packs/` by capability family, not name — the name is the
near-fit topology, recorded per 階 law).

## Receipt

- Repo: frozen-duckdb · base `304a958` · branch `feat/155-c11` · worktree `/Users/sac/frozen-duckdb-wt/c11`
- Gates: `ggen sync run` exit 0 (baseline + pinned-probe render + final restored config); `yaml.safe_load` FAIL and `actionlint` exit 1 on the probe render (the finding, preserved); `cargo build --workspace` green (see ticket History row for the run)
- 比: 0/508 workflow lines manufactured this wave (truthfully reported; paydown path above)
- Ledger deltas: +5 failed edges (FE-1..FE-5) + 1 bootstrap observation + 1 cargo-cicd family edge
- Standing deltas: github-actions-pack ASSESSED (viable family, blocked landing), cargo-cicd-pack REFUSED (family mismatch)
- Falsifiers attempted: pack-fairness re-render (my authoring vs template defect — 2 corruptions were mine, corrected); single-vs-two parser confirmation; TemplateSource shape space (3 variants); both wiring schemas
- What the operator did NOT have to write: this assessment, the probe, and the evidence battery — no production byte was hand-written this wave
