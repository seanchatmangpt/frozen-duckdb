// Minimal-fidelity warehouse clients (lane S2). Mimic real driver flows
// against the warehouse-sim Next.js routes. Hand-written consumer layer —
// the sim's generated routes under app/api/ are the collaborators.
//
// Virtual clock law (no-sleep doctrine): the sim's rendered routes export
// pollUntilTerminal(handle, clock, sleepMs) with injectable clock/sleep —
// that IS the sim's fast-forward surface (no seed/FAST_FORWARD query param
// exists on the rendered GET handlers). Tests advance a VirtualClock
// instead of sleeping real time; no fixed sleeps anywhere.

import {
  snowflakePollUntilTerminal as pollUntilTerminal,
  databricksPollUntilTerminal as dbPollUntilTerminal,
  SNOWFLAKE_PARAMS,
  DATABRICKS_PARAMS,
  type SnowflakeSimResult as PollResult,
  type DatabricksSimResult as DbPollResult,
} from "../lib/statement_state.ts";
type PollState = PollResult["state"];
import type { SimClock } from "../lib/sim_engine.ts";

export type FetchLike = (input: string, init?: RequestInit) => Promise<Response>;

/** Advance the virtual clock by ms and resolve immediately (no real time). */
export function virtualSleep(clock: SimClock): (ms: number) => Promise<void> {
  return async (ms: number) => {
    clock.advance?.(ms);
  };
}

export interface WarehouseClientOptions {
  /** Base URL for route-handler invocation. Default: local sim origin. */
  origin?: string;
  /** Transport override; default global fetch. */
  fetchImpl?: FetchLike;
  /** Bearer token, mirrors both drivers' token auth. */
  token?: string;
}

/** Error envelope the clients throw on non-OK terminal states. */
export class WarehouseError extends Error {
  readonly envelope: Record<string, unknown>;
  constructor(message: string, envelope: Record<string, unknown>) {
    super(message);
    this.name = "WarehouseError";
    this.envelope = envelope;
  }
}

async function readEnvelope(res: Response): Promise<Record<string, unknown>> {
  const text = await res.text();
  try {
    return JSON.parse(text) as Record<string, unknown>;
  } catch {
    return { raw: text };
  }
}

// ---------------------------------------------------------------------------
// SnowflakeClient — SQL API v2: POST /api/v2/statements (202 + Location),
// poll GET /api/v2/statements/{handle}.
// ---------------------------------------------------------------------------

export interface SnowflakeResult {
  handle: string;
  state: PollState;
  elapsedMs: number;
  /** Poll attempts made (>= 2 for a cold-started query — non-vacuity proof). */
  pollCount: number;
  data: { rows: Array<Record<string, unknown>>; rowCount: number };
}

export class SnowflakeClient {
  private readonly origin: string;
  private readonly doFetch: FetchLike;
  private readonly token: string;
  /** poll attempts per handle, for non-vacuity assertions. */
  readonly pollCounts = new Map<string, number>();

  constructor(opts: WarehouseClientOptions = {}) {
    this.origin = opts.origin ?? "http://warehouse-sim.local";
    this.doFetch = opts.fetchImpl ?? ((u, i) => fetch(u, i));
    this.token = opts.token ?? "sim-token";
  }

  /** submit(sql) -> statement handle (from the 202 Location header). */
  async submit(sql: string): Promise<string> {
    const res = await this.doFetch(`${this.origin}/api/v2/statements`, {
      method: "POST",
      headers: {
        "content-type": "application/json",
        authorization: `Bearer ${this.token}`,
      },
      body: JSON.stringify({
        sqlText: sql,
        warehouse: "SIM_WH",
        database: "SIM_DB",
        schema: "PUBLIC",
        asyncMode: true,
      }),
    });
    if (res.status !== 202) {
      throw new WarehouseError(
        `snowflake submit expected 202, got ${res.status}`,
        await readEnvelope(res),
      );
    }
    const location = res.headers.get("location");
    if (!location) throw new WarehouseError("missing Location header on 202", {});
    return location.split("/").filter(Boolean).pop() as string;
  }

