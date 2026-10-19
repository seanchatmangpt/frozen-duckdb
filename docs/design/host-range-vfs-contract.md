# Host-Range VFS Registration Contract (ARD §3.2.A)

Status: UNVERIFIED — contract formalization only. Nothing here is
implemented; no callback has been registered, no court has run. Every
mechanism below is design until a real receipt lands.

See Also: `docs/design/host-call-vfs.md` (architecture: why the host
terminates TLS/HTTP, clean-room wrap, credential isolation),
`docs/design/gate-v05-hostvfs-transfer.md` (transfer budget court and
byte-accounting law), `docs/design/prd-ard-v2.md` (§3.2.A source).

## Layer map

This document is the CONTRACT layer between the two documents above:

- `host-call-vfs.md` = ARCHITECTURE (why a wrap, why not httpfs, where
  TLS lives).
- this doc = CONTRACT (exact symbols, signatures, status vocabulary,
  ownership, alignment, access pattern both sides must honor).
- `gate-v05-hostvfs-transfer.md` = ACCEPTANCE (the 5 MB budget, the
  shim-court non-vacuity flips, the receipt fields).

A change to the architecture that alters any symbol or status code
below is a contract change and must edit this doc in the same commit.

## Registered VFS prefix

The engine registers an internal VFS answering the prefix
`host-range://` at registration time, before any query executes:

```
HostRangeVFS::Register(instance, /*prefix=*/"host-range://")
```

- `URL(URL) -> host-range://<origin>/<path>` is a qualified remote
  object. The VFS dispatches on scheme match; no path is local.
- Registration is idempotent: re-registration with the same prefix is
  a no-op; registration with a different implementation under the same
  prefix returns `InvalidInput` and changes nothing.
- The prefix is internal: user SQL does not type `host-range://`.
  The parquet scanner receives it via `read_parquet('<url>')` and the
  shim rewrites the scheme. (Rewrite rule is architecture, not
  contract; see host-call-vfs.md.)

## Formal signature

```
host_read_range(URL: string,
                Offset: i64,          # byte offset, 0-based, from BOF
                Length: u64,          # requested byte count
                BufferPtr: mut [u8; Length]) -> Status
```

- One call requests exactly one contiguous range. No scatter/gather in
  v1; multi-range batching is a future contract revision (bump the
  prefix to `host-range/v2://` if semantics change).
- Offset >= 0. Offset == 0 is the first byte of the object; the VFS
  must resolve object size via `size` (below) before any negative
  offset could arise — negative offsets are never guest-visible.
- Return contract: on `Success` the callee has written exactly `Length`
  bytes into `BufferPtr` and returned `Success`. Partial fills do not
  exist at this layer — a short read from the wire is retried or
  escalated to a typed error by the host before returning.

## Callbacks

The registered VFS implements, at minimum:

| Callback | Signature |
|---|---|
| `open` | `(URL) -> Handle \| Err(Status)` |
| `read` | `host_read_range` (formal signature above) |
| `write` | `(Handle, Offset, Length, BufferPtr) -> Status` |
| `size` | `(Handle) -> u64 \| Err(Status)` |
| `close` | `(Handle) -> ()` |

Callback contracts:

- `open`: resolves the scheme; no I/O on the wire. The first wire I/O
  is the footer range.
- `read`: the formal signature above.
- `write`: interface parity only. The remote object is read-only;
  every write returns `InvalidInput`.
- `size`: resolves content length once, cached per handle; no
  unbounded GET.
- `close`: drops the handle; pending reads resolve to
  `NetworkDenied`.

`open`/`size` may be issued in either order relative to the first
`read`; the VFS must not require `size` before `read`.

## Status vocabulary (closed set)

All statuses are typed; there is no numeric-only channel and no
silent-zero return.

| Status | Authority | Guest-visible mapping |
|---|---|---|
| `Success` | — | proceed |
| `InvalidInput` | engine (pre-wire) | structured query error |
| `OutOfRange` | engine or host | structured query error |
| `NetworkDenied` | host | structured query error; counter frozen pre-error |
| `QuotaExceeded` | host | structured query error |

Status meanings:

- `Success`: exactly `Length` bytes written into `BufferPtr`.
- `InvalidInput`: negative offset; `Length == 0`; `BufferPtr` outside
  guest linear memory; `write` on a read-only object; duplicate
  prefix registration.
