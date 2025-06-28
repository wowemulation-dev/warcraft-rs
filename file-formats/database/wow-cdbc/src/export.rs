//! Export functionality for DBC data

use crate::{Record, RecordSet, Value};
#[cfg(feature = "serde")]
use serde::Serialize;
use std::collections::HashMap;
use std::io;

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
