# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.2] - 2026-10-29

### Fixed

- Engine-level WASI virtual file system compatibility layer: eliminates advisory `fcntl` locking failures (`F_SETLK`/`F_GETLK`) and maps Linux `O_*` open flags to WASI preview1 bitmasks in DuckDB's `local_file_system.cpp` under `#if defined(__wasi__)`. Statically linking `libduckdb.a` into a `wasm32-wasip1` module achieves correct file semantics with zero consumer shims and zero `--wrap` link arguments.
- The wasm32 archive probe reads past the ar symbol index. A full-engine `libduckdb.a` puts the first wasm object about 10.6 MB in, so a 256 KiB window rejected a real archive.
- wasi-sdk 34's toolchain is `share/cmake/wasi-sdk-p1.cmake`, and its libc++ lives at `share/wasi-sysroot/lib/wasm32-wasip1/eh`. The wasip1 compile uses those paths and turns off `CMAKE_POSITION_INDEPENDENT_CODE` in the pinned DuckDB checkout.
- wasm32-wasip1 links `libduckdb_wasi_posix` (`wasi_posix.c`). wasi-libc does not provide `mmap`, `mlock`, `getpid`, `kill`, `getrlimit`, or `sched_getcpu`. The stubs fail closed. Added no-op stub for `flock(fd, op)`.

## [2.0.1] - 2026-10-22

### Fixed

- `wasm32-wasip1` links `static=duckdb` (`libduckdb.a` built with wasi-sdk) instead of the host dylib. The 2.0.0 build script always emitted `cargo:rustc-link-lib=dylib=duckdb`, so wasm-ld failed with `unable to find library -lduckdb`. The host dylib is not a substitute. Set `WASI_SDK_PATH` to compile the archive into `~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1/libduckdb.a`.

## [2.0.0] - 2026-10-22

### Changed

- Crate version is **2.0.0** so crates.io treats this line as the default
  stable release. The bundled DuckDB engine stays `v2.0.0-dev88595` at
  `b6013f7aa2f36caabb9466ce1791667ea8a280bb`. `0.1.0` remains the previous
  stable crate. `2.0.0-dev` remains a published prerelease.

## [2.0.0-dev] - 2026-10-19


### Added

- **VARIANT type surface**: `Type::Variant` and `LogicalTypeId::Variant` exposed on the main-crate type layer, with a `serde_json::Value` `FromSql` read arm so `VARIANT` columns round-trip into idiomatic Rust values.


### Changed

- **Bundled engine 1.5.5 -> 2.0.0-dev**: upstream pin moved to `duckdb/duckdb@b6013f7aa2f36caabb9466ce1791667ea8a280bb`; crate version tracks the bundled engine per `fd:duckdbVersionLaw` (workspace now 2.0.0-dev across `frozen-duckdb`, `frozen-duckdb-sys`, `frozen-duckdb-builder`), and `test-validation`'s version gate is dev-suffix-tolerant so `2.0.0-dev` passes the pinned-version FFI check.
- **Builder clone/tag split-brain fixed**: the local-compile fallback clones the pinned upstream SHA (`b6013f7aa2f36caabb9466ce1791667ea8a280bb`) instead of mixing a tag checkout with a SHA pin, so dev-line builds no longer straddle two upstream identities.


### Fixed

- **Dead bindgen snapshots removed**: stale generated bindings for superseded header states (`bindgen_bundled_version.rs`, `bindgen_bundled_version_loadable.rs`) dropped from `frozen-duckdb-sys`.
- **Vendored/prebuilt headers re-staged byte-identical**: `crates/frozen-duckdb-builder/vendored-headers/{duckdb.h,duckdb.hpp}` and `prebuilt/{duckdb.h,duckdb.hpp}` refreshed against the 2.0.0-dev upstream snapshot so every acquisition path compiles from the same API surface.

## [1.5.5] - 2026-09-21


### Added

- Automatic download of prebuilt DuckDB libraries from GitHub Releases on first build (macOS: `libduckdb_arm64.dylib`, `libduckdb_x86_64.dylib`), cached under `~/.frozen-duckdb/cache/v1.5.5-{arch}/`.
- Local source-compile fallback, cloning upstream DuckDB at the pinned `v1.5.5` tag, when no release asset is available.
- Vendored DuckDB 1.5.5 headers in `frozen-duckdb-builder`, so builds work offline regardless of how the binary was obtained.
- Runtime `@rpath` entry emitted by the `frozen-duckdb` build script (consuming `DEP_DUCKDB_DUCKDB_LIB_DIR` metadata exposed by `frozen-duckdb-sys`), so executables and tests link and run without `DYLD_LIBRARY_PATH` setup.
- A manufactured verification chain: `make verify` runs the rendered gate battery (`scripts/verify-gates.sh`: fmt check, clippy with warnings denied, workspace build, workspace tests, and the `test-validation` binary gate), then `ggen sync run` (re-render + receipt) and `ggen receipt verify` (receipt-chain check); the Makefile verify targets, gate runner, dry-run-publish gates, and receipt-contract matrix are rendered consequences of marketplace pack facts declared in `ggen.toml`, not hand-maintained files.
- Reconciliation governance for generated artifacts: `docs/GENESIS.md` — a manifest mapping every generated/consequence file to its owning source — checked by `make genesis-check` (source existence, consequence existence, upstream pin match, and ledger-row cross-check), so upstream changes name exactly which projections must re-render and orphans surface as defects.
- **Verify evidence pinned to the exact subject**: every verify-gates run records `ver:subjectHead` / `ver:subjectTree` (`git rev-parse HEAD` / `git rev-parse HEAD^{tree}`) into `scripts/verify-evidence.ttl`, so a committed evidence file proves which HEAD/tree passed the 5/5 gate battery (commit b9fa77c).
- **Standing gate for the ticket ledger**: `scripts/check_sjira_standing.sh` verifies every v26.9.21 ticket's frontmatter `aps:standing` equals its final History standing (explicit non-aps mappings documented in-file; CI_PROJECTION.md excluded; first run checked=45 mismatches=0), and the `gen_sjira_tickets*.sh` renderers gain a standing column threaded from a per-id standing map (terminal History standings; new tickets default OPEN), so regenerated tickets are standing-faithful (commit ed27449).


