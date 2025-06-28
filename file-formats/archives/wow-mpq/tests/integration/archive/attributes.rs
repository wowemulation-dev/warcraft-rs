//! Tests for (attributes) file support

use std::path::PathBuf;
use wow_mpq::{
    Archive,
    special_files::{AttributeFlags, Attributes, FileAttributes},
};

fn get_test_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("test-data")
        .join(filename)
}

#[test]
fn test_load_attributes_from_archive() {
    // This test requires an MPQ file with an (attributes) file
    // We'll create a mock test for now since we don't have test data with attributes

    // Test parsing of attributes data directly
    let mut data = Vec::new();

    // Header
    data.extend_from_slice(&100u32.to_le_bytes()); // version
    data.extend_from_slice(&(AttributeFlags::CRC32 | AttributeFlags::MD5).to_le_bytes()); // flags

    // 2 files
    let block_count = 2;

    // CRC32 values
    data.extend_from_slice(&0x12345678u32.to_le_bytes());
    data.extend_from_slice(&0x9ABCDEF0u32.to_le_bytes());

    // MD5 hashes
    data.extend_from_slice(&[1u8; 16]); // First file MD5
    data.extend_from_slice(&[2u8; 16]); // Second file MD5

    let attributes = Attributes::parse(&data.into(), block_count).unwrap();

    assert_eq!(attributes.version, 100);
    assert!(attributes.flags.has_crc32());
    assert!(attributes.flags.has_md5());
    assert!(!attributes.flags.has_filetime());
    assert!(!attributes.flags.has_patch_bit());

    // Check first file attributes
    let file1 = attributes.get_file_attributes(0).unwrap();
    assert_eq!(file1.crc32, Some(0x12345678));
    assert_eq!(file1.md5, Some([1u8; 16]));
    assert_eq!(file1.filetime, None);
    assert_eq!(file1.is_patch, None);

    // Check second file attributes
    let file2 = attributes.get_file_attributes(1).unwrap();
    assert_eq!(file2.crc32, Some(0x9ABCDEF0));
    assert_eq!(file2.md5, Some([2u8; 16]));
}

#[test]
fn test_attributes_with_all_fields() {
    let mut data = Vec::new();

    // Header
    data.extend_from_slice(&100u32.to_le_bytes()); // version
    data.extend_from_slice(&AttributeFlags::ALL.to_le_bytes()); // all flags

    let block_count = 3;

    // CRC32 values
    data.extend_from_slice(&0x11111111u32.to_le_bytes());
    data.extend_from_slice(&0x22222222u32.to_le_bytes());
    data.extend_from_slice(&0x33333333u32.to_le_bytes());

    // Timestamps
    data.extend_from_slice(&0x0123456789ABCDEFu64.to_le_bytes());
    data.extend_from_slice(&0xFEDCBA9876543210u64.to_le_bytes());
    data.extend_from_slice(&0x1122334455667788u64.to_le_bytes());

    // MD5 hashes
    for i in 0..3 {
        data.extend_from_slice(&[i as u8; 16]);
    }

    // Patch bits (1 byte for 3 files)
    data.push(0b00000101); // Files 0 and 2 are patches

    let attributes = Attributes::parse(&data.into(), block_count).unwrap();

    // Check all fields are present
    assert_eq!(attributes.flags.as_u32(), AttributeFlags::ALL);

    // Check file 0
    let file0 = attributes.get_file_attributes(0).unwrap();
    assert_eq!(file0.crc32, Some(0x11111111));
    assert_eq!(file0.filetime, Some(0x0123456789ABCDEF));
    assert_eq!(file0.md5, Some([0u8; 16]));
    assert_eq!(file0.is_patch, Some(true));

    // Check file 1
    let file1 = attributes.get_file_attributes(1).unwrap();
    assert_eq!(file1.crc32, Some(0x22222222));
    assert_eq!(file1.filetime, Some(0xFEDCBA9876543210));
    assert_eq!(file1.md5, Some([1u8; 16]));
    assert_eq!(file1.is_patch, Some(false));

    // Check file 2
    let file2 = attributes.get_file_attributes(2).unwrap();
    assert_eq!(file2.crc32, Some(0x33333333));
    assert_eq!(file2.filetime, Some(0x1122334455667788));
    assert_eq!(file2.md5, Some([2u8; 16]));
    assert_eq!(file2.is_patch, Some(true));
}

