// Warehouse simulation engine — hand-written (the real logic; generated routes call it).
// Deterministic: seeded PRNG + injectable virtual clock. No real timers anywhere.

/** Ledger constants. NOTE: if lane G2 lands lib/ledger.ts, these MUST be imported
 * from there and this local declaration deleted (never redeclare). */
export const LEDGER = {
  /** credits per execution-hour per size-multiplier unit */
  creditsPerHourBase: 8.0,
  /** dollars per TB scanned */
  defaultScanRatePerTB: 85.0,
} as const;

export interface EndpointParams {
  /** path fragment matched against the request URL */
  match: string;
  /** virtual cold-start delay applied once before first request is served */
  coldStartMs: number;
  /** multiplies credit burn */
  sizeMultiplier: number;
  /** $/TB for this endpoint's backing warehouse */
  scanRatePerTB: number;
}

export interface SimClock {
  /** virtual milliseconds */
  now(): number;
  /** optional: advance virtual time by the request latency */
  advance?(ms: number): void;
}

/** Deterministic virtual clock; advanced explicitly. */
export class VirtualClock implements SimClock {
  private t: number;
  constructor(startMs = 0) {
    this.t = startMs;
  }
  now(): number {
    return this.t;
  }
  advance(ms: number): void {
    this.t += ms;
  }
}

/** Real clock for production use (never used in tests). */
export class SystemClock implements SimClock {
  now(): number {
    return Date.now();
  }
}

