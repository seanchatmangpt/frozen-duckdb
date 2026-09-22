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
