# USER-FINDINGS-U7 — CI/CD integrator lane: frozen-duckdb as a consumer would meet it

Status: PARTIAL_ALIVE (probe-based; no consumer build was executed this session —
evidence is real HTTP probes + source reads, cited inline)
Lane: U7 (no git commands run; proposal file only)
Repo: /Users/sac/frozen-duckdb
Date: 2026-10-19

## Method

Written strictly from the outside: no `.github/` edits, no builder source changes.
Evidence = live HTTP probes (2026-10-19) + reads of
`/Users/sac/frozen-duckdb/crates/frozen-duckdb-builder/src/lib.rs` (all line cites
below are that file). The proposed workflow lives at `/tmp/u7/consumer-ci.yml`.

## Finding 1 (killer): every CI run recompiles DuckDB from source

Verified probes, 2026-10-19:

```
$ curl -s -o /dev/null -w "%{http_code}\n" \
    https://github.com/seanchatmangpt/frozen-duckdb/releases/download/v1.5.5/...  [asset URL]
404
$ curl -s -A "u7-lane" -w "\nHTTP %{http_code}\n" https://index.crates.io/fr/oz/frozen-crates... [frozen-duckdb]
HTTP 200  →  {"vers":"0.1.0","pubtime":"2025-10-07", ...deps: duckdb ^1.4.0 ...}
```

