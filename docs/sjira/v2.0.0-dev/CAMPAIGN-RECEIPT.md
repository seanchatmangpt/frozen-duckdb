# CAMPAIGN-RECEIPT — frozen-duckdb — v2.0.0-dev

| field | value |
|---|---|
| repo | `/Users/sac/frozen-duckdb` |
| branch | master |
| HEAD at receipt | `e4cfbd38f9efe55dde549c67d558da9769584fa0` (read-only `git rev-parse HEAD`, 2026-10-19) |
| standing | **PARTIAL_ALIVE** |

Basis (2026-10-19): engine v2.0.0-dev88595 ALIVE (witnessed strings probe,
F10); courts witnessed: 6/11 doctor stages green + stages 11 (revenue,
REV-001..004) and 12 (fleet A–E, aggregate EXIT=3 with every executable leg
PASS) + warehouse-sim determinism/drift/economics courts (evidence receipt.json
on disk) + native GATE courts (latency/throughput/OOM/crash-chaos) — mixed
WITNESSED / DISPATCH-CITED per the final reconciliation roll-up above; KDs
open: 11 of 14 (KD-01/02/05/06/07/08/09/10/11/12/13; KD-03/04/14 CLOSED);
blueprint layer PROPOSED (DOD-14); DOD-20 reconciliation REQUIRED. Blocks are
loud and named, never faked (host-VFS, browser/OPFS FR-01/FR-02, cgroup
rc=2 darwin, BEAM other-repo).

## Campaign commits this wave

| SHA | one-line |
|---|---|
| (filled at integration) | |

## Courts / gates witnessed

- (filled at integration)

## Open residues

- (filled at integration)
- Blueprint layer (BP lanes): written as PROPOSED design docs only
  (`docs/design/*.md`, `schema/blueprint.ttl`,
  `gates/blueprint/blueprint-shapes.ttl`, `LICENSE-ENTERPRISE-DRAFT.md`);
  no court has flipped, no layer claims ALIVE — see
  `docs/design/README.md` index (BP16).
- Engine 2.0.0-dev dylib rebuild from v2.0-cyanoptera pin `b6013f7`
  pending cache clear: the stale `v2.1.0-dev89402` self-reporting artifact
  was deleted; rebuild in flight at receipt time.
- Revision-2 intake (lane R15, 2026-10-19), facts only:
  - Dual-target PRD/ARD rev-2 captured in `docs/design/prd-ard-v2.md`
    ("Revision 2 mapping (PRD-FR / SLA / GATE-V)" section);
    `docs/design/requirements-traceability.md` aliases FR/NFR to
    `PRD-FR-nn` / `SLA-n`; `schema/requirements.ttl` carries `prd2:`
    individuals in the `prd:` scheme (pyshacl conformance stated by
    intake; not re-run by this lane).
  - Five GATE-V specs authored (`docs/design/gate-v0{1,2,4,5}-*.md`,
    GATE-V-03 in `acceptance-criteria-courts.md` AC-3); runner scaffold
    `scripts/courts/run_all.sh` + `gate_v0{1..5}.sh`. Real exit receipt:
    `bash scripts/courts/run_all.sh` -> all five courts
    `NOT-IMPLEMENTED`, aggregate `GATE-V: FAIL (fail-closed:`
    `NOT-IMPLEMENTED and FAIL both block)`, `EXIT=1`. Zero courts
    executed; no ALIVE claims added.
  - GATE-V-03 adopts rev2 numbers: 2GB probe replaces 256MB ceiling
    (traceability line 105). GATE-V-05 (<=5MB host-VFS transfer
    budget, `gate-v05-hostvfs-transfer.md`) is new — no prior court
    doc exists for it.
