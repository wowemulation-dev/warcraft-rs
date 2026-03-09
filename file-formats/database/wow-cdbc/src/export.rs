//! Export and import functionality for DBC data

use crate::{Record, RecordSet, Value};
#[cfg(feature = "serde")]
use crate::{Schema, StringBlock, StringRef};
#[cfg(feature = "serde")]
use serde::Serialize;
use std::collections::HashMap;
use std::io;
#[cfg(feature = "serde")]
use std::sync::Arc;

/// A serializable wrapper for a record value
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum SerializableValue {
    /// String value
    String(String),
    /// 32-bit signed integer
    Int32(i32),
    /// 32-bit unsigned integer
    UInt32(u32),
    /// 32-bit floating point number
    Float32(f32),
    /// Boolean value
    Bool(bool),
    /// 8-bit unsigned integer
    UInt8(u8),
    /// 8-bit signed integer
    Int8(i8),
    /// 16-bit unsigned integer
    UInt16(u16),
    /// 16-bit signed integer
    Int16(i16),
    /// Array of values
    Array(Vec<SerializableValue>),
}

impl SerializableValue {
    /// Convert a Value to a SerializableValue
    fn from_value(value: &Value, record_set: &RecordSet) -> Result<Self, io::Error> {
        match value {
            Value::Int32(v) => Ok(SerializableValue::Int32(*v)),
            Value::UInt32(v) => Ok(SerializableValue::UInt32(*v)),
            Value::Float32(v) => Ok(SerializableValue::Float32(*v)),
            Value::StringRef(v) => {
                let string = record_set
                    .get_string(*v)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
                Ok(SerializableValue::String(string.to_string()))
            }
            Value::Bool(v) => Ok(SerializableValue::Bool(*v)),
            Value::UInt8(v) => Ok(SerializableValue::UInt8(*v)),
            Value::Int8(v) => Ok(SerializableValue::Int8(*v)),
            Value::UInt16(v) => Ok(SerializableValue::UInt16(*v)),
            Value::Int16(v) => Ok(SerializableValue::Int16(*v)),
            Value::Array(values) => {
                let mut array = Vec::with_capacity(values.len());
                for v in values {
                    array.push(Self::from_value(v, record_set)?);
                }
                Ok(SerializableValue::Array(array))
            }
        }
    }
}

/// A serializable record
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct SerializableRecord {
    /// The field values, mapped by field name if a schema is available
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub values: HashMap<String, SerializableValue>,
}

impl SerializableRecord {
    /// Convert a Record to a SerializableRecord
    pub fn from_record(record: &Record, record_set: &RecordSet) -> Result<Self, io::Error> {
        let mut values = HashMap::new();

        if let Some(schema) = record.schema() {
            for (i, field) in schema.fields.iter().enumerate() {
                if let Some(value) = record.get_value(i) {
                    let serializable_value = SerializableValue::from_value(value, record_set)?;
                    values.insert(field.name.clone(), serializable_value);
                }
            }
        } else {
            // No schema, use numeric field names
            for (i, value) in record.values().iter().enumerate() {
                let serializable_value = SerializableValue::from_value(value, record_set)?;
                values.insert(format!("field_{i}"), serializable_value);
            }
        }

        Ok(Self { values })
    }
}

/// Export a record set to JSON
#[cfg(feature = "serde")]
pub fn export_to_json<W: io::Write>(record_set: &RecordSet, writer: W) -> Result<(), io::Error> {
    let mut serializable_records = Vec::with_capacity(record_set.len());

    for record in record_set.records() {
        let serializable_record = SerializableRecord::from_record(record, record_set)?;
        serializable_records.push(serializable_record);
    }

    serde_json::to_writer_pretty(writer, &serializable_records)
        .map_err(|e| io::Error::other(e.to_string()))
}

