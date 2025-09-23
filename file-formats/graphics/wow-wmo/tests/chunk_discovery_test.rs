use binrw::BinRead;
use binrw::io::Cursor;
use std::fs::File;
use std::io::BufReader;
use wow_wmo::{ChunkHeader, ChunkId};

#[test]
fn test_chunk_id_creation() {
    // Test creating a ChunkId from bytes
    let id = ChunkId::from_bytes(*b"MVER");
    assert_eq!(id.as_str(), "MVER");

    // Test that ChunkId stores bytes correctly
    let id2 = ChunkId::from_bytes(*b"MOHD");
    assert_eq!(id2.as_str(), "MOHD");
}

#[test]
fn test_chunk_header_parsing() {
    // MVER chunk header as it appears in file (reversed + size)
    let data = vec![
        b'R', b'E', b'V', b'M', // "REVM" (MVER reversed)
        0x04, 0x00, 0x00, 0x00, // Size: 4 bytes (little-endian)
    ];

    let mut cursor = Cursor::new(&data);
    let header = ChunkHeader::read(&mut cursor).expect("Failed to parse chunk header");

    assert_eq!(header.id.as_str(), "MVER");
    assert_eq!(header.size, 4);
}

#[test]
fn test_chunk_header_with_larger_size() {
    // MOHD chunk header
    let data = vec![
        b'D', b'H', b'O', b'M', // "DHOM" (MOHD reversed)
        0x40, 0x00, 0x00, 0x00, // Size: 64 bytes (little-endian)
    ];

    let mut cursor = Cursor::new(&data);
    let header = ChunkHeader::read(&mut cursor).expect("Failed to parse chunk header");

    assert_eq!(header.id.as_str(), "MOHD");
    assert_eq!(header.size, 64);
}

#[test]
fn test_chunk_discovery_in_file() {
    use wow_wmo::chunk_discovery::discover_chunks;

    // Test with a real WMO file from Classic
    let test_file = "tests/data/vanilla/World/wmo/Azeroth/Buildings/Stormwind/Stormwind.wmo";
    let file =
        File::open(test_file).unwrap_or_else(|_| panic!("Test file not found: {}", test_file));
    let mut reader = BufReader::new(file);

    let discovery = discover_chunks(&mut reader).expect("Failed to discover chunks");

    // Root files should have multiple chunks
    assert!(
        discovery.chunks.len() > 5,
        "Expected multiple chunks in root file"
    );

    // First chunk should be MVER
    assert_eq!(discovery.chunks[0].id.as_str(), "MVER");

    // Should contain essential root chunks
    let chunk_ids: Vec<&str> = discovery.chunks.iter().map(|c| c.id.as_str()).collect();
    assert!(chunk_ids.contains(&"MOHD"), "Missing MOHD chunk");
    assert!(chunk_ids.contains(&"MOMT"), "Missing MOMT chunk");
}

#[test]
fn test_chunk_discovery_in_group_file() {
    use wow_wmo::chunk_discovery::discover_chunks;

    // Test with a group file from Classic
    let test_file = "tests/data/vanilla/World/wmo/Azeroth/Buildings/Stormwind/Stormwind_000.wmo";
    let file =
        File::open(test_file).unwrap_or_else(|_| panic!("Test file not found: {}", test_file));
    let mut reader = BufReader::new(file);

    let discovery = discover_chunks(&mut reader).expect("Failed to discover chunks");

    // Group files should have at least one chunk (MOGP)
    assert!(
        !discovery.chunks.is_empty(),
        "Expected at least one chunk in group file"
    );

    // First chunk should be MVER
    assert_eq!(discovery.chunks[0].id.as_str(), "MVER");
}