### Changed

- Bundled DuckDB upgraded from 1.4.0 to upstream v1.5.5 (released 2026-07-22); DuckDB pin bumped across the builder, test harnesses, and CI defaults; workspace version bumped to 1.5.5 for all three crates (`frozen-duckdb`, `frozen-duckdb-sys`, `frozen-duckdb-builder`); the crate version now tracks the DuckDB release it bundles.
- `test-dependency` now exercises the upstream `duckdb = "1.5"` line alongside the frozen binary.
- Download-path architecture detection honors an explicit single-slice `CMAKE_OSX_ARCHITECTURES` or `ARCH` value: detection order is `CMAKE_OSX_ARCHITECTURES` → `ARCH` → `uname -m`, and an explicit single-slice value drives detection, cache placement, and release-asset naming, while multi-arch lists and unknown values fall through to host detection. Root-causes the `v1.5.5` tag run's x86_64 leg failing closed on arch detection (commit abff9bc).


### Fixed

- CI release assets are now published under the exact names `frozen-duckdb-builder::ensure_binary()` downloads (`libduckdb_{arch}.dylib`), replacing the mismatched legacy `libfrozen_mega_{arch}.dylib` names that forced every user onto the local-compile fallback.
- Bindgen no longer double-includes `duckdb.h` (redefinition errors), and the builder normalizes the cache layout (`duckdb/` headers, plain `libduckdb.dylib` link name) on every binary-acquisition path.
- **CI release-asset rehearsal hardening**: the `v1.5.5` release workflow is rehearsed before the operator cuts the tag — matrix builds set `CMAKE_OSX_ARCHITECTURES` per architecture (honored by the builder's local-compile fallback), and the `ensure_binary()` cache-miss path (release-download miss → pinned local compile) was exercised against the exact asset names and paths the workflow publishes.
- **`column_names()` panic fix**: calling `column_names()` before statement execution no longer panics on a schema unwrap — it returns a proper error. Regression tests cover pre-execution, post-execution, and empty-result calls.
- **docs.rs build pattern**: `frozen-duckdb-sys` detects the `DOCS_RS` environment variable and generates bindings from the vendored headers while skipping library linking, so documentation builds on docs.rs (Linux, no prebuilt `.so`) succeed without a 30+ minute compile.
- **Linux / multi-arch builder support**: platform-aware release-asset naming (`.dylib` on macOS, `.so` on Linux), `CMAKE_OSX_ARCHITECTURES` override honored in the local-compile fallback, and a real Linux branch in `setup_env.sh`. Prebuilt `.so` assets are planned for a later tag; the `v1.5.5` tag ships macOS-only release assets, one per architecture (`libduckdb_arm64.dylib`, `libduckdb_x86_64.dylib`); Linux and Windows builds use the pinned local-compile fallback.
- **test-validation is a real gate**: the `test-validation` harness now asserts — binary present at the builder's expected path, `duckdb_library_version()` matching the pinned version via FFI, vendored header layout intact — and exits nonzero on any failure, instead of printing decorative checkmarks.
- **Repo hygiene**: kcura-era debris removed (dead smoke/CI-gate scripts, stray root test files, empty dataset directories); `HANDWRITTEN.md` ledger seeded.
- **CLI parse panic fixed — `convert` and `summarize` runnable again**: duplicate clap short flags (`-i`/`-o` on `convert --input-format`/`--output-format`, `-m` on `summarize --max-length`) tripped clap's debug asserts and aborted parsing with exit 101, making both commands unusable. The three options are now long-only (hand-repair ledgered in `HANDWRITTEN.md`, guarded by `tests/cli_surface_tests.rs`, which also renders every subcommand's `--help`).
- **DuckDB 1.5 `TIME_NS` rows readable**: DuckDB >= 1.5 exports `TIME_NS` columns as arrow `Time64(Nanosecond)`; the vendored arrow row reader only knew `Microsecond` and hit `unreachable!()` on such rows. The symmetric `Time64(Nanosecond)` arm was added (commit 36b180f), so `TIME_NS` columns now read instead of panicking.

