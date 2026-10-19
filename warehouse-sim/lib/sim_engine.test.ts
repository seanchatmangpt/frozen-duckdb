// sim_engine tests — node:test zero-dep (no test runner in devDependencies).
// Chicago-style: real engine, real state assertions, no mocks. Deterministic
// virtual clock — no real timers.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  SimEngine,
  VirtualClock,
  estimateBytesScanned,
  makePrng,
  sqlHash,
  type EndpointParams,
} from "./sim_engine.ts";

const endpoints: EndpointParams[] = [
  { match: "/api/query", coldStartMs: 1200, sizeMultiplier: 2.0, scanRatePerTB: 85 },
  { match: "/api/heavy", coldStartMs: 3000, sizeMultiplier: 8.0, scanRatePerTB: 200 },
];

function freshEngine(seed = 42) {
  return new SimEngine({ seed, endpoints, clock: new VirtualClock() });
}

test("cold-start: first request pays cold start, warm requests do not", () => {
  const eng = freshEngine();
  const r1 = eng.simulateRequest({
    url: "/api/query",
    sql: "SELECT 1",
    payloadBytes: 100,
    responseBytes: 512,
  });
  assert.equal(r1.coldStartAppliedMs, 1200, "first request pays cold start");

  const r2 = eng.simulateRequest({
    url: "/api/query",
    sql: "SELECT 1",
    payloadBytes: 100,
    responseBytes: 512,
  });
  assert.equal(r2.coldStartAppliedMs, 0, "second request is warm");
});

test("cold-start ordering: endpoints warm independently", () => {
  const eng = freshEngine();
  eng.simulateRequest({ url: "/api/query", sql: "SELECT 1", payloadBytes: 0, responseBytes: 0 });
  eng.simulateRequest({ url: "/api/heavy", sql: "SELECT 1", payloadBytes: 0, responseBytes: 0 });
  const st = eng.inspect();
  assert.deepEqual(st.warmedEndpoints.sort(), ["/api/heavy", "/api/query"]);
});

test("cold-start ordering: latency includes cold start before jitter", () => {
  const eng = freshEngine(7);
  const r = eng.simulateRequest({
    url: "/api/heavy",
    sql: "SELECT 1",
    payloadBytes: 0,
    responseBytes: 0,
  });
  // jitter in [50,300), execution in [40,140]
  assert.equal(r.coldStartAppliedMs, 3000);
  assert.ok(r.jitterMs >= 50 && r.jitterMs < 300, `jitter ${r.jitterMs} out of range`);
  assert.ok(
    r.latencyMs === r.coldStartAppliedMs + r.jitterMs + r.executionTimeMs,
    "latency = cold + jitter + exec",
  );
  assert.ok(r.completedAtMs === r.latencyMs, "virtual clock advanced by latency");
});

test("ledger math exactness: fixed case, hand-computed", () => {
  const eng = new SimEngine({
    seed: 1,
    endpoints: [
      { match: "/x", coldStartMs: 0, sizeMultiplier: 2.0, scanRatePerTB: 85 },
    ],
    clock: new VirtualClock(),
  });
  const payload = 0;
  const sql = "SELECT * FROM t"; // multiplier 1 + 1.5 (select *) + 2.0 (no where) = 4.5
  const r = eng.simulateRequest({ url: "/x", sql, payloadBytes: payload, responseBytes: 1000 });

  const bytesScanned = estimateBytesScanned(sql, payload);
  assert.equal(bytesScanned, Math.round(1e9 * 4.5), "scan estimate = 4.5 GB");
  assert.equal(r.bytesScanned, bytesScanned);

  // executionTimeMs = 40 + 0/100 = 40; hours = 40/3.6e6
  const credits = (40 / 3_600_000) * 2.0 * 8.0;
  assert.equal(r.executionTimeMs, 40);
  assert.ok(
    Math.abs(r.creditsBurned - credits) < 1e-15,
    `credits ${r.creditsBurned} != ${credits}`,
  );

  const fee = (bytesScanned / 1e12) * 85;
  assert.ok(Math.abs(r.scanFeeUsd - fee) < 1e-18, `fee ${r.scanFeeUsd} != ${fee}`);

  const led = eng.ledger();
  assert.equal(led.egressBytes, 1000);
  assert.equal(led.creditsBurned, r.creditsBurned);
  assert.equal(led.scanFeesUsd, r.scanFeeUsd);
});

