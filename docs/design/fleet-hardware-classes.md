# Fleet Hardware Classes — doctor stage 12 (Wave 13, lane T6)

Status: transcription + physical-reality audit, 2026-10-19, lane T6.
Sources transcribed: the lane dispatch (operator harness), the fleet test
file headers under `crates/frozen-duckdb/tests/`, and
`scripts/courts/fleet_d_chaos.sh`, as found on disk at read time.
All spec numbers below are operator claims — ASSUMPTION-UNVERIFIED until
a passing stage-12 run receipts them.

## Class A — workstation

- Hardware target: Apple M3 native (legitimate Class-A silicon proxy per
  the T3 header; wasm32 target not shipped — FR-01/FR-02).
- Court: `test_fleet_class_a_workstation.rs` native leg.
- Boundaries (loud): wasm/OPFS-in-Chrome leg BLOCKED FR-01/FR-02;
  CDP zero-network assertion BLOCKED (no browser to attach to a native
  process).
- Metrics: 1M-row enterprise telemetry aggregation, operator's query
  verbatim (WHERE ts >= DATE, GROUP BY department, count/sum/quantile
  0.99); median-of-30 < 20 ms; RSS delta <= 512 MB.
- Receipt: /tmp/value_audit/fleet001.json

## Class B — industrial telemetry / low-RAM runner

- Hardware target: Linux cgroup runner — `memory.max=128MB`, iptables
  egress sever, 4h soak, `memory.events` OOM ledger. darwin has NO
  cgroup.
- Real gate: `scripts/doctor/cgroup_low_ram_runner.sh` (Linux+root only;
  exits 2 loud elsewhere).
- Proxy: `test_fleet_class_b_industrial.rs` — macOS memory-envelope leg;
  PARTIAL_ALIVE ceiling by construction, never a substitute for the
  kernel-enforced cap.
- Metrics (proxy): engine `memory_limit=128MB` config binds; 20 windows
  x 50k appender frames; post-warmup window RSS <= 110 MB; per-window
  RSS growth decelerating (+2 MiB noise allowance).
- Receipt: /tmp/value_audit/fleet002_proxy.json

## Class C — retail edge

- Hardware target: this host IS the Class-C-capable silicon (honest
  native run; no remote device, no cross-compile, no simulation).
- Court: `test_fleet_class_c_retail.rs`.
- Boundaries: none beyond the falsifiers — (a) imbalance query (HAVING
  store_imbalance <> 0.00) must return zero rows; (b) every epoch digest
  present and successful; (c) transmitted summary payload <= 4 KB.
- Metrics: 100k txns / 10 registers / 10 hourly epochs.
- Receipt: /tmp/value_audit/fleet003.json

## Class D — marine container telemetry (offline)

- Hardware target: native on this host + real subprocesses; a
  two-process chaos court kills -9 the writer mid-transaction.
- Court: `scripts/courts/fleet_d_chaos.sh` +
  `test_fleet_class_d_marine.rs`.
- Boundaries: non-vacuity — kill -9 -> kill -0 SHIM-COURT must flip the
  leg to VACUOUS-GATE; a court that cannot flip is FAIL, never PASS.
- Metrics: 1M events; exactly two legal terminal states (last-committed
  or in-txn sentinel; never a torn third), checksum == closed form
  sum(3n+7); compression (raw_jsonl - db) / raw >= 0.95; satellite
  delta-sync vector <= 500 KB (per-segment MD5 stream x 10k segments).
- Receipt: target/court/fleet-d-chaos-receipt.json

## Class E — telco

- Hardware target: native duckdb leg on this host; the BEAM
  :atomics/:ets actor leg lives in smalt.
- Court: `test_fleet_class_e_telco.rs` (duckdb leg only).
- Boundaries: BEAM actor leg BLOCKED-OTHER-REPO — the
  scheduler-starvation assertion needs the BEAM scheduler; echoed loud,
  never faked. The 1200 µs spec is an ingress-to-queryable bound; this
  leg measures the in-process floor of it.
- Metrics: 50k-row appender batch flush, median < 45 ms/batch over 40
  batches; session-to-queryable < 1200 µs (in-process).
- Receipt: /tmp/value_audit/fleet005.json

## Physical reality on THIS host (darwin)

| class | real silicon here? | consequence for the court |
|---|---|---|
| A | YES native; wasm/OPFS leg NO | native leg runnable; wasm/OPFS leg FR-01/FR-02-BLOCKED loud |
| B | NO — proxy only | cgroup is Linux+root; darwin exits 2 loud; proxy is PARTIAL_ALIVE ceiling |
| C | YES | native run is an honest Class-C witness |
| D | YES (native leg) | chaos + compression + satellite legs runnable (real subprocesses, no mocks) |
| E | duckdb leg YES; BEAM leg NO | duckdb leg runnable; BEAM leg BLOCKED-OTHER-REPO loud |

## Traceability matrix

| spec id | class | court file(s) | stage-12 leg | receipt path |
|---|---|---|---|---|
| TEST-FLEET-001 | A | test_fleet_class_a_workstation.rs | native | /tmp/value_audit/fleet001.json |
| TEST-FLEET-002 | B | cgroup runner + b_industrial proxy | exec + native | /tmp/value_audit/fleet002_proxy.json |
| TEST-FLEET-003 | C | test_fleet_class_c_retail.rs | native | /tmp/value_audit/fleet003.json |
| TEST-FLEET-004 | D | fleet_d_chaos.sh + d_marine.rs | exec + native | target/court/fleet-d-chaos-receipt.json |
| TEST-FLEET-005 | E | test_fleet_class_e_telco.rs | native | /tmp/value_audit/fleet005.json |

Traceability note (KD-12 spec amendment, 2026-10-19): the REV-004 /
TEST-REV-004 anomaly detection assertion (edge-autonomy court,
`crates/frozen-duckdb/tests/test_rev_edge_autonomy.rs`) is governed by
the amended contract in
`docs/sjira/v2.0.0/KD-12-SPEC-AMENDMENT.md` — recall >= 0.98 plus
false-positive ceiling <= 2x injected, NOT exact set match. Witnessed
receipt: docs/sjira/v2.0.0/receipts/rev004.json (recall 1.0000,
148 extras vs 10,000 injected).

## Stage-12 exit contract

`scripts/doctor/12_fleet_hardware.sh`: 0 = every executable leg PASS and
every blocked leg echoed loud; 1 = FAIL (leg ran and failed, or FAIL
co-occurs with NOT-IMPLEMENTED); 3 = NOT-IMPLEMENTED (leg absent or a
filter matched 0 tests). Exit 2 reserved for skip class; the cgroup
runner's darwin rc=2 propagates as NOT-IMPLEMENTED-LOUD. Stage 12 is NOT
wired into `scripts/doctor.sh` — the aggregate edit is the coordinator's
admitted transition after Wave 13 lands.
