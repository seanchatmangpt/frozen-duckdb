# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/) — with the
convention that the crate version mirrors the bundled upstream DuckDB version.

## [1.5.5] - 2026-09-21

Bundled DuckDB upgraded to upstream v1.5.5 (released 2026-07-22). The crate
version now tracks the DuckDB release it bundles: frozen-duckdb 1.5.5 contains
DuckDB 1.5.5.

### Added
- Automatic download of prebuilt DuckDB libraries from GitHub Releases on first
  build (macOS: `libduckdb_arm64.dylib`, `libduckdb_x86_64.dylib`), cached under
  `~/.frozen-duckdb/cache/v1.5.5-{arch}/`.
- Local source-compile fallback, cloning upstream DuckDB at the pinned `v1.5.5`
  tag, when no release asset is available.
- Vendored DuckDB 1.5.5 headers in `frozen-duckdb-builder`, so builds work
  offline regardless of how the binary was obtained.
- Runtime `@rpath` entry emitted by `frozen-duckdb-sys`, so executables and
  tests link and run without `DYLD_LIBRARY_PATH` setup.

### Changed
- DuckDB pin bumped from 1.4.0 to 1.5.5 across the builder, test harnesses, and
  CI defaults; workspace version bumped to 1.5.5 for all three crates
  (`frozen-duckdb`, `frozen-duckdb-sys`, `frozen-duckdb-builder`).
- `test-dependency` now exercises the upstream `duckdb = "1.5"` line alongside
  the frozen binary.

### Fixed
- CI release assets are now published under the exact names
  `frozen-duckdb-builder::ensure_binary()` downloads
  (`libduckdb_{arch}.dylib`), replacing the mismatched legacy
  `libfrozen_mega_{arch}.dylib` names that forced every user onto the
  local-compile fallback.
- Bindgen no longer double-includes `duckdb.h` (redefinition errors), and the
  builder normalizes the cache layout (`duckdb/` headers, plain
  `libduckdb.dylib` link name) on every binary-acquisition path.

### Known limitations
- Prebuilt release assets are macOS-only (arm64 + x86_64); Windows and Linux
  builds use the local-compile fallback. Prebuilt assets for those platforms
  are on the roadmap.