test("ledger accumulates across requests", () => {
  const eng = freshEngine(9);
  for (let i = 0; i < 3; i++) {
    eng.simulateRequest({ url: "/api/query", sql: "SELECT 1", payloadBytes: i * 10, responseBytes: i * 100 });
  }
  const led = eng.ledger();
  const recs = eng.inspect().records;
  assert.equal(recs.length, 3);
  const sumCredits = recs.reduce((s, r) => s + r.creditsBurned, 0);
  const sumFees = recs.reduce((s, r) => s + r.scanFeeUsd, 0);
  assert.equal(led.egressBytes, 0 + 100 + 200);
  assert.ok(Math.abs(led.creditsBurned - sumCredits) < 1e-15);
  assert.ok(Math.abs(led.scanFeesUsd - sumFees) < 1e-18);
});

test("determinism: same seed → identical sequence", () => {
  const a = freshEngine(1234);
  const b = freshEngine(1234);
  const reqs = [
    { url: "/api/query", sql: "SELECT * FROM events WHERE id = 1", payloadBytes: 50, responseBytes: 2048 },
    { url: "/api/heavy", sql: "SELECT * FROM big GROUP BY x LIMIT 10", payloadBytes: 900, responseBytes: 4096 },
    { url: "/api/query", sql: "SELECT count(*) FROM t", payloadBytes: 10, responseBytes: 64 },
  ];
  for (const r of reqs) {
    a.simulateRequest(r);
    b.simulateRequest(r);
  }
  assert.deepEqual(b.inspect(), a.inspect(), "same seed must replay identically");
});

test("determinism: different seed → different jitter sequence", () => {
  const a = freshEngine(1);
  const b = freshEngine(2);
  const ja = a.simulateRequest({ url: "/api/query", sql: "SELECT 1", payloadBytes: 0, responseBytes: 0 }).jitterMs;
  const jb = b.simulateRequest({ url: "/api/query", sql: "SELECT 1", payloadBytes: 0, responseBytes: 0 }).jitterMs;
  assert.notEqual(ja, jb);
});

test("prng unit: mulberry32 is deterministic and in [0,1)", () => {
  const p1 = makePrng(77);
  const p2 = makePrng(77);
  for (let i = 0; i < 100; i++) {
    const v = p1();
    assert.ok(v >= 0 && v < 1);
    assert.equal(v, p2());
  }
});

test("sqlHash: deterministic, differs per input", () => {
  assert.equal(sqlHash("SELECT 1"), sqlHash("SELECT 1"));
  assert.notEqual(sqlHash("SELECT 1"), sqlHash("SELECT 2"));
});

test("egress: response bytes recorded per record and aggregated", () => {
  const eng = freshEngine(5);
  eng.simulateRequest({ url: "/api/query", sql: "SELECT 1", payloadBytes: 0, responseBytes: 700 });
  eng.simulateRequest({ url: "/api/heavy", sql: "SELECT 1", payloadBytes: 0, responseBytes: 300 });
  assert.equal(eng.ledger().egressBytes, 1000);
});

test("no endpoint match throws typed error", () => {
  const eng = freshEngine();
  assert.throws(() => eng.simulateRequest({ url: "/nope", sql: "s", payloadBytes: 0, responseBytes: 0 }), /no endpoint/);
});

test("reset clears state and replays identically", () => {
  const eng = freshEngine(31337);
  const run = () => {
    const rs = [
      eng.simulateRequest({ url: "/api/query", sql: "SELECT * FROM t", payloadBytes: 5, responseBytes: 100 }),
      eng.simulateRequest({ url: "/api/heavy", sql: "SELECT 1 WHERE x=1", payloadBytes: 50, responseBytes: 200 }),
    ];
    return { led: eng.ledger(), recs: eng.inspect().records };
  };
  const first = run();
  eng.reset();
  const st = eng.inspect();
  assert.equal(st.requestCount, 0);
  assert.equal(st.warmedEndpoints.length, 0);
  assert.equal(st.ledger.creditsBurned, 0);
  assert.equal(st.ledger.scanFeesUsd, 0);
  assert.equal(st.ledger.egressBytes, 0);
  assert.equal(st.virtualNowMs, 0, "virtual clock rewound");
  const second = run();
  assert.deepEqual(second, first, "post-reset replay must match first run");
});
