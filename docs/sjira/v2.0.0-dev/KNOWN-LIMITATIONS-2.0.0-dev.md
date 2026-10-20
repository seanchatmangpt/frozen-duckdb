# Known Limitations — 2.0.0-dev

Draft GitHub-release "Known limitations" section, v2.0.0-dev prerelease.
Facts only; every item carries its receipt/KB path and close condition.
Sources re-read on disk 2026-10-19.

## 1. Prebuilt assets are macOS-only (osx_arm64)

Prebuilt engine assets ship for macOS arm64 only. Linux users: a `.so`
candidate is prepared but not dispatched (lane REL6, pending). Windows
users: build falls back automatically to compiling pinned DuckDB `v1.5.5`
source locally (vendored headers in `crates/frozen-duckdb-builder/
vendored-headers/`; no action needed, build just takes longer).
- Receipt: `docs/sjira/v2.0.0-dev/BINARIES-PIPELINE-AUDIT.md`
- Close: REL6 Linux `.so` dispatched and receipts on the release assets.

## 2. `flock` / `tpch` community extensions absent at the dev pin

The DuckDB extension CDN has no `flock`/`tpch` build for our dev pin
`b6013f7aa2` (osx_arm64) — HTTP 404. The tests that need them are
ignore-gated, not failing.
- Receipt: `docs/sjira/v2.0.0-dev/receipts/ADV12-extension-watch.md`
  (real run: `HTTP 404 still absent`, `STILL_ABSENT`, EXIT=1)
- Monitor: `bash scripts/extension_watch.sh` — exit 0 + `AVAILABLE`
  prints the exact un-ignore line.
- Close: watcher prints `AVAILABLE`; run the printed sed + test line.

## 3. flock `INSTALL` fails at the dev pin

`INSTALL flock` fails at this pin — unsupported until upstream publishes a
2.0-compatible build (2.0 AST changes; `parse_function_t` API migration).
Tests are ignore-gated, not failing.
- Receipt: `docs/sjira/v2.0.0-dev/receipts/ADV12-extension-watch.md`
- Close: upstream 2.0-compatible flock build; re-run
  `crates/frozen-duckdb/tests/flock_tests.rs` with `-- --ignored`.

## 4. WASM artifact exceeds the 25 MiB raw-size SLA

`target/wasm/duckdb.wasm` is 41,099,303 bytes (39.20 MiB) vs the 25 MiB
SLA. The gzip -9 wire size (9.46 MiB) passes its 10 MiB SLA. The Code
section is 92.4% of bytes (36.2 MiB), so binaryen passes recover only
0.20% — the trim must come from the compiled feature surface, not
post-build optimization.
- Receipt: `docs/sjira/v2.0.0-dev/receipts/ADV13-wasm-size.md` (exit 1,
  non-vacuous FAIL)
- Close: artifact <=25 MiB via compile-surface trim (feature/link flags,
  extension selection, `-Oz`, LTO, or build split); re-run
  `scripts/wasm_size_audit.sh` exit 0.

## 5. KD-01 — mangled symbols in the native dylib export table

The native dylib exports 21 mangled (non-C-ABI) symbols; upstream
DuckDB's visibility macros do not fence the export table at our pin.
This gates the "C-ABI-only surface" claim only — not the engine build.
Ships per `PUBLISH-DECISION.md` ("Blocks release: No").
- KB: `docs/sjira/v2.0.0-dev/KD-CLOSURE-PLAN.md` (KD-01 row);
  `PUBLISH-DECISION.md`
- Close: upstream CMake export fence (version-script via
  `-DDUCKDB_EXTRA_LINK_FLAGS`) or a local wrapper-ltcg link step; re-run
  the export-fence court (`receipts/ADV3-export-fence.md`).

## 6. crates.io publish on hold for 2.0.0-dev

This is a prerelease; publishing a dev version to crates.io is
permanent (immutable index entry) and is held for the final release.
- KB: `DEFINITION-OF-DONE.md` DOD-18 (crates.io hold decision,
  PROPOSED); `PUBLISH-DECISION.md` (Option B, HOLD FOR FINAL).
- Close: owner release-op decision at 2.0.0-final per DOD-18.

## 7. tpch/flock test gates in the workspace suite

Workspace test runs exclude the extension-dependent tests via
`#[ignore]` (e.g. `crates/frozen-duckdb/tests/flock_tests.rs`, ignore
reason names the pin and the 404). These are the only ignored tests;
they re-enable mechanically when the watcher fires.
- Receipt: `receipts/ADV12-extension-watch.md`; ignore lines in
  `crates/frozen-duckdb/tests/flock_tests.rs` (lines 27, 533)
- Close: same as item 2.

## 8. Latency gate is median-of-3 (load-sensitive)

The latency gate uses median-of-3 over the rev2 three-query workload;
single-run p95 is load-sensitive (one run's round-2 p95 alone would
have flipped the verdict — `receipts/ADV5-median3.md`). Run the gate on
an idle host.
- Receipt: `docs/sjira/v2.0.0-dev/receipts/ADV5-median3.md`
- Close: none required (rule, not a defect); revisit if flake observed.

## See Also
`docs/sjira/v2.0.0-dev/RELEASE-DRAFT-v2.0.0-dev.md` ·
`docs/sjira/v2.0.0-dev/KD-CLOSURE-PLAN.md` ·
`docs/sjira/v2.0.0-dev/PUBLISH-DECISION.md` ·
`KNOWN_DIVERGENCES.md`
