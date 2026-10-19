# economic-model.md

## Status

DESIGN — not marketing, not measured. Every dollar figure below is
ASSUMPTION-UNVERIFIED. The externalization gate is the workload-cost
comparison harness defined in the Falsifier section; until it runs on a
reference dataset, no number in this document may be quoted outside this
file or the BP4 lane.

Owner: blueprint lane BP4. Architecture dependency: BP1 (in-browser
WASM/OPFS execution surface).

## Thesis (subsidized silicon)

Customer analytics workloads — cohort retention, funnels, local ML —
execute on hardware the customer already owns, not on metered cloud
compute. The engine ships to the edge (in-browser WASM with OPFS
persistence per the BP1 architecture); data lands once; scans and
joins run against local storage on the customer's CPU.

The economic claim: end-user silicon is already paid for and sits idle
most of the day. Running analytics on it converts a per-query variable
cost (cloud warehouse scan fees + compute seconds) into a fixed,
already-amortized cost (the device). The vendor's marginal cost per
additional query approaches the cost of serving the WASM bundle and
the data — bytes, not scans.

This is "subsidized" in the specific sense that the customer's own
hardware subsidizes the vendor's compute bill. It is not free: someone
pays for download bandwidth, storage sync, and the engineering cost of
making in-browser execution fast enough.

## Workload surface in scope

- Cohort retention: group-by over event rows, date-bucketed
  aggregation. Scan-heavy, compute-light.
- Funnels: ordered-sequence matching over per-user event streams.
  Scan-heavy, moderate compute.
- Local ML (feature extraction, small-model training, scoring):
  compute-heavy, repeated iterations over local data.

Common property: all three are iterative and exploratory. A single
question fans out into tens of reformulations. Per-query cloud pricing
charges the reformulation loop N times; local execution charges it
approximately zero marginal dollars (electricity and wall-clock only).

## Unit economics

ILLUSTRATIVE ONLY. Every number in this table is
ASSUMPTION-UNVERIFIED — a placeholder for the harness to replace, not
a measured result. Dollar values are shown as symbols, not figures,
because inventing dollar amounts would be fabricated precision.

| Line | Cloud-warehouse path | Edge (WASM/OPFS) path |
|---|---|---|
| Line | Cloud-warehouse path | Edge (WASM/OPFS) path |
|---|---|---|
| Data ingest | A-U: per-GB load / storage-month fee | One-time download: vendor egress bytes |
| Per query | A-U: scan fee ~ bytes; compute seconds | ~$0 marginal $; wall-clock + power |
| Loop amplification | N reformulations x fee (linear in N) | Sublinear: warm OPFS cache |
| Storage at rest | A-U: $/TB-month | Customer disk (OPFS quota) |
| Vendor cost per active user | Scales with query volume | A-U: CDN egress + sync bytes |
| Failure mode | Cost grows with engagement | Cost grows with data size / sync freq |

"A-U" = ASSUMPTION-UNVERIFIED. Every A-U cell is a placeholder the
harness must replace; no dollar figure may be derived from this table
until then.

Margin mechanics, stated precisely: the cloud path has a cost curve
that scales with the customer's most valuable behavior (iterative
question-asking), which is why scan-fee products either throttle
exploration or face bill shock. The edge path decouples vendor cost
from query count; vendor cost then scales with (a) bytes shipped per
user and (b) active-user count, both of which are forecastable. Margin
is the spread between subscription price and byte-delivery cost, not a
spread over metered compute.

What this model does NOT claim:

- It does not claim in-browser execution is faster than a tuned
  cloud warehouse on large data. On big scans it is slower. The claim
  is about cost per unit of insight in the exploratory loop, at data
  sizes that fit OPFS quota.
- It does not claim zero vendor cost. Sync, CDN, support, and
  engineering remain.
- It does not claim OPFS quota sufficiency for arbitrary datasets.
  The in-scope size envelope must be established by measurement, not
  asserted.

## Falsifier (gate before externalization)

A workload-cost comparison harness must measure real costs on a fixed
reference dataset before any economic claim from this document is
stated externally. Shape:

1. Reference dataset: a fixed, versioned event-level dataset (pinned
   SHA; row count and byte size recorded in the receipt).
2. Reference workloads: one cohort-retention query, one funnel query,
   one local-ML feature pass — each written once, run against both
   surfaces.
3. Arm A (cloud): the same queries against a metered cloud
   warehouse; record the provider's own cost meter / bytes-billed
   readout for each run and for a 20-reformulation loop.
4. Arm B (edge): the same queries under the BP1 in-browser build;
   record wall-clock, bytes downloaded, and OPFS bytes at rest.
5. Output: a machine-readable comparison receipt (both arms' raw
   outputs embedded), from which the unit-economics table is
   re-derived. Any number in this table that the receipt does not
   back remains ASSUMPTION-UNVERIFIED and must not be quoted.

Kill condition: if measured Arm B wall-clock makes the exploratory
loop unusable (the practitioner abandons the reformulation), or OPFS
quota rejects the reference dataset, the subsidized-silicon thesis is
refuted for that data-size envelope and this doc is amended to state
the measured boundary — not softened.

## See Also

BP1 architecture lane (in-browser WASM/OPFS execution surface) ·
`docs/sjira/` active window `_LANES.md` · `gates/dry-run-publish/`
(receipt discipline precedent)

## Honest Competitive Boundary (2026-10-19 assessment)

Verdict: not a Snowflake killer — a workload-class extractor.

Wins (grounded): zero per-query meter for on-device exploratory
analytics (hyperscaler cost ∝ queries × bytes scanned; this is
amortized per device); data sovereignty (raw records never leave the
device; sync-digest design transmits only signed digests — DESIGN);
latency/offline (p95 <15ms gate target, GATE-V-03 OOM confinement
witnessed); distribution (builder freeze = zero C++ toolchain for
consumers — ARGUMENT: a warehouse is argued to be unable to ship
inside an app binary).

Where hyperscalers win structurally: fleet/cross-device aggregation
(the collection layer is exactly where they bill — the signed-digest
sync layer is a design doc, not a system); concurrency and scale
(single-writer engine, OPFS quota caps, deliberate 256MB ceilings;
no petabyte joins/RBAC/lineage/SOC2 surface); ecosystem (no dbt/BI
connector/ML-platform interchange); operational risk transfer.

Unreconciled risk: the flagship economics ($0 marginal cost, 95%+
egress collapse, 99% margins) remain PROPOSED — the measurement
harness (two-arm workload-cost comparison) has not run. The business
case is staged behind the same fail-closed courts as the engine.

See Also: soc-valuation-model.md - sync-digest-design.md
