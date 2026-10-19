# soc-valuation-model.md

## Status

DESIGN — not measured, not priced. Every valuation coefficient in this
document is ASSUMPTION-UNVERIFIED until the BP4 measurement harness
(wasm-opfs cost harness, `docs/design/economic-model.md` falsifier) runs
against a real reference dataset and produces a machine-readable
receipt. No dollar figure appears anywhere in this file by policy:
inventing one would be fabricated precision.

Owner: blueprint lane BP15. Forward dependency: BP5 enterprise license
draft (not yet on disk as of 2026-10-19 — this doc defines the SOC
surface the license will condition on; the interface below is
provisional until BP5 lands and is re-checked against it).

## Definition

Synthetic Operational Capacity (SOC) is the metered unit of compute
displaced from cloud to end-user silicon. One unit of SOC is the
execution of one client-side query against local storage (OPFS) under
the BP1 in-browser WASM surface, observed on the client — not inferred
from server logs, not sampled, not modeled.

SOC is a count, not a currency. It measures displaced work: each unit
is a query that did NOT consume a cloud-warehouse scan fee or compute
second, because it executed on silicon the customer already owns. The
vendor's cost to serve it is bytes (WASM bundle + data sync), per BP4's
margin mechanics; SOC is the customer-side counter of that displaced
work, and the bridge between the BP4 economics and the BP5 license.

SOC is deliberately NOT:

- Not a billing meter. The license (BP5) keys condition precedent off
  gross-revenue tiers, not off SOC counts (interface section below).
- Not a market-price index. No coefficient converts SOC to dollars;
  any such conversion is ASSUMPTION-UNVERIFIED until the BP4 harness
  produces a measured cloud-path cost baseline.
- Not a privacy surface. The SOC event stream records query shape
  metadata (see Measurement basis), never row data; the log is designed
  to keep the exported stream from carrying customer data — design
  intent, not a verified guarantee.

## Measurement basis

SOC is observed client-side, in the WASM worker, at the execution
boundary. Two primitive counters:

- Query executions: count of completed `execute` RPC round-trips
  (main-thread client -> worker -> engine), incremented on engine
  completion, not on dispatch — a dispatch that OOMs the worker does
  not count as displaced cloud work.
- Scans: engine-reported per-statement scan volume (rows and/or bytes
  scanned), read from the statement-completion path.

Both counters are emitted as events on the engine's own usage stream,
which is the OCEL telemetry of BP3: each query execution IS an
object-centric event (activity = query-execution; objects = the
session, the OPFS database, the statement; attributes = scan volume,
duration, outcome). No separate telemetry pipeline is required — the
usage log is already an OCEL log by construction, and SOC reports are
derived from it by OCEL-style derivation (filter + group + count), not
by a parallel accounting path that could drift from the execution log.

Counted vs not counted (normative):

- Counted: completed client-side query executions against OPFS-backed
  local storage.
- Not counted: dispatches that fail before engine completion
  (OOM, worker crash, RPC error), background cache maintenance, sync
  bytes, bundle downloads.
- Cross-surface rule: a query that runs on a cloud path (if the
  product ever adds one) is not SOC — SOC counts displaced work only.

## License interface (resolved: LICENSE-ENTERPRISE-DRAFT.md §5 defines SOC as a damages framework; coefficients stay out of the license text)

The enterprise license draft (BP5) is expected to attach the
gross-revenue-tier condition precedent to a product surface whose
definition must not depend on counting SOC directly. The design rule:
the license conditions on gross-revenue tiers; SOC remains an internal
capacity/displacement measure used for product capacity planning and
tier-set calibration, never a chargeback line.

Why key off gross revenue rather than SOC:

- SOC counts are client-side observed and therefore under vendor
  control only partially (clients can be offline, logs truncated by
  quota, privacy settings suppress the stream). A revenue threshold is
  externally verifiable; a client-side counter is not, and a license
  condition must be verifiable by both parties without instrumenting
  the customer's users.
