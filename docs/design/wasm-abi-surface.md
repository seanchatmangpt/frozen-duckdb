# wasm-abi-surface

Status: DRAFT (lane X14). WASM instantiation PROPOSED / UNVERIFIED until
FR-01 (compiled WASM artifact + boundary court). This doc is the
wasm-boundary projection of `boundary-abi-contract.md` (§2.3 of
`sovereign-spec-v26.10.19.md`); the native instantiation of the same
contract is grounded in the repo, this one is not.

## 1. Scope and authority

Sovereign spec source: `sovereign-spec-v26.10.19.md` §2.3 (Unboxed C
ABI surface) and §2.5 (linear memory map). Boundary law lives in
`boundary-abi-contract.md` and is inherited here by reference, not
restated as new law: primitives/pointers/lengths only; no
`Result`/`Option`/`Vec`/`String`/fat slices across the line; no
ownership transfer; no panics across the boundary; every acquire has
exactly one destroy.

Divergence note (honest): `boundary-abi-contract.md` §1 names the
campaign-era export set `init_engine` / `execute_vectorized_query` /
`ingest_batch` / `close_engine`. The sovereign spec §2.3 (operator,
later) supersedes those names with the `fdb_*` family below. This doc
uses the `fdb_*` names; the older names appear only in the boundary
contract's historical text.

## 2. Export signatures (verbatim from §2.3)

Five exports. Signatures are written in the boundary contract's
permitted type vocabulary (`u32`/`i64`/`u64`, raw pointers, adjacent
lengths). All return `u32` status codes per §4.

### 2.1 fdb_init_engine

```
fdb_init_engine(db_path_ptr: const u8*,
                db_path_len: u64,
                mem_limit_mb: u64) -> u32 status
```

Acquire an engine instance (config in, handle out). `db_path_ptr` is
a UTF-8 `(ptr, len)` pair per the boundary string rule (not
NUL-terminated). `mem_limit_mb` bounds the engine working pool and
must fit the §3.1 memory budget. The returned handle occupies a
low u32 slot (wasm32 pointer width); `0` is not a valid handle on
`Success` — a `Success` return with handle `0` is a contract
violation.

### 2.2 fdb_close_engine

```
fdb_close_engine(handle: u32) -> u32 status
```

Paired destroy for `fdb_init_engine`; no implicit lifetimes. After a
`Success` return the handle is dead: any further use is `InvalidInput`
and a court finding, not UB. Exactly one close per successful init.

### 2.3 fdb_ingest_conforming_batch

```
fdb_ingest_conforming_batch(batch_ptr: const u8*,
                            batch_len: u64) -> u32 status
```

Zero-copy Arrow batch ingest: host allocates, engine fills in place
(sovereign §2.3 wording); buffers are released by the owning side via
the paired destroy symbol (boundary lifecycle-pairing rule). The
batch is an Arrow C structure (`ArrowArray` + `ArrowSchema`,
matching the existing native `duckdb_arrow` path).

### 2.4 fdb_execute_query

```
fdb_execute_query(query_ptr: const u8*,
                  query_len: u64,
                  out_record_batch_ptr: u8**,
                  out_record_batch_len: u64*) -> u32 status
```

Vectorized query execution; Arrow C structures out, released via the
paired destroy symbol. On `Success` the out-pair is written: pointer
to the result batch in the Arrow region (§3.1) and its byte length.
On non-`Success` the engine must not leave a partially written
out-pair that the host could read as valid.

### 2.5 fdb_register_host_vfs

```
fdb_register_host_vfs(fetch_fn: u32 callback) -> u32 status
```

Register the host range-read callback under the internal
`host-range/v1://` prefix (sovereign §2.1; contract in
`host-range-vfs-contract.md`). One registration per engine instance;
re-registration is `InvalidInput`.

## 3. Linear memory map (PROPOSED)

### 3.1 Operator-specified layout (sovereign §2.5)

