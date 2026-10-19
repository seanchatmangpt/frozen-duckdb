# Gage R&R for Courts — Protocol, Pilot Data, Capability Verdicts

Lane LSS5 (Measurement-System avatar). Pilot date: 2026-10-19, this machine
(Darwin 25.2.0), lane target dir `target-lane5`, debug build profile, no
`target-cpu=native`. Status: PILOTED — real runs, real numbers; the DoD's
designed-not-run flag on Gage R&R is closed by this receipt.

## 1. Protocol

A **court** is one `cargo test` binary whose verdict is pass/fail plus, for
numeric courts, a measured quantity (wall_s, p95_ms) written to a receipt in
`/tmp/value_audit/`.

**Intra-court repeatability**: run the court 2x back-to-back on a settled
machine; verdicts must agree. A court whose verdict flips between consecutive
same-machine runs is not a measurement system — it is a lottery, and its
numeric target is unmeasurable until fixed.

**Measurement precision**: from the two runs compute mean, sample sd (n-1),
and CV% = 100·sd/mean. Capability bands follow AIAG MSA %GRR convention:
CV < 10% = capable; 10–30% = conditionally capable (averaging protocol
required); > 30% = not capable.

**CI rule**: deterministic verdict-only courts run 1x in CI, fail-closed on
exit code. Noise-sensitive numeric courts use an N-run **median** (N=3)
compared to budget; median over mean because the p95 estimator is
heavy-tailed and median-of-3 kills single-run outliers without inflating CI
wall time.

## 2. Pilot execution receipt (real commands, real output)

Build-freeze disclosure (same-checkout fan-out): during the pilot,
`crates/frozen-duckdb-builder/src/lib.rs` was mid-edit by a concurrent lane
(unclosed-delimiter compile error). I polled under the compile-freeze SLA; it
compiled at the first poll (~90 s). This lane edited no code; files written
are this document and lane build dir `target-lane5` (coordinator deletes at
integration).

### doctor_storage_tests — 2 runs

```
$ CARGO_TARGET_DIR=target-lane5 cargo test -p frozen-duckdb --test doctor_storage_tests
run 1: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s   (exit 0)
run 2: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s   (exit 0)
```

### doctor_types_tests — 2 runs

```
$ CARGO_TARGET_DIR=target-lane5 cargo test -p frozen-duckdb --test doctor_types_tests
run 1: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s   (exit 0)
run 2: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s   (exit 0)
```

### oom_graceful_abort — 2 runs

```
$ CARGO_TARGET_DIR=target-lane5 cargo test -p frozen-duckdb --test oom_graceful_abort
run 1: test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.17s   (exit 0)
run 2: test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.17s   (exit 0)
```

### REV-001 wall_s — 2 runs (receipt /tmp/value_audit/rev001.json)

```
$ CARGO_TARGET_DIR=target-lane5 cargo test -p frozen-duckdb --test test_rev_compliance rev001
run 1: test result: ok. 1 passed; 0 failed; ... finished in 7.75s
       wall_s = 7.688   clause_lt_2_5s = UNMET-BY-MEASURED
run 2: test result: ok. 1 passed; 0 failed; ... finished in 7.76s
       wall_s = 7.694   clause_lt_2_5s = UNMET-BY-MEASURED
```

### GATE-V-01 p95 — 2 runs (receipt /tmp/value_audit/gate_v01_receipt.json)

```
$ CARGO_TARGET_DIR=target-lane5 cargo test -p frozen-duckdb --test gate_v01_latency -- --nocapture
run 1: p95_nearest_rank_ms = 11.1680 (index 949); test gate_v01_p95_under_15ms ... ok
run 2: p95_nearest_rank_ms = 11.2835 (index 949); test gate_v01_p95_under_15ms ... ok
```

### Precision computation (verbatim python3 output)

```
REV-001 wall_s pilot (7.688, 7.694)                     mean=7.6910 sd=0.0042 CV=0.06%
GATE-V-01 p95 pilot (11.168, 11.2835)                   mean=11.2257 sd=0.0817 CV=0.73%
GATE-V-01 recorded pair A (12.25,12.36)                 mean=12.3050 sd=0.0778 CV=0.63%
GATE-V-01 recorded pair B (10.51,10.69)                 mean=10.6000 sd=0.1273 CV=1.20%
GATE-V-01 pooled recorded A+B (n=4)                     mean=11.4525 sd=0.9881 CV=8.63%
GATE-V-01 pilot+pairB (n=4)                             mean=10.9129 sd=0.3717 CV=3.41%
```

