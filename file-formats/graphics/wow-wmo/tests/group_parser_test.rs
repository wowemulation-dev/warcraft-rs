use std::fs::File;
use std::io::{BufReader, Seek};
use wow_wmo::chunk_discovery::discover_chunks;
use wow_wmo::group_parser::parse_group_file;

#[test]
fn test_parse_classic_group_file() {
    // Test with a real Classic group WMO file
    let test_file = "tests/data/vanilla/World/wmo/Azeroth/Buildings/Stormwind/Stormwind_000.wmo";
    let file =
        File::open(test_file).unwrap_or_else(|_| panic!("Test file not found: {}", test_file));
    let mut reader = BufReader::new(file);

    // First discover chunks
    let discovery = discover_chunks(&mut reader).expect("Failed to discover chunks");

    // Reset reader
    reader
        .seek(std::io::SeekFrom::Start(0))
        .expect("Failed to seek");

    // Parse as group file
    let group = parse_group_file(&mut reader, discovery).expect("Failed to parse group file");

    // Verify basic properties
    assert_eq!(group.version, 17, "Expected version 17 for Classic");
    // group_index is i32, so it can be negative or positive
    // n_triangles is i32, so it can be negative or positive
    // n_vertices is i32, so it can be negative or positive
}

#[test]
fn test_parse_multiple_group_files() {
    // Test multiple group files
    let test_files = vec![
        "tests/data/vanilla/World/wmo/Dungeon/KL_DireMaul/KL_Diremaul_Instance_030.wmo",
        "tests/data/vanilla/World/wmo/KhazModan/Cities/Ironforge/ironforge_091.wmo",
        "tests/data/wotlk/World/wmo/Dungeon/Valgarde/Valgarde_003.wmo",
    ];

    for test_file in test_files {
        let file =
            File::open(test_file).unwrap_or_else(|_| panic!("Test file not found: {}", test_file));
        let mut reader = BufReader::new(file);

        // Discover chunks
        let discovery = discover_chunks(&mut reader).expect("Failed to discover chunks");

        // Reset reader
        reader
            .seek(std::io::SeekFrom::Start(0))
            .expect("Failed to seek");

        // Parse group file
        let group = parse_group_file(&mut reader, discovery)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {:?}", test_file, e));

        // Verify it parsed
        assert_eq!(group.version, 17, "Expected version 17");
    }
}

#[test]
fn test_parse_group_header() {
    use binrw::io::Cursor;

    // Create minimal group file data
    let mut data = Vec::new();

    // MVER chunk
    data.extend_from_slice(b"REVM"); // MVER reversed
    data.extend_from_slice(&4u32.to_le_bytes()); // Size: 4
    data.extend_from_slice(&17u32.to_le_bytes()); // Version 17

    // MOGP chunk (minimal group data)
    data.extend_from_slice(b"PGOM"); // MOGP reversed
    data.extend_from_slice(&68u32.to_le_bytes()); // Size: 68 bytes (serialized header size)

    // MOGP header (68 bytes when serialized)
    data.extend_from_slice(&5u32.to_le_bytes()); // group_name (offset into MOGN)
    data.extend_from_slice(&0u32.to_le_bytes()); // descriptive_group_name
    data.extend_from_slice(&0u32.to_le_bytes()); // flags
    data.extend_from_slice(&[0u8; 24]); // bounding_box (6 floats)
    data.extend_from_slice(&0u16.to_le_bytes()); // portal_start
    data.extend_from_slice(&0u16.to_le_bytes()); // portal_count
    data.extend_from_slice(&0u16.to_le_bytes()); // trans_batch_count
    data.extend_from_slice(&0u16.to_le_bytes()); // int_batch_count
    data.extend_from_slice(&0u16.to_le_bytes()); // ext_batch_count
    data.extend_from_slice(&0u16.to_le_bytes()); // padding_or_batch_type_d
    data.extend_from_slice(&[0u8; 4]); // fog_ids (4 bytes)
    data.extend_from_slice(&0u32.to_le_bytes()); // group_liquid
    data.extend_from_slice(&0u32.to_le_bytes()); // unique_id
    data.extend_from_slice(&0u32.to_le_bytes()); // flags2
    data.extend_from_slice(&0i16.to_le_bytes()); // parent_or_first_child_split_group_index
    data.extend_from_slice(&0i16.to_le_bytes()); // next_split_child_group_index

    let mut cursor = Cursor::new(&data);

    // Discover chunks
    let discovery = discover_chunks(&mut cursor).expect("Failed to discover chunks");

    // Reset cursor
    cursor
        .seek(std::io::SeekFrom::Start(0))
        .expect("Failed to seek");

    // Parse group file
    let group = parse_group_file(&mut cursor, discovery).expect("Failed to parse group");

    // Verify parsed values
    assert_eq!(group.version, 17);
    assert_eq!(group.group_name_index, 5); // This is from the group_name field
    assert_eq!(group.n_triangles, 0); // Calculated from data, which is empty
    assert_eq!(group.n_vertices, 0); // Calculated from data, which is empty
}
