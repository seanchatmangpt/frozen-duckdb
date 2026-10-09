# GENERATED — doc-hdit reference skeletons

<!-- GENERATED-BANNER: every file in this directory is rendered by the        -->
<!-- doc-hdit scaffold from a deterministic code-surface extraction. Do NOT   -->
<!-- hand-edit. Regenerate with:                                              -->
<!--                                                                          -->
<!--   python3 /Users/sac/ggen-marketplace/scripts/gen_doc_surface.py code . \ -->
<!--     > /tmp/hdit/frozen-duckdb.code.v3.json                               -->
<!--   /Users/sac/ggen-marketplace/packs/rust-doc-hdit-pack/target/release/doc-hdit scaffold \ -->
<!--       --code /tmp/hdit/frozen-duckdb.code.v3.json                        -->
<!--       --templates /Users/sac/ggen-marketplace/packs/rust-doc-hdit-pack/templates \ -->
<!--       --out docs/reference/generated                                     -->
<!--                                                                          -->
<!-- Court: /Users/sac/ggen-marketplace/packs/rust-doc-hdit-pack/courts/doc_quality.court -->
<!--       (S_coverage >= 0.90, Phi_halluc <= 0.001, Q_density >= 0.65)       -->

## What this directory is

Deterministic reference skeletons for the frozen-duckdb code surface
(46 modules / 1245 public items: 1156 functions, 55 structs, 19 enums,
15 traits across `crates/frozen-duckdb`, `crates/frozen-duckdb-sys`,
`crates/frozen-duckdb-builder`), rendered by the rust-doc-hdit-pack's
`doc-hdit scaffold` verb from `gen_doc_surface.py code` output. The only
hand-writable region anywhere in the surface is the bounded
`AGENT-COMMENTARY` slot in `explanation.md`; everything else is
AGENT-FORBIDDEN.

## Files

| file | content |
|---|---|
| reference.md | AGENT-FORBIDDEN reference tables (module/symbol rows rendered from the code surface) |
| how_to.md | prerequisite verbs list (build surface) |
| explanation.md | summary + bounded AGENT-COMMENTARY slot |
| extractor-generated-surface-config-proposal.md | PROPOSED `.doc-surface.toml` convention for excluding the bindgen sys pair from the coverage denominator (hand-written proposal, not generated; see backlog [38]) |
| README.md | this provenance banner (hand-maintained, this file only) |

## Regeneration + gates

```sh
python3 /Users/sac/ggen-marketplace/scripts/gen_doc_surface.py code /Users/sac/frozen-duckdb \
  > /tmp/hdit/frozen-duckdb.code.v3.json
/Users/sac/ggen-marketplace/packs/rust-doc-hdit-pack/target/release/doc-hdit scaffold \
  --code /tmp/hdit/frozen-duckdb.code.v3.json \
  --templates /Users/sac/ggen-marketplace/packs/rust-doc-hdit-pack/templates \
  --out docs/reference/generated
```

## Coverage audit (2026-10-08, lane scaffold-duck)

Audit inputs: doc-surface claims (before: 100; after: 112) against the same
code surface (1245 public items), via `doc-hdit vectorize` +
`s_coverage_report`.

| gate | before (landed skeleton) | after (backticked regen) |
|---|---|---|
| S_coverage (`doc-hdit audit`) | 0.0811 | 0.9984 |

Remaining uncovered mass was concentrated in the generated bindgen bindings
(`crates/frozen-duckdb-sys/src/bindgen_bundled_version*.rs`: 923 of 1245
public items as of the 2026-10-08 backticked regen) — machine-generated FFI
bindings whose signatures are already fully tabulated in `reference.md`.
With the backticked tables registering as claims, the whole-surface audit
passes at 0.9984; against the bindgen-excluded hand-written denominator
(1245 - 923 = 322 items) coverage is not the binding constraint.

**Generated-surface policy (backlog [38], lane fdb-bindgen 2026-10-08):**
the bindgen sys pair is machine-generated output, declared out of scope for
coverage denominators. The extractor has no per-file config today; the
demonstrated exclusion path is its directory-name vendor policy (scratch
re-extract with the pair under a `vendor/`-named dir: 1706 -> 404 items,
0 bindgen modules). A repo-level `.doc-surface.toml` convention is proposed
in `extractor-generated-surface-config-proposal.md` in this directory,
awaiting extractor-side admission.

## Spot verification (8/8 match)

`ensure_binary` (builder lib.rs:90), `write_to_out_dir` (sys build.rs:199),
`close_brace` (pragma.rs:118), `map` (logical_type.rs:168), `stream_step`
(statement.rs:400), `FlockManager` (flock_manager.rs:51),
`set_prepared_statement_cache_capacity` (cache.rs:49),
`arrow_ffi_to_query_params` (arrow.rs:1234) — rendered signature and kind
match `crates/` source for all 8.