## 3. Provenance note on dispatcher-supplied numbers

Pairs A (12.25/12.36) and B (10.51/10.69) came from the tasking message. Pair
A's values appear NOWHERE in the repo (repo-wide grep over *.md/*.json,
target/ excluded); only 10.51 has on-disk corroboration
(`receipts/AD7-CLAIMS-AUDIT.md`, `receipts/DOC-RESTRUCTURE-RECEIPT.md`;
`gate_v01_receipt.json` carries p95_ms 10.5088). The recorded-run CV figures
that cite pair A are therefore marked dispatcher-supplied. Because the pooled
8.63% figure rests partly on unprovenanced data, I re-ran GATE-V-01 fresh
twice; the on-disk-provenance cross-session figure is pilot+pairB pooled CV
3.41%, and the fresh-pilot-only CV is 0.73%.

## 4. Capability verdicts

| court | type | repeatability | CV% | verdict |
|---|---|---|---|---|
| doctor_storage_tests | deterministic | verdict-identical 2x | n/a | **CAPABLE** |
| doctor_types_tests | deterministic | verdict-identical 2x | n/a | **CAPABLE** |
| oom_graceful_abort | deterministic | verdict-identical 2x | n/a | **CAPABLE** |
| REV-001 wall_s | numeric | 2x | 0.06 (pilot) | **CAPABLE** as an instrument |
| GATE-V-01 p95 | numeric | 2x | 0.73 pilot; 3.41 on-disk cross-session; 8.63 pooled w/ dispatcher-supplied pair | **CAPABLE** within-session; **CONDITIONAL** cross-session (nearest recorded figure to the 10% line is 8.63%, and it leans on unprovenanced data) |

Note on REV-001: the instrument is capable (CV 0.06%) but measures a clause
that is unmet — wall_s 7.69 s vs the 2.5 s clause, repeatably. A repeatable
failure is a real failure: the measurement system works; the system under
test does not yet meet the clause (`COURT-INDEX.md` carries rev001.json as
UNMET-BY-MEASURED, unchanged by this pilot).

## 5. Rule for CI

1. Deterministic courts (all 3 piloted): 1 run in CI, fail-closed on exit code.
2. REV-001: instrument is capable; clause UNMET — treat as budget-enforcing
   court with median-of-N once the clause is reachable; today it fails loudly
   either way, so 1 run suffices.
3. GATE-V-01: N-run median, N=3, vs the 15 ms budget. Basis: cross-session
   spread (pooled CV up to 8.63%, near the 10% line) plus the heavy-tailed
   p95 estimator. Receipt must carry all N raw p95 values, not just the
   median. Budget holds: worst on-disk-adjacent observed p95 is 12.36 ms
   (dispatcher-supplied), worst on-disk value 11.2835 ms, both < 15 ms.
4. Any court piloting into the 10–30% band: median-of-N, N=3 minimum, all raw
   values in the receipt. > 30%: NOT CAPABLE — blocked from budget-enforcement
   duty until re-piloted.
5. Provenance law: measurement-system receipts cite on-disk provenance only;
   dispatcher-supplied numbers are admissible only when explicitly marked as
   such (as done in §3).

## 6. Replay

```
CARGO_TARGET_DIR=target-lane5 cargo test -p frozen-duckdb --test doctor_storage_tests
CARGO_TARGET_DIR=target-lane5 cargo test -p frozen-duckdb --test doctor_types_tests
CARGO_TARGET_DIR=target-lane5 cargo test -p frozen-duckdb --test oom_graceful_abort
CARGO_TARGET_DIR=target-lane5 cargo test -p frozen-duckdb --test test_rev_compliance rev001
CARGO_TARGET_DIR=target-lane5 cargo test -p frozen-duckdb --test gate_v01_latency -- --nocapture
```

Lane LSS5, 2026-10-19.