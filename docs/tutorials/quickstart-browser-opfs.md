# Quickstart: frozen-duckdb in the browser (Web Worker + OPFS)

> **Standing: PARTIAL_ALIVE (KD-05), as of 2026-10-19.** This tutorial
> describes the FR-01 target. The wasm artifact **exists** in two forms:
> `target/wasm/duckdb.wasm` (41,099,303 B / 39.2 MiB, 72 imports, **not
> standalone** — fails `WebAssembly.Instance(mod, {})`) and
> `target/wasm-min/kdc05-min.wasm` (25,886,446 B / 24.69 MiB, standalone,
> instantiable with a 22-symbol WASI/env shim; `duckdb_library_version()` =
> `v2.0.0` under real node instantiation — receipt
> `docs/sjira/v2.0.0/receipts/KDC-05-wasm-min.md`). Nothing here runs in
> a browser yet: the OPFS block layer (`openDuckDbOverOpfs`) and the shim are
> unwritten — that is KD-11 and KD-05-closure work. The TypeScript below is
> the RPC contract (it type-checks only with ambient declarations this page
> now lists). Divergence row: `KNOWN_DIVERGENCES.md` (KD-05).
> Per-section standing: **[WORKS-TODAY]** verified on this tree;
> **[REQUIRES-KD-05-CLOSURE]** contract-only until the browser build lands.

## What you will build

A page that runs DuckDB entirely client-side: the engine lives in one Web
Worker, the main thread sees only a promise-based RPC surface, and the
database persists across reloads through OPFS synchronous access handles.

- Engine: DuckDB 2.0.0 compiled to WebAssembly (pinned upstream SHA
  `b6013f7`)
- Isolation: one dedicated Worker per engine instance
- Persistence: `FileSystemSyncAccessHandle` byte-range I/O (`pread`/`pwrite`
  shape)
- Recovery: DuckDB WAL replay through the same OPFS layer

Design source of truth: `docs/design/wasm-opfs-architecture.md`. Every
untested claim there is marked UNVERIFIED on purpose; the code in this
tutorial is the executable form of that contract.

## What exists today vs what is contract [REQUIRES-KD-05-CLOSURE]

Measured on this tree (2026-10-19, node 2x `WebAssembly.compile` +
`new WebAssembly.Instance(mod, {})`):

```
target/wasm/duckdb.wasm        imports: 72  modules: a            bare instantiate: FAIL
target/wasm-min/kdc05-min.wasm imports: 22  modules: wasi_snapshot_preview1,env  bare instantiate: FAIL
```

`kdc05-min.wasm` instantiates **only** with the 22-symbol
`wasi_snapshot_preview1.{clock_time_get,fd_*,environ_*,random_get,proc_exit}`
+ `env.{_msync_js,__syscall_*}` shim (25 lines of node; shell stubs return
errno 8 / -1 — receipt `KDC-05-wasm-min.md`). The TypeScript in this page
does **not** type-check as printed: `RpcRequest`/`RowBatch` are referenced
but never declared, and OPFS handle types need `--lib es2022,webworker` plus
two ambient interface augmentations. The full compiling form is at the bottom
of this page (Compiling the contract).

## Compiling the contract [WORKS-TODAY]

Verified: `tsc 7.0.2 --noEmit --strict --target es2022 --lib es2022,webworker`
over the two blocks above plus this `ambient.d.ts` — exit 0:

```ts
// ambient.d.ts — required in addition to the printed blocks; tsc 7.0.2, exit 0
type RpcRequest =
  | { id: number; op: "open"; path: string }
  | { id: number; op: "execute"; sql: string }
  | { id: number; op: "fetch"; sql: string };
interface RowBatch { rows: unknown[]; buffers: ArrayBuffer[]; }
interface StorageManager { persist(): Promise<boolean>; }
interface FileSystemFileHandle {
  createSyncAccessHandle(): FileSystemSyncAccessHandle;
}
```

The `[WORKS-TODAY]` claim covers the type-check only. Instantiation,
queries, OPFS persistence: **[REQUIRES-KD-05-CLOSURE]** — no browser path
exists yet (KD-11 glue unwritten).

## The worker (`wasm-worker.ts`) [REQUIRES-KD-05-CLOSURE]

One worker owns the engine, the OPFS handles, and the module cache. A crash
inside the worker tears down only the worker, never the page.

```ts
// wasm-worker.ts — runs inside a dedicated Web Worker (secure context only).
// FileSystemSyncAccessHandle exists ONLY in workers; this file is the
// containment boundary between the engine and the page.

let conn: WorkerSideConn | null = null;

self.onmessage = async (ev: MessageEvent<RpcRequest>) => {
  const req = ev.data;
  try {
    switch (req.op) {
      case "open": {
        // Persistent storage first: eviction must not silently drop the db.
        await navigator.storage.persist();
        const root = await navigator.storage.getDirectory();
        const fh = await root.getFileHandle("app.db", { create: true });
        // The sync access handle is the point: byte-range I/O with
        // pread/pwrite semantics, no CoW trampoline.
        const handle = await fh.createSyncAccessHandle();
        conn = openDuckDbOverOpfs(handle);
        self.postMessage({ id: req.id, ok: true });
        break;
      }
      case "execute": {
        if (!conn) throw new Error("connection not open");
        conn.execute(req.sql);
        self.postMessage({ id: req.id, ok: true });
        break;
      }
      case "fetch": {
        if (!conn) throw new Error("connection not open");
        const res = conn.fetch(req.sql);
        // Row payloads move as transferable ArrayBuffers, zero-copy.
        self.postMessage({ id: req.id, ok: true, rows: res.rows },
          { transfer: res.buffers });
        break;
      }
    }
  } catch (e) {
    // QuotaExceededError maps to a typed out-of-disk reply, never a hang.
    const kind = e instanceof DOMException && e.name === "QuotaExceededError"
      ? "out-of-disk" as const
      : "engine-error" as const;
    self.postMessage({ id: req.id, ok: false, kind, message: String(e) });
  }
};

// openDuckDbOverOpfs adapts the FileSystemSyncAccessHandle to DuckDB's
// FileSystem interface (see "The OPFS block layer" below).
interface WorkerSideConn {
  execute(sql: string): unknown;
  fetch(sql: string): RowBatch;
}

declare function openDuckDbOverOpfs(h: FileSystemSyncAccessHandle):
  WorkerSideConn;
```

