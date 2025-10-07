# Prebuilt DuckDB Binaries

This directory contains pre-compiled DuckDB binaries and headers for the frozen-duckdb project, enabling fast builds without DuckDB compilation bottlenecks.

## Contents

| File | Size | Description |
|------|------|-------------|
| `duckdb.h` | 186KB | C header file for DuckDB API |
| `duckdb.hpp` | 1.8MB | C++ header file for DuckDB API |
| `libduckdb_arm64.dylib` | 50MB | ARM64 binary for Apple Silicon Macs |
| `libduckdb_x86_64.dylib` | 55MB | x86_64 binary for Intel Macs |
| `setup_env.sh` | 1.6KB | Environment setup script |

## Symlinks

| Symlink | Target | Purpose |
|---------|--------|---------|
| `libduckdb.1.4.dylib` | `libduckdb_arm64.dylib` | Version compatibility |
| `libduckdb.1.dylib` | `libduckdb_arm64.dylib` | Version compatibility |
| `libduckdb.dylib` | `libduckdb_arm64.dylib` | Default library link |

## Architecture Detection

The `setup_env.sh` script automatically detects the system architecture and sets up the appropriate binary:

- **Apple Silicon (arm64)**: Uses `libduckdb_arm64.dylib` (50MB)
- **Intel Macs (x86_64)**: Uses `libduckdb_x86_64.dylib` (55MB)

## Performance Benefits

| Build Type | Before | After | Improvement |
|------------|--------|-------|-------------|
| First Build | 1-2 minutes | 7-10 seconds | 85% faster |
| Incremental | 30 seconds | 0.11 seconds | 99% faster |
| Release | 1-2 minutes | 0.11 seconds | 99% faster |

## Usage

```bash
# Set up environment (auto-detects architecture)
source prebuilt/setup_env.sh

# Build your project (now fast!)
cargo build
```

## Maintenance

These binaries should be updated when:
- DuckDB releases new versions with important features/bug fixes
- Security vulnerabilities need patching
- Performance improvements become available

**Total Size:** ~107MB (essential for fast builds)
**Last Updated:** 2025-10-07
