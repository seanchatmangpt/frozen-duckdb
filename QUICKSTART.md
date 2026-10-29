# Quick start

Rust 1.88 or newer.

```bash
cargo new fd-quickstart && cd fd-quickstart
cargo add frozen-duckdb
```

```toml
[dependencies]
frozen-duckdb = "2.0.1"
```

```rust
use frozen_duckdb::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    let version: String = conn.query_row("SELECT version()", [], |row| row.get(0))?;
    println!("{version}");
    Ok(())
}
```

The first build downloads a prebuilt DuckDB library for macOS or Linux
(glibc 2.38+) into `~/.frozen-duckdb/cache/`, or compiles the pinned
engine if no release asset matches the host. Later builds link the cache.

`cargo add frozen-duckdb` without a version constraint resolves the newest
stable release. That is `2.0.1`. The older stable releases are `2.0.0`
and `0.1.0`. The prerelease `2.0.0-dev` is on the index and is not
selected by a bare `cargo add`.

Working from a clone of this repository:

```bash
cargo build --workspace
cargo run --example dropin_replacement
```
