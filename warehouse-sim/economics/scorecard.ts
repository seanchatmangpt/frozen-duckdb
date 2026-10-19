// Per-workload-class disruption scorecard — lane S5 (frozen-duckdb repo).
//
// Extends the G5 economics court (economics/court.ts — reused, not forked:
// same real-HTTP Path A runner shape, same sim-generated ledger/state-machine
// imports) from 3 rev2 queries to 5 workload classes:
//
//   q1 point-filter            (warehouse SELECTIVITY floor case)
//   q2 grouped-aggregation     (rev2 q2 SQL, byte-identical)
//   q3 windowed-rolling-avg    (rev2 q3 SQL, byte-identical)
//   q4 cohort-retention-journey
//   q5 ocel-rehydration        (full 100k-row ordered stream drain)
//
// Path A: real HTTP submit+poll against the local Next sim (Snowflake SQL API
//         v2 surface). Every poll is a real request; latency measured with
//         Date.now around the full submit->SUCCEEDED cycle. Cold start is
//         observed (iteration 1) and amortized across all iterations.
// Path B: sovereign native probe receipt (economics/sovereign_probe --full),
//         frozen-duckdb in-process. Egress 0 and invoice $0.00 by construction.
//
// Disruption assertions per class (the G5 fences, applied per class):
//   F1 latency : sovereign p95 <= 0.05 x warehouse amortized p95
//   F2 egress  : sovereign bytes  <= 0.01 x warehouse bytes
//   F3 invoice : sovereign invoice === $0.00 (exact), counterfactual burn > 0
// A class that does not clear F1 gets an honest NEG row with the REAL ratio —
// the ratio is reported, never tortured to fit.
//
// Usage:
//   node economics/scorecard.ts --sovereign <probe.json> [--base URL]
//        [--json out.json] [--md out.md] [--iters N] [--selftest]
//
// Exits 0 produced, 1 FAIL/VACUOUS, 3 = selftest expected-failure witness.
import { readFile, writeFile } from "node:fs/promises";

// Chicago rule: import the sim's REAL generated modules, no fakes.
import { creditCost } from "../lib/ledger.ts";
import {
  nextState,
  POLL_INTERVAL_MS,
} from "../lib/statement_state.ts";

interface SovereignQuery {
  id: string;
  p95_latency_ms: number;
  cold_latency_ms?: number;
  egress_bytes: number;
  invoice_usd: number;
  rows_returned: number;
}
interface SovereignReceipt {
  leg: string;
  engine: string;
  seed: number;
  rows: number;
  queries: SovereignQuery[];
}

interface WarehouseQuery {
  id: string;
  cold_latency_ms: number;
  amortized_p95_ms: number;
  poll_requests: number;
  network_bytes: number;
  credits: number;
  rows: number;
}

const BASE = process.argv.includes("--base")
  ? process.argv[process.argv.indexOf("--base") + 1]
  : "http://127.0.0.1:3111";
const SOV = process.argv.includes("--sovereign")
  ? process.argv[process.argv.indexOf("--sovereign") + 1]
  : undefined;
const JSON_OUT = process.argv.includes("--json")
  ? process.argv[process.argv.indexOf("--json") + 1]
  : undefined;
const MD_OUT = process.argv.includes("--md")
  ? process.argv[process.argv.indexOf("--md") + 1]
  : undefined;
const ITERS = process.argv.includes("--iters")
  ? parseInt(process.argv[process.argv.indexOf("--iters") + 1], 10)
  : 4;
const SELFTEST = process.argv.includes("--selftest");

function bail(msg: string): never {
  console.error(`[SCORECARD-FAIL] ${msg}`);
  process.exit(1);
}
function assert(cond: boolean, msg: string): void {
  if (!cond) bail(msg);
}

// Map scorecard class -> the sovereign probe record id (--full receipt).
const SOV_ID: Record<string, string> = {
  "point-filter": "q4-point-filter",
  "grouped-aggregation": "q2-funnel-groupby",
  "windowed-rolling-avg": "q3-event-window",
  "cohort-retention-journey": "q5-cohort-journey",
  "ocel-rehydration": "q6-ocel-rehydrate",
};

