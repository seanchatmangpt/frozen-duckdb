// HAND-WRITTEN lib module (header law) — NOT generated; `ggen sync run` must
// never render over this file. GAP12 root cause 1 fix: the admin engine
// singleton previously lived in app/api/_admin/ledger/route.ts, a Next.js
// underscore-PRIVATE folder the App Router excludes from routing, so the
// ledger was unreachable over real HTTP. The singleton now lives here, shared
// by the admin routes (app/api/admin) AND the generated traffic routes (via
// their ledger-bridge hook), so real HTTP traffic and the admin ledger observe
// the SAME engine instance.
//
// Ledger-bridge seam (GAP12 root cause 2): generated submit routes call
// recordQuery() — one simulateRequest per real POST, credited at submit — and
// generated poll routes call recordEgress() on the terminal observation, so
// egress bytes land on the SAME record. Terminal-only egress keeps
// queryCount == number of real submits (never double-counted per poll).

import {
  SimEngine,
  type SimState,
  type QueryRecord,
  type EndpointParams,
} from "./sim_engine.ts";

/** Endpoint params mirror the rendered route families (v2 + 2.0). Keep in sync
 * with schema/warehouse_sim.ttl; admin reads, never writes, the rendered
 * contracts. */
export const ADMIN_ENDPOINTS: EndpointParams[] = [
  {
    match: "/api/v2/statements",
    coldStartMs: 4500,
    sizeMultiplier: 2.0,
    scanRatePerTB: 85.0,
  },
  {
    match: "/api/2.0/sql/statements",
    coldStartMs: 3800,
    sizeMultiplier: 1.5,
    scanRatePerTB: 85.0,
  },
];

const ADMIN_SEED = 0x5eed_0001;

/** Canonical engine instance. Pinned on globalThis so Next dev-mode module
 * reloads cannot silently mint a second (fresh, empty) engine — the
 * fresh-instance ledger is the vacuity trap this accessor forecloses. */
interface AdminGlobal {
  __warehouseSimAdminEngine?: SimEngine;
}
const g = globalThis as typeof globalThis & AdminGlobal;

export function getAdminEngine(): SimEngine {
  if (!g.__warehouseSimAdminEngine) {
    g.__warehouseSimAdminEngine = new SimEngine({
      seed: ADMIN_SEED,
      endpoints: ADMIN_ENDPOINTS,
    });
  }
  return g.__warehouseSimAdminEngine;
}

// ---------------------------------------------------------------------------
// Ledger bridge: handle -> its QueryRecord (same object the engine keeps in
// state.records), plus a supplemental egress total for bytes that arrive after
// the submit-time record was minted (terminal polls). BOTH live on globalThis
// next to the engine pin: Next dev compiles one module instance PER ROUTE, so
// module-level state diverges across routes (witnessed: submit route credited
// queries from its copy while the poll route's copy saw an empty registry).
// globalThis is the one shared heap every compiled route copy observes.
// ---------------------------------------------------------------------------
interface BridgeGlobal {
  __warehouseSimAdminBridge?: {
    handles: Map<string, QueryRecord>;
    egressDelta: number;
  };
}
const gBridge = globalThis as typeof globalThis & BridgeGlobal & AdminGlobal;

function bridge(): { handles: Map<string, QueryRecord>; egressDelta: number } {
  if (!gBridge.__warehouseSimAdminBridge) {
    gBridge.__warehouseSimAdminBridge = { handles: new Map(), egressDelta: 0 };
  }
  return gBridge.__warehouseSimAdminBridge;
}

/** Called by the generated SUBMIT routes after a real POST. Credits the query
 * immediately (creditsBurned, scanFeesUsd, queryCount) with egressBytes 0;
 * egress is attached later by recordEgress on the terminal poll. */
export function recordQuery(input: {
  url: string;
  sql: string;
  payloadBytes: number;
  handle: string;
}): QueryRecord {
  const engine = getAdminEngine();
  const rec = engine.simulateRequest({
    url: input.url,
    sql: input.sql,
    payloadBytes: input.payloadBytes,
    responseBytes: 0,
  });
  bridge().handles.set(input.handle, rec);
  return rec;
}

/** Called by the generated POLL routes on the TERMINAL observation only:
 * attaches real egress bytes to the submit-time record for this handle. No-op
 * for unknown handles (never fabricates a record). */
export function recordEgress(handle: string, bytes: number): void {
  const rec = bridge().handles.get(handle);
  if (!rec) return;
  rec.egressBytes += bytes;
  bridge().egressDelta += bytes;
}

/** Supplemental egress recorded post-submit (terminal polls); the admin ledger
 * route adds this to engine.inspect().ledger.egressBytes for the true total. */
export function egressDelta(): number {
  return bridge().egressDelta;
}

export interface AdminLedgerResponse {
  engine: "sim_engine";
  seed: number;
  queryCount: number;
  totals: SimState["ledger"];
  records: SimState["records"];
}

/** Reset the engine AND the bridge registries — the determinism seam the reset
 * route calls, so a reset ledger cannot be re-poisoned by stale handle
 * mappings. */
export function resetLedger(): void {
  getAdminEngine().reset();
  bridge().handles.clear();
  bridge().egressDelta = 0;
}