Under the ≤256 MB tab-level budget (`memory-bounds.md`), wasm32
linear memory:

| Region | Base | Role |
|---|---|---|
| Query pool | `0x0` | buffer manager, sorts, hash aggs (under `mem_limit_mb`) |
| Arrow region | `0x08000000` | zero-copy Arrow C structures (§2 exports) |
| Scratch pages | `0x0F000000` | host-VFS range buffers + shim state; GATE-V-05 counted |

### 3.2 Tension, stated honestly

The table above is a PROPOSED operator constant set. Actual wasm
segment layout is linker-determined: `lld`/`wasm-ld` places data,
stack, and heap segments, and nothing in a normal toolchain honors
fixed base addresses for runtime regions. The Arrow-region and
scratch bases are therefore a *budget partition* (how the 256 MB is
divided among roles), not a promise about linked addresses, until a
linked artifact proves placement (custom linker script or explicit
allocation at those bases). Standing: PROPOSED / UNVERIFIED — no wasm
build exists to place these regions. FR-01 receipts must include the
actual linked segment map next to this table.

## 4. Status codes

Aligned with `boundary-abi-contract.md` §3 — the same closed set, not
a wasm-local variant:

| code | name | meaning |
|---|---|---|
| 0 | `Success` | call completed; out-params valid |
| 1 | `InvalidInput` | bad ptr/len, non-UTF-8, bad SQL, wrong params/type |
| 2 | `OutOfMemory` | engine alloc failed (incl. `mem_limit_mb` breach) |
| 3 | `DiskExhausted` | persistent write failed for space/quota |
| 4 | `ExecutionHalt` | execution aborted: interrupt, timeout, or contained panic |

Closed set; `u32::MAX` reserved as `Unknown` (always a contract
violation finding). Error detail is fetched from the per-handle error
buffer (boundary contract §4), never packed into the return.

## 5. host_vfs callback alignment

The `fetch_fn` passed to `fdb_register_host_vfs` is the host
implementation of `host_read_range` (`host-range-vfs-contract.md`,
formal signature):

```
host_read_range(URL: string,
                Offset: i64,
                Length: u64,
                BufferPtr: mut [u8; Length]) -> Status
```

Alignment rules for the wasm crossing:

- The guest passes a `(ptr, len)` pair for `URL` (UTF-8), `Offset`
  as `i64`, `Length` as `u64`, and a `BufferPtr` inside the scratch
  region (§3.1), keeping the boundary-abi permitted-type law.
- Return-contract parity: on `Success` the callee has written exactly
  `Length` bytes; partial fills do not exist at this layer; the host
  retries or escalates to a typed status before returning.
- The full five-callback VFS set (`open`/`read`/`write`/`size`/
  `close`) from the host-range contract is unchanged; `read` is the
  `fetch_fn` registered here. `write` remains interface parity only
  (always `InvalidInput`).
- Scratch-region residency of `BufferPtr` makes every VFS byte copy
  GATE-V-05 countable (§3.1); a buffer outside the scratch region is
  a contract violation finding.

## 6. Standing summary

| claim | standing |
|---|---|
| `fdb_*` five-export surface (this doc) | PROPOSED / UNVERIFIED until FR-01 |
| status-code closed set | ALIGNED to boundary-abi §3 (native PARTIAL_ALIVE, wasm UNVERIF.) |
| linear memory map | PROPOSED / UNVERIFIED (linker tension noted in §3.2) |
| `fetch_fn` = `host_read_range` | SPECIFIED; callback UNIMPLEMENTED (host-range-vfs-contract.md) |
| native instantiation | ALIVE (boundary-abi-contract §7) — this doc is NOT that |

## See Also

`boundary-abi-contract.md` · `host-range-vfs-contract.md` ·
`sovereign-spec-v26.10.19.md` · `memory-bounds.md` ·
`wasm-opfs-architecture.md`
