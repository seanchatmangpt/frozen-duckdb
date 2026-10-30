# USER-FINDINGS-U5 — VARIANT-first user probe (lane U5)

Date: 2026-10-19. Subject: `/Users/sac/frozen-duckdb` @ master 0a205b7 (dirty tree, lane-owned files only).
Method: outside view = `docs/reference/variant-types.md` + README VARIANT mentions only; then every
documented API exercised against the real engine via scratch crate `/tmp/u5/probe` (path dep on
`crates/frozen-duckdb`, `features = ["serde_json"]`, prebuilt libduckdb arm64 v2.0.0).
Probe source: `/tmp/u5/probe/src/main.rs`; run receipt: `cd /tmp/u5/probe && cargo run` (exit 0).

## Per-API verdict table

| # | Documented surface | Doc promise | Observed (real run) | Verdict |
|---|---|---|---|---|
| 1 | `CREATE TABLE t(v VARIANT)` DDL | accepted (ALIVE) | `Ok`, engine accepts VARIANT DDL | ALIVE |
| 2 | varchar bind into VARIANT (`INSERT ... VALUES ($1)` with `&str` JSON) | implicit cast-on-write (ALIVE) | `Ok(1)`; row later read back as `{"a":1}` | ALIVE |
| 3 | raw VARIANT column read | projects as `Struct(metadata Binary, value Binary)` (documented gap) | `ValueRef::Struct(StructArray {metadata: Binary [1,0,0], value: Binary [29,123,...]})` — exactly as the "Open gap" section says | MATCH (doc honest) |
| 4 | `::VARCHAR` cast read | JSON text round-trip (ALIVE) | `ValueRef::Text({"a":1})`; round-trips | ALIVE |
| 5 | `serde_json::Value` read of raw col | doc implied the serde arm reads VARIANT | `Err(FromSqlConversionFailure …)` — **was** opaque `InvalidColumnType(0, "v", Struct([("metadata", Blob), ("value", Blob)]))`: hides VARIANT-ness, no workaround named. **FIXED this session** (see below) | GAP → FIXED |
| 6 | NULL variant via `Option<serde_json::Value>` | DT3 guard: `Ok(None)` | raw path `Ok(None)`; cast path `Ok(None)`; no `""`-parse error | ALIVE (DT3 holds) |
| 7 | nested payload insert + `json_extract_string` filter | (doc: read via cast) | insert `{"user":{"id":7,...}}` `Ok(1)`; `WHERE json_extract_string(v,'$.user.id')='7'` returns the row | ALIVE |
| 8 | non-JSON text into VARIANT | not promised either way | `INSERT 'not-json{{'` → `Ok(1)`, silently stored (typeof = VARIANT). No engine-side JSON validation — worth a doc note, not a bug | MATCH (undocumented behavior) |
| 9 | `Value::Variant(String)` bind | (type exists) | `Ok(1)`, stored `{"k":true}` | ALIVE |
| 10 | `ValueRef::Variant` user-facing bind | doc "Bind semantics" describes `ValueRef::Variant` bind arm | `ValueRef` is **not re-exported at the crate root** and does **not** implement `ToSql`, so the documented bind arm is not directly reachable by consumers; reachable paths are `&str` bind (cast-on-write) and `Value::Variant`. Doc overstates reachability | DOC GAP (minor) |

## Worst mismatch found and fixed (code)

Reading a live raw VARIANT column as `serde_json::Value` failed with
`InvalidColumnType(0, "v", Struct([("metadata", Blob), ("value", Blob)]))` — the user sees
`Struct(Blob, Blob)` with zero indication the column is VARIANT or that the documented
`::VARCHAR` workaround exists.

Fix in `crates/frozen-duckdb/src/duckdb/types/serde_json.rs` (`FromSql for serde_json::Value`):
detect the shredded VARIANT projection (fields exactly `metadata`/`value`, both `Binary`) via
`is_shredded_variant`, and return a typed `FromSqlError::Other` whose message names the shredded
`Struct(metadata, value)` projection and the `SELECT v::VARCHAR` workaround. Full decode of the
shredded buffers stays the registered FR-03 close condition; this only makes the refusal
actionable.

Verification (same session, same subject):

- `CARGO_TARGET_DIR=/tmp/u5/target cargo test -p frozen-duckdb --lib --features serde_json serde_json`
  → `2 passed; 0 failed` (`test_json_value`, new `test_raw_variant_read_is_actionable` — asserts the
  message contains "shredded Struct(metadata, value)" and "::VARCHAR", asserts NULL row is
  `Ok(None)`, asserts the `::VARCHAR` round-trip `{"a":1}`).
- Probe rerun: `serde get -> Err(FromSqlConversionFailure(0, Struct([...]), "cannot read a raw
  VARIANT column as serde_json::Value: it projects as the shredded Struct(metadata, value)
  buffers (FR-03 open gap); read it with an explicit cast instead, e.g. SELECT v::VARCHAR FROM t,
  then parse the JSON text"))`.
- `cargo clippy -p frozen-duckdb --all-features` → clean; `cargo fmt -p frozen-duckdb` applied.

Doc updated: `docs/reference/variant-types.md` — serde arm section now names the feature
requirement, the live-column typed error, and the DT3 NULL standing; standing table row added.

## Other findings (not fixed, registered)

- F-U5-1 (doc, minor): "Bind semantics" section implies consumers bind `ValueRef::Variant`
  directly; `ValueRef` is not root-exported and does not impl `ToSql`. Reachable binds:
  `&str`/`String` (cast-on-write) and `Value::Variant(String)`. Doc fix suggested.
- F-U5-2 (undocumented): VARIANT accepts any varchar payload with no JSON validation
  (`'not-json{{'` stores fine); bad JSON surfaces only at consumer parse time. Worth one doc line.
- F-U5-3 (pre-existing, unchanged): raw VARIANT as `Struct(metadata, value)` through the Arrow
  bridge remains the FR-03 open gap; `ValueRef::Variant` still never constructed from live
  columns.

Standing: FIXED (serde error quality) / ALIVE per table / DOC GAP F-U5-1, F-U5-2 open.
