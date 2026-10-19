# blueprint-ci

Design for the blueprint CI extension: a nightly browser burn-in workflow and a
docs-conformance workflow over the blueprint ontology. DESIGN ONLY — no
`.github/workflows/` edits in this lane. Admitting workflow files is a separate
admitted transition that happens only after the BP2 CDP harness exists and
passes its courts locally.

## Status

DESIGN / UNVERIFIED-UNTIL-HARNESS. Nothing here has executed. The YAML below is
a sketch to be validated against the harness, not an admitted artifact.

## Scope

Two workflows, both triggered on schedule plus `workflow_dispatch`:

1. `blueprint-burnin.yml` — nightly chrome-headless-shell burn-in driving the
   wasm build through the BP2 courts
   (`docs/design/chrome-burnin-court.md`).
2. `blueprint-conformance.yml` — Turtle parse of `schema/blueprint.ttl` plus
   SHACL validation via V-G1 (`docs/design/verification-plan.md`).

Out of scope: tag-push binary builds (`build-binaries.yml` stays as-is), the
`make verify` native battery (V-G2, unchanged).

## In-repo precedent for fail-closed versioning

`build-binaries.yml` "Guard version input" step: a `workflow_dispatch` for any
version other than the builder pin (`const VERSION` in
`crates/frozen-duckdb-builder/src/lib.rs`) exits 1 with a loud message instead
of silently building the pinned version. That is the pattern both workflows
follow: an input/artifact that lies about capability is a loud failure, never a
quiet fallback. The burn-in workflow applies the same law to artifacts: a court
that produces no verdict artifact is a FAIL, not a skip.

## Fail-closed rule

For both workflows, every expected artifact is required:

- Missing or unparseable verdict JSON for any court => the job fails with
  `FAIL (missing artifact: <name>)`. There is no "skip if absent" path and no
  `if: always()` swallowing a missing file.
- A court process that crashes before emitting its verdict emits no artifact;
  the aggregation step sees the gap and exits nonzero. Silence is never PASS.
- Chrome launch failure, harness exception, or timeout => FAIL with the typed
  reason in the job summary. `timeout-minutes` turns a hang into a bounded
  failure (same law as `build-binaries.yml`'s 150 and `ci.yml`'s 45).

## Job matrix

### blueprint-burnin.yml (nightly burn-in)

Sketch — UNVERIFIED-UNTIL-HARNESS:

```yaml
# UNVERIFIED-UNTIL-HARNESS — sketch only, not an admitted workflow
name: Blueprint Burn-in
on:
  schedule:
    - cron: "0 7 * * *"   # nightly 07:00 UTC
  workflow_dispatch:

jobs:
  burnin:
    runs-on: ubuntu-latest
    timeout-minutes: 60
    strategy:
      fail-fast: false
      matrix:
        court: [crash-wal-replay, quota-exhaustion, worker-lock-race]
    steps:
      - uses: actions/checkout@v4

      - name: Set up Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build wasm + harness
        run: make wasm-build   # UNVERIFIED-UNTIL-HARNESS: target name TBD by BP2

      # Same guard law as build-binaries.yml: fail if the pin lies.
      - name: Guard harness pin
        run: scripts/wasm/burnin_guard.sh   # compares harness/court IDs against
                                             # the BP2 court registry; any
                                             # mismatch exits 1

      - name: Install chrome-headless-shell
        run: scripts/wasm/fetch_chrome_headless_shell.sh --pin

      - name: Run court
        run: |
          node scripts/wasm/burnin.mjs \
            --court "${{ matrix.court }}" \
            --out "verdicts/${{ matrix.court }}.json"

      - name: Check verdict artifact exists (fail-closed)
        if: always()
        run: |
          set -euo pipefail
          f="verdicts/${{ matrix.court }}.json"
          if [[ ! -s "$f" ]]; then
            echo "FAIL (missing artifact: $f) — silence is never PASS" >&2
            exit 1
          fi
          python3 scripts/wasm/verdict_check.py "$f" \
            --court "${{ matrix.court }}"   # exits 1 on verdict != PASS

      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: verdict-${{ matrix.court }}
          path: verdicts/

  aggregate:
    needs: burnin
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
      - name: Aggregate verdicts
        run: python3 scripts/wasm/aggregate_verdicts.py verdicts/ \
               --out receipt.json --summary SUMMARY.md
        # Exits 1 if any court is missing from the receipt — fail-closed at
        # the aggregation layer too.
      - uses: actions/upload-artifact@v4
        with:
          name: burnin-receipt
          path: receipt.json
```

### blueprint-conformance.yml (docs graph)

Sketch — UNVERIFIED-UNTIL-HARNESS:

```yaml
# UNVERIFIED-UNTIL-HARNESS — sketch only, not an admitted workflow
name: Blueprint Conformance
on:
  schedule:
    - cron: "30 7 * * *"
  workflow_dispatch:
  pull_request:
    paths: ["docs/design/*.ttl", "schema/blueprint.ttl"]
  push:
    branches: [master]
    paths: ["schema/blueprint.ttl"]

jobs:
  conformance:
    runs-on: ubuntu-latest
    timeout-minutes: 15
    steps:
      - uses: actions/checkout@v4

      - name: Sync generated surface
        run: make sync   # ggen sync run; regenerates projections before check

      - name: Turtle parse + SHACL (V-G1)
        run: python3 scripts/validate_blueprint_graph.py \
               schema/blueprint.ttl docs/design/

      - name: Genesis drift check
        run: make genesis-check

      - name: Emit verdict receipt
        if: always()
        run: |
          set -euo pipefail
          python3 scripts/conformance_receipt.py \
            --schema schema/blueprint.ttl \
            --out verdicts/conformance.json   # exits 1 on any FAIL row

      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: conformance-verdict
          path: verdicts/conformance.json
```

## Artifact and receipt outputs

Every court run produces one JSON verdict artifact per court, written by the
harness (never hand-assembled):

```json
{
  "court": "crash-wal-replay",
  "standing": "PASS",
  "subject": { "wasm_sha256": "...", "harness_sha256": "..." },
  "iterations": 40,
  "asserts": 312,
  "started_at": "...", "finished_at": "...",
  "falsifier": "Page.crash mid-transaction leaves OPFS state unopenable"
}
```

- `standing` vocabulary: `PASS | FAIL | VACUOUS` — `VACUOUS` when the court
  passed under its named mutation, per the shim-court non-vacuity law.
- The aggregate `burnin-receipt.json` carries one row per court plus the run
  SHA; a court with no row is a receipt-level FAIL.
- Conformance emits one `conformance.json` with one row per `.ttl` file
  (parse status + SHACL result). A file with no row is FAIL.

## Non-vacuity

Each court's named mutation (from `chrome-burnin-court.md`) is run on a
schedule (e.g. weekly dispatch) so a court that silently stopped being able to
fail is detected. The conformance workflow inherits V-G1's two shims: an
off-vocabulary status triple and a malformed Turtle line must both exit
nonzero.

## Admitting these workflows (follow-on, not this lane)

Sequence when the harness exists: (1) harness + courts land in
`scripts/wasm/` with local PASS receipts; (2) workflows are added as an
admitted transition with one green `workflow_dispatch` run each; (3) the
fail-closed check step is proved non-vacuous by a `SHIM-COURT` run with the
verdict file deleted — the workflow must FAIL on the gap.
