<<<<<<< HEAD
# PACK_FEEDBACK.md — pack-relevant findings from the wave-5 gap/staleness crawl

Findings from the G-ticket crawl fed back to pack authors. Append a section per
ticket; do not edit other tickets' sections. This file is session history (票 law).

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
=======
# PACK_FEEDBACK — wave-5 crawl findings for pack admission (v26.9.21)

Findings from the wave-5 gap/staleness crawl where falsification failed because
the owning capability is missing from the marketplace, or a manifest/fact file
drifted from its render source. Each row names the owning pack/family or ticket
so the finding can become an admitted fact instead of a hand-edited consequence.
Append-only per ticket; do not rewrite prior sections.



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
>>>>>>> feat/155-g2

