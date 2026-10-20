# REL2-sys: sys/links claims re-witness at 2.0.0-dev

Lane REL2. Subject: `/Users/sac/frozen-duckdb` @ master, workspace version
`2.0.0-dev` (`Cargo.toml` `[workspace.package] version = "2.0.0-dev"`). No git
state commands run; scratch probes live outside the tree under
`/tmp/rel2-sys/` with `CARGO_TARGET_DIR=/tmp/rel2-sys/target`.

Standing: **ALIVE** — all three claims witnessed by real execution, 2026-10-19.

## Claim 1 — exactly-one-linker law: only sys links DuckDB

Grep over every `build.rs` in `crates/` for `rustc-link-lib` emitters:

```
$ grep -rn "rustc-link-lib" /Users/sac/frozen-duckdb/crates --include='build.rs'
crates/frozen-duckdb-sys/build.rs:48:    println!("cargo:rustc-link-lib=dylib=duckdb");
crates/frozen-duckdb-sys/build.rs:127:  // Deliberately NO rustc-link-search / rustc-link-lib / rustc-link-arg:
```

Exactly one emitter (`frozen-duckdb-sys`, the crate holding
`links = "duckdb"`); the second hit is the docs.rs branch's negative comment.
The main crate's `build.rs` emits only `cargo:rustc-link-arg=-Wl,-rpath,...`
(`crates/frozen-duckdb/build.rs:17`) — no second `rustc-link-lib`.

Consumer linkage check: 2-crate consumer graph
`links-probe -> frozen-duckdb -> frozen-duckdb-sys` built and **run** from
`/tmp/rel2-sys/links-probe` (real query, not compile-only):

```
links-probe: linked and executed, n=42
$ otool -L /tmp/rel2-sys/target/debug/links-probe | grep libduckdb
	/Users/sac/.frozen-duckdb/cache/v2.0.0-dev-arm64/libduckdb_arm64.dylib (compatibility version 0.0.0, current version 0.0.0)
count=1
```

Single `libduckdb` Mach-O load command. ALIVE.

## Claim 2 — DEP_DUCKDB_DUCKDB_LIB_DIR propagation (real run)

Scratch consumer crate `/tmp/rel2-sys/env-probe` (path-dep on
`frozen-duckdb-sys`) with `build.rs` printing every `DEP_DUCKDB_*` var as
`cargo:warning=ENVPROBE ...`. Built 2026-10-19, actual output:

```
warning: env-probe@0.1.0: ENVPROBE DEP_DUCKDB_DUCKDB_INCLUDE_DIR=/Users/sac/.frozen-duckdb/cache/v2.0.0-dev-arm64
warning: env-probe@0.1.0: ENVPROBE DEP_DUCKDB_DUCKDB_LIB_DIR=/Users/sac/.frozen-duckdb/cache/v2.0.0-dev-arm64
```

`cargo:DUCKDB_LIB_DIR` emitted by
`crates/frozen-duckdb-sys/build.rs` (`println!("cargo:DUCKDB_LIB_DIR=...")`)
reaches dependents as `DEP_DUCKDB_DUCKDB_LIB_DIR` via `links = "duckdb"`
(`crates/frozen-duckdb-sys/Cargo.toml:16`). The repo's own consumer
(`crates/frozen-duckdb/build.rs:9-10`) reads the same var. ALIVE.

## Claim 3 — bindgen single-include

Binding artifact from the real links-probe build:
`/tmp/rel2-sys/target/debug/build/frozen-duckdb-sys-efecf83130e89ea7/out/bindgen.rs`
(365,917 bytes), included exactly once via
`crates/frozen-duckdb-sys/src/lib.rs:9`
(`include!(concat!(env!("OUT_DIR"), "/bindgen.rs"))`).

Exact-text counts at 2.0.0-dev headers:

```
grep -c '^pub struct _duckdb_database {'  -> 1
grep -c '^pub type duckdb_database ='     -> 1
grep -c 'pub struct duckdb_database'      -> 0
```

Note: the probe's literal spec `pub struct duckdb_database` counts 0 because
bindgen renders the opaque handle as `pub struct _duckdb_database` +
`pub type duckdb_database = *mut _duckdb_database` (bindgen.rs:2180/2209).
One definition of each — no duplicate-definition markers; every other
`duckdb_database` occurrence is a reference, not a definition (19 total
matches, 2 definitions). ALIVE.

## Commands (replay)

```bash
# claim 1
grep -rn "rustc-link-lib" /Users/sac/frozen-duckdb/crates --include='build.rs'
cd /tmp/rel2-sys/links-probe && CARGO_TARGET_DIR=/tmp/rel2-sys/target cargo run -q
otool -L /tmp/rel2-sys/target/debug/links-probe | grep -c libduckdb   # 1
# claim 2
cd /tmp/rel2-sys/env-probe && CARGO_TARGET_DIR=/tmp/rel2-sys/target cargo build 2>&1 | grep ENVPROBE
# claim 3
B=/tmp/rel2-sys/target/debug/build/frozen-duckdb-sys-efecf83130e89ea7/out/bindgen.rs
grep -c '^pub type duckdb_database =' "$B"   # 1
```

Scratch trees (`/tmp/rel2-sys/{links-probe,env-probe,target}`) are lane
leases, disposable; the repo tree was not modified.
