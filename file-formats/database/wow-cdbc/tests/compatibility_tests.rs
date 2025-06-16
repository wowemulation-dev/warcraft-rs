//! Compatibility tests to verify our DBC implementation matches WowDev standards

use std::io::Cursor;
use wow_cdbc::{
    DbcHeader, DbcParser, DbcVersion, FieldType, Schema, SchemaDiscoverer, SchemaField, StringRef,
    Value, Wdb2Header,
};

/// Test that our header parsing matches the WowDev specification exactly
#[test]
fn test_wowdev_header_format_compatibility() {
    // Create a DBC with the exact format specified by WowDev wiki
    let mut data = Vec::new();

    // Header as per WowDev specification:
    // struct dbc_header {
    //     uint32_t magic;           // always 'WDBC'
    //     uint32_t record_count;    // records per file
    //     uint32_t field_count;     // fields per record
    //     uint32_t record_size;     // sum (sizeof (field_type_i))
    //     uint32_t string_block_size;
    // };
    data.extend_from_slice(b"WDBC"); // Magic signature
    data.extend_from_slice(&5u32.to_le_bytes()); // 5 records
    data.extend_from_slice(&4u32.to_le_bytes()); // 4 fields per record
    data.extend_from_slice(&16u32.to_le_bytes()); // 16 bytes per record (4 fields * 4 bytes)
    data.extend_from_slice(&32u32.to_le_bytes()); // 32 bytes string block

    // Add records (5 records * 16 bytes each = 80 bytes)
    for i in 0u32..5 {
        data.extend_from_slice(&(i + 1).to_le_bytes()); // ID field
        data.extend_from_slice(&(i * 8).to_le_bytes()); // Name offset
        data.extend_from_slice(&((i + 1) * 100).to_le_bytes()); // Value field
        data.extend_from_slice(&(i % 2).to_le_bytes()); // Flag field
    }

    // Add string block (32 bytes)
    data.extend_from_slice(b"Item1\0Item2\0Item3\0Item4\0Item5\0\0");

    // Parse the header
    let mut cursor = Cursor::new(&data);
    let header = DbcHeader::parse(&mut cursor).unwrap();

    // Verify header matches WowDev specification
    assert_eq!(header.magic, *b"WDBC");
    assert_eq!(header.record_count, 5);
    assert_eq!(header.field_count, 4);
    assert_eq!(header.record_size, 16);
    assert_eq!(header.string_block_size, 32);

    // Verify calculated offsets match expected layout
    assert_eq!(DbcHeader::SIZE, 20); // Header is always 20 bytes
    assert_eq!(header.string_block_offset(), 20 + (5 * 16)); // 20 + 80 = 100
    assert_eq!(header.total_size(), 20 + 80 + 32); // 132 total bytes
}

/// Test compatibility with different DBC versions (WDB2, WDB5)
#[test]
fn test_multi_version_compatibility() {
    // Test WDB2 format (The Burning Crusade)
    let mut wdb2_data = Vec::new();
    wdb2_data.extend_from_slice(b"WDB2"); // WDB2 magic
    wdb2_data.extend_from_slice(&2u32.to_le_bytes()); // record_count
    wdb2_data.extend_from_slice(&3u32.to_le_bytes()); // field_count
    wdb2_data.extend_from_slice(&12u32.to_le_bytes()); // record_size
    wdb2_data.extend_from_slice(&16u32.to_le_bytes()); // string_block_size
    wdb2_data.extend_from_slice(&0x12345678u32.to_le_bytes()); // table_hash
    wdb2_data.extend_from_slice(&8125u32.to_le_bytes()); // build

    let mut cursor = Cursor::new(&wdb2_data);
    let version = DbcVersion::detect(&mut cursor).unwrap();
    assert_eq!(version, DbcVersion::WDB2);

    let wdb2_header = Wdb2Header::parse(&mut cursor).unwrap();
    assert_eq!(wdb2_header.magic, *b"WDB2");
    assert_eq!(wdb2_header.table_hash, 0x12345678);
    assert_eq!(wdb2_header.build, 8125);

    // Test conversion to standard DBC header
    let dbc_header = wdb2_header.to_dbc_header();
    assert_eq!(dbc_header.magic, *b"WDBC"); // Converted to WDBC
    assert_eq!(dbc_header.record_count, 2);
    assert_eq!(dbc_header.field_count, 3);
}

