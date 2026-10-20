# DG3 dispatch receipt — v2.0.0-dev tag dispatch

Standing: **BUILD_FAILED** — tag `v2.0.0-dev` created and pushed from `ee9b65e`;
build-binaries run concluded **failure** (3 of 4 build legs failed); release job
**skipped**; no release, no assets. No retry performed (contract: verbatim capture,
no blind retry).

## Identity

- Repo: https://github.com/seanchatmangpt/frozen-duckdb (canonical checkout
  /Users/sac/frozen-duckdb), branch master
- Tag commit: `ee9b65e fix: appender error UX — per-column failures carry the real engine message`
- Pre-flight: `git status --porcelain | wc -l` = 0 (clean at dispatch, after coordinator
  commit through ee9b65e); workspace version 2.0.0-dev confirmed (Cargo.toml:27)
- Tag action: `git tag v2.0.0-dev && git push origin v2.0.0-dev` →
  `* [new tag] v2.0.0-dev -> v2.0.0-dev`
- Run: **39356742812** — "Build Frozen DuckDB Binaries" @ v2.0.0-dev (push)
  URL: https://github.com/seanchatmangpt/frozen-duckdb/actions/runs/39356742812

## Per-leg matrix (final)

| Job | Conclusion | Failed step |
|---|---|---|
| build-binaries-linux (arm64, ubuntu-24.04-arm) | failure | Build mega-library |
| build-binaries-linux (x86_64, ubuntu-latest) | failure | Build mega-library |
| build-binaries (x86_64, macOS) | failure | Build mega-library |
| build-binaries (arm64, macOS) | **success** | — |
| release | **skipped** | — |

Run conclusion: `failure` (verbatim `gh run view` JSON):

```
{"conclusion":"failure","jobs":[{"conclusion":"failure","name":"build-binaries-linux (arm64, ubuntu-24.04-arm)",...
{"conclusion":"success","name":"build-binaries (arm64)","status":"completed"}
{"conclusion":"skipped","name":"release","status":"completed"}
```

## Failure 1 — both Linux legs (identical): missing libcurl dev headers

Failed step: **Build mega-library**. DuckDB's httpfs extension cmake configure fails:

```
CMake Error at /usr/local/share/cmake-3.31/Modules/FindPackageHandleStandardArgs.cmake:233 (message):
  Could NOT find CURL (missing: CURL_LIBRARY CURL_INCLUDE_DIR)
Call Stack (most recent call first):
  /usr/local/share/cmake-3.31/Modules/FindPackageHandleStandardArgs.cmake:603 (_FPHSA_FAILURE_MESSAGE)
  /usr/local/share/cmake-3.31/Modules/FindCURL.cmake:203 (find_package_handle_standard_args)
  build/_deps/httpfs_extension_fc-src/CMakeLists.txt:29 (find_package)
```

Wrapper error line: `Failed to build mega-library: Failed to compile DuckDB locally:
DuckDB cmake configure failed (exit Some(1)):`

Workflow cause: the linux job's dependency step installs only
`cmake g++ libssl-dev ninja-build` — no `libcurl4-openssl-dev` (and no
`CMAKE_...`/`-DCMAKE_...` curl hint), so httpfs's `find_package(CURL)` fails.
Verified identical on both legs (job logs 118413157137 x86_64,
118413156949 arm64).

## Failure 2 — macOS x86_64 leg: httpfs link failure (OpenSSL symbols)

Failed step: **Build mega-library**. The httpfs loadable extension fails to link:

```
Undefined symbols for architecture x86_64:
  "_SSL_get_error", referenced from:
      duckdb_httplib_openssl::detail::SSLSocketStream::read(char*, unsigned long) in httpfs_httplib_client.cpp.o
  ...
  "_SSL_new", referenced from:
      duckdb_httplib_openssl::detail::setup_client_tls_session(...) in httpfs_httplib_client.cpp.o
clang++: error: linker command failed with link commands failed with exit code 1
ld: symbol(s) not found for architecture x86_64
make[2]: *** [extension/httpfs/httpfs.duckdb_extension] Error 1
make[1]: *** [extension/httpfs/CMakeFiles/httpfs_loadable_extension.dir/all] Error 2
make: *** [all] Error 2
```

(Exact line: `clang++: error: linker command failed with exit code 1
(use -v to see invocation)`.) The httpfs extension compiles
`httpfs_httplib_client.cpp.o` but the link line for
`httpfs.duckdb_extension` is missing OpenSSL (`-lssl -lcrypto`) — the arm64
macOS leg succeeded with the same workflow, so this is an x86_64-specific
link-environment defect on the macOS runner (OpenSSL keg-only path not
reaching the x86_64 link line).

## Release / assets

`gh release view v2.0.0-dev` → `release not found` (exit 1). The release job was
skipped (needs: success of builds). **verify_release_assets.sh was NOT run** — its
subject does not exist. Zero assets shipped. KD-01 mangled-exports check did not run
either.

## What was NOT done

- No retry, no re-run, no workflow edits (shared infra; defect reported, not patched).
- No release creation, no asset upload, no verify_release_assets.sh execution.

## Unblock path (for coordinator)

1. Linux legs: add `libcurl4-openssl-dev` (and consider `pkg-config`) to the
   `apt-get install` line in `.github/workflows/build-binaries.yml` (linux job).
2. macOS x86_64 leg: wire OpenSSL into the httpfs link line on that leg
   (`OPENSSL_ROOT_DIR` from Homebrew `openssl@3` on the x86_64 runner, or
   `-DDUCKDB_EXTENSION_..._LINK_FLAGS "-lssl -lcrypto"`), or disable httpfs for the
   slice if httpfs is not a release requirement — coordinator's call.
3. Fix lands on master → delete+re-tag or move tag to fixed head, re-dispatch.