async function readSovereign(): Promise<{ receipt: SovereignReceipt; queries: SovereignQuery[] }> {
  if (!SOV) bail("missing --sovereign <probe.json>");
  const raw = await readFile(SOV, "utf8");
  const receipt = JSON.parse(raw) as SovereignReceipt;
  assert(Array.isArray(receipt.queries) && receipt.queries.length > 0, "sovereign receipt must carry query records");
  const byId = new Map(receipt.queries.map((q) => [q.id, q]));
  const queries = CLASSES.map((c) => {
    const q = byId.get(SOV_ID[c.id]);
    assert(!!q, `sovereign receipt missing record ${SOV_ID[c.id]} for class ${c.id} (run sovereign_probe --full)`);
    assert(q!.p95_latency_ms > 0, `sovereign latency must be positive for ${q!.id}`);
    return q!;
  });
  return { receipt, queries };
}

// ---- the 5 workload classes -------------------------------------------------

const CLASSES: { id: string; label: string; sql: string }[] = [
  {
    id: "point-filter",
    label: "point filter",
    sql: "SELECT count(*), sum(value) FROM events WHERE event_id = 90000",
  },
  {
    id: "grouped-aggregation",
    label: "grouped aggregation",
    sql: "WITH per_case AS (SELECT case_id, count(*) AS n, sum(value) AS s FROM events GROUP BY case_id) SELECT bucket, count(*) AS cases, sum(s) AS total FROM (SELECT s / 1000 AS bucket, s FROM per_case) t GROUP BY bucket ORDER BY bucket",
  },
  {
    id: "windowed-rolling-avg",
    label: "windowed rolling avg",
    sql: "SELECT sum(rolling) FROM (SELECT avg(value) OVER (PARTITION BY case_id ORDER BY ts ROWS BETWEEN 49 PRECEDING AND CURRENT ROW) AS rolling FROM events)",
  },
  {
    id: "cohort-retention-journey",
    label: "cohort retention journey",
    sql: "WITH first_ts AS (SELECT case_id, min(ts) AS f FROM events GROUP BY case_id) SELECT (e.ts - f.f) / 10000 AS step, count(DISTINCT e.case_id) AS retained FROM events e JOIN first_ts f ON e.case_id = f.case_id GROUP BY step ORDER BY step",
  },
  {
    id: "ocel-rehydration",
    label: "OCEL rehydration",
    sql: "SELECT case_id, ts, event_id, value FROM events ORDER BY case_id, ts, event_id",
  },
];

// ---- Path A: real HTTP against the local sim (court.ts runner shape) -------

function reqBytes(...parts: (string | undefined)[]): number {
  return parts
    .filter((s) => s !== undefined)
    .reduce((n, s) => n + Buffer.byteLength(s as string), 0);
}

async function runWarehouseQuery(
  sql: string,
  qid: string,
  deadlineMs: number,
): Promise<WarehouseQuery> {
  const body = JSON.stringify({
    sqlText: sql,
    warehouse: "COURT_XS",
    database: "COURT_DB",
    schema: "PUBLIC",
    asyncMode: true,
  });
  const latencies: number[] = [];
  let totalBytes = 0;
  let totalPolls = 0;
  let rows = 0;

  for (let it = 0; it < ITERS; it++) {
    const t0 = Date.now();
    const res = await fetch(`${BASE}/api/v2/statements`, {
      method: "POST",
      headers: {
        "content-type": "application/json",
        authorization: "Bearer court-token",
      },
      body,
    });
    const text = await res.text();
    assert(res.status === 202, `submit ${qid}: expected 202, got ${res.status}`);
    const pollUrlPath = res.headers.get("location");
    assert(!!pollUrlPath, `submit ${qid}: missing Location header`);
    let bytes = reqBytes(body, text);

    // Poll with the sim's own generated state machine (client-side clock keyed
    // to the declared cold-start model); every poll is a real HTTP request.
    let state = "QUEUED" as ReturnType<typeof nextState>;
    const started = Date.now();
    let polls = 0;
    while (state !== "SUCCEEDED") {
      assert(
        Date.now() - started < deadlineMs,
        `query ${qid}: poll deadline exceeded in state ${state}`,
      );
      await new Promise((r) => setTimeout(r, POLL_INTERVAL_MS));
      const pr = await fetch(`${BASE}${pollUrlPath ?? ""}`);
      const ptext = await pr.text();
      bytes += reqBytes(pollUrlPath ?? undefined, ptext);
      polls += 1;
      assert(
        pr.status === 200 || pr.status === 202,
        `poll ${qid}: unexpected status ${pr.status}`,
      );
      const elapsed = Date.now() - started;
      state = nextState(state, elapsed);
    }
    latencies.push(Date.now() - t0);
    totalBytes += bytes;
    totalPolls += polls;
    rows = 0;
  }

  // Ledger: computed by the sim's REAL generated creditCost on recorded
  // quantities (total observed wall time + actual network bytes observed).
  const totalMs = latencies.reduce((a, b) => a + b, 0);
  const entry = {
    warehouseHours: totalMs / 3_600_000,
    dcuHours: 0,
    bytesScanned: totalBytes,
  };
  const sorted = [...latencies].sort((a, b) => a - b);
  const pick = (p: number) =>
    sorted[Math.min(sorted.length - 1, Math.ceil(p * sorted.length) - 1)];
  return {
    id: qid,
    cold_latency_ms: latencies[0], // iteration 1 = observed cold start
    amortized_p95_ms: pick(0.95), // amortization across ALL iterations incl cold
    poll_requests: totalPolls,
    network_bytes: totalBytes,
    credits: creditCost(entry),
    rows,
  };
}

