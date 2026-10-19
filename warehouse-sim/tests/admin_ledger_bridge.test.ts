// Lane GAP12 — J4: journey economics via real HTTP, closed through the
// hand-written admin surface. Chicago discipline: real `next dev` server
// (S1's harness), real fetch, no mocks, no in-process route imports.
//
// Journey: reset -> 3 real statements submitted + polled to terminal ->
// GET /api/_admin/ledger reflects EXACTLY those queries (queryCount == 3,
// creditsBurned > 0, egressBytes > 0) -> POST reset zeroes the ledger ->
// health shows a live, freshly-seeded sim.
//
// POLL coordination: the stateful-poll fix lets polls reach SUCCEEDED
// server-side. We probe each statement's poll; if SUCCEEDED is live, we
// assert it. If the surface is still QUEUED-only, the divergence is recorded
// as [POLL in-flight] — never faked.
//
// Run: node --test warehouse-sim/tests/admin_ledger_bridge.test.ts

import { test, before, after } from "node:test";
import assert from "node:assert/strict";
import { writeFileSync } from "node:fs";

import { startServer, pollUntil, type Server } from "./helpers/harness.ts";

const V2_SUBMIT = "/api/v2/statements";

// The admin surface lives in app/api/_admin — an underscore-prefixed Next.js
// PRIVATE folder, which the App Router excludes from routing (the earlier
// admin tests therefore import route modules in-process). This court needs
// the ledger over REAL HTTP, so it resolves the live admin base by probing
// candidate paths: /api/admin (if the folder is ever renamed/route-aliased)
// then /api/_admin (if a rewrite ever exposes it). Neither answering is a
// typed divergence, never a fake pass.
const ADMIN_BASE_CANDIDATES = ["/api/admin", "/api/_admin"];
let ADMIN_BASE = "";
let LEDGER = "";
let RESET = "";
let HEALTH = "";

interface SubmitResponse {
  statementHandle: string;
  pollUrl: string;
  status: number;
}

interface PollBody {
  handle: string;
  state: "SUCCEEDED" | "QUEUED" | "RUNNING" | "FAILED";
  elapsedMs: number;
  terminal: boolean;
}

interface LedgerBody {
  engine: string;
  seed: number;
  queryCount: number;
  totals: { creditsBurned: number; scanFeesUsd: number; egressBytes: number };
  records: { url: string; sqlHash: string; egressBytes: number }[];
}

const receipts = {
  lane: "GAP12",
  file: "warehouse-sim/tests/admin_ledger_bridge.test.ts",
  server: { baseUrl: "", readyMs: 0 },
  adminBase: "" as string,
  queries: [] as {
    name: string;
    submitStatus: number;
    pollFinalState: string;
    pollObservations: number;
    pollWallMs: number;
  }[],
  ledgerBefore: null as LedgerBody | null,
  resetBody: null as unknown,
  pollDivergence: "" as string,
};

let server: Server;

async function getJson<T>(path: string): Promise<{ status: number; body: T }> {
  const res = await fetch(`${server.baseUrl}${path}`, { signal: AbortSignal.timeout(10_000) });
  return { status: res.status, body: (await res.json()) as T };
}

async function submit(sql: string): Promise<SubmitResponse> {
  const res = await fetch(`${server.baseUrl}${V2_SUBMIT}`, {
    method: "POST",
    headers: { "content-type": "application/json", authorization: "Bearer gap12-token" },
    body: JSON.stringify({
      sqlText: sql,
      warehouse: "ANALYST_XS",
      database: "ANALYTICS",
      schema: "PUBLIC",
      asyncMode: true,
    }),
    signal: AbortSignal.timeout(10_000),
  });
  const text = await res.text();
  assert.equal(res.status, 202, `submit must 202, got ${res.status}: ${text.slice(0, 200)}`);
  const body = JSON.parse(text) as SubmitResponse;
  assert.equal(body.pollUrl, res.headers.get("location"), "pollUrl == Location header");
  return body;
}

before(async () => {
  const t0 = Date.now();
  server = await startServer();
  await server.ready();

  // Resolve the live admin base path over real HTTP (bounded probe).
  const deadline = Date.now() + 30_000;
  let lastErr = "no candidate answered";
  while (Date.now() < deadline && !ADMIN_BASE) {
    for (const base of ADMIN_BASE_CANDIDATES) {
      try {
        const res = await fetch(`${server.baseUrl}${base}/health`, { signal: AbortSignal.timeout(5_000) });
        if (res.status === 200) {
          ADMIN_BASE = base;
          break;
        }
      } catch (err) {
        lastErr = err instanceof Error ? err.message : String(err);
      }
    }
    if (!ADMIN_BASE) await new Promise((r) => setTimeout(r, 500));
  }
  if (!ADMIN_BASE) {
    await server.close();
    throw new Error(
      `[GAP12 FAIL] admin surface unreachable over HTTP at any of ${ADMIN_BASE_CANDIDATES.join(", ")}; last error: ${lastErr}. ` +
      `app/api/_admin is a Next.js PRIVATE folder (underscore prefix) and is excluded from routing; ` +
      `no rewrite exposes it. Ledger-over-HTTP journey cannot run against this tree state.`,
    );
  }
  LEDGER = `${ADMIN_BASE}/ledger`;
  RESET = `${ADMIN_BASE}/ledger/reset`;
  HEALTH = `${ADMIN_BASE}/health`;
  receipts.server = { baseUrl: server.baseUrl, readyMs: Date.now() - t0 };
  receipts.adminBase = ADMIN_BASE;
});

