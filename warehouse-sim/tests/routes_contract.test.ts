// Server-side contract tests for the ggen-rendered warehouse-sim routes.
// Chicago discipline: real HTTP against a REAL `next dev` server — no route
// unit mocks. Where a route does not implement declared behavior (auth, 404,
// lifecycle advancement), the test asserts the REAL rendered behavior and the
// divergence is recorded as a finding, never invented around.

import assert from "node:assert/strict";
import { after, before, describe, it } from "node:test";
import { startServer, pollUntil, statesOf, type Server } from "./helpers/harness.ts";

const SNOWFLAKE_SUBMIT = "/api/v2/statements";
const SNOWFLAKE_POLL_BUDGET_MS = 15_000; // covers 4500ms cold start + jitter with headroom
const DATABRICKS_SUBMIT = "/api/2.0/sql/statements";
const DATABRICKS_POLL_BUDGET_MS = 15_000;

let server: Server;

before(async () => {
  server = await startServer();
  await server.ready();
});

after(async () => {
  if (server) await server.close();
});

function uuidShape(s: string): boolean {
  return /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(s);
}

async function json(res: Response): Promise<any> {
  return res.json();
}

describe("Snowflake POST /api/v2/statements", () => {
  it("returns 202 with statementHandle and poll URL shape", async () => {
    const res = await fetch(`${server.baseUrl}${SNOWFLAKE_SUBMIT}`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        sqlText: "SELECT 1",
        warehouse: "WH",
        database: "DB",
        schema: "S",
        asyncMode: true,
      }),
    });
    assert.equal(res.status, 202, `expected 202, got ${res.status}`);
    const body = await json(res);
    assert.equal(typeof body.statementHandle, "string");
    assert.ok(uuidShape(body.statementHandle), `handle not a UUID: ${body.statementHandle}`);
    const location = res.headers.get("location");
    assert.ok(location, "expected Location header on 202");
    assert.equal(location, `/api/v2/statements/${body.statementHandle}`);
    assert.equal(body.pollUrl, `/api/v2/statements/${body.statementHandle}`);
    assert.equal(body.status, 202);
  });
});

describe("Snowflake poll lifecycle GET /api/v2/statements/{handle}", () => {
  it("immediately after submit reports QUEUED (HTTP 202)", async () => {
    const submit = await fetch(`${server.baseUrl}${SNOWFLAKE_SUBMIT}`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ sqlText: "SELECT 1", warehouse: "WH", database: "DB", schema: "S", asyncMode: true }),
    });
    const { statementHandle } = await json(submit);

    const first = await fetch(`${server.baseUrl}${SNOWFLAKE_SUBMIT}/${statementHandle}`);
    assert.equal(first.status, 202, `expected 202 QUEUED, got ${first.status}`);
    const body = await json(first);
    assert.equal(body.state, "QUEUED");
    assert.equal(body.handle, statementHandle);
    assert.equal(body.terminal, false);
  });

  it("polls within the 4500ms+jitter budget; records the REAL settled state", async () => {
    const submit = await fetch(`${server.baseUrl}${SNOWFLAKE_SUBMIT}`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ sqlText: "SELECT 1", warehouse: "WH", database: "DB", schema: "S", asyncMode: true }),
    });
    const { statementHandle } = await json(submit);

    const trace = await pollUntil<any>(
      `${server.baseUrl}${SNOWFLAKE_SUBMIT}/${statementHandle}`,
      (b) => b.state === "SUCCEEDED" || b.state === "FAILED",
      { budgetMs: SNOWFLAKE_POLL_BUDGET_MS, intervalMs: 500 },
    );
    const states = statesOf(trace).filter((s) => s !== "_nonJson");
    console.log(`[FINDING] snowflake poll trace (${trace.observations.length} polls, ${trace.elapsedMs}ms): ${states.join(" -> ")}`);
    // Contract: QUEUED while the 4500ms cold start elapses, RUNNING in the
    // jitter window, SUCCEEDED terminal — all inside the 15s budget.
    assert.equal(trace.settled, true, `poll did not reach terminal within ${SNOWFLAKE_POLL_BUDGET_MS}ms; got ${states.join(" -> ")}`);
    assert.equal(trace.final.body.state, "SUCCEEDED");
    assert.equal(trace.final.status, 200, "SUCCEEDED must carry HTTP 200");
    assert.ok(states[0] === "QUEUED", `first observation must be QUEUED, got ${states[0]}`);
    const uniqueInOrder = [...new Set(states)];
    assert.deepEqual(
      uniqueInOrder,
      ["QUEUED", "RUNNING", "SUCCEEDED"].slice(0, uniqueInOrder.length),
      `state sequence must be a QUEUED-first monotone prefix, got ${states.join(" -> ")}`,
    );
    assert.ok(trace.final.body.terminal === true);
    // Cold-start budget respected: SUCCEEDED no earlier than 4500ms - jitter.
    assert.ok(trace.final.atMs >= 4350, `SUCCEEDED observed at ${trace.final.atMs}ms, before the declared cold-start window`);
  });
});

