# v2.0.4 release receipt — engine re-pin to upstream v2.0.0 LTS

| field | value |
|---|---|
| repo | `/Users/sac/frozen-duckdb` (canonical checkout) |
| work order | Upgrade bundled DuckDB to the upstream v2.0.0 LTS release ("Cinnamon Teal", *Spatula cyanoptera*, released 2026-10-21) and publish crates 2.0.4 |
| upstream pin | `v2.0.0` tag = `d54cbbe3df17e8e5ce68312f483b4155836112aa` (lightweight tag, verified via `git ls-remote`) |
| crate version | 2.0.4 (workspace + sibling pins + locks + docs) |
| operator law | dev-snapshot naming referenced nowhere — `2.0.0-dev`, `v2.0.0-dev88595`, `b6013f7a…` purged tree-wide |
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
- **Sweep census**: `git ls-files | xargs grep -l '2\.0\.0-dev\|b6013f7a'` → zero
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

## Gates (appended at close-out)

- workspace build: PENDING
- cargo test -p frozen-duckdb: PENDING
- test-validation: PENDING
- clippy -D warnings: PENDING
- fmt --check: PENDING
- make verify-gates: PENDING
- make verify: PENDING
- release_checklist.sh: PENDING
- publish_gate.sh (incl. cargo publish --workspace --dry-run): PENDING
- publish chain builder→sys→main + close-out: PENDING
