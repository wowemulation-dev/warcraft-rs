//! Lazy loading functionality for DBC files

use crate::{DbcHeader, Error, FieldType, Record, Result, Schema, StringBlock, Value};
use std::io::{Cursor, Read};
use std::sync::Arc;

/// A record iterator that loads records on-demand
pub struct LazyRecordIterator<'a> {
    /// The cursor for reading records
    cursor: Cursor<&'a [u8]>,
    /// The DBC header
    header: &'a DbcHeader,
    /// The schema, if any
    schema: Option<&'a Schema>,
    /// The current record index
    current_index: u32,
    /// The total number of records
    total_records: u32,
}

impl<'a> LazyRecordIterator<'a> {
    /// Create a new lazy record iterator
    pub fn new(
        data: &'a [u8],
        header: &'a DbcHeader,
        schema: Option<&'a Schema>,
        _string_block: Arc<StringBlock>,
    ) -> Self {
        let mut cursor = Cursor::new(data);
        cursor.set_position(DbcHeader::SIZE as u64);

        Self {
            cursor,
            header,
            schema,
            current_index: 0,
            total_records: header.record_count,
        }
    }
}

impl Iterator for LazyRecordIterator<'_> {
    type Item = Result<Record>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.total_records {
            return None;
        }

        let record = if let Some(schema) = self.schema {
            self.parse_record_with_schema(schema)
        } else {
            self.parse_record_raw()
        };

        self.current_index += 1;
        Some(record)
    }
}

impl LazyRecordIterator<'_> {
    /// Parse a record using a schema
    fn parse_record_with_schema(&mut self, schema: &Schema) -> Result<Record> {
        let mut values = Vec::with_capacity(schema.fields.len());

        for field in &schema.fields {
            let value = if field.is_array {
                let array_size = field.array_size.unwrap_or(0);
                let mut array_values = Vec::with_capacity(array_size);

                for _ in 0..array_size {
                    array_values.push(self.parse_field_value(field.field_type)?);
                }

                Value::Array(array_values)
            } else {
                self.parse_field_value(field.field_type)?
            };

            values.push(value);
        }

        Ok(Record::new(values, Some(Arc::new(schema.clone()))))
    }

    /// Parse a record without a schema
    fn parse_record_raw(&mut self) -> Result<Record> {
        let mut values = Vec::with_capacity(self.header.field_count as usize);

        for _ in 0..self.header.field_count {
            // Without a schema, we assume all fields are 32-bit integers
            let mut buf = [0u8; 4];
            self.cursor.read_exact(&mut buf)?;
            let value = u32::from_le_bytes(buf);
            values.push(Value::UInt32(value));
        }

        Ok(Record::new(values, None))
    }

    /// Parse a field value based on its type
    fn parse_field_value(&mut self, field_type: FieldType) -> Result<Value> {
        crate::field_parser::parse_field_value(&mut self.cursor, field_type)
    }
}

/// A lazy-loading DBC parser
pub struct LazyDbcParser<'a> {
    /// The DBC header
    header: &'a DbcHeader,
    /// The schema, if any
    schema: Option<&'a Schema>,
    /// The raw data of the DBC file
    data: &'a [u8],
    /// The string block
    string_block: Arc<StringBlock>,
}

impl<'a> LazyDbcParser<'a> {
    /// Create a new lazy DBC parser
    pub fn new(
        data: &'a [u8],
        header: &'a DbcHeader,
        schema: Option<&'a Schema>,
        string_block: Arc<StringBlock>,
    ) -> Self {
        Self {
            data,
            header,
            schema,
            string_block,
        }
    }

    /// Get a lazy record iterator
    pub fn record_iterator(&self) -> LazyRecordIterator<'a> {
        LazyRecordIterator::new(
            self.data,
            self.header,
            self.schema,
            Arc::clone(&self.string_block),
        )
    }

    /// Get a record by index
    pub fn get_record(&self, index: u32) -> Result<Record> {
        if index >= self.header.record_count {
            return Err(Error::OutOfBounds(format!(
                "Record index out of bounds: {} (max: {})",
                index,
                self.header.record_count - 1
            )));
        }

        let mut cursor = Cursor::new(self.data);
        let record_position =
            DbcHeader::SIZE as u64 + (index as u64 * self.header.record_size as u64);
        cursor.set_position(record_position);

        if let Some(schema) = self.schema {
            self.parse_record_with_schema(&mut cursor, schema)
        } else {
            self.parse_record_raw(&mut cursor)
        }
    }

    /// Parse a record using a schema
    fn parse_record_with_schema(
        &self,
        cursor: &mut Cursor<&'a [u8]>,
        schema: &Schema,
    ) -> Result<Record> {
        let mut values = Vec::with_capacity(schema.fields.len());

        for field in &schema.fields {
            let value = if field.is_array {
                let array_size = field.array_size.unwrap_or(0);
                let mut array_values = Vec::with_capacity(array_size);

                for _ in 0..array_size {
                    array_values.push(self.parse_field_value(cursor, field.field_type)?);
                }

                Value::Array(array_values)
            } else {
                self.parse_field_value(cursor, field.field_type)?
            };

            values.push(value);
        }

        Ok(Record::new(values, Some(Arc::new(schema.clone()))))
    }

    /// Parse a record without a schema
    fn parse_record_raw(&self, cursor: &mut Cursor<&'a [u8]>) -> Result<Record> {
        let mut values = Vec::with_capacity(self.header.field_count as usize);

        for _ in 0..self.header.field_count {
            // Without a schema, we assume all fields are 32-bit integers
            let mut buf = [0u8; 4];
            cursor.read_exact(&mut buf)?;
            let value = u32::from_le_bytes(buf);
            values.push(Value::UInt32(value));
        }

        Ok(Record::new(values, None))
    }

    /// Parse a field value based on its type
    fn parse_field_value(
        &self,
        cursor: &mut Cursor<&'a [u8]>,
        field_type: FieldType,
    ) -> Result<Value> {
        crate::field_parser::parse_field_value(cursor, field_type)
    }

    /// Get the DBC header
    pub fn header(&self) -> &DbcHeader {
        self.header
    }

    /// Get the schema, if any
    pub fn schema(&self) -> Option<&Schema> {
        self.schema
    }

    /// Get the string block
    pub fn string_block(&self) -> &StringBlock {
        &self.string_block
    }
}
