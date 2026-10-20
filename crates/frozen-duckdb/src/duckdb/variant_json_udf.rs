//! Crate-side gap fix (lane MAX3, 2026-10-19): this build's engine dylib has
//! no statically-linked json extension — `json_extract` /
//! `json_extract_string` are absent from the catalog and INSTALL/LOAD are
//! refused (`this build does not link the loadable_extensions library`), so
//! nested-path reads over shredded VARIANT columns are otherwise impossible.
//!
//! Fix: a feature-gated (`serde_json`) scalar-function pair registered per
//! connection, backed by serde_json — the crate's own JSON dependency — with
//! a minimal JSONPath subset (`$`, `.key`, `["key"]`, `[index]` chains),
//! matching DuckDB's json-extraction contract on the shapes this crate's
//! VARIANT court exercises:
//! - `json_extract(doc, path) -> VARCHAR`: JSON text of the located element
//!   (objects/arrays included), SQL NULL when the path misses.
//! - `json_extract_string(doc, path) -> VARCHAR`: unquoted scalar text, SQL
//!   NULL for objects/arrays/missing keys (DuckDB semantics).
//!
//! Real engine, real serde_json collaborator, no mocks. Chicago-school: the
//! court asserts on stored state read back through the engine.

use serde_json::Value;

use crate::duckdb::{
    core::{DataChunkHandle, LogicalTypeId},
    types::DuckString,
    vscalar::{ScalarFunctionSignature, VScalar},
    vtab::arrow::WritableVector,
    Connection,
};

/// Resolve a minimal JSONPath (`$`, `.key`, `["key"]`, `[n]`) against `doc`.
fn resolve<'a>(doc: &'a Value, path: &str) -> Option<&'a Value> {
    let path = path.trim();
    let rest = path.strip_prefix('$').unwrap_or(path);
    if rest.is_empty() {
        return Some(doc);
    }
    let mut cur = doc;
    let bytes = rest.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'.' => {
                i += 1;
                let start = i;
                while i < bytes.len() && bytes[i] != b'.' && bytes[i] != b'[' {
                    i += 1;
                }
                cur = cur.get(&rest[start..i])?;
            }
            b'[' => {
                i += 1;
                let start = i;
                while i < bytes.len() && bytes[i] != b']' {
                    i += 1;
                }
                if i >= bytes.len() {
                    return None;
                }
                let token = rest[start..i].trim().trim_matches('"').trim_matches('\'');
                i += 1; // skip ']'
                cur = if let Ok(idx) = token.parse::<usize>() {
                    cur.get(idx)?
                } else {
                    cur.get(token)?
                };
            }
            _ => return None,
        }
    }
    Some(cur)
}

fn invoke_extract(
    input: &mut DataChunkHandle,
    output: &mut dyn WritableVector,
    string_semantics: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::duckdb::core::Inserter;
    let len = input.len();
    let doc_vec = input.flat_vector(0);
    let path_vec = input.flat_vector(1);
    let docs = doc_vec.as_mut_ptr::<frozen_duckdb_sys::duckdb_string_t>();
    let paths = path_vec.as_mut_ptr::<frozen_duckdb_sys::duckdb_string_t>();
    let mut out = output.flat_vector();
    for row in 0..len {
        // Copy both strings before mutating the output vector: DuckString
        // borrows from the input chunk's storage.
        // SAFETY: row < len, both vectors are flat varchar vectors over the
        // input chunk's own storage, which outlives this invocation.
        let doc = unsafe { DuckString::new(&mut *docs.add(row)).as_str().to_string() };
        let path = unsafe { DuckString::new(&mut *paths.add(row)).as_str().to_string() };
        if doc_vec.row_is_null(row as u64) || path_vec.row_is_null(row as u64) {
            out.set_null(row);
            continue;
        }
        let parsed: Value = match serde_json::from_str(&doc) {
            Ok(v) => v,
            Err(e) => return Err(format!("json_parse error: {e}").into()),
        };
        match resolve(&parsed, &path) {
            None => out.set_null(row),
            Some(Value::Null) => out.set_null(row),
            // json_extract_string: scalars unquoted; objects/arrays stay NULL
            // (DuckDB json_extract_string semantics).
            Some(v) if string_semantics && matches!(v, Value::Object(_) | Value::Array(_)) => {
                out.set_null(row)
            }
            Some(Value::String(s)) if string_semantics => out.insert(row, s.as_str()),
            Some(v) => out.insert(row, v.to_string().as_str()),
        }
    }
    Ok(())
}

struct JsonExtract;
impl VScalar for JsonExtract {
    type State = ();
    unsafe fn invoke(
        _: &Self::State,
        input: &mut DataChunkHandle,
        output: &mut dyn WritableVector,
    ) -> Result<(), Box<dyn std::error::Error>> {
        invoke_extract(input, output, false)
    }
    fn signatures() -> Vec<ScalarFunctionSignature> {
        vec![ScalarFunctionSignature::exact(
            vec![LogicalTypeId::Varchar.into(), LogicalTypeId::Varchar.into()],
            LogicalTypeId::Varchar.into(),
        )]
    }
}

struct JsonExtractString;
impl VScalar for JsonExtractString {
    type State = ();
    unsafe fn invoke(
        _: &Self::State,
        input: &mut DataChunkHandle,
        output: &mut dyn WritableVector,
    ) -> Result<(), Box<dyn std::error::Error>> {
        invoke_extract(input, output, true)
    }
    fn signatures() -> Vec<ScalarFunctionSignature> {
        vec![ScalarFunctionSignature::exact(
            vec![LogicalTypeId::Varchar.into(), LogicalTypeId::Varchar.into()],
            LogicalTypeId::Varchar.into(),
        )]
    }
}

/// Register `json_extract` and `json_extract_string` on this connection.
/// Call once after opening; idempotent per connection name registration.
pub fn register_json_extract(conn: &Connection) -> crate::duckdb::Result<()> {
    conn.register_scalar_function::<JsonExtract>("json_extract")?;
    conn.register_scalar_function::<JsonExtractString>("json_extract_string")
}
