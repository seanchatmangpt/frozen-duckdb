# X3 — KD-05 close receipt

Date: 2026-10-19. Lane X3. Subject artifact:
`/Users/sac/frozen-duckdb/target/wasm-min/kdc05-min.wasm`
sha256 `be3bad5bd05d3e1eee8e088fab32be6f214c3347860e63e07235764dcb713085`
(same artifact receipted by `KDC-05-wasm-min.md`).

## Verification criterion — executed output

1. Raw size (re-run this lane, not recalled):

```
$ wc -c < /Users/sac/frozen-duckdb/target/wasm-min/kdc05-min.wasm
25886446
```

25,886,446 ≤ 26,214,400 (25 MiB) → **PASS** (24.69 MiB).

2. gzip -9 wire size:

```
$ gzip -9 -c target/wasm-min/kdc05-min.wasm | wc -c
6294562
```

6,294,562 bytes = 6.00 MiB ≤ 9.5 MiB (9,963,264 bytes) → **PASS** with
3.5 MiB headroom.

3. Node instantiation + version gate re-witness (independent re-run on this
lane, real `WebAssembly.Instance` over 22 stubbed WASI/env imports — no
mock of the module itself):

```
$ EXPECT_VERSION=v2.0.0 node /tmp/x3-kd05-rewitness.mjs
[x3] exports: ["memory","duckdb_open","duckdb_close","duckdb_library_version","duckdb_free","_initialize","__indirect_function_table","malloc","free","setThrew","_emscripten_stack_restore","emscripten_stack_get_current"]
[x3] duckdb_library_version() = v2.0.0
[x3] duckdb_open rc= 0 db handle= 1893192n
[x3] heap before=67108864 after=67174400
[x3] PASS
exit=0
```

Version gate: returned string == receipt-pinned `v2.0.0` → PASS.
Open/close cycle rc=0 with non-null db handle; heap under 128 MB ceiling.

## Standing flip

KD-05 → **CLOSED (MINIMAL-SLICE)**. Evidence: raw 24.69 MiB ≤ 25 MiB;
wire 6.00 MiB ≤ 9.5 MiB; version gate witnessed a third time (KDC-05
original + this run). Per LSS9's boundary, the minimal slice IS the
shipped wasm.

Surface note (OPEN, not part of KD-05's close): the full-threaded build
(pthreads/OPFS browser configuration) vs SLA remains OPEN as a separate
surface; it does not gate this row.

No files outside `docs/sjira/v2.0.0/receipts/X3-kd05-close.md` and the
KD-05 register rows were touched. No git operations performed.