- X-wave intake (lane X19, 2026-10-19), facts only; verdicts recorded as
  found on disk at read time, else IN-FLIGHT:
  - Sovereign spec `docs/design/sovereign-spec-v26.10.19.md` captured;
    `docs/design/requirements-traceability.md` §"GATE-V-01..05 → owning
    courts" resolves the §7 master-matrix GATE-V-01 cell in favor of §5
    (rev2 three-query workload supersedes the earlier latency-plan list,
    traceability lines 109-111).
  - Native courts implemented on disk: X3
    `crates/frozen-duckdb/tests/gate_v01_latency.rs`, X8
    `crates/frozen-duckdb/tests/gate_ingest_throughput.rs`, X9
    `crates/frozen-duckdb/tests/oom_graceful_abort.rs` (all present).
    X4 `crash_chaos_native.sh` NOT found on disk — IN-FLIGHT. No X-lane
    verdict receipt files landed under `receipts/` at read time; X3/X4/X8/X9
    verdicts recorded IN-FLIGHT (courts written, runs not yet receipted).
  - OCEL sovereign schema validation: no schema artifact or validation
    receipt found on disk at read time — IN-FLIGHT (not landed, not
    labeled-pass).
  - BEFORE receipts (X16/X17) for the post-rebuild AFTER flip: none
    present under `docs/sjira/v2.0.0-dev/receipts/` at read time —
    IN-FLIGHT.
  - Rebuild from v2.0-cyanoptera pin `b6013f7`: new dylib landed at
    `~/.frozen-duckdb/cache/v2.0.0-dev-arm64/libduckdb.dylib`
    (mtime Oct 19 13:22, 117005184 bytes); self-reports `v1.5.5` in
    strings scan — the stale `v2.1.0-dev89402` self-report is gone.
    Version-string flip to v2.0.0-dev and court re-runs on the rebuilt
    dylib remain open.
- Doctor battery (D/E lanes, 2026-10-19), facts only:
  - Scaffold + phase tests landed: doctor.sh + phase scripts + justfile
    (D2), attest_pin (D3), storage/types/concurrency/ocel tests (D4/D5/D7/
    D8), leak audit (D6), blueprint + blocked-phases + implementation map
    + dual-pass spec docs (D1/D9/D10/D11).
  - E-wave wiring landed: phase wiring (E1), crash-chaos court
    `crash_chaos_native.sh` (E2), bin audit (E3), dual-pass runner
    `dual_pass.sh` (E4).
  - End-to-end validate (E5, read-only): fail-closed pre-rebuild state
    expected and observed — courts cannot attest engine identity on the
    current cache slot; no ALIVE flips claimed this wave.
  - TEST-017 id discrepancy (D9 finding) deferred to implementation-map
    reconciliation (D10).
  - Cache-poisoning incident: v1.5.5 bytes were staged into the
    v2.0.0-dev slot by an unidentified lane build path mid-window
    (~13:22); quarantined by operator. All post-13:22 test receipts on
    that slot are VOID for engine identity; courts must be re-witnessed
    post-rebuild.
- Warehouse-sim testbed (lane G7, 2026-10-19), facts only; standing as
  found on disk at read time:
  - Testbed scaffolded: `warehouse-sim/` landed with Next.js 14.2.15
    scaffold (`package.json`, `next.config.mjs`, `tsconfig.json`,
    `next-env.d.ts`, `.gitignore`), `node_modules/` installed, and the
    ontology `warehouse-sim/schema/warehouse_sim.ttl` (rewritten mid-wave
    by G1 to the `sim:` namespace: endpoints modeled against
    `sim:SovereignBaseline`, poll states, cold-start/jitter, metering
    formulas).
  - ggen-rendered surfaces: PARTIAL. `ggen.toml` + `templates/` + rendered
    `lib/sim_engine.ts` landed (G2), but `app/api/` routes did NOT render:
    real run, `bash courts/determinism.sh`, exit 0 with loud skip —
    `[SKIP] ggen sync run --dry-run failed (rc=1); G2 render not usable`;
    underlying ggen error: `[FM-GEN-008][TEMPLATE_VARIABLE_MISSING]
    template=templates/snowflake_submit_route.ts.tera
    rule=snowflake-submit-route: Variable r.endpointLabel not found in
    context` (ggen 26.10.11, 2026-10-19T21:08:35Z). Standing: PARTIAL_ALIVE
    for the graph+lib surface, BLOCKED for `app/api/` render until G2
    repairs the template context.
  - Determinism court: LANDED (`courts/determinism.sh`); verdict witnessed
    above — loud SKIP, not a pass. Standing: PARTIAL_ALIVE (runs, fails
    loud on the blocked render; no PASS witnessed this window).
  - Economics court: PARTIAL. Sovereign probe leg landed
    (`economics/sovereign_probe/` — real engine, rev2 three-query workload,
    seed 20261019, 100k rows, egress 0 / invoice $0.00 by construction);
    sim-leg runner for the three fail-closed assertions not on disk at
    read time. Standing: UNKNOWN (IN-FLIGHT).
  - Design + run docs landed this lane:
    `warehouse-sim/README.md`, `docs/design/warehouse-sim-testbed.md`.
  - All sim cost constants (`creditCostPerHour 4.0`, `dcuCostPerHour
    4.5`, `scanRatePerTB 65.0`) are ASSUMPTION-UNVERIFIED
    counterfactual parameters; measured truth comes only from the
    two-arm harness in `docs/design/economic-model.md`, which has not
    run. No ALIVE claim is made for any warehouse-sim surface.
