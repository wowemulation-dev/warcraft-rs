use binrw::io::Cursor;
use wow_wmo::chunk_discovery::discover_chunks;

#[test]
fn test_malformed_chunk_recovery() {
    // Create a file with a malformed chunk in the middle
    let mut data = Vec::new();

    // Valid MVER chunk
    data.extend_from_slice(b"REVM"); // MVER reversed
    data.extend_from_slice(&4u32.to_le_bytes()); // Size: 4
    data.extend_from_slice(&17u32.to_le_bytes()); // Version 17

    // Valid MOHD chunk
    data.extend_from_slice(b"DHOM"); // MOHD reversed
    data.extend_from_slice(&64u32.to_le_bytes()); // Size: 64
    data.extend_from_slice(&[0u8; 64]); // Dummy data

    // Malformed chunk - invalid size that would overflow
    data.extend_from_slice(b"XXXX"); // Unknown chunk
    data.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // Invalid huge size

    // Another valid chunk after the malformed one (won't be reached)
    data.extend_from_slice(b"TMOM"); // MOMT reversed
    data.extend_from_slice(&8u32.to_le_bytes()); // Size: 8
    data.extend_from_slice(&[0u8; 8]); // Dummy data

    let mut cursor = Cursor::new(&data);
    let discovery = discover_chunks(&mut cursor).expect("Should handle malformed chunks");

    // Should have discovered the valid chunks before the malformed one
    assert!(discovery.chunks.len() >= 2, "Should discover valid chunks");
    assert_eq!(discovery.chunks[0].id.as_str(), "MVER");
    assert_eq!(discovery.chunks[1].id.as_str(), "MOHD");

    // Check if malformed chunks are tracked
    assert!(
        discovery.has_malformed_chunks(),
        "Should track malformed chunks"
    );
    assert_eq!(
        discovery.malformed_count(),
        1,
        "Should have one malformed chunk"
    );
}

#[test]
fn test_unknown_chunk_tracking() {
    // Create a file with unknown but valid chunks
    let mut data = Vec::new();

    // Valid MVER chunk
    data.extend_from_slice(b"REVM"); // MVER reversed
    data.extend_from_slice(&4u32.to_le_bytes()); // Size: 4
    data.extend_from_slice(&17u32.to_le_bytes()); // Version 17

    // Unknown but valid chunk
    data.extend_from_slice(b"KNNU"); // UNKN reversed - unknown chunk
    data.extend_from_slice(&8u32.to_le_bytes()); // Size: 8
    data.extend_from_slice(&[0x42u8; 8]); // Dummy data

    // Another known chunk
    data.extend_from_slice(b"DHOM"); // MOHD reversed
    data.extend_from_slice(&64u32.to_le_bytes()); // Size: 64
    data.extend_from_slice(&[0u8; 64]); // Dummy data

    let mut cursor = Cursor::new(&data);
    let discovery = discover_chunks(&mut cursor).expect("Should handle unknown chunks");

    // Should have discovered all chunks including unknown ones
    assert_eq!(discovery.chunks.len(), 3, "Should discover all chunks");
    assert_eq!(discovery.chunks[0].id.as_str(), "MVER");
    assert_eq!(discovery.chunks[1].id.as_str(), "????"); // Unknown chunk
    assert_eq!(discovery.chunks[2].id.as_str(), "MOHD");

    // Check if unknown chunks are tracked
    assert!(
        discovery.has_unknown_chunks(),
        "Should track unknown chunks"
    );
    assert_eq!(
        discovery.unknown_count(),
        1,
        "Should have one unknown chunk"
    );
}

#[test]
fn test_truncated_file_handling() {
    // Create a file that ends mid-chunk
    let mut data = Vec::new();

    // Valid MVER chunk
    data.extend_from_slice(b"REVM"); // MVER reversed
    data.extend_from_slice(&4u32.to_le_bytes()); // Size: 4
    data.extend_from_slice(&17u32.to_le_bytes()); // Version 17

    // Truncated chunk - header present but data missing
    data.extend_from_slice(b"DHOM"); // MOHD reversed
    data.extend_from_slice(&64u32.to_le_bytes()); // Size: 64
    data.extend_from_slice(&[0u8; 32]); // Only 32 bytes instead of 64

    let mut cursor = Cursor::new(&data);
    let discovery = discover_chunks(&mut cursor).expect("Should handle truncated files");

    // Should have discovered the complete chunk
    assert!(
        !discovery.chunks.is_empty(),
        "Should discover complete chunks"
    );
    assert_eq!(discovery.chunks[0].id.as_str(), "MVER");

    // Should mark file as truncated
    assert!(discovery.is_truncated(), "Should detect truncation");
}
