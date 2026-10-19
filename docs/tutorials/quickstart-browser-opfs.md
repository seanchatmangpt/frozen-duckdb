# Quickstart: frozen-duckdb in the browser (Web Worker + OPFS)

> **Standing: BLOCKED (KD-05), as of 2026-10-19.** This tutorial describes the
> FR-01 target. The wasm artifact does **not** exist yet:
> `scripts/build_wasm_duckdb.sh` exits 2 against a broken Homebrew emscripten
> install, so there is nothing runnable in a browser today. The TypeScript
> below is **the contract the wasm build must satisfy**, not a runnable
> today-path. Divergence row: `KNOWN_DIVERGENCES.md` (KD-05); the blocking
> receipt is the exit-2 build attempt recorded there. When KD-05 closes, this
> page flips from BLOCKED to a real quickstart with a court receipt path.

## What you will build

A page that runs DuckDB entirely client-side: the engine lives in one Web
Worker, the main thread sees only a promise-based RPC surface, and the
database persists across reloads through OPFS synchronous access handles.

- Engine: DuckDB 2.0.0-dev compiled to WebAssembly (pinned upstream SHA
  `b6013f7`)
- Isolation: one dedicated Worker per engine instance
- Persistence: `FileSystemSyncAccessHandle` byte-range I/O (`pread`/`pwrite`
  shape)
- Recovery: DuckDB WAL replay through the same OPFS layer

Design source of truth: `docs/design/wasm-opfs-architecture.md`. Every
untested claim there is marked UNVERIFIED on purpose; the code in this
tutorial is the executable form of that contract.

## The worker (`wasm-worker.ts`)

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

## The main-thread client (`rpc.ts`)

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

## The OPFS block layer

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

## Falsifiers the build must pass

The wasm artifact closes KD-05 only by passing these (from the design doc's
falsifier list):

1. **Build falsifier.** The Emscripten/wasi build produces a `duckdb_wasm`
   artifact whose `duckdb_library_version()` reports `2.0.0-dev`, matching
   the vendored headers at upstream SHA `b6013f7`.
2. **Persistence falsifier.** A committed write survives `Page.crash` plus
   reload via WAL replay through the OPFS layer. A clean tab close does not
   exercise this falsifier.
3. **Quota falsifier.** Filling OPFS to quota yields the typed
   `out-of-disk` RPC reply — never a hang, and never a silent short-write
   admitted as success.

## Status and receipts

- Divergence row: `KNOWN_DIVERGENCES.md` KD-05 — BLOCKED as of 2026-10-19
  (receipt: `scripts/build_wasm_duckdb.sh` exit 2).
- Downstream: KD-11 browser courts stay blocked until KD-05 closes.
- Window standing: `docs/sjira/v2.0.0-dev/DEFINITION-OF-DONE.md`
  (DOD-16, known-open).
- Design contract: `docs/design/wasm-opfs-architecture.md`.
