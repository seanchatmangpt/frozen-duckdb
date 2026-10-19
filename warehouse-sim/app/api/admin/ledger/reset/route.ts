// HAND-WRITTEN (non-generated admin surface); ggen owns app/api/v2 + app/api/2.0.
// REGENERATION LAW: `ggen sync run` MUST NOT clobber this file — coordinate
// with lane G2's manifest output list before adding any admin path to the
// rendered set. Mutation courts depend on this route's exact standing.
//
// GAP12: moved from app/api/_admin to app/api/admin; reset goes through
// resetLedger() so the bridge's handle/egress registries are cleared with the
// engine state.

import { getAdminEngine, resetLedger } from "../../../../../lib/admin_engine.ts";

/** POST /api/admin/ledger/reset — zero the sim state, re-seed the PRNG. */
export async function POST(): Promise<Response> {
  resetLedger();
  const state = getAdminEngine().inspect();
  return new Response(
    JSON.stringify({
      reset: true,
      queryCount: state.requestCount,
      totals: state.ledger,
    }),
    {
      status: 200,
      headers: { "content-type": "application/json" },
    },
  );
}

async function methodNotAllowed(): Promise<Response> {
  return new Response(JSON.stringify({ error: "method not allowed" }), {
    status: 405,
    headers: { allow: "POST", "content-type": "application/json" },
  });
}

export const GET = methodNotAllowed;
export const PUT = methodNotAllowed;
export const DELETE = methodNotAllowed;
export const PATCH = methodNotAllowed;