// ---- scorecard assembly + assertions ---------------------------------------

function pctl(xs: number[], p: number): number {
  const s = [...xs].sort((a, b) => a - b);
  return s[Math.min(s.length - 1, Math.ceil(p * s.length) - 1)];
}

function buildScorecard(s: SovereignQuery[], w: WarehouseQuery[]) {
  return CLASSES.map((c, i) => {
    const sq = s[i];
    const wq = w[i];
    const latRatio = sq.p95_latency_ms / wq.amortized_p95_ms;
    const coldLatRatio = (sq.cold_latency_ms ?? sq.p95_latency_ms) / wq.cold_latency_ms;
    const egressRatio =
      wq.network_bytes > 0 ? sq.egress_bytes / wq.network_bytes : 0;
    const f1 = latRatio <= 0.05;
    const f2 = egressRatio <= 0.01;
    const f3 = sq.invoice_usd === 0.0 && wq.credits > 0;
    return {
      class_id: c.id,
      label: c.label,
      warehouse: {
        cold_p50_ms: wq.cold_latency_ms,
        amortized_p95_ms: wq.amortized_p95_ms,
        poll_requests: wq.poll_requests,
        egress_bytes: wq.network_bytes,
        counterfactual_credits: wq.credits,
      },
      sovereign: {
        p95_latency_ms: sq.p95_latency_ms,
        cold_latency_ms: sq.cold_latency_ms ?? sq.p95_latency_ms,
        egress_bytes: sq.egress_bytes,
        invoice_usd: sq.invoice_usd,
      },
      ratios: {
        latency_amortized: latRatio,
        latency_cold_vs_cold: coldLatRatio,
        egress: egressRatio,
      },
      assertions: {
        F1_latency_le_0_05x: f1 ? "CLEAR" : "NEG",
        F2_egress_le_0_01x: f2 ? "CLEAR" : "NEG",
        F3_invoice_eq_0: f3 ? "CLEAR" : "NEG",
      },
    };
  });
}

function markdownTable(rows: ReturnType<typeof buildScorecard>): string {
  const line = (s: string) => `| ${s} |`;
  const out: string[] = [];
  out.push(
    line(
      "workload class | wh cold p50 (ms) | wh amortized p95 (ms) | sovereign p95 (ms) | lat ratio | F1 ≤0.05× | wh egress (B) | sov egress (B) | egress ratio | F2 ≤0.01× | counterfactual $ | invoice $ | F3 ≡$0.00",
    ),
  );
  out.push(
    line(
      "--- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | ---",
    ),
  );
  for (const r of rows) {
    out.push(
      line(
        [
          r.label,
          r.warehouse.cold_p50_ms.toFixed(0),
          r.warehouse.amortized_p95_ms.toFixed(0),
          r.sovereign.p95_latency_ms.toFixed(3),
          r.ratios.latency_amortized.toExponential(2),
          r.assertions.F1_latency_le_0_05x,
          r.warehouse.egress_bytes,
          r.sovereign.egress_bytes,
          r.ratios.egress === 0 ? "0" : r.ratios.egress.toExponential(2),
          r.assertions.F2_egress_le_0_01x,
          "$" + r.warehouse.counterfactual_credits.toFixed(6),
          "$" + r.sovereign.invoice_usd.toFixed(2),
          r.assertions.F3_invoice_eq_0,
        ].join(" | "),
      ),
    );
  }
  return out.join("\n");
}

