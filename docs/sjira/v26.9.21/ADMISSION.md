---
id: C9-ADMISSION
dcterms:title: "admission path for frozen-duckdb-pack + failed-edge ledger"
dcterms:created: "2026-09-21"
dcterms:isPartOf: "v26.9.21"
dcterms:source: "packs/marketplace-governance-pack, packs/pack-compatibility-pack, packs/pack-maturity-pack (~/ggen-marketplace)"
rdf:type: oslc_cm:ChangeRequest, earl:TestRequirement
aps:standing: https://w3id.org/chatman/aps#ALIVE
milestone: frozen-duckdb v1.5.5 wave 4 (marketplace capability composition)
---

# ADMISSION — exact admission path for `frozen-duckdb-pack` + FAILED-EDGE LEDGER

Ticket C9. Two deliverables: (1) the exact admission path the parallel-authored pack
`packs/frozen-duckdb-pack` (marketplace worktree `/Users/sac/ggen-marketplace-wt/c6`,
branch `pack/frozen-duckdb-capabilities`) must traverse to become an admitted
marketplace pack; (2) the FAILED-EDGE LEDGER — capabilities this repo needed that no
marketplace pack expresses — feeding the pack author as the prior-art-closure evidence
that makes INVENT lawful (階 step 4).

At authoring time the C6 worktree carried no `packs/frozen-duckdb-pack` yet (checked
2026-09-21: `ls packs/ | grep -i -E 'frozen|duckdb'` → empty on branch
`pack/frozen-duckdb-capabilities`), so the path below is stated generically with pack
citations; it binds the moment the pack directory exists.

## 0. 源 hierarchy (what admission means)

`marketplace.toml → packs/<name>/pack.toml → ontology.ttl → templates/ → gates/`
( Consumer `ggen.toml` [[ontology.pack]] declarations project from it; the catalog is a
deterministic projection — never a second hand-edited copy. )

Admission, mechanically, is: the pack directory exists on the canonical source
(`[source_authority] repository = "seanchatmangpt/ggen-marketplace"`, canonical_branch
`main`, `mirrors_are_provenance_only = true` — marketplace.toml header), passes the
structure inspection (`scripts/marketplace.py::inspect_marketplace()`: pack.toml present
and TOML-valid, no symlinks under `packs/`, per-pack facts coherent), and survives the
real-runtime qualification court with an ALIVE receipt. Branch ancestry is provenance,
never authority — the C6 feature branch confers nothing until merged and qualified.

## 1. 階 compliance (why INVENT is lawful here)

Search ladder run and recorded before authorizing a new pack (evidence in the ledger's
"packs searched" column and the session receipt in §5):

1. REUSE — no existing pack carries any frozen-duckdb capability (both searches below).
2. COMPOSE — nothing exists to compose for these capabilities.
3. EXTEND — no adjacent family to extend (nearest hits are incidental string matches,
   enumerated per row — `failed(edge_i) ≠ failed(G)` recorded, nothing silently pruned).
4. INVENT — therefore lawful, but only under `mg:MinimumNoveltyAdmissionLaw`
   (marketplace-governance-pack/ontology.ttl):
   `mg:requiresPriorArtClosure true` — this ledger is that closure;
   `mg:requiresResidualFalsifier true` — each row names the search that would have found
   a pack and found none;
   `mg:grantsDoAuthority false` — admission never grants consequence-bearing DO
   authority.

## 2. The exact admission path (steps, checks, cutters)

### Step 1 — Construct the pack through the canonical constructor

