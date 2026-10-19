# Quickstart: frozen-duckdb native (Rust, zero C toolchain)

Learn frozen-duckdb by shipping a working query in five lines of Rust. You
need only Rust; no C compiler, no DuckDB install, no `DYLD_*` setup beyond
one documented caveat below.

Prerelease note: `2.0.0-dev` is a **prerelease version**. Cargo semver
matches a prerelease requirement only when the requirement itself names a
prerelease, so pin it exactly:

```bash
cargo new fd-quickstart && cd fd-quickstart
cargo add frozen-duckdb@2.0.0-dev
```

Honest availability note (as of 2026-10-19): `cargo search frozen-duckdb`
reports only `0.1.0` on crates.io; `2.0.0-dev` is **not published yet**.
Until the first `v2.0.0-dev` tag and crates.io release land, consume the
prerelease from a checkout:

```toml
[dependencies]
frozen-duckdb = { path = "/path/to/frozen-duckdb/crates/frozen-duckdb" }
# or: frozen-duckdb = { git = "https://github.com/seanchatmangpt/frozen-duckdb" }
```

## The program

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

Run it:

```bash
cargo run
# User: Alice
```

## Why the first build is fast: the builder freeze

`frozen-duckdb` never compiles DuckDB from source when a cached engine
exists. `frozen-duckdb-builder::ensure_binary()` acquires the engine once,
then the cache is frozen and every later build (this project, the next
project, any project on this machine) reuses it. On this machine the warm
cache is:

```text
$ ls -lh ~/.frozen-duckdb/cache/v2.0.0-dev-arm64/
total 128720
drwxr-xr-x@ 4 sac  staff   128B Oct 19 13:22 duckdb
-rwxr-xr-x@ 1 sac  staff    63M Oct 19 14:05 libduckdb_arm64.dylib
lrwxr-xr-x@ 1 sac  staff    70B Oct 19 13:40 libduckdb.dylib ->
  /Users/sac/.frozen-duckdb/cache/v2.0.0-dev-arm64/libduckdb_arm64.dylib
  (symlink target shown wrapped)
```

That `libduckdb.dylib` symlink plus vendored headers under `duckdb/` are the
builder's self-healing cache layout
(`crates/frozen-duckdb-sys/build.rs`, `crates/frozen-duckdb/build.rs`).
Crate-level benchmark table (`crates/frozen-duckdb/src/lib.rs`): first build
7-10s with a warm cache vs 1-2 minutes from source; incremental 0.11s.

Measured receipt for this tutorial (2026-10-19, scratch crate at
`/tmp/fd-qs-native` with a path dependency; program source identical to the
listing above):

```text
$ cargo run -q            # compiled the dep tree; first exec hit the rpath
                          # caveat below, then fixed and re-run:
$ install_name_tool -add_rpath ~/.frozen-duckdb/cache/v2.0.0-dev-arm64 \
    target/debug/fd-qs-native
$ ./target/debug/fd-qs-native; echo "exit=$?"
User: Alice
exit=0
```

Honest timing note: the very first scratch build took ~22 minutes wall on a
machine running several concurrent workspace builds (the C dependency tree —
`zstd-sys` and friends — compiled from scratch in the scratch target dir).
The engine dylib itself came from the frozen cache; the wait was the Rust/C
dependency tree, not DuckDB. With an unloaded machine and a warm cargo
cache this is materially faster; the "7-10s first build" figure refers to
the DuckDB acquisition step specifically.

## Caveat: first-run dyld error for external consumers (known-open)

If your binary aborts with `Library not loaded: @rpath/libduckdb.dylib`,
the engine dylib resolved at link time but no rpath reached your binary:
`cargo:rustc-link-arg` emitted by a *library* build script
(`crates/frozen-duckdb/build.rs`) does not propagate to a downstream
crate's binary on stable cargo. This affected this tutorial's scratch crate
and was fixed in place, verified:

```text
$ install_name_tool -add_rpath ~/.frozen-duckdb/cache/v2.0.0-dev-arm64 \
    target/debug/fd-qs-native
$ ./target/debug/fd-qs-native; echo "exit=$?"
User: Alice
exit=0
```

Wait for it: this is a consumer-linkage gap, known-open upstream of the
tutorials. A cleaner fix (consumer build script or runtime lookup in the
builder) belongs in the builder, not your app.

## Where to next

- Full feature surface: `docs/reference/` and `docs/how-to/`
- Builder/cache internals: `docs/architecture/binary-management.md`
- Browser target (Web Worker + OPFS): it is BLOCKED (KD-05) as of
  2026-10-19 — see
  `docs/tutorials/quickstart-browser-opfs.md` for the contract, not a
  runnable path.