/** mulberry32 — small, fast, fully deterministic 32-bit PRNG. */
export function makePrng(seed: number): () => number {
  let a = seed >>> 0;
  return function () {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

export interface SimRequest {
  url: string;
  method?: string;
  /** raw SQL text (hashed into the record; never stored verbatim) */
  sql: string;
  /** request payload size in bytes (drives scan estimate heuristic) */
  payloadBytes: number;
  /** actual HTTP response byte count (egress accounting) */
  responseBytes: number;
}

export interface QueryRecord {
  url: string;
  method: string;
  sqlHash: string;
  bytesScanned: number;
  /** virtual wall-clock latency in ms, cold-start + jitter included */
  latencyMs: number;
  coldStartAppliedMs: number;
  jitterMs: number;
  executionTimeMs: number;
  creditsBurned: number;
  scanFeeUsd: number;
  egressBytes: number;
  /** virtual clock timestamp at completion */
  completedAtMs: number;
}

export interface LedgerTotals {
  creditsBurned: number;
  scanFeesUsd: number;
  egressBytes: number;
}

export interface SimState {
  seed: number;
  startedAtMs: number;
  virtualNowMs: number;
  requestCount: number;
  warmedEndpoints: string[];
  ledger: LedgerTotals;
  records: QueryRecord[];
}

export interface SimEngineOptions {
  seed: number;
  endpoints: EndpointParams[];
  clock?: SimClock;
  scanRatePerTB?: number;
}

/** bytesScanned heuristic: structural scan estimate from SQL shape + payload. */
export function estimateBytesScanned(sql: string, payloadBytes: number): number {
  const lower = sql.toLowerCase();
  let multiplier = 1.0;
  if (/\bselect\s+\*/.test(lower)) multiplier += 1.5; // full-row projection
  if (!/\bwhere\b/.test(lower)) multiplier += 2.0; // unfiltered scan
  if (/\bgroup\s+by\b|\border\s+by\b/.test(lower)) multiplier += 0.5;
  if (/\blimit\b/.test(lower)) multiplier *= 0.75;
  // base scan floor 1 GB + payload coupling
  return Math.round((1e9 + payloadBytes * 1024) * multiplier);
}

export class SimEngine {
  private readonly seed: number;
  private prng: () => number;
  private readonly clock: SimClock;
  private readonly endpoints: EndpointParams[];
  private readonly defaultScanRatePerTB: number;
  private readonly warmed: Set<string> = new Set();
  private readonly records: QueryRecord[] = [];
  private creditsBurned = 0;
  private scanFeesUsd = 0;
  private egressBytes = 0;

  constructor(opts: SimEngineOptions) {
    this.seed = opts.seed >>> 0;
    this.prng = makePrng(this.seed);
    this.clock = opts.clock ?? new VirtualClock();
    this.endpoints = opts.endpoints;
    this.defaultScanRatePerTB = opts.scanRatePerTB ?? LEDGER.defaultScanRatePerTB;
  }

  /** Uniform jitter in [50, 300) ms from the seeded PRNG. */
  private jitterMs(): number {
    return 50 + this.prng() * 250;
  }

  /**
   * Simulate one query against the matched endpoint. Advances virtual time by
   * the full latency (cold start + jitter + execution); returns the record.
   */
  simulateRequest(req: SimRequest): QueryRecord {
    const ep = this.endpoints.find((e) => req.url.includes(e.match));
    if (!ep) {
      throw new Error(`sim_engine: no endpoint matches ${req.url}`);
    }

    const startT = this.clock.now();

    // cold-start: paid once per endpoint, before jitter
    let coldStartMs = 0;
    if (!this.warmed.has(ep.match)) {
      coldStartMs = ep.coldStartMs;
      this.warmed.add(ep.match);
    }
    const jitterMs = this.jitterMs();

    // execution time: derived from payload size, deterministic
    const executionTimeMs = 40 + (req.payloadBytes % 10_000) / 100; // 40..140 ms
    const latencyMs = coldStartMs + jitterMs + executionTimeMs;

    this.clock.advance?.(latencyMs);
    const completedAtMs = this.clock.now();

    const bytesScanned = estimateBytesScanned(req.sql, req.payloadBytes);
    const executionTimeHours = executionTimeMs / 3_600_000;
    const creditsBurned = executionTimeHours * ep.sizeMultiplier * LEDGER.creditsPerHourBase;
    const scanFeeUsd = (bytesScanned / 1e12) * ep.scanRatePerTB;

    const rec: QueryRecord = {
      url: req.url,
      method: req.method ?? "POST",
      sqlHash: sqlHash(req.sql),
      bytesScanned,
      latencyMs,
      coldStartAppliedMs: coldStartMs,
      jitterMs,
      executionTimeMs,
      creditsBurned,
      scanFeeUsd,
      egressBytes: req.responseBytes,
      completedAtMs,
    };

    this.records.push(rec);
    this.creditsBurned += creditsBurned;
    this.scanFeesUsd += scanFeeUsd;
    this.egressBytes += req.responseBytes;
    return rec;
  }

  ledger(): LedgerTotals {
    return {
      creditsBurned: this.creditsBurned,
      scanFeesUsd: this.scanFeesUsd,
      egressBytes: this.egressBytes,
    };
  }

  inspect(): SimState {
    return {
      seed: this.seed,
      startedAtMs: 0,
      virtualNowMs: this.clock.now(),
      requestCount: this.records.length,
      warmedEndpoints: [...this.warmed],
      ledger: this.ledger(),
      records: this.records.map((r) => ({ ...r })),
    };
  }

  reset(): void {
    this.warmed.clear();
    this.records.length = 0;
    this.creditsBurned = 0;
    this.scanFeesUsd = 0;
    this.egressBytes = 0;
    // re-seed so post-reset sequences replay identically
    this.prng = makePrng(this.seed);
    if (this.clock instanceof VirtualClock) this.clock.advance?.(-this.clock.now());
  }
}

/** FNV-1a 32-bit, hex-encoded. Deterministic, dependency-free. */
export function sqlHash(sql: string): string {
  let h = 0x811c9dc5;
  for (let i = 0; i < sql.length; i++) {
    h ^= sql.charCodeAt(i);
    h = Math.imul(h, 0x01000193);
  }
  return (h >>> 0).toString(16).padStart(8, "0");
}
