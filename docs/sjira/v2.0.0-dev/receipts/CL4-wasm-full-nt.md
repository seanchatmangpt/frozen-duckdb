# CL4 — wasm FULL build with DISABLE_THREADS=ON (KD-05 size-close attempt)

Standing: **PARTIAL_ALIVE** — build/link/smoke all real and passing; size SLA NOT met.
Lane: CL4, repo /Users/sac/frozen-duckdb, 2026-10-19. No git operations.

## Subject

- Source: preserved tree `target/wasm-build/duckdb`, HEAD verified
  `b6013f7aa2f36caabb9466ce1791667ea8a280bb` (= SOURCE_PIN), version
  `v2.0.0-dev88595` (emitted by CMake configure).
- Upstream flag per ADV1: `-DDISABLE_THREADS=ON` — confirmed real:
  `CMakeLists.txt:605 option(DISABLE_THREADS "Disable support for multi-threading" FALSE)`.
  (`-DDUCKDB_NO_THREADS` is inert — no such option in this tree.)
- Toolchain: real emsdk 6.0.13 (e13ee988...), `~/emsdk/emsdk_env.sh`.

## Configure (emcmake)

```
emcmake cmake -S target/wasm-build/duckdb -B target/wasm-full-nt \
  -DCMAKE_CXX_STANDARD=17 -DCMAKE_CXX_FLAGS="-O3 -flto" \
  -DDUCKDB_EXPLICIT_PLATFORM=wasm_eh -DBUILD_EXTENSIONS="parquet;json" \
  -DENABLE_EXTENSION_AUTOLOADING=OFF -DENABLE_EXTENSION_AUTOINSTALL=OFF \
  -DBUILD_HTTPFS=OFF -DDISABLE_THREADS=ON -DBUILD_SHELL=OFF \
  -DBUILD_UNITTESTS=OFF -DCMAKE_BUILD_TYPE=Release
```

Configure accepted all DuckDB flags; `BUILD_HTTPFS`, `ENABLE_EXTENSION_*`
reported as unused by the project (same as prior lanes; upstream ignores them
in this build path). Extensions statically linked: core_functions, parquet,
json, **icu** (icu is added by upstream
`extension/extension_config.cmake:17-18` — not by our flags).

`cmake --build target/wasm-full-nt -j` → exit 0, `[100%] Built target
duckdb_local_extension_repo` (`target/wasm-full-nt/build.log`).

## Final link

```
em++ -O3 -flto -sWASM=1 -sSTANDALONE_WASM=1 -sALLOW_MEMORY_GROWTH=1 \
  -sEXPORTED_FUNCTIONS=_duckdb_library_version,_duckdb_open,_duckdb_close,_duckdb_query,_duckdb_connect,_duckdb_disconnect \
  -sENVIRONMENT=node -sTOTAL_MEMORY=268435456 --no-entry \
  -o duckdb_nt.js src/libduckdb_static.a tools/cpp/libduckdb_cpp_api.a third_party/*/*.a
```

Link exit 0. Artifact `target/wasm-full-nt/duckdb_nt.wasm`.

## Smoke (node, raw instantiation — `target/wasm-full-nt/smoke.mjs`)

STANDALONE_WASM glue exposes no ccall; smoke instantiates the raw module with
node's `node:wasi` preview1 + zero-return env stubs (unused for the version
call):

```
INSTANTIATE=OK
VERSION=v2.0.0-dev88595
```

Version exact-match confirms acceptance item 2 and 3 PASS.

## Size verdict: FAIL (KD-05)

| artifact | bytes | MiB | node smoke |
|---|---|---|---|
| `duckdb_nt.wasm` (-O3, unstripped by binaryen) | 28,946,095 | 27.61 | PASS |
| `duckdb_nt_oz.wasm` (wasm-opt -Oz) | 27,097,044 | 25.84 | **FAIL — CompileError: unknown import kind 0x7f** (binaryen output uses an import encoding node's V8 does not accept) |
| `oz/duckdb_oz_opt.wasm` (full -Oz rebuild) | 27,114,590 | 25.85 | not smokeable (same binaryen issue) |
| SLA | 26,214,400 | 25.00 | — |

Best node-instantiable artifact = **28,946,095 bytes (27.61 MiB)** —
**over the 25 MiB raw SLA by 2,731,695 bytes (10.4%)** vs raw byte count
(3.4% over SLA for the binaryen-shrunk candidate that node cannot load). The binaryen
shrink is real (−1.85 MB) but unusable under node 26.

sha256:
- `duckdb_nt.wasm`: `1133d257a8f26657c42fc7554671652acade3e61aa6e5b02f73fefd924a4dc4b`
- `duckdb_nt_oz.wasm`: `05baa8261afe67904965f1ebdf19b9ec5d27ff78850dd6c1771b9041e5fbb316`

## Levers not exercised (outside lane ownership)

- icu static link removal via `extension/extension_config_local.cmake` —
  requires writing into the preserved source tree
  `target/wasm-build/duckdb/extension/` (shared, not lane-owned). Expected
  several MB. This is the largest untried lever.
- `--strip-target-features` pass failed (`--enable-gc` required; attempt to
  combine errored: "distinct rec groups would be identical after binary
  writing"). Not pursued.

## Standing

BUILD_ALIVE / SMOKE_ALIVE / **SIZE_FAIL**. KD-05 closes only if the SLA is
defined against the binaryen-unusable number — it is not. Honest verdict:
the DISABLE_THREADS full build is real, instantiable, version-exact, and
still 27.6 MiB raw.
