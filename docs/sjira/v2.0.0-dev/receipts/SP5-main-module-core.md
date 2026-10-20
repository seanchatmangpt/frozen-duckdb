# SP5-main-module-core: MAIN_MODULE core so the ICU sidecar can dynamically load

Date: 2026-10-19. Lane: SP5 (heavy build).
Repo: /Users/sac/frozen-duckdb (no git operations performed).
Ownership held: target/wasm-main-module/ + this receipt only.

## Subject

| field | value |
|---|---|
| source pin | b6013f7aa2f36caabb9466ce1791667ea8a280bb (git -C target/wasm-build/duckdb rev-parse HEAD == pin, verified this session) |
| source tree | target/wasm-build/duckdb (reused read-only; no writes to shared lanes) |
| emcc | 6.0.13 (real emsdk at ~/emsdk; brew emcc fails sanity) |
| core artifact | target/wasm-main-module/artifact/duckdb-main.wasm — 48,841,421 bytes (46.58 MiB) |
| JS glue | target/wasm-main-module/artifact/duckdb-main.js — 611,585 bytes (required: MAIN_MODULE=1 needs the dylink JS runtime) |
| ICU sidecar (pthreads-matched) | target/wasm-main-module/artifact/icu-pthread.duckdb_extension.wasm — 1,344,596 bytes (1.28 MiB), dylink.0 present |
| ICU sidecar (SP4 original) | target/wasm-icu-sidecar/artifact/icu.duckdb_extension.wasm — 1,344,816 bytes, dylink.0 present, but side link lacked shared-memory ABI |

## Build method (relink, not recompile)

- Reused the preserved LTO static archives in target/wasm-build/build (built
  from the same pin) — MAIN_MODULE is a link-time option, so no core recompile
  was needed. Objects were extracted per-archive (emar x into objs1..objs4)
  because MAIN_MODULE=1 links --whole-archive and (a) listing
  libduckdb_static.a twice duplicate-symbols under LTO, and (b) member names
  collide across archives (370 unique objects; single-dir extraction lost 35).
- Third_party archives libduckdb_utf8proc.a / libduckdb_zstd.a are ALREADY
  unity-built inside libduckdb_static.a — adding them duplicates symbols
  (Utf8Proc::Analyze, ZSTD_compressBlock_*). Dropped; core+extensions only.
- Final link (exit 0):
  emcc -O3 -flto -sUSE_PTHREADS=1 -sMAIN_MODULE=1 -fwasm-exceptions --no-entry
    -sALLOW_MEMORY_GROWTH=1
    -sEXPORTED_RUNTIME_METHODS='["ccall","cwrap","UTF8ToString","FS","loadDynamicLibrary","loadWebAssemblyModule"]'
    -sEXPORTED_FUNCTIONS='["_duckdb_open","_duckdb_close","_duckdb_connect","_duckdb_disconnect","_duckdb_query","_duckdb_destroy_result","_duckdb_library_version","_duckdb_free","_duckdb_malloc","_duckdb_value_int32","_duckdb_result_chunk_count","_dlopen","_dlsym","_dlerror","_free"]'
    -Wno-error=undefined $(find objs1 objs2 objs3 objs4 -name '*.o') -lc++ -lc++abi
    -o artifact/duckdb-main.js
  (the -fwasm-exceptions -lc++ -lc++abi triple was required: emcc does not
  auto-add libc++ under MAIN_MODULE with explicit objects; the wasm_eh
  platform objects need the wasmexcept libc++abi variant — noexcept variants
  lack __cxa_throw.)