- Revenue tiers avoid billing on the exploratory reformulation loop
  that BP4 identifies as the core value surface — charging per SOC
  unit would recreate the per-query bill-shock economics the edge
  path exists to eliminate.
- Separation of concerns: valuation drifts (coefficients change as
  the BP4 harness replaces assumptions with measurements) then
  re-calibrates tier boundaries only, without renegotiating the
  condition-precedent language.

Interface contract, provisional until BP5 lands: the license may
reference "Synthetic Operational Capacity" only as a defined term
pointing at this document; it may not embed SOC coefficients, or any
dollar conversion. If BP5 needs a numeric surface, it takes tier
boundaries (revenue numbers), not SOC numbers.

## Valuation coefficients (all ASSUMPTION-UNVERIFIED)

The valuation of one SOC unit is: displaced cloud cost per query minus
vendor delivery cost per query. Both terms require BP4 harness
receipts:

- Displaced cloud cost per query: requires Arm A of the BP4 harness
  (metered cloud warehouse, provider's own bytes-billed readout) on
  the fixed reference dataset, including the 20-reformulation loop.
- Vendor delivery cost per query: requires Arm B (bytes downloaded,
  OPFS at-rest, wall-clock) from the same receipt.
- Loop-amplification factor: the ratio between single-query displaced
  cost and 20-reformulation-loop displaced cost; measured, not
  asserted.

Until those receipts exist, every coefficient (per-query displaced
cost, per-query delivery cost, loop amplification, tier boundary
calibration inputs) is ASSUMPTION-UNVERIFIED and must not be quoted
outside this file, the BP4 lane, or the future BP5 draft. The model is
the algebra; the numbers arrive only from the harness.

## Auditability

SOC reports must be reproducible from client-side event logs. The
binding requirement: for any reported SOC figure R over window W, a
second party given only the exported event-log slice for W must be able
to re-derive R by a deterministic derivation, and get the same number.

Mechanism, tied to BP3's OCEL telemetry: the engine's usage stream is
an OCEL log (events, objects, object types, event types per the OCEL
2.0 shape used by `docs/design/ocel2-compression.md`). SOC derivation
is a query over that log — filter to completed query-execution events,
group by window, count — expressed as a versioned derivation query
stored alongside the report. A SOC report therefore ships as a pair:
(figure, derivation-query id + log-slice digest), so the report names
its own evidence.

Reproducibility requirements, normative:

1. Event log completeness: every counted event carries the counters
   defined in the Measurement basis section, emitted at engine
   completion.
2. Deterministic derivation: the derivation query is pure SQL over the
   event log; same log slice in, same SOC figure out.
3. Digest binding: the report pins the log-slice digest (content
   hash) it was derived from; re-deriving against a different slice is
   a mismatch, not a pass.
4. Counted/not-counted classification is part of the derivation
   query, not a reporting-time judgment call — the same classification
   rule applies at capture and report time.
5. Tamper surface: the exported slice is append-only client-side
   (OPFS), and the derivation query version is pinned in the report.
   Gaps (quota truncation, offline periods) are reported as explicit
   gap markers in the slice, never silently dropped — a report derived
   from a slice with gap markers carries them forward into the report.

Non-reproducible-by-construction surfaces are out of scope and must
not be presented as SOC: sampling, estimation, modeled usage, or
server-side inference.

## Open items

- BP5 license draft does not exist on disk yet; when it lands, the
  interface section must be re-checked against the actual draft and
  the provisional marker removed or the mismatch recorded.
- SOC event schema versioning: the event attributes listed are the
  minimum; a schema-version attribute must be added at implementation
  time and pinned in each report.
- BP4 harness has not produced its first receipt; all coefficients
  remain ASSUMPTION-UNVERIFIED until then.

## See Also

`docs/design/economic-model.md` (BP4 — measurement harness, falsifier,
unit economics placeholders) · `docs/design/wasm-opfs-architecture.md`
(BP1 — in-browser WASM/OPFS execution surface) ·
`docs/design/ocel2-compression.md` (BP3 — OCEL 2.0 log shape and
rehydration) · BP5 enterprise license draft (not yet on disk;
forward dependency)
