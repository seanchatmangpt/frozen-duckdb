# VALIDATED-STATE-THESIS

Status: governing validated-state thesis for the frozen-duckdb v2.0.0-dev campaign, as transcribed from the operator's 2026-10-20 statement; receipt-grounded spot-checks by the coordinator, 2026-10-20.

# Thesis: The Validated and Working State of frozen-duckdb v2.0.0-dev

*Every claim below carries its receipt class. The organizing law: nothing is "done" unless a court ran it, passed it, and a file proves it.*

## Part I — The Engine Itself

### 1.1 Engine integration: ALIVE

The bundled engine is DuckDB 2.0.0-dev pinned at upstream `b6013f7aa2f36caabb9466ce1791667ea8a280bb` (v2.0-cyanoptera), self-reporting `v2.0.0-dev88595` from the linked dylib. Version integrity verified by test-validation Gate 6 (dev-suffix-tolerant prefix match); cache *content* — not just the label — bound by the `ENGINE_VERSION` marker staged atomically at every persist site plus a size-pin sidecar with a 10 MiB floor (KD-14). Re-witnessed this session: `bin_audit` PASS on the fenced cache, and the monotonic/DFG courts ran green on the fenced engine post-restage incident.

### 1.2 The release gate battery: 5/5 GREEN

The full `make verify` ladder — build, clippy (`-D warnings`), fmt, test-validation, workspace tests — exited 0 on the settled machine, genesis-check at 58 checked / 0 failed, ggen double-sync byte-identical. This is the campaign's summary claim, and the most re-run one. Currency note (honest): the last full-ladder re-receipt predates the MAX/SP waves; since then the net-new courts ran green individually and via MAX10's single-command 14/14 sweep (`--features serde_json,vscalar,vtab` — the working feature set, recorded in the receipt), builder suite 59/59 (coordinator re-run on the shared target), and `make gates` GREEN (58 checked). A settled `make verify` re-receipt is queued as the closing receipt.

### 1.3 The API surface: 141+ tests, adversarially hardened

The main crate carries a 141-test suite plus integration courts. The 2.0-specific surface — VARIANT type (bind, read, `::VARCHAR` cast, serde_json feature, shredded raw-read projection with typed actionable error) — hardened through three review rounds, with null-element guards landing under the panic-hunter pass (7 panics fixed; root cause class `CString::new().unwrap()` on external input).

### 1.4 Session-additive surface (MAX wave)

New this session, all court-tested: shredded VARIANT bind path (100k payloads @ 80,244 rows/s, byte-fidelity zero-mismatch, NULL guards — MAX3); `variant_json_udf.rs` (`json_extract` UDF pair + declared `vscalar`/`vtab-arrow` cfg gates, closing two KD-06-class undeclared-gate defects); monotonic ingress CHECK gate with SHIM non-vacuity (MAX8); DFG in-SQL cut detection, 1M edges @ 4.968 ms, oracle-exact (MAX9); per-query BLAKE3 receipts via the real `b3sum` CLI with tamper-red court (MAX4); `STALE_PRECONDITION_REFUSED` two-phase guard, 5-test court (MAX5); POT ingestion with typed pre-buffer rejection, 61,599 rows/s (MAX2); builder poison-exit contract (MAX1); Target-Profile Matrix (SP1, typed `ProfileError`, no silent Core fallback); `abi_receipt` WASM co-pinning (SP2); 9-shim corruption court (SP3).

## Part II — The Cache Fortress (Most-Sieged Subsystem)

### 2.1 Anti-poison: 9/9 states refused

