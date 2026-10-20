# Frozen DuckDB

[![CI](https://github.com/seanchatmangpt/frozen-duckdb/actions/workflows/ci.yml/badge.svg)](https://github.com/seanchatmangpt/frozen-duckdb/actions/workflows/ci.yml)

**Zero-toolchain embedded columnar engine for Rust — one prebuilt dylib,
link-only builds, and a 13-stage Chicago-court verification battery.**

Frozen DuckDB is a drop-in `duckdb-rs` replacement built on the Builder
Sub-Crate Pattern: the DuckDB engine is compiled once into a frozen dynamic
library, cached globally, and linked in seconds. Every release claim is
gated by real courts — the numbers in this file all carry receipt paths.

## Why This Exists

Using DuckDB from Rust today means one of two drags:

- **C++17 compile drag**: building the engine from source means many minutes
  of C++17 compilation (Arrow, ICU, jemalloc, the full extension set) on
  every cold checkout and every clean build. Specific measured build-time
  numbers live with their receipts — `docs/sjira/v2.0.0-dev/CAMPAIGN-RECEIPT.md`
  and `docs/sjira/v2.0.0-dev/DEFINITION-OF-DONE.md` — not in this file.
- **Cloud billing drag**: hosted columnar warehouses charge per scanned
  byte. The measured funnel shows **0 B egress** on the same workload
  (`docs/sjira/v2.0.0-dev/receipts/val004.json`,
  `"egress_bytes": 0`, by construction — a native in-memory engine has no
  network transport).

Frozen DuckDB: the engine is a frozen dylib — `cargo build` links it in
seconds, runs entirely in-process, and egress is zero by construction.

## Audited Performance (receipt-backed only)

Engine identity: DuckDB `v2.0.0-dev88595`, pinned upstream at
`b6013f7aa2f36caabb9466ce1791667ea8a280bb`. Every number below names its
receipt. Numbers marked DISPATCH-CITED are cited from the campaign receipt;
the rest are materialized court JSON on disk.

Receipt dir: `docs/sjira/v2.0.0-dev/receipts/` (R) below.

| Metric | Value | Receipt |
|---|---|---|
| Dylib size (arm64) | 65.9 / 52.4 MB unstr. / str. | `prebuilt/README.md`; `~/.frozen-duckdb/` |
| Ingest throughput (release) | 12.16M rows/s | `CAMPAIGN-RECEIPT.md` (DISPATCH-CITED) |
| Query latency p95 | 10.51 ms (< 15 ms budget) | `R/gate_v01_receipt.json` (10.5088 ms, PASS) |
| OCEL compression / query | 94.93% / 97.01% | `CAMPAIGN-RECEIPT.md` (DISPATCH-CITED) |
| Network egress | 0 B | `R/val004.json` (`"egress_bytes": 0`) |
| Fleet Class-D compression | 99.76% | `CAMPAIGN-RECEIPT.md`; `R/fleet004.json` |

No speculative numbers. Performance-validation commands:
`docs/sjira/v2.0.0-dev/DEFINITION-OF-DONE.md`.

## Quick Start

**Prerelease warning — read before `cargo add`.** crates.io currently hosts
only `frozen-duckdb` **0.1.0**; the `2.0.0-dev` line is NOT on crates.io.
`cargo add frozen-duckdb` will silently install the stale, incompatible
**0.1.0** — no error, wrong crate generation. Until the `2.0.0-dev` publish
lands, install from a checkout:

```bash
# 1. Clone (a `vendors/duckdb-rs` submodule warning is expected and harmless —
#    the crate builds from the vendored headers in frozen-duckdb-builder)
git clone https://github.com/seanchatmangpt/frozen-duckdb.git
# 2. In your own project, add the path dependency (pin a checkout at a known
#    SHA for reproducibility, e.g. d73d017cb59018cd59db7917604da4f5ac18b2eb):
cargo add --path /path/to/frozen-duckdb/crates/frozen-duckdb
```

```toml
[dependencies]
frozen-duckdb = { path = "/path/to/frozen-duckdb/crates/frozen-duckdb" } # 2.0.0-dev
```

A `--git` dependency is NOT a working alternative yet: the repository tree
carries a `vendors/duckdb-rs` gitlink with no `.gitmodules`, so cargo's
`--git`/`--rev` add fails with `no URL configured for submodule
'vendors/duckdb-rs'`. Use the clone + path dep above.

> After the first tagged publish this section becomes
> `frozen-duckdb = "2.0.0"` from crates.io.

```rust
use frozen_duckdb::{Connection, Result};

fn main() -> Result<()> {
    // Drop-in duckdb-rs API — no code changes needed.
    let conn = Connection::open_in_memory()?;

    conn.execute_batch(
        "CREATE TABLE users (id INTEGER, name TEXT);
         INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob');",
    )?;

    let mut stmt = conn.prepare("SELECT name FROM users WHERE id = ?")?;
    let name: String = stmt.query_row([1], |row| row.get(0))?;

    println!("User: {}", name); // Prints: User: Alice
    Ok(())
}
```

First build: the builder tries GitHub Releases (no `v2.0.0-dev` release is
published yet), then falls back to compiling DuckDB locally from pinned
source and caches the result under `~/.frozen-duckdb/cache/`. Subsequent
builds are link-only.

## How It Works

Three-crate workspace:

- **`frozen-duckdb`** — ultra-thin main crate (drop-in replacement)
- **`frozen-duckdb-builder`** — acquires the mega-library (download →
  local-compile fallback → global cache) and vendors the DuckDB 2.0.0-dev
  headers for offline bindgen
- **`frozen-duckdb-sys`** — FFI layer

```
cargo build ──> frozen-duckdb-sys build.rs
                   │
                   ├─ cache hit at ~/.frozen-duckdb/cache/ ──> link
                   ├─ GitHub Release download ───────────────> cache + link
                   └─ local compile (pinned b6013f7) ────────> cache + link
```

The dylib carries a neutral install name (`@rpath/libduckdb.dylib`) and the
build script emits matching runtime rpaths — no `DYLD_*` variables needed.

## Documentation (Diataxis)

- **Tutorials** — [Quick Start](#quick-start) (above);
  `cargo run --example dropin_replacement`
- **How-to guides**
  - CLI tool: `cargo install frozen-duckdb` then
    `frozen-duckdb-cli download --dataset tpch --format parquet --output-dir ./data`
  - Troubleshooting (below): cache, architecture detection, verbosity
  - Doctor battery: `just doctor` (see Verification)
- **Reference**
  - Extension set: [Built-in Extensions](#built-in-extensions)
  - Platform support: below
  - Architecture/ABI surface: `docs/design/extension-abi-hub.md`,
    `docs/design/boundary-abi-contract.md`
  - Requirement traceability: `docs/design/requirements-traceability.md`
  - Docs map: `docs/design/DOC-MAP.md`
- **Explanation**
  - Builder Sub-Crate Pattern rationale: `docs/design/sovereign-spec-v26.10.19.md`
  - Verification philosophy (Chicago courts, anti-vacuity):
    `docs/design/acceptance-criteria-courts.md`,
    `docs/design/verification-plan.md`
  - Court ladder + gates: `docs/design/court-ladder-spec.md`,
    `docs/design/gate-v01-latency.md`, `docs/design/gate-v02-crash-chaos.md`,
    `docs/design/gate-v03-memory-ceiling.md`

## Verification & Governance

- **One command**: `just doctor` — the 13-stage verification battery
  (`scripts/doctor.sh`, stages 00-12: preflight, storage, types, OCEL,
  latency, concurrency, revenue packs, fleet classes, and loud blocks on
  the BLOCKED surfaces). Dual-pass idempotence court: `just court-x2`.
- **Pinned source**: upstream DuckDB main SHA
  `b6013f7aa2f36caabb9466ce1791667ea8a280bb` (v2.0-cyanoptera line); the
  frozen dylib self-reports `v2.0.0-dev88595`.
- **Known divergences**: `KNOWN_DIVERGENCES.md` at the repo root — every
  SDK divergence registered with evidence and a close condition; coverage
  cells render PASS / registered-divergence / loudly-MISSING, never a
  silent skip.
- **Window standing**: the v2.0.0-dev campaign window is **PARTIAL_ALIVE**
  — stated plainly: engine identity, storage/types/latency/concurrency
  courts, OCEL, revenue packs, and fleet classes are PASS/ALIVE;
  browser/OPFS (FR-01/FR-02) surfaces are BLOCKED and never faked.
  Full standing roll-up:
  `docs/sjira/v2.0.0-dev/CAMPAIGN-RECEIPT.md`;
  per-court JSON receipts: `docs/sjira/v2.0.0-dev/receipts/`.

## Platform Support (as of 2.0.0-dev)

- **macOS**: per-architecture dylibs (arm64, x86_64). The prebuilt
  download path requires a published GitHub Release — none exists for
  `v2.0.0-dev` yet, so builds currently compile locally from pinned source
  and cache. No `DYLD_*` setup either way.
- **Linux / Windows**: pinned local-compile fallback. Prebuilt `.so`
  assets are planned.

## Built-in Extensions

Statically built into the bundled binary
(`-DBUILD_EXTENSIONS=parquet;json;icu;httpfs;tpch;tpcds;fts;inet;sqlsmith`,
plus `-DBUILD_JEMALLOC=ON` and `-DBUILD_AUTOLOAD=ON` — see
`crates/frozen-duckdb-builder/src/lib.rs`):

- **JSON**, **Parquet**, **ICU**, **HTTPFS**, **TPC-H/TPC-DS**, **FTS**,
  **INET**, **SQLSmith**, **Jemalloc**, **Autoload**

Not built in this release: Visualizer and TPC-E (out-of-tree in the
2.0.0-dev line); Excel (needs minizip-ng on the build host); VSS is not in
the static set (`crates/frozen-duckdb/tests/vss_tests.rs` detects and skips
gracefully when absent). VARIANT surface: landed — see
`docs/design/variant-shredding-design.md`.

Integration features: Arrow, serde/serde_json, chrono
(`crates/frozen-duckdb/Cargo.toml`). Not integrated: Polars, r2d2.

## Known BLOCKED Features (honest standing)

- **WebAssembly / OPFS (FR-01/FR-02)** — BLOCKED. The wasm ABI surface is
  designed (`docs/design/wasm-abi-surface.md`) and a build script exists
  (`scripts/build_wasm_duckdb.sh`), but no wasm artifact receipt exists;
  FR-01 is unflipped. The value-audit funnel's wasm leg is loud-BLOCKED in
  `docs/sjira/v2.0.0-dev/receipts/val004.json`.
- **Prebuilt release assets for 2.0.0-dev** — BLOCKED on tag/release
  publication (upstream has no `v2.0.0-dev` tag). Until published, first
  builds compile locally from pinned source.
- **Prebuilt Linux `.so` / Windows assets** — planned, not built.
- **Fleet Class-B in-browser envelope** — proxy ceiling only
  (`docs/sjira/v2.0.0-dev/receipts/fleet002_proxy.json`, PARTIAL_ALIVE);
  cgroup gate loud-skips on darwin.
- **BEAM fleet leg (Class E)** — BLOCKED-OTHER-REPO
  (`docs/sjira/v2.0.0-dev/receipts/fleet005.json`).
- Remaining PROPOSED layers (wasm scorecard leg, storage format v2,
  economic model, DMADV charter) are proposals, not landed capabilities —
  see the Blueprint table in `docs/design/` and
  `docs/sjira/v2.0.0-dev/DEFINITION-OF-DONE.md`.

## Development

```bash
git clone https://github.com/seanchatmangpt/frozen-duckdb.git
cd frozen-duckdb
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check
just doctor          # 13-stage verification battery
```

Contributing: [coding standards](docs/contributing/coding-standards.md).
No `unwrap()` in library code; comprehensive error handling; every
performance claim receipted.

### Troubleshooting

1. **"Failed to get frozen DuckDB binary"** — normal on first build: the
   builder is compiling locally; later builds use the cache.
2. **"No cached binary found"** — check `~/.frozen-duckdb/cache/`.
3. **Architecture mismatch** — the builder detects architecture in order:
   explicit single-slice `CMAKE_OSX_ARCHITECTURES` → `ARCH` → `uname -m`;
   release assets are per-architecture.
4. **Verbosity** — the CLI takes a `-v` flag (RUST_LOG has no effect);
   `cargo build -v` for verbose builds.

## License

MIT — see [LICENSE](LICENSE). A commercial enterprise license exists as a
DRAFT only (not landed):
[LICENSE-ENTERPRISE-DRAFT.md](LICENSE-ENTERPRISE-DRAFT.md).

## Final 2.0.0 bump (operator checklist)

When upstream cuts the `v2.0.0-dev` tag, the crate version flips via a
mechanical, scripted edit set — no hand-editing of pins. Five steps:

1. Run the flip script with the new version + upstream SHA:
   `scripts/fastfollow_flip.sh OLD_VER NEW_VER NEW_SHA` — it applies the
   curated edit set (Cargo pins + lock, builder `VERSION`/`SOURCE_PIN`,
   workflow matrices, ggen.toml, schema TTLs, scripts, GENESIS docs) and
   fails closed on any missing file or failed sed.
2. Regenerate projections from their sources (never hand-edit them):
   `make sync && make gates` — Makefile, verify scripts, and gate shapes
   re-render via `ggen sync run`.
3. Run the full gate: `make verify` (verify-gates + ggen receipt verify)
   plus the test ladder (`cargo test --workspace`, clippy `-D warnings`,
   `fmt --check`).
4. Commit per lane and follow the rehearsed cut/publish sequence:
   `docs/sjira/v2.0.0-dev/TAG-REHEARSAL.md` (tag → release assets →
   builder → sys → crate publishes, with index-wait polls and an abort
   doctrine).
5. Close out with post-publish verification per Step 10 of
   TAG-REHEARSAL.md and update the window receipts.

Note: the upstream DuckDB 2.0 release calendar (Oct 21) is the upstream
schedule, not a commitment date for this crate's final release — the flip
happens when the upstream tag exists and the gates pass.
