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

| gate | before | after |
|---|---|---|
| S_coverage (set) | 77/1245 = 0.0618 | 96/1245 = 0.0771 |

Remaining uncovered mass is concentrated in the generated bindgen bindings
(`crates/frozen-duckdb-sys/src/bindgen_bundled_version*.rs`: 891 items) —
machine-generated FFI bindings whose signatures are already fully tabulated
in `reference.md`; they register as uncovered because no doc prose names
them, not because they are absent from the reference tables. Excluding the
two bindgen modules, the scaffold covers 96/354 (0.2712) of the
hand-written surface.

## Spot verification (8/8 match)

`ensure_binary` (builder lib.rs:90), `write_to_out_dir` (sys build.rs:199),
`close_brace` (pragma.rs:118), `map` (logical_type.rs:168), `stream_step`
(statement.rs:400), `FlockManager` (flock_manager.rs:51),
`set_prepared_statement_cache_capacity` (cache.rs:49),
`arrow_ffi_to_query_params` (arrow.rs:1234) — rendered signature and kind
match `crates/` source for all 8.
