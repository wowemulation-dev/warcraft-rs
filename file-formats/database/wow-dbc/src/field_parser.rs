//! Common field parsing functionality shared across modules

use crate::{FieldType, Result, StringRef, Value};
use std::io::Read;

/// Parse a field value based on its type
pub fn parse_field_value<R: Read>(reader: &mut R, field_type: FieldType) -> Result<Value> {
    match field_type {
        FieldType::Int32 => {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf)?;
            Ok(Value::Int32(i32::from_le_bytes(buf)))
        }
        FieldType::UInt32 => {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf)?;
            Ok(Value::UInt32(u32::from_le_bytes(buf)))
        }
        FieldType::Float32 => {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf)?;
            Ok(Value::Float32(f32::from_le_bytes(buf)))
        }
        FieldType::String => {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf)?;
            let offset = u32::from_le_bytes(buf);
            Ok(Value::StringRef(StringRef::new(offset)))
        }
        FieldType::Bool => {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf)?;
            let value = u32::from_le_bytes(buf);
            Ok(Value::Bool(value != 0))
        }
        FieldType::UInt8 => {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf)?;
            Ok(Value::UInt8(buf[0]))
        }
        FieldType::Int8 => {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf)?;
            Ok(Value::Int8(buf[0] as i8))
        }
        FieldType::UInt16 => {
            let mut buf = [0u8; 2];
            reader.read_exact(&mut buf)?;
            Ok(Value::UInt16(u16::from_le_bytes(buf)))
        }
        FieldType::Int16 => {
            let mut buf = [0u8; 2];
            reader.read_exact(&mut buf)?;
            Ok(Value::Int16(i16::from_le_bytes(buf)))
        }
    }
}