- ICU sidecar: rebuilt in-lane (icu-build/, same configure as SP4's script)
  then RE-linked manually:
  emcc icu-build/extension/icu/icu.duckdb_extension -o artifact/icu-pthread.duckdb_extension.wasm
    -O3 -flto -sSIDE_MODULE=2 -pthread -sSHARED_MEMORY=1 -sEXPORTED_FUNCTIONS='["_icu_duckdb_cpp_init"]'
  Root cause (typed): upstream sets WASM_THREAD_FLAGS only under
  -DUSE_WASM_THREADS=ON (CMakeLists.txt:220); with it OFF the extension side
  link (extension_build_tools.cmake:374) emits an unshared-memory side module
  even when CXX flags carry -sUSE_PTHREADS=1. SP4's sidecar hit exactly this:
  instantiation failed with `mismatch in shared state of memory, declared = 0,
  imported = 1`.

## Negative results (honest, load-bearing)

1. MAIN_MODULE=2 is the WRONG main-module mode here. It links the main module
   "like a side module": the core's libc++/emscripten symbols stay external
   imports (150+ env.* imports: _Znwm, mutex, ostream...), the module imports
   its own memory, and GOT.func imports come typed (wasm function references),
   which JS cannot fabricate. 27,573,448-byte MAIN_MODULE=2 artifact retained
   knowledge only; it instantiates and answers version(), but duckdb_open
   traps because libc++ has no implementation behind it. Closing the libc++
   gap under MAIN_MODULE=2 requires emscripten's shared-library arrangement
   (separately linked libc module). MAIN_MODULE=1 is the only self-contained
   mode.
2. Whole-archive + LTO duplicate-symbol failure for MAIN_MODULE=1 via plain
   archive lists (wasm-ld "duplicate symbol" x20 from a single listing of
   libduckdb_static.a) — worked around by linking extracted objects.
3. malloc is not exported by the main module (only duckdb_malloc/free);
   EXPORTED_FUNCTIONS list did not include _malloc; smoke uses duckdb_malloc.
4. Rung D (SQL LOAD) HANGS the runtime (>120s, killed by timeout). The LOAD
   statement reaches DuckDB's internal LoadExternalExtension path but never
   returns. Honest standing: side module LINKS (rung C passes through
   emscripten dylink) but DuckDB's own extension-load path does not complete
   at this pin under this build; not a linkage ABI failure.

## Acceptance ladder — verbatim outputs

Run 1 (SP4 sidecar, rung C fails as predicted — shared-memory ABI mismatch):

```
$ node smoke_glue.cjs   # SIDE = target/wasm-icu-sidecar/artifact/...wasm
[ladder] A: version = v2.0.0-dev88595
[ladder] B: ... PASS (full C API cycle)
[ladder] C: sidecar written to FS, 1344816 bytes
[ladder] C: side-load FAILED: WebAssembly.Instance(): Import #0 "env" "memory": mismatch in shared state of memory, declared = 0, imported = 1
EXIT=2
```

Run 2 (pthreads-matched sidecar, full pass through rung C):

```
$ node smoke_glue.cjs
[ladder] A: version = v2.0.0-dev88595
[ladder] B: open rc=0 db=3034296 connect rc=0 conn=3043488
[ladder] B: query rc=0 cols=1 err_ptr=0 internal=11433760
[ladder] B: chunks = 1
[ladder] B: value(0,0) = 42
[ladder] B: PASS (full C API cycle)
[ladder] C: sidecar written to FS, 1,344,596 bytes
[ladder] C: loadDynamicLibrary -> handle
[ladder] C: PASS (side module linked)
EXIT=0
```

Run 3 (rung D): after C PASS, SQL `LOAD '/icu-pthread.duckdb_extension.wasm'`
issued; no further output; killed at 120 s (EXIT=124).
## Size accounting vs 25 MiB budget (honest FAIL)

| artifact | bytes | MiB |
|---|---||
| minimal standalone core (prior lane) | 24.69/24.54 MiB cited | 24.69 |
| MAIN_MODULE=1 core (this lane, js+wasm; wasm only) | 48,841,421 | 46.58 MiB |
| MAIN_MODULE=2 core (superseded variant) | 27,573,448 | 26.30 MiB |
| SP4 ICU sidecar | 1,344,816 | 1.28 MiB |

MAIN_MODULE metadata + whole-archive (dead-code stripping is disabled by
whole-archive link of every core object) roughly doubles the core vs the
minimal standalone build. 46.58 MiB vs 25 MiB budget: FAIL, honestly
recorded. Note the whole-archive requirement of MAIN_MODULE=1 is the dominant
cost driver (the standalone min core links lazily and strips dead code).

## Standing

- Rungs A (version) and B (full C API query cycle: open -> connect -> query
  SELECT 42 -> value==42 -> destroy/disconnect/close) ALIVE: the main-module
  core executes the full C API query surface under node with the emscripten
  glue.
- Rung C ALIVE: the ICU side module instantiates and LINKS through emscripten
  dylink (loadDynamicLibrary -> handle) — SP4's rung-3 BLOCKED is closed at
  the linkage seam. Required a pthreads-matched sidecar (shared-memory ABI);
  SP4's artifact had the unshared-memory side link (typed root cause above).
- Rung D PARTIAL/BLOCKED: SQL LOAD of the linked extension HANGS inside
  DuckDB's internal load path (>120 s, killed). The extension is linked into
  the process but does not complete registration through DuckDB's
  ExtensionManager at this pin under this build. Typed mechanism: LOAD-path
  hang, not linkage failure; ICU tz queries not witnessed.

## Falsifier / replay

```
source ~/emsdk/emsdk_env.sh
cd /Users/sac/frozen-duckdb/target/wasm-main-module
node smoke_glue.cjs            # exit 0 = rungs A-C alive
# full relink replay in receipt's Build method section
```
