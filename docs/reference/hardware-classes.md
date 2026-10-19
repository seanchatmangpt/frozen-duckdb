# hardware-classes

Information-oriented reference for the doctor stage-12 fleet hardware
classes A-E. Spec and physical-reality audit:
`docs/design/fleet-hardware-classes.md`; receipts:
`docs/sjira/v2.0.0-dev/receipts/COURT-INDEX.md`. All measured numbers
are from the 2026-10-19 campaign runs.

## Matrix

| class | hardware target | real silicon here? | court file |
|---|---|---|---|
| A | workstation (M3; wasm leg FR-01/02-blocked) | YES native | test_fleet_class_a_workstation.rs |
| B | industrial low-RAM Linux cgroup runner | NO — proxy only | cgroup_low_ram_runner.sh |
| C | retail edge | YES | test_fleet_class_c_retail.rs |
| D | marine container telemetry (offline) | YES (native leg) | fleet_d_chaos.sh |
| E | telco | duckdb leg YES; BEAM leg NO | test_fleet_class_e_telco.rs |

## Class A — workstation

1M-row enterprise telemetry aggregation (operator's query verbatim:
WHERE ts >= DATE, GROUP BY department, count/sum/quantile 0.99);
median-of-30 < 20 ms; RSS delta <= 512 MB. wasm/OPFS leg and CDP
zero-network assertion are BLOCKED FR-01/FR-02, echoed loud.
Measured: 2.588 ms (median). Receipt: `/tmp/value_audit/fleet001.json`
(court index: latency MET, RSS WITHIN-BUDGET).

## Class B — industrial telemetry (low-RAM runner)

Real gate: Linux cgroup `memory.max=128MB` + iptables egress sever +
4h soak + `memory.events` OOM ledger; exits 2 loud on darwin (KD-09).
Proxy leg: engine `memory_limit=128MB`, 20 windows x 50k appender
frames, post-warmup window RSS <= 110 MB, per-window growth
decelerating. Measured (proxy): 80.5 MiB window RSS — PASS, and a
PARTIAL_ALIVE ceiling by construction, never a substitute for the
kernel-enforced cap. Receipt: `/tmp/value_audit/fleet002_proxy.json`.

## Class C — retail edge

100k txns / 10 registers / 10 hourly epochs; falsifiers: imbalance
query (HAVING store_imbalance <> 0.00) returns zero rows; every epoch
digest present and successful; summary payload <= 4 KB. Measured:
54.77 ms. Receipt: `/tmp/value_audit/fleet003.json`.

## Class D — marine container telemetry (offline)

1M events; exactly two legal terminal states (last-committed or
in-txn sentinel, never a torn third); checksum == closed-form
sum(3n+7); compression ratio >= 0.95; satellite delta-sync vector
<= 500 KB (per-segment MD5 x 10k segments). Non-vacuity: kill -9 ->
kill -0 SHIM-COURT must flip the leg to VACUOUS-GATE. Measured: 99.76%
compression; chaos 2 kill legs + no-op-kill flip DETECTED. Receipt:
`target/court/fleet-d-chaos-receipt.json`.

## Class E — telco

50k-row appender batch flush, median < 45 ms/batch over 40 batches;
session-to-queryable < 1200 us in-process (the 1200 us spec is the
ingress-to-queryable bound; BEAM actor leg is BLOCKED-OTHER-REPO,
smalt, KD-10). Measured: 11.6 ms/batch. Receipt:
`/tmp/value_audit/fleet005.json`.

## Measured summary

| class | measured | bound | receipt |
|---|---|---|---|
| A | 2.588 ms | median-of-30 < 20 ms | `/tmp/value_audit/fleet001.json` |
| B | 80.5 MiB | post-warmup RSS <= 110 MB (proxy) | `/tmp/value_audit/fleet002_proxy.json` |
| C | 54.77 ms | falsifier set (zero-imbalance, digests, <=4KB) | `/tmp/value_audit/fleet003.json` |
| D | 99.76% | compression ratio >= 0.95 | `target/court/fleet-d-chaos-receipt.json` |
| E | 11.6 ms | < 45 ms/batch median | `/tmp/value_audit/fleet005.json` |

## Stage-12 exit contract

`scripts/doctor/12_fleet_hardware.sh`: exit 0 = every executable leg
PASS and every blocked leg echoed loud; 1 = FAIL; 3 = NOT-IMPLEMENTED
(leg absent or filter matched 0 tests); 2 reserved for skip class
(darwin cgroup rc=2 propagates as NOT-IMPLEMENTED-LOUD). Wired into
`scripts/doctor.sh` per the campaign receipt.

## See Also

`docs/design/fleet-hardware-classes.md` ·
`docs/sjira/v2.0.0-dev/receipts/COURT-INDEX.md` ·
`benchmarking.md`
