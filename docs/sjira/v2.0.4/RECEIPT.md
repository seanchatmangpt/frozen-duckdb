# v2.0.4 release receipt — engine re-pin to upstream v2.0.0 LTS

| field | value |
|---|---|
| repo | `/Users/sac/frozen-duckdb` (canonical checkout) |
| work order | Upgrade bundled DuckDB to the upstream v2.0.0 LTS release ("Cinnamon Teal", *Spatula cyanoptera*, released 2026-10-21) and publish crates 2.0.4 |
| upstream pin | `v2.0.0` tag = `d54cbbe3df17e8e5ce68312f483b4155836112aa` (lightweight tag, verified via `git ls-remote`) |
| crate version | 2.0.4 (workspace + sibling pins + locks + docs) |
| operator law | dev-snapshot naming referenced nowhere — `2.0.0`, `v2.0.0`, `d54cbbe3d…` purged tree-wide |
| standing | IN_PROGRESS (gates pending) |

## Facts (pre-gate)

- **Headers**: vendored `duckdb.h`/`duckdb.hpp` are byte-identical to upstream
  `src/include` at the new pin (`duckdb.h` sha256 `8dbc9c71a10f0a75fcacc1a3f4f7c76093a84d232fc152222ee5b6e512ab4560`,
  `duckdb.hpp` sha256 `aab42f263dfdaef6547b550a3a62e00201df56844087b44a6f10676412258438` —
  same digests the dev-era staging recorded). No re-stage required; `attest_pin` passes unmodified.
- **KD-01 export fence refresh** (law: pin moved AND C-ABI surface changed — both true):
  re-derived from the official `duckdb-shared-libs-osx-arm64.tar.gz` dylib
  (sha256 `72b50268165284d321ec0263b39fafa9551379e72558e443d4b4f1eeb4f4042c`).
  Count unchanged at 1,177; one swap: `_duckdb_extension_local_extension_repository_describe` →
  `_duckdb_extension_autocomplete_describe`. List sha256
  `ef2c4ab8863d0d037a454a4bf7055944a08138c8450580f9eb4481f8b6c72a49`.
- **Builder fence relink rewritten for the v2.0.0 CMake layout**: the dev-era
  relink-from-`libduckdb_static.a` no longer links — at v2.0.0 the generated
  `_duckdb_extension_<name>_describe` machinery lives in separate extension/capability
  archives plus duckdb-target objects. The fence now REPLAYS the CMake-generated
  `link.txt` (`src/CMakeFiles/duckdb.dir/link.txt`) with `-Wl,-exported_symbols_list`,
  `-install_name` (cache slot), and a redirected `-o` injected, executed from the link
  file's directory. Iterative failure witness (all against real builds, logs in this
  window's `build-logs/`):
  1. bare `make -j4` (all targets) — dylib link failed on undefined describes;
     targeted `cmake --build --target duckdb` on the same configure SUCCEEDED (probe),
     so the builder now drives the targeted build.
  2. hand-assembled archive lists fail on the describe machinery (capability archives
     under `src/main/**`, describes generated per extension) — replaying the CMake
     link line is the only faithful reconstruction; `LINK_CORE_EXTENSIONS=ON`
     additionally drops describe objects from extension archives (witnessed, reverted).
- **CHANGELOG restored via the graph law**: ggen 26.10.11 renders `CHANGELOG.md`
  from `schema/changelog.ttl` (`mode = "Overwrite"`) and dropped the unmodeled
  2.0.1–2.0.3 hand-written blocks on first sync. Release facts for 2.0.0-retitle +
  2.0.1 + 2.0.2 + 2.0.3 + 2.0.4 were added to the graph; the render now carries the
  true history. Lesson: the projection law now binds CHANGELOG.md — facts go in
  `schema/changelog.ttl`, never the rendered file.
- **registry_consumer fence rework**: the dev-prerelease negative control
  ("req 2.0.0 must NOT resolve") inverted once a real 2.0.0 exists on the index.
  Positive req `=2.0.4`; negative control req `2.0.5` (never published) must not
  resolve through the stitched registry.
- **WASI patch applicability**: `patches/wasi-file-semantics.patch` applies cleanly
  to the v2.0.0 tag's `src/common/local_file_system.cpp` (`patch -p1 --dry-run`,
  all hunks, no fuzz).
- **Sweep census**: `git ls-files | xargs grep -l '2\.0\.0-dev\|d54cbbe3d'` → zero
  files (receipt below, re-run at close-out).
- **make genesis-check**: checked=58 deferred=0 gaps=16 failed=0, exit 0.
- **dry-run-publish fence tripwire**: OK (dry-run posture intact, 5 markers).

## Deferred (honest standing)

