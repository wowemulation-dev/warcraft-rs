//! DBC file writing functionality

use crate::{Error, FieldType, Record, RecordSet, Result, Schema, Value};
use std::collections::HashMap;
use std::io::{Seek, SeekFrom, Write};

/// Writer for DBC files
#[derive(Debug)]
pub struct DbcWriter<W: Write + Seek> {
    /// The output writer
    writer: W,
    /// The schema to use for writing
    schema: Option<Schema>,
}

impl<W: Write + Seek> DbcWriter<W> {
    /// Create a new DBC writer
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            schema: None,
        }
    }

    /// Set the schema for the writer
    pub fn with_schema(mut self, schema: Schema) -> Self {
        self.schema = Some(schema);
        self
    }

    /// Write a record set to the output
    pub fn write_records(&mut self, record_set: &RecordSet) -> Result<()> {
        // Ensure we have a schema
        let schema = if let Some(schema) = self.schema.as_ref() {
            schema.clone()
        } else if let Some(schema) = record_set.schema() {
            schema.clone()
        } else {
            return Err(Error::InvalidRecord(
                "No schema provided for writing records".to_string(),
            ));
        };

        // Build the string block
        let (string_block, string_offsets) = self.build_string_block(record_set)?;

        // Calculate header values
        let record_count = record_set.len() as u32;
        let field_count = schema.fields.len() as u32;
        let record_size = schema.record_size() as u32;
        let string_block_size = string_block.len() as u32;

        // Write header
        self.writer.seek(SeekFrom::Start(0))?;
        self.writer.write_all(&crate::header::DBC_MAGIC)?;
        self.writer.write_all(&record_count.to_le_bytes())?;
        self.writer.write_all(&field_count.to_le_bytes())?;
        self.writer.write_all(&record_size.to_le_bytes())?;
        self.writer.write_all(&string_block_size.to_le_bytes())?;

        // Write records
        for record in record_set.records() {
            self.write_record(record, &schema, record_set, &string_offsets)?;
        }

        // Write string block
        self.writer.write_all(&string_block)?;

        Ok(())
    }

    /// Build a string block from a record set
    fn build_string_block(
        &self,
        record_set: &RecordSet,
    ) -> Result<(Vec<u8>, HashMap<String, u32>)> {
        let mut string_block = Vec::new();
        let mut string_offsets = HashMap::new();

        // First string is always empty
        string_block.push(0);
        string_offsets.insert(String::new(), 0);

        // Add all strings from the record set
        for record in record_set.records() {
            for value in record.values() {
                if let Value::StringRef(string_ref) = value {
                    let string = record_set.get_string(*string_ref)?;

                    if !string_offsets.contains_key(string) {
                        let offset = string_block.len() as u32;
                        string_offsets.insert(string.to_string(), offset);

                        // Add the string to the block
                        string_block.extend_from_slice(string.as_bytes());
                        string_block.push(0); // Null terminator
                    }
                }
            }
        }

        Ok((string_block, string_offsets))
    }

    /// Write a record to the output
    fn write_record(
        &mut self,
        record: &Record,
        schema: &Schema,
        record_set: &RecordSet,
        string_offsets: &HashMap<String, u32>,
    ) -> Result<()> {
        for (i, field) in schema.fields.iter().enumerate() {
            if let Some(value) = record.get_value(i) {
                self.write_value(value, field.field_type, record_set, string_offsets)?;
            } else {
                // Write default value for the field type
                match field.field_type {
                    FieldType::Int32 => self.writer.write_all(&0i32.to_le_bytes())?,
                    FieldType::UInt32 => self.writer.write_all(&0u32.to_le_bytes())?,
                    FieldType::Float32 => self.writer.write_all(&0.0f32.to_le_bytes())?,
                    FieldType::String => self.writer.write_all(&0u32.to_le_bytes())?,
                    FieldType::Bool => self.writer.write_all(&0u32.to_le_bytes())?,
                    FieldType::UInt8 => self.writer.write_all(&[0u8])?,
                    FieldType::Int8 => self.writer.write_all(&[0u8])?,
                    FieldType::UInt16 => self.writer.write_all(&0u16.to_le_bytes())?,
                    FieldType::Int16 => self.writer.write_all(&0i16.to_le_bytes())?,
                }
            }
        }

        Ok(())
    }

    /// Write a value to the output
    fn write_value(
        &mut self,
        value: &Value,
        field_type: FieldType,
        record_set: &RecordSet,
        string_offsets: &HashMap<String, u32>,
    ) -> Result<()> {
        match (value, field_type) {
            (Value::Int32(v), FieldType::Int32) => self.writer.write_all(&v.to_le_bytes())?,
            (Value::UInt32(v), FieldType::UInt32) => self.writer.write_all(&v.to_le_bytes())?,
            (Value::Float32(v), FieldType::Float32) => self.writer.write_all(&v.to_le_bytes())?,
            (Value::StringRef(v), FieldType::String) => {
                let string = record_set.get_string(*v)?;
                let offset = string_offsets.get(string).unwrap_or(&0);
                self.writer.write_all(&offset.to_le_bytes())?;
            }
            (Value::Bool(v), FieldType::Bool) => self
                .writer
                .write_all(&(if *v { 1u32 } else { 0u32 }).to_le_bytes())?,
            (Value::UInt8(v), FieldType::UInt8) => self.writer.write_all(&[*v])?,
            (Value::Int8(v), FieldType::Int8) => self.writer.write_all(&[*v as u8])?,
            (Value::UInt16(v), FieldType::UInt16) => self.writer.write_all(&v.to_le_bytes())?,
            (Value::Int16(v), FieldType::Int16) => self.writer.write_all(&v.to_le_bytes())?,
            (Value::Array(vals), _) => {
                // Write each value in the array
                for val in vals {
                    self.write_value(val, field_type, record_set, string_offsets)?;
                }
            }
            _ => {
                return Err(Error::TypeConversion(format!(
                    "Type mismatch: {value:?} is not compatible with {field_type:?}"
                )));
            }
        }

        Ok(())
    }
}
