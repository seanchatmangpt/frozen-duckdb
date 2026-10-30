# SV3 — warehouse-sim claims verification (2026-10-19/20)

Subject: `/Users/sac/frozen-duckdb/warehouse-sim` @ master (adbd260 tree, working
tree with in-flight lanes). Verdict per claim, real output embedded. Lane owned
file only: this receipt.

## 1. Test files: all 8 exist and execute

All eight claimed files exist under `warehouse-sim/tests/`:
`routes_contract.test.ts`, `clients.test.ts`, `journeys.test.ts`,
`chaos.test.ts`, `pagination.test.ts`, `admin.test.ts`,
`adversarial.test.ts`, `value_ledger.test.ts` (plus `helpers/` and an extra
`admin_ledger_bridge.test.ts` not in the claim list).

Command (per-file parallel batch; single shared batch also run first — see
flakiness note):

```
cd /Users/sac/frozen-duckdb/warehouse-sim && node --test tests/<file>.test.ts  (x8, parallel)
```

Real counts (node v26.10.0, per-file runner summaries `ℹ tests/pass/fail`):

| file | claimed | real tests | pass | fail | exit |
|---|---|---|---|---|---|
| routes_contract | 10/10 | 10 | 10 | 0 | 0 |
| clients | 6/6 | 6 | 5 | 1 | 1 |
| journeys | 4/4 | 4 | 4 | 0 | 0 |
| chaos | 5/5 | 5 | 5 | 0 | 0 |
| pagination | 2/2 | 2 | 2 | 0 | 0 |
| admin | 5/5 | 5 | 5 | 0 | 0 |
| adversarial | 7/7 | 7 | 4 | 3 | 1 |
| value_ledger | 10/10 | 10 | 10 | 0 | 0 |

Totals: 49 tests, 45 pass, 4 fail.

### Failures (real output)

1. `clients.test.ts` — "error propagation: client surfaces the sim's
   route-failure envelope": asserted `/expected 202, got 500/`, got
   `'snowflake submit expected 202, got 400'`.

   ```
   AssertionError [ERR_ASSERTION]: The input did not match the regular
   expression /expected 202, got 500/. Input:
   'snowflake submit expected 202, got 400'
       at tests/clients.test.ts:122:14
   ```

   The client's error message for a malformed submit is "got 400" but the
   test pins the exact wording "got 500" — the route now correctly returns
   typed 400, the test string is stale.

2. `adversarial.test.ts` A3 — "no 5xx and the ledger stays internally
   consistent (queryCount <= submitted)": polls during a reset race get an
   HTML error page, JSON.parse throws:

   ```
   SyntaxError [Error]: Unexpected token '<', "<!DOCTYPE "... is not valid JSON
   ```

3. `adversarial.test.ts` A5 — "server stays healthy and the ledger stays
   bounded-consistent (500 submits, zero polls)": assertion
   `Expected values to be strictly equal` fails at 13.6ms (fails fast; no
   500-submits loop observed in the parallel run).

4. `adversarial.test.ts` R — "malformed pageToken / page -> 400
   invalid_request (was 500)": pageToken 400 regression closure NOT landed —
   malformed pagination params still fail through a non-typed path.

### Flakiness note (honest variance)

A first single-batch run (`node --test tests/*.test.ts` all eight in one
node process) produced a DIFFERENT failure set: clients Snowflake submit
timed out at 90s (`POST /api/v2/statements` 90208ms), routes_contract's
"determinism: same request twice" diverged (state trajectory
`QUEUED→SUCCEEDED` vs `QUEUED→RUNNING→SUCCEEDED`), plus A5/R timeout-family
failures at 300s. The per-file parallel batch above is the receipt of record
for counts; the single-batch variance is recorded because every failure set
includes poll-timing sensitivity. Test claims should be cited per-file with
the flakiness caveat.

### Claim deltas

- `admin 5/5`, `chaos 5/5`, `journeys 4/4`, `pagination 2/2`,
  `routes_contract 10/10`, `value_ledger 10/10`, `clients 6/6` — clients
  6/6 is FALSE (5/6, stale 500-string assertion).
- `adversarial 7/7` — FALSE (4/7).
- CAMPAIGN-RECEIPT.md:345-346 cites "admin 5/5; clients 6/6; chaos 5/5;
  journeys 4/4" — the clients 6/6 citation is stale.

## 2. Courts

```
-rwxr-xr-x  courts/determinism.sh   (7198 bytes)
-rwxr-xr-x  courts/receipt_chain.sh (5636 bytes)
-rwxr-xr-x  economics/run_scorecard.sh (4636 bytes)
```

All three exist and are executable. Note: `run_scorecard.sh` lives in
`economics/`, not `courts/` — any doc saying "courts/run_scorecard.sh" is
path-wrong.

## 3. ggen sync run idempotency: CONFIRMED

```
ggen sync run (ggen 26.10.11) — two consecutive passes
diff pass1.sha pass2.sha  -> IDEMPOTENT
diff pre.sha  pass1.sha   -> PASS1_NO_CHANGE
```

Second pass `"written": []` (both passes actually — the tree was already
in sync before the first pass). Hashes over `app/ lib/ schema/` `.ts`+`.ttl`
identical across passes.

## 4. Rendered app/api routes

All claimed route surfaces exist on disk as rendered output:

- submit: `app/api/v2/statements/route.ts` (Snowflake)
- poll: `app/api/v2/statements/[handle]/route.ts`
- Databricks pair: `app/api/2.0/sql/statements/route.ts`,
  `app/api/2.0/sql/statements/[statement_id]/route.ts`
- pagination: page params in the poll/handle route (adversarial R test
  targets it; `pagination.test.ts` 2/2 exercises it) — no separate
  pagination route file, pagination is a query-param surface of the poll
  route
- admin: `app/api/admin/health/route.ts`,
  `app/api/admin/ledger/route.ts`,
  `app/api/admin/ledger/reset/route.ts`

## Standing

- Six of eight sim claims VERIFIED as stated; `clients 6/6` and
  `adversarial 7/7` REFUTED (5/6 and 4/7 with the four failures above).
- Courts claim VERIFIED (paths corrected: run_scorecard is economics/).
- Idempotency claim VERIFIED.
- Routes claim VERIFIED.
- Poll-timing flakiness under single-process batch execution recorded as an
  open caveat.