- Stage-matrix reconciliation (lane H4, 2026-10-19), facts only; standing
  as found on disk at read time:
  - The 11-stage ladder is formalized per operator spec as the doctor
    stage matrix: 11 stages 00..10 under `scripts/doctor/`, aggregated
    fail-closed by `scripts/doctor.sh` (11 phase banners; NOT-IMPLEMENTED
    exit 3 fails the aggregate loudly).
  - Scripts renamed to the stage matrix by the D/E waves (old phase-N
    filenames shifted; full table in the Wave 10 reconciliation section
    of `docs/design/doctor-implementation-map.md`).
  - `court-x2` wired: sys-audit stage (`01_sys_audit.sh`) propagates the
    verdict of `scripts/doctor/checks/bin_audit.sh`; crash-chaos court
    `scripts/courts/crash_chaos_native.sh` on disk.
  - Stage standings cited on disk at read time: storage (`02_storage_wal`)
    2/2 green; types (`03_types_variant`) 3/3 green; memory
    (`05_memory_guard`) PARTIAL_ALIVE; OCEL (`07_ocel_compress`) 94.93%
    ALIVE; perf (`08_perf_sla`) non-vacuous PASS; host-VFS
    (`04_host_vfs`) and browser/OPFS (`06_browser_opfs`) BLOCKED FR-01;
    economics (`10_verify_economics`) IN-FLIGHT — its court runner
    `scripts/warehouse-sim/economics/run_court.sh` is not on disk, the
    stage exits 3 loudly, never a faked economics pass.
  - Stale remnant flagged for coordinator: `scripts/doctor/
    09_court_ladder.sh` survives alongside `09_concurrency.sh`; not in
    the `doctor.sh` PHASES array.
  - Rogue-writer incident class (this window, witnessed 4x on one file +
    1x on another + 1 cache event): an unidentified concurrent writer
    clobbered F3's file four times and G1's ontology once; the cache
    dylib went non-Mach-O mid-window. All affected files re-verified
    post-incident; receipts for pre-incident reads on those files are
    superseded. Requirement recorded: a final-integration
    reconciliation pass is REQUIRED before any commit — coordinator
    re-reads every co-touched surface on disk, reconciles against lane
    receipts, and only then commits.

- S-wave residues (lane S7, 2026-10-19), facts only; standing as found
  on disk at write time — all S-lane statuses are IN-FLIGHT:
  - Server/client testing lanes S1-S6 dispatched to validate the
    SaaS-disruption contrast end-to-end (route contracts, SDK-fidelity
    clients, analyst journeys, chaos + concurrency, disruption
    scorecard, admin observability). None has returned a receipt as of
    this write; statuses recorded IN-FLIGHT, standing UNKNOWN until
    receipts land.
  - G2's render self-healed: `ggen sync run` now completes and a
    byte-identical double-sync is green (render -> re-render diff
    empty), lifting the prior FM-GEN-008 `app/api/` BLOCKED noted by
    lane G7. Standing of the rendered surface: repaired, receipts
    pending the S-wave runs.
  - G6's receipt-chain court is FAIL-closed at
    `warehouse-sim/economics/court.ts:117` (chain assertion refuses
    rather than pass on the missing-link path). S5 is authorized to
    fix minimally at that site only; the fix must keep the court
    fail-closed — a repair that converts a genuine chain gap into a
    pass is a vacuous-gate defect, not a fix.
  - All sim cost constants remain ASSUMPTION-UNVERIFIED; no ALIVE
    claim is made for any S-wave surface in this entry.

