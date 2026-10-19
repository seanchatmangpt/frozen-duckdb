// HAND-WRITTEN (non-generated admin surface); ggen owns app/api/v2 + app/api/2.0.
// REGENERATION LAW: `ggen sync run` MUST NOT clobber this file — coordinate
// with lane G2's manifest output list. Courts read this endpoint for liveness
// standing; a rendered overwrite would strand them.
//
// GAP12: moved from app/api/_admin to app/api/admin — underscore-prefixed
// folders are Next.js PRIVATE folders and do not route.

import { getAdminEngine, ADMIN_ENDPOINTS } from "../../../../lib/admin_engine.ts";

const BOOT_MS = Date.now();

/** GET /api/admin/health — liveness + sim identity for court receipts. */
export async function GET(): Promise<Response> {
  const engine = getAdminEngine();
  const state = engine.inspect();
  const body = {
    up: true,
    seed: state.seed,
    endpointCount: ADMIN_ENDPOINTS.length,
    uptimeMs: Date.now() - BOOT_MS,
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