- `OutOfRange`: `Offset + Length > object_size`, or the server
  answered `416`.
- `NetworkDenied`: transport refused — DNS/TCP/TLS failure,
  non-2xx/206, `Range` unsupported (server answered `200` to a range
  request), policy refusal, cancellation, close with pending reads.
- `QuotaExceeded`: host-side byte/count budget exhausted for the
  query session.

Closed-set law: any condition outside this table is a contract bug —
the implementation must map it to the nearest row above or return a
new typed code via a contract revision, never a bare negative int with
no row. The gate falsifier (b) exercises the `NetworkDenied` row via
the `200-instead-of-206` shim; a silent full-download fallback is FAIL
per gate-v05 falsifier (b).

## Buffer ownership

- `BufferPtr` is caller-allocated (the guest owns it in wasm; the
  engine owns it in native). The callee writes into it and returns.
- The host NEVER retains `BufferPtr` past the return of
  `host_read_range`: no handle, no pointer, no subrange cache may
  alias guest linear memory after return. Any host-side cache must
  copy. (This is what makes falsifier 3 of host-call-vfs.md — no
  credential/envelope leak — enforceable: the guest's memory is not
  shared state.)
- No zero-copy escape hatch in v1. If a future revision adds
  zero-copy, it is a new callback with a new ownership clause, not an
  exception to this one.

## Alignment expectations

- No hard alignment requirement on `Offset` or `Length`: parquet
  footers, dictionary pages, and data pages are byte-aligned ranges.
- Recommendations (non-binding, perf only): `Offset % 8 == 0` and
  `Length >= 4 KiB` for page-range reads keep the host copy paths on
  their fast path; the footer read (last 64 KB) is always unaligned at
  its start and is fine.
- The VFS must not reject an unaligned request: rejection by alignment
  is not in the status table and is therefore unlawful.

## Header-first access pattern (binding on callers)

The VFS contract assumes, and the scanner must follow, the three-shape
access law from host-call-vfs.md:

1. **Footer first.** `size` (or a 1-byte range probe) then
   `read(URL, size - 64 KiB, 64 KiB)` — grow to 256 KiB for pathological
   metadata. Parse `FileMetaData` from the trailer before any other
   range.
2. **Dictionary pages per projected column chunk**, using the
   `dict_offset`/`dict_len` from column-chunk metadata. Dictionary-only
   answers must not touch data pages.
3. **Lazy data pages**, only when a predicate survives the dictionary:
   one range per page, never a whole column chunk, never a whole file.

No read path issues an unbounded GET. A caller that requests a range
covering the whole object is lawful at the contract layer (the VFS
must serve it) but violates the access pattern and will blow the
gate-v05 budget — the court, not this doc, is what fails it.

## Dual-host implementation table

| Aspect | Browser (Web Worker) | BEAM (wasmex) |
|---|---|---|
| `read` transport | `fetch(url, {Range: ...})` | `:finch`/`:gun` with `Range` header |
| `size` transport | `HEAD`, else 1-byte range probe | `HEAD`, else 1-byte range probe |
| `Success` path | `206` body copied into wasm memory | body binary copied into wasm memory |
| `OutOfRange` | `416` response -> typed status | `416` response -> typed status |
| `NetworkDenied` | fetch reject, CORS, `200`-to-range | `{:error, r}`, `200`-to-range, timeout |
| `QuotaExceeded` | host counter (gate-v05 law) | same law (`sum(length_copied)`) |
| TLS/trust | browser trust store | OTP `:ssl` + operator CA config |
| Credentials | cookies / worker tokens via fetch | host vault/env/KMS headers pre-flight |
| Cancellation | `AbortController` -> `NetworkDenied` | supervised process kill -> `NetworkDenied` |
| Concurrency | one fetch per range; worker pool | one finch/gun pool per node |
| Backpressure | caller awaits the fetch promise | GenServer call / `:finch` await |

## Standing

UNVERIFIED (terminal for now): no `HostRangeVFS` implementation, no
registered prefix, no wasm artifact. The standing ladder is owned by
gate-v05 (`UNVERIFIED -> PARTIAL_ALIVE on the native leg -> ALIVE on
the wasm boundary leg`); this doc inherits whatever that court proves
and never claims a rung on its own.

See Also: `docs/design/host-call-vfs.md`,
`docs/design/gate-v05-hostvfs-transfer.md`,
`docs/design/prd-ard-v2.md`
