# frozen-duckdb scripts

Support tooling for the frozen-duckdb workspace: builder/fetch scripts, FFI
validation, release checks, repo gates, and ticket renderers. Each file stands
alone; STATUS headers in the file mark triage state where one exists.

## Directory map (families)

### Build and fetch (the frozen-cache pipeline)
`build_frozen_duckdb.sh`, `build_static_duckdb.sh`, `build_frozen_everything.sh`,
`create_frozen_setup.sh`, `create_precompiled_duckdb.sh`, `create_arrow_cache.sh`,
`download_duckdb_binaries.sh`, `precompile_deps.sh`, `use_prebuilt_duckdb.sh`,
`setup_optimized_build.sh`, `setup_workspace_caching.sh`,
`build_duckdb_optimized.sh`, `build_full_featured_duckdb.sh`,
`setup_flock_ollama.sh`.

The builder keeps DuckDB binaries in the central cache
`~/.frozen-duckdb/cache/v{VER}-{arch}/` and normalizes link names there;
`prebuilt/setup_env.sh` points consumers at a repo-local `prebuilt/` copy.

### FFI validation
`test_ffi_simple.sh`, `run_ffi_validation.sh`, `test_new_ffi_structure.sh`,
`test_comprehensive_flock.sh`, `build_go_smoketest.sh`, `smoke_go.go`,
`smoke_go_simple.go`, `duckdb_ffi.h`.

STATUS (TR6 2026-09-21, re-verified G5 2026-09-21): the FFI scripts are
BLOCKED as committed — they resolve `DUCKDB_LIB_DIR` through
`prebuilt/setup_env.sh`, which derives the path from `$0` and therefore
resolves to `scripts/` when sourced by `scripts/` callers (observed live:
`./scripts/test_ffi_simple.sh` exits 1 "DuckDB library not found"; the Go leg
of `run_ffi_validation.sh` fails the same way). The repair is the T6
(scripts-fix) lane; see `docs/sjira/v26.9.21/TR6.md` History.

### Release and verification
`validate_prod_build.sh`, `validate_frozen_approach.sh` (consumer-flow checks
against the crates.io-published crate — re-verify at TR8 release),
`release_checklist.sh`, `bench_gate.sh`, `run_bench_suite.sh`,
`verify_golden_traces.py`, `verify_publish_receipt.py`,
`verify_semantic_coverage.sh`.

### Generated repo gates (do not edit)
`verify-gates.sh` + `verify-evidence.ttl` are rendered by `ggen sync run`
(rule `render-verify-gates`); edit `schema/verify.ttl` and re-render.
`make gates` runs `ggen sync run` + `make genesis-check`
(`docs/GENESIS.md` is the reconciliation manifest). Each verify-gates run also records
`ver:subjectHead` / `ver:subjectTree` (`git rev-parse HEAD` / `HEAD^{tree}`) at gate time,
pinning the evidence to the exact verified subject (commit b9fa77c).

`check_sjira_standing.sh` — standing gate (FDDB-26922-03, hand-written, not
ggen-rendered): verifies each ticket's frontmatter `aps:standing` equals its
final History standing (first run: checked=45 mismatches=0, commit ed27449).

### Hygiene and gap detection
`rg_sweep.sh` — informational pattern sweep (stub/fake/TODO tokens, telemetry
probes, the real frozen-duckdb-sys FFI surface). Never a gate; see
`README_gap_detection.md` for what happened to the former gate scripts.
`check_workspace_deps.sh`, `scan_cli.sh`, `scan_cli_help.sh`, `test_doctests.sh`,
`open_gaps.sh`, `spec_sync_check.sh`, `generate-changelog.sh`,
`ffi_constant_return_check.py`.

### Ticket renderers
`gen_sjira_tickets*.sh` — edit the SPECS table, re-run the script; the
`docs/sjira/` tickets are rendered consequences (票 law: History appends are
the sanctioned exception). Each rendered ticket carries an `aps:standing`
frontmatter column taken from the script's per-id standing map (terminal
History standings; new tickets default OPEN).

## Removed tooling record (de-fakery era)

The kcura-era "de-fakery" tooling was removed after falsification showed it
dead against this repo. Do not resurrect it from old revisions without a pack
admission and bash-3.2-clean implementation:

- TR6 (commit 1a6fd72): `ci_gate.sh`, `ci_gates.sh`, `fake_guard.sh`,
  `smoke_all.sh`, `smoke.py`, `smoke_py.py`, `smoke_node.mjs`,
  `demo_ffi_validation.sh` — kcura-symbol gates, unconditional exits, parse
  failures; evidence in `docs/sjira/v26.9.21/TR6.md` History.
- G5 (commits 075d1d4, dc70ea6): `scan_fakes.sh`, `scan_fakes_core_team.sh`,
  `lib/{config,intelligent_cache,logging,self_healing}.sh` (die under stock
  macOS bash 3.2: `declare -A` at lib/config.sh:39; live exit 2),
  `kcura-config{,.example}.yaml`, `redteam_probe.rs` + `Cargo.toml`
  (manifest has no targets — exit 101; probes were hardcoded mocks),
  `docs_check.sh` (failed `bash -n`; required a kcura MkDocs tree that does
  not exist). Evidence in `docs/sjira/v26.9.21/G5.md` History.

If a static fake-pattern gate is wanted again, manufacture it as a pack fact
with a gate (帳/器 law) — not as loose scripts in this directory.
