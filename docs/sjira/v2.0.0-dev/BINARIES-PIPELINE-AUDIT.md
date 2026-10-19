# BINARIES-PIPELINE-AUDIT — v2.0.0-dev binaries path (lane GH6, read-only audit)

Audit subject: `.github/workflows/build-binaries.yml` @ master (79e803e), builder
`crates/frozen-duckdb-builder/src/lib.rs` (VERSION=2.0.0-dev, SOURCE_PIN=b6013f7),
release gate `scripts/verify_release_assets.sh`, publish gate `gates/dry-run-publish/`.

Method: read-only file inspection; no commands mutated state, no git commands run.
Statuses READY / FIXED-BY-GH2 / BLOCKED refer to the v2.0.0-dev campaign lane plan
(GH2 = the workflow-hardening lane).

## Pipeline links

| # | Link | Status | Evidence (file:line) |
|---|---|---|---|
| 1 | Workflow job graph matches builder fallback chain (cache -> prebuilt -> download -> compile) | READY | Workflow "Build mega-library" step (build-binaries.yml:75-113) compiles and runs a scratch crate calling `ensure_binary()` only. Builder order is exactly: engine-marker-gated cache check (lib.rs:104-131) -> `check_prebuilt_binary` (lib.rs:135; fn at 235-254, looks for `./prebuilt/libduckdb_<arch>.dylib` relative to cwd — absent on a bare runner, so MISS) -> `download_from_github_release` (lib.rs:158; fn at 575-616) -> `compile_duckdb_locally` (lib.rs:170; fn at 619+). Confirmed the chicken-and-egg closes correctly: no release exists for 2.0.0-dev yet ("ALL downloads 404 today", comment lib.rs:580-581), so on a cold runner **the local-compile fallback IS the pipeline**: clone duckdb.git -> checkout SOURCE_PIN b6013f7 (lib.rs:652-666) -> cmake configure with `-DBUILD_EXTENSIONS=parquet;json;icu;httpfs;tpch;tpcds;fts;inet;sqlsmith -DBUILD_JEMALLOC=ON -DBUILD_AUTOLOAD=ON` (lib.rs:686-695) -> `make -j4` -> `find_built_library` -> `lipo` slice verification (lib.rs:429-466) -> atomic copy into the versioned cache. |
| 2 | x86_64 leg cross-compiles on the arm64 macos-latest host | READY | Workflow exports `CMAKE_OSX_ARCHITECTURES=<matrix.arch>` per leg (build-binaries.yml:57) and `ARCH` (yml:48). `detect_architecture` (lib.rs:323-330) treats `CMAKE_OSX_ARCHITECTURES`/`ARCH` as authority over `uname -m` via `resolve_target_architecture`, so the x86_64 leg caches under `v2.0.0-dev-x86_64/libduckdb_x86_64.dylib` (correct placement — the pre-TR7 mislabel bug). `cmake_osx_architectures_for` (lib.rs:387-392) passes an operator-set value verbatim to cmake (lib.rs:696-701), so clang actually emits the x86_64 slice. `check_local_compile_target` (lib.rs:397-408) allows cross-slice on macOS (refuses it off-macOS); `verify_library_slice` via `lipo -archs` (lib.rs:429-466, TR7 hardening) fails closed on label/slice mismatch at cache-hit, prebuilt, download, and compile paths. Unit-tested at lib.rs:1006-1030. |
| 3a | Workflow upload name matches release attach name | READY | Upload: `actions/upload-artifact@v4`, name `libduckdb_<arch>`, path `stage/libduckdb_<arch>.dylib` (build-binaries.yml:131-135); the find step (yml:115-129) hard-fails unless `*/v*/libduckdb_<arch>.dylib` exists in `$HOME/.frozen-duckdb/cache`, so the staged name can only be the release-asset name. Release: `softprops/action-gh-release@v2` files `libduckdb_x86_64/libduckdb_x86_64.dylib` + `libduckdb_arm64/libduckdb_arm64.dylib` (yml:169-171) — download-artifact@v4 restores under the artifact name as a directory, so paths line up. |
| 3b | Release assets match what verify_release_assets.sh expects | READY | Script expects exactly `libduckdb_arm64.dylib` + `libduckdb_x86_64.dylib` (verify_release_assets.sh:18), count==2 with extra-asset breach (lines 44-57), real `gh release download`, `lipo -archs` slice assert (94-98), `strip -x` size ceiling 55 MiB (100-109), ENGINE probe (ctypes native / strings foreign, 65-83), sha256 (119-121); loud exit 2 when the release does not exist yet (31-34). Builder download path normalizes mode to 0755 (lib.rs:605-612) and prebuilt copy does too (lib.rs:270-276); the compile path does NOT chmod (atomic_copy, lib.rs:502-514, preserves make's 0755 — non-issue in practice). Note: upload-artifact v4 zips and does not preserve the exec bit; the builder's chmod-on-download restores it for consumers, and verify_release_assets.sh never asserts permissions, so the name/size/permission chain closes. |
| 4 | Tag-trigger vs dispatch-trigger behavior | FIXED-BY-GH2 (governance) | The "Guard version input" step runs `if: github.event_name == 'workflow_dispatch'` only (build-binaries.yml:59-73); the release job gates on `startsWith(github.ref, 'refs/tags/')` (yml:141). **Governance flag: a `v2.0.0-dev` tag push would build AND release completely unguarded** — the version guard never runs on tag push, so a mistagged push releases whatever the builder pin happens to be, attaching it to the pushed tag. The dispatch path is guarded (input must equal `v2.0.0-dev`, else exit 1) and dispatches cannot create a release (ref is a branch). Recommendation for GH2: replicate the guard for tag pushes (compare `github.ref_name` against the builder pin) or make the release job depend on an explicit version-attestation step. |
| 5 | Secrets/permissions for the release job | READY (with 1 note) | Release job uses only `secrets.GITHUB_TOKEN` (yml:172-173); `softprops/action-gh-release@v2` needs `contents: write` to create the release + upload assets. The workflow declares no `permissions:` block, so it inherits the repo default — if the repo default is read-only (GitHub's recommended setting), **the release job fails with 403 on tag push**. GH2 should add an explicit `permissions: { contents: write }` scoped to the release job (least privilege). `actions/upload-artifact`/`download-artifact` need no extra perms. Nothing else (no PAT, no secrets.CI) is referenced. |
| 6 | Publish-order gate exists downstream | READY | `gates/dry-run-publish/dry-run-publish-domain.ttl:67` encodes "CI release assets green on the tag (build-binaries.yml, exactly libduckdb_arm64.dylib + libduckdb_x86_64.dylib); HOLD the publish while assets are missing" — the SHACL gate is the publication brake; verify_release_assets.sh is its runtime witness. Consistent names across all three layers. |

## Summary

- READY: links 1, 2, 3a, 3b, 6.
- FIXED-BY-GH2: link 4 (tag-push guard gap — governance note, plus link 5's explicit
  `permissions: contents: write` for the release job).
- BLOCKED: none in the audited files.
- Cross-cutting residual: link 1 depends on network egress for the duckdb.git clone and
  the 150-minute job timeout (yml:20) being sufficient for `make -j4` on a 3-core GH
  macOS runner (documented 30-90 min at yml:17-19); both are environmental, not defects.

## Receipt

Audit basis (real inspection commands, this session):

- `ls .github/workflows/ scripts/` — exit 0.
- `grep -n "ensure_binary\|cmake_osx_architectures\|download\|compile" crates/frozen-duckdb-builder/src/lib.rs` — exit 0, 40+ hits incl. lib.rs:93 `pub fn ensure_binary()`.
- `find` for `verify_release_assets*` — exit 0, located `scripts/verify_release_assets.sh` (133 lines, read in full).
- Full reads: build-binaries.yml (174 lines), lib.rs:88-197, 380-508, 548-800.
