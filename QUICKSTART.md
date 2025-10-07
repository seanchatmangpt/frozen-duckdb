# 🚀 Quick Start Guide

Get up and running with frozen DuckDB in under 5 minutes!

## 📋 Prerequisites

- Rust (stable or later)
- Git
- macOS (Linux/Windows support coming soon)

## ⚡ Quick Setup

### 1. Clone the Repository

```bash
git clone https://github.com/yourusername/frozen-duckdb.git
cd frozen-duckdb
```

### 2. Set Up Environment

```bash
source prebuilt/setup_env.sh
```

### 3. Test the Installation

```bash
# Run the basic example
cargo run --example basic_usage

# Run performance comparison
cargo run --example performance_comparison
```

### 4. Use in Your Project

Add to your `Cargo.toml`:

```toml
[dependencies]
duckdb = { version = "1.4.0", default-features = false }
```

Add `build.rs` to your project root:

```rust
use std::env;
use std::path::Path;

fn main() {
    if let Ok(lib_dir) = env::var("DUCKDB_LIB_DIR") {
        let lib_dir = Path::new(&lib_dir);
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=dylib=duckdb");
        println!("cargo:rerun-if-env-changed=DUCKDB_LIB_DIR");
    }
}
```

Set environment before building:

```bash
source /path/to/frozen-duckdb/prebuilt/setup_env.sh
cargo build
```

## 🎯 Expected Results

- **Build time**: ~7-10 seconds (vs 1-2 minutes with bundled DuckDB)
- **Incremental builds**: ~0.11 seconds (vs 30 seconds)
- **Performance**: Same as bundled DuckDB, but much faster builds

## 🔧 Troubleshooting

### Library Not Found

```bash
# Check environment
echo $DUCKDB_LIB_DIR
echo $DUCKDB_INCLUDE_DIR

# Re-source environment
source prebuilt/setup_env.sh
```

### Build Errors

```bash
# Clean and rebuild
cargo clean
source prebuilt/setup_env.sh
cargo build
```

## 📚 Next Steps

- Read the full [README.md](README.md) for detailed documentation
- Check out the [examples/](examples/) directory for more usage patterns
- Visit the [GitHub repository](https://github.com/yourusername/frozen-duckdb) for updates

## 🤝 Need Help?

- Open an issue on GitHub
- Check the troubleshooting section in README.md
- Review the examples for common patterns

---

**Happy coding with frozen DuckDB! 🦆⚡**
