# prd-ard-v2

Status: requirements source of truth for the frozen-duckdb v2.0 blueprint,
transcribed from the operator's PRD/ARD (2026-10-19). Every FR/NFR starts
standing PROPOSED. The only landed substrate today is the NATIVE 2.0.0
engine (bundled from upstream v2.0-cyanoptera b6013f7) — all
WASM/OPFS/25MB/trap-boundary items are unstarted. Traceability:
docs/design/requirements-traceability.md.

## Executive Summary

Frozen-DuckDB v2.0 transitions the embedded analytical engine from an
offline/build-time packaging layer to an active, deterministic, in-browser
and edge-native OLAP runtime. By decoupling from fragile native C++
compilation chains and native OS shared libraries, v2.0 delivers an
embeddable, sandboxed database engine compiled to WebAssembly
(WASI/Emscripten) and native targets.

v2.0 targets client-side data sovereignty, zero-cost distributed analytics,
and zero-panic runtime safety: complex analytical queries, cohort retention
matrices, and multi-object event aggregations execute on the end-user's
local hardware without incurring centralized cloud data warehouse fees or
remote network latency.

## Problem Statement

1. Compilation drag and toolchain fragility: embedding analytical databases
   in multi-platform applications requires complex native toolchains and
   C++17 compilers that break host environments, produce platform-dependent
   shared-object collisions, and complicate CI/CD.
2. The cloud data warehouse tax: high-volume telemetry and event streams are
   piped to remote lakehouses purely to answer exploratory cohort and
   aggregation queries, creating cloud compute, storage, and egress bills.
3. Runtime crash hazards: in-process analytical engines via native FFIs
   carry continuous operational risk — one unaligned memory access, unhandled
   C++ exception, or out-of-memory condition crashes the entire host process.
4. Client-side storage volatility: in-browser analytics have relied on
   ephemeral memory heaps lost on refresh, or slow key-value abstractions
   that cannot handle vectorized columnar scans.

## Target Personas

- Edge & systems engineers: self-contained, statically bound analytical
  database; zero external dynamic dependencies; zero host-process panic risk.
- Enterprise application developers: embedded client-side database executing
  rich analytical SQL inside browser contexts without cloud round-trips.
- Data analysts & process practitioners: interactive, sub-15ms local query
  performance across multi-object event streams; no local database daemons.

## User Journeys & Core Capabilities

- Local-first analytical querying: the application boots the embedded engine
  in a dedicated browser background worker, opens a synchronous access handle
  to browser origin storage, and executes sub-second vectorized SQL over
  millions of rows.
- Zero-cloud continuous aggregation: event streams are written locally in a
  compressed columnar representation; dashboards execute window aggregations
  and retention analyses on-device, synchronizing only cryptographically
  signed digests upstream.
- Fail-safe crash recovery: abrupt power loss or tab crash mid-transaction
  leaves storage that replays the internal write-ahead journal on reopen
  without corruption.

## Functional Requirements

- FR-01 Sandboxed WebAssembly execution: the engine must compile cleanly to
  modern 32-bit and 64-bit WebAssembly targets with disabled native POSIX
  dependencies, executing entirely within linear-memory sandboxes.
- FR-02 Native synchronous storage persistence: must integrate with browser
  OPFS synchronous access handles, providing direct block-level columnar
  persistence that survives page reloads.
- FR-03 Semi-structured type support: full columnar execution support for
  dynamically typed semi-structured data (shredded JSON/Variant) without
  requiring pre-defined static table schemas.
- FR-04 Vectorized on-demand view expansion: evaluate generative window
  series and mathematical expansions in vectorized memory chunks to
  reconstruct dense event logs from compressed segment representations.
- FR-05 Strict memory cap enforcement: configurable linear memory bounds
  with immediate graceful query aborts rather than unhandled host panics or
  out-of-memory termination.

## Non-Functional Requirements

- NFR-01 Query latency: p95 execution latency for analytic aggregations over
  100,000 in-memory rows must complete in under 15 milliseconds.
- NFR-02 Binary footprint: total stripped engine binary footprint must not
  exceed 25 megabytes.
- NFR-03 Storage compression efficiency: run-length and dictionary-compressed
  columnar tables must achieve at least 90% byte-footprint reduction versus
  standard raw JSON representations.