async function main(): Promise<void> {
  const { receipt: sovReceipt, queries: sovereign } = await readSovereign();
  const { seed, rows: nRows } = sovReceipt;

  if (SELFTEST) {
    // Non-vacuity witness: inflate the sovereign leg past every fence and
    // require the SAME assertion machinery to REFUSE. Exit 3 = flipped.
    const doctored: SovereignQuery[] = sovereign.map((q) => ({
      ...q,
      p95_latency_ms: q.p95_latency_ms * 10_000,
      egress_bytes: 1_000_000_000,
      invoice_usd: 42.0,
    }));
    runAssertions(buildScorecard(doctored, synthWarehouse(sovereign)), true);
    console.error(
      "[SCORECARD-VACUOUS] selftest did not flip — assertion machinery accepted a doctored PASS",
    );
    process.exit(1);
  }

  const warehouse: WarehouseQuery[] = [];
  for (const c of CLASSES) {
    process.stderr.write(`[scorecard] Path A ${c.id}\n`);
    warehouse.push(await runWarehouseQuery(c.sql, c.id, 120_000));
  }

  const rows = buildScorecard(sovereign, warehouse);
  runAssertions(rows, false);

  const receipt = {
    scorecard: "warehouse-sim/economics/disruption-scorecard",
    seed,
    rows: nRows,
    iterations_path_a: ITERS,
    sovereign: {
      leg: sovReceipt.leg,
      engine: sovReceipt.engine,
      total_network_bytes: 0,
      total_invoice_usd: 0.0,
    },
    warehouse: {
      engine: "warehouse-sim (Next 14.2.15 local sim; Snowflake SQL API v2 surface)",
      total_network_bytes: warehouse.reduce((n, q) => n + q.network_bytes, 0),
      total_credits: warehouse.reduce((n, q) => n + q.credits, 0),
    },
    classes: rows,
  };
  const md = markdownTable(rows);

  if (JSON_OUT) await writeFile(JSON_OUT, JSON.stringify(receipt, null, 2) + "\n");
  if (MD_OUT) await writeFile(MD_OUT, md + "\n");
  console.log(md);
}

function synthWarehouse(s: SovereignQuery[]): WarehouseQuery[] {
  return s.map((q) => ({
    id: q.id,
    cold_latency_ms: 5_000,
    amortized_p95_ms: 5_000,
    poll_requests: 10,
    network_bytes: 1024,
    credits: 0.001,
    rows: 0,
  }));
}

function runAssertions(
  rows: ReturnType<typeof buildScorecard>,
  expectFail: boolean,
): void {
  const fail = (msg: string) => {
    if (expectFail) {
      console.error(`[SELFTEST-FLIP] refused as required: ${msg}`);
      process.exit(3);
    }
    bail(msg);
  };
  for (const r of rows) {
    // F3 (invoice identity + nonzero counterfactual) is a HARD fence: a
    // nonzero sovereign invoice is a modeled-domain violation, never an
    // honest-negative. This is the fence the selftest must flip.
    if (r.assertions.F3_invoice_eq_0 !== "CLEAR")
      fail(
        `F3 ${r.class_id}: invoice/counterfactual violated (sov $${r.sovereign.invoice_usd.toFixed(2)}, wh credits ${r.warehouse.counterfactual_credits})`,
      );
    // F1/F2 register honest negatives with the REAL ratio — reported, never
    // tortured. A NEG row is a divergence on the scorecard, not a fake pass.
    if (!r.assertions.F2_egress_le_0_01x)
      console.error(
        `[SCORECARD-NEG] F2 ${r.class_id}: egress ratio ${r.ratios.egress.toExponential(2)} > 0.01 — honest-negative row`,
      );
    if (!r.assertions.F1_latency_le_0_05x)
      console.error(
        `[SCORECARD-NEG] F1 ${r.class_id}: sovereign ${r.sovereign.p95_latency_ms.toFixed(3)}ms vs warehouse ${r.warehouse.amortized_p95_ms.toFixed(0)}ms — real ratio ${r.ratios.latency_amortized.toExponential(2)} > 0.05 — honest-negative row`,
      );
  }
}

main().catch((e) => {
  console.error("[SCORECARD-ERROR]", e);
  process.exit(1);
});
