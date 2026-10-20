# REL10 — Release Pre-Flight Table (T8 equivalent, current versions)

Date: 2026-10-19 · Lane REL10 · Subject: /Users/sac/frozen-duckdb @ master
(dbac0b4) · No git state mutations. Scratch: /tmp/rel10-scratch (replicas +
extracted builder package; repo tree untouched except `target/package/`
build artifacts).

## 1. Pre-flight table — `cargo publish --dry-run --allow-dirty`

| Crate | Exit | Verdict | Cargo output (verbatim, real run 2026-10-19) |
|---|---|---|---|
| frozen-duckdb-builder | 0 | PASS (witnessed) | `Packaged 10 files, 457.8KiB (84.7KiB compressed)` … `Compiling frozen-duckdb-builder v2.0.0-dev (…/target/package/frozen-duckdb-builder-2.0.0-dev)` … `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 1.93s` … `Uploading frozen-duckdb-builder v2.0.0-dev (…)` → `warning: aborting upload due to dry run` |
| frozen-duckdb-sys | 101 | EXPECTED-FAIL (ordering artifact — `frozen-duckdb-builder` not yet on the crates.io index) | `error: failed to prepare local package for uploading` / `Caused by:` / `  no matching package named \`frozen-duckdb-builder\` found` / `  location searched: crates.io index` / `  required by package \`frozen-duckdb-sys v2.0.0-dev (/Users/sac/frozen-duckdb/crates/frozen-duckdb-sys)\`` |
| frozen-duckdb | 101 | EXPECTED-FAIL (ordering artifact — `frozen-duckdb-sys` not yet on the index) | see full verbatim block below |

Full verbatim blocks (real run logs, /tmp/rel10-scratch/pub-{sys,main}.log):

`frozen-duckdb-sys` (exit 101):

```
error: failed to prepare local package for uploading

Caused by:
  no matching package named `frozen-duckdb-builder` found
  location searched: crates.io index
  required by package `frozen-duckdb-sys v2.0.0-dev (/Users/sac/frozen-duckdb/crates/frozen-duckdb-sys)`
```

`frozen-duckdb` (exit 101):

```
error: failed to prepare local package for uploading

Caused by:
  no matching package named `frozen-duckdb-sys` found
  location searched: crates.io index
  required by package `frozen-duckdb v2.0.0-dev (/Users/sac/frozen-duckdb/crates/frozen-duckdb)`
```

