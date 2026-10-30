# X1r Receipt — KD-01 relink export fence finalized

Date: 2026-10-19 · Lane: X1r (salvage of interrupted lane X1) · Subject: cached dylib `~/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib`

## State found

The previous lane's scratch dir `/private/tmp/x1` held the completed
relink-fence work: a fresh static build of upstream DuckDB
(`d54cbbe3df17e8e5ce68312f483b4155836112aa`) relinked with
`-Wl,-exported_symbols_list` (`/private/tmp/x1/relink.sh`, `exports.list`),
producing `libduckdb_arm64_fenced4.dylib` (sha256 `aab11996…93d2a5`). The
swap into the cache had been interrupted — the cache still held the ORIGINAL
dylib with the 27,682 mangled `__Z*` exports (sha256 `670acb8a…e2498`).

## What the fence is

A link-time relink of `libduckdb_static.a` with an exported-symbols allowlist:
the dylib's exports trie carries exactly the 1,177 `_duckdb_*` C-ABI symbols
and zero C++-mangled exports. Darwin post-hoc tools cannot do this
(`strip -x` leaves the trie; no objcopy on darwin) — hence relink, per
KD-01/ADV3.

## Probes re-run by X1r (real output)

- Census (fenced4, before swap): `nm -gU` → 1,177 exports, `0` matching
  ` T __Z`; surface diff vs original's `_duckdb_*` set → identical
  (`FENCED-SURFACE-EXACTLY-CABI` — no C-ABI symbol lost).
- Functional probe (`probe2.c`: open/connect/query sum(range(10))/version),
  freshly compiled against fenced4: exit 0,
  `sum(1..9) = 45`, `version = v2.0.0`.
- **Swap-in completed**: cache dylib sha256
  `aab119965ac23ef60b928848fbbcd17565a42d94de5b45bbe8201e521e93d2a5`
  (= fenced4). Pre-swap original preserved at
  `/private/tmp/x1/x1r-pre-swap-backup.dylib` (sha256 `670acb8a…`).
- `cargo test -p frozen-duckdb --release`: 172 lib tests → 171 passed,
  1 failed (`test_vtab_arrow`: missing fixture `examples/int32_decimal.parquet`
  — file absent from tree, **pre-existing**, unrelated to symbol surface).
  Integration tests: query_receipt 1/1, precondition_guard 5/5,
  monotonic_ingress 2/2, dfg_cut_detection 1/1, test_rev_process_mining 1/1
  (1 ignored). `pot_ingestion` FAILED on throughput floor
  (82,120 rows/sec vs 100k) — **differential witness**: same test on the
  ORIGINAL dylib failed identically (81,731 rows/sec), so pre-existing,
  not fence-caused.
- `scripts/doctor/checks/bin_audit.sh` (TEST-002): **PASS, exit 0** —
  stripped size 46,737,232 ≤ 55.0 MiB ceiling; mangled exports 0;
  C-ABI 1,177. Required a one-line fix first: its VERSION-derivation sed
  only matched `const VERSION` but lib.rs now has `pub const VERSION`;
  sed extended to accept both in
  `scripts/doctor/checks/bin_audit.sh` and `scripts/doctor/export_fence.sh`
  (the latter previously exited 3 [MISSING]).
- `scripts/doctor/export_fence.sh`: exit 0, census 1,177 T exports /
  0 mangled / 1,177 C-ABI.
- **Non-vacuity court (bin_audit)**: BASELINE sha `aab11996…` → BEFORE
  exit 0 → SHIM (original 27,682-mangled dylib into cache slot) → AFTER
  exit 1 with `[FAIL] 27682 C++-mangled exports present` → RESTORE sha
  `aab11996…` byte-exact (`RESTORE-EXACT`). The gate can fail.

## Standing

KD-01: CLOSED-LOCAL on this subject (arm64, v2.0.0). The fence is a
local relink, not an upstream fix; upstream fence remains the close
condition in `KNOWN_DIVERGENCES.md` — re-verify whenever the cache is
rebuilt (the builder's local-compile fallback does NOT yet apply
`exports.list`; the fenced dylib must be re-minted after any cache
rebuild, or the gate returns red by design).

## Files

- Fence subject: `/private/tmp/x1/libduckdb_arm64_fenced4.dylib` (in cache)
- Pre-swap backup: `/private/tmp/x1/x1r-pre-swap-backup.dylib`
- Relink recipe: `/private/tmp/x1/relink.sh` + `/private/tmp/x1/exports.list`
- Gate fix: `scripts/doctor/checks/bin_audit.sh` (VERSION regex, `pub const`)
- Evidence tool fix: `scripts/doctor/export_fence.sh` (same regex)
