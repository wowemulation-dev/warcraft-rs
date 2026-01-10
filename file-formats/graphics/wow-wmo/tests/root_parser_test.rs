use std::fs::File;
use std::io::{BufReader, Seek};
use wow_wmo::chunk_discovery::discover_chunks;
use wow_wmo::root_parser::parse_root_file;

#[test]
#[ignore = "Requires WMO test files not available in CI"]
fn test_parse_classic_root_file() {
    // Test with a real Classic root WMO file
    let test_file = "tests/data/vanilla/World/wmo/Azeroth/Buildings/Stormwind/Stormwind.wmo";
    let file =
        File::open(test_file).unwrap_or_else(|_| panic!("Test file not found: {}", test_file));
    let mut reader = BufReader::new(file);

    // First discover chunks
    let discovery = discover_chunks(&mut reader).expect("Failed to discover chunks");

    // Reset reader
    reader
        .seek(std::io::SeekFrom::Start(0))
        .expect("Failed to seek");

    // Parse as root file
    let root = parse_root_file(&mut reader, discovery).expect("Failed to parse root file");

    // Verify basic properties
    assert_eq!(root.version, 17, "Expected version 17 for Classic");
    assert!(root.n_materials > 0, "Root file should have materials");
    assert!(root.n_groups > 0, "Root file should have groups");
    // n_portals and n_lights are u32, so they're always >= 0
}

#[test]
#[ignore = "Requires WMO test files not available in CI"]
fn test_parse_wotlk_root_file() {
    // Test with a WotLK root WMO file
    let test_file =
        "tests/data/wotlk/World/wmo/Northrend/Dragonblight/NewHearthglen/NH_Cathedral.wmo";
    let file =
        File::open(test_file).unwrap_or_else(|_| panic!("Test file not found: {}", test_file));
    let mut reader = BufReader::new(file);

    // First discover chunks
    let discovery = discover_chunks(&mut reader).expect("Failed to discover chunks");

    // Reset reader
    reader
        .seek(std::io::SeekFrom::Start(0))
        .expect("Failed to seek");

    // Parse as root file
    let root = parse_root_file(&mut reader, discovery).expect("Failed to parse root file");

    // Verify basic properties
    assert_eq!(root.version, 17, "Expected version 17 for WotLK");
    assert!(root.n_materials > 0, "Should have materials count");
    assert!(root.n_groups > 0, "Root file should have groups");
}

#[test]
fn test_parse_root_header() {
    use binrw::io::Cursor;

    // Create minimal root file data
    let mut data = Vec::new();

    // MVER chunk
    data.extend_from_slice(b"REVM"); // MVER reversed
    data.extend_from_slice(&4u32.to_le_bytes()); // Size: 4
    data.extend_from_slice(&17u32.to_le_bytes()); // Version 17

    // MOHD chunk (minimal header)
    data.extend_from_slice(b"DHOM"); // MOHD reversed
    data.extend_from_slice(&64u32.to_le_bytes()); // Size: 64

    // MOHD data (64 bytes)
    data.extend_from_slice(&1u32.to_le_bytes()); // nMaterials
    data.extend_from_slice(&2u32.to_le_bytes()); // nGroups
    data.extend_from_slice(&3u32.to_le_bytes()); // nPortals
    data.extend_from_slice(&4u32.to_le_bytes()); // nLights
    data.extend_from_slice(&5u32.to_le_bytes()); // nDoodadNames
    data.extend_from_slice(&6u32.to_le_bytes()); // nDoodadDefs
    data.extend_from_slice(&7u32.to_le_bytes()); // nDoodadSets
    data.extend_from_slice(&[0u8; 36]); // Padding to 64 bytes

    let mut cursor = Cursor::new(&data);

    // Discover chunks
    let discovery = discover_chunks(&mut cursor).expect("Failed to discover chunks");

    // Reset cursor
    cursor
        .seek(std::io::SeekFrom::Start(0))
        .expect("Failed to seek");

    // Parse root file
    let root = parse_root_file(&mut cursor, discovery).expect("Failed to parse root");

    // Verify parsed values
    assert_eq!(root.version, 17);
    assert_eq!(root.n_materials, 1);
    assert_eq!(root.n_groups, 2);
    assert_eq!(root.n_portals, 3);
    assert_eq!(root.n_lights, 4);
}