`pack-authoring-pack` is **deprecated as an independent pack-constructor authority**
(pack.toml: "new pack construction should enter through ggen-self-pack / `ggen pack new`
so structural requirements have one semantic owner"). C6 must scaffold via `ggen pack
new`; its retained `templates/scaffold_{pack_toml,ontology_ttl,gate_rq,template}.tmpl`
show the expected shapes.

### Step 2 — Required structure (enforced by scripts/marketplace.py::inspect_marketplace)

```
packs/frozen-duckdb-pack/
  pack.toml          # [pack] name, SemVer version, description
  ontology.ttl       # RDF/Turtle facts, pack-owned namespace, real individuals
  templates/         # generation templates rendering consumer consequences
  gates/             # SPARQL gates; any returned row refuses manufacture
  targets.toml       # optional target-language declarations
  qualification/consumer.ttl   # optional isolated-qualification fixture
```

Precedents this shape is transcribed from: marketplace-governance-pack (pack.toml,
ontology.ttl, gates/01-lifecycle-eligibility-law.rq, qualification/consumer.ttl),
pack-maturity-pack (same four + five-quadrant templates), pack-compatibility-pack
(pack.toml + ontology.ttl + gates/010_requirement_shape.rq).

Structural law:

- **pack.toml**: `name` matches the directory; `version` is real SemVer
  (precedents: 26.9.12, 0.1.0, 0.1.1, 0.2.0); `description` states what capability the
  pack owns and how each fact was verified, not marketing prose (every law pack's
  description carries its verification story — pack-compatibility-pack's description is
  the canonical example).
- **ontology.ttl**: a pack-owned namespace (precedents `mg:`, `pc:`, `mp:`,
  `https://ggen.dev/ontology/...`); individuals carry real, checkable facts with
  provenance comments naming where each was verified from. Cross-pack object references
  must type for real in the union graph — `scripts/check_cross_pack_references.py`
  refuses dangling types (the RC2 lint, added after standing-ladder-pack's instance of
  this exact defect class; see pack-maturity-pack/qualification/consumer.ttl header).
- **gates/**: SPARQL SELECT gates whose contract is "any returned row refuses
  manufacture", each carrying a `# MESSAGE:` comment naming the law it enforces
  (marketplace-governance-pack/gates/01-lifecycle-eligibility-law.rq is the reference
  form). Gates must be namespace-scoped to the pack's own IRI space so they do not
  wrongly demand pack facts from other packs' individuals (rmcp-pack gate rationale).
- **qualification/consumer.ttl** (optional but precedented): synthetic consumer facts
  unioned only with the pack's own ontology.ttl during isolated qualification — never a
  sibling pack's real ontology (qualify_packs.py isolation rule).

### Step 3 — Governance law check (marketplace-governance-pack)

The pack must qualify as a `mg:CanonicalPack` — "carries durable semantic authority or
an independently consumable capability" — not a technology/release/experiment/maturity
variant: `mg:AdmissionLaw`: "Technology, releases, experiments and maturity variants are
profiles, fixtures, data or versions" (i.e. NOT new top-level packs). The
frozen-duckdb capability family passes this test as stated: prebuilt-dylib acquisition,
staging, link policy and build policy are durable, independently consumable semantics;
the pinned DuckDB 1.5.5 version itself is a profile/individual inside the pack, never
the admission's subject.

Standing: new packs enter as `mg:standing "CANDIDATE"`; QUALIFIED is granted only by a
real `mg:QualificationReceipt` (`mg:forPack`, non-empty `mg:receiptStatus` — the shape
enforced by gates/01 — plus `mg:qualifiedOn`). The refusal shapes the pack's own facts
will be judged by are executable and cited: gates/01-lifecycle-eligibility-law.rq
refuses (a) a LegacyPack with no replacementDecision, (b) an unresolvable decision, (c)
`lifecycleEligible true` without all four preconditions true, (d) ABSORB without a
canonical target, (e) a receipt missing pack or status.

### Step 4 — Qualification court (the only ALIVE-granting surface)

`python3.12 scripts/qualify_packs.py` (tomllib requires 3.11+) qualifies every admitted
pack through the real ggen runtime: each pack is loaded through ggen **twice** in an
isolated temp capsule and must **converge to the same filesystem consequence** within a
per-pass bound (marketplace.toml `[qualification] workers = 4, timeout_seconds = 5`).
This is `mg:replayVerified` made mechanical — two sync passes, identical output. One
opt-in-by-presence exception (CI-05): if the generated tree contains Cargo.toml files
the sync pass actually targeted, they are really `cargo build`'d/`cargo test`'d (120 s
cap) — a pack rendering a broken manifest cannot qualify ALIVE.

Outcome vocabulary is the standing vocabulary: ALIVE | WARN | REFUSED:* — recorded as a
QualificationReceipt; `ALIVE` on this court is what moves the pack CANDIDATE → QUALIFIED.
This ticket did NOT execute the court (it qualifies every admitted pack; the
frozen-duckdb-pack does not exist yet) — court standing for the new pack is UNKNOWN
until C6 lands and the court runs. Claimed-ALIVE-before-court = fabricated evidence.

### Step 5 — Compatibility requirements (pack-compatibility-pack)

Every versioned component the pack depends on gets a real
`pc:CompatibilityRequirement` individual with all three fields non-empty
(`pc:componentName`, `pc:versionRequirement` in PEP 440 specifier syntax,
`pc:boundByPack`) — gate `gates/010_requirement_shape.rq` refuses the empty-field shape
at admission time; the runtime parse is real `packaging.specifiers.SpecifierSet`
machinery via `scripts/check_pack_compatibility.py` (a malformed specifier raises a
typed IncompatibleError naming the string), and `scripts/pack_dependency_order.py`
refuses a dependency cycle before one is silently produced. Law of the matrix
(pack.toml): "Extending the matrix to a new component means adding one real, checkable
`pc:CompatibilityRequirement` individual, verified the same way, not editing a
hand-maintained code table" — bounds are transcribed from actually-pinned versions,
never invented. For frozen-duckdb-pack: the ggen runtime bound (`pc:GgenRuntimeRequirement`
already binds `ggen_runtime >=26.8.0,<27.0.0` marketplace-wide) plus, if the pack
declares a DuckDB-era component, a bound verified against the real pin (e.g. the
duckdb-rs v1.10505.0 / DuckDB 1.5.5 era float this repo pins).

### Step 6 — Maturity ladder (pack-maturity-pack)

Admission floor and Level-5 promotion are different bars. Admission ends at QUALIFIED.
L5 promotion additionally requires, per pack-maturity-pack:

- mechanical evidence: `l5p:cap03` deterministic regeneration, `l5p:cap04` fixed-point
  convergence, `l5p:cap09` generated receipt verification;
- the Diátaxis contract — all four quadrants (tutorial / how-to / reference /
  explanation) with typed structural refusals `L5-DOC-001..010` (missing quadrant,
  missing semantic-authority declaration, missing executable path, undocumented
  refusal/falsifier surface, how-to without authority boundary, etc.);
- the standing rule: `L5DocALIVE = DiataxisClosure ∧ Correspondence ∧ Execution ∧
  Replay` — documentation alone never rounds up to standing; "if ontology, generated
  behavior, documentation, receipts, or replay diverge, promotion must fail closed."

The pack's description already commits it to this floor: it "supplies reusable
mechanical evidence for Level-5 promotion while refusing to invent domain semantics" —
the domain semantics (DuckDB build/link law) are exactly what C6 supplies.

### Step 7 — Consumer composition (this repo, after admission)

Once admitted and QUALIFIED, frozen-duckdb composes it per 階 REUSE: a
`[[ontology.pack]]` entry in `ggen.toml` (name/version must exist in
~/ggen-marketplace/packs/), then `ggen sync run` renders the consequences (gates,
templates) into this repo. Rendered files are consequences: edit the pack/ontology,
never the projection; hand-edits to frozen artifacts are detected by freeze_policy
(checksum-mismatch refusal, rmcp-pack pattern). Each capability that becomes
pack-expressible removes its row from `HANDWRITTEN.md` — the 帳 ledger shrinks
monotonically; growth requires a paydown plan in the same change.

### Who cuts (権)

Agents manufacture intents; authority is the operator's. Cut sequence:

1. C6 agent: author pack content on `pack/frozen-duckdb-capabilities` (worktree only).
2. Coordinator: serialized `--no-ff` merge into the marketplace canonical branch
   (gated on structure inspection + the compatibility lints), per RUNBOOK merge law.
3. Qualification court: marketplace CI/operator run of `qualify_packs.py` — ALIVE
   receipt grants QUALIFIED.
4. Operator: marketplace release cut (`[marketplace] version` bump → versioned GitHub
   Release; catalog is deterministic projection). Admission never grants DO authority
   (`mg:grantsDoAuthority false`); any consequential DO remains
   explicit-cut + operator-supplied authority + brokered receipt.

## 3. FAILED-EDGE LEDGER

Capabilities this repo needed that NO marketplace pack expresses. Each row records the
search that failed (prior-art closure), so the pack author owns exactly these residuals.
`failed(edge_i) ≠ failed(G)`: incidental near-hits are named per row; nothing pruned
silently. Search method, run 2026-09-21 against ~/ggen-marketplace (canonical tree,
packs count ≈ 250):

- Catalog search: `ggen pack search {duckdb,dylib,rpath,docs.rs,cmake,universal,macos,ffi}`
  → `"results": [], "total": 0` for every term (exit 0; JSON preserved in ticket History).
- Content search: `grep -rli` over `packs/` per term; every hit opened and read in
  context. All hits were incidental (enumerated per row).

| # | capability | packs searched | why no pack expresses it | intended owner (frozen-duckdb-pack fact?) | evidence |
|---|------------|----------------|--------------------------|-------------------------------------------|----------|
| 1 | macOS universal dylib staging: fat arm64+x86_64 dylib acquisition into `~/.frozen-duckdb/cache/v{VER}-{arch}/`, vendored 1.5.5 headers, dead-fallback removal | catalog: `universal`, `macos`, `dylib` → 0. content grep `universal.?(\|)binary\|dylib\|lipo\|-target arm64` → 0 files | No pack models binary-artifact staging at all; the only dylib strings in packs/ are tcps-release-pack's own artifact globs (`libtcps_ffi*.dylib`, ontology.ttl:5816/5825) — a release-manifest enumeration for a different product, not a reusable staging capability | YES — acquisition/staging family | MILESTONE.md "Done" bullet 1; HANDWRITTEN.md wave-2 row; session greps 2026-09-21 |
| 2 | DOCS_RS no-link builds: under `DOCS_RS=1`, generate bindings from vendored headers and skip linking/rpath so docs.rs typechecks without the dylib | catalog: `docs.rs` → 0. content grep `docs\.rs\|docsrs\|DOCS_RS` → 3 files, all prose | rmcp-pack mentions docs.rs only as a provenance negation ("not docs.rs prose", pack.toml/ontology.ttl) — no pack states a docs.rs cross-compile build-policy fact anywhere | YES — build-policy family | HANDWRITTEN.md TR4 row; crates/frozen-duckdb-sys/build.rs; session grep 2026-09-21 |
| 3 | DuckDB release-asset-name law: real asset scheme `libduckdb_{arch}.dylib` from seanchatmangpt/frozen-duckdb releases; fictional upstream schemes refused | catalog: `duckdb` → 0. content grep `libduckdb` → 0 files; `duckdb` → 1 file | ggen-legacy-ingestion-pack/registry-a.tsv:225 is a legacy registry row (`fmt  duckdb  database-schema`) — ingestion data about another tool, not a capability | YES — fetch-scheme/asset-name family | HANDWRITTEN.md T6 row (scripts/create_frozen_setup.sh); MILESTONE.md "CI release-asset naming fixed"; session greps 2026-09-21 |
| 4 | rpath propagation: builder exports `DEP_DUCKDB` build-script metadata; consumers get LC_RPATH to the cache dir; verified via `otool -l \| grep -A2 LC_RPATH` | catalog: `rpath` → 0. content grep `rpath\|install.name\|otool` → 11 files, all `grammarPath`/`detectorPath`/`rendererPath` false positives | Zero real occurrences; no pack models dynamic-linker path propagation as a fact or gate | YES — link-policy family | HANDWRITTEN.md wave-2 + T7 context; MILESTONE.md "Runtime @rpath propagation"; session grep 2026-09-21 |
| 5 | cmake extension-list recipe for DuckDB 1.5.x source builds (extension list pinned per era) | catalog: `cmake` → 0. content grep `cmake` → 4 files | tcps-release-pack/ontology.ttl:5342 and tcps lifecycle.py reference cmake only as a toolchain version probe (`cmake --version`) in release environment reports — no build recipe; ggen-legacy-ingestion hits are registry data | YES — source-build family | T6 context (build_frozen_duckdb.sh / build_static_duckdb.sh pins); session grep 2026-09-21 |
| 6 | dylib install-name/link-name compatibility law: DuckDB ≥ 1.5 dylibs carry neutral `@rpath/libduckdb.dylib`; setup creates `libduckdb.dylib/.1/.1.4` compat symlinks onto the arch dylib | same search as row 4 (install.name/otool/dylib) → only tcps artifact globs | The tcps globs enumerate tcps's own outputs; nothing encodes install-name compatibility or symlink-family law for third-party dylibs | YES — link-name family (pairs with row 4) | HANDWRITTEN.md T6 row (prebuilt/setup_env.sh); MILESTONE.md "libduckdb.dylib link name" |
| 7 | source-build dependency pinning: unpinned duckdb-rs clones pinned to `--branch v1.10505.0` (the crate float encoding DuckDB 1.5.5) | covered by rows 3/5 searches; `ggen pack search duckdb` → 0 | No pack expresses era-matched source-build pinning; pack-compatibility-pack pins *component versions as requirements*, not *build-time clone pins* — adjacent family, different semantic (pc: checks installed versions; this law pins a fetch) | YES — source-build family (near-hit recorded: pc: is COMPOSE-adjacent, still failed(edge)) | HANDWRITTEN.md T6 row; MILESTONE.md T6 context |
| 8 | consumer-crate validation harness: standalone workspace isolation (`test-dependency/` own `[workspace]` table) + era-matched crates.io pin (`duckdb = "1.10505"`) | catalog: `ffi` → 0; no consumer-validation pack found in packs/ survey | tcps-ffi is a product crate, not a validation-harness capability; no pack generates an isolated downstream consumer workspace for drop-in-compat proof | YES — consumer-validation family | HANDWRITTEN.md T7 P5 row (commit 80f5902); MILESTONE.md T7 DoD (5) |
| 9 | error-taxonomy repair paired with regression tests: `column_names()` returns a typed error pre-execution instead of panicking on schema unwrap; paired pre/post-execution + empty-result regression tests | no pack found for error-taxonomy+regression pairing in packs/ survey (gates/ in law packs enforce ontology shapes, not runtime error taxonomies) | All existing gates are SPARQL graph-shape courts; none generates paired repair+regression-test consequences for runtime error behavior | YES — error-taxonomy + regression-gate family | HANDWRITTEN.md TR3 row; MILESTONE.md wave-2 T2A context |

Ledger rule: a row leaves this table (and its HANDWRITTEN.md twin) only when the
admitted, QUALIFIED pack expresses the semantic element and this repo renders it as a
consequence — proven by a `ggen sync run` receipt, never by assertion. C6 owns turning
rows 1–9 into ontology facts + gates inside the ONE pack (families inside a pack; no
pack-per-capability sprawl — 階 step 3/4 topology law).

## 4. What this ticket did NOT do (falsifiers declared, not hidden)

- Qualification court NOT executed for frozen-duckdb-pack: the pack does not exist yet;
  its standing is UNKNOWN until C6 lands and `qualify_packs.py` runs. §2 Step 4 cites
  the court's documented mechanics from its source, not an observed run.
- `ggen law validate` not run against the law packs: it validates a project's own
  `[law]`-configured graph, not an arbitrary pack directory; gate execution for
  admitted packs belongs to the qualification court.
- No [[ontology.pack]] composition added to this repo's ggen.toml: the pack is
  unadmitted; composing it now would break `ggen sync run`. Composition is a later
  wave's scope.

## 5. Session receipt (証 — observed execution, 2026-09-21)

| command | exit | note |
|---------|------|------|
| `ggen pack search {duckdb,dylib,rpath,docs.rs,cmake,universal,macos,ffi}` | 0 each | `"results": [], "total": 0` for all eight terms |
| `grep -rli -E 'rpath\|install.name\|otool\|dylib' packs/` (and per-term variants) | 0 | every hit read in context; incidental per rows 1–9 |
| `ls /Users/sac/ggen-marketplace-wt/c6/packs/ \| grep -i -E 'frozen\|duckdb'` | 1 (no match) | C6 pack not yet present at authoring time |
| `ggen --version` | 0 | ggen 26.8.18 |
| `ggen sync run` (this worktree) | 0 | ggen 26.8.18; schema/domain.ttl BLAKE3 df8037f0… |
| `cargo build --workspace` (this worktree) | 0 | Finished dev profile in 29.40s; prebuilt v1.5.5-arm64 dylib linked |
