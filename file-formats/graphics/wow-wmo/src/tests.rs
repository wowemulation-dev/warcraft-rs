#[cfg(test)]
use crate::converter::WmoConverter;
use crate::parser::chunks;
use crate::types::ChunkId;
use crate::validator::WmoValidator;
use crate::writer::WmoWriter;
use crate::*;
use std::io::Cursor;

#[test]
fn test_chunk_id_from_str() {
    let id = ChunkId::from_str("MVER");
    assert_eq!(id.0, [b'M', b'V', b'E', b'R']);

    let id = ChunkId::from_str("MOHD");
    assert_eq!(id.0, [b'M', b'O', b'H', b'D']);
}

#[test]
fn test_chunk_header_read_write() {
    let header = chunk::ChunkHeader {
        id: ChunkId::from_str("TEST"),
        size: 42,
    };

    let mut buffer = Vec::new();
    header.write(&mut buffer).unwrap();

    assert_eq!(buffer.len(), chunk::ChunkHeader::SIZE);
    // WMO files store chunk IDs in little-endian order, so bytes are reversed
    assert_eq!(&buffer[0..4], b"TSET");

    let mut cursor = Cursor::new(buffer);
    let read_header = chunk::ChunkHeader::read(&mut cursor).unwrap();

    assert_eq!(read_header.id.0, header.id.0);
    assert_eq!(read_header.size, header.size);
}

#[test]
fn test_chunk_read() {
    // Create a test chunk
    let mut buffer = Vec::new();

    // Write header
    let header = chunk::ChunkHeader {
        id: ChunkId::from_str("DATA"),
        size: 4,
    };
    header.write(&mut buffer).unwrap();

    // Write data
    buffer.extend_from_slice(&[1, 2, 3, 4]);

    let mut cursor = Cursor::new(buffer);
    let chunk = chunk::Chunk::read(&mut cursor).unwrap();

    assert_eq!(chunk.header.id.0, [b'D', b'A', b'T', b'A']);
    assert_eq!(chunk.header.size, 4);

    // Read the data
    cursor.set_position(0);
    let chunk = chunk::Chunk::read(&mut cursor).unwrap();
    let data = chunk.read_data(&mut cursor).unwrap();

    assert_eq!(data, vec![1, 2, 3, 4]);
}

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
        id: chunks::MVER,
        size: 4,
    };
    mver_header.write(&mut buffer).unwrap();
    buffer.extend_from_slice(&[17, 0, 0, 0]); // Version 17 (Classic)

    // MOHD chunk (minimal header)
    let mohd_header = chunk::ChunkHeader {
        id: chunks::MOHD,
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
    assert_eq!(wmo.version, WmoVersion::Classic);
    assert_eq!(wmo.materials.len(), 0);
    assert_eq!(wmo.groups.len(), 0);
    assert_eq!(wmo.portals.len(), 0);
    assert_eq!(wmo.lights.len(), 0);
    assert_eq!(wmo.doodad_defs.len(), 0);
    assert_eq!(wmo.doodad_sets.len(), 0);
}

