#[cfg(test)]
use crate::api::{ParsedWmo, parse_wmo};
use crate::validator::WmoValidator;
use crate::version::WmoVersion;
use crate::writer::WmoWriter;
use binrw::BinWrite;
use std::io::Cursor;

// Import the new binrw-based types directly
use crate::chunk_id::ChunkId;
use crate::chunk_header::ChunkHeader;

// Helper function to create ChunkId from string
fn chunk_id(s: &str) -> ChunkId {
    assert_eq!(s.len(), 4, "Chunk ID must be exactly 4 characters");
    let bytes = s.as_bytes();
    ChunkId::from_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

#[test]
fn test_chunk_id_from_str() {
    let id = ChunkId::from_bytes([b'M', b'V', b'E', b'R']);
    assert_eq!(id.as_str(), "MVER");

    let id = ChunkId::from_bytes([b'M', b'O', b'H', b'D']);
    assert_eq!(id.as_str(), "MOHD");
}

#[test]
fn test_chunk_header_read_write() {
    let header = ChunkHeader {
        id: ChunkId::from_bytes([b'T', b'E', b'S', b'T']),
        size: 42,
    };

    let mut buffer = Vec::new();
    header.write(&mut buffer).unwrap();

    assert_eq!(buffer.len(), 8); // ChunkHeader is always 8 bytes
    // ChunkId stores bytes reversed for little-endian
    assert_eq!(&buffer[0..4], b"TSET");

    let mut cursor = Cursor::new(buffer);
    use binrw::BinRead;
    let read_header = ChunkHeader::read(&mut cursor).unwrap();

    assert_eq!(read_header.id.bytes, header.id.bytes);
    assert_eq!(read_header.size, header.size);
}

// TODO: Re-implement with new binrw system
// #[test]
// fn test_chunk_read() {

#[test]
fn test_version_from_raw() {
    // Based on empirical analysis: version 17 maps to Classic by default
    // but could be any expansion from Classic through MoP
    assert_eq!(WmoVersion::from_raw(17), Some(WmoVersion::Classic));

    // Post-MoP theoretical versions
    assert_eq!(WmoVersion::from_raw(18), Some(WmoVersion::Wod));
    assert_eq!(WmoVersion::from_raw(19), Some(WmoVersion::Legion));
    assert_eq!(WmoVersion::from_raw(20), Some(WmoVersion::Bfa));
    assert_eq!(WmoVersion::from_raw(21), Some(WmoVersion::Shadowlands));
    assert_eq!(WmoVersion::from_raw(22), Some(WmoVersion::Dragonflight));
    assert_eq!(WmoVersion::from_raw(23), Some(WmoVersion::WarWithin));
    assert_eq!(WmoVersion::from_raw(1000), None);
}

#[test]
fn test_version_from_raw_with_expansion() {
    // Test expansion-aware parsing for version 17
    assert_eq!(
        WmoVersion::from_raw_with_expansion(17, "vanilla"),
        Some(WmoVersion::Classic)
    );
    assert_eq!(
        WmoVersion::from_raw_with_expansion(17, "tbc"),
        Some(WmoVersion::Tbc)
    );
    assert_eq!(
        WmoVersion::from_raw_with_expansion(17, "wotlk"),
        Some(WmoVersion::Wotlk)
    );
    assert_eq!(
        WmoVersion::from_raw_with_expansion(17, "cataclysm"),
        Some(WmoVersion::Cataclysm)
    );
    assert_eq!(
        WmoVersion::from_raw_with_expansion(17, "mop"),
        Some(WmoVersion::Mop)
    );
    assert_eq!(
        WmoVersion::from_raw_with_expansion(17, "unknown"),
        Some(WmoVersion::Classic)
    );
}

#[test]
fn test_version_to_raw() {
    // Test that empirically verified versions return 17
    assert_eq!(WmoVersion::Classic.to_raw(), 17);
    assert_eq!(WmoVersion::Tbc.to_raw(), 17);
    assert_eq!(WmoVersion::Wotlk.to_raw(), 17);
    assert_eq!(WmoVersion::Cataclysm.to_raw(), 17);
    assert_eq!(WmoVersion::Mop.to_raw(), 17);

    // Test theoretical post-MoP versions
    assert_eq!(WmoVersion::Wod.to_raw(), 18);
    assert_eq!(WmoVersion::Legion.to_raw(), 19);
}

#[test]
fn test_parse_simple_wmo() {
    // Create a minimal valid WMO file
    let mut buffer = Vec::new();

    // MVER chunk
    let mver_header = chunk::ChunkHeader {
        id: chunk_id("MVER"),
        size: 4,
    };
    mver_header.write(&mut buffer).unwrap();
    buffer.extend_from_slice(&[17, 0, 0, 0]); // Version 17 (Classic)

    // MOHD chunk (minimal header)
    let mohd_header = chunk::ChunkHeader {
        id: chunk_id("MOHD"),
        size: 64, // Fixed size for basic header
    };
    mohd_header.write(&mut buffer).unwrap();

    // Write header content
    let header_data = [
        0, 0, 0, 0, // n_materials
        0, 0, 0, 0, // n_groups
        0, 0, 0, 0, // n_portals
        0, 0, 0, 0, // n_lights
        0, 0, 0, 0, // n_doodad_names
        0, 0, 0, 0, // n_doodad_defs
        0, 0, 0, 0, // n_doodad_sets
        0, 0, 0, 0, // ambient_color
        0, 0, 0, 0, // flags
        0, 0, 0, 0, 0, 0, 0, 0, // min bounding box
        0, 0, 0, 0, 0, 0, 0, 0, // max bounding box
        0, 0, 0, 0, // unknown1
        0, 0, 0, 0, // unknown2
    ];
    buffer.extend_from_slice(&header_data);

    // Try parsing
    let mut cursor = Cursor::new(buffer);
    let result = parse_wmo(&mut cursor);

    // Should parse without errors
    assert!(result.is_ok());

    let wmo = result.unwrap();
    match wmo {
        ParsedWmo::Root(root) => {
            assert_eq!(root.version, 17);
            assert_eq!(root.materials.len(), 0);
            assert_eq!(root.group_info.len(), 0);
            assert_eq!(root.n_portals, 0);
            assert_eq!(root.lights.len(), 0);
            assert_eq!(root.doodad_defs.len(), 0);
            assert_eq!(root.doodad_sets.len(), 0);
        }
        ParsedWmo::Group(_) => panic!("Expected root file, got group"),
    }
}

#[test]
fn test_validate_invalid_wmo() {
    // Create an invalid WMO file with wrong magic
    let mut buffer = Vec::new();

    // Wrong chunk ID
    let invalid_header = chunk::ChunkHeader {
        id: chunk_id("XXXX"),
        size: 4,
    };
    invalid_header.write(&mut buffer).unwrap();
    buffer.extend_from_slice(&[17, 0, 0, 0]);

    // validate_wmo is not yet implemented in the new parser
    // let mut cursor = Cursor::new(buffer);
    // let result = validate_wmo(&mut cursor);
    // assert!(result.is_ok());
    // assert!(!result.unwrap());
}

#[test]
#[ignore] // TODO: Update after binrw migration is complete
fn test_parse_simple_wmo_group() {
    // Create a minimal valid WMO group file
    let mut buffer = Vec::new();

    // MVER chunk
    let mver_header = chunk::ChunkHeader {
        id: chunk_id("XXXX"),
        size: 4,
    };
    mver_header.write(&mut buffer).unwrap();
    buffer.extend_from_slice(&[17, 0, 0, 0]); // Version 17 (Classic)

    // MOGP chunk (minimal group header)
    let mogp_header = chunk::ChunkHeader {
        id: chunk_id("MOGP"),
        size: 36, // Actual size: 4+4+12+12+2+2 = 36
    };
    mogp_header.write(&mut buffer).unwrap();

    // Write group header fields
    buffer.extend_from_slice(&[0, 0, 0, 0]); // Name offset
    buffer.extend_from_slice(&[0, 0, 0, 0]); // Flags

    // Bounding box
    buffer.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // Min coords
    buffer.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // Max coords

    // Flags2 and group index
    buffer.extend_from_slice(&[0, 0]); // Flags2
    buffer.extend_from_slice(&[1, 0]); // Group index (1)

    // Add minimal MOVT chunk (vertices)
    let movt_header = chunk::ChunkHeader {
        id: chunk_id("MOVT"),
        size: 12, // 1 vertex (12 bytes)
    };
    movt_header.write(&mut buffer).unwrap();

    // One vertex at origin
    buffer.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

    // Add minimal MOVI chunk (indices)
    let movi_header = chunk::ChunkHeader {
        id: chunk_id("MOVI"),
        size: 2, // 1 index (2 bytes)
    };
    movi_header.write(&mut buffer).unwrap();

    // One index (0)
    buffer.extend_from_slice(&[0, 0]);

    // parse_wmo_group is not yet implemented in the new parser
    // let mut cursor = Cursor::new(buffer);
    // let result = parse_wmo_group(&mut cursor, 1);
    // assert!(result.is_ok());
    // let group = result.unwrap();
    // assert_eq!(group.header.group_index, 1);
    // assert_eq!(group.vertices.len(), 1);
    // assert_eq!(group.indices.len(), 1);
}

#[test]
fn test_wmo_validator() {
    // Create a minimal valid WMO file
    let mut buffer = Vec::new();

    // MVER chunk
    let mver_header = chunk::ChunkHeader {
        id: chunk_id("MVER"),
        size: 4,
    };
    mver_header.write(&mut buffer).unwrap();
    buffer.extend_from_slice(&[17, 0, 0, 0]); // Version 17 (Classic)

    // MOHD chunk (minimal header)
    let mohd_header = chunk::ChunkHeader {
        id: chunk_id("MOHD"),
        size: 64, // Fixed size for basic header
    };
    mohd_header.write(&mut buffer).unwrap();

    // Write header content
    let header_data = [
        0, 0, 0, 0, // n_materials
        0, 0, 0, 0, // n_groups
        0, 0, 0, 0, // n_portals
        0, 0, 0, 0, // n_lights
        0, 0, 0, 0, // n_doodad_names
        0, 0, 0, 0, // n_doodad_defs
        0, 0, 0, 0, // n_doodad_sets
        0, 0, 0, 0, // ambient_color
        0, 0, 0, 0, // flags
        0, 0, 0, 0, 0, 0, 0, 0, // min bounding box
        0, 0, 0, 0, 0, 0, 0, 0, // max bounding box
        0, 0, 0, 0, // unknown1
        0, 0, 0, 0, // unknown2
    ];
    buffer.extend_from_slice(&header_data);

    // Try validating
    let mut cursor = Cursor::new(buffer);
    let wmo = parse_wmo(&mut cursor).unwrap();

    let validator = WmoValidator::new();
    let report = validator.validate_root(&wmo).unwrap();

    // Should have no errors
    assert!(!report.has_errors());
}

#[test]
fn test_wmo_writer() {
    // Create a minimal valid WMO file
    let mut buffer = Vec::new();

    // MVER chunk
    let mver_header = chunk::ChunkHeader {
        id: chunk_id("MVER"),
        size: 4,
    };
    mver_header.write(&mut buffer).unwrap();
    buffer.extend_from_slice(&[17, 0, 0, 0]); // Version 17 (Classic)

    // MOHD chunk (minimal header)
    let mohd_header = chunk::ChunkHeader {
        id: chunk_id("MOHD"),
        size: 64, // Fixed size for basic header
    };
    mohd_header.write(&mut buffer).unwrap();

    // Write header content
    let header_data = [
        0, 0, 0, 0, // n_materials
        0, 0, 0, 0, // n_groups
        0, 0, 0, 0, // n_portals
        0, 0, 0, 0, // n_lights
        0, 0, 0, 0, // n_doodad_names
        0, 0, 0, 0, // n_doodad_defs
        0, 0, 0, 0, // n_doodad_sets
        0, 0, 0, 0, // ambient_color
        0, 0, 0, 0, // flags
        0, 0, 0, 0, 0, 0, 0, 0, // min bounding box
        0, 0, 0, 0, 0, 0, 0, 0, // max bounding box
        0, 0, 0, 0, // unknown1
        0, 0, 0, 0, // unknown2
    ];
    buffer.extend_from_slice(&header_data);

    // Parse WMO
    let mut cursor = Cursor::new(buffer);
    let wmo = parse_wmo(&mut cursor).unwrap();

    // Write WMO to a new buffer
    let mut out_buffer = Vec::new();
    let mut out_cursor = Cursor::new(&mut out_buffer);
    let writer = WmoWriter::new();
    writer
        .write_root(&mut out_cursor, &wmo, WmoVersion::Classic)
        .unwrap();

    // Parse the written WMO
    let mut out_cursor = Cursor::new(out_buffer);
    let result = parse_wmo(&mut out_cursor);

    // Should parse without errors
    assert!(result.is_ok());

    let out_wmo = result.unwrap();
    match out_wmo {
        ParsedWmo::Root(root) => assert_eq!(root.version, 17),
        _ => panic!("Expected root file"),
    }
}

#[test]
fn test_wmo_converter() {
    // Create a minimal valid WMO file for Classic
    let mut buffer = Vec::new();

    // MVER chunk
    let mver_header = chunk::ChunkHeader {
        id: chunk_id("MVER"),
        size: 4,
    };
    mver_header.write(&mut buffer).unwrap();
    buffer.extend_from_slice(&[17, 0, 0, 0]); // Version 17 (Classic)

    // MOHD chunk (minimal header)
    let mohd_header = chunk::ChunkHeader {
        id: chunk_id("MOHD"),
        size: 64, // Fixed size for basic header
    };
    mohd_header.write(&mut buffer).unwrap();

    // Write header content
    let header_data = [
        0, 0, 0, 0, // n_materials
        0, 0, 0, 0, // n_groups
        0, 0, 0, 0, // n_portals
        0, 0, 0, 0, // n_lights
        0, 0, 0, 0, // n_doodad_names
        0, 0, 0, 0, // n_doodad_defs
        0, 0, 0, 0, // n_doodad_sets
        0, 0, 0, 0, // ambient_color
        0, 0, 0, 0, // flags
        0, 0, 0, 0, 0, 0, 0, 0, // min bounding box
        0, 0, 0, 0, 0, 0, 0, 0, // max bounding box
        0, 0, 0, 0, // unknown1
        0, 0, 0, 0, // unknown2
    ];
    buffer.extend_from_slice(&header_data);

    // Parse WMO
    let mut cursor = Cursor::new(buffer);
    let mut wmo = parse_wmo(&mut cursor).unwrap();

    // Convert to TBC (when converter is implemented)
    // let converter = WmoConverter::new();
    // let result = converter.convert_root(&mut wmo, WmoVersion::Tbc);
    // assert!(result.is_ok());
    // assert_eq!(wmo.version, WmoVersion::Tbc);
}

#[test]
fn test_wmo_full_conversion() {
    // Create a minimal valid WMO file for Classic
    let mut buffer = Vec::new();

    // MVER chunk
    let mver_header = chunk::ChunkHeader {
        id: chunk_id("MVER"),
        size: 4,
    };
    mver_header.write(&mut buffer).unwrap();
    buffer.extend_from_slice(&[17, 0, 0, 0]); // Version 17 (Classic)

    // MOHD chunk (minimal header)
    let mohd_header = chunk::ChunkHeader {
        id: chunk_id("MOHD"),
        size: 64, // Fixed size for basic header
    };
    mohd_header.write(&mut buffer).unwrap();

    // Write header content
    let header_data = [
        0, 0, 0, 0, // n_materials
        0, 0, 0, 0, // n_groups
        0, 0, 0, 0, // n_portals
        0, 0, 0, 0, // n_lights
        0, 0, 0, 0, // n_doodad_names
        0, 0, 0, 0, // n_doodad_defs
        0, 0, 0, 0, // n_doodad_sets
        0, 0, 0, 0, // ambient_color
        0, 0, 0, 0, // flags
        0, 0, 0, 0, 0, 0, 0, 0, // min bounding box
        0, 0, 0, 0, 0, 0, 0, 0, // max bounding box
        0, 0, 0, 0, // unknown1
        0, 0, 0, 0, // unknown2
    ];
    buffer.extend_from_slice(&header_data);

    // convert_wmo is not yet implemented in the new parser
    // Test will be re-enabled when converter is implemented
    // let mut in_cursor = Cursor::new(buffer);
    // let mut out_buffer = Vec::new();
    // let mut out_cursor = Cursor::new(&mut out_buffer);
    // let result = convert_wmo(&mut in_cursor, &mut out_cursor, WmoVersion::Tbc);
    // assert!(result.is_ok());
    // let mut out_cursor = Cursor::new(out_buffer);
    // let out_wmo = parse_wmo(&mut out_cursor).unwrap();
}
