// VERIFIED LIST PRICES as supplied by the operator 2026-10-19; every figure
// ASSUMPTION-UNVERIFIED until a customer bill is audited — these are
// counterfactual rate-card parameters, not measurements.
//
// Lane V1 (frozen-duckdb). Consumers: economics/value_ledger.ts and its
// court (tests/value_ledger.test.ts). Prices are the arithmetic inputs to
// the ledger's attested unit-value formulas; a figure here is never a claim
// about what anyone actually paid.

// --- Datadog -----------------------------------------------------------
// $0.10 per GB of ingest; $2.50 per 1M events indexed (30-day retention).
export const DATADOG = {
  ingestPerGbUsd: 0.1,
  eventsIndexedPer1mUsd: 2.5,
  retentionDays: 30,
} as const;

// --- Snowflake ---------------------------------------------------------
// $3 per credit (Enterprise edition); Medium = 4 credits/hr, Large = 8
// credits/hr.
export const SNOWFLAKE = {
  creditUsd: 3,
  mediumCreditsPerHour: 4,
  largeCreditsPerHour: 8,
} as const;

/** Medium warehouse: $12/hr. */
export const SNOWFLAKE_MEDIUM_PER_HOUR = SNOWFLAKE.creditUsd * SNOWFLAKE.mediumCreditsPerHour; // 12
/** Large warehouse: $24/hr. */
export const SNOWFLAKE_LARGE_PER_HOUR = SNOWFLAKE.creditUsd * SNOWFLAKE.largeCreditsPerHour; // 24

// --- Celonis -----------------------------------------------------------
// $1.26M/yr TEI baseline; quoted as $105k/mo.
export const CELONIS = {
  teiAnnualUsd: 1_260_000,
  monthlyUsd: 105_000,
} as const;

// --- Amplitude ---------------------------------------------------------
// $250k/yr Tier 2.
export const AMPLITUDE = {
  tier2AnnualUsd: 250_000,
} as const;

/** Amplitude monthly run-rate: $250k/12 = $20,833.33/mo. */
export const AMPLITUDE_MONTHLY_USD = AMPLITUDE.tier2AnnualUsd / 12; // 20833.333...
