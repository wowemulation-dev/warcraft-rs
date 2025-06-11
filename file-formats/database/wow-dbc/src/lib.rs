//! # wow_dbc
//!
//! A library for parsing World of Warcraft DBC (Database Client) files.
//!
//! ## Features
//!
//! - Parse DBC files from World of Warcraft
//! - Support for different DBC versions (WDBC, WDB2, WDB5, etc.)
//! - Schema-based parsing with validation
//! - Export to JSON and CSV formats
//! - Command-line interface for working with DBC files
//!
//! ## Example
//!
//! ```no_run
//! use std::fs::File;
//! use std::io::BufReader;
//! use wow_dbc::{DbcParser, FieldType, Schema, SchemaField, Value};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Open a DBC file
//!     let file = File::open("SpellItemEnchantment.dbc")?;
//!     let mut reader = BufReader::new(file);
//!
//!     // Parse the DBC file
//!     let parser = DbcParser::parse(&mut reader)?;
//!
//!     // Print header information
//!     let header = parser.header();
//!     println!("Record Count: {}", header.record_count);
//!     println!("Field Count: {}", header.field_count);
//!
//!     // Define a schema for SpellItemEnchantment.dbc
//!     let mut schema = Schema::new("SpellItemEnchantment");
//!     schema.add_field(SchemaField::new("ID", FieldType::UInt32));
//!     schema.add_field(SchemaField::new("Charges", FieldType::UInt32));
//!     schema.add_field(SchemaField::new("Type1", FieldType::UInt32));
//!     schema.add_field(SchemaField::new("Type2", FieldType::UInt32));
//!     schema.add_field(SchemaField::new("Type3", FieldType::UInt32));
//!     schema.add_field(SchemaField::new("Amount1", FieldType::Int32));
//!     schema.add_field(SchemaField::new("Amount2", FieldType::Int32));
//!     schema.add_field(SchemaField::new("Amount3", FieldType::Int32));
//!     schema.add_field(SchemaField::new("ItemID", FieldType::UInt32));
//!     schema.add_field(SchemaField::new("Description", FieldType::String));
//!     schema.set_key_field("ID");
//!
//!     // Apply the schema and parse records
//!     let parser = parser.with_schema(schema)?;
//!     let record_set = parser.parse_records()?;
//!
//!     // Access records
//!     if let Some(record) = record_set.get_record(0) {
//!         if let Value::UInt32(id) = record.get_value_by_name("ID").unwrap() {
//!             println!("ID: {}", id);
//!         }
//!
//!         if let Value::StringRef(desc_ref) = record.get_value_by_name("Description").unwrap() {
//!             let desc = record_set.get_string(*desc_ref)?;
//!             println!("Description: {}", desc);
//!         }
//!     }
//!
//!     // Look up a record by key
//!     if let Some(record) = record_set.get_record_by_key(123) {
//!         // Work with the record...
//!     }
//!
//!     Ok(())
//! }
//! ```

mod error;
#[cfg(any(feature = "serde", feature = "csv_export"))]
mod export;
mod field_parser;
mod header;
mod parser;
mod schema;
mod schema_discovery;
mod schema_loader;
mod stringblock;
mod types;
mod versions;
mod writer;

#[cfg(feature = "mmap")]
mod mmap;

mod lazy;

#[cfg(feature = "parallel")]
mod parallel;

#[cfg(feature = "cli")]
pub mod dbd;

pub use error::Error;
pub use header::DbcHeader;
pub use lazy::{LazyDbcParser, LazyRecordIterator};
pub use parser::{DbcParser, Record, RecordSet, Value};
pub use schema::{FieldType, Schema, SchemaField};
pub use schema_discovery::{Confidence, DiscoveredField, DiscoveredSchema, SchemaDiscoverer};
pub use stringblock::{CachedStringBlock, StringBlock};
pub use types::*;

#[cfg(feature = "yaml")]
pub use schema_loader::{SchemaDefinition, SchemaFieldDefinition};

#[cfg(feature = "serde")]
pub use export::export_to_json;

#[cfg(feature = "csv_export")]
pub use export::export_to_csv;

#[cfg(feature = "mmap")]
pub use mmap::MmapDbcFile;

#[cfg(feature = "parallel")]
pub use parallel::parse_records_parallel;

pub use versions::{DbcVersion, Wdb2Header, Wdb5Header};
pub use writer::DbcWriter;

