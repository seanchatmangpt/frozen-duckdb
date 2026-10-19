// HAND-WRITTEN (non-generated admin surface); ggen owns app/api/v2 + app/api/2.0.
// REGENERATION LAW: `ggen sync run` MUST NOT clobber this file — coordinate
// with lane G2's manifest output list before adding any admin path to the
// rendered set. Mutation courts depend on this route's exact standing.
//
// GAP12: moved from app/api/_admin (a Next.js PRIVATE folder — underscore
// prefixes do not route) to app/api/admin so the ledger is reachable over
// real HTTP. The engine singleton itself lives in lib/admin_engine.ts.

import {
  getAdminEngine,
  egressDelta,
  type AdminLedgerResponse,
} from "../../../../lib/admin_engine.ts";

/** GET /api/admin/ledger — full ledger JSON: per-query records + totals
 * (creditsBurned, scanFeesUsd, egressBytes, queryCount), read from the SAME
 * engine instance the traffic path uses. egressDelta() folds in egress bytes
 * recorded after submit (terminal polls) onto the true total. */
export async function GET(): Promise<Response> {
  const engine = getAdminEngine();
  const state = engine.inspect();
  const totals = {
    ...state.ledger,
    egressBytes: state.ledger.egressBytes + egressDelta(),
  };
  const body: AdminLedgerResponse = {
    engine: "sim_engine",
    seed: state.seed,
    queryCount: state.requestCount,
    totals,
    records: state.records,
  };
  return new Response(JSON.stringify(body), {
    status: 200,
    headers: { "content-type": "application/json" },
  });
}

async function methodNotAllowed(): Promise<Response> {
  return new Response(JSON.stringify({ error: "method not allowed" }), {
    status: 405,
    headers: { allow: "GET", "content-type": "application/json" },
  });
}

export const POST = methodNotAllowed;
export const PUT = methodNotAllowed;
export const DELETE = methodNotAllowed;
export const PATCH = methodNotAllowed;
