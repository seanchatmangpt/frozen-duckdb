# Host-Call Virtualization: Custom DuckDB VFS over the wasm Boundary

Status: UNVERIFIED — no wasm build exists yet. Every mechanism below is design
rationale until a VFS court proves it on a real remote parquet.

See Also: `docs/design/` (sibling designs), README.md (local-compile fallback)

## Why Not Compile httpfs

Operator's rationale, carried forward as design constraint:

1. **OpenSSL/CA bloat.** httpfs links libcurl + OpenSSL plus the CA bundle. The
   wasm binary grows from ~20 MB to ~80 MB. The whole point of frozen-duckdb is
   a thin drop-in; an 80 MB artifact kills the distribution story.
2. **CORS.** httpfs assumes a native process issuing requests; the browser build
   must honor fetch() CORS semantics anyway, so httpfs's own HTTP layer buys
   nothing and fights the platform.
3. **CA mismatch.** Browsers own their trust store; embedding a CA bundle in
   wasm duplicates (and can disagree with) the host's trust decisions. In BEAM,
   the host runtime already has TLS with the operator's CA config. Either way
   the in-wasm copy is wrong or redundant.

Conclusion: terminate TLS and HTTP at the host; wasm does only reads.

## Clean-Room VFS Wrap Architecture

We do not modify DuckDB. We implement the DuckDB `VirtualFileSystem` /
`FileSystem` interface in frozen-duckdb-sys's C++ shim layer and register it as
the default VFS when built for wasm. Filesystem callbacks do not open sockets;
they marshal across the boundary into the host.

Exported host import (wasm import namespace `env`):

```
int32_t wasm_host_fetch_range(const char* url, int64_t offset, int64_t length,
                              uint8_t* out_buf, int32_t out_buf_cap)
```

Returns byte count on success, negative typed error code on failure (see
Falsifier). The host writes into wasm linear memory at `out_buf`; no host-side
copy escapes into a handle the guest cannot see.

```
+------------------------------------------------------------+
| DuckDB core (wasm, unmodified)                             |
|   parquet scanner                                          |
|     |  file reads                                          |
|     v                                                       |
|   HostCallVFS (C++ shim in frozen-duckdb-sys)              |
|     |  Read(offset, length) -> range request               |
|     v                                                       |
|   [wasm boundary]  wasm_host_fetch_range(url,off,len,buf)  |
+-----|------------------------------------------------------+
      |
      v
+------------------------------------------------------------+
| Host runtime (owns TLS, DNS, credentials, CORS)            |
|   browser: Worker fetch(url, {Range: "bytes=off-end"})     |
|   BEAM:    :finch/:gun request -> copy into linear memory  |
+------------------------------------------------------------+
```

This mirrors the existing builder-subcrate split: the guest stays
ultra-thin; acquisition complexity (here, transport) lives host-side.

## Header-First Parquet Scan Mechanics

Parquet is range-request friendly. The VFS court and the scanner only ever
issue three read shapes:

1. **Footer.** Last 64 KB (grows to 256 KB for pathological metadata): read
   file length first (HEAD or a 1-byte range probe), then
   `Range: bytes=-(64*1024)`. Parse FileMetaData from the trailer.
2. **Row-group dictionary pages.** For each projected column chunk,
   `Range: bytes=<dict_offset>-<dict_offset+dict_len>` from column-chunk
   metadata. Dictionary-decode answers most low-cardinality scans without
   touching data pages.
3. **Data pages (lazy).** Only when a predicate survives the dictionary:
   `Range: bytes=<page_start>-<page_end>` per page, never a whole column.

No read path ever issues an unbounded GET. The host may additionally advertise
`Accept-Encoding: identity` — the guest cannot decompress-transfer-encodings.

## TLS Termination at Host

Zero crypto in the guest: no OpenSSL, no CA bundle, no mbedTLS. The host
performs DNS, TCP, TLS, and HTTP semantics; the guest receives plaintext
parquet bytes. Artifact stays at the ~20 MB baseline. Trust decisions
(redirects, HSTS, certificate pinning, proxies) belong to the operator's host
runtime, not a wasm bundle frozen at build time.

## Credential and PQC-Envelope Isolation

Credentials never enter linear memory. The guest asks for a URL; the host
resolves that URL against its own auth context:

- browser: same-origin cookies / worker-scoped tokens attached by fetch, not
  readable by the guest.
- BEAM: the host process holds tokens/vault handles and attaches headers before
  the request leaves the node.

For PQC-envelope objects: the host decrypts (or has the KMS/edge decrypt)
before the first byte is copied into `out_buf`. The guest sees plaintext
parquet, always. Envelope keys, token caches, and signature verification stay
outside the sandbox; a compromised guest cannot exfiltrate credentials it was
never given.

## Dual-Host Table

| Aspect | Browser (Web Worker) | BEAM (wasmex) |
|---|---|---|
| Transport | fetch() + Range header | :finch/:gun with Range header |
| TLS | browser trust store | BEAM OTP ssl / CA files |
| Credentials | cookies, worker tokens | host process vault/env/KMS |
| Concurrency | one fetch per range; worker pool | one finch/gun pool per node |
| Concurrency limits | HTTP/2 multiplexing | pool_size + HTTP/2 streams |
| Cancellation | AbortController | supervised request process kill |
| Memory crossing | ArrayBuffer -> wasm memory | :mandatory binary copy into memory |
| Backpressure | caller awaits fetch promise | GenServer call / await on finch |
| Error surface | fetch rejection -> negative code | {:error, reason} -> negative code |

## Falsifier

A VFS court (wasm build required) must prove, on a real remote parquet:

1. **Range serve:** a scan of the remote parquet footer succeeds using only
   byte-range requests; the server (or a recording proxy) observes no
   full-file GET. Receipt must show the recorded request log.
2. **Denied-range is structured, not a hang:** the host refuses a range (416 or
   policy denial); `wasm_host_fetch_range` returns a negative typed code and the
   scanner surfaces a structured DuckDB error. A court that hangs, or that
   converts denial to zeros/silence, is FAIL.
3. **No credential leak:** dump guest linear memory during a scan against an
   authenticated URL; no token bytes present.

All three UNVERIFIED until a wasm build exists and the court runs.
