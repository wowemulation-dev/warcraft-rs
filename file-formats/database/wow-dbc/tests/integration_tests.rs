//! Integration tests for the DBC parser.

use std::io::Cursor;
use wow_dbc::{DbcHeader, DbcParser, FieldType, Schema, SchemaField, StringRef, Value};

#[test]
fn test_header_parsing() {
    // Create a test DBC file
    let data = create_test_dbc();
    let mut cursor = Cursor::new(&data);

    // Parse the header
    let header = DbcHeader::parse(&mut cursor).unwrap();

    // Check header values
    assert_eq!(header.magic, *b"WDBC");
    assert_eq!(header.record_count, 2);
    assert_eq!(header.field_count, 3);
    assert_eq!(header.record_size, 12);
    assert_eq!(header.string_block_size, 19);
}

#[test]
fn test_schema_validation() {
    // Create a test DBC file
    let data = create_test_dbc();
    let mut cursor = Cursor::new(&data);

    // Parse the DBC file
    let parser = DbcParser::parse(&mut cursor).unwrap();
    let header = parser.header();

    // Create a valid schema
    let mut schema = Schema::new("Test");
    schema.add_field(SchemaField::new("ID", FieldType::UInt32));
    schema.add_field(SchemaField::new("Name", FieldType::String));
    schema.add_field(SchemaField::new("Value", FieldType::UInt32));
    schema.set_key_field("ID");

    // Validate the schema
    assert!(
        schema
            .validate(header.field_count, header.record_size)
            .is_ok()
    );

    // Create an invalid schema (missing a field)
    let mut invalid_schema = Schema::new("Invalid");
    invalid_schema.add_field(SchemaField::new("ID", FieldType::UInt32));
    invalid_schema.add_field(SchemaField::new("Name", FieldType::String));

    // Validate the invalid schema
    assert!(
        invalid_schema
            .validate(header.field_count, header.record_size)
            .is_err()
    );
}

#[test]
fn test_record_parsing() {
    // Create a test DBC file
    let data = create_test_dbc();

    // Parse the DBC file
    let parser = DbcParser::parse_bytes(&data).unwrap();

    // Create a schema
    let mut schema = Schema::new("Test");
    schema.add_field(SchemaField::new("ID", FieldType::UInt32));
    schema.add_field(SchemaField::new("Name", FieldType::String));
    schema.add_field(SchemaField::new("Value", FieldType::UInt32));
    schema.set_key_field("ID");

    // Apply the schema and parse records
    let parser = parser.with_schema(schema).unwrap();
    let record_set = parser.parse_records().unwrap();

    // Check record count
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

    // Check second record
    let record = record_set.get_record(1).unwrap();
    if let Value::UInt32(id) = record.get_value_by_name("ID").unwrap() {
        assert_eq!(*id, 2);
    } else {
        panic!("Expected UInt32 for ID");
    }

    if let Value::StringRef(name_ref) = record.get_value_by_name("Name").unwrap() {
        let name = record_set.get_string(*name_ref).unwrap();
        assert_eq!(name, "Second");
    } else {
        panic!("Expected StringRef for Name");
    }

    if let Value::UInt32(value) = record.get_value_by_name("Value").unwrap() {
        assert_eq!(*value, 200);
    } else {
        panic!("Expected UInt32 for Value");
    }
}

#[test]
fn test_key_lookup() {
    // Create a test DBC file
    let data = create_test_dbc();

    // Parse the DBC file
    let parser = DbcParser::parse_bytes(&data).unwrap();

    // Create a schema
    let mut schema = Schema::new("Test");
    schema.add_field(SchemaField::new("ID", FieldType::UInt32));
    schema.add_field(SchemaField::new("Name", FieldType::String));
    schema.add_field(SchemaField::new("Value", FieldType::UInt32));
    schema.set_key_field("ID");

    // Apply the schema and parse records
    let parser = parser.with_schema(schema).unwrap();
    let record_set = parser.parse_records().unwrap();

    // Look up record by key
    let record = record_set.get_record_by_key(2).unwrap();
    if let Value::StringRef(name_ref) = record.get_value_by_name("Name").unwrap() {
        let name = record_set.get_string(*name_ref).unwrap();
        assert_eq!(name, "Second");
    } else {
        panic!("Expected StringRef for Name");
    }

    // Try to look up a non-existent key
    let record = record_set.get_record_by_key(999);
    assert!(record.is_none());
}

