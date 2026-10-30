# ADV4 Receipt — Local Extension Build Attempt (KD-02 / tpch 404s)

Lane: ADV4, 2026-10-19. Repo: /Users/sac/frozen-duckdb. No git operations.
Owned files only: `scripts/build_extensions_local.sh` (new) + this receipt.

## Subject

- Engine pin: duckdb `d54cbbe3df17e8e5ce68312f483b4155836112aa`
  (engine `SELECT version()` → `v2.0.0`), cached engine
  `~/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib`.
- Upstream 404s re-confirmed live this session:
  - `http://extensions.duckdb.org/d54cbbe3d/osx_arm64/tpch.duckdb_extension.gz` → HTTP 404
  - `https://community-extensions.duckdb.org/d54cbbe3d/osx_arm64/flock.duckdb_extension.gz` → HTTP 404

## Result in one line

- **tpch: loadable built at the pin and LOADING GREEN against the cached dev
  engine** (ALIVE behind one one-line harness change — Blocker B1).
- **flock: honest loud-fail** — source-incompatible with the pin's
  parser-extension API. KD-02's flock half stays open, upstream-blocked.

## tpch — what ran, what it produced

Native build from a fresh shallow clone at the exact pin (the preserved
`target/wasm-build/duckdb` tree is the wasm tree; no native tree existed, so
`scripts/build_extensions_local.sh tpch` cloned the pin):

    [adv4] source tree at pin: d54cbbe3df17e8e5ce68312f483b4155836112aa
    [adv4] configuring (ninja, release, tpch as unlinked loadable)
    [adv4] building tpch_loadable_extension (pulls the core static lib)
    [adv4] artifact: .../target/adv4/native-duckdb/build/extension/tpch/tpch.duckdb_extension ( 23M)
    [adv4] installing into ~/.duckdb/extensions/d54cbbe3d/osx_arm64/
    [adv4] done
    EXIT=0

Configure line (exact):

    cmake -S $TREE -B build -G Ninja -DCMAKE_BUILD_TYPE=Release \
      -DBUILD_EXTENSIONS="tpch" -DBUILD_EXTENSIONS_ONLY=0 \
      -DBUILD_UNITTESTS=OFF -DENABLE_SANITIZER=OFF -DCMAKE_OSX_ARCHITECTURES=arm64
    cmake --build build --target tpch_loadable_extension -j$(sysctl -n hw.ncpu)

Two lessons burned into the script:

1. `BUILD_EXTENSIONS_ONLY=1` removes `duckdb_static` from the build graph, so
   the loadable link dies with `ld: library 'duckdb_static' not found`. The
   loadable statically embeds the core engine — the core must be built.
2. CMake caches the flag: switching a previous extensions-only configure
   requires an explicit `-DBUILD_EXTENSIONS_ONLY=0`, not flag removal.

## LOAD witnesses (real output)

Rust probe (`target/adv4/probe`, `frozen-duckdb` crate as dependency,
`Connection::open_with_flags(":memory:", Config::default().allow_unsigned_extensions()?)`):

    version(): v2.0.0
    LOAD-OK /Users/sac/frozen-duckdb/target/adv4/native-duckdb/build/extension/tpch/tpch.duckdb_extension
    tpch state: true/false/NOT_INSTALLED   # before install
    lineitem rows: 60175                   # CALL dbgen(sf=0.01), exact TPC-H SF=0.01 count
    orders rows: 15000
    part rows: 2000
    customer rows: 1500

No-network INSTALL witness (after copying the artifact to
`~/.duckdb/extensions/d54cbbe3d/osx_arm64/` — note the version directory is
the SHORT SHA from the registry URL, NOT `v2.0.0`):

    INSTALL+LOAD tpch: OK (local dir, no explicit path)
    state: true/true
    lineitem rows: 60175

`CALL tpch(1)` note: prepare errors with `Table Function with name "tpch" is
not in the catalog, but it exists in the tpch extension` on this pin — the
tpch(1) helper table function is gone/renamed in the v2.0.0 tpch
extension; `dbgen` and direct queries over the generated tables work. The
ignored test `test_tpch_query_execution` uses `PRAGMA tpch(4)` and will need
checking against this when un-ignored (Blocker B3).

## What still blocks full un-ignore of the repo tests

Ran the ignored tests un-ignored as a court:

    cargo test -p frozen-duckdb --test tpch_integration_test -- --ignored
    ...
    failures: (all 6)
    test result: FAILED. 0 passed; 6 failed

