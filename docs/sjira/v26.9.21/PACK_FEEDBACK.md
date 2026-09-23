# PACK_FEEDBACK

Pack-relevant findings fed back from the frozen-duckdb waves.



# PACK_FEEDBACK

Pack-relevant findings fed back from the frozen-duckdb waves.



# PACK_FEEDBACK

Pack-relevant findings fed back from the frozen-duckdb waves.




# PACK_FEEDBACK.md — wave 5 pack-relevant findings

Findings from the wave-5 crawl tickets that should become pack facts, pack
gates, or pack-owned surfaces instead of staying as repo-side hand repairs.
Append-only per ticket; the coordinator promotes into
`~/ggen-marketplace/packs/*` via marketplace admission.













## G9 — pack feedback: author updates (frozen-duckdb-pack)

Branch `pack/frozen-duckdb-capabilities-g9` (marketplace worktree
`~/ggen-marketplace-wt/g9`, base C6 @301ebc42f). Evidence base: this repo at
63070af (PR #3 merged, wave-4 composition landed).

1. **Five C6 laws verified, zero drift** (falsification pass against current
   code): cache normalization (`crates/frozen-duckdb-builder/src/lib.rs`
   `v{VERSION}-{arch}` under `~/.frozen-duckdb/cache`), sys build.rs
   `DOCS_RS` no-link branch + `rerun-if-env-changed=DOCS_RS` + `-Wl,-rpath`
   (crates/frozen-duckdb-sys/build.rs), version encoding (workspace 1.5.5 ↔
   `duckdb = "1.10505"` in test-dependency/Cargo.toml), asset naming
   (`libduckdb_{arch}.{dylib|so}`, lib.rs `release_asset_name_for` +
   prebuilt/setup_env.sh), compat symlinks (`libduckdb.dylib/.1.dylib/.1.4.dylib`).
2. **Four new laws authored into the pack** (five → nine): `ggen-consumer-config`
   (FM-CONFIG-003/101, `[ontology].imports` operative, schema exclusivity),
   `rendered-consequence-ownership` (Makefile/render-makefile-verify union
   clobber, wave-4 dry-run Fix 2), `fact-file-per-concern` (domain.ttl
   multi-writer hazard, FM-GEN-008 zero-rows class), `receipt-runtime-state`
   (`.ggen/`+`.ggen-v2/` untracked secrets, BLAKE3 chains). Evidence anchors:
   ggen.toml, CI_PROJECTION.md, Makefile, MILESTONE.md dry-run section,
   .gitignore, PUBLISH_RUNBOOK.md, .ggen-v2/receipt.json.
3. **Gates extended**: new `gates/025_consumer_config.rq` (config-form
   statuses; operative only for the two lawful forms; refusals must cite FM-
   codes) + structural checks 8–11 in `gates/030_structural_check.py`.
   Corrupted-fixture refusal proofs recorded: corrupt 030 exit 1 (7 refusals),
   clean 030 exit 0; corrupt 025 1 row, clean 4-gate battery 0 rows;
   `ggen graph validate` exit 0 (255 quads).
4. **Adoption contract corrected**: the README's `[[ontology.pack]]` form was
   falsified by this repo's own probe battery — the pack now instructs
   `[ontology].imports` (absolute paths) + `[[generation.rules]]`.
5. **Cross-pack falsifications (recorded, not edited)** — see G9.md History.

Pack bumped to 0.1.1. Commits: 056053cfc (laws+fixture), 7fa25d466 (gates),
c47375c03 (README+version). Never pushed; coordinator merges.

















## G6 — pack feedback: in-code doc-comment crawl (crates/** doc claims)

Branch `feat/155-g6` (worktree `~/frozen-duckdb-wt/g6`, base 63070af). Every fix
below was falsified by execution or external probe before editing; diff is
100% comment lines (mechanically checked). 301/301 tests green before AND after.

1. **duckdb-rs encoding law now stated at the point of use**: lib.rs before-state
   was `duckdb = "1.4.0"` (a real crates.io version, 2025-09-17, but the legacy
   plain series = DuckDB 1.4.x era). Fixed to `duckdb = "1.10505.0"` with the law
   in a comment: crate version = 1.MAJOR*10000 + MINOR*100 + PATCH; 1.10505.0 =
   DuckDB 1.5.5. Independently verified against crates.io (1.10500.0..1.10505.0
   all exist, newest 2026-07-22). Corroborates the pack law G9 cites from
   test-dependency/Cargo.toml. PACK CANDIDATE: expose this law as a pack fact so
   rendered docs never drift from it.
2. **Binary-size claims were 2x stale everywhere**: docs said 55MB (x86_64) /
   50MB (arm64); measured v1.5.5 dylibs are BOTH 117,005,184 bytes (~112MB) in
   ~/.frozen-duckdb/cache/v1.5.5-{arm64,x86_64}/. Fixed in lib.rs,
   architecture.rs, env_setup.rs. PACK CANDIDATE: size claims should render
   from release-asset metadata, not literals.
3. **CLI defect (recorded, NOT fixed — behavior edit out of G6 scope)**: clap
   short-flag collisions make two subcommands panic on ANY invocation in debug
   builds (clap debug_asserts): `convert` (-i on both `input` and
   `input_format`, -o on both output args) and `summarize` (-m on both
   `max_length` and `model`). Observed: `convert --help` and `summarize --help`
   abort with SIGABRT-style panic, exit 101. Needs a future code ticket: give
   input_format/output_format/max_length distinct or no shorts.
4. **LLM stub honesty**: `embed` and `search` always abort with
   "not implemented" panics (main.rs `.expect(...)`, exit 101 observed);
   generate_embeddings returns Err (array extraction TODO); semantic_search
   returns Err unconditionally. Help strings and doc comments now say so.
   `complete --input` reads the WHOLE file as one prompt (was advertised as
   one-per-line batch). `filter --prompt` has NO {{text}} substitution
   (advertised; never implemented). `summarize --strategy extractive` is the
   default fallback, not a distinct strategy. PACK CANDIDATE: a CLI-contract
   gate asserting help claims vs behavior for these five subcommands.
5. **Formats matrix falsified**: convert supports ONLY csv→parquet and
   parquet→csv (help advertised json/arrow). download formats are
   dataset-dependent: chinook rejects duckdb/arrow with a warning (observed,
   exit 0, only chinook.csv written); tpch accepts csv/parquet/duckdb with
   fallback. Help now states the matrix.
6. **Verbosity trap documented**: info/test/benchmark emit only `tracing` INFO
   logs; at default WARN the commands print NOTHING (observed: `info` silent,
   exit 0). Docs now say `-v` is required. PACK CANDIDATE: user-facing command
   output should be println!, not info! (behavior ticket candidate).
7. **Release reality**: NO frozen-duckdb release or tag exists (GitHub API
   returns []; asset URL 404). Builder comments previously implied macOS v1.5.5
   assets shipped; corrected to "all downloads 404 today; local compile is the
   operative path". TR2 coordination unaffected.
8. **Exit codes**: main.rs doc claimed exit 2 (env not configured) and 3
   (binary validation failed); grep + probes show NO code path emits either.
   Doc corrected: 0/1/4 plus 101 for the stub panics.
9. **TPC-H row counts**: sf=0.01 measured 86,805 data rows across 8 tables
   (lineitem 60,175, orders 15,000); docs said ~19,000 total, main.rs comment
   said ~1,500. Corrected at both sources.
10. **Verified-true comment claims** (falsifier attempted, claim survived):
    sys build.rs rpath — removing the `cargo:rustc-link-arg=-Wl,-rpath` line
    makes the sys lib unittest abort in the loader ("Library not loaded",
    SIGABRT observed); experiment reverted exactly, sys 89/89 green after.
    builder cache normalization (`v1.5.5-arm64/` with `duckdb/` headers and
    `libduckdb.dylib` symlink) matches the on-disk cache.

Commits: 008b1f9 (crate-root docs), 869a206 (CLI docs), 3f9d7c8 (builder docs).
Never pushed; coordinator merges.













## G2 — root docs + changelog facts crawl (2026-09-21)

| # | finding | repo evidence | proposed pack home / owner | disposition |
|---|---------|---------------|----------------------------|-------------|
| 1 | `docs/GENESIS.md` manifest still names `scripts/generate-changelog.sh` (conventional-changelog-cli) as the source of `CHANGELOG.md`; the actual owner since C7 is the ggen.toml rule `changelog` (`schema/changelog.ttl` facts → `templates/CHANGELOG.md.tmpl`, Overwrite mode). The stale row misdirects re-render procedure and contradicts the dispatch CHANGELOG law. | `docs/GENESIS.md` CHANGELOG.md row vs `ggen.toml` rule `changelog` (output_file CHANGELOG.md) | row repair in `docs/GENESIS.md` (hand: SOURCE, edited in place) | OPEN — owner ticket G4 |
| 2 | `HANDWRITTEN.md` intro asserts "This repo has **no ggen pack projection**: nothing here is rendered from a marketplace pack" — false since the wave-4 ggen bootstrap (11 generation rules, 13 ontology imports; Makefile, `CHANGELOG.md`, `scripts/verify-gates.sh`, gates/*, `generated/receipt_contract_matrix.json` are pack-rendered). The ledger's scope statement must track the 源 hierarchy or it licenses unledgered hand-edits of rendered files. | `HANDWRITTEN.md` head vs `ggen.toml` [[generation.rules]] census | intro repair in `HANDWRITTEN.md` (帳 ledger preamble, owner G4) | OPEN — owner ticket G4 |
| 3 | Release-asset shape contradiction: `.github/workflows/build-binaries.yml` publishes per-architecture single-slice dylibs (matrix `arch` × `CMAKE_OSX_ARCHITECTURES`, no lipo merge step), while README, MIGRATION_GUIDE, changelog fact f6, and `PR_BODY.md` claimed each released dylib is universal ("one ~117MB asset serves arm64 and x86_64"). The staged development cache IS universal (upstream binaries, `lipo` verified), which is where the universal claim crept in. G2 de-universalized its slice (README, MIGRATION_GUIDE, changelog-f6) to describe what the workflow provably publishes; `PR_BODY.md` "(universal arm64 + x86_64)" remains. Decide the asset law: either add a lipo merge step (docs re-universalize) or accept per-arch assets (fix PR_BODY.md). | `.github/workflows/build-binaries.yml` (matrix, no lipo) vs `lipo -info ~/.frozen-duckdb/cache/v1.5.5-arm64/libduckdb.dylib` (x86_64 + arm64) | CI asset-shape law — owner tickets G7 (workflow) + G8 (PR_BODY); coordinator decides | OPEN — G7/G8, coordinator cut |
| 4 | The Diataxis docs tree (`docs/{tutorials,how-to,reference,explanation,api,architecture,cli,contributing,guides,testing,llm}/…`, C2 renders) has no changelog entry under 1.5.5 and no `ggen.toml` render rule (its provenance is hand-rendered under C2; the unified-schema re-render is recorded as future in the HANDWRITTEN drift rows). Notable user-facing addition — candidate changelog fact once its render path settles. | `ggen.toml` output_file census (no docs/** rules) vs `docs/` tree; HANDWRITTEN.md drift row "C2 Diataxis re-render under unified schema" | `schema/changelog.ttl` facts (castle-changelog-release-pack) after provenance settles | OPEN — G1/G3 + coordinator |

### G2 verification notes (what held, for the record)

- README/QUICKSTART/MIGRATION claims verified TRUE against code or observation:
  three-crate workspace; vendored headers at
  `crates/frozen-duckdb-builder/vendored-headers/`; cache path
  `~/.frozen-duckdb/cache/v1.5.5-{arch}/`; CLI subcommands
  `download`/`convert`/`flock-setup`/`info` (`src/cli/commands.rs`); bin name
  `frozen-duckdb-cli` ([[bin]]); `frozen-duckdb-sys/build.rs` calls
  `ensure_binary()`; `dropin_compatibility_tests` exists; all four named
  examples exist; `docs/contributing/coding-standards.md` exists; `ci.yml`
  exists (badge target); LFS patterns for binary assets in `.gitattributes`;
  `ARCH` env touches only `prebuilt/setup_env.sh` + `architecture` module;
  Linux/Windows local-compile fallback (`lib.rs` asset law, `.so` assets
  correctly documented as not yet published).













## G3 — docs/ tree staleness crawl (docs/{architecture,guides,api,cli,contributing,testing})

| # | finding | evidence | proposed pack home |
|---|---------|----------|--------------------|
| G3-1 | CLI argument-surface uniqueness is an enforceable law: duplicate clap shorts made `convert`/`summarize` panic at parse (exit 101) while every compile gate stayed green. G3 hand-repaired it + added `crates/frozen-duckdb/tests/cli_surface_tests.rs` (clap `debug_assert()` + help render for every subcommand) as a permanent tripwire; row in HANDWRITTEN.md. | `frozen-duckdb-cli convert --help` → exit 101 "Short option names must be unique ... '-i' in use by both 'input' and 'input_format'"; `summarize` same with '-m'; all 12 subcommands exit 0 after ac77ae3 | frozen-duckdb-cli pack — argument-surface + help-render gate family (unadmitted) |
| G3-2 | Binary-name authority: the crate builds exactly one binary, `frozen-duckdb-cli` ([[bin]] in crates/frozen-duckdb/Cargo.toml), yet 222 doc invocations across docs/** named `frozen-duckdb <cmd>` and the *runtime error strings* still print `Run 'frozen-duckdb flock-setup' first` (src/main.rs). A pack fact "the [[bin]] name is the single CLI naming authority; docs and user-facing error strings derive from it" would have prevented the whole class. | grep counts per file in G3 receipt; src/main.rs:179 (and siblings) | frozen-duckdb-cli pack — naming-authority fact + string gate (unadmitted); the stale runtime strings in src/main.rs still need a code-lane fix |
| G3-3 | CLI docs are a projection waiting to happen: every falsified claim in docs/api/cli.md + docs/cli/* was a divergence between the clap `Commands` enum and hand-written prose (option tables, defaults, strategies, exit codes, format matrices). Deriving the reference tables from the enum (or a SHACL/gate diffing `--help` output against the docs) collapses the drift permanently. | 12 corrected option tables in commits 9d3939b/108716b; exit codes 2/3 in docs never existed in src/main.rs; convert matrix claimed 10 pairs, implementation has 2 | frozen-duckdb-cli pack — docs-rendered-from-enum family; or ggen-verify-pack gate `diff <(frozen-duckdb-cli --help --verbose) docs` (unadmitted) |
| G3-4 | Capability overclaims in dataset/convert surface (chinook `--format duckdb` no-ops with a warning and exit 0; convert implements only CSV↔Parquet; TPC-H fixed at SF 0.01) suggest a gate asserting documented capabilities ⊆ implemented capabilities per command. | crates/frozen-duckdb/src/cli/dataset_manager.rs (convert_dataset match arms; convert_chinook_to_format warn arm; dbgen(sf = 0.01)) | repo-hygiene pack — capability-census gate family (unadmitted) |
| G3-5 | Docs-tree integrity is gateable and cheap: (a) every relative markdown link in docs/** resolves, (b) every repo path referenced from docs/** exists. G3 found 4 dead links + a stale `src/` top-level claim; post-fix sweep reports 0 dead links / 0 missing paths. Natural reusable gate. | G3 receipt: dead-link sweep (6 found pre-fix, 0 post-fix); path sweep | ggen-verify-pack / repo-hygiene pack — docs-integrity gate (unadmitted) |
| G3-6 | OUT OF SLICE (for the owning ticket): docs/llm/ carries the same `frozen-duckdb <cmd>` drift (~20+ invocations in embeddings.md, semantic-search.md, ...). G3 did not touch it (slice boundary). | grep `frozen-duckdb (download\|embed\|search\|...)` over docs/llm/ | same as G3-2 |
| G3-7 | Test-count claims rot fast: docs asserted "30 core tests, 7 Flock failing" / "36% pass" / "27% success" while the measured truth is 301/301 (17 suites, exit 0). A pack gate pinning doc-quoted test counts to a generated receipt (verify-evidence.ttl) would keep prose honest or forceUNKNOWN markers. | build-optimization.md, overview.md ADR 003, flock.md, architecture-decisions.md (all fixed); `cargo test --workspace` exit 0, 301 passed | evidence-standing-pack — doc-claim/receipt binding (unadmitted extension) |
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
# PACK_FEEDBACK — wave 5 crawl findings for the marketplace (v26.9.21)

Feed for 法面 work (G9 and successors): findings the crawl tickets hit that belong
in `packs/`, not in this repo's hand-written surface. One section per ticket,
appended newest-last. A finding lands here only with evidence (command + exit or
source citation), never assertion.







## G7 — workflows + projection docs crawl (2026-09-21)

1. **github-actions-pack v0.3.2 — all five C11 failed edges re-verified STILL
   TRUE** (pack unchanged since `97e630cb9` 2026-09-10, tree clean; ggen still
   26.8.18):
   - FE-1 re-proven LIVE: transient `[[ontology.pack]]` in a consumer
     `ggen.toml` → `ggen sync run` exit 1, `[FM-CONFIG-003]` "unknown field
     `pack`, expected one of `source`, `imports`, `base_iri`, `prefixes`,
     `standard_only`" (frozen-duckdb worktree g7, 2026-09-21; config restored).
   - FE-2/3/4/5 source-confirmed on the current pack sources: gate 020 branch
     (3) 40-hex pin law; no `gha:withBlock` in ontology.ttl (with:/env: remain
     fixed-indent GROUP_CONCAT renders); unconditional top-level
     `permissions:`; GROUP_CONCAT ordering mechanism.
   - => The C11 paydown path is unchanged and still owed: `gha:withBlock`
     property + reindent rendering, conditional/job-level-only `permissions:`,
     and declarative-schema pack wiring. Until then, 508 hand-written workflow
     lines in this repo are unprojectable (CI_PROJECTION.md FE-1..FE-5).
2. **frozen-duckdb-pack exists pre-admission** — branch
   `pack/frozen-duckdb-capabilities` @ `301ebc42f`, v0.1.0 (pack.toml,
   ontology.ttl, 2 templates, 3 SPARQL gates + 1 executable structural gate,
   qualification/consumer.ttl, README). Absent from canonical `main`
   (`git show main:packs/frozen-duckdb-pack/pack.toml` → path does not exist);
   qualification court never ran against it; standing UNKNOWN.
   ADMISSION.md updated to this reality this wave.
3. **`actions/cache@v3` now trips actionlint's "runner too old" finding**
   (ci.yml + ci-simple.yml; bumped to @v4 this wave — live-runner proof lands
   with the next PR run). Note for any future gha: fact vocabulary: action
   *versions* rot independently of commit SHAs; gate 020 pins the ref, nothing
   notices a too-old major tag. A `gha:` freshness gate would close a gap the
   SHA law structurally cannot see.
4. **Builder arch detection vs CMAKE_OSX_ARCHITECTURES (future
   frozen-duckdb builder/CI pack family)**: post-TR7,
   `crates/frozen-duckdb-builder` passes the env to cmake (slice labeled
   correctly) but `detect_architecture()` still keys off `uname -m` and cache
   placement follows that detection — so the build-binaries x86_64 leg
   compiles a correct x86_64 slice, places it under
   `v1.5.5-arm64/libduckdb_arm64.dylib`, and the workflow find step fails
   closed (documented in build-binaries.yml this wave). Detection/placement
   should follow the env before the next tag cut; a pack expressing
   "multi-arch release staging" (ADMISSION ledger row 1 family) should carry
   this as a gate-able fact, not a convention.
=======
# PACK_FEEDBACK — v26.9.21 wave 5 (crawl findings → pack/marketplace feedback)

Append-only. One section per ticket. Rows name the pack-owning surface, the observed
defect or gap, evidence, and the intended owner. Nothing here mutates a pack directly;
the marketplace owner courts each row through admission.



## G10 — cross-cutting consistency crawler (2026-09-21)

| # | pack / surface | finding | evidence | intended owner | class |
|---|----------------|---------|----------|----------------|-------|
| 1 | `readme-diataxis-pack` `templates/index.md.tmpl:53` | Index template emits the Playground link (`[Playground](../playground/)`) unconditionally, but `playground/` is projected only when the consumer declares `rdx:PlaygroundFile` rows — consumers with zero playground rows get a permanent dead link in a GENERATED file they may not hand-edit. Observed in frozen-duckdb: `docs/index.md:34` → `FILE MISS`, repo has no `playground/`. Template should gate the Playground section on declared rows (same row-projection pattern as `rdx:ExampleFile` / `playground-file.tmpl`). | `grep -n playground /Users/sac/ggen-marketplace/packs/readme-diataxis-pack/templates/index.md.tmpl` → line 53 hardcoded; `ggen sync run` re-renders the link every run (hand-edit would be frozen out); link-checker over docs/**/*.md → 1 residual miss | readme-diataxis-pack author (upstream pack fix); consumer-side workaround: declare one playground row or accept the link | pack defect (template emits link for absent content) |
| 2 | `readme-diataxis-pack` index shape | The generated index covers only the 15 pack-rendered files (tutorials/how-to/reference/explanation/meta). Seven hand-authored doc families in the consumer repo (docs/{guides,architecture,api,cli,testing,llm,contributing}/) are unreachable from `docs/index.md` — 26 files including 4 zero-inbound-link files (docs/cli/dataset-operations.md, docs/contributing/development-setup.md, docs/how-to/deploy-test-and-use.md, docs/llm/semantic-search.md). Pack gap: no declared fact row family for "consumer-authored extra doc families" in the index projection. | index.md.tmpl section list vs `ls docs/{guides,architecture,api,cli,testing,llm,contributing}`; zero-reference grep over repo | readme-diataxis-pack author (extend ontology with a supplemental-index fact family) or consumer declares those families as pack facts | pack gap (index does not aggregate non-rendered families) |
| 3 | no pack expresses doc-link integrity | 7 dead internal links survived three waves of doc tickets (T4 swept prose, not hrefs). No marketplace pack/gate renders or verifies markdown link/anchor integrity over docs/**. A `docs-link-gate` (render-time or verify-time: resolve every relative href + heading anchor against the tree) would have caught all 7 at first render. | wave-5 G10 link-crawl: 7 FILE MISS across 3 files; none caught by T4's prose sweep | future docs-integrity pack (unadmitted) | capability gap |
>>>>>>> feat/155-g10


# PACK_FEEDBACK

Pack-relevant findings fed back from the frozen-duckdb waves.



# PACK_FEEDBACK

Pack-relevant findings fed back from the frozen-duckdb waves.



# PACK_FEEDBACK

Pack-relevant findings fed back from the frozen-duckdb waves.


# PACK_FEEDBACK — wave 5 (v26.9.21) gap/staleness crawl → pack feedback

Findings from the wave-5 crawl tickets that belong in pack facts, templates, gates,
or the ggen schema, so the next render cycle reproduces the corrections instead of
regressing them. Append one section per ticket. Section order = ticket order.

---













## G1 — readme-diataxis-pack (Diataxis renders)

All 15 C2 renders carry `GENERATED by ggen from readme-diataxis-pack` markers, but no
operative render path exists in the consumer (declarative ggen.toml cannot express the
pack's multi-SELECT templates — FM-CONFIG-003, GENESIS.md known edge). Every finding
below was hand-corrected in place (ledgered UNSUPPORTED in HANDWRITTEN.md) and must be
folded into the pack's fact-seeding + templates before the schema-convergence re-render,
or the re-render will regenerate falsehoods the crawl just removed.

### Fact-source defects (README is the pack's fact seed; these README claims are false against repo reality)

1. **"Universal binary" asset law is wrong.** README lines 14/21/40 (and every render
   echoing them: meta.md features, faq.md ARCH answer, tech-stack.md, lessons doc)
   claim each release asset is a universal arm64+x86_64 binary. Reality (builder
   `release_asset_name_for`, TR7 asset law, `.github/workflows/build-binaries.yml`):
   assets are **per architecture**, `libduckdb_{arch}.dylib`, one artifact per matrix
   leg. The docs' ARCH story inverted the real reason ARCH is inert: the builder
   ignores ARCH because it auto-detects via `uname -m` — not because assets are
   universal. Pack fix: seed `rdx:` facts from the builder's asset law, not README
   prose; a gate asserting "no universal-asset claim without a lipo step in the
   release workflow" would have caught this.
2. **Extension list overstates the build.** README lists Visualizer, TPC-E, Excel as
   bundled-and-enabled. Builder cmake (`crates/frozen-duckdb-builder/src/lib.rs`)
   builds only `parquet;json;icu;httpfs;tpch;tpcds;fts;inet;sqlsmith` + jemalloc +
   autoload; visualizer/tpce are out-of-tree in DuckDB 1.5.x and excel needs
   minizip-ng. meta.md now states the built set + autoload-on-demand for the rest.
3. **crates.io distribution claim unverifiable as written.** `frozen-duckdb = "1.5.5"`
   / `cargo install frozen-duckdb` do not resolve: crates.io has only `0.1.0`
   (verified via crates.io API, 2026-09-21). usage-examples.md now shows the git
   dependency and `cargo install --path crates/frozen-duckdb`. Pack fix: distribution
   facts need a provenance check (query crates.io at render time) or a gate refusing
   version claims that outpace the registry.

### Template / pack-mechanism findings

4. **`playground-file.tmpl` renders a nav link without materializing the playground.**
   docs/index.md shipped `[Playground](../playground/)` — the directory never existed
   in this repo (git history: absent since inception); the link was dead from the
   first render. Replaced with the real, executed crate-examples entry. Pack fix:
   a template that emits a link must either render its target or the gate must check
   link targets (see 6).
5. **HTTP-API reference residue on a Rust library.** reference/api.md rendered the
   pack's `rdx:ApiEndpoint`/`rdx:ApiParameter` tables (Method|Path) — empty, and
   meaningless for a crate with no HTTP surface. Replaced with the real re-export
   table + `cargo doc` pointer. Pack fix: the API template should project the
   language's real surface (rustdoc for crates) or refuse when no endpoint facts
   exist, rather than emitting an empty HTTP skeleton.
6. **Pack gates check RDF shape, not rendered truth.** Both pack gates
   (010_required_properties, 020_ordering_integrity) validate fact completeness and
   ordering; none validates rendered output against reality (dead links, claimed
   commands, claimed env vars). The crawl found: an env var (`RUST_LOG`) documented
   but read nowhere; `contributing.md` and `code of conduct` paths that don't exist;
   an empty Support section (template emits the heading with no ContactChannel facts).
   Pack fix: add a post-render gate family — link-target existence, command
   executability (recipe smoke), env-var presence in source.

### Consumer defects discovered by executing the docs (not pack defects, recorded for the ledger trail)

7. **CLI `convert` panicked on every invocation** (clap debug assert: short `-i` bound
   to both `input` and `input_format`; `-o` similarly doubly bound). Fixed long-only
   for the format args; documented recipe then executed green (csv→parquet, exit 0).
8. **CLI `summarize` panicked likewise** (`-m` bound to both `max_length` and `model`).
   Fixed; full 12-subcommand registration sweep now panic-free.
9. **`convert --output-format json` is advertised by the flag's own help** ("csv,
   parquet, json, arrow") but rejected at runtime ("Unsupported conversion: csv to
   json"). Left for the CLI owner; the documented recipe (csv→parquet) is true.

### Verified-alive renders (no correction needed)

- tutorials/getting-started.md: clone URL matches `git remote -v`; `cargo build
  --workspace` exit 0; `cargo run --example dropin_replacement` exit 0 (added the
  executed test-suite step).
- how-to/deploy.md: tag→release trigger verified against `build-binaries.yml`
  (`on: push: tags: v*`); execution correctly not attempted (権: no tag/push).
- reference/functions.md: all four signatures verified in vendored source
  (`Connection::open_in_memory`/`execute_batch`/`prepare`, `Statement::query_row`).
- explanation/related.md: all four link targets verified.

---
# PACK_FEEDBACK.md — wave 5 (v26.9.21)

Findings from the wave-5 crawl tickets that point at missing or extendable
pack capabilities. One section per ticket, appended in merge order. A row
here is a proposal for marketplace admission, not an admitted fact.









## G5 — scripts/ crawl: headers + drift

| # | finding | evidence | proposed owner pack / family | type |
|---|---------|----------|------------------------------|------|
| 1 | Script STATUS headers are a recurring hand-authored mechanism (TR6 seeded 7, G5 verified/edited them). No pack renders or validates them; stale claims were only caught by hand crawling. | TR6 History row 8; G5 live re-verification (test_ffi_simple.sh exit 1; run_ffi_validation.sh Go-leg exit 1) | repo-hygiene pack — script-metadata family: render/validate STATUS blocks + a gate that flags headers older than the claims they name (e.g. "library resolution" claims vs current builder layout) | EXTEND/INVENT |
| 2 | No static fake-pattern gate exists after the kcura-era scripts were removed (TR6: ci_gate/ci_gates/fake_guard; G5: scan_fakes*, redteam_probe.rs). The one surviving sweep (rg_sweep.sh) is informational-only. | G5 removal evidence (bash -n failures, `declare -A` under bash 3.2, manifest exit 101, mock probes); trimmed rg_sweep.sh live run | ggen-verify-pack family: add a `ver:RequiredCheck` fact in schema/verify.ttl for a de-fakery token scan bound to REAL crate paths (rendered into verify-gates.sh) — not loose shell scripts | EXTEND |
| 3 | Shell portability law is unwritten: the entire kcura tooling cluster died under stock macOS bash 3.2 (`declare -A` is bash-4 syntax; `#!/usr/bin/env bash` resolves to 3.2.57 here). | G5 live probes: scan_fakes.sh exit 2 (lib/config.sh:39); scan_fakes_core_team.sh death at line 29 | repo-hygiene pack — shell-portability gate: bash -n plus a bash-3.2-compat scan (or a declared interpreter requirement) for scripts/*.sh | INVENT |
| 4 | `prebuilt/setup_env.sh` resolves `DUCKDB_LIB_DIR` from `$0`, which is wrong for every `source` caller; two tickets (TR6, T6 handoff) have now tripped over it. | G5 live: sourced from scripts/ callers it exports `.../scripts` as the lib dir and even prints `source .../scripts/setup_env.sh` in its usage text | frozen-duckdb-builder pack — env-resolution family: `BASH_SOURCE`-based resolution law (the T6 repair lane; the fact belongs in ontology so the repair is a re-render, not a one-off) | EXTEND |
| 5 | Residual kcura branding/probes remain in live scripts after G5's bounded scope: functional dead refs in `spec_sync_check.sh` (10 refs; targets `docs/specs/errors.yaml` which does not exist), `use_prebuilt_duckdb.sh` (3), `ffi_constant_return_check.py` (3), `open_gaps.sh`, `setup_optimized_build.sh`; cosmetic KC_*/kcura-* strings in builder scripts and lib doc text. | G5 grep census (33 files with kcura/kc_/KCura mentions; 7 with functional path refs) | repo-hygiene pack — naming-law family: a gate that greps scripts/ for dead-brand symbols; remaining cleanup is T6-adjacent paydown, ledgered here so it is not silently pruned | EXTEND |
| 6 | The central-cache layout law (`~/.frozen-duckdb/cache/v{VER}-{arch}/`, arch-normalized names, neutral link names) is recorded in HANDWRITTEN row 3 but no gate verifies prose claims about it; TR6/G5 STATUS headers had to be hand-re-verified twice in one day. | HANDWRITTEN.md row 3; TR6 + G5 re-verifications of the same claims | frozen-duckdb-builder pack — cache-layout family: a verifier that checks lib resolution claims in scripts/docs against the actual builder output | EXTEND |
# PACK_FEEDBACK — v26.9.21 wave 5 (gap/staleness crawl → pack feedback)

Findings from the wave-5 crawl tickets (G1..G10, manufactured at 63070af).
One section per ticket; pack-relevant findings only — things whose lawful home
is a pack fact, template, or gate, not repo-side prose debt. Appended by each
G-ticket in its own worktree; coordinator merges.





## G8 — release docs coherence crawl (slice: PUBLISH_RUNBOOK, MILESTONE, PR_BODY, ISSUE_1_COMMENT, TPUB)

Fixed in-repo this wave at the prose layer (lawful per G8 scope: "doc text in
place for prose"); the durable fixes belong upstream in the named packs.

1. **dry-run-publish-pack: phase-1 evidence command hardcodes PR state.** The
   rendered `gates/dry-run-publish-gates.md` phase-1 row carries
   `gh pr view 3 … (OPEN)` — stale the moment the PR merges (now MERGED at
   a3a69e4). The render cannot be hand-edited (器: EDIT THE SOURCES), so the
   stale annotation persists until a source edit + re-render. Pack
   improvement: express the expected PR state as a lifecycle (OPEN→MERGED) or
   bind it live, so phase-1 evidence survives the merge event. Recorded here,
   NOT fixed (owned by the pack/schema sources).
2. **PUBLISH_RUNBOOK is manual composition of three sources** (TPUB base
   render + C4 modeled-gate section + C5 provenance gate), coherent only after
   a wave-5 crawl stitched them: "above/below" wording drift, the provenance
   gate absent from the linear cut sequence and from phase rows 2/6 until
   harmonized. Pack improvement: a runbook-assembly template that renders the
   operator procedure as ONE document — base steps + phase mapping +
   provenance gate composed at render time, not crawl time.
3. **The overclaim fence does not propagate to comms artifacts.**
   DRY-RUN-OVERCLAIM binds the modeled domain and gate language, but
   ISSUE_1_COMMENT.md carried present-tense publication claims ("it shipped",
   "v1.5.5 is out") conditioned on post timing only by a prose subtitle — and
   the runbook's original step-10 command would have posted the meta header
   verbatim. Pack improvement: comms templates render with a structural
   precondition fence header plus a machine-checkable body-extraction rule
   (post-condition: publish steps complete; post the body only).
4. **Merge-event staleness is structural.** Runbook step 1 ("Merge PR #3",
   `# OPEN`) and the `<merge-sha>` tag placeholder went stale at the merge;
   issue #1 flipped to CLOSED by the same merge ("closes #1" auto-closure) —
   three facts in two files needed a crawl to reconcile. Pack improvement:
   parameterize the runbook render on release-state facts (merge SHA, tag
   existence, issue/PR state) or declare the periodic crawl the lawful
   reconciler in the pack itself.
5. **C5 gate placement.** The provenance gate's "run before every cut and at
   close-out" lived outside the linear cut sequence (only the close-out step
   mentioned it). Pack improvement: phase mappings should natively include the
   provenance-gate rows (phase 2 pre-cut law; phase 6 close-out re-run with
   `chain_hash`), as now harmonized in the runbook.
