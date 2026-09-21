---
dcterms:title: "frozen-duckdb v1.5.5 — Definition of Done"
dcterms:created: "2026-09-21"
rdf:type: prov:Activity
aps:standing: https://w3id.org/chatman/aps#OPEN
---

# MILESTONE v26.9.21 — frozen-duckdb v1.5.5 (DuckDB 1.5.5, closes issue #1)

## Done (coordinator, this session)

- [x] DuckDB v1.5.5 universal dylib + headers staged in ~/.frozen-duckdb/cache/v1.5.5-{arm64,x86_64}; ctypes + CLI verified v1.5.5
- [x] C API 1.4→1.5 diff: 0 removals / 0 renames / 0 layout changes; all 459 referenced symbols present (bindgen SAFE)
- [x] Pins bumped to 1.5.5: builder, scripts, CI defaults, docs, workspace version (waves feat/155-{crates,docs,scripts,ci}, merged)
- [x] CI release-asset naming fixed: assets now match ensure_binary() download URLs
- [x] Builder: vendored 1.5.5 headers, duckdb/ layout, libduckdb.dylib link name, cache normalization on every acquisition path
- [x] sys: bindgen double-include fixed; links=duckdb + DEP metadata
- [x] Vendored duckdb-rs API layer restored to compilation (mod wiring, crate:: paths, arrow 56 / hashlink 0.10 / strum 0.27 / fallible-* / rust_decimal / cast)
- [x] Runtime @rpath propagation via DEP_DUCKDB build script; CLI bin executes against 1.5.5 (dyld clean)
- [x] Workspace cargo build → exit 0

## Open (this wave — tickets T2A..T8)