describe("Databricks POST /api/2.0/sql/statements", () => {
  it("returns 200 with statement_id", async () => {
    const res = await fetch(`${server.baseUrl}${DATABRICKS_SUBMIT}`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        statement: "SELECT 1",
        warehouseId: "abc123",
        catalog: "main",
        schema: "default",
        waitTimeout: "0s",
      }),
    });
    assert.equal(res.status, 200, `expected 200, got ${res.status}`);
    const body = await json(res);
    assert.ok(uuidShape(body.statement_id), `statement_id not a UUID: ${body.statement_id}`);
    assert.equal(body.status.state, "QUEUED");
    assert.equal(body.pollUrl, `/api/2.0/sql/statements/${body.statement_id}`);
    // Databricks has no Location header contract in the rendered route:
    assert.equal(res.headers.get("location"), null);
  });

  it("poll reaches terminal or records the REAL state within budget", async () => {
    const submit = await fetch(`${server.baseUrl}${DATABRICKS_SUBMIT}`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ statement: "SELECT 1", warehouseId: "abc123", catalog: "main", schema: "default", waitTimeout: "0s" }),
    });
    const { statement_id } = await json(submit);

    const first = await fetch(`${server.baseUrl}${DATABRICKS_SUBMIT}/${statement_id}`);
    assert.equal(first.status, 200);
    const firstBody = await json(first);
    assert.equal(firstBody.status.state, "QUEUED");
    assert.equal(firstBody.statement_id, statement_id);
    assert.equal(firstBody.terminal, false);

    const trace = await pollUntil<any>(
      `${server.baseUrl}${DATABRICKS_SUBMIT}/${statement_id}`,
      (b) => b.status?.state === "SUCCEEDED" || b.status?.state === "FAILED",
      { budgetMs: DATABRICKS_POLL_BUDGET_MS, intervalMs: 500 },
    );
    const states = statesOf(trace).filter((s) => s !== "_nonJson");
    console.log(`[FINDING] databricks poll trace (${trace.observations.length} polls, ${trace.elapsedMs}ms): ${states.join(" -> ")}`);
    // Contract: 3800ms cold start, then RUNNING -> SUCCEEDED with response_code 200.
    assert.equal(trace.settled, true, `poll did not reach terminal within ${DATABRICKS_POLL_BUDGET_MS}ms; got ${states.join(" -> ")}`);
    assert.equal(trace.final.body.status.state, "SUCCEEDED");
    assert.equal(trace.final.body.status.response_code, 200);
    assert.equal(trace.final.status, 200);
    assert.ok(states[0] === "QUEUED", `first observation must be QUEUED, got ${states[0]}`);
    const uniqueInOrder = [...new Set(states)];
    assert.deepEqual(
      uniqueInOrder,
      ["QUEUED", "RUNNING", "SUCCEEDED"].slice(0, uniqueInOrder.length),
      `state sequence must be a QUEUED-first monotone prefix, got ${states.join(" -> ")}`,
    );
    assert.equal(trace.final.status, 200);
  });
});

