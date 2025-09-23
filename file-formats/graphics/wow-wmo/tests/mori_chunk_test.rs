use std::io::Cursor;
use wow_wmo::{ParsedWmo, parse_wmo};

#[test]
fn test_mori_chunk_parsing() {
    // Create test data for a group file with MORI chunk
    let mut data = Vec::new();

    // MVER chunk (version)
    data.extend_from_slice(b"REVM");
    data.extend_from_slice(&8u32.to_le_bytes()); // size
    data.extend_from_slice(&17u32.to_le_bytes()); // version
    data.extend_from_slice(&[0; 4]); // padding

    // MOGP chunk (group header) - this makes it a group file
    // Must contain nested chunks, so size = header + nested chunks
    let nested_chunks_size = 8 + 6 + 8 + 12 + 8 + 8; // MOVI header+data + MOVT header+data + MORI header+data
    let mogp_size = 68 + nested_chunks_size; // header + nested chunks
    data.extend_from_slice(b"PGOM");
    data.extend_from_slice(&(mogp_size as u32).to_le_bytes()); // size
    data.extend_from_slice(&[0; 68]); // dummy group header

    // Nested chunks inside MOGP
    // MOVI chunk (minimal vertex indices for valid group)
    data.extend_from_slice(b"IVOM");
    data.extend_from_slice(&6u32.to_le_bytes()); // size (3 triangles * 2 bytes each)
    data.extend_from_slice(&0u16.to_le_bytes());
    data.extend_from_slice(&1u16.to_le_bytes());
    data.extend_from_slice(&2u16.to_le_bytes());

    // MOVT chunk (minimal vertex positions for valid group)
    data.extend_from_slice(b"TVOM");
    data.extend_from_slice(&12u32.to_le_bytes()); // size (1 vertex * 12 bytes)
    data.extend_from_slice(&[0; 12]); // dummy vertex data (x, y, z as floats)

    // MORI chunk (triangle strip indices)
    data.extend_from_slice(b"IROM");
    data.extend_from_slice(&8u32.to_le_bytes()); // size (4 indices * 2 bytes each)
    data.extend_from_slice(&0u16.to_le_bytes()); // first strip index
    data.extend_from_slice(&1u16.to_le_bytes()); // second strip index
    data.extend_from_slice(&2u16.to_le_bytes()); // third strip index
    data.extend_from_slice(&3u16.to_le_bytes()); // fourth strip index

    let mut reader = Cursor::new(data);
    let result = parse_wmo(&mut reader);

    assert!(result.is_ok(), "MORI parsing should succeed");
    let wmo = result.unwrap();

    // Verify this is a group file and get the group data
    if let ParsedWmo::Group(group) = wmo {
        // Verify triangle strip indices were parsed correctly
        assert_eq!(group.triangle_strip_indices.len(), 4);
        assert_eq!(group.triangle_strip_indices[0], 0);
        assert_eq!(group.triangle_strip_indices[1], 1);
        assert_eq!(group.triangle_strip_indices[2], 2);
        assert_eq!(group.triangle_strip_indices[3], 3);
    } else {
        panic!("Expected group WMO file, got root file");
    }
}

#[test]
fn test_mori_chunk_empty() {
    // Test group file without MORI chunk
    let mut data = Vec::new();

    // MVER chunk
    data.extend_from_slice(b"REVM");
    data.extend_from_slice(&8u32.to_le_bytes());
    data.extend_from_slice(&17u32.to_le_bytes());
    data.extend_from_slice(&[0; 4]);

    // MOGP chunk (makes it a group file)
    let nested_chunks_size = 8 + 6 + 8 + 12; // MOVI header+data + MOVT header+data
    let mogp_size = 68 + nested_chunks_size;
    data.extend_from_slice(b"PGOM");
    data.extend_from_slice(&(mogp_size as u32).to_le_bytes());
    data.extend_from_slice(&[0; 68]);

    // MOVI chunk (minimal for valid group)
    data.extend_from_slice(b"IVOM");
    data.extend_from_slice(&6u32.to_le_bytes());
    data.extend_from_slice(&[0; 6]);

    // MOVT chunk (minimal for valid group)
    data.extend_from_slice(b"TVOM");
    data.extend_from_slice(&12u32.to_le_bytes());
    data.extend_from_slice(&[0; 12]);

    let mut reader = Cursor::new(data);
    let result = parse_wmo(&mut reader);

    assert!(result.is_ok(), "Parsing without MORI should succeed");
    let wmo = result.unwrap();

    if let ParsedWmo::Group(group) = wmo {
        // Should have empty triangle strip indices list
        assert_eq!(group.triangle_strip_indices.len(), 0);
    } else {
        panic!("Expected group WMO file, got root file");
    }
}

#[test]
fn test_mori_chunk_large_indices() {
    // Test with larger triangle strip indices
    let mut data = Vec::new();

    // MVER chunk
    data.extend_from_slice(b"REVM");
    data.extend_from_slice(&8u32.to_le_bytes());
    data.extend_from_slice(&17u32.to_le_bytes());
    data.extend_from_slice(&[0; 4]);

    // MOGP chunk
    let nested_chunks_size = 8 + 6 + 8 + 12 + 8 + 4; // MOVI header+data + MOVT header+data + MORI header+data
    let mogp_size = 68 + nested_chunks_size;
    data.extend_from_slice(b"PGOM");
    data.extend_from_slice(&(mogp_size as u32).to_le_bytes());
    data.extend_from_slice(&[0; 68]);

    // MOVI chunk (minimal for valid group)
    data.extend_from_slice(b"IVOM");
    data.extend_from_slice(&6u32.to_le_bytes());
    data.extend_from_slice(&[0; 6]);

    // MOVT chunk (minimal for valid group)
    data.extend_from_slice(b"TVOM");
    data.extend_from_slice(&12u32.to_le_bytes());
    data.extend_from_slice(&[0; 12]);

    // MORI chunk with larger indices
    data.extend_from_slice(b"IROM");
    data.extend_from_slice(&4u32.to_le_bytes()); // size (2 indices * 2 bytes each)
    data.extend_from_slice(&12345u16.to_le_bytes()); // larger index
    data.extend_from_slice(&54321u16.to_le_bytes()); // another larger index

    let mut reader = Cursor::new(data);
    let result = parse_wmo(&mut reader);

    assert!(result.is_ok(), "Large MORI indices parsing should succeed");
    let wmo = result.unwrap();

    if let ParsedWmo::Group(group) = wmo {
        assert_eq!(group.triangle_strip_indices.len(), 2);
        assert_eq!(group.triangle_strip_indices[0], 12345);
        assert_eq!(group.triangle_strip_indices[1], 54321);
    } else {
        panic!("Expected group WMO file, got root file");
    }
}
