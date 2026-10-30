# UPGRADE-WO — frozen-duckdb — v2.0.0

| field | value |
|---|---|
| repo | `/Users/sac/frozen-duckdb` (canonical checkout) |
| work order | Upgrade to DuckDB 2.0 (dev pin, no release tag) |
| upstream pin | `d54cbbe3df17e8e5ce68312f483b4155836112aa` (duckdb `main`; no `v2.0*` tags exist as of 2026-10-19) |
| crate version target | `2.0.0` (mirrors bundled DuckDB; dev-suffix prerelease, NOT published) |
| lane | W11 (writer), per `_LANES.md` (12 writer + 4 verifier + 4 auditor lanes) |
| standing | **PARTIAL_ALIVE** |
| date opened | 2026-10-19 |

Basis (2026-10-19): engine flip ALIVE (rebuilt dylib self-reports
`v2.0.0`, witnessed strings probe, CAMPAIGN-RECEIPT F10 section);
court battery green per DOD-2..10 (storage 2/2, types 3/3, leak, concurrency
2/2 w/ vacuity catch, OCEL 94.93/97.01 + SHIM flip, latency 12.3ms
non-vacuous, throughput, OOM both legs); final standing is PARTIAL_ALIVE not
ALIVE because 11 KD rows remain open (KD-01/02/05/06/07/08/09/10/11/12/13 in
`KNOWN_DIVERGENCES.md` — mangled exports, flock 404, wasm/host-VFS, REV-001
clause, leak-clause surface, cgroup/BEAM/browser legs, anomaly precision,
registry substrate), the blueprint layer is PROPOSED (no court flipped,
DOD-14), DOD-15/16/17/18 are open, and DOD-20 final-integration
reconciliation is REQUIRED. Standing vocabulary below ties ALIVE to zero
unresolved falsifier triggers across A1–A4; the open KD register discharges
that bar only for PARTIAL_ALIVE.

## Scope

One canonical checkout, no worktrees. Lane partition and git ownership per
`docs/sjira/v2.0.0/_LANES.md`: lanes never run git state-changing commands; the
coordinator owns all transitions and commits. This document is lane W11's write surface
within `docs/sjira/v2.0.0/`; no other file is modified by this lane.

## Phase-0 probe facts (recorded 2026-10-19)

- Upstream pin: `d54cbbe3df17e8e5ce68312f483b4155836112aa` on `main`. No `v2.0` tags
  exist upstream as of 2026-10-19 — the compile fallback clones the pinned SHA, not a
  tag.
- Header probe: `DUCKDB_TYPE` enum values 1–39 unchanged from 1.5.5; new types
  `GEOMETRY=40`, `VARIANT=41`, `TIMESTAMP_TZ_NS=42`.
- Struct layouts: `duckdb_hugeint`, `duckdb_interval`, `duckdb_list_entry`,
  `duckdb_string_t` byte-identical to the vendored 1.5.5 headers; string accessors
  intact.
- Staged header digest: `shasum -a 256 duckdb.h` =
  `8dbc9c71a10f0a75fcacc1a3f4f7c76093a84d232fc152222ee5b6e512ab4560`.
- Decisions:
  - No new typed `Config` options until the runtime probe is possible (deferred).
  - `test-dependency` gains an `upstream-compat` feature; dual-engine linking is blocked
    by `links = "duckdb"` (one native lib per process).
  - No crates.io publish of the `2.0.0` prerelease — hold for the final release
    (permanent-publish risk).
  - Compile fallback clones the pinned SHA `b6013f73…` (no dev tag exists to fetch).

## Deliverables (file ownership per _LANES.md)

| deliverable | owner lane |
|---|---|
| sys pin: upstream SHA `b6013f73…`, sys Cargo.toml | W1 |
| vendored `duckdb.h` / `duckdb.hpp` refresh (sha `8dbc9c71…`) | W2 |
| builder cache versioning, cache dir labeled `v2.0.0` | W3 |
| `logical_type.rs` new type codes 40/41/42 | W4 |
| struct audit (hugeint/interval/list_entry/string_t) | W5 |
| `config.rs` surface (deferred options documented) | W6 |
| public types surface | W7 |
| test-dependency/test-validation manifests (`upstream-compat` feature) | W8 |
| workflows | W9 |
| ggen.toml + generated surface (`make sync`) | W10 |
| this milestone window (`docs/sjira/v2.0.0/`) | W11 |
| scripts/ + README.md | W12 |

Shared seams: `ggen.toml`/Makefile wholly W10; W8 manifests-only; no co-owned files.

## Falsifier

The work order fails if any of:

1. Any verifier receipt (V1–V4) lacks real command output — command, exit code, and
   actual output. A receipt that asserts without output is SUSPECT, not a receipt.
2. V2's receipt does not prove `duckdb_library_version()` returns a `2.0.0*`
   string (dev-suffix tolerance: `2.0.0` followed by anything, or bare `2.0.0`)
   consistent with the builder pin.
3. V3's cache-poison falsifier fails: a cache directory labeled `v2.0.0` must
   contain a dylib that self-reports a `v2.0.0*` version. If a stale 1.5.5 dylib
   satisfies the `v2.0.0` cache label, or the version probe passes on a mismatched
   library, the cache versioning is vacuous and the work order fails.
4. Any auditor finding (A1–A4) shows a generated file hand-edited (Makefile, generated/
   surfaces) rather than regenerated via `make sync`.

## Standing vocabulary

- **IN_PROGRESS** (current): deliverables not yet all landed and verifier-receipted.
- ALIVE requires: every deliverable on disk, every V-lane receipt carrying real
  command + exit + output, falsifiers 2 and 3 witnessed passing, zero unresolved
  falsifier triggers across A1–A4.
- BLOCKED: pinned upstream SHA becomes unfetchable, or `links = "duckdb"` conflicts
  surface that cannot be resolved within the lane partition.

## Wave-2 (2026-10-19, recorded by W11)

Wave 2 ran 19 orthogonal lanes over the same canonical checkout:

- **V1–V5** — verifier lanes; findings logged in `receipts/VERIFIER-RECEIPTS-LOG.md`.
- **A15–A28** — auditor lanes (14 auditors, extending the original A1–A4 partition).
- **D1** — docs repair lane (consumed A1's ~100 doc-hit sweep).

Coordinator repairs landed at integration (coordinator-owned transitions):

1. `prebuilt/` header re-stage after W2's path miss (see W11-correction notes below).
2. `setup_optimized_build.sh` + 2 other script pins (W12 scope, applied by
   coordinator after the lane closed).
3. `create_frozen_setup` dual probe.
4. `test-dependency` path-dependency fix — pre-existing defect, surfaced by the
   wave, repaired at integration.
5. `statement.rs` Variant bind arm (new type code 41 arm missing from match).
6. `value_ref.rs` `expect()` fix (A18 finding).
7. `cargo fmt` hunk.

## W11-correction notes

- W2 initially staged the refreshed headers into
  `crates/frozen-duckdb-builder/prebuilt/` — the wrong tree; the shipped header
  copy lives at the repo-root `prebuilt/`. The coordinator re-staged root
  `prebuilt/` and removed the stray directory.
- Corrected `duckdb.hpp` digest:
  `aab42f263dfdaef6547b550a3a62e00201df56844087b44a6f10676412258438`.
  (`duckdb.h` digest `8dbc9c71…` from Phase-0 is unaffected.)

## Execution boundary

Nothing in this document asserts execution. Commands, exits, and digests belong in
verifier receipts (`receipts/`, dropped by the coordinator) and auditor findings, not
here. This work order records scope, probe facts, ownership, and the falsifier only.
