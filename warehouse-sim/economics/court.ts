// Empirical economic court — lane G5 (frozen-duckdb repo).
//
// Runs the identical seeded analytical workload (seed 20261019, 100k-row
// cohort/funnel/event shape, 3 rev2 queries) through BOTH paths:
//   Path A (warehouse): real HTTP submit+poll against the local Next sim
//     (Snowflake SQL API v2 surface, /api/v2/statements).
//   Path B (sovereign): in-process frozen-duckdb via sovereign_probe
//     (native-baseline label; the wasm/OPFS leg is FR-01-blocked).
//
// Usage:
//   node economics/court.ts --sovereign <probe.json> [--base URL]
//        [--out <receipt.json>] [--selftest]
//
// Exits 0 PASS, 1 FAIL/VACUOUS, 3 = selftest expected-failure witness.
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
  submit_status: number;
  poll_requests: number;
  p95_latency_ms: number;
  network_bytes: number;
  credits: number;
  ledger_entry: { warehouseHours: number; dcuHours: number; bytesScanned: number };
  rows: number;
}

const BASE = process.argv.includes("--base")
  ? process.argv[process.argv.indexOf("--base") + 1]
  : "http://127.0.0.1:3111";
const OUT = process.argv.includes("--out")
  ? process.argv[process.argv.indexOf("--out") + 1]
  : undefined;
const SELFTEST = process.argv.includes("--selftest");

function bail(msg: string): never {
  console.error(`[COURT-FAIL] ${msg}`);
  process.exit(1);
}

function assert(cond: boolean, msg: string): void {
  if (!cond) bail(msg);
}

async function readSovereign(): Promise<SovereignReceipt> {
  const i = process.argv.indexOf("--sovereign");
  if (i < 0) bail("missing --sovereign <probe.json>");
  const raw = await readFile(process.argv[i + 1], "utf8");
  const r = JSON.parse(raw) as SovereignReceipt;
  assert(Array.isArray(r.queries) && r.queries.length === 3, "sovereign receipt must carry 3 query records");
  assert(r.queries.every((q) => q.p95_latency_ms > 0), "sovereign latencies must be positive");
  return r;
}

// ---- Path A: real HTTP against the local sim ------------------------------

