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
