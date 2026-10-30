# Release Draft — frozen-duckdb v2.0.0 (PREPARED INPUT, NOT THE ACT)

Authority note: creating even a *draft* GitHub Release is operator
authority. This file is the prepared input for that act — the `gh
release create` command below is NOT executed by lane GH3. It runs only
after a green `build-binaries` workflow_dispatch produces the two assets.

## 1. Transport receipts (executed 2026-10-19)

- `gh auth status` → exit 0; logged in as `seanchatmangpt` (github.com),
  scopes include `repo`, `workflow`. Active account: true.
- `gh repo view --json nameWithOwner` → `seanchatmangpt/frozen-duckdb`.
  Target repo confirmed.

## 2. Draft release body (notes-file content)

-----BEGIN NOTES FILE-----

# frozen-duckdb v2.0.0 (prerelease)

Bundled DuckDB **2.0.0**, pinned to upstream `v2.0-cyanoptera` SHA
`d54cbbe3df17e8e5ce68312f483b4155836112aa` (short `b6013f7`). No upstream
`v2.0.0` tag exists yet; this pin is upstream main at the named SHA and
moves with it until the final tag lands. Standing: **PARTIAL_ALIVE** —
every claim names its receipt (`docs/sjira/v2.0.0/CAMPAIGN-RECEIPT.md`,
Wave 14 final reconciliation, 2026-10-19).

## Highlights

- **Bundled DuckDB 2.0.0** — engine dylib self-reports
  `v2.0.0` (strings probe, WITNESSED). Built from pin
  `b6013f7`; the upstream final tag swaps the pin when it lands.
- **VARIANT type surface** — doctor stage `03_types_variant` 3/3 green.
- **Stateful poll/pagination warehouse-sim** — KD-03/04 closed; journeys
  settle SUCCEEDED over real HTTP 4/4.
- **13-stage doctor battery** — `just doctor`, stages 00–12, fail-closed;
  currently 13-stage (6 green, remaining KD-mapped) plus loud named blocks. Never a faked pass.
- **Value-audit battery** — real cost-avoidance courts on live receipts
  (`receipts/val001..004.json`).

## Audited numbers (receipt-backed only)

Receipt dir: `docs/sjira/v2.0.0/receipts/` (R). DISPATCH-CITED = cited
from `CAMPAIGN-RECEIPT.md` with owning artifact verified on disk.

| Metric | Value | Receipt |
|---|---|---|
| Dylib size (arm64) | 65.9 / 52.4 MB unstr. / str. | `prebuilt/README.md:21,25` |
| Ingest throughput (release) | 12.16M rows/s | DISPATCH-CITED |
| Query latency p95 | 10.51 ms (budget < 15 ms) | `R/gate_v01_receipt.json` |
| OCEL compress / query | 94.93% / 97.01% | DISPATCH-CITED |
| Network egress | 0 B (by construction, no transport) | `R/val004.json` |
| Fleet Class-D compression | 99.76% | DISPATCH-CITED; `R/fleet004.json` |
| OOM behavior | typed abort; SELECT 42 post-OOM | `docs/how-to/how-to-memory-caps.md` |
| Crash-chaos | 3 kill legs + no-op-kill flip DETECTED | PARTIAL_ALIVE, DISPATCH-CITED |

## Breaking changes

- **Prerelease cycle** pinned to upstream main `b6013f7`; semver caveats
  apply until a tagged upstream release.
- **Unpublishable dev versions** — crates.io `frozen-duckdb` is still at
  `0.1.0`; consume the `2.0.0` line from this repository.
- **`flock` extension unavailable at the dev pin** (404 at the
  community-extension registry); tests `#[ignore]`d with the 404
  verbatim (KD-02).
- **Storage-version gates** — nested-schema storage requires storage
  version v2.0.0; 1.5.x databases need the export/recreate path for
  nested schemas (`docs/design/storage-format-v2.md`).

## Known divergences (KD register summary)