describe("error paths (REAL rendered behavior asserted)", () => {
  it("unknown handle: route does NOT implement 404 — returns 202 QUEUED for arbitrary UUID", async () => {
    const unknown = crypto.randomUUID();
    const res = await fetch(`${server.baseUrl}${SNOWFLAKE_SUBMIT}/${unknown}`);
    const body = await json(res);
    // FINDING: no handle registry exists; any UUID is accepted as QUEUED.
    assert.equal(res.status, 202);
    assert.equal(body.state, "QUEUED");
    assert.equal(body.handle, unknown);
  });

  it("wrong method on submit route: Next.js returns 405", async () => {
    for (const method of ["PUT", "DELETE"]) {
      const res = await fetch(`${server.baseUrl}${SNOWFLAKE_SUBMIT}`, { method });
      assert.equal(res.status, 405, `expected 405 for ${method}, got ${res.status}`);
      await res.arrayBuffer();
    }
    const resDb = await fetch(`${server.baseUrl}${DATABRICKS_SUBMIT}`, { method: "PUT" });
    assert.equal(resDb.status, 405, `expected 405 for Databricks PUT, got ${resDb.status}`);
    await resDb.arrayBuffer();
  });

  it("missing auth header: route does NOT implement auth — 202 without Authorization", async () => {
    // FINDING: AUTH_SCHEME "Bearer" is declared in the route constants but no
    // auth check exists in POST or GET. Real behavior: unauthenticated submit
    // is accepted.
    const res = await fetch(`${server.baseUrl}${SNOWFLAKE_SUBMIT}`, {
      method: "POST",
      headers: { "content-type": "application/json" }, // no Authorization
      body: JSON.stringify({ sqlText: "SELECT 1", warehouse: "WH", database: "DB", schema: "S", asyncMode: true }),
    });
    assert.equal(res.status, 202, `unauthenticated submit got ${res.status}; real behavior recorded`);
    await json(res);
    const poll = await fetch(`${server.baseUrl}${SNOWFLAKE_SUBMIT}/${crypto.randomUUID()}`);
    assert.equal(poll.status, 202, `unauthenticated poll got ${poll.status}`);
    await poll.arrayBuffer();
  });

  it("malformed JSON body: real behavior recorded", async () => {
    const res = await fetch(`${server.baseUrl}${SNOWFLAKE_SUBMIT}`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: "not-json{",
    });
    // Next.js route handler that throws in request.json() -> 500. Record it.
    console.log(`[FINDING] malformed JSON body -> HTTP ${res.status}`);
    assert.ok([400, 500].includes(res.status), `unexpected status for malformed body: ${res.status}`);
    await res.arrayBuffer();
  });
});

describe("determinism: same request twice", () => {
  it("same submit twice -> same status/shape, fresh handles; same poll state sequence", async () => {
    const submit = () =>
      fetch(`${server.baseUrl}${SNOWFLAKE_SUBMIT}`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ sqlText: "SELECT 1", warehouse: "WH", database: "DB", schema: "S", asyncMode: true }),
      }).then((r) => r.json());

    const [a, b] = await Promise.all([submit(), submit()]);
    assert.equal(Object.keys(a).sort().join(","), Object.keys(b).sort().join(","));
    assert.notEqual(a.statementHandle, b.statementHandle, "handles should be freshly minted UUIDs");
    assert.equal(a.status, b.status);

    // Two identical polls (different fresh handles) must yield the same
    // state sequence over the observation window.
    const poll = (h: string) =>
      pollUntil<any>(
        `${server.baseUrl}${SNOWFLAKE_SUBMIT}/${h}`,
        (b) => b.state === "SUCCEEDED" || b.state === "FAILED",
        { budgetMs: SNOWFLAKE_POLL_BUDGET_MS, intervalMs: 500 },
      );
    const [ta, tb] = await Promise.all([poll(a.statementHandle), poll(b.statementHandle)]);
    const states = (t: typeof ta) => statesOf(t).filter((s) => s !== "_nonJson");
    // Determinism: identical requests walk the identical state sequence
    // (per-handle jitter is FNV-1a over a fixed seed, never wall-clock).
    const sa = states(ta), sb = states(tb);
    assert.deepEqual([...new Set(sa)], [...new Set(sb)], `state trajectories diverge: ${sa.join(" -> ")} vs ${sb.join(" -> ")}`);
    assert.equal(sa.at(-1), "SUCCEEDED", `expected terminal SUCCEEDED, got ${sa.join(" -> ")}`);
    assert.equal(sb.at(-1), "SUCCEEDED", `expected terminal SUCCEEDED, got ${sb.join(" -> ")}`);
  });
});