/// Import a record set from JSON, using a schema to determine field types.
///
/// The JSON must be an array of objects where each key matches a field name
/// in the provided schema. String fields are expected as JSON strings; numeric
/// and boolean fields are expected as their corresponding JSON primitives.
///
/// Returns a `RecordSet` ready to be written with `DbcWriter`.
#[cfg(feature = "serde")]
pub fn import_from_json<R: io::Read>(
    reader: R,
    schema: Schema,
) -> Result<RecordSet, io::Error> {

    let rows: Vec<serde_json::Map<String, serde_json::Value>> =
        serde_json::from_reader(reader).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    // Build a string block incrementally.
    // Offset 0 is always the empty string (null terminator only).
    let mut string_data: Vec<u8> = vec![0u8];
    // Map from string content to its offset in the string block.
    let mut string_offsets: HashMap<String, u32> = HashMap::new();
    string_offsets.insert(String::new(), 0);

    let schema_arc = Arc::new(schema.clone());
    let mut records: Vec<Record> = Vec::with_capacity(rows.len());

    for (row_idx, row) in rows.iter().enumerate() {
        let mut values: Vec<Value> = Vec::with_capacity(schema.fields.len());

        for field in &schema.fields {
            let json_val = row.get(&field.name).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "row {}: missing field '{}' required by schema",
                        row_idx, field.name
                    ),
                )
            })?;

            let value = if field.is_array {
                let arr = json_val.as_array().ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("row {}: field '{}' expected a JSON array", row_idx, field.name),
                    )
                })?;

                let expected = field.array_size.unwrap_or(0);
                if arr.len() != expected {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "row {}: field '{}' array has {} elements, expected {}",
                            row_idx, field.name, arr.len(), expected
                        ),
                    ));
                }

                let mut elements = Vec::with_capacity(arr.len());
                for (elem_idx, elem) in arr.iter().enumerate() {
                    elements.push(parse_scalar(
                        elem,
                        field.field_type,
                        &mut string_data,
                        &mut string_offsets,
                        row_idx,
                        &format!("{}[{}]", field.name, elem_idx),
                    )?);
                }
                Value::Array(elements)
            } else {
                parse_scalar(
                    json_val,
                    field.field_type,
                    &mut string_data,
                    &mut string_offsets,
                    row_idx,
                    &field.name,
                )?
            };

            values.push(value);
        }

        records.push(Record::new(values, Some(Arc::clone(&schema_arc))));
    }

    let string_block = StringBlock::from_bytes(string_data);
    Ok(RecordSet::new(records, Some(schema_arc), string_block))
}

/// Parse a single scalar JSON value into a DBC `Value` given a field type.
#[cfg(feature = "serde")]
fn parse_scalar(
    json_val: &serde_json::Value,
    field_type: crate::FieldType,
    string_data: &mut Vec<u8>,
    string_offsets: &mut HashMap<String, u32>,
    row_idx: usize,
    field_name: &str,
) -> Result<Value, io::Error> {
    use crate::FieldType;

    let type_err = |expected: &str| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "row {}: field '{}' expected {}, got {}",
                row_idx, field_name, expected, json_val
            ),
        )
    };

    Ok(match field_type {
        FieldType::Int32 => {
            let v = json_val.as_i64().ok_or_else(|| type_err("integer"))?;
            Value::Int32(v as i32)
        }
        FieldType::UInt32 => {
            let v = json_val.as_u64().ok_or_else(|| type_err("unsigned integer"))?;
            Value::UInt32(v as u32)
        }
        FieldType::Float32 => {
            let v = json_val.as_f64().ok_or_else(|| type_err("float"))?;
            #[allow(clippy::cast_possible_truncation)]
            Value::Float32(v as f32)
        }
        FieldType::Bool => {
            // Accept JSON boolean or 0/1 integers
            let v = if let Some(b) = json_val.as_bool() {
                b
            } else if let Some(n) = json_val.as_u64() {
                n != 0
            } else {
                return Err(type_err("boolean"));
            };
            Value::Bool(v)
        }
        FieldType::UInt8 => {
            let v = json_val.as_u64().ok_or_else(|| type_err("unsigned integer"))?;
            Value::UInt8(v as u8)
        }
        FieldType::Int8 => {
            let v = json_val.as_i64().ok_or_else(|| type_err("integer"))?;
            Value::Int8(v as i8)
        }
        FieldType::UInt16 => {
            let v = json_val.as_u64().ok_or_else(|| type_err("unsigned integer"))?;
            Value::UInt16(v as u16)
        }
        FieldType::Int16 => {
            let v = json_val.as_i64().ok_or_else(|| type_err("integer"))?;
            Value::Int16(v as i16)
        }
        FieldType::String => {
            let s = json_val.as_str().ok_or_else(|| type_err("string"))?;
            let offset = intern_string(s, string_data, string_offsets);
            Value::StringRef(StringRef::new(offset))
        }
    })
}

