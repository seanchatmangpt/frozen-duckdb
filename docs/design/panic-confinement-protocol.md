# Panic Confinement Protocol (NFR-04)

Lane: P8 (PRD, write lane). Repo: /Users/sac/frozen-duckdb.
Standing: PROPOSED (design); current substrate standing: NOT CONFINED on
native path. Depends on: FR-01 (WASM sandbox) — hard dependency edge.

## Current substrate (brutal honesty)

TODAY, on the native FFI path, this crate cannot confine panics or
faults. Concretely:

- `crates/frozen-duckdb-sys/` issues raw `unsafe` `duckdb_*` calls into
  the native C++ library. Any segfault, assertion, or abort inside
  DuckDB C++ kills the host process outright. Rust observes nothing.
- The crate does not pin `panic = "abort"`, so Rust-side panics unwind.
  An unwind that crosses an `extern "C"` FFI frame is undefined
  behavior per the platform ABI. There is no `catch_unwind` boundary
  at the crate edge today.
- C++-side faults (OOM in the native allocator, internal DuckDB
  exceptions that escape as traps) are entirely outside Rust's control.
  No Rust mechanism can catch them.

The repo's own law bans `unwrap`/`expect` on external input at the Rust
layer; that law is real but bounded: it addresses Rust-side panics only.
A C++ fault is not a Rust panic and cannot be disciplined from Rust.
Record honestly: NFR-04 is NOT MET on the native path and cannot be met
there by any Rust-only mechanism.

## Dependency edge: NFR-04 requires FR-01

The actual mitigation is the WASM sandbox (FR-01). In WASM, faults are
structural: any out-of-linear-memory access, trap, or abort raises a
wasm trap at the sandbox boundary. The host catches the trap and
converts it into a typed error. The host process survives. This is the
only path on which panic confinement is achievable, because WASM's
linear memory model makes a faulting access a recoverable event rather
than a process-fatal one.

Consequence: NFR-04 is fundamentally FR-01-dependent. Ordering:

1. FR-01 (WASM execution path) lands.
2. NFR-04 confinement semantics are specified against that path.
3. Native path retains documented partial mitigations only (below).

If FR-01 is BLOCKED or descoped, NFR-04 is BLOCKED with it. Do not
claim NFR-04 ALIVE while any supported execution path lacks a trap
boundary.

## Native-side partial mitigations (PROPOSED, not yet built)

These reduce Rust-side blast radius but do NOT confine C++ faults.
They are worth specifying so the native path degrades gracefully where
it can.

### M1. `catch_unwind` at the crate boundary

Wrap the public API surface (query entry points) in
`std::panic::catch_unwind` and convert a caught panic into a typed
`Error::PanicConfined` status.

Caveats, documented up front:

- `catch_unwind` must not wrap a frame that is mid-FFI-call into C++.
  If the panic originates inside native code, the stack is C++ and
  unwinding across it is UB. Scope: confine Rust-side panics only,
  in frames that hold no live FFI call. Assert no active
  `duckdb_*` call inside the guarded closure.
- `UnwindSafe` bounds: public API takes `&Database`/`&Connection`;
  wrap with `AssertUnwindSafe` deliberately and document why the
  interior state is safe to continue from (or refuse continuation:
  return an error that marks the connection poisoned).
- Requires `panic = "unwind"` in the release profile; if the profile
  ever flips to `abort`, M1 is void — add a build-time check that
  fails the gate if `panic` is not `unwind` while NFR-04 is claimed.

### M2. Memory-limit-driven graceful query abort

Drive DuckDB's `memory_limit` setting from a crate-level config so an
over-consuming query aborts inside the engine with a normal DuckDB
error, not a native OOM. Ties to FR-05 / AC-3 (resource limits).

- Set `memory_limit` (and `temp_directory` for spilling) at
  connection setup; expose a builder knob.
- The abort surfaces as a DuckDB error string over FFI, which the
  crate maps to a typed `OutOfMemory` status (see M3).
- This is the mechanism the NFR-04 falsifier exercises.

### M3. DuckDB error-data API for C++-side error propagation

Use DuckDB's structured error data (error type / message fields via
the C API) rather than string-matching, so engine-side failures
(including the M2 memory abort) arrive as typed values. Map
`DUCKDB_ERROR_OUT_OF_MEMORY` (and equivalents) to
`frozen_duckdb::Error::OutOfMemory`. Unknown classes map to a
catch-all typed variant — never a panic, never `unwrap`.

## Typed status surface (PROPOSED)

```rust
pub enum Error {
    // ...
    OutOfMemory { context: String },
    PanicConfined { payload: String },
    SandboxTrapped { trap: String }, // FR-01 path
}
```

## Falsifier (NFR-04 acceptance)

A deliberately OOM'd query must return the typed `OutOfMemory`
status, and the engine must be reusable after.

Concrete court:

1. Open a connection with a small `memory_limit` (e.g. 64 MiB).
2. Run a query that provably exceeds it (e.g. a large cross join or
   `range()` aggregation sized past the limit).
3. Assert: the returned error is `Error::OutOfMemory` (typed match,
   not string contains), the call returned normally (no
   panic/abort/signal), and a subsequent trivial query on the same
   connection (or a fresh connection on the same engine) succeeds.
4. Non-vacuity: run the same query with a generous limit and assert
   it succeeds — the court must be able to flip both ways.

For the FR-01 path, the same court runs with the query in the WASM
sandbox and asserts `Error::SandboxTrapped` (or mapped
`OutOfMemory`) with the host process alive.

Court status today: BLOCKED — the OOM currently kills the native
process before a typed status can be returned (M2/M3 not landed), and
FR-01 is not landed.

## Standing summary

| Surface | Standing |
|---|---|
| Native FFI confinement | NOT CONFINED (process-fatal on C++ fault) |
| Rust panic boundary (M1) | PROPOSED |
| Memory-limit abort (M2) | PROPOSED (ties FR-05/AC-3) |
| Typed error data (M3) | PROPOSED |
| WASM trap boundary (FR-01) | DEPENDENCY — NFR-04 gates on it |

## See Also

FR-01 (WASM sandbox), FR-05 (resource limits / AC-3),
`crates/frozen-duckdb-sys/` (FFI surface).
