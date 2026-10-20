//! [`ToSql`] and [`FromSql`] implementation for JSON `Value`.

use serde_json::Value;

use crate::duckdb::{
    types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef},
    Result,
};

/// Serialize JSON `Value` to text.
impl ToSql for Value {
    #[inline]
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(serde_json::to_string(self).unwrap()))
    }
}

/// Deserialize text/blob to JSON `Value`.
impl FromSql for Value {
    #[inline]
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        // U5 court finding: a raw VARIANT column read through the FFI
        // Arrow bridge projects as the engine's shredded internal
        // representation Struct(metadata Binary, value Binary) — never as
        // ValueRef::Variant (the registered FR-03 open gap). Falling
        // through to InvalidType here surfaced as the opaque
        // InvalidColumnType(0, "v", Struct([("metadata", Blob), ("value",
        // Blob)])), which hides VARIANT-ness and the documented ::VARCHAR
        // workaround. Name both instead.
        if let ValueRef::Struct(arr, _) = &value {
            if is_shredded_variant(arr) {
                return Err(FromSqlError::Other(
                    "cannot read a raw VARIANT column as serde_json::Value: it \
                     projects as the shredded Struct(metadata, value) buffers \
                     (FR-03 open gap); read it with an explicit cast instead, \
                     e.g. SELECT v::VARCHAR FROM t, then parse the JSON text"
                        .into(),
                ));
            }
        }
        match value {
            ValueRef::Text(s) => serde_json::from_slice(s),
            ValueRef::Blob(b) => serde_json::from_slice(b),
            // VARIANT surfaces as a Utf8 array whose payload is the variant's
            // JSON rendering; parse it directly.
            ValueRef::Variant(arr, idx) => {
                use arrow::array::{LargeStringArray, StringArray};
                // DT3: a null variant element decodes to Value::Null via the
                // Option<T> FromSql impl, not a parse error on "".
                if arr.is_null(idx) {
                    return Ok(serde_json::Value::Null);
                }
                let payload = if let Some(s) = arr.as_any().downcast_ref::<StringArray>() {
                    s.value(idx)
                } else if let Some(s) = arr.as_any().downcast_ref::<LargeStringArray>() {
                    s.value(idx)
                } else {
                    return Err(FromSqlError::InvalidType);
                };
                serde_json::from_str(payload)
            }
            _ => return Err(FromSqlError::InvalidType),
        }
        .map_err(|err| FromSqlError::Other(Box::new(err)))
    }
}

/// True when the struct is the engine's shredded VARIANT projection: exactly
/// the fields `metadata` (Binary) and `value` (Binary). See
/// `docs/reference/variant-types.md` ("Open gap") and
/// `docs/design/variant-shredding-design.md` FR-03.
fn is_shredded_variant(arr: &arrow::array::StructArray) -> bool {
    let fields = arr.fields();
    fields.len() == 2
        && fields[0].name() == "metadata"
        && fields[1].name() == "value"
        && fields[0].data_type() == &arrow::datatypes::DataType::Binary
        && fields[1].data_type() == &arrow::datatypes::DataType::Binary
}

#[cfg(test)]
mod test {
    use crate::duckdb::{types::ToSql, Connection, Result};

    fn checked_memory_handle() -> Result<Connection> {
        let db = Connection::open_in_memory()?;
        db.execute_batch("CREATE TABLE foo (t TEXT, b BLOB)")?;
        Ok(db)
    }

    #[test]
    fn test_json_value() -> Result<()> {
        let db = checked_memory_handle()?;

        let json = r#"{"foo": 13, "bar": "baz"}"#;
        let data: serde_json::Value = serde_json::from_str(json).unwrap();
        db.execute(
            "INSERT INTO foo (t, b) VALUES (?, ?)",
            [&data as &dyn ToSql, &json.as_bytes()],
        )?;

        let t: serde_json::Value = db.query_row("SELECT t FROM foo", [], |r| r.get(0))?;
        assert_eq!(data, t);
        let b: serde_json::Value = db.query_row("SELECT b FROM foo", [], |r| r.get(0))?;
        assert_eq!(data, b);
        Ok(())
    }

    /// U5 court: reading a live raw VARIANT column as serde_json::Value must
    /// name the shredded-VARIANT projection and the ::VARCHAR workaround, not
    /// the opaque InvalidColumnType(Struct(metadata, value)).
    #[test]
    fn test_raw_variant_read_is_actionable() -> Result<()> {
        let db = Connection::open_in_memory()?;
        db.execute_batch(
            "CREATE TABLE t (v VARIANT); INSERT INTO t VALUES ('{\"a\":1}'), (NULL);",
        )?;
        let mut stmt = db.prepare("SELECT v FROM t")?;
        let mut rows = stmt.query([])?;
        let mut saw_payload = false;
        let mut saw_null = false;
        while let Some(row) = rows.next()? {
            let got: Result<Option<serde_json::Value>> = row.get(0);
            match got {
                // DT3 guard: NULL variant reads as Ok(None) via Option<T>.
                Ok(None) => saw_null = true,
                Err(crate::duckdb::Error::FromSqlConversionFailure(_, _, ref err)) => {
                    let msg = err.to_string();
                    assert!(
                        msg.contains("shredded Struct(metadata, value)")
                            && msg.contains("::VARCHAR"),
                        "error not actionable: {msg}"
                    );
                    saw_payload = true;
                }
                other => panic!("unexpected result for raw VARIANT read: {other:?}"),
            }
        }
        assert!(saw_payload && saw_null);
        // The documented workaround still round-trips.
        let text: String =
            db.query_row("SELECT v::VARCHAR FROM t WHERE v IS NOT NULL", [], |r| {
                r.get(0)
            })?;
        assert_eq!(text, r#"{"a":1}"#);
        Ok(())
    }
}
