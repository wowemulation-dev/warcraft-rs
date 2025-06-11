//! Parallel processing for DBC files

use crate::{DbcHeader, FieldType, Record, RecordSet, Result, Schema, StringBlock, Value};
use rayon::prelude::*;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex};

/// Parse records in parallel
pub fn parse_records_parallel(
    data: &[u8],
    header: &DbcHeader,
    schema: Option<&Schema>,
    string_block: Arc<StringBlock>,
) -> Result<RecordSet> {
    // Create a vector to hold the records
    let records: Arc<Mutex<Vec<Option<Record>>>> =
        Arc::new(Mutex::new(vec![None; header.record_count as usize]));

    // Define the chunk size based on the number of records
    let chunk_size = std::cmp::max(
        1,
        header.record_count as usize / rayon::current_num_threads(),
    );

    // Process in parallel
    (0..header.record_count as usize)
        .collect::<Vec<_>>()
        .par_chunks(chunk_size)
        .try_for_each(|chunk| -> Result<()> {
            let mut cursor = Cursor::new(data);

            for &index in chunk {
                // Seek to the position of the record
                let record_position =
                    DbcHeader::SIZE as u64 + (index as u64 * header.record_size as u64);
                cursor.seek(SeekFrom::Start(record_position))?;

                // Parse the record
                let record = if let Some(schema) = schema {
                    parse_record_with_schema(&mut cursor, schema, header)?
                } else {
                    parse_record_raw(&mut cursor, header)?
                };

                // Store the record
                records.lock().unwrap()[index] = Some(record);
            }

            Ok(())
        })?;

    // Convert from Vec<Option<Record>> to Vec<Record>
    let records = records
        .lock()
        .unwrap()
        .iter()
        .map(|r| r.clone().unwrap())
        .collect();

    Ok(RecordSet::new(
        records,
        schema.map(|s| Arc::new(s.clone())),
        (*string_block).clone(),
    ))
}

/// Parse a record with a schema in parallel
fn parse_record_with_schema<R: Read + Seek>(
    cursor: &mut R,
    schema: &Schema,
    _header: &DbcHeader,
) -> Result<Record> {
    let mut values = Vec::with_capacity(schema.fields.len());

    for field in &schema.fields {
        let value = if field.is_array {
            let array_size = field.array_size.unwrap_or(0);
            let mut array_values = Vec::with_capacity(array_size);

            for _ in 0..array_size {
                array_values.push(parse_field_value(cursor, field.field_type)?);
            }

            Value::Array(array_values)
        } else {
            parse_field_value(cursor, field.field_type)?
        };

        values.push(value);
    }

    Ok(Record::new(values, Some(Arc::new(schema.clone()))))
}

/// Parse a record without a schema in parallel
fn parse_record_raw<R: Read + Seek>(cursor: &mut R, header: &DbcHeader) -> Result<Record> {
    let mut values = Vec::with_capacity(header.field_count as usize);

    for _ in 0..header.field_count {
        // Without a schema, we assume all fields are 32-bit integers
        let mut buf = [0u8; 4];
        cursor.read_exact(&mut buf)?;
        let value = u32::from_le_bytes(buf);
        values.push(Value::UInt32(value));
    }

    Ok(Record::new(values, None))
}

/// Parse a field value based on its type in parallel
fn parse_field_value<R: Read + Seek>(cursor: &mut R, field_type: FieldType) -> Result<Value> {
    crate::field_parser::parse_field_value(cursor, field_type)
}