- No GitHub release assets exist for the engine dylib/so/dll: the asset URL 404s
  (v1.5.5 tag cut, release missing — matches the README's known open item).
- Worse than the task brief assumed: the builder's download URL is keyed to
  `VERSION = "2.0.0"` (lib.rs:17, URL format at lib.rs:1154-1158), so even if
  the 1.5.5 assets were published, the dev builder would still 404 — it asks for
  `releases/download/v2.0.0/libduckdb_{arch}.{dylib,so}`. The asset set must
  be attached to a **v2.0.0 release**, or VERSION must move to 1.5.5 before
  assets are attached to v1.5.5. Publishing 1.5.5 assets alone does NOT unblock
  the download path for the current builder.
- Consumers can't use a path dep cross-repo and can't use a crates.io dep: the
  name `frozen-duckdb` on crates.io is an UNRELATED 0.1.0 crate (2025-10-07,
  deps `duckdb ^1.4.0`, features `{}`, cksum `3702950cb1fd…`; sparse index
  `fr/crates…`, HTTP 200). `frozen-duckdb-builder` and `frozen-duckdb-sys`
  return NoSuchKey on the sparse index — never published. Consequences:
  (a) today's only install route is the **git dependency**, and (b) when
  `publish-crates.yml` eventually runs, the `frozen-duckdb` publish will fail
  with "name already in use" unless the crate is renamed (e.g. `frozen-duckdb2`
  is worse; a scoped name like `seanchatman-frozen-duckdb` or reclaiming the
  name is required — reclaim requires the squatter's cooperation or crates.io
  support; rename is the realistic route).
- Net effect today: the local-compile fallback is the operative path on every
  OS, every run. Measured class: 30-90 min per OS per run (DuckDB v1.5.x
  Release + extensions per the TR2 rehearsal in comments at lib.rs:1258-1263),
  and CI cache cannot persist `~/.frozen-duckdb/` across runs without
  `actions/cache`. Ubuntu/macos compile ≈30-60 min; windows expected worse.

## Finding 2: windows leg — extension law vs reality

`library_extension_for` (lib.rs:50-57) answers "dll" for windows, so
`get_binary_path` and `release_asset_name_for` are **internally consistent** —
the builder does seek `libduckdb_{arch}.dll` on windows, both for download and
cache naming, and `find_built_library` includes `.dll` locations
(lib.rs:1344-1357). The extension law itself is consistent; what is missing is
everything downstream of it.

- No `.dll` release asset exists or is planned (lib.rs:46-49 doc comment).
- Local-compile on windows: **UNKNOWN, expected to fail**. Concrete spawn points
  that have never run on a windows runner:
  - `Command::new("make").args(["-j4"])` (lib.rs:1294) — `make` is not present
    on windows-latest; needs choco make + MSVC env (vcvars) for the cmake
    configure to select the right generator. `Command::new("cmake")`
    (lib.rs:1264) is spawned without a vcvars shell,
    so even with make installed the configure step likely selects a wrong
    generator.
  - `git clone duckdb/duckdb` + `git checkout SOURCE_PIN` would work on
    windows (git present), the clone itself is fine.
  - `check_local_compile_target` (lib.rs:673-684) does not block windows
    (only cross-arch on macOS is refused).
  - Marked UNKNOWN honestly: **no Windows compile has ever been executed in
    this repo's history** — this is a code-read verdict, not a run receipt.
  - The slice checker does have a windows branch (PE magic `MZ`,
    lib.rs:742-747, tested at lib.rs:1652 `rel5_wrong_magic_fails_on_windows_branch`)
    — verification logic is windows-ready; the build toolchain path is not.
  - Not analyzed this session (out of U7 scope): DLL export-symbol/`__declspec`
    concerns on the FFI surface; that is sys-crate scope, not builder scope.

## Finding 3: the unblock proposal

Two-part fix, in dependency order:

### 3a. Consumer-side mitigation: actions/cache on ~/.frozen-duckdb

Recipe in `/tmp/u7/consumer-ci.yml`: key on the builder's pinned
DuckDB `SOURCE_PIN`
(`d54cbbe3df17e8e5ce68312f483b4155836112aa`,
lib.rs:20), duplicated into a hashable repo file (`.frozen-duckdb-pin`) since
consumers cannot `hashFiles()` a Rust const. Cache-hit invalidates exactly when
the engine bytes must change. With a warm cache, CI drops 30-90 min → seconds.

### 3b. Project-side fix: release assets + trigger

- GH2's OpenSSL wiring landed in `.github/workflows/build-binaries.yml:58-85`
  (brew openssl@3, arch-correct x86_64 keg via Rosetta brew probe, exports
  `OPENSSL_ROOT_DIR` passed through to cmake). That unblocks the macOS asset
  build; the workflow needs a run of build-binaries via **workflow_dispatch**
  (dispatch is the guarded path per BINARIES-PIPELINE-AUDIT.md link 4 — a
  v2.0. governance note: tag pushes are UNGUARDED, so use dispatch, not a tag
  push). Output must land on a release tagged **v2.0.0** to match the
  builder's download URL (`VERSION = "2.0.0"`, lib.rs:17).
- After dispatch, probe one asset URL from each OS leg to confirm the download
  path turns from 404 to 200; re-probe after every tag cut until CI download
  path is ALIVE.
- Open scope item for the release lane: windows `.dll` asset is documented as
  not planned (lib.rs:46-49), so windows consumers stay on the compile
  fallback + cache even after the macOS/Linux asset dispatch; windows remains
  UNKNOWN until someone runs `compile_duckdb_locally` on a windows runner.

## Verdict per OS (as of 2026-10-19)

| OS | download | compile fallback | consumer CI today |
|---|---|---|---|
| ubuntu-latest | 404 | works (cmake+make preinstalled) | 30-90 min/run; cache recipe mitigates |
| macos-latest | 404 asset | works (verified class: TR2 rehearsal) | 30-90 min/run; cache recipe mitigates |
| windows-latest | 404 (dll asset absent) | UNKNOWN (make/vcvars never run) | BLOCKED-or-slow; cache recipe untested on windows |

Standing: PARTIAL_ALIVE. Falsifier for the mitigation: warm-cache CI run where
the engine stage takes seconds and `RUST_LOG=frozen_duckdb_builder=info` shows
cache hit, not compile. Falsifier for the fix: post-dispatch asset URL returns
200 and a cold consumer CI run downloads in < 1 min on ubuntu/macos.

## CORRECTION (coordinator, 2026-10-20): finding 2 refuted by crates.io API

Live API (curl, User-Agent set): `frozen-duckdb` 0.1.0 sole owner = `seanchatmangpt`
(exact match with gh auth), repo field = this repo. The 0.1.0 CONTENT is a legacy
unrelated implementation (deps duckdb ^1.4.0), but the name is OURS. PB14's CLEAR
verdict stands. No rename required — 2.0.0 publishes as an owner version bump.
Publish-crates.yml name-collision risk: refuted.
