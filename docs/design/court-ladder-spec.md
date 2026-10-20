# court-ladder-spec

Status: GOVERNING STAGE MATRIX for the Chicago Verification Courts program.
The CURRENT repo wiring (`docs/design/doctor-implementation-map.md`) uses a
10-phase 0-9 layout; the reconciliation from that layout to the 13 stages
on disk (00..12 under `scripts/doctor/`, 6 green, remaining KD-mapped) is
lane H2's transition and is recorded in the map. Where the
two disagree, THIS matrix is the target state.

- Scope: PRD/ARD transcription of the operator's mandate for verification
  courts over the frozen-duckdb substrate.
- Standing column: honest as-of 2026-10-19. Real green courts vs PROPOSED.
- Normative language: MUST / SHOULD / MAY per usual; falsifiers are binding.

## 1. Executive mandate

The substrate carries five claims. Each claim gets a court; a court is an
executed falsifier, not a narrative. A claim without a green court receipt
is UNKNOWN, regardless of how confident the prose about it is.

| # | Substrate claim | Court subject |
|---|---|---|
| C1 | Zero-compilation DuckDB via builder sub-crate: link real | 01 doctor |
| | library, no local cargo compile of DuckDB in the hot path | |
| C2 | Drop-in duckdb-rs replacement: same API surface, real storage | 04 types |
| C3 | Full type fidelity across the FFI boundary | 04 types + 06 memory |
| C4 | Acceptable footprint and memory behavior | 06 memory + 08 compression |
| C5 | Performance claims (latency, economics) are measured, not | 09 latency + 11 |
| | asserted | economics |

Law: A=mu(O*), R=receipt(A). A stage that has not produced a receipt with
real command output does not exist for planning purposes.

## 2. Problem statement

Three hazards motivate the ladder. Each is a witnessed failure class, not a
hypothetical.

1. **Synthetic-mock hazard.** Tests that mock the collaborator under test
   verify the test's model of the system, not the system. A mock passes
   precisely because it was built to agree. Any stage that can pass with a
   fake FFI, fake storage, or a stubbed DuckDB is vacuous. (Ref:
   testing-chicago-style: real collaborators, assert on final state.)

2. **Unverified economic claim.** The substrate implies an economic claim
   (compile-time and download-bandwidth savings vs source builds). This
   claim has never been executed as a measurement; it is currently an
   unreceipted assertion. Stage 11 exists to make it a number or retire it.

3. **Flaky timing.** Duration assertions with absolute millisecond bounds
   are load-bound and flake exactly under the load that matters (witnessed:
   fence-10 class flakes). Time is not a proxy for state; bounded polls on
   real observables replace sleep-and-hope. (Ref: no-sleep-testing.)

## 3. Personas

- **Operator (Sean)**: issues the mandate, reads receipts, kills lanes.
  Skims; front-load signal. Trusts only receipts with real output.
- **Court author (lane agent)**: writes stage specs and their falsifiers.
  Must make every gate able to exit non-zero; a gate that cannot fail is
  not a gate.
- **Integrator (coordinator)**: owns git transitions, runs the ladder
  end-to-end, deletes lane build dirs at integration, records standing
  flips gated on receipts.
- **Skeptic (adversarial lane)**: grades courts, never builds what it
  audits; hunts vacuous passes and tautological assertions.

## 4. Guiding laws (four)

### L1. Zero-mock

Real collaborators wherever practical: real subprocess, real files, real
local DuckDB library. Mock/patch/mockall of an owned, locally-runnable
collaborator is banned by default. A hand-written real implementation of an
interface is NOT a mock. Only exception: infeasible real collaborator, with
(1) stated reason, (2) a real degraded alternative considered first,
(3) a simple fake, not a mock, named as such.

### L2. Zero-sleep

`Process.sleep` / `thread::sleep` as a coordination or test-orchestration
step is banned. Bounded polls on real observables (deadline-checked,
immediate typed failure on unexpected shape) or structural sync instead.
Absolute ms bounds fence only the first leaked sleep; prefer relative ratio
fences against a sleep-free baseline. Only lawful sleep is runtime
scheduling of a process that genuinely has nothing to do
(`# SLEEP-CLASS: SCHEDULING`).

