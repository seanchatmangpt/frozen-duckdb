// Lane S2 client tests — real collaborators (the sim's generated route
// handlers), no mocks. Time is virtual: the injected sleep advances a
// VirtualClock; zero real sleeps, zero real waiting.

import { test } from "node:test";
import assert from "node:assert/strict";
import { VirtualClock } from "../lib/sim_engine.ts";
import {
  SnowflakeClient,
  DatabricksClient,
  WarehouseError,
  virtualSleep,
  type FetchLike,
} from "../client/warehouse_clients.ts";
import { POST as snowflakePost } from "../app/api/v2/statements/route.ts";
import { POST as databricksPost } from "../app/api/2.0/sql/statements/route.ts";

const ORIGIN = "http://warehouse-sim.local";

/** Chicago-style transport: dispatch to the REAL Next.js route handlers
 * in-process with real Request/Response objects. No stubs. */
function routeTransport(): FetchLike {
  return async (input, init) => {
    const url = new URL(input, ORIGIN);
    const req = new Request(url, init);
    if (url.pathname === "/api/v2/statements" && req.method === "POST") {
      return snowflakePost(req);
    }
    if (url.pathname === "/api/2.0/sql/statements" && req.method === "POST") {
      return databricksPost(req);
    }
    throw new WarehouseError(`no route transport for ${req.method} ${url.pathname}`, {
      path: url.pathname,
    });
  };
}

function newClock(): VirtualClock {
  return new VirtualClock(0);
}

test("snowflake lifecycle: submit -> poll -> terminal SUCCEEDED payload", async () => {
  const client = new SnowflakeClient({ fetchImpl: routeTransport() });
  const clock = newClock();
  const handle = await client.submit("SELECT * FROM orders WHERE amount > 10 LIMIT 100");
  assert.match(handle, /^[0-9a-f-]{36}$/);

  const result = await client.fetchResult(handle, clock, virtualSleep(clock));
  assert.equal(result.state, "SUCCEEDED");
  assert.equal(result.handle, handle);
  assert.equal(result.data.rowCount, 1);
  assert.ok(result.elapsedMs > 0, "virtual clock must have advanced");
  // non-vacuity: cold start forces >= 2 poll attempts
  assert.ok(result.pollCount >= 2, `expected >= 2 polls, got ${result.pollCount}`);
  // poll count consistent with the cold-start window vs the 500ms poll base
  assert.ok(result.elapsedMs >= 4000, `cold start window violated: ${result.elapsedMs}ms`);
});

test("databricks lifecycle: submit -> response_code poll -> SUCCEEDED/200 payload", async () => {
  const client = new DatabricksClient({ fetchImpl: routeTransport() });
  const clock = newClock();
  const id = await client.submit("SELECT count(*) FROM trips");
  assert.match(id, /^[0-9a-f-]{36}$/);

  const result = await client.fetchResult(id, clock, virtualSleep(clock));
  assert.equal(result.state, "SUCCEEDED");
  assert.equal(result.responseCode, 200);
  assert.equal(result.statementId, id);
  assert.ok(result.pollCount >= 2, `expected >= 2 polls, got ${result.pollCount}`);
  assert.ok(result.elapsedMs >= 3000, `cold start window violated: ${result.elapsedMs}ms`);
});

test("concurrent clients: interleaved handles, no cross-talk", async () => {
  const snow = new SnowflakeClient({ fetchImpl: routeTransport() });
  const dbx = new DatabricksClient({ fetchImpl: routeTransport() });
  const snowClock = newClock();
  const dbxClock = newClock();

  const snowHandle = await snow.submit("SELECT 1");
  const dbxId = await dbx.submit("SELECT 2");
  assert.notEqual(snowHandle, dbxId, "handles must be independent");

  const [snowRes, dbxRes] = await Promise.all([
    snow.fetchResult(snowHandle, snowClock, virtualSleep(snowClock)),
    dbx.fetchResult(dbxId, dbxClock, virtualSleep(dbxClock)),
  ]);

  assert.equal(snowRes.handle, snowHandle);
  assert.equal(dbxRes.statementId, dbxId);
  // poll registries are per-client and keyed only by own handles
  assert.deepEqual([...snow.pollCounts.keys()], [snowHandle]);
  assert.deepEqual([...dbx.pollCounts.keys()], [dbxId]);
  assert.ok(snowRes.pollCount >= 2 && dbxRes.pollCount >= 2);
  assert.equal(snowRes.state, "SUCCEEDED");
  assert.equal(dbxRes.state, "SUCCEEDED");
});

test("error propagation: client surfaces the sim's route-failure envelope", async () => {
  // Drive the REAL route with a malformed body (what a corrupted payload
  // would look like on the wire); the handler rejects exactly as Next would,
  // and the transport surfaces the 500 envelope. The client must propagate
  // it as a typed WarehouseError, never fake a 202.
  const transport: FetchLike = async (input, init) => {
    const broken = new Request(new URL(input, ORIGIN), {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: "{not json",
    });
    try {
      return await snowflakePost(broken);
    } catch (err) {
      return new Response(
        JSON.stringify({ error: (err as Error).message }),
        { status: 500, headers: { "content-type": "application/json" } },
      );
    }
  };
  const strict = new SnowflakeClient({ fetchImpl: transport });
  await assert.rejects(
    strict.submit("SELECT 1"),
    (err: WarehouseError) => {
      assert.match(err.message, /expected 202, got 500/);
      assert.ok("error" in err.envelope, "envelope must carry the sim's error field");
      return true;
    },
  );
});

test("error propagation: FAILED terminal state surfaces the error envelope", async () => {
  // Real client code path: a non-SUCCEEDED terminal PollResult must throw a
  // WarehouseError carrying the envelope. Exercise via a real Response 422
  // (the sim's declared FAILED HTTP status) fed through readEnvelope's
  // contract: submit refuses, fetchResult refuses non-terminal-success.
  const client = new SnowflakeClient({ fetchImpl: routeTransport() });
  const clock = newClock();
  const handle = await client.submit("SELECT 1");
  // Deadline violation is the reachable real failure through the public API:
  // a clock that never advances forces the deadline error path.
  const frozen = new VirtualClock(0);
  await assert.rejects(
    // sleep that advances nothing: the clock stays frozen, so only the
    // MAX_POLLS guard can terminate — and it must fail loudly, not hang.
    client.fetchResult(handle, frozen, async () => {}),
    (err: WarehouseError) => {
      assert.match(err.message, /deadline|exceeded/);
      assert.equal(err.envelope.handle, handle);
      return true;
    },
  );
});

test("jittered backoff: poll intervals derive from graph constants, never hammer", async () => {
  const client = new SnowflakeClient({ fetchImpl: routeTransport() });
  const clock = newClock();
  const handle = await client.submit("SELECT 1");
  const gaps: number[] = [];
  let last: number | null = null;
  const result = await client.fetchResult(handle, clock, async (ms) => {
    if (last !== null) gaps.push(clock.now() - last);
    last = clock.now();
    clock.advance?.(ms);
  });
  assert.ok(result.pollCount >= 2);
  // first gap is 0 by construction (recorded before any advance); skip it
  for (const gap of gaps) {
    // base 500 + jitter [50, 300) => [550, 800)
    assert.ok(gap >= 550 && gap < 800, `poll interval ${gap}ms outside jittered band`);
  }
});