- Revenue-pack battery (lane R5, 2026-10-19), facts only; standing as
  found on disk at write time:
  - Battery dispatched: R1-R4 own the four TEST-REV courts
    (`test_rev_compliance.rs`, `test_rev_process_mining.rs`,
    `test_rev_clean_room.rs`, `test_rev_edge_autonomy.rs` — all four
    files on disk under `crates/frozen-duckdb/tests/`); R5 owns the
    fail-closed doctor stage `scripts/doctor/11_revenue_packs.sh`, the
    pack spec doc `docs/design/revenue-packs.md`, and this
    bookkeeping. Stage 11 is NOT wired into `scripts/doctor.sh` — that
    aggregate edit is the coordinator's admitted transition after the
    lanes land.
  - Real stage run (this lane, `CARGO_TARGET_DIR=target-laneR5 bash
    scripts/doctor/11_revenue_packs.sh`): EXIT=1, all four legs FAIL —
    cargo rc=101 compile errors, not test failures: E0277
    `Vec<Box<dyn ToSql>>: AppenderParams` at
    `test_rev_compliance.rs:81` (6 errors), E0277
    `(i64,i64,i64,f64): AppenderParams` at
    `test_rev_edge_autonomy.rs:131`. The stage correctly refuses to
    read a compile abort as a pass. No PASS witnessed for any REV leg;
    standing IN-FLIGHT, owned by R1-R4. Browser/OPFS leg echoed loudly
    BLOCKED FR-01/FR-02 in the stage output (never faked).
  - CORRECTION RECORD: X4's earlier "WAL-replay atomicity defect"
    finding was an artifact of E2's broken fixture script (the fixture
    bugs are itemized in E2's receipt) — the finding does NOT evidence
    an engine defect. Re-facts witnessed after the correction: Z2's
    court ran green on 3 legs; GATE-V-02 native leg is PARTIAL_ALIVE
    green. No upstream (engine) defect is claimed by this correction;
    the defect claim is withdrawn to the fixture layer where it
    belongs.

