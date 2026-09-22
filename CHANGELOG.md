# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.5.5] - 2026-09-21


### Added

- Automatic download of prebuilt DuckDB libraries from GitHub Releases on first build (macOS: `libduckdb_arm64.dylib`, `libduckdb_x86_64.dylib`), cached under `~/.frozen-duckdb/cache/v1.5.5-{arch}/`.
- Local source-compile fallback, cloning upstream DuckDB at the pinned `v1.5.5` tag, when no release asset is available.
- Vendored DuckDB 1.5.5 headers in `frozen-duckdb-builder`, so builds work offline regardless of how the binary was obtained.
- Runtime `@rpath` entry emitted by the `frozen-duckdb` build script (consuming `DEP_DUCKDB_DUCKDB_LIB_DIR` metadata exposed by `frozen-duckdb-sys`), so executables and tests link and run without `DYLD_LIBRARY_PATH` setup.
- A manufactured verification chain: `make verify` runs the rendered gate battery (`scripts/verify-gates.sh`: fmt check, clippy with warnings denied, workspace build, workspace tests, and the `test-validation` binary gate), then `ggen sync run` (re-render + receipt) and `ggen receipt verify` (receipt-chain check); the Makefile verify targets, gate runner, dry-run-publish gates, and receipt-contract matrix are rendered consequences of marketplace pack facts declared in `ggen.toml`, not hand-maintained files.
- Reconciliation governance for generated artifacts: `docs/GENESIS.md` — a manifest mapping every generated/consequence file to its owning source — checked by `make genesis-check` (source existence, consequence existence, upstream pin match, and ledger-row cross-check), so upstream changes name exactly which projections must re-render and orphans surface as defects.


### Changed

- Bundled DuckDB upgraded from 1.4.0 to upstream v1.5.5 (released 2026-07-22); DuckDB pin bumped across the builder, test harnesses, and CI defaults; workspace version bumped to 1.5.5 for all three crates (`frozen-duckdb`, `frozen-duckdb-sys`, `frozen-duckdb-builder`); the crate version now tracks the DuckDB release it bundles.
- `test-dependency` now exercises the upstream `duckdb = "1.5"` line alongside the frozen binary.


### Fixed

- CI release assets are now published under the exact names `frozen-duckdb-builder::ensure_binary()` downloads (`libduckdb_{arch}.dylib`), replacing the mismatched legacy `libfrozen_mega_{arch}.dylib` names that forced every user onto the local-compile fallback.
- Bindgen no longer double-includes `duckdb.h` (redefinition errors), and the builder normalizes the cache layout (`duckdb/` headers, plain `libduckdb.dylib` link name) on every binary-acquisition path.
- **CI release-asset rehearsal hardening**: the `v1.5.5` release workflow is rehearsed before the operator cuts the tag — matrix builds set `CMAKE_OSX_ARCHITECTURES` per architecture (honored by the builder's local-compile fallback), and the `ensure_binary()` cache-miss path (release-download miss → pinned local compile) was exercised against the exact asset names and paths the workflow publishes.
- **`column_names()` panic fix**: calling `column_names()` before statement execution no longer panics on a schema unwrap — it returns a proper error. Regression tests cover pre-execution, post-execution, and empty-result calls.
- **docs.rs build pattern**: `frozen-duckdb-sys` detects the `DOCS_RS` environment variable and generates bindings from the vendored headers while skipping library linking, so documentation builds on docs.rs (Linux, no prebuilt `.so`) succeed without a 30+ minute compile.
- **Linux / multi-arch builder support**: platform-aware release-asset naming (`.dylib` on macOS, `.so` on Linux), `CMAKE_OSX_ARCHITECTURES` override honored in the local-compile fallback, and a real Linux branch in `setup_env.sh`. Prebuilt `.so` assets are planned for a later tag; the `v1.5.5` tag ships macOS-only release assets, one per architecture (`libduckdb_arm64.dylib`, `libduckdb_x86_64.dylib`); Linux and Windows builds use the pinned local-compile fallback.
- **test-validation is a real gate**: the `test-validation` harness now asserts — binary present at the builder's expected path, `duckdb_library_version()` matching the pinned version via FFI, vendored header layout intact — and exits nonzero on any failure, instead of printing decorative checkmarks.
- **Repo hygiene**: kcura-era debris removed (dead smoke/CI-gate scripts, stray root test files, empty dataset directories); `HANDWRITTEN.md` ledger seeded.