/// Result type used throughout the library
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn create_test_dbc() -> Vec<u8> {
        let mut data = Vec::new();

        // Header
        data.extend_from_slice(b"WDBC"); // Magic
        data.extend_from_slice(&2u32.to_le_bytes()); // Record count
        data.extend_from_slice(&3u32.to_le_bytes()); // Field count
        data.extend_from_slice(&12u32.to_le_bytes()); // Record size
        data.extend_from_slice(&19u32.to_le_bytes()); // String block size

        // Records
        // Record 1
        data.extend_from_slice(&1u32.to_le_bytes()); // ID
        data.extend_from_slice(&0u32.to_le_bytes()); // Name offset
        data.extend_from_slice(&100u32.to_le_bytes()); // Value

        // Record 2
        data.extend_from_slice(&2u32.to_le_bytes()); // ID
        data.extend_from_slice(&6u32.to_le_bytes()); // Name offset (Second starts at 6)
        data.extend_from_slice(&200u32.to_le_bytes()); // Value

        // String block
        data.extend_from_slice(b"First\0Second\0Extra\0"); // String block

        data
    }

    #[test]
    fn test_header_parsing() {
        let data = create_test_dbc();
        let mut cursor = Cursor::new(&data);

        let header = DbcHeader::parse(&mut cursor).unwrap();

        assert_eq!(header.magic, *b"WDBC");
        assert_eq!(header.record_count, 2);
        assert_eq!(header.field_count, 3);
        assert_eq!(header.record_size, 12);
        assert_eq!(header.string_block_size, 19);
    }

    #[test]
    fn test_schema_validation() {
        let data = create_test_dbc();
        let mut cursor = Cursor::new(&data);

        let parser = DbcParser::parse(&mut cursor).unwrap();
        let header = parser.header();

        let mut schema = Schema::new("Test");
        schema.add_field(SchemaField::new("ID", FieldType::UInt32));
        schema.add_field(SchemaField::new("Name", FieldType::String));
        schema.add_field(SchemaField::new("Value", FieldType::UInt32));
        schema.set_key_field("ID");

        // This should pass
        assert!(
            schema
                .validate(header.field_count, header.record_size)
                .is_ok()
        );

        // Now let's test some invalid schemas
        let mut invalid_schema = Schema::new("Invalid");
        invalid_schema.add_field(SchemaField::new("ID", FieldType::UInt32));
        invalid_schema.add_field(SchemaField::new("Name", FieldType::String));
        // Missing a field

        assert!(
            invalid_schema
                .validate(header.field_count, header.record_size)
                .is_err()
        );
    }

    #[test]
    fn test_value_display() {
        use crate::Value;

        // Test various value types
        assert_eq!(format!("{}", Value::Int32(42)), "42");
        assert_eq!(format!("{}", Value::UInt32(100)), "100");
        assert_eq!(format!("{}", Value::Float32(3.5)), "3.5");
        assert_eq!(format!("{}", Value::Bool(true)), "true");
        assert_eq!(format!("{}", Value::Bool(false)), "false");
        assert_eq!(format!("{}", Value::UInt8(255)), "255");
        assert_eq!(format!("{}", Value::Int8(-128)), "-128");
        assert_eq!(format!("{}", Value::UInt16(65535)), "65535");
        assert_eq!(format!("{}", Value::Int16(-32768)), "-32768");
        assert_eq!(
            format!("{}", Value::StringRef(StringRef::new(42))),
            "StringRef(42)"
        );

        // Test array display
        let array = Value::Array(vec![Value::Int32(1), Value::Int32(2), Value::Int32(3)]);
        assert_eq!(format!("{}", array), "[1, 2, 3]");

        // Test empty array
        let empty_array = Value::Array(vec![]);
        assert_eq!(format!("{}", empty_array), "[]");
    }

    #[test]
    fn test_record_parsing() {
        let data = create_test_dbc();

        let parser = DbcParser::parse_bytes(&data).unwrap();

        let mut schema = Schema::new("Test");
        schema.add_field(SchemaField::new("ID", FieldType::UInt32));
        schema.add_field(SchemaField::new("Name", FieldType::String));
        schema.add_field(SchemaField::new("Value", FieldType::UInt32));
        schema.set_key_field("ID");

        let parser = parser.with_schema(schema).unwrap();
        let record_set = parser.parse_records().unwrap();

        assert_eq!(record_set.len(), 2);

        // Check first record
        let record = record_set.get_record(0).unwrap();
        if let Value::UInt32(id) = record.get_value_by_name("ID").unwrap() {
            assert_eq!(*id, 1);
        } else {
            panic!("Expected UInt32 for ID");
        }

        if let Value::StringRef(name_ref) = record.get_value_by_name("Name").unwrap() {
            let name = record_set.get_string(*name_ref).unwrap();
            assert_eq!(name, "First");
        } else {
            panic!("Expected StringRef for Name");
        }

        if let Value::UInt32(value) = record.get_value_by_name("Value").unwrap() {
            assert_eq!(*value, 100);
        } else {
            panic!("Expected UInt32 for Value");
        }

        // Check record by key
        let record = record_set.get_record_by_key(2).unwrap();
        if let Value::StringRef(name_ref) = record.get_value_by_name("Name").unwrap() {
            let name = record_set.get_string(*name_ref).unwrap();
            assert_eq!(name, "Second");
        } else {
            panic!("Expected StringRef for Name");
        }
    }
}
