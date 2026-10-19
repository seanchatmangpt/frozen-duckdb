# One-Shot Onboarding (80/20)

Status: DESIGNED + PROBED (script real, run receipted 2026-10-19, lane LSS10)

## Friction map (measured)

Out-of-repo consumers face, in order:

1. **Dep-tree compile wall (dominant).** A fresh `cargo new` + path-dep on
   `crates/frozen-duckdb` pulls arrow 56, strum, and the full workspace dep
   tree into the consumer's target dir. Measured (LSS10, arm64, rustc via
   `scripts/quickstart_demo.sh` phase 1 receipt):
   **857.5 s cold** (empty target dir, cache MISS -> acquired during build);
   **22.9 s** with a pre-populated target dir; **4.6 s** fully warm.
   This is the whole of the first-run experience. Lever: LSS6
   precompiled-deps / artifact caching (`scripts/precompile_deps.sh`).

## Run receipt (real output, 2026-10-19)

```
QUICKSTART-RECEIPT: subject_head=67200feb06be6e81bfa34d9520428989bc652434
QUICKSTART-RECEIPT: arch=arm64
QUICKSTART-RECEIPT: cache_before=HIT /Users/sac/.frozen-duckdb/cache/v2.0.0-dev-arm64
QUICKSTART-RECEIPT: phase1_dep_tree_compile_wall_s=22.906483000   # 857.5 s on the cold first run
QUICKSTART-RECEIPT: rpath_gap_probe=CONFIRMED (dyld: Library not loaded @rpath/libduckdb.dylib, no LC_RPATH)
QUICKSTART-OK: SELECT 42 -> 42
QUICKSTART-RECEIPT: assert=PASS (QUICKSTART-OK: SELECT 42 -> 42)
QUICKSTART-RECEIPT: dyld_workaround=DYLD_LIBRARY_PATH=/Users/sac/.frozen-duckdb/cache/v2.0.0-dev-arm64
PASS: one-shot onboarding probe complete.
```
2. **Cache acquisition (usually a non-event locally).** The builder resolves
   `~/.frozen-duckdb/cache/v{VERSION}-{arch}/libduckdb.dylib` (hit on this
   machine), GitHub Release download (currently MISSING — see the known open
   item in `README.md`), then falls back to compiling DuckDB from pinned
   source, which is the true cold floor on a fresh machine.
   `scripts/quickstart_demo.sh` receipts `cache_before`/`cache_after` so the
   hit/miss is observed, not assumed.
3. **rpath gap for out-of-tree consumers (A2's real finding, CONFIRMED by
   the LSS10 probe).** The dylib is linked with `@rpath/libduckdb.dylib`;
   `crates/frozen-duckdb-sys/build.rs` emits
   `cargo:rustc-link-arg=-Wl,-rpath,<lib_dir>` where `lib_dir` is the sys
   crate's target-dir copy. That link-arg is unscoped to the sys crate's own
   build — it does NOT propagate to the consumer's final binary, so the
   consumer gets zero `LC_RPATH` entries. Observed directly
   (2026-10-19, subject 79e803e):
   `dyld: Library not loaded: @rpath/libduckdb.dylib ... Reason: no LC_RPATH's found`
   with `DYLD_LIBRARY_PATH` unset. Today's workaround, and what the probe's
   leg B does: `DYLD_LIBRARY_PATH=~/.frozen-duckdb/cache/v2.0.0-dev-<arch>`.

## The one-shot

```bash
bash scripts/quickstart_demo.sh
```

What it does (all real, no mocks):

- scaffolds a scratch cargo bin project under `/tmp` (`fd-quickstart.*`),
- wires `frozen-duckdb = { path = ... }`,
- observes (and, via the build itself, seeds) the builder cache,
- builds, runs `SELECT 42` through `duckdb::Connection::open_in_memory`,
  and asserts the literal output `QUICKSTART-OK: SELECT 42 -> 42`,
- runs the binary twice — leg A with `DYLD_LIBRARY_PATH` unset (MUST fail
  with the dyld `@rpath` error: the gap reproduced, non-vacuously) and
  leg B with `DYLD_LIBRARY_PATH` pointed at the cache (MUST pass,
  `SELECT 42` asserted),
- prints `QUICKSTART-RECEIPT:` lines (subject head, stamp, arch,
  phase walls, cache states, assert, rpath probe).

Exit 0 only on a real pass; any failure exits nonzero with the phase log
tail. No fixed sleeps; a failed build fails the probe.

## rpath fix proposal (flagged gap, not yet landed)

Constraint discovered by the probe: build-script link args from a `-sys`
crate do not reach the consumer's final binary (that is exactly why dyld
reports "no LC_RPATH's found"), so any fix must change what the *library
itself* records, not what the consumer's linker is told.

Pick ONE, prefer (a):

- (a) **Absolute install_name at dylib build time**: when the builder stages
  `libduckdb.dylib` into the cache, rewrite its install name to the absolute
  cache path (`install_name_tool -change`/`-id
  ~/.frozen-duckdb/cache/v{VERSION}-{arch}/libduckdb.dylib`). Consumers then
  embed an absolute path and need no rpath and no `DYLD_LIBRARY_PATH`.
  Downside: relocating the cache invalidates consumers (acceptable — the
  cache path is already the stable identity, and the builder owns restaging).
- (b) **Runtime fallback lookup**: in `frozen-duckdb-builder`, on load
  failure, retry with the resolved cache path prepended to
  `DYLD_LIBRARY_PATH` for the child — keeps link-time behavior unchanged.

(b) is the 80/20 fallback if (a) hits codesigning/ad-hoc-signature walls
(`install_name_tool` invalidates the signature; re-sign with `codesign -f -s -`).

(b) is the 80/20 fallback if (a) hits rustc flag-scoping walls.

## 80/20 claim boundary

- PROBED: the 5-line program compiles, links, and runs against the real
  dylib out-of-tree with a path dep, from one command, on this machine
  (receipt above / in the run log).
- CONFIRMED (not assumed): the rpath gap is real on this subject — the
  consumer binary carries no `LC_RPATH` and fails to load bare; the probe
  proves both directions (bare run fails, cache-dir `DYLD_LIBRARY_PATH` run
  passes `SELECT 42`).
- NOT claimed: fresh-machine cold floor (GitHub Release assets for 2.0.0-dev
  are unpublished, so a fresh machine hits the local-compile fallback —
  measured separately, not here), Windows/Linux consumer surfaces, released
  crates.io consumption (path dep only), the rpath fix itself (proposal only,
  gap flagged to the release lane).
- Known drag stated honestly: phase 1 dep-tree compile dominates; fix lever
  is LSS6, not this script.
