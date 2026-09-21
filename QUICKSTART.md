# 🚀 Quick Start Guide

Get up and running with frozen DuckDB in under 5 minutes!

## 📋 Prerequisites

- Rust (stable or later)
- Git
- macOS (Linux/Windows support coming soon)

## ⚡ Quick Setup

### 1. Clone the Repository

```bash
git clone https://github.com/seanchatmangpt/frozen-duckdb.git
cd frozen-duckdb
```

### 2. Test the Installation

```bash
# Build the workspace (first build downloads the prebuilt DuckDB dylib automatically)
cargo build

# Run the basic example
cargo run --example basic_usage

# Run performance comparison
cargo run --example performance_comparison

# Test TPC-H data generation
cargo run -- download --dataset tpch --output-dir test_data
```

No environment setup is required: on first build, `frozen-duckdb-builder::ensure_binary()`
downloads `libduckdb_{arch}.dylib` from this repository's GitHub Releases into
`~/.frozen-duckdb/cache/v1.5.5-{arch}/`, vendored 1.5.5 headers make bindgen work
offline, and the emitted runtime `@rpath` means binaries and tests run without any
`DYLD_LIBRARY_PATH` setup. `prebuilt/setup_env.sh` remains available for the legacy
manual-prebuilt workflow only.

### 3. Use in Your Project

Add to your `Cargo.toml`:

```toml
[dependencies]
frozen-duckdb = "1.5.5"
```

That's it — no custom `build.rs`, no environment variables. The `frozen-duckdb-sys`
build script invokes `frozen-duckdb-builder::ensure_binary()`, which fetches the
prebuilt dylib (falling back to a local compile pinned at upstream tag `v1.5.5`
only if no release asset is reachable), and the emitted runtime `@rpath` lets your
binaries and tests run with no `DYLD_LIBRARY_PATH` configuration.

## 🎯 Expected Results

- **Build time**: ~7-10 seconds (vs 1-2 minutes with bundled DuckDB)
- **Incremental builds**: ~0.11 seconds (vs 30 seconds)
- **Performance**: Same as bundled DuckDB, but much faster builds

## 🔧 Troubleshooting

### Library Not Found

```bash
# Check the cache directory — the builder stores the dylib here
ls -la ~/.frozen-duckdb/cache/

# Force a fresh download attempt
cargo clean && cargo build
```

If no release asset is reachable, the builder falls back to compiling DuckDB
locally, pinned at upstream tag `v1.5.5`.

### Build Errors

```bash
# Clean and rebuild
cargo clean
cargo build
```

## 📚 Next Steps

- Read the full [README.md](README.md) for detailed documentation
- Check out the [examples/](examples/) directory for more usage patterns
- Visit the [GitHub repository](https://github.com/seanchatmangpt/frozen-duckdb) for updates

## 🤝 Need Help?

- Open an issue on GitHub
- Check the troubleshooting section in README.md
- Review the examples for common patterns

---

**Happy coding with frozen DuckDB! 🦆⚡**