Ordering law (PB4 §4 #4, re-witnessed): builder → sys → main, topological;
each `101` above is the registry-absence of its path dep, not a packaging
defect — both crates package clean (`cargo package -l`, exit 0, file lists
in §3).

## 2. Compensating verification — replica build with publish-rewritten manifests

The `101`s only prove the path dep cannot resolve against the index; they do
not prove the packaged tree builds once the ordering gap closes. The
prescribed compensating check: replicate the packaged file sets with
publish-rewritten manifests (path dep → version-only
`frozen-duckdb-builder = "2.0.0-dev"` / `frozen-duckdb-sys = "2.0.0-dev"`),
resolved against the **extracted builder `.crate`** (the exact bytes that
would be uploaded) via `[patch.crates-io]` (same source-replacement
mechanism as PB4 §2).

Shape (scratch /tmp/rel10-scratch):

- `builder-pkg/` — extracted `frozen-duckdb-builder-2.0.0-dev.crate`
  (86,770 bytes real `.crate`).
- `sys-replica/` — full copy of `crates/frozen-duckdb-sys`; manifest
  rewritten: `frozen-duckdb-builder = "2.0.0-dev"` (no `path =`).
- `main-replica/` — full copy of `crates/frozen-duckdb`; manifest
  rewritten: `frozen-duckdb-sys = "2.0.0-dev"` (no `path =`).
- Scratch workspace root (real root manifest with members swapped) carrying
  `[patch.crates-io]` for sys + builder → local replicas.

Real runs (fresh `CARGO_TARGET_DIR` per replica, clean):

```
# sys-replica (dep: frozen-duckdb-builder = "2.0.0-dev")
   Compiling frozen-duckdb-builder v2.0.0-dev (/private/tmp/rel10-scratch/builder-pkg)
   Compiling frozen-duckdb-sys v2.0.0-dev (/private/tmp/rel10-scratch/sys-replica)
warning: frozen-duckdb-sys@2.0.0-dev: Using prebuilt DuckDB binary:
  /Users/sac/.frozen-duckdb/cache/v2.0.0-dev-arm64/libduckdb_arm64.dylib
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 28.71s
exit=0
```

```
# main-replica (dep: frozen-duckdb-sys = "2.0.0-dev")
   Compiling frozen-duckdb-builder v2.0.0-dev (/private/tmp/rel10-scratch/builder-pkg)
   Compiling frozen-duckdb-sys v2.0.0-dev (/private/tmp/rel10-scratch/sys-replica)
   Compiling frozen-duckdb v2.0.0-dev (/private/tmp/rel10-scratch/main-replica)
warning: frozen-duckdb-sys@2.0.0-dev: Using prebuilt DuckDB binary: …libduckdb_arm64.dylib
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 20.51s
exit=0
```

Verdict: the publish-rewritten dep graph (version-only requirements bound to
the extracted builder package) builds clean through all three crates — the
`101` standing is purely publish ORDER, not package CONTENT.
Additionally, `cargo package` for builder ran its in-package verify-compile
(`Finished … in 1.65s`, exit 0) — the extracted bytes themselves compile.

Residual gap (unchanged from PB4 delta table #1/#2/#6): index checksums,
real registry resolution without `[patch]`, and runtime dylib resolution
(KD-15 rpath) are NOT exercised by this replica.

## 3. Header shipping gate + package file sets

`cargo package -l --allow-dirty -p frozen-duckdb-builder` (exit 0) contains:

```
vendored-headers/duckdb.h
vendored-headers/duckdb.hpp
```

Header gate: **PASS** — both vendored headers ship in the builder package
(sys's build.rs/bindgen consumes them; docs.rs path depends on them).

File sets (real `cargo package -l`, exit 0 each):

- builder (10): `.cargo_vcs_info.json`, `Cargo.lock`, `Cargo.toml`,
  `Cargo.toml.orig`, `LICENSE-DUCKDB.md`, `README.md`,
  `examples/rel1_witness.rs`, `src/lib.rs`, `vendored-headers/duckdb.h`,
  `vendored-headers/duckdb.hpp`
- sys (13): `.cargo_vcs_info.json`, `Cargo.lock`, `Cargo.toml`,
  `Cargo.toml.orig`, `LICENSE-DUCKDB.md`, `README.md`, `build.rs`,
  `src/error.rs`, `src/lib.rs`, `src/raw_statement.rs`, `src/string.rs`,
  `wrapper.h`, `wrapper_ext.h` — build.rs + both wrappers ship.
- main (80 files): src tree, `src/main.rs`, 4 examples, 20 test files
  (AD8-excluded machine-local fixtures confirmed absent: `gate_v01_latency`,
  fleet A–E, rev, value_audit tests are NOT in the list — the 12
  `warning: ignoring test … not included in the published package` lines in
  the main dry-run log match the exclude list exactly).

## 4. Package sizes

| Crate | Basis | Size |
|---|---|---|
| frozen-duckdb-builder | real `.crate` (`ls -l` on
  `target/package/frozen-duckdb-builder-2.0.0-dev.crate`) | 86,770 bytes (84.7 KiB compressed; 457.8 KiB / 10 files unpacked per cargo) |
| frozen-duckdb-sys | computed from `cargo package -l` file sizes | 13 files, 16,263 bytes (15.9 KiB) |
| frozen-duckdb | computed from `cargo package -l` file sizes | 80 files, 884,241 bytes (863.5 KiB) |

## 5. Standing

- builder `publish --dry-run`: **ALIVE** (exit 0, upload aborted by dry-run
  — no publish performed).
- sys / main `publish --dry-run`: **EXPECTED-FAIL-101** (ordering artifact;
  dep absent from index by construction pre-order).
- Compensating replica build (publish-rewritten manifests, extracted
  builder bytes): **ALIVE** (sys exit 0, main exit 0, clean target dirs).
- Header gate: **PASS** (duckdb.h + duckdb.hpp in builder package).
- No publish; `published`/`crates-io-uploaded`/`release-complete` never
  claimed. Scratch replicas live only under /tmp/rel10-scratch.
