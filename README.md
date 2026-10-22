# Frozen DuckDB

Drop-in replacement for the [`duckdb`](https://crates.io/crates/duckdb) crate.
Same Rust API. The DuckDB engine is a prebuilt library, cached after the
first build, so later builds link instead of compiling C++.

Crate version **2.0.0**. The bundled engine is DuckDB `v2.0.0-dev88595`,
upstream commit `b6013f7aa2f36caabb9466ce1791667ea8a280bb`.

`frozen-duckdb` **0.1.0** on crates.io is an older generation. Depend on
`2.0.0`.

## Install

```bash
cargo add frozen-duckdb
```

```toml
[dependencies]
frozen-duckdb = "2.0.0"
```

Rust 1.88 or newer.

```rust
use frozen_duckdb::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch(
        "CREATE TABLE users (id INTEGER, name TEXT);
         INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob');",
    )?;
    let mut stmt = conn.prepare("SELECT name FROM users WHERE id = ?")?;
    let name: String = stmt.query_row([1], |row| row.get(0))?;
    println!("User: {name}");
    Ok(())
}
```

A git dependency does not work. The tree has a `vendors/duckdb-rs` gitlink
and no `.gitmodules`, so Cargo stops with `no URL configured for submodule`.

## What you get

Three crates, published in this order:

| Crate | Role |
|---|---|
| `frozen-duckdb` | `duckdb-rs`-compatible API |
| `frozen-duckdb-sys` | FFI and linking (`links = "duckdb"`) |
| `frozen-duckdb-builder` | Download, local compile, and cache |

`frozen-duckdb-sys` is the only `links = "duckdb"` crate in this family.
It cannot be linked into the same binary as `libduckdb-sys` / `duckdb`.

First build: the builder looks for a cached library under
`~/.frozen-duckdb/cache/`, then a GitHub Release asset for tag
`v2.0.0-dev.3` (`libduckdb_{arm64,x86_64}.dylib` and
`libduckdb_{arm64,x86_64}.so`), then compiles the pinned upstream source.
Later builds reuse the cache. macOS binaries use `@rpath` and do not need
`DYLD_LIBRARY_PATH`.

Optional features (off by default): `serde_json`, `chrono`, `vtab`,
`vscalar`, `vtab-arrow`. The `frozen-duckdb-cli` binary is installed with
`cargo install frozen-duckdb --features serde_json`.

## Platforms

- **macOS** arm64 and x86_64: release dylibs.
- **Linux** arm64 and x86_64: release `.so` files. Consumers need glibc
  2.38 or newer (Ubuntu 24.04+, Debian trixie). The cached `.so` must be
  on the loader path at runtime.
- **Windows**: local compile from the pinned source. No release asset.

## Limits

- WebAssembly and browser OPFS are not part of this release.
- Excel and VSS are not in the static extension set. The static set is
  parquet, json, icu, httpfs, tpch, tpcds, fts, inet, and sqlsmith.
- Registered differences from upstream `duckdb-rs` are listed in
  [KNOWN_DIVERGENCES.md](KNOWN_DIVERGENCES.md).

## Migrating from `duckdb`

```bash
cargo remove duckdb
cargo add frozen-duckdb
```

```rust
use frozen_duckdb::{Connection, Result};
```

Details: [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md).

## Development

```bash
git clone https://github.com/seanchatmangpt/frozen-duckdb.git
cd frozen-duckdb
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

## License

MIT. See [LICENSE](LICENSE). DuckDB itself is MIT; the bundled notice is
`LICENSE-DUCKDB.md` inside each published crate.
