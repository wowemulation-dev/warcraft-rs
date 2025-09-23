use std::fs::File;
use std::io::BufReader;
use wow_wmo::{ParsedWmo, parse_wmo, parse_wmo_with_metadata};

#[test]
fn test_parse_root_file_unified_api() {
    // Test unified API with a root file
    let test_file = "tests/data/vanilla/World/wmo/Azeroth/Buildings/Stormwind/Stormwind.wmo";
    let file =
        File::open(test_file).unwrap_or_else(|_| panic!("Test file not found: {}", test_file));
    let mut reader = BufReader::new(file);

    // Use unified API
    let parsed = parse_wmo(&mut reader).expect("Failed to parse WMO");

    // Should return a Root variant
    match parsed {
        ParsedWmo::Root(root) => {
            assert_eq!(root.version, 17, "Expected version 17");
            assert!(root.n_groups > 0, "Root should have groups");
            // n_materials is u32, always >= 0
        }
        ParsedWmo::Group(_) => {
            panic!("Expected root file, got group");
        }
    }
}

#[test]
fn test_parse_group_file_unified_api() {
    // Test unified API with a group file
    let test_file = "tests/data/vanilla/World/wmo/Azeroth/Buildings/Stormwind/Stormwind_000.wmo";
    let file =
        File::open(test_file).unwrap_or_else(|_| panic!("Test file not found: {}", test_file));
    let mut reader = BufReader::new(file);

    // Use unified API
    let parsed = parse_wmo(&mut reader).expect("Failed to parse WMO");

    // Should return a Group variant
    match parsed {
        ParsedWmo::Group(group) => {
            assert_eq!(group.version, 17, "Expected version 17");
            // group_index is i32, can be negative or positive
            // n_vertices is i32, can be negative or positive
        }
        ParsedWmo::Root(_) => {
            panic!("Expected group file, got root");
        }
    }
}

#[test]
fn test_parse_multiple_files() {
    // Test API with multiple files
    let test_files = vec![
        (
            "tests/data/vanilla/World/wmo/Dungeon/KZ_Uldaman/KZ_Uldaman_A.wmo",
            true,
        ), // root
        (
            "tests/data/vanilla/World/wmo/Dungeon/KL_DireMaul/KL_Diremaul_Instance_030.wmo",
            false,
        ), // group
        (
            "tests/data/wotlk/World/wmo/Northrend/Dragonblight/NewHearthglen/NH_Cathedral.wmo",
            true,
        ), // root
        (
            "tests/data/wotlk/World/wmo/Dungeon/Valgarde/Valgarde_003.wmo",
            false,
        ), // group
    ];

    for (file_path, is_root) in test_files {
        let file =
            File::open(file_path).unwrap_or_else(|_| panic!("Test file not found: {}", file_path));
        let mut reader = BufReader::new(file);

        let parsed = parse_wmo(&mut reader)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {:?}", file_path, e));

        // Verify correct type detection
        match (parsed, is_root) {
            (ParsedWmo::Root(_), true) => {}   // Correct
            (ParsedWmo::Group(_), false) => {} // Correct
            _ => panic!("Incorrect file type detection for {}", file_path),
        }
    }
}

#[test]
fn test_malformed_chunk_detection() {
    use binrw::io::Cursor;

    // Create data with malformed chunk
    let mut data = Vec::new();

    // Valid MVER chunk
    data.extend_from_slice(b"REVM"); // MVER reversed
    data.extend_from_slice(&4u32.to_le_bytes()); // Size: 4
    data.extend_from_slice(&17u32.to_le_bytes()); // Version 17

    // Malformed chunk with huge size
    data.extend_from_slice(b"\xFF\xFF\xFF\xFF"); // Invalid chunk ID
    data.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // Invalid huge size

    let mut cursor = Cursor::new(&data);

    // Use metadata API to check malformed tracking
    let result = parse_wmo_with_metadata(&mut cursor);

    match result {
        Ok(parsed_result) => {
            // Should have detected malformed chunks
            assert!(
                parsed_result.discovery.has_malformed_chunks(),
                "Should detect malformed chunks"
            );
            assert_eq!(
                parsed_result.discovery.malformed_count(),
                1,
                "Should have one malformed chunk"
            );
        }
        Err(_) => {
            // Error is also acceptable for severely malformed data
        }
    }
}

#[test]
fn test_unknown_chunk_detection() {
    use binrw::io::Cursor;

    // Create data with unknown chunk
    let mut data = Vec::new();

    // Valid MVER chunk
    data.extend_from_slice(b"REVM"); // MVER reversed
    data.extend_from_slice(&4u32.to_le_bytes()); // Size: 4
    data.extend_from_slice(&17u32.to_le_bytes()); // Version 17

    // Unknown but valid chunk
    data.extend_from_slice(b"KNNU"); // UNKN reversed - unknown chunk
    data.extend_from_slice(&8u32.to_le_bytes()); // Size: 8
    data.extend_from_slice(&[0x42u8; 8]); // Dummy data

    // Valid MOHD chunk
    data.extend_from_slice(b"DHOM"); // MOHD reversed
    data.extend_from_slice(&64u32.to_le_bytes()); // Size: 64
    data.extend_from_slice(&[0u8; 64]); // Dummy data

    let mut cursor = Cursor::new(&data);

    // Use metadata API to check unknown chunk tracking
    let result = parse_wmo_with_metadata(&mut cursor);

    assert!(
        result.is_ok(),
        "Should parse successfully with unknown chunks"
    );
    let parsed_result = result.unwrap();

    // Should have detected unknown chunk
    assert!(
        parsed_result.discovery.has_unknown_chunks(),
        "Should detect unknown chunks"
    );
    assert_eq!(
        parsed_result.discovery.unknown_count(),
        1,
        "Should have one unknown chunk"
    );

    // Should still parse known chunks correctly
    assert_eq!(parsed_result.wmo.version(), 17, "Should parse version");
}

#[test]
fn test_file_type_property() {
    // Test that file_type property is exposed
    let test_file = "tests/data/vanilla/World/wmo/Azeroth/Buildings/Stormwind/Stormwind.wmo";
    let file = File::open(test_file).unwrap();
    let mut reader = BufReader::new(file);

    let parsed = parse_wmo(&mut reader).expect("Failed to parse");

    // Check we can get file type
    let file_type = parsed.file_type();
    assert_eq!(file_type, wow_wmo::file_type::WmoFileType::Root);
}
