// ValueLedger — lane V1 (frozen-duckdb repo).
//
// Takes REAL court receipts (events ingested, compression ratio, query
// latencies, iterations) and computes attested unit values per the
// operator's formulas. All arithmetic derives from
// economics/rate_cards.ts constants — nothing hardcoded below the rate
// layer. Every computed figure is ASSUMPTION-UNVERIFIED until a customer
// bill is audited; the ledger attests the arithmetic, not the prices.
import {
  DATADOG,
  SNOWFLAKE,
  CELONIS,
  AMPLITUDE,
  SNOWFLAKE_LARGE_PER_HOUR,
  AMPLITUDE_MONTHLY_USD,
} from "./rate_cards.ts";

export interface CourtReceiptInput {
  /** Events actually ingested in the run (real receipt). */
  eventsIngested: number;
  /** Compression ratio observed in the run (e.g. 20 means 20x). */
  compressionRatio: number;
  /** Real per-query latencies in ms from the court run. */
  queryLatenciesMs: number[];
  /** Iterations executed in the court run. */
  iterations: number;
}

export interface AttestedUnitValues {
  /** $/run for the event batch (Datadog index pricing). */
  eventRunUsd: number;
  /** $ per event at the observed batch size. */
  perEventUsd: number;
  /** Celonis $/mo equivalence (TEI annual / 12). */
  celonisMonthlyUsd: number;
  /** $ for the whole 1k-query battery (Snowflake Large hourly). */
  queryBatteryUsd: number;
  /** $ per query in the battery. */
  perQueryUsd: number;
  /** Amplitude $/mo equivalence (Tier 2 annual / 12). */
  amplitudeMonthlyUsd: number;
  /** Post-compression egress-equivalent bytes. */
  egressBytesPostCompression: number;
}

/** Round to 2 decimals (money). */
export function usd(x: number): number {
  return Math.round(x * 100) / 100;
}

export class ValueLedger {
  /**
   * Operator formula: $1.25 per 500k-event run. Derived, not hardcoded:
   * events/1M x DATADOG.eventsIndexedPer1mUsd. 500k -> $1.25 exactly.
   */
  eventRunCost(events: number): number {
    return (events / 1_000_000) * DATADOG.eventsIndexedPer1mUsd;
  }

  /**
   * Operator formula: $105k/mo Celonis equivalence. Derived:
   * CELONIS.teiAnnualUsd / 12 (must equal CELONIS.monthlyUsd).
   */
  celonisMonthly(): number {
    const derived = CELONIS.teiAnnualUsd / 12;
    if (usd(derived) !== usd(CELONIS.monthlyUsd)) {
      throw new Error(
        `rate card inconsistency: TEI/12 = ${derived} != monthly ${CELONIS.monthlyUsd}`,
      );
    }
    return derived;
  }

  /**
   * Operator formula: $33.12 per 1k-query battery. Derived from the real
   * receipt latencies: total wall hours x SNOWFLAKE_LARGE_PER_HOUR.
   * (1000 queries x 4.968 s avg = 4968 s = 1.38 h; 1.38 x $24 = $33.12.)
   */
  queryBatteryCost(queryLatenciesMs: number[]): number {
    const totalMs = queryLatenciesMs.reduce((a, b) => a + b, 0);
    const hours = totalMs / 3_600_000;
    return hours * SNOWFLAKE_LARGE_PER_HOUR;
  }

  /** Operator formula: $20,833.33/mo. Derived: Amplitude Tier 2 annual / 12. */
  amplitudeMonthly(): number {
    return AMPLITUDE.tier2AnnualUsd / 12;
  }

  /** Attest all unit values for one real court receipt. */
  attest(receipt: CourtReceiptInput): AttestedUnitValues {
    const eventRunUsd = this.eventRunCost(receipt.eventsIngested);
    const queryBatteryUsd = this.queryBatteryCost(receipt.queryLatenciesMs);
    return {
      eventRunUsd: usd(eventRunUsd),
      perEventUsd: usd(eventRunUsd / receipt.eventsIngested),
      celonisMonthlyUsd: usd(this.celonisMonthly()),
      queryBatteryUsd: usd(queryBatteryUsd),
      perQueryUsd: usd(queryBatteryUsd / receipt.queryLatenciesMs.length),
      amplitudeMonthlyUsd: usd(this.amplitudeMonthly()),
      egressBytesPostCompression: Math.round(
        receipt.iterations * receipt.eventsIngested / receipt.compressionRatio,
      ),
    };
  }

  /** Full monthly equivalence stack from one attested receipt. */
  monthlyEquivalence(): { celonisUsd: number; amplitudeUsd: number; snowflakeLargeHoursUsd: number } {
    return {
      celonisUsd: usd(this.celonisMonthly()),
      amplitudeUsd: usd(AMPLITUDE_MONTHLY_USD),
      snowflakeLargeHoursUsd: usd(SNOWFLAKE_LARGE_PER_HOUR),
    };
  }
}
