# How To: Integrate the Host-Range VFS

Problem-oriented guide: how a host (browser Web Worker or BEAM/wasmex)
serves remote Parquet ranges to frozen-duckdb through the registered
`host-range://` VFS.

**Honest header — design-stage.** This contract is FR-02 design-stage,
registered as KD-06 in `KNOWN_DIVERGENCES.md`:
"`host-range://` VFS unimplemented (design only)". Nothing here is
implemented; no callback has been registered, no court has run. What
this guide gives you is the integration contract a future implementation
must honor — not a working recipe. The standing ladder is owned by
GATE-V-05 (`docs/design/gate-v05-hostvfs-transfer.md`); this guide
inherits whatever that court proves and claims no rung of its own.

See Also: `docs/design/host-range-vfs-contract.md` (the contract source),
`docs/design/host-call-vfs.md` (architecture),
`docs/design/gate-v05-hostvfs-transfer.md` (acceptance court)

## The contract in one call

The engine registers an internal VFS answering the prefix
`host-range://`; the host implements one range-read primitive:

```
host_read_range(URL: string,
                Offset: i64,          # 0-based, from BOF
                Length: u64,
                BufferPtr: mut [u8; Length]) -> Status
```

One call = one contiguous range. No scatter/gather in v1. On `Success`
the callee has written EXACTLY `Length` bytes into `BufferPtr`; partial
fills do not exist at this layer (the host retries or escalates before
returning).

Callbacks to implement: `open`, `read` (above), `write`, `size`,
`close`.

- `write` is parity only: the remote object is read-only; every write
  returns `InvalidInput`.
- `size` caches content length per handle (one resolve, no unbounded
  GET). The VFS must not require `size` before `read`.
- `close` with pending reads resolves them to `NetworkDenied`.

## Status vocabulary (closed set)

Typed statuses only; there is no numeric channel and no silent-zero
return.

| Status | Authority | Meaning |
|---|---|---|
| `Success` | — | exactly `Length` bytes written |
| `InvalidInput` | engine | neg offset; `Length==0`; bad buffer; write on read-only; dup reg |
| `OutOfRange` | engine or host | `Offset+Length > size`, or server answered `416` |
| `NetworkDenied` | host | DNS/TCP/TLS fail, non-2xx/206, 200-to-range, policy refusal, cancel |
| `QuotaExceeded` | host | byte/count budget exhausted for the session |

Closed-set law: any condition outside this table is a contract bug —
map it to the nearest row or do a contract revision, never a bare
negative int with no row.

## Buffer ownership (the leak fence)

`BufferPtr` is caller-allocated. The host NEVER retains it past the
return — no handle, no pointer, no subrange cache may alias guest
memory after return. Any host-side cache must COPY. This is what makes
the credential-leak falsifier enforceable. No zero-copy escape hatch in
v1.

## The access pattern your host must serve

Header-first, three shapes, no unbounded GET ever:

1. **Footer first**: `size` (or 1-byte probe), then
   `read(URL, size - 64 KiB, 64 KiB)` (grow to 256 KiB for pathological
   metadata). Parse Parquet `FileMetaData` before any other range.
2. **Dictionary pages** per projected column chunk, via
   `dict_offset`/`dict_len`. Dictionary-only answers must not touch
   data pages.
3. **Lazy data pages**, one range per page, only when a predicate
   survives the dictionary. Never a whole column chunk, never a whole
   file.

Alignment: no hard requirement on Offset/Length; the VFS must NOT
reject unaligned requests (rejection by alignment is not in the status
table, hence unlawful). Perf-only hints: `Offset % 8 == 0`,
`Length >= 4 KiB`.

## Dual-host transport cheat sheet

| Aspect | Browser (Web Worker) | BEAM (wasmex) |
|---|---|---|
| `read` transport | `fetch(url, {Range: ...})` | `:finch`/`:gun` with `Range` header |
| `size` transport | `HEAD`, else 1-byte range probe | same |
| `OutOfRange` | `416` -> typed status | `416` -> typed status |
| `NetworkDenied` | fetch reject, CORS, 200-to-range | `{:error, r}`, 200-to-range, timeout |
| TLS | browser trust store | OTP `:ssl` + operator CA config |
| Credentials | cookies / worker tokens | host vault/env/KMS headers pre-flight |
| Cancellation | `AbortController` -> `NetworkDenied` | supervised kill -> `NetworkDenied` |

## Non-vacuity: the 200-instead-of-206 shim

The GATE-V-05 falsifier injects a server that answers `200` (full body)
to range requests. A compliant host maps that to `NetworkDenied` — a
silent full-download fallback is FAIL. If your integration passes the
court without that shim ever being able to flip the outcome, the court
is vacuous; fix the court.

## Close condition

KD-06 closes when the host-call VFS is implemented AND the GATE-V-05
<= 5 MB transfer court runs green on a real subject. Until then every
mechanism here is design; do not cite this guide as a working capability.

See Also: `KNOWN_DIVERGENCES.md` (KD-06),
`docs/how-to/how-to-run-chicago-doctor.md` (stage 04_host_vfs)