Full register: `KNOWN_DIVERGENCES.md`. 3 of 14 closed (KD-03/04/14); 11 open.
Open: KD-01 (21 C++-mangled `__Z*` exports breach C-ABI-only contract),
KD-02 (flock 404), KD-05 (no wasm/OPFS), KD-06 (host-range VFS design-only),
KD-07 (REV-001 wall 4.23–6.05s under load), KD-08 (allocator-stats C API
absent), KD-09 (cgroup leg needs Linux+root), KD-10 (BEAM leg in smalt),
KD-11 (browser substrate courts blocked), KD-12 (anomaly exact-match false),
KD-13 (cargo registry corruption, repaired).

## Verification

```bash
just doctor        # 13-stage fail-closed battery (stages 00-12)
just doctor-quick  # preflight stage only
just doctor-court  # dual-pass idempotence court (TEST-025)
```

Witnessed state: 13-stage (6 green, remaining KD-mapped); stages 11/12 green on every
executable leg with loud named blocks where hardware/substrate is absent.
Overall standing PARTIAL_ALIVE. Blocks are loud and named, never faked.

## Install

No prebuilt binaries are attached to this release. Build from the pinned
source — the builder crate resolves `libduckdb_{arch}.dylib` from the
release assets when present and otherwise compiles DuckDB locally from
the same `b6013f7` pin (30–90 min on first build; cached under
`~/.frozen-duckdb/cache/v2.0.0-{arm64|x86_64}/`):

```toml
[dependencies]
duckdb = { git = "https://github.com/seanchatmangpt/frozen-duckdb", tag = "v2.0.0", package = "frozen-duckdb" }
```

## Upgrade path (from 1.5.5)

- API-identical drop-in `duckdb-rs` replacement — `MIGRATION_GUIDE.md`
  ("Quick Migration (2 minutes)").
- 1.5.x database files: use `EXPORT DATABASE` / import into a
  v2.0.0-storage-version DB for nested-schema workloads.
- Pin via the builder cache slot to freeze the dev pin:
  `~/.frozen-duckdb/cache/v2.0.0-arm64/`.

See Also: `CAMPAIGN-RECEIPT.md` · `KNOWN_DIVERGENCES.md` ·
`MIGRATION_GUIDE.md` · `docs/design/doctor-implementation-map.md`

-----END NOTES FILE-----

## 3. Exact release command (operator-run, after green build-binaries)

Write the body between the markers above to `/tmp/notes-v2.0.0.md`,
then:

```bash
gh release create v2.0.0 \
  --draft \
  --prerelease \
  --title "frozen-duckdb v2.0.0 (DuckDB v2.0-cyanoptera @ b6013f7)" \
  --notes-file /tmp/notes-v2.0.0.md \
  seanchatmangpt/frozen-duckdb
```

Expected assets (upload to the draft once build-binaries is green):

- `libduckdb_arm64.dylib`
- `libduckdb_x86_64.dylib`

Asset law: per-arch names `libduckdb_{arch}.dylib`, arch
`x86_64|arm64` (aarch64 normalized to arm64). Since DuckDB v1.5.x macOS
upstream ships only the universal `libduckdb-osx-universal.zip`; the
per-arch name is a symlink to the universal dylib — do not "fix" the
platform string to `osx-arm64`/`osx-amd64`.

Post-asset verification checklist (run per downloaded asset):

```bash
shasum -a 256 libduckdb_arm64.dylib libduckdb_x86_64.dylib
# cross-check against build-binaries job log sha256sums
lipo -info libduckdb_arm64.dylib    # expect: arm64
lipo -info libduckdb_x86_64.dylib   # expect: x86_64
```

Then publish the draft (operator act) and re-run
`python3 scripts/verify_publish_receipt.py` from repo root: exit 0 = GREEN,
1 = RED gate failure, 2 = unusable (missing receipt matrix render or ggen CLI).

## 4. Status

BLOCKED-BY-DESIGN for execution: this lane's deliverable is this file.
The `gh release create` act requires (a) operator authority and (b) a
green build-binaries dispatch, in that order.