Root cause per failure: the tests call `Connection::open_in_memory()` without
config, so LOAD of the unsigned locally built extension hits the signature
gate:

    Cannot change allow_unsigned_extensions setting while database is running
    (when set post-open)
    ...
    LOAD fails with "signature is either missing or invalid and unsigned
    extensions are disabled by configuration (allow_unsigned_extensions)"

Blockers, all outside ADV4 file ownership (coordinator actions):

- **B1 (one line)**: tests must open with
  `Config::default().allow_unsigned_extensions()` — locally built extensions
  are unsigned; the signature gate is checked even for local INSTALLs.
- **B2 (setup step)**: run `scripts/build_extensions_local.sh tpch` once to
  place the artifact in `~/.duckdb/extensions/d54cbbe3d/osx_arm64/` (add to
  doctor/verify-gates as a non-default opt-in; NOT in default CI since it
  compiles the whole core, minutes of CPU).
- **B3 (test-content check)**: `test_tpch_query_execution` uses
  `PRAGMA tpch(4)`; on this pin the `tpch(1)`-style helper failed in the
  probe. Verify before flipping the ignore; if the pragma is gone, rewrite the
  test to query the generated tables directly (e.g. the Q4 join over
  lineitem/orders).

## flock — loud-fail (KD-02 flock half stays open)

Attempted per plan: cloned `dais-labo/flock` (HEAD dev, pushed 2026-09-30,
submodule `extension-ci-tools` b777c70), re-pointed its `duckdb` submodule
from `08e34c44` to the engine pin `d54cbbe3d` so any produced loadable would
carry the right version stamp, then `make release -j8`:

    git -C duckdb fetch --depth 1 origin d54cbbe3d... && git checkout --detach FETCH_HEAD
    HEAD is now at d54cbbe3d Bump httpfs (#27476)
    make release → Error 2, 3 compile errors in src/flock_extension.cpp

Real compiler output (target/adv4/flock-build-2.log):

    flock_extension.hpp:34:26: error: assigning to 'parse_function_t'
      (aka 'ParserExtensionParseResult (*)(ParserExtensionInfo *, const vector<SimpleToken> &)')
      from incompatible type
      'ParserExtensionParseResult (ParserExtensionInfo *, const std::string &)'
    flock_extension.cpp:32:12: error: no matching constructor for
      initialization of 'Parser'  (Parser now requires ClientContext/ParserOptions)
    flock_extension.cpp:53:62: error: comparison of distinct pointer types

Typed conclusion: **UNSUPPORTED-UPSTREAM** for flock at this pin. flock's
parser-extension implementation targets the pre-v2.0
`parse_function_t(const std::string&)` + default-constructible `Parser` API;
the pin changed `parse_function_t` to `vector<SimpleToken>` and requires
explicit Parser construction. Even flock's own pinned duckdb submodule
(`08e34c44`, newer than our pin) would build a loadable stamped with the
wrong version — the engine's version check would refuse it. Close condition
unchanged: upstream flock migrates to the v2.0 parser-extension API.
Building it for them (porting flock's parser shim) is possible local work but
out of ADV4 scope (would create a maintained fork).

## Artifacts

- Script: `/Users/sac/frozen-duckdb/scripts/build_extensions_local.sh`
  (`tpch` green, `flock` typed loud-fail, `bash -n` clean)
- tpch loadable: `target/adv4/native-duckdb/build/extension/tpch/tpch.duckdb_extension`
  (23,798,174 bytes, Release arm64, version-stamped `d54cbbe3d`)
- Installed copy: `~/.duckdb/extensions/d54cbbe3d/osx_arm64/tpch.duckdb_extension`
- Build trees (scratch, lane-leased, safe to delete at integration):
  `target/adv4/native-duckdb/` (pin clone + build), `target/adv4/flock/`,
  probe `target/adv4/probe/`
- Logs: `target/adv4/tpch-configure.log`, `tpch-build.log`,
  `flock-build-2.log`, `flock-script-run.log`

## Standing

- tpch local closure: **ALIVE behind B1/B2/B3** — extension verified loading
  and generating exact TPC-H data on the exact cached engine; repo tests
  remain `#[ignore]`d until the coordinator applies B1–B3 (test files and
  gate wiring are outside ADV4 ownership).
- flock: **UNSUPPORTED-UPSTREAM** — KD-02's flock half stays open with the
  typed reason above; the `#[ignore]` reasons in `flock_tests.rs` remain
  accurate.
- KD-02 status: **PARTIAL** — the "community extensions have no build for the
  dev pin" class is now split: tpch is locally closable with a one-line
  harness change; flock remains upstream-blocked (API migration, not just a
  missing artifact).