Nine poison cache states constructed and refused: truncated dylib with valid marker, foreign marker, every one now typed-refused with `POISON_STATE_REFUSED: <path> (<invariant>)` + exit 1 at both cache-hit paths (MAX1's typed exit contract), plus byte-exact marker law, size-pin demotion with serve-on-demote eliminated (a hole the siege found), Mach-O magic check, atomic temp+rename persistence, 10 MiB floor. 46 builder tests + the AD1/AD6 matrix + the 4-scenario self-reexec poison court stand behind this.

### 2.2 Concurrency: BuildLock proven

The `BuildLock` (create-new + PID liveness + steal for dead holders) closed a witnessed double-compile race; acquisition verified by bounded poll on a trusted-entry observable — never by sleep.

### 2.3 macOS linking: closed and self-healing

The absent-`LC_RPATH` defect is closed via `install_name_tool -id` stamping + ad-hoc codesign at cache persist, with self-heal. AD3r's break-attempt scrubbed the id and the self-heal restored it byte-exact (sha-identical baseline) — re-witnessed this session.

KD-01 fence: the live-cache dylib is the fenced relink (1,177 C-ABI exports, 0 mangled); `bin_audit` non-vacuity court proves the gate flips red on the unfenced dylib. Fence-restage hazard filed as the open work order (builder local-compile doesn't apply `exports.list`).

## Part III — The Court Battery (Physical, Real, Non-Vacuous)

### 3.1 Performance courts

- **Latency (GATE-V-01)**: p95 10.51–13.36 ms median-of-3 vs <15 ms budget, release profile. The court *genuinely failed* under fan-out contention (up to 73.9 ms) and passed settled — the flake is a witnessed property of the methodology's own load, recorded rather than suppressed.
- **Throughput**: 12.16 M rows/s packed ingest (release) vs ≥100k budget — ~120× headroom. OCEL-segment ingest 7.09 M rows/s.
- **Compression**: OCEL 94.93% (100k), 97.01% (1M), 99.76% (marine fleet corpus) — all vs ≥90–95% budgets.
- **DFG query**: 4.968 ms median over 1M edges (MAX9), oracle-exact vs a brute-force Rust group-by.

### 3.2 Confinement and durability

- **OOM**: typed graceful abort under 256 MB `memory_limit`, host alive, post-OOM queries working; 4 GB completion leg green.
- **Leak audit**: 20k-iteration scan, final 5k-window +0 bytes.
- **Crash-chaos**: real `kill -9` mid-transaction, WAL replay to a legal post-state, no-op-kill control DETECTED (non-vacuity). One false positive (torn-txn) withdrawn when the fixture was shown to link a cross-engine dylib — withdrawal receipted.
- **Kernel cgroup**: 128 MB hard cgroup, RSS held, exit-137 negative control, virtualized-kernel caveat stated (KD-09 CLOSED).

### 3.3 Non-vacuity as a discipline

Every load-bearing gate carries a SHIM-COURT. The discipline *caught real bugs*: a vacuous audit regex (21 vs the true 27,682 mangled exports), a fail-open slice verifier, a render-time signature gap. MAX8/SP2/SP3 carried the pattern into the wasm/ingress surface this session, all flipped.

## Part IV — Supply Chain and Provenance

- **Pinned headers**: vendored `duckdb.h`/`duckdb.hpp` byte-compared against the upstream pin (TEST-001).
- **Publish receipts**: signed, validated by a dedicated script, GREEN.
- **Value-audit chain**: mint-then-verify with BLAKE3 digests; tamper court red, restore green; extended by MAX4's per-query receipts (length-prefixed frame digest, manifest refuses unmanifested extras).
- **Registry-shaped cleanroom**: stitched directory registry with negative control proving the prerelease fence; packaged bytes proven to compile via replica builds.
- **License compliance**: MPL premise inverted by observation — upstream is MIT at this pin; files corrected.
- **Dependency audit**: RUSTSEC critical trio eliminated (zip 9, MSRV 1.88 evidence-forced); residuals documented in `deny.toml`, lockfiles committed.

## Part V — WASM and Edge

- **Minimal slice ALIVE**: 24.69 MiB → **converged 24.54 MiB** (binaryen `--converge` with the required feature flags — bulk-memory/nontrapping-float-to-int/sign-ext/mutable-globals; plain invocation exits 1, a witnessed toolchain fact), promoted `target/wasm-min/kdc05-min.wasm`, sha `1fdeda0d…4bc1c7`, promotion byte-identical, loads in node V8, full smoke PASS.
- **Honest floor**: full/threaded builds cannot meet 25 MiB raw (27.61–39.2 MiB across three independent builds: CL4/CL4b/MAX6); the icu-removal lever proven dead (+12 bytes). Wire size passes at 6.67–9.46 MiB gzip.
- **ABI co-pinning (SP2)**: `abi_receipt` custom section carries pin + module digest (over the stripped canonical module — any code mutation flips it); verifier fails closed typed `ABI_MISMATCH` on digest-corrupt / section-strip / wrong-pin — all flipped, restore byte-exact; stamp idempotent.
- **Corruption court (SP3)**: 9 shims (truncation ×3, code-flip ×3, export-corruption ×3) all produce catchable typed `CompileError`, process healthy after every catch.
- **Sidecar topology (SP4/SP5)**: ICU sidecar built (1.28 MiB, pin-stamped); the dynamic-load seam is now mechanically mapped — `MAIN_MODULE=1` required (mode 2 leaves GOT.func imports unreachable), shared-memory relink required for side modules, main-module core runs the full C API query cycle, but `LOAD` registration hangs in `LoadExternalExtension` at this pin — typed BLOCKED, receipted with replay commands.
- **Profile matrix (SP1)**: `Core/Threaded/AllExtensions` with 24/32/42 MiB ceilings and typed `ProfileError` — the Threaded/Enterprise tiers' artifacts honestly don't exist within budget (UNSUPPORTED typed, not hidden).

## Part VI — Distribution Mechanics (Witnessed End-to-End)

- **The release exists**: tag `v2.0.0-dev.3` (at `a6b5704`) → all four build legs green (the mac x86_64 leg after a wrong-arch OpenSSL link failure was root-caused and fixed with a from-source keg) → **GitHub Release published with 4 real binaries** (66–83 MB each).
- **The download path proven in a real container**: fresh `rust:trixie`, empty cache → `ensure_binary()` fetched from the live release → ELF magic, size pin, `ENGINE_VERSION` → dynamic link → `DUCKDB_VERSION=v2.0.0-dev88595`, `SELECT 42` = 42 — with the load-bearing claim **sha256(served) == sha256(asset)** (`0f330035…`): the executing binary *is* the GitHub download.
- **The version guard hardened**: rejects unrelated versions, accepts dev-suffix tags; its non-vacuity witnessed by two real rejection events (v-prefix dispatch arg; dev.2 tag) before the fix.
- **Typed residuals**: linux `.so` requires glibc ≥ 2.38 (new **KD-16**); `LD_LIBRARY_PATH` rpath contract on linux; bindgen needs `libclang-dev` in cleanroom images.
- **crates.io**: metadata, dry-runs, publish chain scripted and DRYRUN-verified — held at the DOD-18 operator gate (a `2.0.0-dev` prerelease is permanently unpublishable), awaiting token and the upstream final tag.

## Part VII — Process Validation

- **KD register**: 16 rows — **10 CLOSED** (incl. KD-01 CLOSED-LOCAL with the fence + non-vacuity court; AD3r's adversarial break-attempts reopened none) / 2 PARTIAL_ALIVE / open remainder enumerated (upstream issue duckdb/duckdb#27511-class rows, watcher, parked, spec-amended, KD-16 typed).
- **Lean Six Sigma self-audit**: Pareto 21.4% concurrent-writer class; Gage R&R within-session capable; 5S reclaimed 30+ GB; five poka-yoke installed, one witnessed live-firing.
- **User-persona wave**: 12 personas found real gaps, fixed at the Pareto top, receipted otherwise.
- **Cross-lane discipline**: compile-freeze SLA disclosures, coordinator-owned git, `commit -F` law, lane-lease cleanup — enforced, violations caught and corrected in-session.

## Conclusion: What "Working" Means Here

The validated state is not "the tests pass." It is: **the engine runs (gates green, most recently re-witnessed on the fenced engine), the cache cannot be poisoned (9/9 refused, now with a typed exit contract), the courts can fail (every load-bearing gate flipped at least once), the provenance chain binds numbers to commits, a real wasm artifact fits the edge budget and survives corruption with co-pinned ABI provenance, and the distribution path — tag → guarded build → publish → container download → sha-verified execution → SELECT 42 — has been witnessed end-to-end against the live release.**

The residual is stated, not hidden: wasm full-build size (KD-05 partial), host-VFS delegation (KD-06, parked), OPFS browser courts (KD-11 partial), upstream-watch rows, **KD-16 glibc floor on linux consumers**, and crates.io held at its operator gate. Under the fail-closed contract, that enumeration *is* the completion criterion — the register says what remains until receipts say otherwise.