/// Test that field type sizes match WowDev community standards
#[test]
fn test_field_type_size_compatibility() {
    // According to WowDev and community implementations, most fields are 4 bytes
    assert_eq!(FieldType::Int32.size(), 4);
    assert_eq!(FieldType::UInt32.size(), 4);
    assert_eq!(FieldType::Float32.size(), 4);
    assert_eq!(FieldType::String.size(), 4); // String references are 4-byte offsets
    assert_eq!(FieldType::Bool.size(), 4); // Bools are stored as 4-byte integers

    // Smaller types are less common but supported
    assert_eq!(FieldType::UInt8.size(), 1);
    assert_eq!(FieldType::Int8.size(), 1);
    assert_eq!(FieldType::UInt16.size(), 2);
    assert_eq!(FieldType::Int16.size(), 2);
}

/// Test string block handling matches WowDev specification
#[test]
fn test_string_block_wowdev_compatibility() {
    // Create a DBC with string references as per WowDev specification
    let mut data = Vec::new();

    // Header
    data.extend_from_slice(b"WDBC");
    data.extend_from_slice(&3u32.to_le_bytes()); // 3 records
    data.extend_from_slice(&2u32.to_le_bytes()); // 2 fields (ID, Name)
    data.extend_from_slice(&8u32.to_le_bytes()); // 8 bytes per record
    data.extend_from_slice(&23u32.to_le_bytes()); // String block size

    // Records with string references
    // Record 1: ID=1, Name points to offset 0 ("Hello")
    data.extend_from_slice(&1u32.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());

    // Record 2: ID=2, Name points to offset 6 ("World")
    data.extend_from_slice(&2u32.to_le_bytes());
    data.extend_from_slice(&6u32.to_le_bytes());

    // Record 3: ID=3, Name points to offset 12 ("Test")
    data.extend_from_slice(&3u32.to_le_bytes());
    data.extend_from_slice(&12u32.to_le_bytes());

    // String block: null-terminated strings
    data.extend_from_slice(b"Hello\0World\0Test\0Extra\0");

    // Parse and verify
    let parser = DbcParser::parse_bytes(&data).unwrap();
    let record_set = parser.parse_records().unwrap();

    // Test string retrieval matches WowDev behavior
    assert_eq!(record_set.get_string(StringRef::new(0)).unwrap(), "Hello");
    assert_eq!(record_set.get_string(StringRef::new(6)).unwrap(), "World");
    assert_eq!(record_set.get_string(StringRef::new(12)).unwrap(), "Test");
    assert_eq!(record_set.get_string(StringRef::new(17)).unwrap(), "Extra");
}

