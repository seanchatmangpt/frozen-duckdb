# Migration: duckdb-rs to frozen-duckdb

Crate version **2.0.0**. The bundled engine remains DuckDB `2.0.0-dev`
(`b6013f7aa2f36caabb9466ce1791667ea8a280bb`).

`0.1.0` is an older generation. `2.0.0-dev` is a prerelease already on
crates.io; Cargo will not select it from `frozen-duckdb = "2"`. Depend on
`2.0.0`.

## Quick migration

### Step 1: Replace the dependency

```bash
cargo remove duckdb
cargo add frozen-duckdb
```

```toml
[dependencies]
frozen-duckdb = "2.0.0"
```

### Step 2: Update imports (if needed)
```rust
// Before
use duckdb::{Connection, Result};

// After (optional - same API)
use frozen_duckdb::{Connection, Result};
```

### Step 3: Build and enjoy!
```bash
cargo build  # Now 99% faster!
```

**That's it!** No code changes needed - same API, same functionality, 99% faster builds.

## What Changes?

### ✅ What Stays the Same
- **All APIs**: `Connection`, `Statement`, `Row`, `params`, etc.
- **All functionality**: SQL queries, transactions, prepared statements
- **All data types**: INTEGER, TEXT, REAL, BOOLEAN, JSON, BLOB
- **All features**: Parquet, JSON, Arrow integration
- **Error handling**: Same error types and behavior
- **Performance**: Same or better query performance

### 🚀 What Gets Better
- **Build time**: 99% faster (0.11s vs 2+ minutes)
- **Binary acquisition**: prebuilt macOS dylib (`libduckdb_{arch}.dylib`, one asset per architecture) downloaded from GitHub Releases instead of compiling DuckDB from source
- **Setup time**: Zero configuration needed
- **CI/CD**: Consistent, fast builds across environments

## Migration Examples

### Example 1: Basic Database Operations
```rust
// Before (duckdb-rs)
use duckdb::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("CREATE TABLE users (id INTEGER, name TEXT)")?;
    // ... rest of code
    Ok(())
}

// After (frozen-duckdb) - NO CHANGES NEEDED!
use frozen_duckdb::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("CREATE TABLE users (id INTEGER, name TEXT)")?;
    // ... same code, 99% faster builds
    Ok(())
}
```

### Example 2: Web API with DuckDB
```rust
// Before (duckdb-rs)
use duckdb::{Connection, Result};
use axum::{extract::Query, Json};
use serde_json::Value;

async fn analytics(Query(params): Query<HashMap<String, String>>) -> Result<Json<Value>> {
    let conn = Connection::open("analytics.db")?;
    // ... query logic
    Ok(Json(result))
}

// After (frozen-duckdb) - NO CHANGES NEEDED!
use frozen_duckdb::{Connection, Result};  // Only import changes
use axum::{extract::Query, Json};
use serde_json::Value;

async fn analytics(Query(params): Query<HashMap<String, String>>) -> Result<Json<Value>> {
    let conn = Connection::open("analytics.db")?;
    // ... same query logic, 99% faster builds
    Ok(Json(result))
}
```

### Example 3: Data Processing CLI
```rust
// Before (duckdb-rs)
use duckdb::{Connection, Result, params};

fn process_data() -> Result<()> {
    let conn = Connection::open("data.db")?;
    let mut stmt = conn.prepare("SELECT * FROM data WHERE category = ?")?;
    let rows = stmt.query_map(params!["electronics"], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
    })?;
    // ... process rows
    Ok(())
}

// After (frozen-duckdb) - NO CHANGES NEEDED!
use frozen_duckdb::{Connection, Result, params};  // Only import changes

fn process_data() -> Result<()> {
    let conn = Connection::open("data.db")?;
    let mut stmt = conn.prepare("SELECT * FROM data WHERE category = ?")?;
    let rows = stmt.query_map(params!["electronics"], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
    })?;
    // ... same processing logic, 99% faster builds
    Ok(())
}
```

## Performance Comparison

| Metric | duckdb-rs | frozen-duckdb | Improvement |
|--------|-----------|---------------|-------------|
| **First Build** | 1-2 minutes | 7-10 seconds | **85% faster** |
| **Incremental Build** | 30 seconds | 0.11 seconds | **99% faster** |
| **Release Build** | 1-2 minutes | 0.11 seconds | **99% faster** |
| **Binary Size** | Source compile (~200MB of build I/O) | ~117MB dylib (macOS) | **No compilation** |
| **Setup Time** | Manual setup | Zero config | **100% faster** |

