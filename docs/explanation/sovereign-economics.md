# Sovereign Economics

Why the architecture is shaped the way it is, in money. This is an
understanding-oriented doc: it explains the *economic* argument behind
executing analytics on the customer's own silicon instead of a metered
cloud warehouse. It is not the spec (that is
`docs/design/economic-model.md`) and not the testbed (that is
`warehouse-sim/`, design in `docs/design/warehouse-sim-testbed.md`).

## The subsidized-silicon argument

Customer analytics workloads — cohort retention, funnels, process
mining, local ML — run today on metered cloud compute: scan fees and
compute seconds, billed per query. The sovereign position inverts this:
the engine ships to hardware the customer already owns and has already
amortized. The customer's device subsidizes the vendor's compute bill.
The vendor's marginal cost per additional query stops being a scan fee
and becomes bytes — the WASM bundle and the data.

The exploratory loop is where the inversion pays. A single analyst
question fans out into tens of reformulations. Metered pricing charges
the loop N times; local execution charges it approximately zero
marginal dollars (wall-clock and electricity only). Cloud-path cost
scales with the customer's most valuable behavior; edge-path cost scales
with bytes shipped and active users, both forecastable.

## The measured physical quantities

These are real, receipted measurements from the v2.0.0-dev campaign
(`docs/sjira/v2.0.0-dev/CAMPAIGN-RECEIPT.md`, final reconciliation
2026-10-19):

- **Throughput: 12.16M rows/sec** (release build, ingest/throughput
  court). Standing: DISPATCH-CITED — the citing artifact is on disk;
  the run output is cited through the lane receipt.
- **p95 10.5088 ms** over 1,000 measured 3-query iterations, real
  engine (GATE-V-01 value ledger, `/tmp/value_audit/val003.json`,
  rendered in the cash-audit output below). WITNESSED by re-running
  the renderer for this doc.
- **Egress: 0 bytes, invoice $0.00** for the sovereign leg across all
  five workload classes — by construction (in-process, nothing leaves
  the machine) and witnessed (`warehouse-sim/economics/.evidence/`,
  seed 20261019, 100k rows; disruption scorecard F2/F3 CLEAR x5).

Physical quantities are measured. Everything dollar-denominated below
is not.

## The counterfactual rate-card method

You cannot A/B test a customer's cloud bill, so the campaign uses a
counterfactual: measure the real physical workload on the sovereign
engine, then price the *same* workload at published list prices of the
displaced service. Method (implemented in
`warehouse-sim/economics/render_value_audit.ts`):

1. Run the real workload on the real engine; the court emits a receipt
   with physical quantities only (events, bytes, segments, ms).
2. Apply an explicit rate card (operator-supplied list prices:
   Datadog $0.10/GB-mo ingestion + $1.70/M-events-mo indexing,
   Celonis $0.004/segment, Amplitude $200/M-events-mo, cadence 252
   business days/yr).
3. Render an annual-equivalence figure per section, with the receipt's
   court verdict printed verbatim — a FAIL verdict is shown against
   its own section, never masked by the dollar math.

Observed render (this doc, `npx tsx economics/render_value_audit.ts
--dir /tmp/value_audit`, 2026-10-19) — excerpt:

```text
SECTION 2 — PROCESS-MINING DISPLACEMENT (OCEL/DFG court, VAL-002)
  court verdict: PASS
  measured: 250,000 conforming OCEL segments mined (DFG cycle-time,
            6.8 ms median, real engine)
  annual equivalence: $252,000.00
SECTION 3 — CLOUD-CREDIT DISPLACEMENT (GATE-V-01 court, VAL-003)
  court verdict: ALIVE: p95 10.5088 ms real, $33.12/day avoided,
                 $12090/yr avoided, 0.00 credits consumed
  annual equivalence: $12,090.00
TOTAL AUDITED (annual equivalence, ASSUMPTION-UNVERIFIED rate cards):
  $264,544.62
```

Basis: `docs/design/economic-model.md` (thesis + unit-economics frame)
and `docs/design/disruption-scorecard.md` (per-class measurements,
**5x CLEAR** on all three disruption assertions F1/F2/F3).

### The cold-dominated-sim caveat

The scorecard's 5x CLEAR latency ratios carry a caveat: the warehouse
sim's amortized p95 is dominated by the cold-start window (cold p50
~5.0-5.3 s equals the amortized p95 on every row), so the amortized leg
amortizes a cold epoch across iterations rather than measuring a warm
warehouse. The egress and invoice ratios (F2/F3) are unaffected — 0
bytes and $0.00 are structural, not timing-dependent. Treat the F1
latency multiples as an upper bound, not a fleet-representative figure.

## The honest boundary

Every dollar figure in this doc is **ASSUMPTION-UNVERIFIED**. The rate
cards are operator-supplied list prices, unaudited; the sim's cost
constants (`creditCostPerHour 4.0`, `dcuCostPerHour 4.5`,
`scanRatePerTB 65.0`) are counterfactual parameters, not tariffs.

Specifically:

- The **$264,544.62 rendered total** (dispatch cited $264,545.20; the
  live render this doc observed gives $264,544.62 — the ~$0.58 delta
  is a rounding/citation drift worth noting, not hiding) is an
  **annual-equivalence illustration**, not a customer-bill audit. It
  prices one measured workload at list rates; no customer invoice was
  audited to produce it.
- The **$300-350k/yr net floor** sometimes quoted for this campaign is
  **the operator's claim**, carried here as such. It is not derived in
  any repo artifact; it awaits the two-arm workload-cost comparison
  harness defined in `docs/design/economic-model.md` (Falsifier
  section), which has **not run**. Until it runs on a reference
  dataset with both arms' raw outputs embedded in a receipt, no
  dollar figure in this doc may be quoted externally as fact.

The kill condition stands as written in the economic model: if the
harness shows the edge loop is unusable at the reference data size, or
OPFS quota rejects it, the subsidized-silicon thesis is refuted for
that envelope — amended, not softened.

## See Also

`docs/design/economic-model.md` (thesis + falsifier harness) -
`docs/design/disruption-scorecard.md` (measured 5x CLEAR table) -
`docs/design/soc-valuation-model.md` -
`warehouse-sim/economics/render_value_audit.ts` (renderer) -
`docs/sjira/v2.0.0-dev/CAMPAIGN-RECEIPT.md` (campaign standing)