#[test]
fn test_validate_invalid_wmo() {
    // Create an invalid WMO file with wrong magic
    let mut buffer = Vec::new();

    // Wrong chunk ID
    let invalid_header = chunk::ChunkHeader {
        id: ChunkId::from_str("XXXX"),
        size: 4,
    };
    invalid_header.write(&mut buffer).unwrap();
    buffer.extend_from_slice(&[17, 0, 0, 0]);

    let mut cursor = Cursor::new(buffer);
    let result = validate_wmo(&mut cursor);

    // Should return false for invalid WMO
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[test]
fn test_parse_simple_wmo_group() {
    // Create a minimal valid WMO group file
    let mut buffer = Vec::new();

    // MVER chunk
    let mver_header = chunk::ChunkHeader {
        id: chunks::MVER,
        size: 4,
    };
    mver_header.write(&mut buffer).unwrap();
    buffer.extend_from_slice(&[17, 0, 0, 0]); // Version 17 (Classic)

    // MOGP chunk (minimal group header)
    let mogp_header = chunk::ChunkHeader {
        id: chunks::MOGP,
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
        id: chunks::MOVT,
        size: 12, // 1 vertex (12 bytes)
    };
    movt_header.write(&mut buffer).unwrap();

    // One vertex at origin
    buffer.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

    // Add minimal MOVI chunk (indices)
    let movi_header = chunk::ChunkHeader {
        id: chunks::MOVI,
        size: 2, // 1 index (2 bytes)
    };
    movi_header.write(&mut buffer).unwrap();

    // One index (0)
    buffer.extend_from_slice(&[0, 0]);

    // Try parsing
    let mut cursor = Cursor::new(buffer);
    let result = parse_wmo_group(&mut cursor, 1);

    // Should parse without errors
    assert!(result.is_ok());

    let group = result.unwrap();
    assert_eq!(group.header.group_index, 1);
    assert_eq!(group.vertices.len(), 1);
    assert_eq!(group.indices.len(), 1);
}

#[test]
fn test_wmo_validator() {
    // Create a minimal valid WMO file
    let mut buffer = Vec::new();

    // MVER chunk
    let mver_header = chunk::ChunkHeader {
        id: chunks::MVER,
        size: 4,
    };
    mver_header.write(&mut buffer).unwrap();
    buffer.extend_from_slice(&[17, 0, 0, 0]); // Version 17 (Classic)

    // MOHD chunk (minimal header)
    let mohd_header = chunk::ChunkHeader {
        id: chunks::MOHD,
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
        id: chunks::MVER,
        size: 4,
    };
    mver_header.write(&mut buffer).unwrap();
    buffer.extend_from_slice(&[17, 0, 0, 0]); // Version 17 (Classic)

    // MOHD chunk (minimal header)
    let mohd_header = chunk::ChunkHeader {
        id: chunks::MOHD,
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
    assert_eq!(out_wmo.version, WmoVersion::Classic);
}

#[test]
fn test_wmo_converter() {
    // Create a minimal valid WMO file for Classic
    let mut buffer = Vec::new();

    // MVER chunk
    let mver_header = chunk::ChunkHeader {
        id: chunks::MVER,
        size: 4,
    };
    mver_header.write(&mut buffer).unwrap();
    buffer.extend_from_slice(&[17, 0, 0, 0]); // Version 17 (Classic)

    // MOHD chunk (minimal header)
    let mohd_header = chunk::ChunkHeader {
        id: chunks::MOHD,
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

    // Convert to TBC
    let converter = WmoConverter::new();
    let result = converter.convert_root(&mut wmo, WmoVersion::Tbc);

    // Should convert without errors
    assert!(result.is_ok());
    assert_eq!(wmo.version, WmoVersion::Tbc);
}

#[test]
fn test_wmo_full_conversion() {
    // Create a minimal valid WMO file for Classic
    let mut buffer = Vec::new();

    // MVER chunk
    let mver_header = chunk::ChunkHeader {
        id: chunks::MVER,
        size: 4,
    };
    mver_header.write(&mut buffer).unwrap();
    buffer.extend_from_slice(&[17, 0, 0, 0]); // Version 17 (Classic)

    // MOHD chunk (minimal header)
    let mohd_header = chunk::ChunkHeader {
        id: chunks::MOHD,
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

    // Use the convert_wmo function
    let mut in_cursor = Cursor::new(buffer);
    let mut out_buffer = Vec::new();
    let mut out_cursor = Cursor::new(&mut out_buffer);

    let result = convert_wmo(&mut in_cursor, &mut out_cursor, WmoVersion::Tbc);

    // Should convert without errors
    assert!(result.is_ok());

    // Verify the output
    let mut out_cursor = Cursor::new(out_buffer);
    let out_wmo = parse_wmo(&mut out_cursor).unwrap();

    // Note: Both Classic and TBC use version 17, so parsing will return Classic
    // Feature differentiation is done through chunk presence, not version numbers
    assert_eq!(out_wmo.version, WmoVersion::Classic); // Expected: parses as Classic due to version 17

    // The actual conversion logic should be tested through feature availability
    // rather than version number comparison
}
