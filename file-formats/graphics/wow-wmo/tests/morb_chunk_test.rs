use std::io::Cursor;
use wow_wmo::{ParsedWmo, parse_wmo};

#[test]
fn test_morb_chunk_parsing() {
    // Create test data for a group file with MORB chunk
    let mut data = Vec::new();

    // MVER chunk
    data.extend_from_slice(b"REVM");
    data.extend_from_slice(&8u32.to_le_bytes());
    data.extend_from_slice(&17u32.to_le_bytes());
    data.extend_from_slice(&[0; 4]);

    // MOGP chunk with nested chunks
    let nested_chunks_size = 8 + 6 + 8 + 12 + 8 + 10; // MOVI + MOVT + MORB headers and data
    let mogp_size = 68 + nested_chunks_size;
    data.extend_from_slice(b"PGOM");
    data.extend_from_slice(&(mogp_size as u32).to_le_bytes());
    data.extend_from_slice(&[0; 68]); // MOGP header

    // MOVI chunk (minimal vertex indices)
    data.extend_from_slice(b"IVOM");
    data.extend_from_slice(&6u32.to_le_bytes());
    data.extend_from_slice(&[0; 6]);

    // MOVT chunk (minimal vertex positions)
    data.extend_from_slice(b"TVOM");
    data.extend_from_slice(&12u32.to_le_bytes());
    data.extend_from_slice(&[0; 12]);

    // MORB chunk (additional render batches)
    data.extend_from_slice(b"BROM");
    data.extend_from_slice(&10u32.to_le_bytes()); // size (1 batch * 10 bytes)

    // Single MorbEntry: start_index(u16) + index_count(u16) + min_index(u16) + max_index(u16) + flags(u8) + material_id(u8)
    data.extend_from_slice(&100u16.to_le_bytes()); // start_index
    data.extend_from_slice(&50u16.to_le_bytes()); // index_count
    data.extend_from_slice(&0u16.to_le_bytes()); // min_index
    data.extend_from_slice(&49u16.to_le_bytes()); // max_index
    data.extend_from_slice(&0u8.to_le_bytes()); // flags
    data.extend_from_slice(&1u8.to_le_bytes()); // material_id

    let mut reader = Cursor::new(data);
    let result = parse_wmo(&mut reader);

    assert!(result.is_ok(), "MORB parsing should succeed");
    let wmo = result.unwrap();

    if let ParsedWmo::Group(group) = wmo {
        // Verify additional render batches were parsed correctly
        assert_eq!(group.additional_render_batches.len(), 1);
        let batch = &group.additional_render_batches[0];
        assert_eq!(batch.start_index, 100);
        assert_eq!(batch.index_count, 50);
        assert_eq!(batch.min_index, 0);
        assert_eq!(batch.max_index, 49);
        assert_eq!(batch.flags, 0);
        assert_eq!(batch.material_id, 1);
    } else {
        panic!("Expected group WMO file, got root file");
    }
}

#[test]
fn test_morb_chunk_empty() {
    // Test group file without MORB chunk
    let mut data = Vec::new();

    // MVER chunk
    data.extend_from_slice(b"REVM");
    data.extend_from_slice(&8u32.to_le_bytes());
    data.extend_from_slice(&17u32.to_le_bytes());
    data.extend_from_slice(&[0; 4]);

    // MOGP chunk
    let nested_chunks_size = 8 + 6 + 8 + 12; // MOVI + MOVT headers and data
    let mogp_size = 68 + nested_chunks_size;
    data.extend_from_slice(b"PGOM");
    data.extend_from_slice(&(mogp_size as u32).to_le_bytes());
    data.extend_from_slice(&[0; 68]);

    // MOVI chunk
    data.extend_from_slice(b"IVOM");
    data.extend_from_slice(&6u32.to_le_bytes());
    data.extend_from_slice(&[0; 6]);

    // MOVT chunk
    data.extend_from_slice(b"TVOM");
    data.extend_from_slice(&12u32.to_le_bytes());
    data.extend_from_slice(&[0; 12]);

    let mut reader = Cursor::new(data);
    let result = parse_wmo(&mut reader);

    assert!(result.is_ok(), "Parsing without MORB should succeed");
    let wmo = result.unwrap();

    if let ParsedWmo::Group(group) = wmo {
        // Should have empty additional render batches list
        assert_eq!(group.additional_render_batches.len(), 0);
    } else {
        panic!("Expected group WMO file, got root file");
    }
}
