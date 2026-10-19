# KD Closure Plan — v2.0.0-dev

Lane FIN7, 2026-10-19. Per-row closure plan for the 11 open KD rows in
`/Users/sac/frozen-duckdb/KNOWN_DIVERGENCES.md`. Closed rows (KD-03, KD-04,
KD-14) are out of scope.

Standing vocabulary: open rows are BLOCKED, PARTIAL_ALIVE, or UNKNOWN.
Release framing: the engine release gate is `make verify` + `make gates` +
the test ladder (repo CLAUDE.md). None of the open KD rows gate
`cargo test --workspace`. They gate the SOVEREIGN platform claims
(browser/OPFS, wasm, host-VFS, hardware-honest latency) only.

Evidence re-read on disk this session:
- `/tmp/value_audit/rev001.json` (KD-07), `rev004.json` (KD-12),
  `gate_v01_receipt.json` (KD-08), `fleet002_proxy.json` (KD-09) exist.
- `/tmp/value_audit/fleet002_cgroup_docker.json` DOES NOT EXIST (no
  `*cgroup*` file under /tmp/value_audit/) — the DOCKER1 leg has not
  landed a receipt. KD-09 docker leg status: IN FLIGHT, no witness.

## Per-row plans

Format: plan / owner class / effort / blocks 2.0.0-final?

### KD-01 — native dylib symbol table

- Plan: investigate an upstream CMake export fence —
  `-DDUCKDB_EXTRA_LINK_FLAGS` with a linker version-script fencing
  exports to the C ABI; check duckdb `CMakeLists.txt` and the
  `DUCKDB_API` visibility macros for an official filter, passed via
  the builder's cmake args. If no upstream knob exists, wrap at link
  time in `crates/frozen-duckdb-builder` (wrapper-ltcg step).
- Owner: UPSTREAM if a visibility/export flag exists; LOCAL
  wrapper-ltcg otherwise.
- Effort: M.
- Blocks release: No — it gates the C-ABI-only surface claim,
  not the engine build.

### KD-02 — community extensions (flock)

- Plan: no LOCAL action; wait for upstream to publish a flock
  build compatible with the dev pin, then un-ignore and rerun.
- Owner: UPSTREAM.
- Effort: S (idle; S of re-verify when the build lands).
- Blocks release: No — 2 tests `#[ignore]`ed loud; suite green
  without them.

### KD-05 — wasm artifact

- Plan: GAP1 status PENDING. Prior exit 2 was a broken brew
  emscripten. Install a pinned emsdk (not brew) under
  `~/.frozen-duckdb/emsdk`, rerun `scripts/build_wasm_duckdb.sh`
  to produce `duckdb.wasm`, run the version gate. Fallback:
  upstream v2.0.0 release assets when cut.
- Owner: UPSTREAM preferred (release assets); LOCAL emsdk build
  as fallback.
- Effort: M (LOCAL) / S (upstream).
- Blocks release: Yes for the FR-01 browser claim family; No for
  the engine binary itself.

### KD-06 — host-VFS (`host-range://`)

- Plan pointer: `docs/design/host-range-vfs-contract.md` is the
  contract; implement the host-call VFS shim per contract, then
  land the GATE-V-05 ≤5MB court green. Sequenced after KD-05
  (full court needs the wasm substrate).
- Owner: LOCAL.
- Effort: L.
- Blocks release: No — sovereign-surface claim only.

### KD-07 — REV-001 latency clause

- Plan: GAP8 idle-machine re-measure of the 10M-row compliance
  wall on an unloaded host; close if <2.5s. If still ≥2.5s,
  propose a spec amendment replacing the absolute <2.5s clause
  with a hardware-class-relative budget (per-core throughput
  ratio), keeping the clause hardware-honest.
- Owner: SPEC (amendment) after a LOCAL re-measure.
- Effort: S.
- Blocks release: No — already hardware-honest in the register.

### KD-08 — GATE-V-01 leak clause (allocator stats)

- Plan: file an upstream C API feature request for allocator
  stats export. Until then the D6 RSS-deceleration proxy remains
  the memory-leak witness; no LOCAL shim.
- Owner: UPSTREAM.
- Effort: S to file; L if we ever implement locally.
- Blocks release: No — UNSUPPORTED-BY-CURRENT-SURFACE, covered
  by the D6 proxy.

### KD-09 — Class B hardware court (cgroup)

- Plan: docker leg — `fleet002_cgroup_docker.json` absent on
  disk today; DOCKER1 in flight, no receipt. When it lands it
  witnesses the kernel memory limit on Linux; else wait for a
  Linux+root host to run `cgroup_low_ram_runner.sh`.
- Owner: OTHER-REPO / infra (docker leg) or LOCAL host.
- Effort: S once a leg lands.
- Blocks release: No — the darwin proxy already PASSes at its
  ceiling (fleet002_proxy.json: 85,557,248 B peak vs
  115,343,360 B ceiling, PARTIAL_ALIVE).

### KD-10 — Class E actor leg (BEAM)

- Plan: land the BEAM `:atomics`/`:ets` scheduler court in
  smalt (test_fleet_class_e_telco), then carry the cross-repo
  receipt here. No LOCAL code.
- Owner: OTHER-REPO (smalt).
- Effort: M (smalt-side).
- Blocks release: No — Class E coverage claim, engine-
  independent.

### KD-11 — browser substrate courts

- Plan: harness build gated on KD-05. Once `duckdb.wasm` passes
  the version gate, build the CDP harness per BP2 spec, land
  TEST-015..017 + GATE-V-02 CDP leg + GATE-V-04 against
  chrome-headless-shell.
- Owner: LOCAL (after KD-05).
- Effort: L.
- Blocks release: No directly; inherits KD-05's block on the
  browser claims.

### KD-12 — anomaly detection precision

- Plan: spec decision proposal — adopt a recall-only contract
  (recall 1.0000 witnessed in rev004.json) with the false-
  positive tail (148 noise-tail extras) recorded OBSERVED, not
  ALIVE. Add a QUALIFY threshold knob only as a follow-up if a
  precision bound is ever demanded. Do not tighten the threshold
  now — it risks recall, the property the court asserts.
- Owner: SPEC.
- Effort: S.
- Blocks release: No — reporting-claim precision, not engine
  behavior.

## Release statement

2.0.0-final is NOT blocked by any of the 11 open rows for the
engine gate. KD-05 is the only row that blocks a shipped CLAIM
family (FR-01/FR-02 browser + wasm, KD-11's courts, KD-06's full
court). If the release notes make any browser/wasm claim, KD-05
(then KD-11, KD-06) become release-blocking for those claims,
not for the binary.

See Also: /Users/sac/frozen-duckdb/KNOWN_DIVERGENCES.md;
docs/sjira/v2.0.0-dev/DEFINITION-OF-DONE.md