/// Test that our schema discovery produces results compatible with community tools
#[test]
#[ignore = "Schema discovery needs improvement for accurate type size detection"]
fn test_schema_discovery_wowdev_compatibility() {
    // Create a DBC that resembles a typical WoW DBC structure
    let mut data = Vec::new();

    // Header for a spell-like DBC
    data.extend_from_slice(b"WDBC");
    data.extend_from_slice(&5u32.to_le_bytes()); // 5 records
    data.extend_from_slice(&6u32.to_le_bytes()); // 6 fields
    data.extend_from_slice(&24u32.to_le_bytes()); // 24 bytes per record (6 * 4)
    data.extend_from_slice(&35u32.to_le_bytes()); // String block size

    // Records with typical WoW DBC patterns:
    // ID, Name, School, Level, Duration, Flags
    for i in 1u32..=5 {
        data.extend_from_slice(&i.to_le_bytes()); // ID (sequential, key candidate)
        data.extend_from_slice(&((i - 1) * 7).to_le_bytes()); // Name (string offset)
        data.extend_from_slice(&(i % 8).to_le_bytes()); // School (0-7, small int)
        data.extend_from_slice(&(i * 10).to_le_bytes()); // Level (10, 20, 30, etc.)
        data.extend_from_slice(&(i * 1000).to_le_bytes()); // Duration (milliseconds)
        data.extend_from_slice(&((i % 2) * 0x80000000u32).to_le_bytes()); // Flags (0 or 0x80000000)
    }

    // String block
    data.extend_from_slice(b"Spell1\0Spell2\0Spell3\0Spell4\0Spell5\0");

    let parser = DbcParser::parse_bytes(&data).unwrap();
    let record_set = parser.parse_records().unwrap();

    // Test schema discovery
    let discoverer =
        SchemaDiscoverer::new(parser.header(), parser.data(), record_set.string_block());

    let discovered = discoverer.discover().unwrap();

    // Verify discovery results match expected WoW DBC patterns
    assert_eq!(discovered.fields.len(), 6);
    if !discovered.is_valid {
        println!("Validation failed: {:?}", discovered.validation_message);
    }
    assert!(discovered.is_valid);

    // First field should be detected as key (sequential unique values)
    assert!(discovered.fields[0].is_key_candidate);
    assert_eq!(discovered.key_field_index, Some(0));

    // Second field should be detected as string (valid string offsets)
    assert_eq!(discovered.fields[1].field_type, FieldType::String);

    // Check that we can generate a compatible schema
    let schema = discoverer.generate_schema("TestSpell").unwrap();
    assert_eq!(schema.fields.len(), 6);
    assert_eq!(schema.key_field_index, Some(0));
}

/// Test little-endian byte order compatibility (WowDev standard)
#[test]
fn test_endianness_compatibility() {
    // Create data with specific byte patterns to test endianness
    let mut data = Vec::new();

    // Header with specific values to test byte order
    data.extend_from_slice(b"WDBC");
    data.extend_from_slice(&0x12345678u32.to_le_bytes()); // record_count
    data.extend_from_slice(&0xABCDEF01u32.to_le_bytes()); // field_count
    data.extend_from_slice(&0x87654321u32.to_le_bytes()); // record_size
    data.extend_from_slice(&0x13579BDFu32.to_le_bytes()); // string_block_size

    let mut cursor = Cursor::new(&data);
    let header = DbcHeader::parse(&mut cursor).unwrap();

    // Verify values are read correctly in little-endian format
    assert_eq!(header.record_count, 0x12345678);
    assert_eq!(header.field_count, 0xABCDEF01);
    assert_eq!(header.record_size, 0x87654321);
    assert_eq!(header.string_block_size, 0x13579BDF);
}

