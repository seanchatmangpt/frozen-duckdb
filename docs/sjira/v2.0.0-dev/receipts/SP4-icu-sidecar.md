# SP4-icu-sidecar: ICU extension as standalone loadable wasm module

Date: 2026-10-19. Lane: SP4.
Subject: `/Users/sac/frozen-duckdb/target/wasm-icu-sidecar/artifact/icu.duckdb_extension.wasm`
Script: `/Users/sac/frozen-duckdb/scripts/build_wasm_icu_sidecar.sh`
Repo: `/Users/sac/frozen-duckdb` (no git operations performed).

## Command

```
source ~/emsdk/emsdk_env.sh
/Users/sac/frozen-duckdb/scripts/build_wasm_icu_sidecar.sh
```

Exit 0 (final run, incremental — all objects cached).

## Subject identity

| field | value |
|---|---|
| source pin | `b6013f7aa2f36caabb9466ce1791667ea8a280bb` (verified `git rev-parse HEAD` == pin) |
| source tree | `/Users/sac/frozen-duckdb/target/wasm-build/duckdb` (reused; clone fallback in script) |
| emcc | 6.0.13 (real emsdk at `~/emsdk`; brew emcc fails sanity — no wasm LLVM backend) |
| platform | `DUCKDB_EXPLICIT_PLATFORM=wasm_eh` |
| mode | `WASM_LOADABLE_EXTENSIONS=1` — extension built-but-not-linked |
| configure | `BUILD_EXTENSIONS="" BUILD_UNITTESTS=OFF BUILD_SHELL=OFF`; unused-var warnings on `BUILD_HTTPFS / ENABLE_EXTENSION_AUTOINSTALL / ENABLE_EXTENSION_AUTOLOADING` (upstream CMake: "change them at runtime instead") |
| cxx flags | `-O3 -flto -sUSE_PTHREADS=1` (matches `scripts/build_wasm_duckdb.sh` proven set) |
| build target | `icu_loadable_extension` (bare `icu` target does not exist at this pin — first attempt: `gmake: *** No rule to make target 'icu'`) |
| extension enable | upstream local-dev hook: `extension/extension_config_local.cmake` with `duckdb_extension_load(icu)` in the scratch checkout (the mechanism `extension/extension_config.cmake` itself documents for local dev) |

## Acceptance ladder

### Rung 1 — artifact with a size: PASS

```
-rw-r--r--@ 1 sac  staff  4579830 Oct 19 20:11 icu.duckdb_extension
-rwxr-xr-x@ 1 sac  staff  1344816 Oct 19 20:11 icu.duckdb_extension.wasm
```

- `icu.duckdb_extension.wasm` = 1,344,816 bytes (1.28 MiB), side module
  (`-sSIDE_MODULE=2`), metadata appended by upstream `scripts/append_metadata.cmake`
  — `strings` confirms `b6013f7aa2` twice (install-type + extension-version fields).
- `icu.duckdb_extension` = 4,579,830 bytes — an ar **archive** (pre-link static
  stage), NOT a wasm module. Do not ship it as one.
- One earlier link failure was environmental: bare `emcc` on PATH resolved to the
  broken brew emscripten (no wasm LLVM); relink under emsdk env passes. All 62
  extension objects compiled clean.

### Rung 2 — magic/file sanity: PASS

```
[icu-sidecar] MAGIC SKIP (not a wasm module): .../icu.duckdb_extension
[icu-sidecar] MAGIC OK: .../icu.duckdb_extension.wasm starts with \0asm
file(1): icu.duckdb_extension.wasm: WebAssembly (wasm) binary module version 0x1 (MVP)
```

### Rung 3 — smoke-load attempt: BLOCKED (typed; mechanism named)

Smoke harness: `/Users/sac/frozen-duckdb/target/wasm-icu-sidecar/smoke_load.mjs`,
run against the distribution core `target/wasm-min/duckdb.wasm` (ADV1 receipt:
`BUILD_EXTENSIONS=""`, no pthreads, no LTO, same pin, same emcc 6.0.13).

```
[smoke-load] exports: ["memory","main","__indirect_function_table","duckdb_open",
"duckdb_close","duckdb_library_version","duckdb_free","_start","setThrew",
"_emscripten_stack_restore","emscripten_stack_get_current"]
[smoke-load] BLOCKED: missing exports: duckdb_connect,duckdb_query,
duckdb_destroy_result,duckdb_disconnect
```

The minimal core exports no query surface, so a `LOAD` statement cannot even be
issued against it. Two independent seam gaps, both upstream-verifiable:

1. **Core is not a main module.** `grep -rn MAIN_MODULE` over the pinned tree:
   zero hits outside `third_party` — the duckdb wasm core target never builds
   with `-sMAIN_MODULE`, while extensions link with `-sSIDE_MODULE=2`
   (`extension/extension_build_tools.cmake:303,374`). A side module's imports
   (`env.__memory_base`, `GOT.func`/`GOT.mem`, 330 imports total) can only be
   satisfied by a main module; standalone instantiation fails as expected:
   `WebAssembly.Instance(): Import #0 "env": module is not an object or function`.
   The ADV1/MAX7 distribution core is a standalone wasm, so `LOAD` of any
   dynamic extension cannot resolve.
2. **Config surface**: the ADV1 core was built with a minimal
   `EXPORTED_FUNCTIONS` list (`duckdb_open/close/library_version/free` only) —
   even the C-API query path is absent from the artifact.

Cheaper-path probe (operator ask): the extension registry does NOT serve icu
wasm_eh at the pin:

```
HTTP/2 404  https://extensions.duckdb.org/b6013f7aa2f36caabb9466ce1791667ea8a280bb/wasm_eh/icu.duckdb_extension.wasm
HTTP/2 404  v2.0-cyanoptera/wasm_eh   HTTP/2 404  main/wasm_eh
HTTP/2 404  v2.0.0-dev/wasm_eh        HTTP/2 200  v1.4.3/wasm_eh
```

Only the v1.4.3 release line serves icu wasm_eh (HTTP 200) — not compatible
with a v2.0.0-dev core pin.

## Standing

**PARTIAL_ALIVE / BLOCKED at rung 3.**

- Rungs 1-2 ALIVE: a real, correctly-formatted 1.28 MiB icu side module exists
  at `/Users/sac/frozen-duckdb/target/wasm-icu-sidecar/artifact/icu.duckdb_extension.wasm`.
- Rung 3 BLOCKED — not hidden: the dynamic-loading seam is not wired at this
  pin. Closing it requires a core built with `-sMAIN_MODULE=2` (matching
  pthreads/LTO settings, and a full EXPORTED_FUNCTIONS query surface), which is
  a separate core-build lane; this lane's ~35-min budget did not rebuild the
  core.

## Falsifier / replay

```
source ~/emsdk/emsdk_env.sh && \
  /Users/sac/frozen-duckdb/scripts/build_wasm_icu_sidecar.sh   # exit 0, artifact regenerates
node /Users/sac/frozen-duckdb/target/wasm-icu-sidecar/smoke_load.mjs  # exit 2 = BLOCKED typed
```

A future core lane that exports `duckdb_query` and links a MAIN_MODULE core
flips rung 3 by rerunning the smoke harness.