- NFR-04 Host isolation (zero panic): a segmentation fault, fatal allocator
  error, or invalid pointer access inside the engine must be trapped within
  the sandbox interface, returning a structured error code to the host
  without terminating the host process.

## System Architecture

```
+=====================================================================+
|                     HOST RUNTIME BOUNDARY                           |
|              (Browser Worker / Application VM)                      |
+=====================================================================+
                          |  Shared memory / FFI protocol
                          v
+=====================================================================+
|                FROZEN-DUCKDB v2.0 SANDBOX                           |
|  Unboxed execution interface (C ABI / WASM exports):                |
|    init_engine(config)        execute_vectorized_query(sql)         |
|    ingest_batch(buffer, len)  close_engine()                        |
|  ------------------------------------------------------------------ |
|  Vectorized columnar core:                                          |
|    SIMD-accelerated vector scans     Arrow RecordBatch bridge       |
|    PEG SQL parser & query planner    Variant shredder               |
|  ------------------------------------------------------------------ |
|  Bounded memory & buffer manager:                                   |
|    linear memory pool guard (<=256MB)  fixed-size query work bufs   |
+=====================================================================+
                          |  Synchronous block I/O
                          v
+=====================================================================+
|              PERSISTENT BLOCK STORAGE LAYER                         |
|     (Browser OPFS / local block volume / encrypted VFS)             |
+=====================================================================+
```

Note from transcription: the operator's original diagram names a "PEG SQL
parser". The pinned upstream engine's parser architecture is whatever the
pinned 2.0-dev source carries; the parser-internals claim is UNVERIFIED and
carried at face value as a requirement, not an engine fact.

## Subsystem Specifications

### A. Host-to-engine boundary interface

- ABI conventions: pure unboxed C-compatible FFI. Arguments are primitive
  integers, unboxed pointers, and byte lengths only.
- Memory transfer contract: zero-copy bidirectional sharing via
  pre-allocated shared memory buffers. Batch ingestion accepts raw byte
  pointers of packed columnar arrays; no intermediate serialization (e.g.
  JSON string parsing) at the boundary.
- Error propagation protocol: every boundary function returns an explicit
  status code (Success, InvalidInput, OutOfMemory, DiskExhausted,
  ExecutionHalt). Diagnostics are retrieved via a secondary thread-safe
  error-buffer fetch function.

### B. Storage & virtual file system (VFS) layer

- Synchronous access mechanics: in browser contexts the VFS targets
  synchronous file handles directly inside dedicated execution threads,
  ensuring atomic read-and-write operations at native speeds.
- Journaling & page layout: 4KB-aligned columnar page layout; changes append
  to an internal write-ahead journal; automatic recovery at initialization
  when an unclean shutdown marker is detected.
- Concurrency isolation: exclusive single-writer locks at the storage-handle
  level. Concurrent read sessions run on independent memory contexts;
  conflicting writes are rejected immediately with structured error codes,
  never blocking or corrupting storage.

### C. Vectorized query & data representation

- Semi-structured Variant representation: semi-structured columns are parsed
  directly into in-memory decomposed columnar buffers (shredded keys and
  values) so projection and filter on nested attributes never require
  full-row decompression.
- Arrow in-memory equivalence: query output chunks conform to standard
  columnar layout (contiguous validity bitmasks, offset buffers, data
  vectors) enabling direct zero-copy consumption by host pipelines.

## Responsibility & Component Matrix

| Architectural area | Functional scope | Key constraint | Success metric |
| --- | --- | --- | --- |
| Sandboxed engine core | Query parsing, optimization, columnar execution | Zero POSIX/OS thread or network dependencies | Clean compilation across WASM and embedded native targets |
| Virtual file system | Storage block reads, writes, truncation, flushing | Exclusive single-handle access, synchronous block calls | 100% crash recovery across abrupt process halts |
| Boundary marshaller | Ingestion buffering, query response streaming | Zero-copy byte buffer manipulation | >=100,000 rows/sec ingestion throughput |
| Memory allocator guard | Dynamic memory tracking, query ceiling enforcement | Hard upper threshold (e.g. 256MB/512MB) | Zero unhandled out-of-memory crashes |

## Verification & Acceptance Criteria

1. Substrate portability: the engine must initialize, construct tables,
   ingest data, and return query results on both native headless targets and
   inside real browser background worker threads without modification.
