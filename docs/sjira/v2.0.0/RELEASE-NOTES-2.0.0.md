# RELEASE-NOTES-2.0.0 — frozen-duckdb 2.0.0

Milestone doc for the `v2.0.0` prerelease. Patterned on the operator's
v1.5.5 release notes (`docs/sjira/v26.9.21/PR_BODY.md`, merged PR #3 body,
preserved verbatim as release evidence). Release channel: prerelease
`2.0.0`, pinned to upstream DuckDB `v2.0-cyanoptera` SHA
`d54cbbe3df17e8e5ce68312f483b4155836112aa` (short `b6013f7`). No upstream
`v2.0.0` tag exists yet; the shipped pin is
upstream main at the named SHA. Engine dylib self-reports `v2.0.0`
(strings probe, WITNESSED, `CAMPAIGN-RECEIPT.md` §"Engine flip (F10)").
Standing: **PARTIAL_ALIVE** — every number below names its receipt.

## Summary

This release makes crate `frozen-duckdb` 2.0.0 track upstream DuckDB
2.0.0 at pin `b6013f7`: the crate version mirrors the targeted upstream
release. It is a drop-in bump for existing users of the `duckdb`/`duckdb-rs`
API surface — the same `Connection`/`Statement`/`Row` public API, now backed
by the 2.0.0 engine (see `MIGRATION_GUIDE.md`, "Quick Migration
(2 minutes)").

Three structural problems are fixed along the way:

1. **Release assets now match the downloader.** The prior release's CI left
   the GitHub Release without assets; consumers silently fell back to a full
   local DuckDB compile (30-90 min first build, witnessed out-of-tree by
   `receipts/PB4-registry-rehearsal.md`). `build-binaries.yml` now stages and
   uploads `libduckdb_{arch}.dylib` — the exact names
   `frozen-duckdb-builder::ensure_binary()` downloads (lib.rs:78 asset law;
   `TAG-REHEARSAL.md` step 3), with a workflow-dispatch guard that hardcodes
   `BUILDER_VERSION="2.0.0"` and fails loudly on any other input.
2. **The drop-in API layer carries the 2.0.0 surface.** The vendored
   duckdb-rs API layer compiles against the dev engine with the VARIANT type
   surface green (doctor stage `03_types_variant` 3/3, DISPATCH-CITED,
   `CAMPAIGN-RECEIPT.md` §"Court battery finals") and the storage-version
   gates documented (`docs/design/storage-format-v2.md`).
3. **Linked binaries find the dylib with no environment setup.** `frozen-duckdb-sys`
   emits `cargo:rustc-link-search=native={cache dir}` so the linker records
   the found library's Mach-O install ID, and the builder now stamps that
   install ID to the absolute cache path (`ensure_install_id`,
   `crates/frozen-duckdb-builder/src/lib.rs`; KD-15 CLOSED, receipt
   `receipts/ADV9-rpath.md`) — a consumer binary runs bare, no
   `DYLD_LIBRARY_PATH`, no `.cargo/config.toml` rustflags.

The ask: no open GitHub issue exists for 2.0 support yet (issue #1, the v1.5
ask, was closed 2026-09-22 by PR #3). This release answers the same ask
one major ahead — "a version with DuckDB 2.0" — ahead of the upstream
`v2.0.0` tag.

## Changes

### builder — cache + install-ID stamp + build lock
- Versioned cache slot `~/.frozen-duckdb/cache/v2.0.0-{arch}/`;
  `ensure_binary()` download law `libduckdb_{arch}.{dylib|so}` (lib.rs:78);
  local source-compile fallback pinned at `b6013f7` (clone → checkout →
  cmake `-DBUILD_EXTENSIONS=parquet;json;icu;httpfs;tpch;tpcds;fts;inet;sqlsmith`
  `-DBUILD_JEMALLOC=ON -DBUILD_AUTOLOAD=ON` → `make -j4` → `lipo` slice
  verification → atomic cache copy; full chain audited in
  `BINARIES-PIPELINE-AUDIT.md` row 1).
- **KD-15 install-ID stamp** (`receipts/ADV9-rpath.md`): new `ensure_install_id()`
  (macOS; non-macOS no-op) wired at four sites — `copy_prebuilt_to_cache`,
  `persist_verified_for`, `compile_duckdb_locally` (stamp BEFORE size-pin
  staging; measured 65,900,736 → 65,535,744 bytes on the real engine), and
  `accept_cached_entry` (self-heal for pre-existing caches, idempotent on
  second build). Failure to stamp is a loud `warn!` + `Ok(false)`, never a
  bricked build; re-sign via `codesign --force --sign -`, hard-error on
  re-sign failure.
- **BuildLock** (ADV11): create-exclusive + PID-liveness + steal; bounded
  poll on a trusted-entry observable; 8 lock courts — closes the
  double-compile race (commit `dbac0b4`).
- Vendored DuckDB headers (`vendored-headers/{duckdb.h,duckdb.hpp}`) ship in
  the builder crate — bindgen works offline regardless of how the binary was
  obtained.

### sys — bindgen + links
- Bindgen from the builder's vendored headers; docs.rs mode (`DOCS_RS=1`)
  generates bindings and emits **no** linking directive (AD8, PASS 4.89 s).
- `links = "duckdb"` + `cargo:rustc-link-search=native={cache dir}` — the
  propagating channel ADV9 proved (`cargo:rustc-link-arg` from a dependency
  build script does NOT reach a consumer's final link; dead ends documented
  in `receipts/ADV9-rpath.md`).

### API layer — 2.0.0 drop-in surface
- Drop-in `duckdb-rs` replacement: same `Connection`/`Statement`/`Row`
  surface; dependency set per the restored layer (arrow 56, hashlink 0.10,
  strum 0.27, fallible-iterator/streaming-iterator, rust_decimal, cast).
- VARIANT type surface green (stage `03_types_variant` 3/3, DISPATCH-CITED).
- Storage-version gates: nested-schema storage requires an explicit v2.0.0
  storage version; 1.5.x databases take the export/recreate path
  (`docs/design/storage-format-v2.md:14,27-28`).

### linking — install-ID stamp (KD-15 closed, macOS)
- Consumers need NO env: after the stamp, the linker records the absolute
  cache path in `LC_LOAD_DYLIB`. Real end-to-end (`receipts/ADV9-rpath.md`):
  `otool -D` → stamped absolute ID; out-of-tree consumer
  (`/tmp/adv9/consumer`, links sys only) → `CONSUMER_BARE_RUN_OK`, exit 0
  with `env -u DYLD_LIBRARY_PATH`; second build idempotent. Pre-ADV9
  reproduction (exit 134, "no LC_RPATH's found") is in the same receipt.
  Linux is UNVERIFIED by construction — ELF has no LC_ID_DYLIB; the stamp
  is a macOS-only no-op.

### CI — release asset naming + dispatch guard
- `build-binaries.yml` uploads `libduckdb_{arch}.dylib` matching
  `ensure_binary()` exactly (no `libfrozen_mega_*` leftovers); fires on
  `push: tags: v*` and `workflow_dispatch` with a hardcoded
  `BUILDER_VERSION="2.0.0"` guard (yml lines 12/60-73; any other input
  fails the run loudly). Prior record is red and named:
  runs 36239600176 (tag v1.5.5) and 37940771273 (tag v26.10.8) both FAILED;
  no GitHub Release exists today (`TAG-REHEARSAL.md` §0.2).

### scripts + docs
- `scripts/publish_chain.sh` (order orchestrator builder → sys → main,
  bounded index polls, POLL-STEP classified); `scripts/verify_release_assets.sh`
  (REL8: `PASS — 2/2` + sha256, exit 0 gate); `scripts/fastfollow_flip.sh`
  (mechanical version flip, 29 FLIP lines rehearsed, `receipts/PB8-flip-publish.md`);
  `scripts/cleanroom/registry_consumer.sh` (zero-compile consumer court,
  commit `fc2bbec`); `scripts/doctor.sh` 13-stage fail-closed battery with
  stages 11/12 wired; `scripts/extension_watch.sh` (KD-02 close-condition
  watcher, `receipts/ADV12-extension-watch.md`).
- Docs: `MIGRATION_GUIDE.md`, `KNOWN_DIVERGENCES.md` (11 open / 3 closed),
  `KNOWN-LIMITATIONS-2.0.0.md`, `TAG-REHEARSAL.md`, `PUBLISH-DECISION.md`.

### versioning
- Workspace version `2.0.0`; all three crates publish as `2.0.0`.
  Internal path dependencies carry `version = "2.0.0"` keys
  (publish-safe; triggers the registry rewrite proven in the pre-flight).
  MSRV 1.87, empirically proven (`receipts/ADV10-msrv-proof.md`, inherited
  by all three crates). `publish = false` on test-dependency / test-validation
  (PB1, commit plan ADV16).

## Test plan

Consolidated receipts, newest gate identity first.

### Final battery (generated, `scripts/verify-evidence.ttl` @ 2026-10-20T00:49:40Z, head dbac0b4)

| gate | tool | exit |
|---|---|---|
| build-workspace | `cargo build --workspace` | 0 |
| clippy-deny-warnings | `cargo clippy --workspace`<br>`--all-features -- -D warnings` | 0 |
| fmt-check | `cargo fmt --all -- --check` | **1 — open andon** |
| test-validation | `cargo run -p test-validation` | 0 |
| test-workspace | `cargo test --workspace` | 0 |

One red row is disclosed, not buried: `fmt-check` exit 1 at the last
regeneration. `make verify` (which regenerates this file) must be green on
the merged head before any tag — TAG-REHEARSAL step 1, halt rule [b].

### Census (integration census, ADV16, 2026-10-19)

`git status --porcelain` / `git diff --stat` census: 9 modified + 20
untracked = 29 integration paths, all lane-attributed in
`ADV16-INTEGRATION.md` §1; no lane target dirs (`target-lane*`) in status;
diff scope +92/−47 with no out-of-ownership edits. (Named as ADV16 here
because no receipt file under `receipts/` carries a "CG3" identifier — the
integration census lane in the dispatched wave is ADV16, and a "CG3 census"
citation would be unreceipted.)

### Crate batteries (real output embedded in receipts)

| battery | result | receipt |
|---|---|---|
| builder | `test result: ok. 43 passed; 0 failed` (post-KD-15 stamp) | `receipts/ADV9-rpath.md` |
| sys | `89 passed; 0 failed` (stub/link courts) | `receipts/ADV9-rpath.md` |
| builder @ 2.0.0 flip scratch | `35 passed; 0 failed` (pre-ADV1) | `receipts/PB8-flip-publish.md` |
| GATE-V-01 latency, median-of-3 | median p95 13.36 ms < 15 ms, PASS | `receipts/ADV5-median3.md` |
| KD-02 watcher | `HTTP 404 still absent`, EXIT=1 | `receipts/ADV12-extension-watch.md` |
| wasm size court | FAIL exit 1 — 39.20 vs 25 MiB; wire PASS | `receipts/ADV13-wasm-size.md` |

### Court table (from CAMPAIGN-RECEIPT standing roll-up, 2026-10-19)

| surface | standing | basis |
|---|---|---|
| engine v2.0.0 dylib | ALIVE | WITNESSED (strings probe) |
| storage/types/leak/concurrency courts | PASS | DISPATCH-CITED |
| OCEL 94.93/97.01% + SHIM flip | PASS | DISPATCH-CITED |
| latency/throughput/OOM courts | PASS | DISPATCH-CITED |
| crash-chaos (Z2) | PARTIAL_ALIVE | DISPATCH-CITED |
| revenue packs REV-001..004 | PASS / OBSERVED | DISPATCH-CITED |
| fleet classes A-E | PASS + loud blocks | DISPATCH-CITED |
| warehouse-sim courts + S-wave | PASS | mixed WITNESSED/DISPATCH-CITED |
| doctor ladder | 13-stage (6 green, remaining KD-mapped) | DISPATCH-CITED |
| browser/OPFS (FR-01/FR-02) | BLOCKED | loud, never faked |
| blueprint layer | PROPOSED | no court flipped (BP16) |
| cost constants (sim) | ASSUMPTION-UNVERIFIED | counterfactual params |

## Release pre-flight (AD8 package-chain audit, 2026-10-19, master @ 79e803e)

Method: `cargo package --allow-dirty --offline --list` per crate + full grep
sweep of every ship-list entry + extracted `.crate` content inspection +
real builds (docs.rs-mode sys build, isolated registry-scan binary compiled
with rustc against a docs.rs-shaped `$CARGO_HOME`).

### Publish dry-run table

| crate | packaging | outcome |
|---|---|---|
| frozen-duckdb-builder | exit 0 | READY — packaged 437.2 KiB (79.3 KiB
  compressed, PB8 step 6); live `cargo publish --dry-run` exit 0 (PB8
  step 8, flipped final version) |
| frozen-duckdb-sys | fails at dep resolution | ordering artifact — builder `2.0.0` not
  on the registry; a version-pinned path dep resolves from the registry
  during publish verification. Order: **builder → sys → main** |
| frozen-duckdb | fails at dependency resolution | same class |

Compensating verification (the AD8/DOD-4 replica proof): leak-clean ship
lists verified per crate (AD8 per-crate verdicts: builder 8 files, sys 12,
main 77 after the AD8 exclusion of 13 machine-coupled court fixtures — zero
`/Users/...` hits post-fix); docs.rs mode sys build PASS (4.89 s, bindings
from vendored headers, no link directive); the `find_vendored_headers()`
registry-candidate branch exercised in isolation against the real `.crate`
layout → `FOUND .../frozen-duckdb-builder-2.0.0/vendored-headers`;
consumer cleanroom court (commit `fc2bbec`, local PASS — cache-hit, no
DuckDB compile, v2.0.0 query).

Server-side acceptance of the version form is the one hop no dry-run can
witness (PB8); it is operator authority on final day. `2.0.0` as a
prerelease form is DISPUTED on disk (FIN6: unpublishable registry gate;
PB8: local pipeline accepted the flipped final `2.0.0`; PUBLISH-DECISION.md).

### Header shipping gate

`cargo package -p frozen-duckdb-builder --list` contains
`vendored-headers/duckdb.h` and `vendored-headers/duckdb.hpp` (AD8 ship
list, row builder). Also `LICENSE-DUCKDB.md` must be present in all three
crate dirs before the flip lands (PB8 DEFECT, since corrected to
MIT-attribution per coordinator, commit `dbac0b4`).

### Package sizes (crates.io cap: 10 MiB per `.crate`)

| crate | `.crate` | unpacked raw tree |
|---|---|---|
| frozen-duckdb-builder | **437.2 KiB** observed (79.3 KiB compressed; PB8) | 345,566 B (AD8) |
| frozen-duckdb-sys | not producible pre-publication | 15,709 B (AD8) |
| frozen-duckdb | not producible pre-publication | 970,309 B (AD8) |

All far under the cap (largest raw tree 0.97 MB; largest `.crate` observed
70 KB — AD8's own `--list`-mode measurement; PB8's 437.2 KiB is the full
`cargo package` packaging including the verification recompile output.
Both numbers are cited as found; the discrepancy is a method difference,
not a drift). Headers dominate the builder package and compress well.

## Release sequence for the operator

Full sheets: `TAG-REHEARSAL.md` (steps 1-10, precondition table P0-P6,
abort doctrine) and `STANDARD-WORK-RELEASE.md` (phases 0-3 with halts H1-H3,
andon rules, 55-95 min cycle). Sequence (publish_chain.sh order):

1. Pre-cut repo law: `ggen sync run` ×2 byte-identical; `cargo build --workspace`;
   `python3 scripts/verify_publish_receipt.py` exit 0; `make verify` green
   (incl. the fmt-check andon above closed).
2. Tag `v2.0.0` on the merge HEAD and push (tag, not `v2.0.0` — the
   dispatch guard hardcodes `2.0.0`).
3. Asset dispatch (GATE — HOLD everything until green):
   `gh workflow run build-binaries.yml --ref v2.0.0 -f version=v2.0.0`;
   watch; then `bash scripts/verify_release_assets.sh v2.0.0` →
   `PASS — 2/2` + sha256, exit 0. Expected release assets, exactly:

   ```
   libduckdb_arm64.dylib
   libduckdb_x86_64.dylib
   ```

   Any other name (`libfrozen_mega_*` leftovers count as wrong) → rerun the
   workflow. Publishing before assets exist sends every macOS first-build
   into a 404 download or a full source compile — the one irreversible step
   that can be wrongly ordered.
4. `cargo publish -p frozen-duckdb-builder` (first permanent step; 409 =
   already published).
5. Index wait for builder (bounded poll ≤10 min,
   `scripts/publish_chain.sh`).
6. `cargo publish -p frozen-duckdb-sys` (`no matching package named
   frozen-duckdb-builder found` = index lag → wait 60 s, retry the exact
   same command; 409 = done).
7. Index wait for sys; then `cargo publish -p frozen-duckdb` (failure
   handling identical).
8. docs.rs builds (observe, not a gate): docs.rs-mode sys build rehearsed
   green (`receipts/PB5-docsr.md`, ALIVE all three).
9. Post-publish verification: crates.io `max_version` poll; sparse index
   200 ×3; clean-machine smoke (`cargo add frozen-duckdb@2.0.0` +
   `Connection::open`, no `DYLD_LIBRARY_PATH`, cache auto-populated);
   provenance gate re-run (`verify_publish_receipt.py` exit 0, final head
   `chain_hash` into the History row); fast-follow flips
   (`scripts/fastfollow_flip.sh`) only after step 3 is green, each citing
   this window + `scripts/check_sjira_standing.sh` clean.

Abort doctrine (from TAG-REHEARSAL §3): before step 4 everything is
recoverable; from step 4 on each success is permanent (yank ≠ delete).
Halts H1-H3 require operator judgment; never retry-over a permanent step,
never improvise manifest edits mid-sequence.

## Known limitations

Full register with close conditions: `KNOWN-LIMITATIONS-2.0.0.md`;
divergences: `KNOWN_DIVERGENCES.md` (11 open / 3 closed). Summary:

- **Prebuilt assets are macOS-only** (osx_arm64). Linux `.so` candidate
  prepared, not dispatched (lane REL6, `scripts/build_linux_prebuilt.sh`,
  pin `b6013f7`; IN-FLIGHT per `CAMPAIGN-RECEIPT.md`). Windows: automatic
  source-compile fallback pinned at `b6013f7` — 30-90 min first build
  (`BINARIES-PIPELINE-AUDIT.md` row 1).
- **`flock`/`tpch` community extensions 404 at the dev pin.** Tests
  ignore-gated, not failing (`#[ignore]` at
  `crates/frozen-duckdb/tests/flock_tests.rs:27,533`); watcher
  `scripts/extension_watch.sh` prints the exact un-ignore line on
  `AVAILABLE` (`receipts/ADV12-extension-watch.md`, EXIT=1 today).
- **KD-01**: 27,682 mangled `__Z*` exports breach the C-ABI-only contract
  (ADV3-corrected count, from "21"); UPSTREAM-flag fix class; "Blocks
  release: No" (`KD-CLOSURE-PLAN.md`).
- **WASM artifact 39.20 MiB > 25 MiB raw SLA** (wire gzip 9.46 MiB passes
  its 10 MiB SLA); Code section 92.4% — trim must come from the compile
  surface, not post-build passes (`receipts/ADV13-wasm-size.md`).
- **crates.io publish held** (DOD-18 / PUBLISH-DECISION Option B, HOLD FOR
  FINAL); a dev/prerelease version on crates.io is a permanent index entry.
  The live-registry prerelease-acceptance question has no receipt on disk;
  if the operator overrides toward publish-now, a live
  `cargo publish --dry-run` is the mandatory first move.

## See Also

`docs/sjira/v2.0.0/CAMPAIGN-RECEIPT.md` · `TAG-REHEARSAL.md` ·
`STANDARD-WORK-RELEASE.md` · `PUBLISH-DECISION.md` ·
`KNOWN-LIMITATIONS-2.0.0.md` · `KNOWN_DIVERGENCES.md` ·
`docs/sjira/v26.9.21/PR_BODY.md` (v1.5.5 pattern source)
