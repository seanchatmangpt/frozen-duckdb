// HAND-WRITTEN (non-generated admin surface tests); ggen owns app/api/v2 + app/api/2.0.
// Non-vacuity: the ledger must reflect REAL traffic on the SHARED engine
// instance — a fresh-instance ledger is the vacuity trap under test.
// Run: node --test warehouse-sim/tests/admin.test.ts (Node >= 22 type stripping).

import { test } from "node:test";
import assert from "node:assert/strict";
import { register } from "node:module";

// Route modules use relative imports with explicit .ts extensions so Node's
// type-stripping runner resolves them without a bundler.
const ledgerMod = await import("../app/api/admin/ledger/route.ts");
const healthMod = await import("../app/api/admin/health/route.ts");
const resetMod = await import("../app/api/admin/ledger/reset/route.ts");
const { getAdminEngine } = await import("../lib/admin_engine.ts");
void register; // keep import used if runner needs loaders later

type LedgerBody = {
  engine: string;
  seed: number;
  queryCount: number;
  totals: { creditsBurned: number; scanFeesUsd: number; egressBytes: number };
  records: unknown[];
};

async function jsonOf(res: Response): Promise<unknown> {
  return res.json();
}

test("health is up before any traffic, with seed and endpointCount", async () => {
  const res = await healthMod.GET();
  assert.equal(res.status, 200);
  const body = (await jsonOf(res)) as {
    up: boolean;
    seed: number;
    endpointCount: number;
    uptimeMs: number;
  };
  assert.equal(body.up, true);
  assert.equal(typeof body.seed, "number");
  assert.equal(body.endpointCount, 2);
  assert.ok(body.uptimeMs >= 0);
});

test("ledger reflects real traffic on the shared instance (queryCount >= 1)", async () => {
  await resetMod.POST();
  const engine = getAdminEngine();
  const rec = engine.simulateRequest({
    url: "/api/v2/statements/abc-123",
    method: "POST",
    sql: "SELECT * FROM sales",
    payloadBytes: 2048,
    responseBytes: 512,
  });
  assert.ok(rec.creditsBurned > 0);

  const res = await ledgerMod.GET();
  assert.equal(res.status, 200);
  const body = (await jsonOf(res)) as LedgerBody;
  assert.equal(body.engine, "sim_engine");
  assert.ok(body.queryCount >= 1, `queryCount=${body.queryCount}`);
  assert.equal(body.queryCount, 1);
  assert.equal(body.records.length, 1);
  assert.equal(body.totals.creditsBurned, rec.creditsBurned);
  assert.equal(body.totals.egressBytes, 512);
  assert.ok(body.totals.scanFeesUsd > 0);
});

test("reset zeroes state without restart (determinism seam for courts)", async () => {
  const engine = getAdminEngine();
  engine.simulateRequest({
    url: "/api/2.0/sql/statements",
    sql: "SELECT count(*) FROM events",
    payloadBytes: 128,
    responseBytes: 64,
  });
  assert.ok(engine.inspect().requestCount >= 1);
  const res = await resetMod.POST();
  assert.equal(res.status, 200);
  const body = (await jsonOf(res)) as {
    reset: boolean;
    queryCount: number;
    totals: { creditsBurned: number };
  };
  assert.equal(body.reset, true);
  assert.equal(body.queryCount, 0);
  assert.equal(body.totals.creditsBurned, 0);
  const after = await ledgerMod.GET();
  const afterBody = (await jsonOf(after)) as LedgerBody;
  assert.equal(afterBody.queryCount, 0);
  assert.equal(afterBody.records.length, 0);
});

test("health remains up and stable across traffic + reset", async () => {
  const before = (await jsonOf(await healthMod.GET())) as { up: boolean; seed: number };
  assert.equal(before.up, true);
  await resetMod.POST();
  getAdminEngine().simulateRequest({
    url: "/api/v2/statements/x",
    sql: "SELECT 1",
    payloadBytes: 16,
    responseBytes: 8,
  });
  const after = (await jsonOf(await healthMod.GET())) as { up: boolean; seed: number };
  assert.equal(after.up, true);
  assert.equal(after.seed, before.seed, "seed is stable across traffic and reset");
});

test("405 on wrong methods for every admin route", async () => {
  for (const [name, mod, allow] of [
    ["ledger", ledgerMod, ["POST", "PUT", "DELETE", "PATCH"]],
    ["health", healthMod, ["POST", "PUT", "DELETE", "PATCH"]],
    ["reset", resetMod, ["GET", "PUT", "DELETE", "PATCH"]],
  ] as const) {
    for (const m of allow) {
      const res = await (mod as Record<string, () => Promise<Response>>)[m]();
      assert.equal(res.status, 405, `${name} ${m} should be 405`);
      assert.equal(res.headers.get("allow"), name === "reset" ? "POST" : "GET");
    }
  }
});
