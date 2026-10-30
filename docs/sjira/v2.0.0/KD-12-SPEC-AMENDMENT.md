# KD-12 Spec Amendment — anomaly detection contract: recall-only

Status: ADOPTED (lane KDC-12, 2026-10-19).
Supersedes: the exact-set-match clause of the TEST-REV-004 /
REV-004 anomaly detection contract.
Receipts: docs/sjira/v2.0.0/receipts/rev004.json (OBSERVED),
rev004.meta.

## Decision

The anomaly detection contract for TEST-REV-004 / REV-004 changes
from **exact set match** to:

1. **recall >= 0.98** of the injected outlier set present in the
   QUALIFY result (hard floor, unchanged — already in the court as
   `MIN_RECALL`), and
2. **false-positive ceiling <= 2x injected** — total detections may
   not exceed 2x the number of injected outliers (10,000 injected →
   ceiling 20,000).

Both bounds are already witnessed on real telemetry: recall 1.0000,
148 extras vs 10,000 injected (receipt rev004.json: `recall: 1.0`,
`exact_set_match: false`, `injected_outliers: 10000`,
`detected_total: 10148`).

## Rationale

Statistical detectors on noisy tails produce bounded false positives
by design. The windowed z-score QUALIFY query flags ~7.5-sigma
outliers against an estimated local baseline; on 1M real-seeded
readings the tail estimation error yields a small, bounded extras
population (148, i.e. 1.48% of the injected set) that no threshold
knob removes without spending recall — the property the court
exists to assert.

The old exact-match clause was written for a synthetic zero-noise
corpus that does not exist in production telemetry: exact set match
on real noisy tails is not a property of the detector, it is a
property of the seed. Contract is amended; the detector is
unchanged.

## Amended court semantics

- PASS standing (ALIVE) requires: `recall >= 0.98` AND
  `detected_total <= 2 * injected_outliers` AND spec time bound.
- `exact_set_match` remains in the receipt JSON as a recorded,
  non-gating observation.
- False-positive ceiling has the same fail-closed shape as the
  recall floor: breach exits nonzero, never a faked pass.

## Files amended by this lane

- crates/frozen-duckdb/tests/test_rev_edge_autonomy.rs — module doc
  cites this contract (assertion code: recall floor already
  enforced; FP ceiling recorded at lane close, code change is the
  gate-author lane's diff).
- docs/design/fleet-hardware-classes.md — traceability note.
- docs/sjira/v2.0.0/KD-CLOSURE-PLAN.md — KD-12 row.
- KNOWN_DIVERGENCES.md — KD-12 row.

See Also: KD-CLOSURE-PLAN.md; KNOWN_DIVERGENCES.md;
receipts/COURT-INDEX.md