- Fleet hardware battery (lane T6, 2026-10-19), facts only; statuses per
  disk at read time (files were still landing from T1-T5 mid-lane):
  - Battery dispatched: T1 owns `scripts/doctor/cgroup_low_ram_runner.sh`
    (Class B real gate); T2 owns `scripts/courts/fleet_d_chaos.sh` +
    `test_fleet_class_d_marine.rs`; T3 owns
    `test_fleet_class_a_workstation.rs`; T4 owns
    `test_fleet_class_b_industrial.rs`; T5 owns
    `test_fleet_class_c_retail.rs`; T6 owns
    `scripts/doctor/12_fleet_hardware.sh` +
    `docs/design/fleet-hardware-classes.md` + this bookkeeping. Stage 12
    is NOT wired into `scripts/doctor.sh` — aggregate edit is the
    coordinator's admitted transition.
  - Real stage run (this lane, `CARGO_TARGET_DIR=target-laneT6 bash
    scripts/doctor/12_fleet_hardware.sh`), final snapshot EXIT=1
    "STAGE 12 FLEET HARDWARE: FAIL + NOT-IMPLEMENTED — fail-closed":
    - Class A native leg: cargo rc=101 (compile break in a concurrently
      in-flight T-lane file, not a test failure).
    - Class B cgroup leg: NOT-IMPLEMENTED-LOUD, runner not on disk.
    - Class B proxy leg: cargo rc=101 — syntax error at
      `test_fleet_class_b_industrial.rs:138` ("expected `{`, found `[`"),
      T4-owned in-flight file.
    - Class C leg: cargo rc=101 (in-flight compile break, T5-owned).
    - Class D chaos court: EXECUTED and FAILED rc=1 —
      "VERDICT FAIL: engine identity probe produced no version"
      (real court verdict, owned by T2).
    - Class D marine compression leg: cargo rc=101 — break at
      `test_fleet_class_d_marine.rs:213`, T2-owned in-flight file.
    - Class E duckdb leg: cargo rc=101 (in-flight compile break); BEAM
      leg echoed loud BLOCKED-OTHER-REPO (smalt), never faked.
    - Class A wasm/OPFS leg echoed loud BLOCKED FR-01/FR-02.
  - An earlier same-lane run hit a SEPARATE pre-existing environment
    defect: corrupted cargo-registry extractions (`memchr-2.8.3`,
    `libm-0.2.16` missing module files under
    `~/.cargo/registry/src/`), E0583/E0433. A registry re-extraction
    repair was attempted and DENIED by the permission system; the repair
    is coordinator-routed. The defect self-resolved by a concurrent
    lane's re-extraction before the final snapshot run (same cargo
    filters then compiled past the registry and reached the T-lane
    source errors above).
  - No PASS witnessed for any fleet leg by T6; standing IN-FLIGHT, owned
    by T1-T5. The stage correctly refuses to read a compile abort as a
    failure count or a missing leg as a pass (census vs compile-abort
    rule; rc=0-with-0-tests is exit-3 class, not a pass).
  - CORRECTION (T6, same lane, minutes later): T1-T5 landed and fixed
    their files; T6 also fixed its own stage-side filter bug (legs now
    invoke `cargo test --test <target>` — the earlier name-filter
    matched 0 test fns because the fns are `fleet00N_*`, not the file
    names). Final snapshot run (`CARGO_TARGET_DIR=target-laneT6 bash
    scripts/doctor/12_fleet_hardware.sh`) EXIT=3:
    - CLASS A native: PASS (1 test ok)
    - CLASS B cgroup: runner now ON DISK, exited rc=2 on darwin per its
      contract — propagated NOT-IMPLEMENTED-LOUD by stage 12 (witnessed
      the rc=2 propagation path live)
    - CLASS B proxy: PASS (1 test ok)
    - CLASS C: PASS (1 test ok)
    - CLASS D chaos court: VERDICT PASS — "2 real kill -9 legs green
      (200000/260000 events in-flight); completed leg green at 300000;
      no-op-kill flip DETECTED", rc=0; compression leg: 3 tests ok
    - CLASS E duckdb leg: PASS (2 tests ok); BEAM leg loud
      BLOCKED-OTHER-REPO
    Aggregate EXIT=3 on this host is the dispatched behavior: every
    executable leg passes, but the Class B cgroup gate requires Linux
    and exits 2 loud here, so the ALL-CLEAN line does not fire on
    darwin. It can only fire on a Linux host with root.

## Final reconciliation (H4/GAP13, 2026-10-19) — Wave 14 walk

Method: every standing below was re-read from disk this pass, not
recalled from session memory. Receipts under `receipts/` remain thin
(only `VERIFIER-RECEIPTS-LOG.md` is present); where a final standing
cites a lane receipt not materialized as a file, the owning artifact
(script/test/receipt.json) was verified present on disk and the
dispatch figure is recorded as DISPATCH-CITED, distinct from
WITNESSED. Lines ≤100 chars.

### Engine flip (F10) — ALIVE, WITNESSED this pass

- `strings ~/.frozen-duckdb/cache/v2.0.0-dev-arm64/libduckdb.dylib |
  grep -m1 v2.0.0-dev` -> `v2.0.0-dev88595`. Stale `v2.1.0-dev89402`
  self-report: absent. `v1.5.5` string still present in the binary
  (compat surface), no longer the identity.
- dylib sha256 prefix witnessed this pass:
  `acb8bc49ebae74f5…`; dispatch identity `8a22ef19` recorded as the
  F10 flip citation (thin arm64, cache slot
  `v2.0.0-dev-arm64`, dir mtime Oct 19 13:40).
- Standing: ALIVE (identity flip WITNESSED; sha dual-cited as
  DISPATCH-CITED `8a22ef19` / WITNESSED `acb8bc49ebae74f5`).

### Court battery finals (dispatch-cited; owning artifacts verified on disk)

- Storage `02_storage_wal.sh`: 2/2 — WITNESSED this pass:
  `test result: ok. 2 passed; 0 failed`, EXIT=0 (/tmp/st02.log).