  /**
   * fetchResult(handle): poll loop with jittered backoff derived from the
   * route's POLL_INTERVAL_MS/JITTER constants, deadline-bounded. The clock is
   * injected (VirtualClock in tests, SystemClock in production); the injected
   * sleep advances the clock instead of blocking.
   */
  async fetchResult(
    handle: string,
    clock: SimClock,
    sleepMs: (ms: number) => Promise<void> = virtualSleep(clock),
  ): Promise<SnowflakeResult> {
    const DEADLINE_MS = 30_000;
    const MAX_POLLS = 10_000;
    const start = clock.now();
    const jittered = async (): Promise<void> => {
      // interval = POLL_INTERVAL_MS + jitter in [JITTER_MS_MIN, JITTER_MS_MAX)
      const { POLL_INTERVAL_MS, JITTER_MS_MIN, JITTER_MS_MAX } = SNOWFLAKE_PARAMS;
      const jitter = JITTER_MS_MIN + Math.random() * (JITTER_MS_MAX - JITTER_MS_MIN);
      await sleepMs(POLL_INTERVAL_MS + jitter);
    };

    let polls = 0;
    const terminal = await pollUntilTerminal(handle, () => clock.now(), async (ms) => {
      polls += 1;
      this.pollCounts.set(handle, polls);
      if (polls > MAX_POLLS) {
        throw new WarehouseError(
          `snowflake poll exceeded ${MAX_POLLS} attempts for ${handle}`,
          { handle, polls },
        );
      }
      if (clock.now() - start > DEADLINE_MS) {
        throw new WarehouseError(
          `snowflake poll deadline (${DEADLINE_MS}ms virtual) exceeded for ${handle}`,
          { handle, polls },
        );
      }
      await jittered();
      void ms;
    });

    if (terminal.state !== "SUCCEEDED") {
      throw new WarehouseError(`snowflake statement ${handle} ended ${terminal.state}`, {
        handle,
        state: terminal.state,
      });
    }
    return {
      handle: terminal.handle,
      state: terminal.state,
      elapsedMs: terminal.elapsedMs,
      pollCount: polls,
      data: { rows: [{ handle }], rowCount: 1 },
    };
  }
}

// ---------------------------------------------------------------------------
// DatabricksClient — Statement Execution API: POST /api/2.0/sql/statements
// (200 + statement_id), poll by response_code on
// GET /api/2.0/sql/statements/{statement_id}.
// ---------------------------------------------------------------------------

export interface DatabricksResult {
  statementId: string;
  state: DbPollResult["status"]["state"];
  responseCode?: number;
  elapsedMs: number;
  pollCount: number;
  data: { dataArray: Array<Record<string, unknown>>; rowCount: number };
}

export class DatabricksClient {
  private readonly origin: string;
  private readonly doFetch: FetchLike;
  private readonly token: string;
  readonly pollCounts = new Map<string, number>();

  constructor(opts: WarehouseClientOptions = {}) {
    this.origin = opts.origin ?? "http://warehouse-sim.local";
    this.doFetch = opts.fetchImpl ?? ((u, i) => fetch(u, i));
    this.token = opts.token ?? "sim-token";
  }

  /** submit(sql) -> statement_id (200 on submit, per the sim's contract). */
  async submit(sql: string): Promise<string> {
    const res = await this.doFetch(`${this.origin}/api/2.0/sql/statements`, {
      method: "POST",
      headers: {
        "content-type": "application/json",
        authorization: `Bearer ${this.token}`,
      },
      body: JSON.stringify({
        statement: sql,
        warehouseId: "sim-warehouse",
        catalog: "main",
        schema: "default",
        waitTimeout: "0s",
      }),
    });
    if (res.status !== 200) {
      throw new WarehouseError(
        `databricks submit expected 200, got ${res.status}`,
        await readEnvelope(res),
      );
    }
    const body = (await res.json()) as { statement_id?: string };
    if (!body.statement_id) {
      throw new WarehouseError("databricks submit missing statement_id", body);
    }
    return body.statement_id;
  }

  /**
   * fetchResult(statementId): response_code polling with jittered backoff,
   * deadline-bounded, injectable clock (VirtualClock in tests).
   */
  async fetchResult(
    statementId: string,
    clock: SimClock,
    sleepMs: (ms: number) => Promise<void> = virtualSleep(clock),
  ): Promise<DatabricksResult> {
    const DEADLINE_MS = 30_000;
    const MAX_POLLS = 10_000;
    const start = clock.now();
    const { POLL_INTERVAL_MS, JITTER_MS_MIN, JITTER_MS_MAX } = DATABRICKS_PARAMS;

    let polls = 0;
    const terminal = await dbPollUntilTerminal(statementId, () => clock.now(), async () => {
      polls += 1;
      this.pollCounts.set(statementId, polls);
      if (polls > MAX_POLLS) {
        throw new WarehouseError(
          `databricks poll exceeded ${MAX_POLLS} attempts for ${statementId}`,
          { statementId, polls },
        );
      }
      if (clock.now() - start > DEADLINE_MS) {
        throw new WarehouseError(
          `databricks poll deadline (${DEADLINE_MS}ms virtual) exceeded for ${statementId}`,
          { statementId, polls },
        );
      }
      const jitter = JITTER_MS_MIN + Math.random() * (JITTER_MS_MAX - JITTER_MS_MIN);
      await sleepMs(POLL_INTERVAL_MS + jitter);
    });

    if (terminal.status.state !== "SUCCEEDED" || terminal.status.response_code !== 200) {
      throw new WarehouseError(
        `databricks statement ${statementId} ended ${terminal.status.state}`,
        { statementId, status: terminal.status },
      );
    }
    return {
      statementId: terminal.statement_id,
      state: terminal.status.state,
      responseCode: terminal.status.response_code,
      elapsedMs: terminal.elapsedMs,
      pollCount: polls,
      data: { dataArray: [{ statement_id: statementId }], rowCount: 1 },
    };
  }
}
