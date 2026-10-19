# warehouse-sim

Ontology-driven behavioral + economic simulator of the Snowflake and Databricks
SQL APIs, used as a **Chicago test fixture** for the frozen-duckdb campaign: a
real running HTTP surface whose responses are generated from an RDF graph, with
zero cloud dependency in CI.

It is NOT a mock library. It is a real Next.js server that renders its route
handlers from `schema/warehouse_sim.ttl` via ggen, so provider parity (paths,
handle params, 202-vs-200 submit semantics, poll state machines, cold-start
timing, cost constants) is one graph compared against two provider profiles —
N implementations of one calculus, no O(N^2) drift.

## Standing

Every cost constant in `schema/warehouse_sim.ttl`
(`creditCostPerHour 4.0`, `dcuCostPerHour 0.22`, `scanRatePerTB 5.0`) is an
**ASSUMPTION-UNVERIFIED counterfactual parameter**. They drive the metering
formulas (`InvoiceUsd = DBUsConsumed x dcuCostPerHour + BytesScanned x
$5/TB`) and the economics court; they are not measured prices and must not be
quoted as such. Verifying them is the job of the two-arm measurement harness
in `docs/design/economic-model.md`, which has not run.

## Layout

- `schema/warehouse_sim.ttl` — SOURCE OF TRUTH. Cloud warehouse endpoints
  (`sim:SnowflakeSqlApi`, `sim:DatabricksSqlApi`) modeled against
  `sim:SovereignBaseline` (in-process DuckDB: egress 0, invoice $0.00,
  latency budget 15ms), poll states, cold-start/jitter, metering formulas.
- `ggen.toml` — STANDALONE manifest rendering `lib/sim_engine.ts` and the
  API routes from the graph. NOT the root repo manifest.
- `templates/*.tera` — ggen templates (the rendered output's only source).
- `lib/sim_engine.ts` — GENERATED from the graph; never hand-edited.
- `app/api/...` — GENERATED route handlers (never hand-edited).
- `courts/determinism.sh` — two-pass render court (byte-identical or FAIL).
- `economics/sovereign_probe/` — Rust probe: the REAL engine (frozen-duckdb
  crate) running the rev2 three-query workload (seed 20261019, 100k rows)
  with `egress_bytes 0`, `invoice_usd 0.00`; the Path-B leg of the economics
  court.
- `node_modules/` — Next.js 14.2.15 / React 18.3.1 toolchain (installed).

## How to run

```bash
cd warehouse-sim
ggen sync run        # render sim_engine + routes from the ontology
npm install          # already installed here; fresh clone needs it
npm run dev          # Next dev server on :3000
npm run typecheck
bash courts/determinism.sh   # two-pass render; byte-identical or FAIL
cargo run --release --manifest-path economics/sovereign_probe/Cargo.toml
```

## The GENERATED-files law

`lib/sim_engine.ts` and `app/api/**` are rendered consequences of
`schema/warehouse_sim.ttl`. Edit the graph, re-run `ggen sync run`, and the
routes re-render. Hand-editing a generated file is a SUSPECT receipt and the
determinism court will flag it (a non-empty render diff proves the templates
stopped being the source of truth).

## Standing of the build-out (2026-10-19, as found on disk)

- LANDED: Next scaffold (`package.json`, `next.config.mjs`, `tsconfig.json`,
  `next-env.d.ts`, `.gitignore`, `node_modules/`); `schema/warehouse_sim.ttl`
  (sim: ontology, ASSUMPTION-UNVERIFIED constants);
  `ggen.toml` + `templates/` (G2); `lib/sim_engine.ts` + `lib/sim_engine.test.ts`;
  `courts/determinism.sh`; `economics/sovereign_probe/`.
- NOT LANDED: `app/api/` routes (ggen render currently fails — see below).
- Court standing (as witnessed 2026-10-19): `bash courts/determinism.sh`
  logs `[SKIP] ggen sync run --dry-run failed (rc=1); G2 render not usable`
  — the `snowflake-submit-route` rule fails with `FM-GEN-008
  TEMPLATE_VARIABLE_MISSING: Variable r.endpointLabel not found in context`
  (`templates/snowflake_submit_route.ts.tera`). Court skip is loud; no PASS
  is claimed. Economics court runner (three fail-closed assertions): not on
  disk as a single entrypoint; only the sovereign probe leg exists. No ALIVE
  claim is made for any sim surface.
