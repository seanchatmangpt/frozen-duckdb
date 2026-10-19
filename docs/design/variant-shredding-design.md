# Variant Shredding Design (FR-03)

Status: DRAFT — grounded in landed native reality (A23 findings, quoted).
All unexecuted steps are UNVERIFIED.

## 1. Context

FR-03 targets first-class semi-structured VARIANT support. The native
substrate has already landed DuckDB's VARIANT type (type id 41) across the
FFI surface. This doc records what is ALIVE, what remains open, and the
design for shredded columnar reads.

## 2. Landed native reality (A23 findings, verbatim input)

A23's real probe findings ARE the design input:

- `DUCKDB_TYPE_VARIANT=41` is integrated: `LogicalTypeId::Variant`,
  `Type::Variant`, `ValueRef::Variant`, the bind arm, and serde_json
  `FromSql` all exist in the landed tree.
- VARIANT DDL and `'x'::VARIANT` casts are **accepted** by the engine.
- varchar binds **implicitly cast** into VARIANT columns.
- Raw VARIANT columns project through the FFI Arrow bridge as
  `Struct(metadata Binary, value Binary)` — the engine's shredded
  representation leaks through unshredded.
- Exact round-trip is reachable today via an explicit `::VARCHAR` cast on
  projection.
- Open lib-side read gap: `ValueRef::Variant` is never constructed from
  live columns; current semantics are Utf8 passthrough.

## 3. Standing of each surface

| Surface | Standing | Evidence class |
|---|---|---|
| Type id 41 integration | ALIVE | landed code, A23 probes |
| DDL + `::VARIANT` casts | ALIVE | A23 probe: accepted |
| varchar bind implicit cast | ALIVE | A23 probe |
| Arrow projection as Struct(metadata, value) | ALIVE | A23 probe |
| `::VARCHAR` round-trip | ALIVE | A23 probe |
| `ValueRef::Variant` from live columns | UNVERIFIED (open gap) | none yet |
| True shredded columnar reads | UNVERIFIED | design only |

## 4. Design

### 4.1 Phase 1 — close the lib-side read gap (next native step)

Construct `ValueRef::Variant` from live columns by decoding the Arrow
Struct the bridge already emits: read the `metadata` and `value` Binary
buffers and surface a typed variant view. Until this lands, reads stay
Utf8 passthrough or explicit `::VARCHAR`.

### 4.2 Phase 2 — true shredded columnar reads

Consume the engine's shredder directly: project `metadata`/`value`
buffers as separate columnar streams instead of a wrapped Struct.
Pruning and partial projection operate on shredded paths. UNVERIFIED —
no execution receipt yet.

### 4.3 WASM-side FR-03

The WASM side inherits native behavior by construction; no independent
WASM variant semantics. Any WASM divergence is a defect, not a design
surface.

## 5. Falsifiers

- Phase 1: a live `SELECT v FROM t` still yields Utf8 passthrough after
  the read-gap fix lands.
- Phase 2: shredded read returns bytes whose round-trip differs from the
  `::VARCHAR` baseline round-trip on the same rows.

## 6. Receipt discipline

No standing flip or ALIVE claim for Phases 1-2 without a real command
receipt (exact query, actual output, exit code). Every unexecuted step
above remains UNVERIFIED regardless of prose confidence.
