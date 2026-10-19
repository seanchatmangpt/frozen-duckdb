// Court for lane V1 — rate cards + ValueLedger. Zero-dep node:test,
// Chicago-style: exact math on fixed inputs, no mocks, no sleeps.
//   node --test tests/value_ledger.test.ts   (or run this file directly)
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  DATADOG,
  SNOWFLAKE,
  CELONIS,
  AMPLITUDE,
  SNOWFLAKE_MEDIUM_PER_HOUR,
  SNOWFLAKE_LARGE_PER_HOUR,
  AMPLITUDE_MONTHLY_USD,
} from "../economics/rate_cards.ts";
import {
  ValueLedger,
  usd,
} from "../economics/value_ledger.ts";

test("rate card constants pin the operator's figures exactly", () => {
  assert.equal(DATADOG.ingestPerGbUsd, 0.1);
  assert.equal(DATADOG.eventsIndexedPer1mUsd, 2.5);
  assert.equal(SNOWFLAKE.creditUsd, 3);
  assert.equal(SNOWFLAKE.mediumCreditsPerHour, 4);
  assert.equal(SNOWFLAKE.largeCreditsPerHour, 8);
  assert.equal(CELONIS.teiAnnualUsd, 1_260_000);
  assert.equal(CELONIS.monthlyUsd, 105_000);
  assert.equal(AMPLITUDE.tier2AnnualUsd, 250_000);
});

test("snowflake hourly rates derive from credits x credit price", () => {
  assert.equal(SNOWFLAKE_MEDIUM_PER_HOUR, 12); // $3 x 4 credits
  assert.equal(SNOWFLAKE_LARGE_PER_HOUR, 24); // $3 x 8 credits
});

test("operator formula: $1.25 per 500k-event run (derived, not hardcoded)", () => {
  const ledger = new ValueLedger();
  assert.equal(ledger.eventRunCost(500_000), 1.25);
  assert.equal(ledger.eventRunCost(1_000_000), 2.5);
  // Proportionality: 2x events -> 2x cost (proves derivation, not lookup).
  assert.equal(ledger.eventRunCost(1_000_000), 2 * ledger.eventRunCost(500_000));
});

test("operator formula: $105k/mo Celonis equivalence (TEI annual / 12)", () => {
  const ledger = new ValueLedger();
  assert.equal(ledger.celonisMonthly(), 105_000);
});

test("operator formula: $33.12 per 1k-query battery (Large hourly x real latencies)", () => {
  const ledger = new ValueLedger();
  // 1000 queries x 4968 ms = 4,968,000 ms = 1.38 h; 1.38 x $24 = $33.12.
  const latencies = Array.from({ length: 1000 }, () => 4968);
  const cost = ledger.queryBatteryCost(latencies);
  assert.equal(usd(cost), 33.12);
  // Non-uniform latencies with the same total must cost the same
  // (proves it reads the sum, not the shape).
  const shuffled = [...latencies];
  shuffled[0] = 4968 + 1000;
  shuffled[1] = 4968 - 1000;
  assert.equal(usd(ledger.queryBatteryCost(shuffled)), 33.12);
});

test("operator formula: $20,833.33/mo Amplitude equivalence (Tier 2 annual / 12)", () => {
  const ledger = new ValueLedger();
  assert.equal(usd(ledger.amplitudeMonthly()), 20833.33);
  assert.equal(usd(AMPLITUDE_MONTHLY_USD), 20833.33);
});

test("attest(): full receipt -> exact attested unit values", () => {
  const ledger = new ValueLedger();
  const latencies = Array.from({ length: 1000 }, () => 4968);
  const v = ledger.attest({
    eventsIngested: 500_000,
    compressionRatio: 20,
    queryLatenciesMs: latencies,
    iterations: 20,
  });
  assert.equal(v.eventRunUsd, 1.25);
  assert.equal(v.perEventUsd, 0); // $1.25/500k = $0.0000025 -> rounds to 0 at money precision
  assert.equal(v.celonisMonthlyUsd, 105_000);
  assert.equal(v.queryBatteryUsd, 33.12);
  assert.equal(v.perQueryUsd, 0.03); // 33.12/1000 = 0.03312 -> 0.03
  assert.equal(v.amplitudeMonthlyUsd, 20833.33);
  assert.equal(v.egressBytesPostCompression, 500_000); // 20 iters x 500k / 20x
});

test("attest(): compression ratio changes egress bytes only, prices unchanged", () => {
  const ledger = new ValueLedger();
  const latencies = Array.from({ length: 4 }, () => 4_968_000);
  const a = ledger.attest({
    eventsIngested: 1_000_000,
    compressionRatio: 10,
    queryLatenciesMs: latencies,
    iterations: 1,
  });
  const b = ledger.attest({
    eventsIngested: 1_000_000,
    compressionRatio: 100,
    queryLatenciesMs: latencies,
    iterations: 1,
  });
  assert.equal(a.egressBytesPostCompression, 100_000);
  assert.equal(b.egressBytesPostCompression, 10_000);
  assert.equal(a.queryBatteryUsd, b.queryBatteryUsd);
  assert.equal(a.eventRunUsd, b.eventRunUsd);
});

test("celonisMonthly() fails closed on rate-card inconsistency", () => {
  const ledger = new ValueLedger();
  // Mutate the in-memory constant to simulate a drifted rate card.
  const anyCelonis = CELONIS as unknown as { monthlyUsd: number };
  const saved = anyCelonis.monthlyUsd;
  anyCelonis.monthlyUsd = 104_999;
  assert.throws(() => ledger.celonisMonthly(), /rate card inconsistency/);
  anyCelonis.monthlyUsd = saved;
  assert.equal(ledger.celonisMonthly(), 105_000);
});

test("monthlyEquivalence(): exact stack", () => {
  const ledger = new ValueLedger();
  const m = ledger.monthlyEquivalence();
  assert.deepEqual(m, {
    celonisUsd: 105_000,
    amplitudeUsd: 20833.33,
    snowflakeLargeHoursUsd: 24,
  });
});
