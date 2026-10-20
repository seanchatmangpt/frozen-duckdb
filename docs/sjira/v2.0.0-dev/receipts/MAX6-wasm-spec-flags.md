# MAX6 — wasm spec-flags size-close attempt (operator exact flag set)

Standing: **BLOCKED (size SLA FAIL)** — build/link/smoke all real and passing; raw-size and wire gates split. Lane MAX6, repo /Users/sac/frozen-duckdb, 2026-10-19. No git operations.

## Subject

- Source: preserved tree `target/wasm-build/duckdb`, upstream HEAD `b6013f7aa2` (= SOURCE_PIN), emits `v2.0.0-dev88595` (smoke-verified below).
- Toolchain: emsdk 6.0.13 (`e13ee988ce3d87a738dd8f3661d02223308c412a`), `~/emsdk/emsdk_env.sh`, node v26.10.0, macOS arm64.
- Owner dir: `target/wasm-spec/` (fresh build dir; preserved tree untouched).

## Configure (emcmake) — exact operator flag set

```
emcmake cmake -S target/wasm-build/duckdb -B target/wasm-spec \
  -DCMAKE_CXX_STANDARD=17 -DCMAKE_CXX_FLAGS="-O3 -flto" \
  -DDISABLE_THREADS=ON -DBUILD_SHELL=OFF -DBUILD_UNITTESTS=OFF \
  -DBUILD_EXTENSIONS="json;parquet" -DDUCKDB_EXPLICIT_PLATFORM=wasm_eh \
  -DCMAKE_BUILD_TYPE=Release
→ exit 0
-- Linked into DuckDB: extensions [core_functions, parquet, json],
   capabilities [httplib, loadable_extensions, local_extension_repository]
```

Configure accepts every flag. Note: **icu is NOT linked** in this build (upstream
`extension_config.cmake` adds icu only when `BUILD_EXTENSIONS` includes it or when
the default-all path runs; the CL4 receipt's icu lever was real and this flag set
exercised it) — yet the artifact is still larger than CL4's (below).

## Build

```
cmake --build target/wasm-spec -j 16   → exit 0
[100%] Built target duckdb_local_extension_repo   (target/wasm-spec/build.log)
```

## Final link (exact operator flags)

```
em++ -O3 -flto -sWASM=1 -sSTANDALONE_WASM=1 -sFILESYSTEM=0 \
  -sINITIAL_MEMORY=64MB -sALLOW_MEMORY_GROWTH=1 \
  -sEXPORTED_FUNCTIONS="['_duckdb_open','_duckdb_query','_duckdb_library_version','_duckdb_free','malloc','free']" \
  --no-entry -o duckdb_spec.wasm \
  src/libduckdb_static.a tools/cpp/libduckdb_cpp_api.a third_party/*/*.a
```

First attempt exited with `em++: error: undefined exported symbol: "free" [-Werror]`.
Relink with `-Wno-error=undefined` → link succeeds, but `em++: warning: undefined
exported symbol: "free" / "malloc"` persists and **neither symbol is in the export
table** (LTO internalizes them; `duckdb_free` is exported and is the supported
free path). Honest deviation from the operator spec's export list.

## Artifact + size ladder (acceptance results)

| # | gate | result | verdict |
|---|---|---|---|
| 1 | raw ≤ 25 MiB (26,214,400 B) | **29,242,868 B = 27.89 MiB** | **FAIL** (+2,028,468 B, +7.7%) |
| 2 | node WASI smoke `version==v2.0.0-dev88595` | `INSTANTIATE=OK` / `VERSION=v2.0.0-dev88595`, exit 0 (`target/wasm-spec/smoke_spec.mjs`) | PASS |
| 3 | gzip wire ≤ 9.5 MiB | 6,986,843 B = 6.67 MiB (gzip -9) | PASS |
| 4 | wasm-opt -Oz | 27,274,225 B = 26.01 MiB (`--all-features` needed; without it: `Fatal: error validating input`) | shrunk −1.92 MB but **NOT loadable**: node `CompileError: unknown import kind 0x7f` — same binaryen/V8 incompatibility CL4 receipted |

Comparison row:

| build | raw | node loadable | smoke |
|---|---|---|---|
| KDC-05 min slice | 24.69 MiB | yes | PASS |
| CL4 full-nt | 27.61 MiB | yes | PASS |
| **MAX6 spec-flags** | **27.89 MiB** | yes | PASS |

The operator spec flag set does **not** close KD-05: it is 0.28 MiB *larger* than
CL4's link despite dropping icu (plausible causes, not receipts: `-sFILESYSTEM=0`
+ 64MB initial-memory layout, export-set differences, LTO variance). KDC-05's
min slice remains the only ≤25 MiB loadable artifact.

sha256:
- `duckdb_spec.wasm`: `06dc589de516e8a6a7ac996ea427e318fa0d0e17fb1b115e56cdddcab2f9f9bf`
- `duckdb_spec_oz.wasm`: `b9877322c79cba361ea9babf998e6e538d8b2ac86617224fbfc4c366176a7746`
- `duckdb_spec.wasm.gz`: `aaf48ae722fe9cb934d507b72b8a85d4ec7821d64786ce433410ff5db5cbda6a`

## Import/export surface (node-verified on raw artifact)

```
imports: 22 (wasi_snapshot_preview1.{...} + env stubs — same 22-shim shape as KDC-05)
exports: ["memory","duckdb_open","duckdb_query","duckdb_library_version",
          "duckdb_free","_initialize","__indirect_function_table","setThrew",
          "_emscripten_stack_restore","emscripten_stack_get_current"]
```

`malloc`/`free` absent (em++ undefined-export warnings above); `duckdb_free`
present. Smoke exercised `duckdb_library_version` only.

## Commands + exits (real transcript anchors)

- configure exit 0; build exit 0 (`target/wasm-spec/build.log`).
- first link: `em++: error: undefined exported symbol: "free" [-Werror]` (non-zero).
- relink exit 0 with `-Wno-error=undefined`; warnings retained in this receipt.
- `node smoke_spec.mjs` → `INSTANTIATE=OK` / `VERSION=v2.0.0-dev88595`, exit 0.
- `wasm-opt -Oz` (no feature flag): `Fatal: error validating input`; with
  `--all-features`: emits 27,274,225 B but node `CompileError: unknown import
  kind 0x7f`.

## Levers not exercised

- KDC-05-style minimal slice combined with the spec's `-sFILESYSTEM=0` +
  64MB memory flags (KDC-05 used ENVIRONMENT=shell, no FILESYSTEM flag).
- `-DRUST_OPTIMIZATION`-class flags n/a; upstream `extension_config_local.cmake`
  icu removal not needed here (icu already absent).
- Strip pass over the raw artifact with `wasm-opt -Oz` features matched to the
  V8-acceptable subset only (unexplored).