/// Intern a string into the string block, returning its offset.
///
/// If the string already exists in the block the existing offset is returned,
/// avoiding duplicates.
#[cfg(feature = "serde")]
fn intern_string(
    s: &str,
    string_data: &mut Vec<u8>,
    string_offsets: &mut HashMap<String, u32>,
) -> u32 {
    if let Some(&existing) = string_offsets.get(s) {
        return existing;
    }
    let offset = string_data.len() as u32;
    string_data.extend_from_slice(s.as_bytes());
    string_data.push(0); // null terminator
    string_offsets.insert(s.to_string(), offset);
    offset
}

/// Export a record set to CSV
#[cfg(feature = "csv_export")]
pub fn export_to_csv<W: io::Write>(record_set: &RecordSet, writer: W) -> Result<(), io::Error> {
    use csv::WriterBuilder;

    if record_set.is_empty() {
        return Ok(());
    }

    // Get field names
    let field_names = if let Some(schema) = record_set.schema() {
        schema
            .fields
            .iter()
            .map(|f| f.name.clone())
            .collect::<Vec<_>>()
    } else {
        // No schema, use numeric field names
        let record = record_set.get_record(0).unwrap();
        (0..record.len())
            .map(|i| format!("field_{i}"))
            .collect::<Vec<_>>()
    };

    // Create CSV writer
    let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(writer);

    // Write header
    csv_writer.write_record(&field_names)?;

    // Write records
    for record in record_set.records() {
        let mut row = Vec::with_capacity(field_names.len());

        for (i, _) in field_names.iter().enumerate() {
            if let Some(value) = record.get_value(i) {
                let string_value = match value {
                    Value::Int32(v) => v.to_string(),
                    Value::UInt32(v) => v.to_string(),
                    Value::Float32(v) => v.to_string(),
                    Value::StringRef(v) => {
                        record_set.get_string(*v).unwrap_or_default().to_string()
                    }
                    Value::Bool(v) => v.to_string(),
                    Value::UInt8(v) => v.to_string(),
                    Value::Int8(v) => v.to_string(),
                    Value::UInt16(v) => v.to_string(),
                    Value::Int16(v) => v.to_string(),
                    Value::Array(values) => {
                        let mut array_str = String::new();
                        for (j, val) in values.iter().enumerate() {
                            if j > 0 {
                                array_str.push('|');
                            }
                            match val {
                                Value::Int32(v) => array_str.push_str(&v.to_string()),
                                Value::UInt32(v) => array_str.push_str(&v.to_string()),
                                Value::Float32(v) => array_str.push_str(&v.to_string()),
                                Value::StringRef(v) => {
                                    array_str
                                        .push_str(record_set.get_string(*v).unwrap_or_default());
                                }
                                Value::Bool(v) => array_str.push_str(&v.to_string()),
                                Value::UInt8(v) => array_str.push_str(&v.to_string()),
                                Value::Int8(v) => array_str.push_str(&v.to_string()),
                                Value::UInt16(v) => array_str.push_str(&v.to_string()),
                                Value::Int16(v) => array_str.push_str(&v.to_string()),
                                Value::Array(_) => array_str.push_str("<nested_array>"),
                            }
                        }
                        array_str
                    }
                };

                row.push(string_value);
            } else {
                row.push(String::new());
            }
        }

        csv_writer.write_record(&row)?;
    }

    csv_writer.flush()?;
    Ok(())
}

#[cfg(test)]
#[cfg(feature = "serde")]
mod tests {
    use super::*;
    use crate::{DbcParser, DbcWriter, FieldType, Schema, SchemaField};
    use std::io::Cursor;

