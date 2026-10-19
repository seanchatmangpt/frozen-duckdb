# VERIFIER-RECEIPTS-LOG — v2.0.0-dev wave 2 (skeleton)

One line per verifier/auditor lane: one-line finding summary. Full receipts with
real command output are dropped in `receipts/` by the coordinator. This log is a
skeleton index — a row without a backing receipt file is SUSPECT until the
coordinator lands the full receipt.

| lane | finding summary |
|---|---|
| V2 | DOCS_RS PASS 89/89 |
| V2a | 22/22 |
| A2 | 13/13 LAYOUT-MATCH |
| A4 | 14/14 |
| A5 | clean |
| A7 | all-pass |
| A8 | pass-with-caveats |
| A9 | shape-match |
| A11 | no-drift |
| A12 | 1 misattribution |
| A13 | consistent |
| A14 | partial-fixed |
| A1 | ~100 doc hits → routed to D1 (docs repair) |
| A3 | 2 residues → fixed/recorded |

## Notes

- A9 finding 1 drove the `_LANES.md` W7 ownership scoping (path-scoped to
  `types/`, `row.rs`, `vtab/arrow.rs`).
- A18 finding resolved by the coordinator's `value_ref.rs` `expect()` fix (see
  UPGRADE-WO.md Wave-2 section).
- A1's ~100 doc hits fed lane D1.
