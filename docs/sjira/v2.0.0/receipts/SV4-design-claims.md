# SV4 — docs/design + KNOWN_DIVERGENCES status-accuracy audit (2026-10-19)

Sealed by coordinator (SV4 lane had no write tools). Full audit text below —
verdicts: 10 heavy claims → 5 RECEIPT-VERIFIED, 2 DISPATCH-CITED (labeled),
1 DISPATCH-CITED (kaizen arithmetic), 1 OVERCLAIMED ($264,545.73 → corrected
on disk to $264,544.62), KD rows 14/15 hold.

[F-1] KD-14 CLOSED standing was stale at audit time (builder suite mid-edit,
43 tests, 2 failures incl. control-row FAIL) — RESOLVED post-audit: current
builder battery 46/46 green (post slice-floor + magic + marker work).
[F-2] KD-15 row orphaned outside the KD table by the line-26 See Also — noted.
[F-3] Heavy claims live in docs/sjira/v2.0.0/ + docs/explanation/, not docs/design/.
[F-4] $264,545.73 (dispatch figure) corrected to $264,544.62 on-disk figure.

Full per-claim table: see the SV4 lane output (10-row table, all receipt paths).
