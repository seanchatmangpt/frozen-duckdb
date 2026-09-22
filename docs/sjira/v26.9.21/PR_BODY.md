# feat: DuckDB 1.5.5 support — pin bump, release asset fix, API layer restoration (closes #1)

> **STATUS (wave-5 G8, 2026-09-22):** PR #3 is **MERGED** (merge commit
> a3a69e4, 2026-09-22T00:59:40Z). This file is the merged PR's body, preserved
> verbatim as release evidence — every checklist box, dry-run table, and cited
> test head below is historical, not open work. The live release procedure is
> `PUBLISH_RUNBOOK.md`; current state lives in `MILESTONE.md`.

## Summary

This release makes crate `frozen-duckdb` v1.5.5 track upstream DuckDB v1.5.5:
the crate version now mirrors the bundled DuckDB release. It is a drop-in bump
for existing users of the `duckdb`/`duckdb-rs` API surface — the same public
API, now backed by DuckDB 1.5.5.

Three structural problems are fixed along the way:

1. **Release assets never matched the downloader.** The CI workflow published
   `libfrozen_mega_{arch}.dylib`, while `frozen-duckdb-builder::ensure_binary()`
   downloads `libduckdb_{arch}.dylib` — so every user silently fell back to a
   full local DuckDB compile. Assets now ship under the exact downloaded names.
2. **The vendored duckdb-rs API layer did not compile.** Module wiring,
   `crate::` paths, and the dependency set (arrow 56, hashlink 0.10, strum 0.27,
   fallible-iterator/streaming-iterator, rust_decimal, cast) are restored so the
   drop-in API layer is part of the build again.
3. **Linked binaries could not find the dylib at runtime.** `frozen-duckdb-sys`
   now propagates the cache directory via `links = "duckdb"` + `DEP_DUCKDB`
   metadata, and the `frozen-duckdb` build script turns that into an `@rpath`
   entry — executables and tests run with no `DYLD_LIBRARY_PATH` setup.

Closes #1 (asked 2026-03-23: "Do you plan to release a version with DuckDB 1.5?").

## Changes

### builder — cache normalization + vendored headers
- Normalize the `~/.frozen-duckdb/cache/v1.5.5-{arch}/` layout on **every**
  binary-acquisition path: headers under `duckdb/`, dylib as plain
  `libduckdb.dylib` (neutral link name).
- Vendored DuckDB 1.5.5 headers (`vendored-headers/{duckdb.h,duckdb.hpp}`)
  ship inside the `frozen-duckdb-builder` crate — header generation works
  offline regardless of how the binary was obtained.
- Automatic download of prebuilt universal macOS dylibs
  (`libduckdb_{arm64,x86_64}.dylib`) from GitHub Releases on first build;
  local source-compile fallback pinned at upstream tag `v1.5.5` otherwise.

### sys — bindgen + links
- Fixed bindgen double-include of `duckdb.h` (redefinition errors).
- `links = "duckdb"` declared; build script exposes `cargo:DUCKDB_LIB_DIR` so
  dependents receive `DEP_DUCKDB_DUCKDB_LIB_DIR` and exactly one crate links
  DuckDB.

### API layer — vendored duckdb-rs restored to compilation
- Module wiring and `crate::` paths repaired across the vendored tree.
- Dependency set updated to the versions the restored layer needs:
  `arrow 56`, `hashlink 0.10`, `strum 0.27`, `fallible-iterator 0.3`,
  `fallible-streaming-iterator 0.1`, `rust_decimal`, `cast`.

### linking — runtime @rpath propagation
- `frozen-duckdb` build script reads `DEP_DUCKDB_DUCKDB_LIB_DIR` and emits an
  `@rpath`/`rustc-link-arg` entry pointing at the cache directory, so CLI
  binaries, tests, and dependent crates execute against the frozen 1.5.5 dylib
  with a clean environment.

### CI — release asset naming
- `build-binaries.yml` stages and uploads `libduckdb_{arch}.dylib`, matching
  `ensure_binary()` download URLs exactly; default build version bumped to
  `v1.5.5`.

### scripts + docs — sweeps
- Version pins swept 1.4.x → 1.5.5 across scripts and docs; generated-manifest
  pinning updated to `duckdb = "1.10505.0"` (duckdb-rs encoding of DuckDB
  1.5.5); drop-in example and README updated to 1.5.5 reality.

### versioning
- Workspace version 1.5.5; `frozen-duckdb`, `frozen-duckdb-sys`,
  `frozen-duckdb-builder` all publish as 1.5.5. Internal path dependencies
  carry `version = "1.5.5"` keys (publish-safe).

## Test plan

> Final consolidated run on merged head `be1b611`: `cargo test --workspace` → **exit 0, 294 passed / 0 failed across 16 suites**.

Gate results owned by their tickets are recorded in
`docs/sjira/v26.9.21/MILESTONE.md` and each ticket's History. Slots below are
to be filled from those tickets before merge:

- ✅ `cargo test -p frozen-duckdb --lib` → exit 0 (T2A: 140 passed; superseded on merged head by full-workspace run)
- ✅ frozen_duckdb_tests / dropin_compatibility_tests / core_functionality_tests → exit 0 (10 + 7 + 8 passed)
- ✅ arrow 6/6, parquet 6/6, vss 8/8, tpch 6/6 (real INSTALL/LOAD/dbgen runs; dbgen serialized — process-global state, SIGSEGV under parallel harness); flock 11/11 SKIP-gated on FLOCK_TEST=1; polars compile-gated (UNSUPPORTED: polars dev-dep not carried)
- ✅ `cargo build --workspace --examples` → exit 0; `cargo run --example basic_usage` → exit 0 (1000 queries in 153 ms); `cargo run --example performance_comparison` → exit 0
- ✅ remaining `1.4.0` md mentions: CHANGELOG release note, CONTEXT.md Phase-1 history, ADR snapshot, sjira tickets — all intentional-historical
- ✅ clippy `-D warnings` → exit 0; `cargo fmt --all -- --check` → exit 0; all workflows yaml-parse (re-verified on merged head)
- ✅ zero live v1.4 pins in scripts/; `bash -n` green; duckdb-rs clones pinned v1.10505.0 (verified via git ls-remote)
- ✅ cache normalization observed live (deleted duckdb/ + libduckdb.dylib → build restored both from vendored headers + symlink); LC_RPATH = v1.5.5-arm64 cache; `env -i` CLI run exit 0; ctypes `duckdb_library_version()` == v1.5.5

### Release pre-flight (T8, observed 2026-09-21)

| crate | command | exit | outcome |
|-------|---------|------|---------|
| frozen-duckdb-builder | `cargo publish --dry-run --allow-dirty -p frozen-duckdb-builder` | 0 | PASS — packaged + full verification build from unpacked package |
| frozen-duckdb-sys | `cargo publish --dry-run --allow-dirty -p frozen-duckdb-sys` | 101 | `no matching package named frozen-duckdb-builder found` on crates.io — builder 1.5.5 not yet published (ordering artifact, see note) |
| frozen-duckdb | `cargo publish --dry-run --allow-dirty -p frozen-duckdb` | 101 | `no matching package named frozen-duckdb-sys found` — same class |

Note on the two failures: cargo resolves path+version dependencies from the
registry during publish verification, so a downstream crate's dry-run cannot
pass until its upstream crate is actually live on crates.io. This is expected
pre-publication behavior, not a manifest defect — the presence of the
`version = "1.5.5"` keys is exactly what triggers the registry rewrite.
Compensating verification: exact replicas of the packaged file sets (as listed
by `cargo package --list`) with the publish-rewritten manifests (inheritance
flattened, path dep → version-only) were built clean against the extracted
builder package: sys replica exit 0, frozen-duckdb replica exit 0.

Header shipping gate: `cargo package -p frozen-duckdb-builder --list` contains
`vendored-headers/duckdb.h` and `vendored-headers/duckdb.hpp`.

Package sizes (crates.io limit 10 MiB per `.crate`):

| crate | `.crate` | unpacked |
|-------|----------|----------|
| frozen-duckdb-builder | 424.4 KiB (observed) | 2.3 MiB |
| frozen-duckdb-sys | not producible pre-publication | ~717 KiB (computed from package file list) |
| frozen-duckdb | not producible pre-publication | ~802 KiB (computed from package file list) |

All well under the cap; headers dominate the builder package and compress well.

### Release sequence for the operator (in order)

1. `cargo publish -p frozen-duckdb-builder`
2. Wait for the crates.io index to reflect builder 1.5.5 (usually under a
   minute; a `no matching package named frozen-duckdb-builder found` on the
   next step means the index has not caught up — retry).
3. `cargo publish -p frozen-duckdb-sys`
4. `cargo publish -p frozen-duckdb`

### Release-asset verification checklist (after tag push)

- [ ] `build-binaries.yml` run on tag `v1.5.5` completes green
- [ ] GitHub release contains `libduckdb_arm64.dylib` and `libduckdb_x86_64.dylib`
      (exact names, no `libfrozen_mega_*` leftovers)
- [ ] Clean machine: `cargo new t && cd t && cargo add frozen-duckdb` +
      a `Connection::open` + query smoke test builds and runs with no
      `DYLD_LIBRARY_PATH` and no local DuckDB checkout
- [ ] `~/.frozen-duckdb/cache/v1.5.5-{arch}/` populated on that machine by the
      automatic download (check `duckdb/` headers + `libduckdb.dylib` present)
- [ ] `python3 -c "import ctypes; print(ctypes.CDLL('<cache>/libduckdb.dylib').duckdb_library_version())"`
      prints `v1.5.5`

## Known limitations

- Prebuilt release assets are **macOS-only** (universal arm64 + x86_64).
  Windows and Linux builds fall back to compiling DuckDB v1.5.5 from source
  locally (automatic, pinned); prebuilt assets for those platforms are on the
  roadmap.
- The two downstream `--dry-run` outcomes above can only be re-verified green
  after the real publication of `frozen-duckdb-builder` (crates.io index
  resolution) — see the release sequence.