#[test]
fn test_string_block() {
    // Create a test DBC file
    let data = create_test_dbc();

    // Parse the DBC file
    let parser = DbcParser::parse_bytes(&data).unwrap();
    let record_set = parser.parse_records().unwrap();

    // Check string retrieval
    let string = record_set.get_string(StringRef::new(0)).unwrap();
    assert_eq!(string, "First");

    let string = record_set.get_string(StringRef::new(6)).unwrap();
    assert_eq!(string, "Second");

    let string = record_set.get_string(StringRef::new(13)).unwrap();
    assert_eq!(string, "Extra");

    // Try to get a string with an invalid offset
    let result = record_set.get_string(StringRef::new(999));
    assert!(result.is_err());
}

#[test]
fn test_array_fields() {
    // Create a test DBC file with array fields
    let data = create_test_dbc_with_arrays();
    let mut cursor = Cursor::new(&data);

    // Parse the DBC file
    let parser = DbcParser::parse(&mut cursor).unwrap();

    // Create a schema with array fields
    let mut schema = Schema::new("TestArray");
    schema.add_field(SchemaField::new("ID", FieldType::UInt32));
    schema.add_field(SchemaField::new_array("Values", FieldType::UInt32, 3));
    schema.set_key_field("ID");

    // Apply the schema and parse records
    let parser = parser.with_schema(schema).unwrap();
    let record_set = parser.parse_records().unwrap();

    // Check record count
    assert_eq!(record_set.len(), 2);

    // Check first record
    let record = record_set.get_record(0).unwrap();
    if let Value::UInt32(id) = record.get_value_by_name("ID").unwrap() {
        assert_eq!(*id, 1);
    } else {
        panic!("Expected UInt32 for ID");
    }

    if let Value::Array(values) = record.get_value_by_name("Values").unwrap() {
        assert_eq!(values.len(), 3);
        if let Value::UInt32(v1) = &values[0] {
            assert_eq!(*v1, 10);
        } else {
            panic!("Expected UInt32 for Values[0]");
        }
        if let Value::UInt32(v2) = &values[1] {
            assert_eq!(*v2, 20);
        } else {
            panic!("Expected UInt32 for Values[1]");
        }
        if let Value::UInt32(v3) = &values[2] {
            assert_eq!(*v3, 30);
        } else {
            panic!("Expected UInt32 for Values[2]");
        }
    } else {
        panic!("Expected Array for Values");
    }
}

// Helper function to create a test DBC file
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
    data.extend_from_slice(&6u32.to_le_bytes()); // Name offset
    data.extend_from_slice(&200u32.to_le_bytes()); // Value

    // String block
    data.extend_from_slice(b"First\0Second\0Extra\0"); // String block

    data
}

// Helper function to create a test DBC file with array fields
fn create_test_dbc_with_arrays() -> Vec<u8> {
    let mut data = Vec::new();

    // Header
    data.extend_from_slice(b"WDBC"); // Magic
    data.extend_from_slice(&2u32.to_le_bytes()); // Record count
    data.extend_from_slice(&4u32.to_le_bytes()); // Field count
    data.extend_from_slice(&16u32.to_le_bytes()); // Record size
    data.extend_from_slice(&0u32.to_le_bytes()); // String block size

    // Records
    // Record 1
    data.extend_from_slice(&1u32.to_le_bytes()); // ID
    data.extend_from_slice(&10u32.to_le_bytes()); // Values[0]
    data.extend_from_slice(&20u32.to_le_bytes()); // Values[1]
    data.extend_from_slice(&30u32.to_le_bytes()); // Values[2]

    // Record 2
    data.extend_from_slice(&2u32.to_le_bytes()); // ID
    data.extend_from_slice(&40u32.to_le_bytes()); // Values[0]
    data.extend_from_slice(&50u32.to_le_bytes()); // Values[1]
    data.extend_from_slice(&60u32.to_le_bytes()); // Values[2]

    // Empty string block

    data
}