### L3. Non-vacuity

Every gate must be able to refuse. A gate that accepts every input is
vacuous and its PASS carries zero bits. Non-vacuity is proven by mutation:
inject a known-bad variant, the gate MUST refuse (shim-court pattern:
BASELINE -> BEFORE -> SHIM -> AFTER -> RESTORE with sha256 restore check).
A skip is loud (`[SKIP] <reason>`), never a faked pass.

### L4. Fail-closed exit contract

Every stage command obeys one exit contract, no exceptions:

| Exit | Meaning |
|---|---|
| 0 | PASS — court ran, all assertions held, receipt emitted |
| 1 | FAIL — court ran, at least one assertion broken (loud, typed) |
| 2 | REFUSED — preconditions missing (env, artifact, authority); |
| | court did NOT run; must never be read as PASS |
| 3 | SKIP — named, reasoned skip of a sub-check; never silent |
| 124 | TIMEOUT — deadline exceeded; counts as FAIL, never retry-silent |

Anything outside this set is a harness defect. CI keys on exit codes only.

## 5. The 13-stage ladder

```
[01 doctor] -> [02 env] -> [03 storage] -> [04 types] -> [05 host-vfs]
     |                                                  |
     +--> [06 memory] -> [07 browser] -> [08 compression]
                                              |
     [09 latency] -> [10 concurrent] -> [11 verify-economics]
                                     -> COURT-x2 (dual pass)
```

Reflow (<= 98 chars):

```
[01 doctor] -> [02 env] -> [03 storage] -> [04 types] -> [05 host-vfs]
    -> [06 memory] -> [07 browser] -> [08 compression]
    -> [09 latency] -> [10 concurrent] -> [11 verify-economics]
    -> COURT-x2 dual pass (each stage passes twice before standing flip)
```

## 6. Stage specs

### 01 doctor — substrate health
- Mechanism: real link of the mega-library through frozen-duckdb-sys;
  verify vendored headers present; no local DuckDB compile attempted.
- Assertions: library resolves and opens a real in-memory DB; `SELECT 42`
  returns 42 over the real FFI; exit contract L4 holds.
- Standing: wired in the 10-phase map (phase 0). See Status header.

### 02 env — environment pin
- Mechanism: assert toolchain, DuckDB version pin (1.5.5), cache dir
  `~/.frozen-duckdb/cache/` state, extension availability.
- Assertions: version fingerprint matches pin; missing artifact yields
  exit 2 (REFUSED), not a silent pass.
- Standing: wired in the 10-phase map (phase 1).

### 03 storage — real persistence
- Mechanism: real file-backed DB over real FFI: create, insert, commit,
  close, reopen, verify.
- Assertions: rows survive a real close/reopen cycle; crash-simulated
  variant (kill between insert and reopen) is handled without fake data.
- Standing: ALIVE — court 2/2 green.

### 04 types — type fidelity
- Mechanism: round-trip every supported type through the FFI boundary.
- Assertions: for each type T: write(T) -> read == T, including NULL,
  edge values (min/max), DECIMAL scale, BLOB bytes, TIMESTAMP precision.
- Standing: ALIVE — court 3/3 green.

### 05 host-vfs — host filesystem abstraction
- Mechanism: VFS-level access against the host FS via the real library.
- Assertions: file create/read/write/delete observable through the DB;
  bounded poll on observable file state, no sleeps.
- Standing: PROPOSED.

### 06 memory — memory behavior
- Mechanism: allocate/write/read/destroy large sets; observe real memory
  counters, not call counts.
- Assertions: bounded memory growth under a fixed workload; no leak across
  N cycles; deterministic cleanup observable in final state.
- Standing: PARTIAL_ALIVE — core path exercised, leak fence not closed.

### 07 browser — browser/wasm surface
- Mechanism: browser-or-wasm target court per FR-01.
- Assertions: FR-01 blocked — the surface cannot yet run a real court.
- Standing: PROPOSED (FR-01-blocked). Gate MUST exit 2 (REFUSED), never
  fake a pass while blocked.