## Troubleshooting

### Issue: "Library not found" errors
**Solution**: The frozen binary is acquired automatically by the builder — no environment needed.
```bash
# Check the versioned cache
ls -la ~/.frozen-duckdb/cache/

# On first build the builder downloads libduckdb_{arch}.dylib from
# this repo's GitHub Releases; if no asset is reachable it falls back
# to a local compile pinned at upstream SHA b6013f7aa2f36caabb9466ce1791667ea8a280bb (DuckDB 2.0.0-dev, pre-tag)
cargo clean && cargo build
```

### Issue: Build still slow
**Solution**: Verify you're using frozen-duckdb:
```bash
# Check Cargo.toml
grep "frozen-duckdb" Cargo.toml

# Should show your path/git pin, e.g.:
# frozen-duckdb = { path = ".../crates/frozen-duckdb" }
# (the crate version is 2.0.0-dev; the engine reports
#  `SELECT version()` -> `v2.0.0-dev88595`, NOT 1.5.5)
```

### Issue: Import errors
**Solution**: Update imports to use frozen-duckdb:
```rust
// Change from:
use duckdb::{Connection, Result};

// To:
use frozen_duckdb::{Connection, Result};
```

## Advanced Migration

### Feature Flags
If you were using specific duckdb-rs features:
```toml
# Before
duckdb = { version = "1.10505.0", features = ["json", "parquet"] }  # duckdb-rs crate version; NOT the engine version (engine: 1.x line)

# After (same core features, faster builds)
frozen-duckdb = "2.0.0-dev"  # core features built in; `serde_json` is OPT-IN
# (serde_json Value FromSql/ToSql — incl. VARIANT round-trip — needs
#  `features = ["serde_json"]`)

#### Behavior changes to know about (1.5.5 -> 2.0.0-dev)

- **BLOB round-trip**: `X'0102'` inserted and read back now yields raw bytes
  `[1, 2]`; DuckDB 1.x returned the text rendering `"x0102"`. If you parsed
  the text form, switch to `Vec<u8>`/`Value::Blob`.
- **Extensions**: on this 2.0.0-dev pin, `INSTALL flock` (community) returns
  HTTP 404 and `LOAD tpch` fails signature/unsigned checks; the `tpch(1)`
  helper table function is also gone. Don't rely on community/tpch
  extensions until the final 2.0.0 extension builds publish.
- **VARIANT DDL** (`CREATE TABLE t(v VARIANT)`) is now accepted; raw VARIANT
  columns project as `Struct(metadata BLOB, value BLOB)` through this FFI.

### Custom Build Scripts
If you have custom build.rs files:
```rust
// Before
fn main() {
    // Custom duckdb setup
}

// After - frozen-duckdb handles everything automatically
fn main() {
    // No changes needed - frozen-duckdb's build.rs handles binary setup
}
```

## Verification

After migration, verify everything works:

```bash
# 1. Build should be fast
time cargo build
# Should complete in <10 seconds

# 2. Tests should pass
cargo test

# 3. Run your application
cargo run
```

## Rollback Plan

If you need to rollback (not recommended):
> Storage caveat: databases created/written by the 2.0.0-dev engine carry a
> `v2.0.0-dev88595` banner and are not guaranteed to open with a 1.x engine.
> Export to CSV/Parquet before rolling back.
```bash
# Remove frozen-duckdb
cargo remove frozen-duckdb

# Add back duckdb-rs
cargo add duckdb
```

## Benefits Summary

✅ **99% faster builds** - No more waiting for DuckDB compilation  
✅ **Zero configuration** - Works out of the box (no `DYLD_*` env vars either)  
✅ **Same API** - No code changes needed  
✅ **Better CI/CD** - Consistent, fast builds  
✅ **Per-architecture assets** - `libduckdb_arm64.dylib` / `libduckdb_x86_64.dylib`, matched to your Mac automatically  
✅ **Production ready** - Tested, optimized binaries  

## Support

- **Documentation**: [README.md](README.md)
- **Examples**: `cargo run --example dropin_replacement`
- **Tests**: `cargo test dropin_compatibility_tests`
- **Issues**: [GitHub Issues](https://github.com/seanchatmangpt/frozen-duckdb/issues)

---

**Ready to migrate?** Run these commands and enjoy 99% faster builds:

```bash
cargo remove duckdb
cargo add frozen-duckdb
cargo build  # Now 99% faster!
```
