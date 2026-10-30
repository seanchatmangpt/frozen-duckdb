# PB9 — Docs-Surface Audit (per-crate)

Date: 2026-10-19 · Subject: master@adbd260 · Lane PB9 · lane target dir `target-pb9` deleted at end.

## Method

Real commands on the canonical checkout: `cargo package -l -p <crate>`
(shipped files), `CARGO_TARGET_DIR=target-pb9 cargo doc --no-deps -p <crate>`
(rendered landing page + warnings, counted from real rustdoc output).
`target-pb9` deleted after the run.

## frozen-duckdb

- **Shipped docs files** (`cargo package -l`): `README.md` (workspace-root
  README via `readme.workspace = true`), no CHANGELOG, no `docs/`.
  Everything else is code/tests/examples (4 examples ship — good).
- **Landing page** (`target-pb9/doc/frozen_duckdb/index.html`): title
  `frozen_duckdb - Rust`, h1 `Crate frozen_duckdb`, real crate-level doc from
  `src/lib.rs` (drop-in replacement, version-encoding note, quick start).
  Not a stub. Rendered body ~16.6 KB.
- **rustdoc warnings: 15** — all `unresolved link` (broken intra-doc links):
  `appender_params.rs` (7), `config.rs` (3: `AUTOMATIC`, `ASC`,
  `NULLS_FIRST`), `statement.rs:228` (`Statement::exists`), `params.rs:269`
  (`Params`), `error.rs:51` (`Error::ToSqlConversionFailure`),
  `types/to_sql.rs:74` (`ParamsFromIter` class). Note: `documentation`
  metadata is set to the GitHub URL, not docs.rs.
- **README rendering on docs.rs/crates.io**: 3 relative links that 404 in the
  packaged README (repo-root paths not in the package):
  `docs/contributing/coding-standards.md` (README.md:216), `LICENSE`
  (README.md:233), `LICENSE-ENTERPRISE-DRAFT.md` (README.md:235). No images
  (only the CI badge — absolute GitHub URL, safe). Root `LICENSE` file is NOT
  packaged (only `LICENSE-DUCKDB.md` inside the main crate).

## frozen-duckdb-builder

- **Shipped docs files**: `README.md` only. No CHANGELOG, no docs/.
- **Landing page**: title/h1 correct, real doc comment from `src/lib.rs`
  (builder description, `ensure_binary` visible). Small but not a stub.
- **rustdoc warnings: 0.** Clean.

## frozen-duckdb-sys

- **Shipped docs files**: `README.md` only. No CHANGELOG, no docs/.
- **Landing page**: correct title/h1, large rendered page (~271 KB — full FFI
  surface). Real, not a stub.
- **rustdoc warnings: 3** — all from generated `bindgen.rs` (doxygen comments
  from vendored `duckdb.h`): 2 unresolved links (`types`, `entry_idx`),
  1 `invalid_codeblock_attributes` (` ```c ` one-liner). Not hand-written
  source; fix belongs in `wrapper.h` allowlist/filtering or
  `#![allow(rustdoc::...)]` in the bindgen doc-passing config, not in
  generated code.

## Fixes-needed (ordered)

1. **`documentation` metadata** (workspace `Cargo.toml:33`) points to the
   GitHub repo URL — should be `https://docs.rs/frozen-duckdb` (crate-level;
   sys/builder inherit workspace value pointing at the repo).
2. **README 404 links** (README.md:216, 233, 235): relative links to
   `docs/contributing/coding-standards.md`, `LICENSE`,
   `LICENSE-ENTERPRISE-DRAFT.md` resolve against repo root; on docs.rs they
   404. Fix: absolute GitHub blob URLs, or include LICENSE in the package
   (`include`/license file packaging).
3. **No CHANGELOG shipped in any crate.** Root `CHANGELOG.md` exists but is
   not packaged. Add per-crate or workspace-root changelog to `include`.
4. **Main crate: 15 broken intra-doc links** — mostly `[Params]`,
   `[ParamsFromIter]`, `[params_from_iter]`, `Connection::query_one/limit`,
   config reference-style links. Fix targets: `appender_params.rs`,
   `config.rs:74/86/92`, `statement.rs:228`, `params.rs:269`,
   `error.rs:51`, `types/to_sql.rs:74`.
5. **sys: 3 warnings from bindgen-generated docs** — fix at the source
   (`wrapper.h` doxygen) or suppress via bindgen options; do not edit
   generated `bindgen.rs`.
6. **LICENSE not packaged** in any crate (license = "MIT" metadata only).
   docs.rs will render README without a license file in the package.

## Landing-page verdict

All three landing pages render real crate-level docs (no stubs). The gap is
warnings + packaging (no LICENSE/CHANGELOG, broken README links), not
landing-page quality.