- [ ] T2A — Core test suite green
- [ ] Context: tests/ are integration tests — 'crate::' paths are illegal there; use frozen_duckdb::. test_all_types.rs moved from lib-cfg-test to tests/: fix paths, then run it against DuckDB 1.5.5 and repair the EXCLUDE list / match fallthrough (todo!() panics on unknown columns; 1.5.5 may add GEOMETRY/VARIANT) and any golden drift, empirically driven by failures. dev-dep pretty_assertions is already declared.
- [ ] DoD: [ ] cargo test -p frozen-duckdb --lib → exit 0  [ ] cargo test -p frozen-duckdb --test frozen_duckdb_tests --test dropin_compatibility_tests --test core_functionality_tests → exit 0 (or documented #[ignore] + UNSUPPORTED row in History)  [ ] all fixes committed atomically on feat/155-tests-core
- [ ] T2B — Extension test suite green/gated
- [ ] Context: fix 'use duckdb::' → 'use frozen_duckdb::'. polars_tests.rs imports a polars crate that is not a dependency → gate whole file with #![cfg(feature = "polars")] + UNSUPPORTED(polars dev-dep not carried) note; do NOT add polars. flock_tests.rs needs community extension flock + live Ollama → runtime gate: early-return SKIP unless env FLOCK_TEST=1. tpch/parquet: official 1.5.5 dylib ships parquet statically; tpch needs network INSTALL — attempt real run; if network-blocked, env-gate DUCKDB_NET_TESTS=1 with default-on and note. vss probes soft-skip by design — keep.
- [ ] DoD: [ ] cargo test -p frozen-duckdb --test arrow_tests --test parquet_tests --test vss_tests --test tpch_integration_test --test flock_tests → exit 0 (gated skips documented in History)  [ ] committed
- [ ] T3 — Examples compile + run
- [ ] Context: 'use duckdb::' rot. Fix to frozen_duckdb::, build all examples, then EXECUTE basic_usage against the frozen 1.5.5 dylib (execution evidence, not inspection). performance_comparison may be slow — cap with timeout 300.
- [ ] DoD: [ ] cargo build --workspace --examples → exit 0  [ ] cargo run --example basic_usage → exit 0, output in History  [ ] cargo run --example performance_comparison → exit 0 or BLOCKED with preserved output
- [ ] T4 — Docs accuracy final pass
- [ ] Context: verify every CURRENT-behavior claim against implemented reality: release asset names libduckdb_{arch}.dylib; vendored 1.5.5 headers in frozen-duckdb-builder (offline builds); @rpath propagation (no DYLD_LIBRARY_PATH needed for bins/tests); universal macOS dylib serving arm64+x86_64; local-compile fallback pinned v1.5.5; crate version 1.5.5 mirrors DuckDB 1.5.5. Grep-anchor each claim in code before asserting it. Historical records (CONTEXT.md phases, ADRs) stay untouched.
- [ ] DoD: [ ] grep -rn '1\.4\.0' --include='*.md' → only intentional-historical hits, each with justification in History  [ ] README 'DuckDB 1.5 Support' section matches final reality incl. issue #1 answered  [ ] committed
- [ ] T5 — CI parity: clippy/fmt/workflows
- [ ] Context: ci.yml gates are fmt --check, clippy --all-targets --all-features -D warnings, build, test, examples. Make those pass locally. Prefer minimal targeted #[allow] at vendored module roots (comment: vendored from duckdb-rs 1.4.0-era; do not churn) over broad lint suppression; cargo fmt the workspace if the diff is mechanical. build-binaries.yml was fixed on feat/155-ci (asset names libduckdb_{arch}.dylib) — re-verify against the new vendored-headers builder reality and ci-simple/test-minimal coherence. yaml-parse every workflow you touch.
- [ ] DoD: [ ] cargo clippy --workspace --all-targets --all-features -- -D warnings → exit 0 (or documented ci.yml amendment)  [ ] cargo fmt --check → exit 0  [ ] workflows yaml-parse  [ ] committed
- [ ] T6 — Scripts repair: fiction + float pins
- [ ] Context: create_frozen_setup.sh fetches libduckdb_{arch}.dylib assets that never existed upstream (fictional scheme) — rewrite to fetch from this repo's GitHub Releases (the real post-fix scheme) or remove the dead path with an UNSUPPORTED note. build_frozen_duckdb.sh clones duckdb-rs default branch unpinned → pin --branch v1.10505.0 (duckdb-rs crate version encoding DuckDB 1.5.5). Sweep remaining 1.4.x pins (create_frozen_setup.sh has v1.4.1). bash -n everything touched. Do NOT execute build scripts.
- [ ] DoD: [ ] grep -rn 'v1\.4' scripts/ → zero live pins  [ ] bash -n on all edited scripts → exit 0  [ ] committed
- [ ] T7 — Verification: cache normalization + rpath
- [ ] Context: prove the builder's normalization and rpath propagation by execution. (1) Delete ~/.frozen-duckdb/cache/v1.5.5-arm64/duckdb/ and libduckdb.dylib → cargo build -p frozen-duckdb → assert both restored (vendored headers + symlink) and build exit 0. (2) otool -l target/debug/frozen-duckdb-cli  —  grep -A2 LC_RPATH shows the cache dir. (3) env -i HOME=/Users/sac PATH=/usr/bin:/bin <abs> frozen-duckdb-cli --help → exit 0 (clean-env dyld proof). (4) python3 ctypes duckdb_library_version() == v1.5.5 post-mutation. (5) cd test-dependency && cargo build → dual-engine attempt, record outcome (UNKNOWN acceptable, preserve output).
- [ ] DoD: [ ] all five probes executed with command+exit recorded in History  [ ] standing declared
- [x] T8 — Release preflight + comms artifacts
- [ ] Context: (1) cargo publish --dry-run in order: frozen-duckdb-builder, frozen-duckdb-sys, frozen-duckdb — fix nothing; record all three outcomes. Verify vendored headers ship: cargo package -p frozen-duckdb-builder --list  —  grep vendored-headers. Record package sizes (crates.io limit 10MiB). (2) Render PR_BODY.md — title 'feat: DuckDB 1.5.5 support — pin bump, release asset fix, API layer restoration (closes #1)' + Summary/Changes/Test-plan with [FILL] slots for cross-ticket evidence. (3) Render ISSUE_1_COMMENT.md — warm reply to @dfeyer (opened 2026-03-23: 'Did you plan to release a version with DuckDB 1.5?'): done, crate v1.5.5 tracks upstream v1.5.5, drop-in, automatic prebuilt download (macOS universal arm64+x86_64), vendored headers, honest note that Windows/Linux assets remain roadmap, #{PR_NUMBER} placeholder. (4) MILESTONE.md: mark DoD done-items and list operator cuts (merge, tag v1.5.5 push, release-asset verify, cargo publish, post comment).
- Outcomes (2026-09-21, direct evidence): builder dry-run exit 0 (full verification build from unpacked package); sys dry-run exit 101 and frozen-duckdb dry-run exit 101 — cargo resolves path+version deps from crates.io during publish verification, so a downstream dry-run cannot pass until its upstream crate is live on the index (expected ordering artifact, not a manifest defect); compensated with clean builds (exit 0 each) of exact packaged-file-set replicas carrying the publish-rewritten manifests. Headers gate: `vendored-headers/{duckdb.h,duckdb.hpp}` present in `cargo package -p frozen-duckdb-builder --list`. Sizes: builder 424.4 KiB `.crate` / 2.3 MiB unpacked (observed); sys ~717 KiB and frozen-duckdb ~802 KiB unpacked (computed from package file lists) — all far under the 10 MiB cap. Full outcome table + release sequence in `PR_BODY.md`.
- [x] DoD: [x] three dry-runs executed + recorded  [x] vendored-headers in package list  [x] PR_BODY.md + ISSUE_1_COMMENT.md + MILESTONE.md rendered  [x] committed (same commit as this update; SHA in T8 History)

## Operator cuts (権 — agent-attempted only with fresh operator authority)

- [ ] Merge feat/duckdb-1.5.5 → master (coordinator opens draft PR; operator merges)
- [ ] git tag v1.5.5 && git push origin v1.5.5 → build-binaries.yml publishes libduckdb_{arm64,x86_64}.dylib
- [ ] Verify release assets + clean-machine download path — checklist rendered in docs/sjira/v26.9.21/PR_BODY.md (Release-asset verification checklist)
- [ ] cargo publish — STRICT order with index wait: `cargo publish -p frozen-duckdb-builder` → wait for crates.io index to reflect 1.5.5 (usually <1 min) → `-p frozen-duckdb-sys` → `-p frozen-duckdb`. T8 pre-flight (2026-09-21): builder dry-run green; sys/frozen-duckdb dry-runs cannot pass until builder/sys are live on the index (cargo resolves their path+version deps from crates.io) — a `no matching package named frozen-duckdb-{builder,sys} found` error there means the index has not caught up, not a broken manifest. Version keys verified: `version = "1.5.5"` present on all internal path deps. Sizes verified ≤ 2.3 MiB, well under the 10 MiB crates.io cap.
- [ ] Post ISSUE_1_COMMENT.md on issue #1 after publish (substitute #{PR_NUMBER} with the merged PR number first)
