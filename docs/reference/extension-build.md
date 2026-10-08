# Extension / Library Build Pipeline

How the frozen `libduckdb` mega-library is produced, verified, and linked.
Grounded in `crates/frozen-duckdb-builder/src/lib.rs`,
`crates/frozen-duckdb-sys/build.rs`, `.github/workflows/build-binaries.yml`,
and `prebuilt/README.md`.

## What the "extension" actually is

There is **no separately compiled C-extension surface** in this repo. The C
surface is DuckDB's own C API, consumed in two ways:

- **FFI bindings, not a C build**: `crates/frozen-duckdb-sys` links the
  prebuilt shared library and runs bindgen over the pinned headers
  (`crates/frozen-duckdb-sys/build.rs` — `bindgen` on `wrapper.h` with
  `-I{lib_dir}`; no `cc` crate compiles any repo-owned C. The only C file,
  `scripts/duckdb_ffi.h`, is a validation fixture, not compiled into the
  library).
- **The compiled artifact is upstream DuckDB itself**, built with a fixed
  extension set via cmake — see the local-compile fallback below. Extension
  set: `parquet;json;icu;httpfs;tpch;tpcds;fts;inet;sqlsmith`, with
  `BUILD_JEMALLOC=ON` and `BUILD_AUTOLOAD=ON`
  (`crates/frozen-duckdb-builder/src/lib.rs:576-581`).

Pinned upstream tag: **DuckDB v1.5.5** (`const VERSION = "1.5.5"`,
`crates/frozen-duckdb-builder/src/lib.rs:15`; the workspace crate version is
also 1.5.5). The crates.io version-encoding law: `1.MAJOR*10000 +
MINOR*100 + PATCH`, so DuckDB 1.5.5 is published/consumed as `duckdb =
"1.10505.0"` (`crates/frozen-duckdb/src/lib.rs:15-18`,
`test-dependency/Cargo.toml:8-9`).

## Build order at `cargo build` time

`crates/frozen-duckdb-sys/build.rs` calls
`frozen_duckdb_builder::ensure_binary()`, which resolves in this order:

1. **Versioned cache** `~/.frozen-duckdb/cache/v1.5.5-{arch}/libduckdb_{arch}.{dylib,so}`
   — if present, it is slice-verified (`lipo -archs`) before use; a stale
   pre-TR7 cache holding the wrong slice fails the build with a remove-and-
   rebuild instruction.
2. **`prebuilt/` in the project directory** — `prebuilt/libduckdb_{arch}.dylib`
   (same naming law as release assets); slice-verified, then copied into the
   cache with the vendored headers.
3. **GitHub Release download** — asset
   `libduckdb_{arch}.dylib` (macOS) / `libduckdb_{arch}.so` (Linux) from
   `releases/download/v1.5.5/`. As of this writing **no release/tag has been
   published**, so downloads return HTTP 404 on every platform and the
   fallback runs (`release_asset_name_for`, lib.rs:66-76).
4. **Local compile fallback** — the operative path today (see below).

After any of these, the cache is normalized: vendored headers are placed at
`{cache}/duckdb/duckdb.h` / `duckdb.hpp` (`ensure_headers`) and a plain
`libduckdb.{dylib,so}` symlink is created (`ensure_link_name`) so `-lduckdb`
and the `@rpath/libduckdb.dylib` install name resolve.

## Arch detection (TR7)

`detect_architecture` (`crates/frozen-duckdb-builder/src/lib.rs:291`):

1. `CMAKE_OSX_ARCHITECTURES` — wins if it names exactly one supported slice
   (`x86_64`, `arm64`/`aarch64`).
2. `ARCH` — wins if the first is unset/empty/multi-arch/unknown (an invalid
   earlier value does not shadow a valid later one).
3. `uname -m` host fallback; an unrecognized host arch fails closed.

Multi-arch lists (`arm64;x86_64`), mis-cased (`X86_64`, `amd64`), and
`arm64e` are never adopted as a target. On macOS, when only the label is
known, the builder pins `-DCMAKE_OSX_ARCHITECTURES={label}` so the slice
clang emits cannot diverge from the label. Off macOS, a cross-arch local
compile is refused outright (`check_local_compile_target`) — a cross-arch
target would silently mislabel a host-arch library.

## Fail-closed verification