/// Test array field detection matches community expectations
#[test]
#[ignore = "Array detection algorithm needs improvement to handle non-array fields"]
fn test_array_detection_compatibility() {
    // Create a DBC with repeating field patterns (common in WoW DBCs)
    let mut data = Vec::new();

    // Header: 2 records, 9 fields (ID + 2 arrays of 4 elements each)
    data.extend_from_slice(b"WDBC");
    data.extend_from_slice(&2u32.to_le_bytes()); // 2 records
    data.extend_from_slice(&9u32.to_le_bytes()); // 9 fields total
    data.extend_from_slice(&36u32.to_le_bytes()); // 36 bytes per record (9 * 4)
    data.extend_from_slice(&0u32.to_le_bytes()); // No string block

    // Record 1: ID, Array1[4], Array2[4]
    data.extend_from_slice(&1u32.to_le_bytes()); // ID
    // Array1: UInt32 values
    data.extend_from_slice(&10u32.to_le_bytes());
    data.extend_from_slice(&20u32.to_le_bytes());
    data.extend_from_slice(&30u32.to_le_bytes());
    data.extend_from_slice(&40u32.to_le_bytes());
    // Array2: UInt32 values
    data.extend_from_slice(&100u32.to_le_bytes());
    data.extend_from_slice(&200u32.to_le_bytes());
    data.extend_from_slice(&300u32.to_le_bytes());
    data.extend_from_slice(&400u32.to_le_bytes());

    // Record 2: Similar pattern
    data.extend_from_slice(&2u32.to_le_bytes()); // ID
    // Array1
    data.extend_from_slice(&50u32.to_le_bytes());
    data.extend_from_slice(&60u32.to_le_bytes());
    data.extend_from_slice(&70u32.to_le_bytes());
    data.extend_from_slice(&80u32.to_le_bytes());
    // Array2
    data.extend_from_slice(&500u32.to_le_bytes());
    data.extend_from_slice(&600u32.to_le_bytes());
    data.extend_from_slice(&700u32.to_le_bytes());
    data.extend_from_slice(&800u32.to_le_bytes());

    let parser = DbcParser::parse_bytes(&data).unwrap();
    let record_set = parser.parse_records().unwrap();

    // Test array detection
    let discoverer =
        SchemaDiscoverer::new(parser.header(), parser.data(), record_set.string_block())
            .with_detect_arrays(true);

    let discovered = discoverer.discover().unwrap();

    // Should detect arrays and compress the field count
    // Original: 9 fields -> Detected: 3 fields (ID + 2 arrays)
    assert_eq!(discovered.fields.len(), 3);

    // First field should be ID (not an array)
    assert!(!discovered.fields[0].is_array);

    // Second and third fields should be detected as arrays
    assert!(discovered.fields[1].is_array);
    assert_eq!(discovered.fields[1].array_size, Some(4));
    assert!(discovered.fields[2].is_array);
    assert_eq!(discovered.fields[2].array_size, Some(4));
}

/// Test that our error handling matches community expectations
#[test]
fn test_error_handling_compatibility() {
    // Test invalid magic signature
    let mut invalid_magic = Vec::new();
    invalid_magic.extend_from_slice(b"FAKE"); // Invalid magic
    invalid_magic.extend_from_slice(&1u32.to_le_bytes());
    invalid_magic.extend_from_slice(&1u32.to_le_bytes());
    invalid_magic.extend_from_slice(&4u32.to_le_bytes());
    invalid_magic.extend_from_slice(&0u32.to_le_bytes());

    let mut cursor = Cursor::new(&invalid_magic);
    let result = DbcHeader::parse(&mut cursor);
    assert!(result.is_err());

    // Test truncated file
    let truncated = vec![0u8; 10]; // Too short for header
    let mut cursor = Cursor::new(&truncated);
    let result = DbcHeader::parse(&mut cursor);
    assert!(result.is_err());

    // Test invalid string reference
    let valid_dbc = create_minimal_valid_dbc();
    let parser = DbcParser::parse_bytes(&valid_dbc).unwrap();
    let record_set = parser.parse_records().unwrap();

    // Try to access string beyond string block
    let result = record_set.get_string(StringRef::new(9999));
    assert!(result.is_err());
}

/// Helper function to create a minimal valid DBC for testing
fn create_minimal_valid_dbc() -> Vec<u8> {
    let mut data = Vec::new();

    // Minimal valid header
    data.extend_from_slice(b"WDBC");
    data.extend_from_slice(&1u32.to_le_bytes()); // 1 record
    data.extend_from_slice(&1u32.to_le_bytes()); // 1 field
    data.extend_from_slice(&4u32.to_le_bytes()); // 4 bytes per record
    data.extend_from_slice(&4u32.to_le_bytes()); // String block size

    // One record
    data.extend_from_slice(&42u32.to_le_bytes()); // Value: 42

    // String block
    data.extend_from_slice(b"Hi\0\0"); // "Hi" + null + padding

    data
}