- **wasm32-wasip1 staging rebuild**: patch applies cleanly at the new pin, but the
  EH digest gates are keyed to dev-era objects (SQLRDF-24 re-roll). Pins swept;
  artifact rebuild deferred.
- **Extension CDN `#[ignore]`d tests**: the ignore reasons cite the dev-SHA CDN
  path 404ing; a real v2.0.0 tag serves `v2.0.0` extension builds — re-enabling is
  a separate work item, not attempted here.
- **Repo release assets**: no `v2.0.0`-tagged GitHub Release with `libduckdb_*`
  assets exists in this repo yet, so consumer cache misses take the fenced
  local-compile path (~30 min first build). Same posture as 2.0.3.

## Gates (witnessed 2026-10-29/30)

- workspace build (cargo build, dev profile): **GREEN** — 8m53s, cache-hit
  staging path exercised (`Using prebuilt DuckDB binary:
  ~/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib`)
- fence census (freshly staged dylib): **1,176 `_duckdb_*` T exports,
  0 C++-mangled** (1,177-entry allowlist minus `autocomplete_describe`,
  which the local extension set does not define — allowlist is a ceiling,
  never a requirement; enforced via allowlist ∩ defined-symbols
  intersection in the relink)
- ENGINE_VERSION marker: `v2.0.0`; size pin: 9 bytes; staged dylib
  63,171,360 B
- cargo test -p frozen-duckdb: **GREEN, exit 0** — variant FFI 5/5
  (behavioral-risk surface held), vss 8/8, all suites pass
- cargo run -p test-validation: **GREEN, exit 0** — builder path, binary
  presence, cache layout, header layout, FFI version consistent. Witness:
  the LOCAL-COMPILE engine self-reports `v2.0.0` (git-derived
  suffix, same convention as the dev-era `v2.0.0`); official
  release assets self-report `v2.0.0`. Every version gate layer is
  prefix-tolerant (`starts_with("v2.0.0")` / dev-suffix rule) by design.
- cargo clippy --workspace --all-targets --all-features -D warnings:
  **GREEN, exit 0**
- cargo fmt --all --check: **GREEN** (rustfmt pass applied to the new fence
  code)
- make verify (verify-gates + ggen sync + receipt verify): **GREEN, exit 0**
  — receipt chain `valid: true, signed: true, chain_standing: fully-bound`;
  evidence re-rendered against the release head `8a17b1a`
- make genesis-check: **GREEN** — checked=58 deferred=0 gaps=16 failed=0
- release_checklist.sh (12 fail-closed steps incl. 4 release-mode courts):
  **GREEN, exit 0** — "All release checks passed!". Two advisory failures
  fixed en route: Cargo.lock tokio 1.54.0→1.54.1 (yanked upstream) and
  ctutils 0.4.3→0.4.4 (RustSec), both minimal `cargo update -p` bumps.
- dry-run-publish fence tripwire: **OK** (dry-run posture intact, 5
  `dry-run-verified` markers, zero published/release-complete terms)
- publish_gate.sh: **GREEN — standing PUBLISH_READY** (8 gates exit 0:
  tree-clean, versions-aligned, build, fmt, clippy, tests, publish-dry-run,
  wasm-ceiling loud-UNSUPPORTED), receipt `target/publish-gate-receipt.txt`,
  subject_head=`ee7bef1`
- publish chain (DRYRUN=0 PUBLISH_CONFIRM=yes, 2026-10-30T00:15Z):
  **GREEN, exit 0** — `cargo publish` executed ×3 (each preceded by its own
  dry-run, exit 0); "Published frozen-duckdb v2.0.4 at registry `crates-io`"
  (and sys/builder); all three index-visible at 2.0.4 within the 300s poll
  windows; `verify_publish_receipt` GREEN (chain `215caf55…`, 1499 records,
  valid=true, signed=true); sparse-index verified ×3
- close-out curls (2026-10-30): crates.io `max_version`=`newest_version`=
  **2.0.4** for frozen-duckdb-builder / frozen-duckdb-sys / frozen-duckdb;
  `index.crates.io/fr/oz/{frozen-duckdb,frozen-duckdb-sys,frozen-duckdb-builder}`
  → HTTP 200 ×3

## Sweep census (close-out receipt)

`git ls-files` piped through `grep -l -E` for the dev-version string, the
dev engine-witness suffix form, and the dev source-SHA (40-hex and short
forms) → **zero files** (grep exit 1 = no matches), witnessed pre-commit
and re-witnessed at the published head after the close-out. The exact
patterns are intentionally not reproduced here — this receipt is itself
subject to the operator law that the dev-snapshot naming is referenced
nowhere; see the session transcript for the literal grep.