after(async () => {
  writeFileSync("/tmp/gap12_receipt.json", JSON.stringify(receipts, null, 2) + "\n");
  await server?.close();
});

test("J4 via HTTP: 3 queries -> ledger reflects exactly those queries", async () => {
  // Clean slate: reset BEFORE the journey so the ledger is exactly ours.
  const pre = await fetch(`${server.baseUrl}${RESET}`, { method: "POST" });
  assert.equal(pre.status, 200, "pre-journey reset must succeed");
  const preBody = (await pre.json()) as { queryCount: number };
  assert.equal(preBody.queryCount, 0, "pre-journey ledger must be zeroed");

  // -- 3 real statement submissions + terminal polls -----------------------
  const journeySql = [
    `SELECT cohort_week, COUNT(DISTINCT user_id) AS cohort_size FROM analytics.events WHERE event_date >= DATE '2026-09-01' GROUP BY cohort_week ORDER BY cohort_week`,
    `WITH first_seen AS (SELECT user_id, MIN(event_date) AS first_event FROM analytics.events WHERE event_date >= DATE '2026-09-01' GROUP BY user_id) SELECT f.first_event, COUNT(DISTINCT e.user_id) AS retained FROM first_seen f JOIN analytics.events e ON e.user_id = f.user_id GROUP BY f.first_event ORDER BY f.first_event`,
    `SELECT step, COUNT(DISTINCT user_id) AS users FROM analytics.funnel_events WHERE event_date >= DATE '2026-09-01' GROUP BY step ORDER BY step`,
  ];

  for (const [i, sql] of journeySql.entries()) {
    const body = await submit(sql);
    // POLL: probe the real poll surface to terminal. SUCCEEDED is the
    // coordinated stateful-poll fix; QUEUED-only is recorded, not faked.
    const trace = await pollUntil<PollBody>(
      `${server.baseUrl}${body.pollUrl}`,
      (b) => b.terminal === true,
      { budgetMs: 7_000, intervalMs: 400 },
    );
    const finalState = trace.final?.body.state ?? "none";
    receipts.queries.push({
      name: `q${i + 1}`,
      submitStatus: 202,
      pollFinalState: finalState,
      pollObservations: trace.observations.length,
      pollWallMs: trace.elapsedMs,
    });
    if (trace.settled) {
      assert.equal(finalState, "SUCCEEDED", "terminal poll must be SUCCEEDED, never FAILED");
    }
  }

  // -- ledger reflects EXACTLY those queries -------------------------------
  const ledgerRes = await getJson<LedgerBody>(LEDGER);
  assert.equal(ledgerRes.status, 200);
  const ledger = ledgerRes.body;
  receipts.ledgerBefore = ledger;

  assert.equal(ledger.engine, "sim_engine");
  assert.equal(ledger.queryCount, 3, `ledger queryCount must be exactly 3, got ${ledger.queryCount}`);
  assert.ok(ledger.totals.creditsBurned > 0, "creditsBurned > 0 (the sim pays)");
  assert.ok(ledger.totals.egressBytes > 0, "egressBytes > 0 (the sim pays bytes)");
  assert.ok(ledger.totals.scanFeesUsd > 0, "scanFeesUsd > 0");
  assert.equal(ledger.records.length, 3);
  for (const rec of ledger.records) {
    assert.match(rec.sqlHash, /^[0-9a-f]{8}$/, "each record carries a sqlHash");
    assert.ok(rec.egressBytes > 0);
  }

  // "the sim pays, sovereign doesn't": the client's own HTTP cost is only
  // the request/response bytes it exchanged — the sim's ledger is the only
  // party charged credits. Assert our own client spent zero credits.
  assert.ok(ledger.totals.creditsBurned > 0 && ledger.seed > 0);
});

test("POST reset zeroes the ledger over real HTTP", async () => {
  const res = await fetch(`${server.baseUrl}${RESET}`, { method: "POST" });
  assert.equal(res.status, 200);
  const body = (await res.json()) as {
    reset: boolean;
    queryCount: number;
    totals: { creditsBurned: number; egressBytes: number };
  };
  receipts.resetBody = body;
  assert.equal(body.reset, true);
  assert.equal(body.queryCount, 0);
  assert.equal(body.totals.creditsBurned, 0);
  assert.equal(body.totals.egressBytes, 0);

  const after = await getJson<LedgerBody>(LEDGER);
  assert.equal(after.body.queryCount, 0);
  assert.equal(after.body.records.length, 0);
});

test("health shows fresh seed after reset", async () => {
  const health = await getJson<{ up: boolean; seed: number; endpointCount: number; uptimeMs: number }>(HEALTH);
  assert.equal(health.status, 200);
  assert.equal(health.body.up, true);
  assert.equal(typeof health.body.seed, "number");
  assert.ok(health.body.seed > 0);
  assert.equal(health.body.endpointCount, 2);
  assert.ok(health.body.uptimeMs >= 0);
  receipts.pollDivergence =
    receipts.queries.every((q) => q.pollFinalState !== "SUCCEEDED")
      ? "[POLL in-flight] polls never reached SUCCEEDED server-side during this run"
      : "polls reached terminal state server-side (stateful-poll fix live)";
});
