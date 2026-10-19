# Prebuilt DuckDB Binaries

This directory supports the **manual prebuilt workflow** of the frozen-duckdb project: it carries the DuckDB 2.0.0-dev headers and the environment setup script. The dylibs themselves are **not committed to the repository** — they are published as GitHub Release assets (`libduckdb_{arch}.dylib`) and downloaded automatically by `frozen-duckdb-builder::ensure_binary()` on first build.

## Contents

Sizes below are **measured on this tree at DuckDB 2.0.0-dev** (2026-10-19),
not claimed from upstream.

| File | Size (measured) | Description |
|------|------|-------------|
| `duckdb.h` | 284KB (290,195 bytes) | C header file for DuckDB 2.0.0-dev API |
| `duckdb.hpp` | 431B | Thin umbrella C++ header (includes `duckdb/main/*.hpp`; the full upstream `duckdb.hpp` is ~2.0MB, but this tree ships the umbrella form) |
| `setup_env.sh` | 3.4KB | Environment setup script (legacy manual workflow) |

The same headers are vendored inside `crates/frozen-duckdb-builder/vendored-headers/`, so the normal `cargo build` path works offline even when a release download carries only the dylib.

## Release Assets (downloaded, not stored here)

Sizes measured on DuckDB 2.0.0-dev (2026-10-19). Per-arch thin dylibs; a
stripped arm64 dylib comes out at 52.4MB.

| Asset | Size (measured) | Contains |
|-------|-----------------|----------|
| `libduckdb_arm64.dylib` | ~66MB unstripped (52.4MB stripped) | arm64 dylib |
| `libduckdb_x86_64.dylib` | ~66MB unstripped | x86_64 dylib |

## Symlinks

`setup_env.sh` creates these symlinks when sourced:

| Symlink | Target | Purpose |
|---------|--------|---------|
| `libduckdb.dylib` | `libduckdb_{arch}.dylib` | **Primary link name** — DuckDB >= 1.5 dylibs carry the neutral install name `@rpath/libduckdb.dylib`, so this is the name `@rpath` and `-lduckdb` resolution use |
| `libduckdb.1.dylib` | `libduckdb_{arch}.dylib` | Legacy version compatibility (1.4-era consumers) |
| `libduckdb.1.4.dylib` | `libduckdb_{arch}.dylib` | Legacy version compatibility (1.4-era consumers) |

## Architecture Detection

The `setup_env.sh` script detects the system architecture (`ARCH` override or `uname -m`) and selects the matching binary. Assets are per-architecture: `libduckdb_arm64.dylib` for arm64 hosts, `libduckdb_x86_64.dylib` for x86_64 hosts.

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

**Last Updated:** 2026-10-19