Every library consumed or downloaded is slice-checked on macOS with
`lipo -archs` (`verify_library_slice_for`, lib.rs:397): a library whose
carried slice does not include its label errors ("TR7 slice mismatch"),
a non-Mach-O (e.g. a 404 HTML body) errors ("not a readable Mach-O"), and a
mismatched **download is deleted again, never cached**
(`persist_verified_for`, lib.rs:451). Off macOS the check reports
UNVERIFIED (logged), never a silent pass.

## Local-compile fallback (the operative path)

`compile_duckdb_locally` (lib.rs:519):

```
git clone --depth 1 --branch v1.5.5 https://github.com/duckdb/duckdb.git
cmake .. -DCMAKE_BUILD_TYPE=Release \
  -DBUILD_EXTENSIONS=parquet;json;icu;httpfs;tpch;tpcds;fts;inet;sqlsmith \
  -DBUILD_JEMALLOC=ON -DBUILD_AUTOLOAD=ON \
  [-DCMAKE_OSX_ARCHITURES=<label or operator value>]   # macOS only
make -j4
```

Output resolved from `build/duckdb/libduckdb.dylib` (v1.5.x layout;
`find_built_library` also probes older layouts), slice-verified, cached.
Configure/make failures surface the captured stderr in the anyhow chain
instead of a bare "failed" message.

## CI release pipeline

`.github/workflows/build-binaries.yml` — triggered by `v*` tags or manual
dispatch (the dispatch `version` input must match the builder pin
`const VERSION`; mismatch refuses the run). Matrix: **macOS only**
(`macos-latest`), `arch: [x86_64, arm64]`, timeout 150 min.

Per leg:

1. Export `ARCH` and `CMAKE_OSX_ARCHITECTURES` from `matrix.arch`
   (`macos-latest` is an arm64 runner, so `uname -m` alone would mislabel
   the x86_64 leg — the TR7 fix). Both env vars are what detection, cache
   placement, and the cmake flag consume.
2. A throwaway Cargo project depends on `frozen-duckdb-builder` (path dep)
   and runs `ensure_binary()` (`cargo run --release`). Cache miss = full
   DuckDB source compile, 30-90 min on GH runners.
3. Find step: `$HOME/.frozen-duckdb/cache/**/v*/libduckdb_{arch}.dylib`,
   staged as `stage/libduckdb_{arch}.dylib`.
4. Upload artifact `libduckdb_{arch}`; the release job (tags only) attaches
   both dylibs via `softprops/action-gh-release@v2`.

## Platform matrix

| Platform | CI release asset | Consumer path |
|---|---|---|
| macOS arm64 | `libduckdb_arm64.dylib` (~117MB) | download, else `prebuilt/`, else local compile |
| macOS x86_64 | `libduckdb_x86_64.dylib` (~117MB) | same |
| Linux | none published (`.so` assets planned with a later tag) | local compile fallback (host arch only; cross-arch refused) |
| Windows | none, none planned | local compile fallback; extension kept for cache-name consistency only |

On docs.rs builds, `crates/frozen-duckdb-sys/build.rs` detects `DOCS_RS` and
generates bindings from the builder's `vendored-headers/` with **zero
linking directives** — no library needed (build.rs:85-138).

## Release-artifact layout

```
GitHub Release v1.5.5
├── libduckdb_x86_64.dylib      # DuckDB 1.5.5 + bundled extension set
└── libduckdb_arm64.dylib

~/.frozen-duckdb/cache/v1.5.5-{arch}/
├── libduckdb_{arch}.dylib      # cached copy of the release asset
├── libduckdb.dylib -> ...      # -lduckdb / @rpath link name
└── duckdb/
    ├── duckdb.h                # vendored/pinned headers (bindgen input)
    └── duckdb.hpp
```

Asset, cache, and `prebuilt/` names all follow one law:
`libduckdb_{arch}.{dylib|so}` (TR7 asset law, lib.rs:74-76), enforced by the
unit test `test_release_asset_name_matches_host_binary_path`.

## Linking chain

`frozen-duckdb-sys` (links = "duckdb") emits `-L` + `-lduckdb` + rpath
link-arg and `cargo:DUCKDB_LIB_DIR` metadata;
`crates/frozen-duckdb/build.rs` reads `DEP_DUCKDB_DUCKDB_LIB_DIR` and emits
the runtime rpath (`-Wl,-rpath,{lib_dir}`), unscoped so the lib's own test
harness also loads (`crates/frozen-duckdb/build.rs:17`).