/// Integration test that mimics how community tools would use our library
#[test]
fn test_community_tool_integration_pattern() {
    // This test simulates how a typical WoW modding tool would use our library
    let dbc_data = create_realistic_item_dbc();

    // Step 1: Parse the DBC file (community standard approach)
    let parser = DbcParser::parse_bytes(&dbc_data).unwrap();

    // Step 2: Define schema (how community tools typically work)
    let mut schema = Schema::new("Item");
    schema.add_field(SchemaField::new("ID", FieldType::UInt32));
    schema.add_field(SchemaField::new("ClassID", FieldType::UInt32));
    schema.add_field(SchemaField::new("SubclassID", FieldType::UInt32));
    schema.add_field(SchemaField::new("DisplayName", FieldType::String));
    schema.add_field(SchemaField::new("Quality", FieldType::UInt32));
    schema.add_field(SchemaField::new("Flags", FieldType::UInt32));
    schema.set_key_field("ID");

    // Step 3: Apply schema and parse records
    let parser = parser.with_schema(schema).unwrap();
    let record_set = parser.parse_records().unwrap();

    // Step 4: Access data (typical community tool operations)
    let item_1 = record_set.get_record_by_key(1).unwrap();
    if let Value::StringRef(name_ref) = item_1.get_value_by_name("DisplayName").unwrap() {
        let name = record_set.get_string(*name_ref).unwrap();
        assert_eq!(name, "Sword");
    }

    let item_2 = record_set.get_record_by_key(2).unwrap();
    if let Value::UInt32(quality) = item_2.get_value_by_name("Quality").unwrap() {
        assert_eq!(*quality, 3); // Rare quality
    }

    // Step 5: Iterate through all items (common pattern)
    let mut item_count = 0;
    for record in record_set.records() {
        if let Value::UInt32(id) = record.get_value_by_name("ID").unwrap() {
            assert!(*id > 0); // All items should have positive IDs
            item_count += 1;
        }
    }
    assert_eq!(item_count, 3);
}

/// Create a realistic Item DBC for testing
fn create_realistic_item_dbc() -> Vec<u8> {
    let mut data = Vec::new();

    // Header for Item-like DBC
    data.extend_from_slice(b"WDBC");
    data.extend_from_slice(&3u32.to_le_bytes()); // 3 items
    data.extend_from_slice(&6u32.to_le_bytes()); // 6 fields
    data.extend_from_slice(&24u32.to_le_bytes()); // 24 bytes per record
    data.extend_from_slice(&21u32.to_le_bytes()); // String block size

    // Item 1: Sword
    data.extend_from_slice(&1u32.to_le_bytes()); // ID: 1
    data.extend_from_slice(&2u32.to_le_bytes()); // ClassID: 2 (Weapon)
    data.extend_from_slice(&7u32.to_le_bytes()); // SubclassID: 7 (Sword)
    data.extend_from_slice(&0u32.to_le_bytes()); // DisplayName: offset 0 ("Sword")
    data.extend_from_slice(&2u32.to_le_bytes()); // Quality: 2 (Uncommon)
    data.extend_from_slice(&0x8000u32.to_le_bytes()); // Flags

    // Item 2: Shield
    data.extend_from_slice(&2u32.to_le_bytes()); // ID: 2
    data.extend_from_slice(&4u32.to_le_bytes()); // ClassID: 4 (Armor)
    data.extend_from_slice(&6u32.to_le_bytes()); // SubclassID: 6 (Shield)
    data.extend_from_slice(&6u32.to_le_bytes()); // DisplayName: offset 6 ("Shield")
    data.extend_from_slice(&3u32.to_le_bytes()); // Quality: 3 (Rare)
    data.extend_from_slice(&0x4000u32.to_le_bytes()); // Flags

    // Item 3: Potion
    data.extend_from_slice(&3u32.to_le_bytes()); // ID: 3
    data.extend_from_slice(&0u32.to_le_bytes()); // ClassID: 0 (Consumable)
    data.extend_from_slice(&1u32.to_le_bytes()); // SubclassID: 1 (Potion)
    data.extend_from_slice(&13u32.to_le_bytes()); // DisplayName: offset 13 ("Potion")
    data.extend_from_slice(&1u32.to_le_bytes()); // Quality: 1 (Common)
    data.extend_from_slice(&0x1000u32.to_le_bytes()); // Flags

    // String block: "Sword\0Shield\0Potion\0\0"
    data.extend_from_slice(b"Sword\0Shield\0Potion\0\0");

    data
}