- Types `03_types_variant`: 3/3 green — DISPATCH-CITED
  (artifacts on disk). Leak-audit deceleration law: green.
- Concurrency `09_concurrency`: 2/2 green with vacuity catch
  recorded — DISPATCH-CITED.
- OCEL `07_ocel_compress`: 94.93/97.01% (compress / query) plus
  SHIM flip — DISPATCH-CITED.
- Latency: 10.5-12.4ms, non-vacuous both directions —
  DISPATCH-CITED. Throughput: 12.16M rows/sec release —
  DISPATCH-CITED. OOM graceful-abort: both legs X9 green —
  DISPATCH-CITED.
- Crash-chaos: PARTIAL_ALIVE green (Z2, 3 legs + flip) —
  DISPATCH-CITED; `scripts/courts/crash_chaos_native.sh` on disk.
- X4-defect WITHDRAWAL: "WAL-replay atomicity defect" was an E2
  fixture bug (correction already on record in the residues above);
  final state: no engine defect claimed, fixture-layer defect.
- Revenue packs: REV-001 variance-0 with honest UNMET-clause
  report; REV-002 6.33ms; REV-003 18.84ms ALIVE; REV-004 recall
  1.0 OBSERVED (not self-claimed) — DISPATCH-CITED; stage
  `scripts/doctor/11_revenue_packs.sh` on disk, `11_revenue_packs`
  wired into `doctor.sh`.
- Fleet classes: A 2.59ms; B proxy 80.5MiB envelope + cgroup gate
  loud-skip on darwin (rc=2 propagation witnessed live by T6); C
  54.77ms; D 99.76% compression + chaos 2 kill legs + no-op-kill
  flip DETECTED; E 11.6ms + BEAM loud BLOCKED-OTHER-REPO —
  DISPATCH-CITED; stage 12 artifacts on disk.
- Warehouse-sim: determinism + drift courts PASS (determinism.diff
  bytes 0, `warehouse-sim/courts/receipt.json` — WITNESSED on
  disk); economics court PASS x3 + selftest (evidence at
  `warehouse-sim/economics/.evidence/run-*/receipt.json`, seed
  20261019, 100k rows, egress 0 / invoice $0.00 — WITNESSED on
  disk); scorecard 5x CLEAR; admin 5/5; clients 6/6; chaos 5/5;
  journeys 4/4 — DISPATCH-CITED.
- Doctor ladder: 6/11 executable stages green + loud blocks on the
  BLOCKED FR-01/FR-02 surfaces (host-VFS, browser/OPFS) —
  DISPATCH-CITED; `scripts/doctor.sh` + stages 00-12 on disk,
  stages 11/12 now wired.

### Incident record — final state

- Cache poison (v1.5.5 bytes staged into the v2.0.0-dev slot
  ~13:22): quarantined by operator; rebuilt dylib in the slot
  self-reports `v2.0.0-dev88595` (witnessed above); receipts
  post-13:22 were re-witnessed where required — receipts still
  lacking a re-witness remain marked as such, never upgraded.
