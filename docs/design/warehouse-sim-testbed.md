# Warehouse-Sim Testbed — Design

Part of the frozen-duckdb v2.0.0 campaign. Source of truth:
`warehouse-sim/schema/warehouse_sim.ttl`. Companion docs:
`docs/design/economic-model.md` (two-arm measurement harness),
`warehouse-sim/README.md`.

## Pipeline

```
schema/warehouse_sim.ttl  --(ggen sync run)-->  app/api/** routes
        |                                          + lib/ledger.ts
        |                                                  |
        v                                                  v
  poll state machines +                     determinism court (byte-identical
  cold-start + ledger                       replay across runs; drift = FAIL)
  constants                                          |
        |                                            v
        +------------------------------------> economics court
                                             (3 assertions, fail-closed)
```

The graph is the single source; routes and ledger are rendered consequences.
A hand-edited route is a defect, not a shortcut — the graph-drift court
re-renders and diffs, and a non-empty diff proves the templates are no longer
the source of truth (court FAIL).

## The two court roles

**1. Zero-cloud CI fixture.** CI has no Snowflake or Databricks credentials
and must never need them. The sim is a real local HTTP server (Next.js) that
speaks both provider dialects — 202+statementHandle vs 200+statement_id, poll
state machines, cold-start windows — so the client-side integration surface
(pollers, retry/backoff, ledger accounting) is exercised against a real
process with real sockets, Chicago-style, in every CI run. Real collaborator,
zero metered cost.

**2. Empirical economics court.** The sim's virtual credit + byte ledger
(cold-start burn, per-hour warehouse rates, per-TB scan rates) turns each
simulated workload into a cost readout. The court asserts, fail-closed:

1. zero-burn equivalence: an equivalent workload that never touches a cloud
   warehouse costs 0.00 virtual credits on-device — a vacuous zero-burn sim
   (one that produces zero by construction, e.g. no work modeled) is FAIL;
2. cold-start dominance: cold-start burn dominates per-query cost for the
   exploratory reformulation loop, matching the thesis in
   `docs/design/economic-model.md`;
3. determinism: identical inputs produce byte-identical ledger receipts
   across runs — any drift is FAIL.

## Chicago framing: counterfactual fixture, not a mock

The simulator is NOT a mock of the sovereign engine. Path B (the actual
frozen-duckdb WASM/OPFS edge execution, the BP1 surface) is always the real
engine executing real queries. The sim is the **counterfactual fixture for
the CONTRAST proof**: it models what the same workload would cost and how it
would behave on the cloud path (Path A) so the two arms can be compared
without spending a cloud dollar. Mocking the real engine would verify the
test's model; modeling the counterfactual world tests the real engine against
its alternative. That is the Chicago distinction this testbed encodes.

Every cost constant in the graph is an ASSUMPTION-UNVERIFIED counterfactual
parameter (see README). The economics court's output is a counterfactual
readout, never a quote of real prices; measured truth comes only from the
two-arm harness in `docs/design/economic-model.md`, which has not run.

## Standing (2026-10-19, as witnessed on disk)

- Graph + templates + generated `lib/sim_engine.ts` + determinism court +
  sovereign probe (Path B leg) landed. `app/api/` routes NOT rendered: ggen
  dry-run fails (`FM-GEN-008 TEMPLATE_VARIABLE_MISSING`, variable
  `r.endpointLabel` in `templates/snowflake_submit_route.ts.tera`), so the
  determinism court logs a loud `[SKIP]` rather than a pass.
- Determinism/economics courts: no PASS witnessed this window; standing
  UNKNOWN (IN-FLIGHT) until the render failure is repaired and both courts
  re-run with receipts.

## Falsifiers

- **Vacuous zero-burn sim = FAIL.** If the sim reports zero burn without
  modeling work (no cold-start window, no scan bytes, no queue time), the
  zero-burn assertion passes for the wrong reason — the court refuses.
- **Determinism drift = FAIL.** Two runs of the same workload must emit
  byte-identical ledger receipts; timestamped or run-seeded output is a
  defect, and drift kills the receipt's replay property.
- **Graph drift = FAIL (templates not the source).** `ggen sync run` must
  reproduce `app/api/**` and `lib/ledger.ts` exactly. A non-empty diff after
  re-render proves someone hand-edited generated output — the routes, not the
  graph, became the source of truth. The court fails closed on that.

## See Also

`warehouse-sim/README.md` · `docs/design/economic-model.md` ·
`docs/design/court-ladder-11.md` · active window `_LANES.md`

## Testing matrix (Wave 11, S-wave; appended 2026-10-19)

Layers x owning test file x what each layer proves. Every cost
constant exercised by any layer is ASSUMPTION-UNVERIFIED (see README).

| layer | owning test file | what it proves |
|---|---|---|
| layer | owning test file | what it proves |
|---|---|---|
| route contracts | `tests/routes_contract.test.ts` (S1) | `app/api/**` matches the graph |
| SDK clients | `tests/clients.test.ts`, `client/` (S2) | real wire protocol, not a stub |
| analyst journeys | `tests/journeys.test.ts` (S3) | analyst flows succeed over real HTTP |
| chaos + concurrency | S4 live-sim probes | sane degradation under fault/load |
| disruption scorecard | S5 scorecard, `economics/` | cost contrast fail-closed |
| admin observability | S6 admin surface | missing state loud, not silent |
| admin observability | S6 admin surface | sim state observable; missing state loud, not silent |

### Chain of custody for the disruption claim

ontology(graph) -> ggen render -> real HTTP sim -> ledger counterfactual
-> scorecard assertions.

1. `warehouse-sim/schema/warehouse_sim.ttl` is the sole source; routes
   and `lib/ledger.ts` are ggen projections (hand-edit = graph drift).
2. ggen render must be byte-identical on double-sync (determinism
   court); drift = FAIL.
3. Journeys/chaos exercise the sim over real HTTP, never in-process
   fakes (Chicago: real collaborators).
4. The ledger emits byte-identical replay receipts; the counterfactual
   burn is computed from ledger state, not asserted inline.
5. The scorecard's assertions are the only place the disruption
   contrast becomes a claim — fail-closed at
   `economics/court.ts:117` until S5's minimal repair lands.

Until each layer's court returns a PASS with a real receipt, the
disruption claim stands at UNKNOWN — no layer narrates a pass.