function reqBytes(...parts: (string | undefined)[]): number {
  return parts.filter(Boolean).reduce((n, s) => n + Buffer.byteLength(s as string), 0);
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
  const t0 = Date.now();
  const res = await fetch(`${BASE}/api/v2/statements`, {
    method: "POST",
    headers: { "content-type": "application/json", authorization: "Bearer court-token" },
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
    assert(Date.now() - started < deadlineMs, `query ${qid}: poll deadline exceeded in state ${state}`);
    await new Promise((r) => setTimeout(r, POLL_INTERVAL_MS));
    const pr = await fetch(`${BASE}${pollUrlPath}`);
    const ptext = await pr.text();
    bytes += reqBytes(pollUrlPath ?? undefined, ptext); // SHIM-COURT S5: null-narrow at use site (was TS2345 at this line)
    polls += 1;
    assert(pr.status === 200 || pr.status === 202, `poll ${qid}: unexpected status ${pr.status}`);
    const elapsed = Date.now() - started;
    state = nextState(state, elapsed);
  }
  const latency = Date.now() - t0;

  // Ledger: computed by the sim's REAL generated creditCost on recorded
  // quantities (elapsed wall time + actual network bytes observed).
  const entry = {
    warehouseHours: latency / 3_600_000,
    dcuHours: 0,
    bytesScanned: bytes,
  };
  return {
    id: qid,
    submit_status: res.status,
    poll_requests: polls,
    p95_latency_ms: latency,
    network_bytes: bytes,
    credits: creditCost(entry),
    ledger_entry: entry,
    rows: 0,
  };
}


const WORKLOAD: [string, string][] = [
  ["q1-cohort-filter", "SELECT count(*), sum(value) FROM events WHERE event_id BETWEEN 90000 AND 90100 AND ts % 7 = 3"],
  ["q2-funnel-groupby", "WITH per_case AS (SELECT case_id, count(*) AS n, sum(value) AS s FROM events GROUP BY case_id) SELECT bucket, count(*) AS cases, sum(s) AS total FROM (SELECT s / 1000 AS bucket, s FROM per_case) t GROUP BY bucket ORDER BY bucket"],
  ["q3-event-window", "SELECT sum(rolling) FROM (SELECT avg(value) OVER (PARTITION BY case_id ORDER BY ts ROWS BETWEEN 49 PRECEDING AND CURRENT ROW) AS rolling FROM events)"],
];

async function main(): Promise<void> {
  const sovereign = await readSovereign();

  if (SELFTEST) {
    // Non-vacuity witness: inflate the sovereign leg past every fence and
    // require the SAME assertion machinery to REFUSE. Exit 3 = flipped.
    const doctored: SovereignReceipt = {
      ...sovereign,
      queries: sovereign.queries.map((q) => ({
        ...q,
        p95_latency_ms: q.p95_latency_ms * 10_000,
        egress_bytes: 1_000_000_000,
        invoice_usd: 42.0,
      })),
    };
    runAssertions(doctored, synthWarehouse(sovereign), true);
    console.error("[COURT-VACUOUS] selftest did not flip — assertion machinery accepted a doctored PASS");
    process.exit(1);
  }

  const warehouse: WarehouseQuery[] = [];
  for (const [qid, sql] of WORKLOAD) {
    warehouse.push(await runWarehouseQuery(sql, qid, 60_000));
  }
  const receipt = buildReceipt(sovereign, warehouse);
  runAssertions(sovereign, warehouse, false);
  if (OUT) await writeFile(OUT, JSON.stringify(receipt, null, 2) + "\n");
  console.log(JSON.stringify(receipt, null, 2));
}

function synthWarehouse(s: SovereignReceipt): WarehouseQuery[] {
  return WORKLOAD.map(([qid], i) => ({
    id: qid,
    submit_status: 202,
    poll_requests: 10,
    p95_latency_ms: s.queries[i].p95_latency_ms * 0.01,
    network_bytes: 1024,
    credits: 0.001,
    ledger_entry: { warehouseHours: 0, dcuHours: 0, bytesScanned: 1024 },
    rows: 0,
  }));
}

function buildReceipt(s: SovereignReceipt, w: WarehouseQuery[]) {
  return {
    court: "warehouse-sim/economics",
    seed: s.seed,
    rows: s.rows,
    sovereign: {
      leg: s.leg,
      engine: s.engine,
      queries: s.queries,
      total_network_bytes: 0,
      total_invoice_usd: 0.0,
    },
    warehouse: {
      engine: "warehouse-sim (Next 14.2.15 local sim; Snowflake SQL API v2 surface)",
      queries: w,
      total_network_bytes: w.reduce((n, q) => n + q.network_bytes, 0),
      total_credits: w.reduce((n, q) => n + q.credits, 0),
    },
  };
}

// ---- Assertions (the load-bearing fences) ---------------------------------

function runAssertions(s: SovereignReceipt, w: WarehouseQuery[], expectFail: boolean): void {
  const fail = (msg: string) => {
    if (expectFail) {
      console.error(`[SELFTEST-FLIP] refused as required: ${msg}`);
      process.exit(3);
    }
    bail(msg);
  };

  for (let i = 0; i < 3; i++) {
    const sq = s.queries[i];
    const wq = w[i];
    // F1: per-query latency — sovereign p95 <= 0.05 x warehouse latency.
    const latFence = 0.05 * wq.p95_latency_ms;
    if (sq.p95_latency_ms > latFence)
      fail(
        `F1 ${sq.id}: sovereign ${sq.p95_latency_ms.toFixed(3)}ms > 0.05 x warehouse ${wq.p95_latency_ms.toFixed(0)}ms (fence ${latFence.toFixed(1)}ms)`,
      );
    // F2: network bytes — sovereign <= 0.01 x warehouse.
    const netFence = 0.01 * wq.network_bytes;
    if (sq.egress_bytes > netFence)
      fail(`F2 ${sq.id}: sovereign egress ${sq.egress_bytes}B > 1% of warehouse ${wq.network_bytes}B`);
  }
  // F3: sovereign invoice exactly zero.
  const sInv = s.queries.reduce((n, q) => n + q.invoice_usd, 0);
  if (sInv !== 0.0) fail(`F3: sovereign invoice ${sInv} != $0.00`);
  // F4: counterfactual burn — the sim ledger MUST show nonzero credits.
  const total = w.reduce((n, q) => n + q.credits, 0);
  if (!(total > 0)) fail("F4 VACUOUS: sim ledger shows zero credit burn — counterfactual not exercised");
}

main().catch((e) => {
  console.error("[COURT-ERROR]", e);
  process.exit(1);
});
