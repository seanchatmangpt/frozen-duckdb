# Release Notes — frozen-duckdb 2.0.0-dev

Release channel: prerelease `2.0.0-dev`, pinned to upstream DuckDB
`v2.0-cyanoptera` SHA `b6013f7aa2f36caabb9466ce1791667ea8a280bb` (short
`b6013f7`). No upstream `v2.0.0` tag exists yet; the shipped pin is
upstream main at the named SHA. Standing: **PARTIAL_ALIVE** — every claim
below names its receipt; DISPATCH-CITED numbers come from
`CAMPAIGN-RECEIPT.md` (Wave 14 final reconciliation, 2026-10-19).

## Highlights

- **Bundled DuckDB 2.0.0-dev.** Engine dylib self-reports
  `v2.0.0-dev88595` (strings probe, WITNESSED,
  `CAMPAIGN-RECEIPT.md` §"Engine flip (F10)"). Built from the
  `v2.0-cyanoptera` pin `b6013f7`; the upstream final tag swaps the pin
  when it lands.
- **VARIANT type surface.** Doctor stage `03_types_variant` 3/3 green
  (DISPATCH-CITED, `CAMPAIGN-RECEIPT.md` §"Court battery finals").
- **Stateful poll/pagination warehouse-sim.** Rendered sim routes are
  stateful (KD-03/04 closed); journeys settle SUCCEEDED over real HTTP
  4/4 (DISPATCH-CITED, `CAMPAIGN-RECEIPT.md`).
- **13-stage doctor battery.** `just doctor` runs stages 00–12
  fail-closed (`scripts/doctor.sh`; stages 11/12 wired). Currently
  6/11 core stages green plus loud named blocks — never a faked pass
  (DISPATCH-CITED, `CAMPAIGN-RECEIPT.md` §"Standing roll-up").
- **Value-audit battery.** Real cost-avoidance courts on live receipts:
  p95 latency, egress, funnel, and anomaly recall
  (`receipts/val001..004.json`).

## Audited numbers (receipt-backed only)

Receipt dir: `docs/sjira/v2.0.0-dev/receipts/` (R below). DISPATCH-CITED
means cited from `CAMPAIGN-RECEIPT.md` with the owning artifact verified
on disk; the rest are court JSON materialized on disk.

| Metric | Value | Receipt |
|---|---|---|
| Dylib size (arm64) | 65.9 / 52.4 MiB unstr. / str. | `prebuilt/README.md:21,25` |
| Ingest throughput (release) | 12.16M rows/s | DISPATCH-CITED: `CAMPAIGN-RECEIPT.md` |
| Query latency p95 | 10.51 ms (budget < 15 ms) | `R/gate_v01_receipt.json` (10.5088, ALIVE) |
| OCEL compress / query | 94.93% / 97.01% | DISPATCH-CITED: `CAMPAIGN-RECEIPT.md` |
| Network egress | 0 B (by construction, no transport) | `R/val004.json` (`egress_bytes: 0`) |
| Fleet Class-D compression | 99.76% | DISPATCH-CITED; `R/fleet004.json` |
| OOM behavior | typed abort; SELECT 42 post-OOM | `docs/how-to/how-to-memory-caps.md`; X9 cited |
| Crash-chaos | 3 kill legs + no-op-kill flip DETECTED | PARTIAL_ALIVE, DISPATCH-CITED |

## Breaking changes

- **Prerelease cycle.** Version `2.0.0-dev` is a prerelease line pinned
  to upstream main `b6013f7`; semver caveats apply until a tagged
  upstream release (README "Quick Start" caveat).
- **Unpublishable dev versions.** The crates.io `frozen-duckdb` package
  is still at `0.1.0`; the `2.0.0-dev` line is consumed from this
  repository until crates.io publication lands
  (`docs/how-to/usage-examples.md:17-18`).
- **`flock` extension unavailable at the dev pin.** No upstream build
  exists for `b6013f7` (HTTP 404 at the community-extension registry);
  flock tests are `#[ignore]`d with the 404 verbatim (KD-02,
  `KNOWN_DIVERGENCES.md`).
- **Storage-version gates.** Nested-schema storage is gated behind
  storage version v2.0.0 by the dev runtime; a DB attached without an
  explicit v2.0.0 storage version is rejected for nested-schema usage
  (`docs/design/storage-format-v2.md:14,27-28`). 1.5.x databases need
  the export/recreate path for nested schemas.

## Known divergences (KD register summary)

Full register with evidence paths and close conditions:
`KNOWN_DIVERGENCES.md`. 3 of 14 closed (KD-03/04/14); 11 open.

| id | one-liner | status |
|---|---|---|
| KD-01 | 21 C++-mangled `__Z*` exports breach C-ABI-only contract | open |
| KD-02 | flock extension 404 at dev pin; tests `#[ignore]`ed | open |
| KD-03 | warehouse-sim poll routes were static; now stateful | CLOSED |
| KD-04 | generated routes exported helpers; moved to lib | CLOSED |
| KD-05 | no `frozen_duckdb.wasm` (FR-01); wasm/OPFS legs blocked | open |
| KD-06 | `host-range://` VFS design-only, GATE-V-05 UNVERIFIED | open |
| KD-07 | REV-001 wall 4.23–6.05s vs <2.5s clause (under load) | open |
| KD-08 | allocator-stats C API absent; leak clause UNSUPPORTED | open |
| KD-09 | Class B cgroup leg needs Linux+root; darwin proxy only | open |
| KD-10 | Class E BEAM actor leg lives in smalt (other repo) | open |
| KD-11 | browser substrate courts FR-01/FR-02-blocked | open |
| KD-12 | anomaly exact-match false (recall 1.0 OBSERVED only) | open |
| KD-13 | cargo registry corruption by cleanup actor; repaired | open |
| KD-14 | shared cache write race; atomic temp+rename landed | CLOSED |

## Verification

```bash
just doctor        # 13-stage fail-closed battery (stages 00-12)
just doctor-quick  # preflight stage only
just doctor-court  # dual-pass idempotence court (TEST-025)
```

Current witnessed state: 6/11 core stages green; stages 11 (revenue,
REV-001..004) and 12 (fleet A–E) green on every executable leg with
loud blocks echoed where hardware/substrate is absent (Class B cgroup
rc=2 on darwin, BEAM leg BLOCKED-OTHER-REPO); host-VFS and browser/OPFS
stages exit loud BLOCKED (FR-01/FR-02). Overall standing PARTIAL_ALIVE
(`CAMPAIGN-RECEIPT.md` frontmatter and final History basis). Blocks are
loud and named, never faked.

## Upgrade path (from 1.5.5)

- Dependency swap is API-identical: same `Connection`/`Statement`/`Row`
  surface, drop-in `duckdb-rs` replacement — see `MIGRATION_GUIDE.md`
  ("Quick Migration (2 minutes)").
- 1.5.x database files: the dev engine applies storage-version gates;
  use `EXPORT DATABASE` / import into a v2.0.0-storage-version DB for
  nested-schema workloads (`docs/design/storage-format-v2.md` upgrade
  sequence).
- Prerelease pin moves with upstream main until the `v2.0.0` tag; pin
  via the builder's cache slot if you need to freeze
  (`~/.frozen-duckdb/cache/v2.0.0-dev-arm64/`).

See Also: `CAMPAIGN-RECEIPT.md` (canonical campaign receipt);
`KNOWN_DIVERGENCES.md`; `MIGRATION_GUIDE.md`;
`docs/design/doctor-implementation-map.md`
