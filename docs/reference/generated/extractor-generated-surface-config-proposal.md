# Proposal: repo-level generated-surface config for `gen_doc_surface.py`

Status: PROPOSED — awaiting extractor-side admission. The extractor
(`scripts/gen_doc_surface.py` in ggen-marketplace) is owned by the
vendor-policy lane; this lane made zero extractor code changes.

## Problem (backlog [38], audit @d45739d)

The doc-hdit coverage audit @d45739d reports S_coverage 0.8789 with a
179-item gap, of which 131 items are the bindgen sys pair:

- `crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs`
- `crates/frozen-duckdb-sys/src/bindgen_bundled_version_loadable.rs`

526 FFI declarations x2 — machine-generated bindgen output, not
hand-written surface. Their signatures are already fully tabulated in
`docs/reference/generated/reference.md`, so the "gap" is a denominator
artifact, not documentation debt.

## Demonstrated mechanism (2026-10-08, no repo or extractor changes)

The extractor today has **no repo-level config/ignore file**. Its only
exclusion mechanism is directory-name based (`VENDOR_DIRS = {"vendor",
".ggen-v2", "third_party"}`, applied inside `iter_files` via
`vendor_part`). No banner detection, no per-file config.

Scratch re-extract (`git archive HEAD` into `/tmp/hdit/fd-excl`, then the
sys pair relocated under a directory literally named `vendor/` in the
scratch copy only):

- baseline: 51 modules / 1706 items; bindgen pair = 650 + 652 = 1302 items
- scratch relocation: **404 items, 0 bindgen modules** — the pair is fully
  excluded by the existing vendor-directory mechanism alone.

Conclusion: the existing mechanism excludes the pair when its files live
under a `VENDOR_DIRS`-named directory; what is missing is a way to express
"these files are generated" without relocating source files.

## Proposal

A repo-level config file `.doc-surface.toml` at repo root:

```toml
# .doc-surface.toml — generated-surface policy for gen_doc_surface.py
[[generated]]
paths = [
  "crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs",
  "crates/frozen-duckdb-sys/src/bindgen_bundled_version_loadable.rs",
]
reason = "bindgen output; signatures already tabulated in docs/reference/generated/reference.md"
```

### Config semantics requested from the extractor lane

1. `[[generated]] paths` — exact repo-relative file paths to drop from the
   public code-surface denominator (modules, paths, directories) and audit
   denominators, exactly as `VENDOR_DIRS` entries are dropped today.
2. Symbols from generated files that docs legitimately reference classify
   `external_documented` (NOT phantom, NOT uncovered), mirroring the
   vendor policy's `known_external` merge.
3. `--include-generated` flag to opt back into the full surface for
   full-surface audits, mirroring `--include-vendor`.
4. Backward compatibility: absent `.doc-surface.toml`, behavior is exactly
   today's.

## Falsifier

The proposal is refuted if any of the following holds after extractor-side
implementation:

- re-extracting the canonical checkout with `.doc-surface.toml` present
  still emits any `bindgen_bundled_version*` module, or
- the emitted surface differs from the scratch 404-item surface (other
  than by intentional hand-written-surface changes), or
- the vendor policy regression suite fails on a repo with no
  `.doc-surface.toml`.

## Gate receipt (this lane, 2026-10-08)

Re-extract of the scratch copy with the pair under `vendor/` shows the sys
pair excluded-or-configured by an existing, unmodified extractor mechanism:
1706 -> 404 items, 0 bindgen modules. Until extractor-side config lands,
the pair's exclusion is documented policy, not implemented behavior.
