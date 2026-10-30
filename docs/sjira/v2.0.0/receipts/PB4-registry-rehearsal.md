# PB4 — Registry-Shaped Rehearsal (no publish)

Date: 2026-10-19 · Lane PB4 · Subject: /Users/sac/frozen-duckdb @ master
(adbd260) · No git state mutations. Scratch: /tmp/pb4.

## 1. `cargo package` rehearsal

- `cargo package --allow-dirty --offline -p frozen-duckdb-builder`
  → **PASS** (exit 0). Real output:
  `Packaged 8 files, 437.1KiB (79.3KiB compressed)`; verify-compile of
  `target/package/frozen-duckdb-builder-2.0.0` finished in 1.50s.
  `.crate` contents (real `tar tzf`):
  `.cargo_vcs_info.json`, `Cargo.lock`, `Cargo.toml`, `Cargo.toml.orig`,
  `README.md`, `src/lib.rs`, `vendored-headers/duckdb.h`,
  `vendored-headers/duckdb.hpp`.
- `cargo package --allow-dirty --offline -p frozen-duckdb-sys` → **FAIL,
  expected**: `no matching package named frozen-duckdb-builder found,
  location searched: crates.io index`.
- `cargo package --allow-dirty -p frozen-duckdb` → **FAIL, expected**:
  `no matching package named frozen-duckdb-sys found`.
  These two failures are the registry-resolution gap itself — `cargo
  package` resolves the workspace's `path =` deps against the crates.io
  index, and 2.0.0 does not exist there. This is exactly what
  publish-order fixes (sys → builder → main, topological, each version
  on the index before its dependent packages).

## 2. Consumer rehearsal via `[patch.crates-io]` (source-replacement shape)

Consumer project: `/tmp/pb4/consumer` — depends on `frozen-duckdb =
"2.0.0"` from the index namespace, with all three crates bound to the
local checkout via `[patch.crates-io]` (the same source-replacement
mechanism the registry index uses). Run the README quickstart verbatim.

Result (real run):

```
Compiling frozen-duckdb-builder v2.0.0 (…/crates/frozen-duckdb-builder)
Compiling frozen-duckdb-sys v2.0.0 (…/crates/frozen-duckdb-sys)
warning: frozen-duckdb-sys@2.0.0: Using prebuilt DuckDB binary:
  /Users/sac/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib
Finished `dev` profile … in 45.10s
Running `target/debug/pb4-consumer`
User: Alice        ← after rpath fix, no env var
```

- First run **failed at loader time**: `dyld: Library not loaded:
  @rpath/libduckdb.dylib` — the binary linked but could not locate
  `libduckdb.dylib` (this is the already-registered KD-15 rpath
  divergence, now reproduced on a consumer out-of-tree).
- Two real fixes proven:
  1. `DYLD_LIBRARY_PATH=<cache dir>` (env-only, no rebuild).
  2. Consumer-side link-arg (durable, no env):
     `.cargo/config.toml` → `rustflags = ["-C", "link-arg=-Wl,-rpath,
     /Users/sac/.frozen-duckdb/cache/v2.0.0-arm64"]` → `User: Alice`,
     exit 0, no env var. This is the exact one-line consumer workaround
     to document for flip-day (or the sys build.rs should emit the
     install-name/rpath itself — open item).

## 3. `cargo install --path` rehearsal

Skipped as directed (lane task exclusion, not a silent skip).

## 4. Flip-day delta: real registry publish vs this rehearsal

What changes and what this rehearsal could NOT exercise:

| # | Delta | Rehearsal status |
|---|-------|------------------|
| 1 | **Path→registry resolution**: deps of sys/builder/main resolve against the crates.io index; the `path =` fields are stripped from the published manifests. Rehearsal used `[patch.crates-io]` source replacement instead. | NOT exercised for sys/main (packaging them required the index to have their deps) |
| 2 | **Checksums**: the index assigns each `.crate` a sha256; consumers verify it. Rehearsal had no checksum layer. | NOT exercised |
| 3 | **`.crate` contents of sys/main**: only builder's `.crate` was built (8 files above). sys carries `build.rs`, `wrapper.h`, `wrapper_ext.h` — must confirm they land in its `.crate` (they are not in `include` exclusions; `cargo package -l` on flip-day confirms). | PARTIAL (builder only) |
| 4 | **Publish order**: sys must be on the index before main packages; builder before sys packages. Topological: sys → builder → main? No — sys depends on builder (`frozen-duckdb-sys` Cargo.toml declares `frozen-duckdb-builder = { path = …, version = "2.0.0" }`), so **builder → sys → main**. | Identified, not exercised |
| 5 | **`cargo package --offline` lock/yank**: `yoke-derive v0.8.3 is yanked` warning surfaced during packaging; a registry build re-resolves, yanked pins can break consumer resolution. | Observed, unresolved |
| 6 | **Runtime dylib resolution**: registry consumers get no `target/debug` cwd; the KD-15 `@rpath/libduckdb.dylib` load failure is the default experience. Proven consumer-side fixes: rpath link-arg (durable) or DYLD_LIBRARY_PATH (env). | Reproduced + fix proven |
| 7 | **`.cargo_vcs_info.json` / Cargo.lock in .crate**: builder's .crate carries a Cargo.lock; registry consumers ignore the in-crate lock (workspace resolver only). | informational |
| 8 | **Version match law**: the crate version mirrors the DuckDB version (2.0.0); tag/release assets (`v2.0.0` GitHub Release is unpublished — README open item) must exist before main's builder can download rather than compile locally. | known open item |

## 5. Standing

- Builder `.crate` builds and its verify-compile passes: **ALIVE**.
- Consumer quickstart via source-replacement: **ALIVE** (with rpath
  link-arg; without it, loader fails — KD-15 confirmed out-of-tree).
- sys/main `cargo package`: **EXPECTED-FAIL (registry-absence)**, the
  gap publish order closes.
- No publish performed; `published`/`crates-io-uploaded`/
  `release-complete` never claimed.
