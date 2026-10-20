# USER-FINDINGS-U11 — tutorial walk-through as WASM-curious user (KD-05/KD-11)

Date: 2026-10-19. Lane U11. Repo: /Users/sac/frozen-duckdb.
Subject: `docs/tutorials/quickstart-browser-opfs.md` (walked verbatim;
artifact claims re-tested on this tree), edits limited to that file.

## Real artifact probes (node, exit 0 both runs)

```
$ node -e '<WebAssembly.compile + new WebAssembly.Instance(mod, {}) per file>'
target/wasm/duckdb.wasm        imports: 72  modules: a            bare instantiate: FAIL — TypeError: ... Import #0 "a": module is not an object or function
target/wasm-min/kdc05-min.wasm imports: 22  modules: wasi_snapshot_preview1,env  bare instantiate: FAIL — TypeError: ... Import #0 "wasi_snapshot_preview1": module is not an object or function
```

```
$ ls -la target/wasm/duckdb.wasm target/wasm-min/kdc05-min.wasm
-rwxr-xr-x@ 1 sac  staff  41099303 Oct 19 15:55 target/wasm/duckdb.wasm
-rwxr-xr-x@ 1 sac  staff  25886446 Oct 19 18:25 target/wasm-min/kdc05-min.wasm
```

## TS compile probe (scratch project /tmp/u11-tsc, tsc 7.0.2, npm i typescript)

```
$ npx tsc --ignoreConfig --noEmit --strict --target es2022 --lib es2022,dom,webworker.importscripts wasm-worker.ts rpc.ts
rpc.ts(8,36): error TS2304: Cannot find name 'RowBatch'.
rpc.ts(37,37): error TS2304: Cannot find name 'RowBatch'.
wasm-worker.ts(7,42): error TS2552: Cannot find name 'RpcRequest'. Did you mean 'Request'?
wasm-worker.ts(18,33): error TS2339: Property 'createSyncAccessHandle' does not exist on type 'FileSystemFileHandle'.
wasm-worker.ts(51,23): error TS2304: Cannot find name 'RowBatch'.
wasm-worker.ts(54,40): error TS2304: Cannot find name 'FileSystemSyncAccessHandle'.
exit=1   (6 errors; the tutorial TS does NOT compile as printed)
```

```
$ npx tsc --ignoreConfig --noEmit --strict --target es2022 --lib es2022,webworker wasm-worker.ts rpc.ts ambient.d.ts
exit=0
```

Ambient declarations required (now embedded in the tutorial):
`RpcRequest`/`RowBatch` types (referenced but never declared), `StorageManager.persist`,
`FileSystemFileHandle.createSyncAccessHandle`; lib must be `es2022,webworker`
(dom lib hides `FileSystemSyncAccessHandle`; `webworker` lib's `StorageManager` lacks `persist`).
Full ambient.d.ts (12 lines) and exact command are in the tutorial's new
"Compiling the contract" section — re-ran it verbatim, exit 0.

## OPFS/claims findings

- WORKS-TODAY: nothing in a browser. Only the tsc type-check of the RPC
  contract (with the ambient decls). Both wasm artifacts fail bare
  instantiation; the 22-import shim is required (KDC-05 receipt) and is not
  published or checked in — it lives in the KDC-05 receipt transcript.
- REQUIRES-KD-05-CLOSURE: worker instantiation, `execute`/`fetch`, OPFS
  `createSyncAccessHandle` block layer, WAL-replay falsifier, quota
  falsifier, COOP/COEP+SAB for any pthreads build (not yet built).
- Size reality vs the old page: the page previously stated no sizes at all
  and said the artifact "does not exist". Reality: 39.2 MiB non-standalone
  (`target/wasm/duckdb.wasm`, 72 imports) and 24.69 MiB standalone slice
  (`target/wasm-min/kdc05-min.wasm`, 22 imports, under the 25 MiB SLA).

## Edits landed (top 2 falsehoods, per lane scope)

1. Header standing block: BLOCKED/"artifact does not exist" → PARTIAL_ALIVE
   with measured sizes, import counts, version gate, receipt pointer.
2. New sections "What exists today vs what is contract" (real probe output,
   22-import shim requirement, TS-does-not-compile-as-printed) and
   "Compiling the contract" (ambient.d.ts, exact tsc command, exit 0,
   [WORKS-TODAY] limited to type-check). Per-section standing labels added
   to all headers; KD-05 status row flipped to PARTIAL_ALIVE with receipt
   path. No other files touched.

## Standing

Tutorial: PARTIAL_ALIVE (type-check of contract WORKS-TODAY; browser runtime
REQUIRES-KD-05-CLOSURE, gated on KD-11 glue + published shim). This receipt
changes no ticket standing frontmatter; no standing flip to gate.