    fn make_test_schema() -> Schema {
        let mut schema = Schema::new("Test");
        schema.add_field(SchemaField::new("ID", FieldType::UInt32));
        schema.add_field(SchemaField::new("Name", FieldType::String));
        schema.add_field(SchemaField::new("Value", FieldType::UInt32));
        schema.set_key_field("ID");
        schema
    }

    fn make_test_dbc() -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(b"WDBC");
        data.extend_from_slice(&2u32.to_le_bytes()); // record_count
        data.extend_from_slice(&3u32.to_le_bytes()); // field_count
        data.extend_from_slice(&12u32.to_le_bytes()); // record_size
        data.extend_from_slice(&19u32.to_le_bytes()); // string_block_size
        // Record 1
        data.extend_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(&1u32.to_le_bytes()); // "First" at offset 1
        data.extend_from_slice(&100u32.to_le_bytes());
        // Record 2
        data.extend_from_slice(&2u32.to_le_bytes());
        data.extend_from_slice(&7u32.to_le_bytes()); // "Second" at offset 7
        data.extend_from_slice(&200u32.to_le_bytes());
        // String block: \0First\0Second\0Extra\0
        data.extend_from_slice(b"\x00First\x00Second\x00Extra\x00");
        data
    }

    #[test]
    fn test_json_roundtrip() {
        // Parse original DBC
        let dbc_bytes = make_test_dbc();
        let parser = DbcParser::parse_bytes(&dbc_bytes).unwrap();
        let parser = parser.with_schema(make_test_schema()).unwrap();
        let original = parser.parse_records().unwrap();

        // Export to JSON
        let mut json_buf = Vec::new();
        export_to_json(&original, &mut json_buf).unwrap();

        // Import from JSON
        let imported = import_from_json(Cursor::new(&json_buf), make_test_schema()).unwrap();

        assert_eq!(imported.len(), 2);

        let r0 = imported.get_record(0).unwrap();
        assert!(matches!(r0.get_value(0), Some(Value::UInt32(1))));
        assert!(matches!(r0.get_value(2), Some(Value::UInt32(100))));

        if let Some(Value::StringRef(sr)) = r0.get_value(1) {
            assert_eq!(imported.get_string(*sr).unwrap(), "First");
        } else {
            panic!("expected StringRef for Name field");
        }

        let r1 = imported.get_record(1).unwrap();
        assert!(matches!(r1.get_value(0), Some(Value::UInt32(2))));
        assert!(matches!(r1.get_value(2), Some(Value::UInt32(200))));

        if let Some(Value::StringRef(sr)) = r1.get_value(1) {
            assert_eq!(imported.get_string(*sr).unwrap(), "Second");
        } else {
            panic!("expected StringRef for Name field");
        }

        // Write the imported record set back to a DBC and re-parse it
        let mut out_buf: Vec<u8> = Vec::new();
        let mut writer = DbcWriter::new(Cursor::new(&mut out_buf));
        writer.write_records(&imported).unwrap();

        let parser2 = DbcParser::parse_bytes(&out_buf).unwrap();
        let parser2 = parser2.with_schema(make_test_schema()).unwrap();
        let final_set = parser2.parse_records().unwrap();

        assert_eq!(final_set.len(), 2);

        if let Some(Value::StringRef(sr)) = final_set.get_record(0).unwrap().get_value(1) {
            assert_eq!(final_set.get_string(*sr).unwrap(), "First");
        } else {
            panic!("expected StringRef after write-read cycle");
        }
    }

    #[test]
    fn test_import_missing_field_error() {
        let json = r#"[{"ID": 1, "Value": 100}]"#; // missing "Name"
        let result = import_from_json(Cursor::new(json.as_bytes()), make_test_schema());
        assert!(result.is_err());
    }

    #[test]
    fn test_import_type_mismatch_error() {
        let json = r#"[{"ID": "not_a_number", "Name": "hi", "Value": 1}]"#;
        let result = import_from_json(Cursor::new(json.as_bytes()), make_test_schema());
        assert!(result.is_err());
    }
}
