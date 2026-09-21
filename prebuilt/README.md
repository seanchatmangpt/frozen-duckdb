# Prebuilt DuckDB Binaries

This directory supports the **manual prebuilt workflow** of the frozen-duckdb project: it carries the DuckDB 1.5.5 headers and the environment setup script. The dylibs themselves are **not committed to the repository** — they are published as GitHub Release assets (`libduckdb_{arch}.dylib`) and downloaded automatically by `frozen-duckdb-builder::ensure_binary()` on first build.

## Contents

| File | Size | Description |
|------|------|-------------|
| `duckdb.h` | 244KB | C header file for DuckDB 1.5.5 API |
| `duckdb.hpp` | 2.0MB | C++ header file for DuckDB 1.5.5 API |
| `setup_env.sh` | 1.7KB | Environment setup script (legacy manual workflow) |

The same headers are vendored inside `crates/frozen-duckdb-builder/vendored-headers/`, so the normal `cargo build` path works offline even when a release download carries only the dylib.

## Release Assets (downloaded, not stored here)

| Asset | Size | Contains |
|-------|------|----------|
| `libduckdb_arm64.dylib` | ~117MB | universal binary (arm64 + x86_64) |
| `libduckdb_x86_64.dylib` | ~117MB | universal binary (arm64 + x86_64) |

## Symlinks

`setup_env.sh` creates these symlinks when sourced:

| Symlink | Target | Purpose |
|---------|--------|---------|
| `libduckdb.dylib` | `libduckdb_{arch}.dylib` | **Primary link name** — DuckDB >= 1.5 dylibs carry the neutral install name `@rpath/libduckdb.dylib`, so this is the name `@rpath` and `-lduckdb` resolution use |
| `libduckdb.1.dylib` | `libduckdb_{arch}.dylib` | Legacy version compatibility (1.4-era consumers) |
| `libduckdb.1.4.dylib` | `libduckdb_{arch}.dylib` | Legacy version compatibility (1.4-era consumers) |

## Architecture Detection

The `setup_env.sh` script detects the system architecture (`ARCH` override or `uname -m`) and selects the matching binary. As of DuckDB 1.5.5 each asset is a universal binary (arm64 + x86_64), so either asset runs on any supported Mac.

## Performance Benefits

| Build Type | Before | After | Improvement |
|------------|--------|-------|-------------|
| First Build | 1-2 minutes | 7-10 seconds | 85% faster |
| Incremental | 30 seconds | 0.11 seconds | 99% faster |
| Release | 1-2 minutes | 0.11 seconds | 99% faster |

## Usage

```bash
# Optional legacy workflow (auto-detects architecture, creates symlinks)
source prebuilt/setup_env.sh
cargo build

# Normal workflow needs none of this:
cargo build   # builder downloads libduckdb_{arch}.dylib automatically
```

## Maintenance

These headers should be updated when:
- DuckDB releases new versions with important features/bug fixes (keep in sync with `crates/frozen-duckdb-builder/vendored-headers/`)
- The release assets are rebuilt by `build-binaries.yml` on a new tag

**Last Updated:** 2026-09-21
