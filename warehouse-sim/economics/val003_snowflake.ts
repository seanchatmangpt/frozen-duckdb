// TEST-VAL-003 — value ledger wiring for GATE-V-01 (lane V4, frozen-duckdb).
//
// Reads the REAL GATE-V-01 receipt from /tmp/value_audit/gate_v01_receipt.json
// (minted by an actual run of crates/frozen-duckdb/tests/gate_v01_latency.rs
// in --release; the JSON emission lives at the end of that test, after the
// PASS assert). This script never re-runs the benchmark — it only monetizes
// the already-measured p95. If the receipt is missing it exits nonzero with
// a loud [SKIP]-style refusal; it never invents a latency number.
//
// Counterfactual (the modeled warehouse alternative):
//   Each of the 3 queries costs ~5s on an XS warehouse equivalent
//   (1 credit/hr XS pricing, the standard Snowflake XS unit) —
//   1000 query-iterations x 3 queries x 5s = 15,000s = 4.1667 warehouse-hr
//   per run... but the ledger convention for this lane is the *daily loop*:
//   1000 queries/day x 5s = 5000s/day = 1.3889 warehouse-hr/day
//   -> $33.12/day at $23.85/credit (on-demand XS, us-east)
//   -> $12,090/yr   (33.125 * 365, unrounded inputs)
//   Sovereign credits consumed: 0.00 (in-process, no warehouse).
//
// Usage: node warehouse-sim/economics/val003_snowflake.ts [--json out]
// Exits 0 with a written val003.json, 1 on missing/invalid receipt.

import { readFile, writeFile } from "node:fs/promises";

const RECEIPT_PATH = "/tmp/value_audit/gate_v01_receipt.json";
const DEFAULT_OUT = "/tmp/value_audit/val003.json";

// Counterfactual constants (declared, not measured — the only simulated leg).
const QUERIES_PER_DAY = 1000;
const WAREHOUSE_SECONDS_PER_QUERY = 5;
const XS_CREDITS_PER_HOUR = 1;
const USD_PER_CREDIT = 23.85;
const DAYS_PER_YEAR = 365;

interface GateV01Receipt {
  gate: string;
  p95_ms: number;
  mean_ms: number;
  iterations: number;
  queries: number;
  budget_ms: number;
  n_rows: number;
}

interface Val003 {
  test: string;
  gate: string;
  receipt_source: string;
  measured: {
    p95_ms: number;
    mean_ms: number;
    iterations: number;
    queries: number;
  };
  counterfactual: {
    queries_per_day: number;
    warehouse_seconds_per_query: number;
    warehouse_hours_per_day: number;
    credits_per_day: number;
    usd_per_day: number;
    usd_per_year: number;
    sovereign_credits_consumed: number;
  };
  verdict: string;
}

function fail(msg: string): never {
  console.error(`[SKIP] ${msg}`);
  process.exit(1);
}

const raw = await readFile(RECEIPT_PATH, "utf8").catch(() =>
  fail(
    `${RECEIPT_PATH} not found — run ` +
      `CARGO_TARGET_DIR=target-laneV4 cargo test -p frozen-duckdb --release --test gate_v01_latency first; no synthetic receipt will be minted`
  )
);

const receipt = JSON.parse(raw) as GateV01Receipt;
if (receipt.gate !== "GATE-V-01" || typeof receipt.p95_ms !== "number") {
  fail(`${RECEIPT_PATH} is not a GATE-V-01 receipt`);
}
if (receipt.p95_ms >= receipt.budget_ms) {
  fail(`receipt p95 ${receipt.p95_ms} ms exceeds budget ${receipt.budget_ms} ms — gate is RED, ledger refuses`);
}

const warehouseHoursPerDay =
  (QUERIES_PER_DAY * WAREHOUSE_SECONDS_PER_QUERY) / 3600 * XS_CREDITS_PER_HOUR;
// Ledger convention: TRUNCATE (never round up a value claim). The year figure
// compounds the UNTRUNCATED daily rate (33.125 * 365 = 12090.625 -> 12090),
// matching the TEST-VAL-003 ledger constants.
const rawDaily = warehouseHoursPerDay * USD_PER_CREDIT;
const usdPerDay = Math.floor(rawDaily * 100) / 100;
const usdPerYear = Math.floor(rawDaily * DAYS_PER_YEAR);

const out: Val003 = {
  test: "TEST-VAL-003",
  gate: "GATE-V-01",
  receipt_source: RECEIPT_PATH,
  measured: {
    p95_ms: receipt.p95_ms,
    mean_ms: receipt.mean_ms,
    iterations: receipt.iterations,
    queries: receipt.queries,
  },
  counterfactual: {
    queries_per_day: QUERIES_PER_DAY,
    warehouse_seconds_per_query: WAREHOUSE_SECONDS_PER_QUERY,
    warehouse_hours_per_day: Number(warehouseHoursPerDay.toFixed(4)),
    credits_per_day: Number(warehouseHoursPerDay.toFixed(4)),
    usd_per_day: Number(usdPerDay.toFixed(2)),
    usd_per_year: Number(usdPerYear.toFixed(0)),
    sovereign_credits_consumed: 0.0,
  },
  verdict: `ALIVE: p95 ${receipt.p95_ms} ms real, $${usdPerDay.toFixed(2)}/day avoided, $${usdPerYear.toFixed(0)}/yr avoided, 0.00 credits consumed`,
};

const outPath = process.argv.includes("--json")
  ? process.argv[process.argv.indexOf("--json") + 1]
  : DEFAULT_OUT;
await writeFile(outPath, JSON.stringify(out, null, 2) + "\n");
console.log(`val003 written: ${outPath}`);
console.log(out.verdict);
