# GATE-V-05: Host-VFS Remote Parquet Transfer Budget Court

Status: UNVERIFIED — depends on host-call VFS + wasm build
(`docs/design/host-call-vfs.md`). No court runs until both exist.
Every mechanism below is design until a real receipt lands.

See Also: `docs/design/host-call-vfs.md`,
`docs/design/acceptance-criteria-courts.md`,
`docs/design/verification-plan.md`

## Gate claim

Filtering a remote 500 MB Parquet file through the host range-request
VFS transfers at most 5 MB across the wasm boundary: the Parquet
metadata footer plus the row groups the filter actually selects.
Everything else (row-group index skips, predicate pushdown) manifests
as requests never made — bytes never copied into linear memory.

Pass: `total_bytes_transferred <= 5_000_000`.
The number reported is the exact host-side counter, not an estimate.

## Transfer accounting mechanism

The host wraps every `wasm_host_fetch_range` invocation with a
byte counter. Accounting law: **every byte copied into linear memory
is counted** — the count is `sum(length_copied)` over all successful
range calls in one query session, incremented inside the host
implementation after the copy completes (not at request-issue time, so
failed/partial requests do not inflate or deflate the receipt).

```
HostTransferCounter (per query session):
  total_bytes: u64          # copied-into-linear-memory only
  request_count: u64        # number of successful range calls
  per_call: [(offset, len)] # full log, receipt field
```

The court receipt reports `total_bytes` (exact integer),
`request_count`, and the `per_call` log. The log is the audit trail:
a passing budget with a suspicious log (e.g. one 400 MB call) is a
FAIL by inspection, not a PASS.

Native-host variant: the same counter wraps the native
`HostCallVFS::Read` path, so the gate has a native-runnable leg before
wasm exists — it proves the accounting and the row-group pruning, not
the boundary marshal. Standing under that leg: PARTIAL_ALIVE, never
ALIVE (ALIVE requires the wasm boundary on the exact subject).

## Test fixture

- **Generator**: `scripts/fixtures/gen_remote_parquet.py` (new).
  Deterministic seed (fixed constant, documented in the script);
  generates a 500 MB (± file-format overhead) Parquet file:
  500 columns-free simple schema (`id BIGINT, val DOUBLE,
  label VARCHAR`), ~200 row groups of ~2.5 MB each, `label` values
  drawn so that exactly 8 row groups match the court filter.
  Cloud-free: written to a local directory, no network egress.
- **Server**: local HTTP server with `Range: bytes=...` support
  serving the fixture (`python3 -m range_server` style harness in
  `scripts/fixtures/`; must answer `206 Partial Content` and expose
  `Accept-Ranges: bytes`). Reproducible: same seed, same file,
  same sha256 recorded in the receipt.
- **Query shape**:
  `SELECT count(*), sum(val) FROM read_parquet('<url>') WHERE label = '<court-label>'`
  — selective enough that DuckDB's parquet reader, using the footer
  + row-group statistics, touches only the ~8 matching row groups
  (~20 MB raw) — but the budget is 5 MB, so the gate additionally
  requires the reader to prune via statistics without reading full
  matched-row-group payload twice and without reading skipped groups.
  Fixture tuning (group size, match count) is the calibration knob;
  the generator documents the expected footer + matched-group byte
  math so 5 MB is achievable, not aspirational.

## Court shape (shim-court protocol)

Self-contained script `scripts/courts/gate-v05-transfer.sh`,
BASELINE → BEFORE → SHIM → AFTER → RESTORE:

1. **BASELINE** — `sha256sum` the fixture parquet + the VFS counter
   source file; record both.
2. **BEFORE** — run the transfer gate against the honest VFS counter;
   expect exit 0 with `total_bytes <= 5_000_000` in the receipt.
3. **SHIM** — apply the naive full-download shim (below), marked
   `SHIM-COURT`.
4. **AFTER** — re-run; expect NONZERO exit (budget blown). The AFTER
   flip is the non-vacuity proof: a court that cannot flip is FAIL,
   not PASS.
5. **RESTORE** — revert shim; `sha256sum` must equal BASELINE
   byte-exact or the court reports the subject CORRUPTED and exits
   nonzero (fail-closed).

## Falsifiers

**(a) Naive full-download shim (non-vacuity).** Shim the host range
callback to ignore `offset`/`length` and fetch the whole file once
(marked `SHIM-COURT`). The gate must FAIL with the counter reporting
~500 MB (`> 5_000_000`). If the gate still passes, the accounting is
vacuous — the gate is measuring nothing — and the court exits 1 as
`VACUOUS GATE`.

**(b) Missing Range support → typed error, not silent full fetch.**
Shim the local server to answer `200 OK` (full body) instead of
`206 Partial Content` to range requests (marked `SHIM-COURT`). The
VFS must return a typed error
(`RANGE_NOT_SUPPORTED` / negative code across the boundary per
`host-call-vfs.md`), the query refuses, and the counter stays at its
pre-error value. A silent fallback that downloads 500 MB and proceeds
is a FAIL: the contract is refuse-loudly, not degrade-quietly.

## Standing ladder

- UNVERIFIED (now): no host-call VFS, no wasm artifact.
- PARTIAL_ALIVE: native leg green (counter + pruning proven, native
  VFS path, local Range server) + falsifier (a) flips.
- ALIVE: wasm boundary leg green on the exact subject — receipt binds
  `total_bytes`, fixture sha256, engine build id, and both falsifier
  flips. No ALIVE claim without the wasm receipt.

## Loud skips

Missing toolchain, missing fixture generator output, or a local
server that cannot demonstrate `206` on a probe request ⇒ the court
emits `[SKIP] <reason>` and exits nonzero-as-registered-divergence.
A SKIP is never recorded as a green cell.

## Receipt

`target/courts/gate-v05/receipt.json`: fixture sha256, server probe
result, `total_bytes`, `request_count`, `per_call` log, falsifier
(a) blown-bytes figure, falsifier (b) typed-error code, BASELINE/
RESTORE digests, verdict. Bound to git HEAD via `git rev-parse`
(read-only; courts never mutate git state).
