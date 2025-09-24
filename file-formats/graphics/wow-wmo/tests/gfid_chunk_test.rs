use std::io::Cursor;
use wow_wmo::{ParsedWmo, parse_wmo};

#[test]
fn test_gfid_chunk_parsing() {
    // Create test data with GFID chunk
    let mut data = Vec::new();

    // MVER chunk (version) - chunk IDs are stored in reversed byte order
    data.extend_from_slice(b"REVM");
    data.extend_from_slice(&8u32.to_le_bytes()); // size
    data.extend_from_slice(&17u32.to_le_bytes()); // version
    data.extend_from_slice(&[0; 4]); // padding to align

    // MOHD chunk (header with group count)
    data.extend_from_slice(b"DHOM");
    data.extend_from_slice(&64u32.to_le_bytes()); // size
    data.extend_from_slice(&2u32.to_le_bytes()); // n_materials
    data.extend_from_slice(&3u32.to_le_bytes()); // n_groups
    data.extend_from_slice(&0u32.to_le_bytes()); // n_portals
    data.extend_from_slice(&0u32.to_le_bytes()); // n_lights
    data.extend_from_slice(&0u32.to_le_bytes()); // n_doodad_names
    data.extend_from_slice(&0u32.to_le_bytes()); // n_doodad_defs
    data.extend_from_slice(&0u32.to_le_bytes()); // n_doodad_sets
    data.extend_from_slice(&[0; 36]); // padding to reach 64 bytes

    // MOMT chunk (materials) - needed for root file detection
    data.extend_from_slice(b"TMOM");
    data.extend_from_slice(&128u32.to_le_bytes()); // size (2 materials * 64 bytes each)
    data.extend_from_slice(&[0; 128]); // dummy material data

    // GFID chunk (group file IDs)
    data.extend_from_slice(b"DIFG");
    data.extend_from_slice(&12u32.to_le_bytes()); // size (3 IDs * 4 bytes each)
    data.extend_from_slice(&123456u32.to_le_bytes()); // first group file ID
    data.extend_from_slice(&789012u32.to_le_bytes()); // second group file ID
    data.extend_from_slice(&345678u32.to_le_bytes()); // third group file ID

    let mut reader = Cursor::new(data);

    let result = parse_wmo(&mut reader);

    assert!(result.is_ok(), "GFID parsing should succeed");
    let wmo = result.unwrap();

    // Verify this is a root file and get the root data
    if let ParsedWmo::Root(root) = wmo {
        // Verify group file IDs were parsed correctly
        assert_eq!(root.group_file_ids.len(), 3);
        assert_eq!(root.group_file_ids[0], 123456);
        assert_eq!(root.group_file_ids[1], 789012);
        assert_eq!(root.group_file_ids[2], 345678);
    } else {
        panic!("Expected root WMO file, got group file");
    }
}

#[test]
fn test_gfid_chunk_empty() {
    // Test with no GFID chunk (older WoW versions)
    let mut data = Vec::new();

    // MVER chunk (version)
    data.extend_from_slice(b"REVM");
    data.extend_from_slice(&8u32.to_le_bytes()); // size
    data.extend_from_slice(&17u32.to_le_bytes()); // version
    data.extend_from_slice(&[0; 4]); // padding

    // MOHD chunk (minimal header)
    data.extend_from_slice(b"DHOM");
    data.extend_from_slice(&64u32.to_le_bytes()); // size
    data.extend_from_slice(&[0; 64]); // all zeros

    // MOMT chunk (materials) - needed for root file detection
    data.extend_from_slice(b"TMOM");
    data.extend_from_slice(&0u32.to_le_bytes()); // size (no materials)

    let mut reader = Cursor::new(data);
    let result = parse_wmo(&mut reader);

    assert!(result.is_ok(), "Parsing without GFID should succeed");
    let wmo = result.unwrap();

    // Should have empty group file IDs list
    if let ParsedWmo::Root(root) = wmo {
        assert_eq!(root.group_file_ids.len(), 0);
    } else {
        panic!("Expected root WMO file, got group file");
    }
}

#[test]
fn test_gfid_chunk_single_id() {
    // Test with single group file ID
    let mut data = Vec::new();

    // MVER chunk
    data.extend_from_slice(b"REVM");
    data.extend_from_slice(&8u32.to_le_bytes());
    data.extend_from_slice(&17u32.to_le_bytes());
    data.extend_from_slice(&[0; 4]);

    // MOHD chunk
    data.extend_from_slice(b"DHOM");
    data.extend_from_slice(&64u32.to_le_bytes());
    data.extend_from_slice(&[0; 64]);

    // MOMT chunk (materials) - needed for root file detection
    data.extend_from_slice(b"TMOM");
    data.extend_from_slice(&0u32.to_le_bytes()); // size (no materials)

    // GFID chunk with single ID
    data.extend_from_slice(b"DIFG");
    data.extend_from_slice(&4u32.to_le_bytes()); // size (1 ID * 4 bytes)
    data.extend_from_slice(&999888u32.to_le_bytes()); // single group file ID

    let mut reader = Cursor::new(data);
    let result = parse_wmo(&mut reader);

    assert!(result.is_ok(), "Single GFID parsing should succeed");
    let wmo = result.unwrap();

    if let ParsedWmo::Root(root) = wmo {
        assert_eq!(root.group_file_ids.len(), 1);
        assert_eq!(root.group_file_ids[0], 999888);
    } else {
        panic!("Expected root WMO file, got group file");
    }
}
