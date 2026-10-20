# ADV5 Receipt: GATE-V-01 median-of-3 (LSS5 §5 rule 3)

Date: 2026-10-19. Lane: ADV5. Subject head: `adbd26014ad62d4f4a2da54a75b9d328dba053fb`,
tree: `377ad914639001fbf53cf59bc9ae9bd3e8fb6ddd` (printed by the test's own
subject binding).

## Change

`crates/frozen-duckdb/tests/gate_v01_latency.rs`: the single-round measurement
moved into `measure_round(round)` (fresh in-memory connection + fresh seeded
100k-row table per round, 50 warmup + 1000 measured iterations, nearest-rank
p95, unchanged SQL constants). The test now runs 3 rounds, takes the MEDIAN of
the three p95s, asserts `median_p95 < 15.0 ms`, and prints all three raw p95s
plus the median (receipt law §5: all N raw values, not just the median).
The V4 JSON receipt (`/tmp/value_audit/gate_v01_receipt.json`) now carries
`mode: median-of-3` and the full `p95_ms_rounds` array, written only after the
assert passes.

## Command (real)

```
CARGO_TARGET_DIR=target-adv5 cargo test --release -p frozen-duckdb --test gate_v01_latency -- --nocapture
```

Exit 0. `test result: ok. 1 passed; 0 failed` — finished in 35.55 s.

## Measured (Apple M3 Max, 48 GB, 16 cores, release profile)

| round | mean_ms | median_ms | p95_nearest_rank_ms |
|---|---|---|---|
| 1 | 10.7708 | 10.7319 | **11.2730** |
| 2 | 11.6839 | 10.9361 | **15.0504** |
| 3 | 11.3054 | 11.0058 | **13.3609** |

**median p95 = 13.3609 ms < 15.0 ms budget → PASS**

Spread note: the three p95s span 11.27–15.05 ms (max/min = 1.335) — round 2's
raw p95 exceeded the budget on a single-run gate. This is precisely the
contention flake (DOD-2's F1 73.9 ms debug / CG3's 21.1 ms class) that §5
rule 3 targets: the median absorbs a single contended round without weakening
the budget, because a persistently contended run fails all three medians.

## Falsifier check (non-vacuity)

The assert is load-bearing on real output: with this same run's round-2 p95 of
15.05 ms, a single-run gate (pre-change form, budget on one p95) would have
panicked. The median rule is what makes this run pass; a mutation forcing all
three rounds above 15 ms fails the assert by construction.

## Verdict

GATE-V-01 ALIVE under the LSS5 §5 CI rule (median-of-N, N=3, all raw p95s in
receipt). Budget not weakened: 15.0 ms unchanged.

Lane build dir `target-adv5/` left for coordinator deletion at integration
(same-checkout-fanout cleanup law).
