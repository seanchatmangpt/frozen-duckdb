# DG5-chain — crates.io serial chain rehearsal (P1, no token actuation)

Date: 2026-10-19 · Lane DG5 · Subject: master cbeb249 (working tree)
Standing: **READY-EXCEPT-TOKEN**

## Steps executed (real output, this run)

| step | command | result |
|---|---|---|
| publish-flag refusal | `cargo publish --dry-run --allow-dirty -p test-validation` | exit 101 — `error: \`test-validation\` cannot be published. \`package.publish\` must be set to \`true\` or a non-empty list in Cargo.toml to publish.` |
| builder dry-run | `cargo publish --dry-run --allow-dirty -p frozen-duckdb-builder` | **exit 0** — `Uploading frozen-duckdb-builder v2.0.0-dev … warning: aborting upload due to dry run` (witnessed, matches prior) |
| chain plan | `DRYRUN= bash scripts/publish_chain.sh` (default DRYRUN=1) | exit 0, standing `dry-run-verified` |
| secret-name check | `grep CARGO_REGISTRY_TOKEN .github/workflows/publish-crates.yml` | all 3 publish jobs bind `secrets.CARGO_REGISTRY_TOKEN` (lines 97/141/185), each fail-closed `REFUSED[NO_TOKEN]`, environment `release` |

## Chain-order confirmation (3)

`publish_chain.sh` DRYRUN output, verbatim — order builder → wait → sys →
wait → main, bounded index polls between hops (`wait_for_index`:
POLL_TIMEOUT=300s default, backoff 2s→30s, classified SLEEP-CLASS: POLL-STEP):

```
DRYRUN: cargo publish --manifest-path crates/frozen-duckdb-builder/Cargo.toml
DRYRUN: wait_for_index frozen-duckdb-builder 2.0.0-dev
DRYRUN: cargo publish --manifest-path crates/frozen-duckdb-sys/Cargo.toml
DRYRUN: wait_for_index frozen-duckdb-sys 2.0.0-dev
DRYRUN: cargo publish --manifest-path crates/frozen-duckdb/Cargo.toml
DRYRUN: python3 scripts/verify_publish_receipt.py
DRYRUN: sparse-index check (skip — no real publish performed)
```

Matches the script's real-mode branch exactly (`publish_crate` →
`wait_for_index` interleaved).

## LIVE run procedure (4)

Prerequisite (OPERATOR, one-time): `cargo login <token>` — operator authority,
never executed in-lane. The script's own credential check is existence-level
only (`cargo login --help`); the token is never read, written, or echoed.

Operator executes:

```bash
DRYRUN=0 PUBLISH_CONFIRM=yes bash scripts/publish_chain.sh
```

Exact commands the script will execute in real mode, in order (each real
publish is preceded by an advisory per-crate dry-run
`cargo publish --manifest-path <manifest> --dry-run 2>/dev/null || true`):

```bash
cargo publish --manifest-path crates/frozen-duckdb-builder/Cargo.toml
# wait_for_index frozen-duckdb-builder 2.0.0-dev
#   bounded poll: curl https://crates.io/api/v1/crates/frozen-duckdb-builder
#   (JSON version-list probe; backoff 2s→30s, REFUSED on 300s timeout)
cargo publish --manifest-path crates/frozen-duckdb-sys/Cargo.toml
# wait_for_index frozen-duckdb-sys 2.0.0-dev
cargo publish --manifest-path crates/frozen-duckdb/Cargo.toml
# then: python3 scripts/verify_publish_receipt.py
# then: wait_for_index for all three crates (sparse-index verification)
```

## Post-publish verification list (6)

Sparse index URLs, each must list 2.0.0-dev (HTTP 200 with a `"vers"` entry
`"2.0.0-dev"`):

- https://index.crates.io/fr/oz/frozen-duckdb-builder
- https://index.crates.io/fr/oz/frozen-duckdb-sys
- https://index.crates.io/fr/oz/frozen-duckdb

Equivalently via the API (what the script polls):
`https://crates.io/api/v1/crates/frozen-duckdb{,-sys,-builder}` — JSON
`versions[].num` must contain `2.0.0-dev` for all three.

## Operator handoff block

```
OPERATOR-ONLY STEPS (token-gated actuation — outside lane authority):

1. crates.io -> me -> new token (publish-only scope, frozen-duckdb trusted crate? no — fresh publish, no trusted publishing configured)
2. cargo login <token>
3. DRYRUN=0 PUBLISH_CONFIRM=yes bash scripts/publish_chain.sh
4. On RED: REFUSED[Index_TIMEOUT] -> retry after index propagation; any other
   failure -> consult docs/sjira/v2.0.0-dev/receipts/DG5-chain.md

The lane does NOT hold the token and does NOT run step 3.
```

## Findings (non-blocking)

- `u2-probe` (workspace member) carries **no `publish = false`** —
  `cargo publish --dry-run -p u2-probe` exits 0. Not in the publish chain
  (`-p` targets only the three crates), but flagging so it is either stamped
  `publish = false` (as `test-validation` is) or removed from members before
  the live run.
- `publish-crates.yml` validate job REFUSES any `-dev` version suffix
  (`REFUSED[DEV_SUFFIX]`) — the workflow path can only publish a
  de-suffixed release version; the `2.0.0-dev` serial chain is
  script-driven (`publish_chain.sh`) only.