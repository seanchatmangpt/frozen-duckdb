# X5 — CI binaries build status receipt

Date: 2026-10-20 (UTC run started 2026-10-20T02:02:30Z)
Repo: /Users/sac/frozen-duckdb (master)
Commands: `gh run list --workflow build-binaries.yml --limit 3` (exit 0),
`gh run view 39356742812` (exit 0),
`gh run view 39356742812 --log-failed` (exit 1 — logs contain failure text; expected non-zero via pipe tail),
`gh release view v2.0.0 --json assets` → `release not found` (exit 1).

## Run: v2.0.0 — Build Frozen DuckDB Binaries (run 39356742812, push of tag v2.0.0)

Conclusion: **failure**, elapsed 21m30s. Triggered by DG3's tag push.

| Leg | Runner | Result | Elapsed | Failed step |
|---|---|---|---|---|
| build-binaries-linux (arm64) | ubuntu-24.04-arm | X failure | 1m51s | Build mega-library |
| build-binaries-linux (x86_64) | ubuntu-latest | X failure | 2m1s | Build mega-library |
| build-binaries (x86_64) macOS | macos-15-xlarge? (x86_64) | X failure | 11m38s | Build mega-library (httpfs link errors) |
| build-binaries (arm64) macOS | macos arm64 | PASS | 21m21s | — (artifact `libduckdb_arm64` uploaded) |
| release | — | skipped (needs) | 0s | — |

## Failure causes

- Linux x86_64 + arm64 (both, ~2 min in): local-compile fallback cmake configure failed —
  `Could NOT find CURL (missing: CURL_LIBRARY CURL_INCLUDE_DIR)` in
  `build/_deps/httpfs_extension_fc-src/CMakeLists.txt:29` (httpfs extension dep, pulled
  because the prebuilt release asset for v2.0.0 does not exist → fallback compiled
  DuckDB from source). Fix direction: install `libcurl4-openssl-dev` (and note the vcpkg
  warning for httpfs) or build without httpfs.
- macOS x86_64: compile got much further (11m38s), died at link of the httpfs loadable
  extension: `clang++: error: linker command failed with exit code 1`,
  `make: *** [extension/httpfs/httpfs.duckdb_extension] Error 1` (httpfs/OpenSSL symbol
  link errors in httpfs_httplib_client.cpp.o). Same httpfs extension is the common factor.
- macOS arm64 leg passed (21m21s); artifact `libduckdb_arm64` uploaded, but the `release`
  job never ran (needs all legs green), so no GitHub Release v2.0.0 exists:
  `gh release view v2.0.0 --json assets` → `release not found` (exit 1).

## Asset state

No release, no assets. `ensure_binary()` cache-miss fallback (local source compile) will
remain the path for all legs except nothing — the one macOS arm64 artifact sits in CI
artifacts, not in a release.

## Standing

CI status for v2.0.0 binaries: **BLOCKED** — 3 of 4 build legs fail on the httpfs
extension (missing libcurl dev headers on Linux; OpenSSL link failure on macOS x86_64);
release job blocked on all-green needs.
