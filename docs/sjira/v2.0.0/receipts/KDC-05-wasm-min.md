# KDC-05 — wasm minimal slice (KD-05)

Date: 2026-10-19. Lane KDC-05. Owner dir: `target/wasm-min/`.
Source: PRESERVED tree `target/wasm-build/duckdb` (pin-verified), incremental
config in `target/wasm-build/build-min/` (flags verified against the LSS9
spec before building: `BUILD_EXTENSIONS=`, `BUILD_SHELL=OFF`,
`BUILD_UNITTESTS=OFF`, `-O3`, `DUCKDB_EXPLICIT_PLATFORM=wasm_eh`, no
`-sUSE_PTHREADS`; `CMAKE_CXX_FLAGS` exports `_duckdb_library_version` etc.).

## Commands + exit codes

```
# 1. incremental emscripten build of the static lib (cmake --build target duckdb)
make -j14 duckdb
→ [100%] Built target duckdb  (exit=0)

# 2. PIC side-module output (cmake lib target) — NOT standalone:
#    src/libduckdb.so = WebAssembly, 67,508,421 bytes raw,
#    49,144 imports (GOT/env, PIC) — instantiable only as a side module.

# 3. final standalone link (this lane's addition to the slice):
cd target/wasm-build/build-min && em++ -O3 \
  -s WASM=1 -s STANDALONE_WASM=1 -s ENVIRONMENT=shell \
  -s ALLOW_MEMORY_GROWTH=1 -s INITIAL_MEMORY=64MB -s MAXIMUM_MEMORY=4GB \
  -s EXPORTED_FUNCTIONS='["_duckdb_library_version","_duckdb_open","_duckdb_close","_duckdb_free","_malloc","_free"]' \
  src/libduckdb_static.a third_party/*/*.a extension/*/*.a \
  --no-entry -o target/wasm-min/kdc05-min.wasm
→ exit 0 (first attempt with `emcc` failed: undefined std::runtime_error dtor /
  __cxa_allocate_exception — C++ link needs em++; recorded, not worked around)

# 4. llvm-strip
~/emsdk/upstream/bin/llvm-strip target/wasm-min/kdc05-min.wasm
→ size unchanged (25,886,446 B; em++ -O3 already --strip-debug'd)

# 5. node probes (see transcript in this receipt)
```

## Artifact

`target/wasm-min/kdc05-min.wasm` — 25,886,446 bytes = **24.69 MiB**
(SLA 25 MiB = 26,214,400 B raw → PASS, before even relying on the strip).

## Acceptance transcript

```
[A2] size: 25886446 bytes = 24.69 MiB; imports: 22 ; exports:
     ["memory","duckdb_open","duckdb_close","duckdb_library_version",
      "duckdb_free","_initialize","__indirect_function_table","malloc",
      "free","setThrew","_emscripten_stack_restore",
      "emscripten_stack_get_current"]
[A2] duckdb_library_version exported: true
[A3] instantiable with 22-line wasi shim: OK
[A3] duckdb_library_version() = v2.0.0
[A5] heap before=67108864 after 64MB malloc=69009408 (ceiling 128MB)
[A5] heap OK
```

(A1 artifact exists = the `ls -l` above; A4 = 24.69 MiB < 25 MiB.)

Notes:
- The module needs a 22-symbol WASI/env shim to instantiate
  (`wasi_snapshot_preview1.{clock_time_get,fd_*,environ_*,random_get,
  proc_exit}` + `env.{_msync_js,__syscall_*}`); it is `STANDALONE_WASM`
  reactor-style (`crt1_reactor.o`, `--no-entry`), not zero-import. The full
  shim is 25 lines of node; shell stubs return errno 8 / -1.
- Prior failed instantiate attempts (receipted, honest): the raw PIC
  `libduckdb.so` (49,144 imports) and the concurrent-lane `duckdb.wasm`
  (23 env imports) both fail `WebAssembly.Instance(mod, {})` with
  `Import #0: module is not an object or function`. Standalone link is the
  load-bearing step.

## Standing

KD-05 → **PARTIAL_ALIVE**: minimal slice closes the version-gate claim
(v2.0.0 verified by real instantiation, not just an export
listing). The 25 MiB SLA for the FULL browser build (pthreads/OPFS) stays
open on the pthreads build; this slice proves the no-threads no-shell
configuration lands under SLA.

Unlocks KD-11: this minimal wasm is the module the OPFS courts mount.

## Concurrent-lane collision (disclosed)

Files `duckdb.wasm` (25,887,785 B, 23 imports, not standalone),
`duckdb-standalone.wasm` (7,034 B, 0 imports), and `smoke.mjs` appeared in
`target/wasm-min/` at 18:21 from another writer; this lane's receipted
artifact is `kdc05-min.wasm`. No files outside the lane's ownership were
touched.