#[test]
fn test_attributes_roundtrip() {
    // Create attributes with various configurations
    let mut file_attrs = Vec::new();

    for i in 0..5 {
        let mut attr = FileAttributes::new();
        attr.crc32 = Some(0x10000000 + i);
        attr.filetime = Some(0x0100000000000000 * (i as u64 + 1));
        attr.md5 = Some([i as u8; 16]);
        attr.is_patch = Some(i % 2 == 0);
        file_attrs.push(attr);
    }

    let original = Attributes {
        version: 100,
        flags: AttributeFlags::new(AttributeFlags::ALL),
        file_attributes: file_attrs,
    };

    // Convert to bytes
    let bytes = original.to_bytes().unwrap();

    // Parse back
    let parsed = Attributes::parse(&bytes.into(), 5).unwrap();

    // Verify
    assert_eq!(parsed.version, original.version);
    assert_eq!(parsed.flags.as_u32(), original.flags.as_u32());
    assert_eq!(parsed.file_attributes.len(), original.file_attributes.len());

    for i in 0..5 {
        let orig = &original.file_attributes[i];
        let parsed = &parsed.file_attributes[i];
        assert_eq!(orig.crc32, parsed.crc32);
        assert_eq!(orig.filetime, parsed.filetime);
        assert_eq!(orig.md5, parsed.md5);
        assert_eq!(orig.is_patch, parsed.is_patch);
    }
}

#[test]
fn test_archive_load_attributes() {
    // Test loading attributes from an actual MPQ archive
    let path = get_test_path("attributes/archive_with_attributes.mpq");

    // Skip test if the file doesn't exist
    if !path.exists() {
        eprintln!("Test archive not found at {path:?}, skipping test");
        return;
    }

    let mut archive = Archive::open(&path).unwrap();
    archive.load_attributes().unwrap();

    if let Some(attrs) = archive.attributes() {
        println!(
            "Archive has attributes with flags: {:08X}",
            attrs.flags.as_u32()
        );

        // Get attributes for first file
        if let Some(file_attrs) = attrs.get_file_attributes(0) {
            if let Some(crc) = file_attrs.crc32 {
                println!("First file CRC32: {crc:08X}");
            }
            if let Some(md5) = file_attrs.md5 {
                println!("First file MD5: {md5:02X?}");
            }
        }
    }
}

#[test]
fn test_attributes_loaded_automatically() {
    // Test that attributes are loaded automatically when opening an archive
    let path = get_test_path("attributes/archive_with_attributes.mpq");

    // Skip test if the file doesn't exist
    if !path.exists() {
        eprintln!("Test archive not found at {path:?}, skipping test");
        return;
    }

    // Open archive without explicitly loading attributes
    let archive = Archive::open(&path).unwrap();

    // Attributes should be loaded automatically
    assert!(
        archive.attributes().is_some(),
        "Attributes should be loaded automatically when opening an archive"
    );

    if let Some(attrs) = archive.attributes() {
        println!(
            "Automatically loaded attributes with flags: {:08X}",
            attrs.flags.as_u32()
        );
    }
}

#[test]
fn test_no_attributes_archive() {
    use tempfile::TempDir;
    use wow_mpq::OpenOptions;

    // Test that archives without attributes work correctly
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("empty.mpq");

    // Create an empty archive
    OpenOptions::new()
        .version(wow_mpq::FormatVersion::V1)
        .create(&path)
        .unwrap();

    // Open archive - attributes loading should not fail even if no attributes exist
    let archive = Archive::open(&path).unwrap();

    // No attributes should be present
    assert!(
        archive.attributes().is_none(),
        "Empty archive should not have attributes"
    );
}