### 08 compression — footprint
- Mechanism: compress/decompress real data sets through the library.
- Assertions: measured compression ratio against a pinned corpus; lossless
  round-trip byte equality on every sample.
- Standing: measured 94.93% ratio on pinned corpus — court green.

### 09 latency — timing without flake
- Mechanism: paired rounds, min-of-N against an independent sleep-free
  baseline (ratio fence, per L2). No absolute ms bounds.
- Assertions: non-vacuity proven — the gate refuses an injected slow path
  (12.3ms observed vs injected degrade); poll-based, zero sleeps.
- Standing: non-vacuous, 12.3ms measured — court green.

### 10 concurrent — parallel access
- Mechanism: N real threads/processes against one real DB instance.
- Assertions: no interleaving corruption; serializable outcomes on
  conflicting writes; bounded poll on quiescence, no sleeps.
- Standing: PROPOSED.

### 11 verify-economics — the economic claim
- Mechanism: execute the claim as measurement: cold-build-from-source time
  and bandwidth vs cache-hit link path, on pinned subjects.
- Assertions (the formulas):
  - `savings_time = T_source_build - T_cache_link` (must be > 0 on pinned
    hardware for the claim to stand)
  - `savings_bytes = S_source_tarball - S_prebuilt_artifact` (must be > 0)
  - `payback = T_cache_link_first_run + T_cache_fill` vs `T_source_build`
    (cache path must pay back within one cold start)
- Standing: PROPOSED (in-flight).

## 7. COURT-x2 dual-pass invariant

Every stage command runs TWICE per standing flip: pass 1 proves the
assertions hold; pass 2 proves they hold again on a fresh process (catches
state leakage, ordering luck, one-shot luck). Both passes must exit 0 and
both receipts must be retained. Three pass-2-specific assertions:

1. **Pass 2 is fresh**: pass 2 starts a new process; any state that only
   pass 1 could have created (caches, locks, temp files) must not be
   required by pass 2 — or must be declared part of the fixture contract.
2. **Pass 2 is identical in spec**: same command, same env contract, same
   exit semantics; only the receipt index differs.
3. **Pass 2 catches mutations too**: if a non-vacuity mutation (L3) is
   injected, pass 2 MUST also refuse — a court that passes pass 2 after
   passing a mutated pass 1 is vacuous.

## 8. Machine-readable matrix

| Stage | Command | Invariant | Standing |
|---|---|---|---|
| 01 doctor | `make doctor` (per map) | L4 exit contract; real FFI open | wired (map) |
| 02 env | `make env-check` | exit 2 on missing pin; no silent pass | wired (map) |
| 03 storage | `cargo test -p frozen-duckdb -- storage` | close/reopen persistence; 2/2 | ALIVE |
| 04 types | `cargo test -p frozen-duckdb -- types` | full round-trip fidelity; 3/3 | ALIVE |
| 05 host-vfs | `court run 05-host-vfs` | FS observables via real VFS | PROPOSED |
| 06 memory | `court run 06-memory` | leak fence over N cycles | PARTIAL_ALIVE |
| 07 browser | `court run 07-browser` | FR-01; exit 2 while blocked | PROPOSED (FR-01-blocked) |
| 08 compression | `court run 08-compression` | ratio + lossless round-trip; 94.93% | ALIVE |
| 09 latency | `court run 09-latency` | ratio fence; non-vacuous; 12.3ms | ALIVE |
| 10 concurrent | `court run 10-concurrent` | no corruption under N writers | PROPOSED |
| 11 verify-economics | `court run 11-economics` | savings formulas > 0 | PROPOSED (in-flight) |

## Falsifier (whole ladder)

A stage or the program is dead if: any court passes while its real
collaborator is stubbed; any gate accepts a non-vacuity mutation; any
standing flip cites no receipt; the matrix above contradicts
`docs/design/doctor-implementation-map.md` after H2's reconciliation lands.

## See Also

`docs/design/doctor-implementation-map.md` · `scripts/check_sjira_standing.sh`
