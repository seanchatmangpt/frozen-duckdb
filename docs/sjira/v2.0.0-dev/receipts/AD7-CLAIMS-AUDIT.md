# AD7 Claims Audit — v2.0.0-dev (2026-10-19)

Falsification round over the 10 most load-bearing claims. Verdicts:
7 HOLDS / 2 IMPRECISE / 1 FALSE; the 3 corrections were landed by the
coordinator (MB-vs-MiB unit fix in README + RELEASE-DRAFT;
fleet004.json citation retarget in docs/reference/hardware-classes.md;
OCEL corpus qualifier added to README).

| # | Claim | Verdict | Note |
|---|---|---|---|
| 1 | 65.9/52.4 dylib size | IMPRECISE (unit) | MiB→MB corrected; stripped figure is prebuilt/README measurement |
| 2 | 12.16M rows/s | HOLDS | DISPATCH-CITED; corpus unqualified, labeled |
| 3 | p95 10.51ms | HOLDS | gate_v01_receipt.json 10.5088; not the release-profile number |
| 4 | OCEL 94.93/97.01% | IMPRECISE (corpus) | corpus qualifier added to README |
| 5 | 0 B egress | HOLDS | by-construction, labeled |
| 6 | Class-D 99.76% | HOLDS | recompute 536,576/221,033,690 = 0.9976 ✓ |
| 7 | Class-D receipt citation | FALSE (citation) | was chaos artifact (v1.5.5); retargeted to fleet004.json |
| 8 | $264,544.62 total | HOLDS | live-render correction carried verbatim |
| 9 | $12,090/yr | HOLDS | unrounded carry, internally consistent |
| 10 | chaos legs claim | HOLDS | matches fleet004_chaos.json legs a-d |