2. Crash-chaos resilience: terminating the host execution thread
   mid-transaction during a continuous write loop must leave the physical
   storage mountable, with all committed checkpoints intact and verifiable
   via checksum.
3. Memory confinement: a deliberate Cartesian join or large aggregation
   designed to exceed the memory limit must fail gracefully with an
   OutOfMemory status code, preserving engine state and host memory.
4. Compression proof: storing 1,000,000 structured process events must yield
   a database file at least 90% smaller than the uncompressed JSON event
   equivalent.

## See Also

requirements-traceability.md - verification-plan.md - wasm-opfs-architecture.md

## Revision 2 — Dual-Target Release & Host-VFS (2026-10-19)

Operator's revised mandate (2026-10-19), transcribed from session direction.
Id-note: Revision-1's FR-01..05 / NFR-01..04 numbering above is superseded by
the PRD-FR-* / SLA-* scheme in this revision; the alias map lives in
docs/design/requirements-traceability.md.

### Baseline facts

1. Native size divide: the native release split is 66.9MB (feature engine)
   versus 53.2MB (baseline engine). Any single-binary "everything bundled"
   story dies against this divide.
2. 80/20 extension reality: most real workloads need a small fixed extension
   set; everything else is served through Host-VFS virtualization rather
   than static bundling.
3. Type conformance boundary: 5-of-6 type conformance is the observed
   ceiling; the sixth type is the known boundary, not a regression.
4. Freeze-as-moat: pinning the engine (bundled upstream v2.0-cyanoptera
   b6013f7) is the moat — hermetic, reproducible, no floating upstream.

### Dual-target PRD requirements

- PRD-FR-01 Hermetic static bundling: the pinned engine is statically
  bundled; extension autoinstall and autoloading are OFF. Boundary: no
  runtime download path; a missing extension is a typed error.
- PRD-FR-02 Host-VFS virtualization: non-bundled capability is virtualized
  via the host. cURL and OpenSSL are explicitly excluded from
  virtualization. Boundary: host-supplied I/O only; no network/TLS inside
  the engine.
- PRD-FR-03 VARIANT API with zero-panic confinement. Boundary: any
  malformed input returns a structured error, never a host crash.
- PRD-FR-04 OPFS synchronous persistence in browser contexts. Boundary:
  block-level columnar storage surviving page reloads.
- PRD-FR-05 Bounded memory with hard cap. Boundary: <=256MB linear memory;
  graceful abort, not OOM kill.

### Service-level agreements

- SLA-1 Query latency: p95 < 15ms for analytic aggregation over 100k rows.
- SLA-2 Footprint: <=25MB wasm engine binary; <=10MB wire (download) size.
- SLA-3 Ingest throughput: >100,000 transitions/sec sustained ingestion.
- SLA-4 Durability: 100% WAL recovery — committed state survives abrupt
  halt, every time.

### Dual-target topology

```
+--------------------------------------------------------------------+
|                        HOST RUNTIME                                |
|     (native embedder / Browser Worker with OPFS handles)           |
+--------------------------------------------------------------------+
            |  unboxed C ABI (native)  |  WASM exports (browser)
            v                          v
+--------------------------------------------------------------------+
|                FROZEN-DUCKDB v2.0 ENGINE (frozen)                  |
|  Target A: NATIVE static bundle (66.9MB feature / 53.2MB base)     |
|    - PRD-FR-01: autoinstall OFF, autoloading OFF                   |
|    - bundled: the pinned 80/20 extension set only                  |
|  Target B: WASM (<=25MB) with Host-VFS virtualization              |
|    - PRD-FR-02: host-range:// VFS; cURL/OpenSSL EXCLUDED           |
|    - PRD-FR-04: OPFS sync access handles (block I/O)               |
|  Shared core:                                                      |
|    - PRD-FR-03: VARIANT API, zero-panic (structured errors)        |
|    - PRD-FR-05: bounded memory pool, <=256MB hard cap              |
+--------------------------------------------------------------------+
            |  synchronous block I/O
            v
+--------------------------------------------------------------------+
|   PERSISTENT BLOCK STORAGE (OPFS / native block volume)            |
|   WAL journal; SLA-4: 100% recovery on unclean shutdown            |
+--------------------------------------------------------------------+
```

