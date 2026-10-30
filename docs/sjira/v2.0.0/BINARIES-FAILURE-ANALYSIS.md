# BINARIES-FAILURE-ANALYSIS — build-binaries.yml historical failures (GH1, 2026-10-19)

Both historical `Build Frozen DuckDB Binaries` runs FAILED; no GitHub Release with
`libduckdb_{arch}.dylib` assets has ever been published, which is why every local build
falls back to the 30-90 min DuckDB source compile.

## Run inventory

| run | ref | date | duration | failed legs |
|---|---|---|---|---|
| 36239600176 | v1.5.5 | 2026-09-26 | 11m0s | x86_64 + arm64 (both "Build mega-library") |
| 37940771273 | v26.10.8 | 2026-10-09 | 15m38s | x86_64 (build failure); arm64 (fail-fast cancel) |

## Run 36239600176 (v1.5.5, 2026-09-26)

Job matrix: `build-binaries (x86_64)` (ID 108397496481) and
`build-binaries (arm64)` (ID 108397496504) — both failed at step "Build mega-library"
after ~10m50s. `release` never ran (0s, skipped).

### Failure excerpt (x86_64 leg)

```
2026-09-26T11:41:32Z  Downloaded smallvec v1.16.2        <- last cargo line
2026-09-26T11:52:09Z  Failed to build mega-library: Failed to compile DuckDB locally
2026-09-26T11:52:09Z  ##[error]Process completed with exit code 1.
```

### arm64 leg

The arm64 leg's failed-step log lines are not retrievable via `--log-failed` for this
run; what IS witnessed: the leg failed at the same "Build mega-library" step, 4 seconds
after x86_64 (job duration 10m53s vs 10m49s), with the identical exit-1 shape —
consistent with the same root cause. No independent arm64 signal exists.

### Root cause

- Timeline: cargo downloads end at 11:41:32Z; the failure line lands at 11:52:09Z — a
  ~10.5-minute silent gap. The v1.5.5-era `frozen-build` harness suppressed the CMake
  compiler/linker output entirely (the run-2 code carries the retrospective comment
  `"Failed to compile DuckDB locally" and no diagnostic` — this run is that defect, witnessed).
- Classified: **compile error at the old pin (v1.5.5), diagnostics swallowed**. NOT a
  150-min timeout (died at ~11 min), NOT the asset-find or release step (never reached —
  "Find built binary" / "Upload binary" both show `-` skipped).
- The v1.5.5 GitHub Release assets were never published (README known open item), so
  `ensure_binary()` hit the source-compile fallback in CI and it failed. The exact compiler
  error is unrecoverable — the log contains no diagnostic.
- Post-job `fatal: No url found for submodule path 'vendors/duckdb-rs' in .gitmodules`
  (exit 128) is cleanup noise, not a cause.

### Fix class

- Pin change (v1.5.5 -> 2.0.0) + diagnostic surfacing. Both already landed:
  the current workflow sets `CMAKE_OSX_ARCHITECTURES` per matrix leg, and the builder's
  error path now prints `{:#}` anyhow chains instead of the bare message.
- **Reproducibility at 2.0.0**: LOW. The dev pin (v2.0-cyanoptera b6013f7) compiles
  clean locally (X3/V1 evidence). The residual risk is CI-only (runner toolchain skew),
  not the pin.

## Run 37940771273 (v26.10.8, 2026-10-09)

Job matrix: `build-binaries (x86_64)` (ID 113854270298) failed at "Build mega-library"
at 13m59s; `build-binaries (arm64)` (ID 113854270552) was **canceled** at 14m28s
(`##[error]The operation was canceled.` at 2026-10-14:16Z) — fail-fast killed it after
the x86_64 leg failed, so the arm64 leg has no independent signal (annotation: "The
process '/opt/homebrew/bin/git' failed with exit code 128" is post-cleanup noise on both).
`release` job skipped again.

### Failure excerpt (x86_64 leg, the real signal)

```
[duckdb] extension/httpfs/httpfs_httplib_client.cpp.o + more httpfs objects:
  "_SSL_get1_peer_certificate", referenced from: ... in httpfs_httplib_client.cpp.o
  "_SSL_get_error", ... "_SSL_new", ... "_SSL_write", ... "_TLS_client_method", ...
  "_X509_get_subject_name", ... (dozens of `_SSL_*` / `_X509_*` symbols)
ld: symbol(s) not found for architecture x86_64
clang++: error: linker command failed with exit code 1 (use -v to see invocation)
make[2]: *** [extension/httpfs/httpfs.duckdb_extension] Error 1
make[1]: *** [extension/httpfs/CMakeFiles/httpfs_loadable_extension.dir/all] Error 2
make: *** [all] Error 2
##[error]Process completed with exit code 1.
```

### Root cause

- Classified: **compile/link error at the old pin (v26.10.8 source snapshot), extension
  link stage** — the httpfs extension's vendored httplib with OpenSSL was linked without
  the OpenSSL static libs on the CI x86_64 leg. NOT a timeout (13m59s << 150m), NOT
  asset-find, NOT release.
- This is a leg-specific environment/link-order failure (x86_64 only reached link; arm64
  was canceled before its own outcome), consistent with a missing `-lssl -lcrypto`
  link configuration for the loadable-extension target on the runner.
- **Reproducibility at 2.0.0**: MEDIUM. Same extension-link failure class can recur
  on the CI x86_64 leg (CI-only so far; the dev pin compiles clean locally where the
  OpenSSL discovery differs). The current workflow's per-leg `CMAKE_OSX_ARCHITECTURES`
  does not address OpenSSL discovery on the runner. If it recurs, fix class =
  **workflow change** (install/link openssl@3 explicitly for the extension stage, or
  disable the httpfs extension in the mega-library build — the frozen library's Rust
  consumers do not need a loadable httpfs extension binary).
- NOT a timeout-class failure, so a timeout bump is not the fix for either run.

## Bottom line

1. Both failures are **build/compile-class, at the old pins, in the "Build mega-library"
   step**, well short of the 150-min timeout. Asset-find and release steps never executed.
2. Run 1 (v1.5.5): silent compile failure, diagnostics swallowed by the old harness —
   unrooted further than "DuckDB local compile failed at v1.5.5 in CI".
3. Run 2 (v26.10.8): precisely rooted — httpfs extension link failure, missing OpenSSL
   symbols (`_SSL_*`, `_X509_*`), `ld: symbol(s) not found for architecture x86_64`.
4. Neither would necessarily reproduce at the 2.0.0 pin (compiles clean locally,
   X3/V1); the one CI-only residual risk is the x86_64 httpfs/OpenSSL extension link.
5. Fix classes: run 1 = pin change + diagnostic surfacing (landed); run 2 = workflow
   change (explicit OpenSSL wiring for the extension stage, or drop httpfs from the
   mega-library extension set) **if** it recurs at 2.0.0.
