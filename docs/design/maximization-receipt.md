# Maximization-Spec Closure Receipt (MAX wave)

Date: 2026-10-19. Lane MAX10 (consolidate). Window:
`docs/sjira/v2.0.0/`. Every claim below cites a receipt file on disk
under `docs/sjira/v2.0.0/receipts/` or a command run in this lane with
its real output. Lanes without a receipt file (MAX2, MAX5, SP5, REL4r,
REL6r) are listed under In-Flight, never cited as landed.

## 1. Four-Boundary Table

| boundary | nearest landed court | receipt path | standing + evidence |
|---|---|---|---|
| Zero-Poison | CacheAdmission typed-verdict poison exit court — 4 scenarios, each asserting process exit 1 + typed stderr (silent zero-exit on poison = court failure) | `docs/sjira/v2.0.0/receipts/MAX1-poison-exit.md` | PASS: 46 lib + 1 court (4 scenarios) + 12 profile builder tests green |
| Zero-Poison (stale-state leg) | STALE_PRECONDITION_REFUSED two-phase guard (analysis-time hash vs execution-time re-observed hash; mismatch aborts pre-execution) | `crates/frozen-duckdb/tests/precondition_guard.rs` (MAX5 receipt pending, in-flight) | 5 passed / 0 failed this lane, command in §2 |
| Zero-Alloc | Memory-ceiling probe family (memory_limit=256MB, probe toward ~2GB nominal, typed OutOfMemory never crash); spec `docs/design/gate-v03-memory-ceiling.md` grounded in AC-3 / PRD-FR-05 | `docs/sjira/v2.0.0/receipts/COURT-INDEX.md` rows `rev004.json` (OBSERVED, memory_limit=256MB), `fleet002_proxy.json` (latency MET, RSS WITHIN-BUDGET, proxy ceiling), `gate_v01_receipt.json` (p95 10.5088ms < 15ms) | PARTIAL_ALIVE — native legs receipted; wasm 256MB linear-memory leg is a separate future leg per the gate-v03 spec |
| Zero-Leak | Package leak scan: per-crate ship-list + content sweep for `/Users/sac`, `/tmp/`, `target/`, `.env`, `secret`, lane fixture paths; all 3 crates clean, largest `.crate` 81 KB (limit 10 MB) | `docs/sjira/v2.0.0/receipts/PB6-leakscan.md` | PASS (leak scan clean on all 3 crates) |
| Zero-Drift | Generated-surface gate: `make gates` (ggen sync + genesis-check) + SP2 ABI-receipt baseline identity | run in this lane + `docs/sjira/v2.0.0/receipts/SP2-abi-receipt.md` | `genesis-check: checked=58 deferred=0 gaps=16 failed=0` / `GATES GREEN: sync exit 0 + genesis-check exit 0` (exit 0, this lane); SP2 subject sha256 identical to BASELINE |

## 2. Net-New Implementations (this wave)

All six suites re-run by this lane (MAX10) on
`CARGO_TARGET_DIR=target-lane10`, one command:

```
cargo test -p frozen-duckdb --features serde_json,vscalar,vtab \
  --test pot_ingestion --test monotonic_ingress --test variant_shredded_bind \
  --test query_receipt --test precondition_guard --test dfg_cut_detection
```

Aggregate real output: 14 passed, 0 failed across the six suites. Per-suite:

| implementation | lane | court file | tests (this lane re-run) | receipt |
|---|---|---|---|---|
| POT ingestion (1M causal edges + incomparability pairs, bulk appender, direct vector binds, DFG support aggregate) | MAX2 | `crates/frozen-duckdb/tests/pot_ingestion.rs` | 1 passed (15.50s) | no receipt file yet (in-flight). X1r differential witness: failed throughput floor on BOTH fenced and original dylib (82,120 vs 81,731 rows/s vs 100k) — pre-existing, not fence-caused (`X1r-kd01-fence.md`); the floor assertion passed in this lane's run |
| Monotonic ingress gate (CHECK(prev_ts) refuses out-of-order frames at the DB boundary; non-vacuity by shim flip — gate removed -> refusal test flips to failure) | MAX8 | `crates/frozen-duckdb/tests/monotonic_ingress.rs` | 2 passed (0.04s) | `MAX8-monotonic-gate.md` — PASS, gate proven load-bearing |
| Shredded VARIANT bind (JSON via `variant_json_udf::register_json_extract` UDF, read back through shredded binds, never opaque strings; 100k heterogeneous payloads at 80,244 rows/s) | MAX3 | `crates/frozen-duckdb/tests/variant_shredded_bind.rs` + `crates/frozen-duckdb/src/duckdb/variant_json_udf.rs` | 4 passed (1.99s) | `MAX3-shredded-variant.md` — 4 passed; requires `--features serde_json,vscalar` (receipt's own correction: plain invocation does NOT compile) |
| Per-query detached receipts (BLAKE3 over `sql_text \|\| input_hash \|\| output_row_count \|\| output_first_vector_bytes`, JSON line per query, session manifest, tamper court: edited receipt -> verify FAILS, re-mint -> green; b3sum CLI) | MAX4 | `crates/frozen-duckdb/tests/query_receipt.rs` | 1 passed (0.28s) | no dedicated receipt file yet; independently witnessed by `X1r-kd01-fence.md` (query_receipt 1/1 on release) |
| Precondition guard (`STALE_PRECONDITION_REFUSED: expected <hash4>, got <hash4>`; inline SHA-256, no new dependency) | MAX5 | `crates/frozen-duckdb/tests/precondition_guard.rs` + `crates/frozen-duckdb/src/duckdb/precondition.rs` | 5 passed (0.09s) | no receipt file yet (in-flight) |
| DFG cut court (in-SQL cut detection vs exact test-side oracle, 1,000,000 edges, 4.968 ms median of 5) | MAX9 | `crates/frozen-duckdb/tests/dfg_cut_detection.rs` | 1 passed (0.41s) | `MAX9-dfg-cut.md` — PASS, oracle-exact |

Supporting net-new builder surface: typed profile matrix
(`crates/frozen-duckdb-builder/src/profile.rs`, 12 tests; `SP1-profile-matrix.md`,
59/59 builder suite green) and the poison-exit court
(`crates/frozen-duckdb-builder/tests/poison_state_exit.rs`; MAX1).

## 3. KD-05 / KD-06 Movement This Wave

**KD-05 (wasm artifact)** — moved from BLOCKED-on-slice to a promoted
distribution candidate, with the full-spec path honestly closed at this pin:

- `KDC-05-wasm-min.md`: minimal standalone module 25,886,446 B = 24.69 MiB,
  link exit 0 (em++; emcc attempt failed on C++ exception symbols — recorded).
- `X3-kd05-close.md`: raw 25,886,446 <= 26,214,400 PASS; gzip -9 wire
  6,294,562 B = 6.00 MiB <= 9.5 MiB PASS (3.5 MiB headroom); node
  instantiation re-witness, `v2.0.0`.
- `MAX6-wasm-spec-flags.md`: operator exact flag set, full extensions
  (json;parquet) — raw 29,242,868 B = **27.89 MiB** vs 25 MiB gate: FAIL
  (+7.7%). Standing BLOCKED (size SLA FAIL), build/link/smoke PASS.
- `CL4b-icu-free.md`: icu lever dead — icu-free build does not drop the size
  floor (icu was never linked). Closes the CL4-identified lever.
- `MAX7-converge.md`: `wasm-opt -O3 --converge` exit 0 (509s user), 25,731,939 B
  = **24.54 MiB** (-0.60%), node load + open/close smoke PASS on the converged
  module (CL4's caveat does NOT reproduce on the minimal build). Coordinator
  promotion: `/tmp/max7/post.wasm` -> `target/wasm-min/kdc05-min.wasm`,
  post-copy sha256 `1fdeda0d…4bc1c7` byte-identical.

Net: KD-05 has a PASSING, promoted, sub-25MiB minimal artifact; the
full-spec (json+parquet) build cannot meet 25 MiB at this pin (MAX6 + CL4b
jointly close the flag/extension lever space).

**KD-06 (host-VFS `host-range://`)** — first local slice landed:

- `CL13-hostvfs.md`: `read_local_range(file, offset, length)` table function
  over real `std::fs` pread; typed `HostRangeStatus::{InvalidInput, OutOfRange}`
  closed set; 6/6 integration tests
  (`cargo test -p frozen-duckdb --features vtab --test host_vfs_local` ->
  exit 0, `6 passed; 0 failed`). Also surfaced and declared the previously
  UNDECLARED `vtab` feature gate that kept the whole vtab surface dead code.
- `SP4-icu-sidecar.md`: 1.28 MiB icu side module built (exit 0); smoke exit 2
  typed BLOCKED — dynamic-load seam needs `-sMAIN_MODULE=2` core (rung 3 open).

Related fence work (KD-01, adjacent): `X1r-kd01-fence.md` — relink with
`-Wl,-exported_symbols_list` gives exactly 1,177 `_duckdb_*` C-ABI exports /
0 mangled; swap into cache complete (sha `aab11996…`); non-vacuity court:
SHIM (original 27,682-mangled dylib) -> `bin_audit.sh` AFTER exit 1
`[FAIL] 27682 C++-mangled exports present` -> RESTORE byte-exact.
KD-01 CLOSED-LOCAL; upstream fence remains the close condition.

## 4. Honest OPEN Boundary

1. **Network host-callback delegation** — KD-06's landed slice is local
   filesystem only (`CL13-hostvfs.md`). The `host-range://` network leg and
   the GATE-V-05 <=5MB court (`KD-CLOSURE-PLAN.md` §KD-06) are not exercised;
   the browser/full court remains open.
2. **Browser OPFS courts** — `KDC-11-opfs.md` (KDC11r) is ALIVE on
   L1/L3/L4 via the first CDP chrome-headless-shell court (run17 full PASS
   exit 0) over `target/wasm/duckdb.wasm` (41,099,303 B), but the L2
   programmatic OPFS surface is typed MISSING. The wasm 256MB linear-memory
   ceiling leg of Zero-Alloc rides this same open wasm surface
   (`gate-v03-memory-ceiling.md`).
3. **25 MiB full-build SLA** — closed honestly as UNREACHABLE at this pin
   for the full-spec build: MAX6 27.89 MiB FAIL (+7.7%), CL4b lever dead;
   only the minimal slice meets it (24.54 MiB promoted, 24.69 MiB
   receipt-closed with 6.00 MiB wire). Full browser spec claims stay blocked
   behind this (KD-05 family per `KD-CLOSURE-PLAN.md`).
4. **Fence-restage hazard (KD-01 work order)** — the fence is a local relink;
   the builder's local-compile fallback does NOT apply `exports.list`
   (`X1r-kd01-fence.md` standing paragraph). A poison-refusal cache restage
   (the MAX1 path, which re-mints the dylib) silently reverts the dylib to
   27,682 mangled exports; the gate returns red by design but the fence must
   be re-minted manually per cache rebuild. Open work order: wire
   `exports.list` into the builder fallback.
5. **In-flight lanes without receipt files** (not cited as landed): MAX2
   (POT ingestion receipt), MAX5 (precondition guard receipt), SP5, REL4r,
   REL6r. Per `X8r-consolidation.md` falsifier: these flip to landed only by
   writing receipts with real embedded command output.

## Replay

```
cargo test -p frozen-duckdb --features serde_json,vscalar,vtab \
  --test pot_ingestion --test monotonic_ingress --test variant_shredded_bind \
  --test query_receipt --test precondition_guard --test dfg_cut_detection
make gates
```
