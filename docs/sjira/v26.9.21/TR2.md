---
id: TR2
dcterms:title: "Release rehearsal + build-binaries hardening"
dcterms:created: "2026-09-21"
dcterms:isPartOf: "v26.9.21"
rdf:type: oslc_cm:ChangeRequest, earl:TestRequirement
aps:standing: https://w3id.org/chatman/aps#UNSUPPORTED
milestone: frozen-duckdb v1.5.5 wave 3 (hardening, pre-publish)
worktree: /Users/sac/frozen-duckdb-wt/r2
branch: feat/155-r2
---

# TR2 — Release rehearsal + build-binaries hardening

## Scope (files you may change)

```
.github/workflows/build-binaries.yml
```

Anything outside scope: do not edit — record BLOCKED with the reason instead.

## Work order


Context: the v1.5.5 tag run has never executed — rehearse it before the operator cuts. (1) HARDEN the workflow: matrix builds x86_64 on an arm64 runner — the local-compile fallback needs CMAKE_OSX_ARCHITECTURES to label the slice correctly; set it per matrix arch in the workflow AND honor an env override in builder (coordinated: TR7 owns builder lib.rs — set the env here, TR7 makes builder read it; note this dependency in History). (2) REHEARSE the ensure_binary cache-miss path locally in isolation: HOME=/tmp/fd-rehearsal-home cargo build of a scratch project depending on the builder — exercises download-404 → local compile of DuckDB v1.5.5. That is a 15-45 min full source build: run it with a generous timeout; if it exceeds 40 min, record partial evidence (cmake configured + make progressing) and stand PARTIAL. Verify the produced artifact lands at the path/name the workflow's find step expects.
DoD: [ ] workflow hardened + yaml-parse green  [ ] rehearsal executed with command+exit (or PARTIAL with preserved progress evidence)  [ ] committed

## Contract (all tickets)

- Work ONLY in `/Users/sac/frozen-duckdb-wt/r2` on `feat/155-r2`. NEVER touch the main checkout /Users/sac/frozen-duckdb or sibling worktrees. Never push, never touch remotes, never publish.
- Atomic commits, conventional messages, on your branch only.
- Verify before claiming: preserve command + exit. A gate you did not run is UNKNOWN, not green.
- Append every transition to the History table below (the one sanctioned write outside your worktree), then STOP — the coordinator merges.

## History

| ts | standing | branch+SHA | gates+exits | remaining |
|----|----------|------------|-------------|-----------|
| 2026-09-21 | OPEN | feat/155-r2@c55d4f2 | not started | all of DoD |
| 2026-09-21 | PARTIAL_ALIVE | feat/155-r2@00d0310 | yq parse OK; actionlint OK (HEAD had 3 findings in-scope: SC2086 x2, action-gh-release@v1 runner too old — all fixed in 00d0310; also mkdir -p src never-run bug + CMAKE_OSX_ARCHITECTURES per matrix arch) | rehearsal + artifact-placement check |
| 2026-09-21 | PARTIAL_ALIVE | feat/155-r2@00d0310 | rehearsal run: HOME=/tmp/fd-rehearsal-home cargo run --release (scratch dep on builder by path) → REHEARSAL_EXIT=1; cache-miss + download-404 EXERCISED (log /tmp/fd-rehearsal/rehearsal.log); local-compile fallback BROKEN: builder cmake `-DBUILD_EXTENSIONS=ON` invalid for DuckDB v1.5.5 (expects extension-name list; cmake tries extension literal "ON" → configure exit 1) AND builder never checks cmake/make exit status — lib.rs is TR7 scope, not edited (BLOCKED for TR7). Corrected recipe found: BUILD_EXTENSIONS=parquet;json;icu;httpfs;tpch;tpcds;fts;inet;sqlsmith + BUILD_JEMALLOC=ON + BUILD_AUTOLOAD=ON → cmake exit 0 (visualizer/tpce out-of-tree in v1.5.5; excel needs minizip-ng absent on runner unless brew-installed). Corrected make -j4 in progress. | corrected-compile proof, artifact-placement, final row |
| 2026-09-21 | PARTIAL_ALIVE | feat/155-r2@00d0310 | corrected recipe proof: cmake exit 0 (/tmp/fd-rehearsal/cmake-fixed3.log) + make -j4 MAKE_EXIT=0 in ~17 min → build/src/libduckdb.dylib 59MB Mach-O arm64; env mechanism proven: CMAKE_OSX_ARCHITECTURES=x86_64 env → cmake cache → x86_64 dylib on arm64 host (/tmp/fd-rehearsal/cmake-env-test). Placement: find on empty cache = fail-closed (empty → exit 1, matches rehearsal state); artifact placed via builder's own get_binary_path naming → workflow verbatim find+stage exit 0 → stage/libduckdb_arm64.dylib → release basename matches builder download URL releases/download/v1.5.5/libduckdb_{arch}.dylib. TR7 DEPENDENCY: workflow exports CMAKE_OSX_ARCHITECTURES per matrix arch since 00d0310; until builder honors it (arch detection + cmake flags fix + status checks), x86_64 leg fails closed at 'Find built binary' (no mislabeled slice ships). Worktree clean @00d0310. | nothing for TR2; TR7: lib.rs cmake flags + status checks + env override |
| 2026-09-21 | UNSUPPORTED | feat/155-r2@00d0310 | 帳 row: hand-hardened 19(+)/6(−) lines in .github/workflows/build-binaries.yml on 産面. Missing capability: no pack individual/template renders frozen-duckdb release build-binaries workflow (nearest family: tcps-release-pack tcps:CiWorkflow — individuals are fixed workflowBody literals for tcps's own tier-1, not parameterized for this repo). Intended owner: frozen-duckdb CI pack individual under the CiWorkflow family. Admitted by ticket TR2 scope (coordinator-issued). Date 2026-09-21. | paydown: pack EXTEND by future ticket, not TR2 |