- Cargo registry corruption (`memchr-2.8.3`, `libm-0.2.16` E0583/
  E0433): root-caused to the cleanup actor, not a lane; repaired
  in place by re-extraction (a later lane's build compiled past
  the same filters, witnessed in T6's final snapshot).
- Rogue-writer class (4x F3 file + 1x G1 ontology + 1 cache event):
  all affected surfaces re-verified post-incident; pre-incident
  reads on those files superseded.

### Standing roll-up

| surface | standing | basis |
|---|---|---|
| engine v2.0.0-dev88595 dylib | ALIVE | WITNESSED (strings probe above) |
| storage/types/leak/concurrency courts | PASS | DISPATCH-CITED |
| OCEL 94.93/97.01 + SHIM flip | PASS | DISPATCH-CITED |
| latency/throughput/OOM courts | PASS | DISPATCH-CITED |
| crash-chaos (Z2) | PARTIAL_ALIVE | DISPATCH-CITED, court on disk |
| revenue packs REV-001..004 | PASS / OBSERVED | DISPATCH-CITED |
| fleet classes A-E | PASS + loud blocks | DISPATCH-CITED |
| warehouse-sim courts + S-wave | PASS | mixed WITNESSED/DISPATCH-CITED |
| doctor ladder | 6/11 + loud blocks | DISPATCH-CITED |
| browser/OPFS (FR-01/FR-02) | BLOCKED | loud, never faked |
| blueprint layer | PROPOSED | no court flipped (BP16) |
| cost constants (sim) | ASSUMPTION-UNVERIFIED | counterfactual params |

## Release-prep wave (REL13, 2026-10-19) — final residues

Facts only; statuses as found on disk at read time. No lane-claimed
PASS is upgraded by this entry; WITNESSED means this lane ran the probe
itself this pass.

### Standing corrections (KD register)

- **KD-01 (native dylib symbol table)**: demangled census corrected to
  **27,682** C++-mangled `__Z*` T exports — the prior "21" was an
  audit-regex artifact in `bin_audit` (only `__ZZ` local symbols
  matched). Source: `nm -gU … | grep -cE 'T[[:space:]]*__Z'` -> 27682;
  `strip -x` no-op on exports (LC_DYLD_EXPORTS_TRIE). Receipt:
  `receipts/ADV3-export-fence.md` (WITNESSED by ADV3, cited here).
- **KD-09 (Class B cgroup)**: docker-closed. `KNOWN_DIVERGENCES.md`
  KD-09 row now carries the DOCKER1 container leg: RSS HELD under hard
  `--memory 128m`, exit 0; 32MB negative control exit 137 (kernel limit
  binds; virtualized-kernel caveat recorded). Row standing
  PARTIAL_ALIVE; `scripts/doctor/docker_low_ram_runner.sh` +
  `Dockerfile.lowram` on disk.
- **KD-15 (consumer @rpath)**: macOS-closed. install-ID stamp at the
  cache persist layer + self-heal in `accept_cached_entry`; builder
  43/43 green; live cache stamped in place, pin re-staged. Linux
  remains UNVERIFIED (ELF has no LC_ID_DYLIB; stamp is a no-op there).
  Receipt: `receipts/ADV9-rpath.md`.

### REL lane deliverables — status at read time

- REL5 Windows feasibility table: LANDED —
  `receipts/REL5-windows.md`. Key findings: upstream v1.5.5 DOES ship
  Windows prebuilts (`libduckdb-windows-{amd64,arm64}.zip`, live API
  probe exit 0); builder naming law already maps windows -> `.dll`
  (unit-tested); `verify_library_slice_for` is macOS-only and degrades
  to Unverifiable elsewhere (honest, disclosed).
- REL6 Linux .so candidate: **IN-FLIGHT** —
  `scripts/build_linux_prebuilt.sh` on disk (linux/arm64 container
  build from pin `b6013f7`, colima/docker); `/tmp/rel6` exists and is
  EMPTY; no `rel6-duckdb-build` container present. No `.so` artifact,
  no build receipt at read time.
- REL7 CI Linux leg proposal: landed as a diff —
  `.github/workflows/build-binaries.yml` carries `build-binaries-linux`
  (job + artifact upload `libduckdb_${arch}-linux`, wiring into the
  publish job via `needs`); modified-not-committed at read time.
- REL8 asset checklist extension: landed as a diff —
  `scripts/verify_release_assets.sh` carries the +74-line extension;
  modified-not-committed at read time.
- REL9 release-notes pattern alignment:
  `RELEASE-NOTES-2.0.0.md` landed, follows the standing pattern
  (prerelease channel + upstream pin `b6013f7`, PARTIAL_ALIVE framing,
  every claim naming its receipt; DISPATCH-CITED numbers cited from the
  Wave 14 roll-up).
- REL1-REL3 (builder/sys/API re-witnesses), REL4 (consolidated test
  plan), REL10 (known-limitations doc), REL11 (pre-flight re-run):
  statuses as found — no receipts under `receipts/` matching these
  lanes at read time; recorded IN-FLIGHT, owned by their lanes.
- Wave 16 standing: PARTIAL_ALIVE — Windows table + release notes +
  CI/checklist diffs landed; Linux .so candidate and the remaining
  REL lanes IN-FLIGHT; three KD standing corrections on record with
  receipts cited.
