# variant-types

Reference for DuckDB's VARIANT type on the frozen engine: type id 41,
landed across the FFI, Arrow bridge, value layer, and SQL surface.
Design doc: `docs/design/variant-shredding-design.md` (FR-03).

## Type identity

- `DUCKDB_TYPE_VARIANT = 41`
  (`crates/frozen-duckdb-sys` — `DUCKDB_TYPE_DUCKDB_TYPE_VARIANT`).
- Rust mapping: `LogicalTypeId::Variant` in
  `crates/frozen-duckdb/src/duckdb/core/logical_type.rs` and
  `Type::Variant` in `src/duckdb/types/mod.rs`.
- The engine accepts `VARIANT` column DDL and `'x'::VARIANT` casts
  (ALIVE — A23 probe receipt); a `VARCHAR` binds implicitly into a
  VARIANT column via cast on write.

## Bind semantics (write path)

`ValueRef::Variant` binds as its JSON-text payload
(`src/duckdb/statement.rs`, bind arm): the payload string is bound
with `duckdb_bind_varchar_length` and the engine casts varchar ->
VARIANT. A non-string payload is a typed `InvalidInput`-family error,
never a panic.

## Read semantics

- The C Arrow path carries a VARIANT column as a Utf8 payload holding
  the variant's JSON rendering, with extension metadata
  `duckdb.variant` in the field metadata
  (`src/duckdb/types/mod.rs`). Because the payload type alone is
  indistinguishable from plain text, the conservative `Type` mapping
  is `Text`; Variant-ness is recovered at the value layer.
- `serde_json` arm (`src/duckdb/types/serde_json.rs`):
  `ValueRef::Variant(arr, idx)` parses the Utf8 payload (StringArray
  or LargeStringArray) directly as `serde_json::Value`.
- Arrow bridge (`src/duckdb/vtab/arrow.rs`, `LogicalTypeId::Variant`
  arm): the variant payload is the JSON rendering in a
  varchar-style `duckdb_string_t`, so the flat-vector conversion is
  identical to the Varchar arm — an explicit arm, not a panic.
- `::VARCHAR` cast round-trip: `CAST(variant_col AS VARCHAR)` /
  `typeof(...)::VARCHAR` surfaces the JSON rendering; the rendering
  round-trips back through the varchar->VARIANT cast on write.

## Open gap — raw VARIANT through the Arrow bridge

A raw VARIANT column read through the FFI Arrow bridge projects as
`Struct(metadata Binary, value Binary)` — the engine's shredded
internal representation — not as the Utf8 JSON rendering the lib-side
arms above expect (A23 probe, ALIVE). Correspondingly,
`ValueRef::Variant` is never constructed from live columns yet: the
open lib-side read gap is registered in
`docs/design/variant-shredding-design.md` (standing table, UNVERIFIED
row) and tracked for the FR-03 close condition — decode the
`metadata`/`value` Binary buffers into a typed variant view. Until it
lands, read VARIANT with an explicit `::VARCHAR` cast (JSON text), or
via `json_extract_string` on the cast payload.

Standing summary:

| surface | standing |
|---|---|
| DDL + `::VARIANT` casts | ALIVE |
| varchar binds into VARIANT | ALIVE |
| Arrow projection as Struct(metadata, value) | ALIVE (bridge reality) |
| Utf8 JSON payload arms (bind, serde_json, flat-vector) | ALIVE for text-payload columns |
| `ValueRef::Variant` from live raw VARIANT columns | UNVERIFIED (open gap) |

## See Also

`docs/design/variant-shredding-design.md` · `schemas-ocel.md` ·
`functions.md` · `api.md`