### Subsystem specifications (Revision 2)

#### A. host-range:// VFS contract

- The host registers a range-read VFS under the `host-range://` scheme.
  The engine issues bounded range requests; the host answers from its own
  storage (HTTP range, local cache, or OPFS mirror).
- Host-side signature (conceptual, language-agnostic):

```
host_read_range(ctx: *mut u8, path: *const u8, path_len: usize,
                offset: u64, buf: *mut u8, buf_len: usize) -> i32
```

- Return: bytes read on success; negative structured error codes on failure
  (mapping to the boundary status protocol in Subsystem A, Revision 1).
- Exclusion: any path or capability requiring cURL or OpenSSL is refused at
  registration with a typed error — the host never proxies network/TLS.

#### B. 80/20 static integration + autoloading lockdown

- The native bundle statically links exactly the 80/20 extension set
  (see extension-matrix-8020.md). Everything outside that set is a
  Host-VFS or host-call concern, never a bundle member.
- Lockdown: `autoinstall` and `autoloading` are compiled/configured OFF in
  both targets. An unknown extension reference is a typed
  UNSUPPORTED-style error at bind time — no silent fetch, no silent load.
- Rationale: the 66.9MB/53.2MB native divide is the budget; every static
  addition must justify itself against SLA-2.

#### C. Unboxed ABI / zero-copy / allocation guard

- Unboxed boundary: primitive integers, unboxed pointers, byte lengths only
  (extends Revision-1 Subsystem A; no boxed variants, no JSON at the edge).
- Zero-copy: ingestion and query output operate on pre-registered shared
  buffers; Arrow-compatible columnar layout for direct host consumption.
- Allocation guard: all engine allocation flows through the bounded pool
  (PRD-FR-05). Cap breach aborts the query with OutOfMemory status; host
  memory and engine state survive. Sustained throughput target SLA-3.

### Gates

- GATE-V-01 Hermetic bundle (PRD-FR-01).
  Pass: engine runs with network disabled; zero autoinstall/autoloading
  attempts observed.
  Fail: any runtime fetch attempt, or successful load of a non-bundled
  extension.
- GATE-V-02 Host-VFS virtualization (PRD-FR-02).
  Pass: host-range:// serves all non-bundled I/O; cURL/OpenSSL registration
  refused with a typed error.
  Fail: engine performs direct network/TLS I/O, or host-range:// silently
  falls back to a bundled path.
- GATE-V-03 VARIANT zero-panic (PRD-FR-03).
  Pass: fuzzed/malformed VARIANT inputs all return structured errors; host
  process alive.
  Fail: any host crash, trap escape, or unhandled C++ exception.
- GATE-V-04 OPFS persistence (PRD-FR-04).
  Pass: kill mid-write, reopen: 100% WAL replay, committed checkpoints
  intact (SLA-4).
  Fail: any unmountable storage or lost committed checkpoint.
- GATE-V-05 Memory + SLA envelope (PRD-FR-05, SLA-1..3).
  Pass: p95<15ms@100k; >100k transitions/sec; <=25MB wasm / <=10MB wire;
  cap breach yields OutOfMemory status.
  Fail: any SLA miss, or a cap breach crashing instead of aborting
  gracefully.

### Traceability & ontology governance

- This revision supersedes Revision-1 numbering (FR-01..05, NFR-01..04);
  the frozen alias map FR->PRD-FR / NFR->SLA lives in
  docs/design/requirements-traceability.md. New receipts cite PRD-FR-*/SLA-*
  IDs only; the old IDs are read-only aliases.
- Standing: PRD-FR-02 (Host-VFS), the SLA-2 wasm envelope, and the dual-topology
  are PROPOSED until a gate receipt lands; PRD-FR-01/03/04/05 inherit the
  Revision-1 substrate state (native 2.0.0 engine landed; WASM/OPFS
  unstarted). Gates GATE-V-01..05 are the admission courts — a requirement
  flips standing only on a passing gate run, never on prose.
- Generated artifacts (gates/, generated/) are never hand-edited; gate
  evidence refreshes only through their generators, per repo convention.

## See Also (Revision 2)

extension-matrix-8020.md - host-call-vfs.md - requirements-traceability.md -
verification-plan.md - wasm-opfs-architecture.md