## The main-thread client (`rpc.ts`) [REQUIRES-KD-05-CLOSURE]

The main thread never links the engine. It gets a promise API that mirrors
the native `frozen-duckdb` surface and rejects on worker death so a dead
worker is a typed error, not a hung promise.

```ts
// rpc.ts — main-thread client.
export type RpcRequest =
  | { id: number; op: "open"; path: string }
  | { id: number; op: "execute"; sql: string }
  | { id: number; op: "fetch"; sql: string };

export type RpcReply =
  | { id: number; ok: true; rows?: RowBatch }
  | { id: number; ok: false; kind: "out-of-disk" | "engine-error" };

export class FrozenDuckClient {
  private nextId = 0;
  private inflight = new Map<number, (r: RpcReply) => void>();

  constructor(private worker: Worker) {
    worker.onmessage = (ev: MessageEvent<RpcReply>) => {
      const reply = ev.data;
      const slot = this.inflight.get(reply.id);
      if (!slot) return; // reply for a request this client never sent
      this.inflight.delete(reply.id);
      slot(reply);
    };
    worker.onerror = () => {
      // Dead worker surfaces as a typed error on every inflight call.
      for (const [, slot] of this.inflight) {
        slot({ id: -1, ok: false, kind: "engine-error" });
      }
      this.inflight.clear();
    };
  }

  async open(path: string): Promise<void> {
    const r = await this.call({ id: this.nextId++, op: "open", path });
    if (!r.ok) throw new Error(`open failed: ${r.kind}`);
  }

  async query(sql: string): Promise<RowBatch> {
    const r = await this.call({ id: this.nextId++, op: "fetch", sql });
    if (!r.ok) throw new Error(`query failed: ${r.kind}`);
    if (!r.rows) throw new Error("query succeeded but returned no rows");
    return r.rows;
  }

  private call(req: RpcRequest): Promise<RpcReply> {
    return new Promise((resolve) => {
      this.inflight.set(req.id, resolve);
      this.worker.postMessage(req);
    });
  }
}
```

## The OPFS block layer [REQUIRES-KD-05-CLOSURE]

`openDuckDbOverOpfs` wraps a `FileSystemSyncAccessHandle` in DuckDB's
`FileSystem` interface. The mapping (from
`docs/design/wasm-opfs-architecture.md`):

- `handle.read(buf, {at})` / `write(buf, {at})` / `truncate(len)` /
  `getSize()` / `flush()` / `close()` map onto `FileSystem::Read`/`Write`
  directly — the same shape as `pread`/`pwrite`.
- Writes flush at WAL commit boundaries only; relying on implicit flush is
  unspecified behavior per the OPFS spec.
- Linear memory is capped at 256MB; pin `SET memory_limit='192MB'` on the
  engine so blocks plus query intermediates stay under the cap with headroom
  for wasm stack, FFI buffers, and JS transfer buffers.
- On `QuotaExceededError`, surface the typed `out-of-disk` RPC reply.

## Falsifiers the build must pass [REQUIRES-KD-05-CLOSURE]

The wasm artifact closes KD-05 only by passing these (from the design doc's
falsifier list):

1. **Build falsifier.** The Emscripten/wasi build produces a `duckdb_wasm`
   artifact whose `duckdb_library_version()` reports `2.0.0`, matching
   the vendored headers at upstream SHA `b6013f7`.
2. **Persistence falsifier.** A committed write survives `Page.crash` plus
   reload via WAL replay through the OPFS layer. A clean tab close does not
   exercise this falsifier.
3. **Quota falsifier.** Filling OPFS to quota yields the typed
   `out-of-disk` RPC reply — never a hang, and never a silent short-write
   admitted as success.

## Status and receipts

- Divergence row: `KNOWN_DIVERGENCES.md` KD-05 — PARTIAL_ALIVE as of
  2026-10-19 (receipt:
  `docs/sjira/v2.0.0/receipts/KDC-05-wasm-min.md`: standalone slice
  24.69 MiB < 25 MiB SLA, version gate `v2.0.0` PASS under real
  node instantiation with the 22-import shim).
- Downstream: KD-11 browser courts stay blocked until KD-05 closes.
- Window standing: `docs/sjira/v2.0.0/DEFINITION-OF-DONE.md`
  (DOD-16, known-open).
- Design contract: `docs/design/wasm-opfs-architecture.md`.
