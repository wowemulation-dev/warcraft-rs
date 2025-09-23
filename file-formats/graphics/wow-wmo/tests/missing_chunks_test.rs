use binrw::BinWrite;
use std::io::Cursor;
use wow_wmo::api::{ParsedWmo, parse_wmo};
use wow_wmo::chunk_header::ChunkHeader;
use wow_wmo::chunk_id::ChunkId;

/// Helper to write a chunk to a buffer
fn write_chunk(buffer: &mut Vec<u8>, id: &str, data: &[u8]) {
    let header = ChunkHeader {
        id: ChunkId::from_bytes([
            id.as_bytes()[0],
            id.as_bytes()[1],
            id.as_bytes()[2],
            id.as_bytes()[3],
        ]),
        size: data.len() as u32,
    };
    let mut cursor = Cursor::new(buffer);
    cursor.set_position(cursor.get_ref().len() as u64);
    header.write(&mut cursor).unwrap();
    cursor.get_mut().extend_from_slice(data);
}

#[test]
fn test_parse_motx_chunk() {
    // MOTX contains null-terminated texture filenames
    let mut buffer = Vec::new();

    // MVER
    write_chunk(&mut buffer, "MVER", &[17, 0, 0, 0]);

    // MOHD (minimal header)
    let mohd_data = vec![0u8; 64];
    write_chunk(&mut buffer, "MOHD", &mohd_data);

    // MOTX - texture filenames
    let mut motx_data = Vec::new();
    motx_data.extend_from_slice(b"texture1.blp\0");
    motx_data.extend_from_slice(b"texture2.blp\0");
    motx_data.extend_from_slice(b"path/to/texture3.blp\0");
    write_chunk(&mut buffer, "MOTX", &motx_data);

    let mut cursor = Cursor::new(buffer);
    let result = parse_wmo(&mut cursor);

    assert!(result.is_ok());
    let wmo = result.unwrap();

    match wmo {
        ParsedWmo::Root(root) => {
            assert_eq!(root.textures.len(), 3);
            assert_eq!(root.textures[0], "texture1.blp");
            assert_eq!(root.textures[1], "texture2.blp");
            assert_eq!(root.textures[2], "path/to/texture3.blp");
        }
        _ => panic!("Expected root file"),
    }
}

#[test]
fn test_parse_mosb_chunk() {
    // MOSB contains skybox filename
    let mut buffer = Vec::new();

    // MVER
    write_chunk(&mut buffer, "MVER", &[17, 0, 0, 0]);

    // MOHD
    let mohd_data = vec![0u8; 64];
    write_chunk(&mut buffer, "MOHD", &mohd_data);

    // MOSB - skybox (optional, can be empty)
    write_chunk(&mut buffer, "MOSB", b"skybox.mdx\0");

    let mut cursor = Cursor::new(buffer);
    let result = parse_wmo(&mut cursor);

    assert!(result.is_ok());
    let wmo = result.unwrap();

    match wmo {
        ParsedWmo::Root(root) => {
            assert!(root.skybox.is_some());
            assert_eq!(root.skybox.unwrap(), "skybox.mdx");
        }
        _ => panic!("Expected root file"),
    }
}

#[test]
fn test_parse_mopv_chunk() {
    // MOPV contains portal vertices (Vec3 positions)
    let mut buffer = Vec::new();

    // MVER
    write_chunk(&mut buffer, "MVER", &[17, 0, 0, 0]);

    // MOHD with 1 portal
    let mut mohd_data = vec![0u8; 64];
    mohd_data[8..12].copy_from_slice(&1u32.to_le_bytes()); // n_portals = 1
    write_chunk(&mut buffer, "MOHD", &mohd_data);

    // MOPV - portal vertices (4 vertices for a quad)
    let mut mopv_data = Vec::new();
    // Vertex 0: (0, 0, 0)
    mopv_data.extend_from_slice(&0.0f32.to_le_bytes());
    mopv_data.extend_from_slice(&0.0f32.to_le_bytes());
    mopv_data.extend_from_slice(&0.0f32.to_le_bytes());
    // Vertex 1: (10, 0, 0)
    mopv_data.extend_from_slice(&10.0f32.to_le_bytes());
    mopv_data.extend_from_slice(&0.0f32.to_le_bytes());
    mopv_data.extend_from_slice(&0.0f32.to_le_bytes());
    // Vertex 2: (10, 10, 0)
    mopv_data.extend_from_slice(&10.0f32.to_le_bytes());
    mopv_data.extend_from_slice(&10.0f32.to_le_bytes());
    mopv_data.extend_from_slice(&0.0f32.to_le_bytes());
    // Vertex 3: (0, 10, 0)
    mopv_data.extend_from_slice(&0.0f32.to_le_bytes());
    mopv_data.extend_from_slice(&10.0f32.to_le_bytes());
    mopv_data.extend_from_slice(&0.0f32.to_le_bytes());

    write_chunk(&mut buffer, "MOPV", &mopv_data);

    let mut cursor = Cursor::new(buffer);
    let result = parse_wmo(&mut cursor);

    assert!(result.is_ok());
    let wmo = result.unwrap();

    match wmo {
        ParsedWmo::Root(root) => {
            assert_eq!(root.portal_vertices.len(), 4);
            assert_eq!(root.portal_vertices[0].x, 0.0);
            assert_eq!(root.portal_vertices[1].x, 10.0);
            assert_eq!(root.portal_vertices[2].y, 10.0);
            assert_eq!(root.portal_vertices[3].x, 0.0);
        }
        _ => panic!("Expected root file"),
    }
}

#[test]
fn test_parse_mopt_chunk() {
    // MOPT contains portal information (20 bytes per portal)
    let mut buffer = Vec::new();

    // MVER
    write_chunk(&mut buffer, "MVER", &[17, 0, 0, 0]);

    // MOHD with 1 portal
    let mut mohd_data = vec![0u8; 64];
    mohd_data[8..12].copy_from_slice(&1u32.to_le_bytes()); // n_portals = 1
    write_chunk(&mut buffer, "MOHD", &mohd_data);

    // MOPT - portal info (20 bytes)
    let mut mopt_data = Vec::new();
    mopt_data.extend_from_slice(&0u16.to_le_bytes()); // start_vertex
    mopt_data.extend_from_slice(&4u16.to_le_bytes()); // n_vertices
    mopt_data.extend_from_slice(&1.0f32.to_le_bytes()); // normal.x
    mopt_data.extend_from_slice(&0.0f32.to_le_bytes()); // normal.y
    mopt_data.extend_from_slice(&0.0f32.to_le_bytes()); // normal.z
    mopt_data.extend_from_slice(&5.0f32.to_le_bytes()); // distance

    write_chunk(&mut buffer, "MOPT", &mopt_data);

    let mut cursor = Cursor::new(buffer);
    let result = parse_wmo(&mut cursor);

    assert!(result.is_ok());
    let wmo = result.unwrap();

    match wmo {
        ParsedWmo::Root(root) => {
            assert_eq!(root.portals.len(), 1);
            assert_eq!(root.portals[0].start_vertex, 0);
            assert_eq!(root.portals[0].n_vertices, 4);
            assert_eq!(root.portals[0].normal.x, 1.0);
            assert_eq!(root.portals[0].distance, 5.0);
        }
        _ => panic!("Expected root file"),
    }
}

#[test]
fn test_parse_mopr_chunk() {
    // MOPR contains portal references (8 bytes per reference)
    let mut buffer = Vec::new();

    // MVER
    write_chunk(&mut buffer, "MVER", &[17, 0, 0, 0]);

    // MOHD
    let mohd_data = vec![0u8; 64];
    write_chunk(&mut buffer, "MOHD", &mohd_data);

    // MOPR - portal references
    let mut mopr_data = Vec::new();
    mopr_data.extend_from_slice(&1u16.to_le_bytes()); // portal_index
    mopr_data.extend_from_slice(&5u16.to_le_bytes()); // group_index
    mopr_data.extend_from_slice(&1i16.to_le_bytes()); // side (1 = positive, -1 = negative)
    mopr_data.extend_from_slice(&0u16.to_le_bytes()); // padding

    write_chunk(&mut buffer, "MOPR", &mopr_data);

    let mut cursor = Cursor::new(buffer);
    let result = parse_wmo(&mut cursor);

    assert!(result.is_ok());
    let wmo = result.unwrap();

    match wmo {
        ParsedWmo::Root(root) => {
            assert_eq!(root.portal_refs.len(), 1);
            assert_eq!(root.portal_refs[0].portal_index, 1);
            assert_eq!(root.portal_refs[0].group_index, 5);
            assert_eq!(root.portal_refs[0].side, 1);
        }
        _ => panic!("Expected root file"),
    }
}

#[test]
fn test_parse_modn_chunk() {
    // MODN contains doodad names (null-terminated M2 filenames)
    let mut buffer = Vec::new();

    // MVER
    write_chunk(&mut buffer, "MVER", &[17, 0, 0, 0]);

    // MOHD with doodad counts
    let mut mohd_data = vec![0u8; 64];
    mohd_data[16..20].copy_from_slice(&3u32.to_le_bytes()); // n_doodad_names = 3
    write_chunk(&mut buffer, "MOHD", &mohd_data);

    // MODN - doodad names
    let mut modn_data = Vec::new();
    modn_data.extend_from_slice(
        b"World\\Kalimdor\\Orgrimmar\\PassiveDoodads\\Braziers\\OrcBrazier.m2\0",
    );
    modn_data.extend_from_slice(b"World\\Generic\\Human\\Passive Doodads\\Furniture\\Chair01.m2\0");
    modn_data.extend_from_slice(b"World\\Generic\\Orc\\Passive Doodads\\Banners\\OrcBanner01.m2\0");

    write_chunk(&mut buffer, "MODN", &modn_data);

    let mut cursor = Cursor::new(buffer);
    let result = parse_wmo(&mut cursor);

    assert!(result.is_ok());
    let wmo = result.unwrap();

    match wmo {
        ParsedWmo::Root(root) => {
            assert_eq!(root.doodad_names.len(), 3);
            assert!(root.doodad_names[0].ends_with("OrcBrazier.m2"));
            assert!(root.doodad_names[1].ends_with("Chair01.m2"));
            assert!(root.doodad_names[2].ends_with("OrcBanner01.m2"));
        }
        _ => panic!("Expected root file"),
    }
}

#[test]
fn test_parse_movv_movb_chunks() {
    // MOVV contains visible block vertices, MOVB contains block definitions
    // These are often empty but we should handle them
    let mut buffer = Vec::new();

    // MVER
    write_chunk(&mut buffer, "MVER", &[17, 0, 0, 0]);

    // MOHD
    let mohd_data = vec![0u8; 64];
    write_chunk(&mut buffer, "MOHD", &mohd_data);

    // MOVV - visible block vertices (empty is common)
    write_chunk(&mut buffer, "MOVV", &[]);

    // MOVB - visible block list (empty is common)
    write_chunk(&mut buffer, "MOVB", &[]);

    let mut cursor = Cursor::new(buffer);
    let result = parse_wmo(&mut cursor);

    assert!(result.is_ok());
    let wmo = result.unwrap();

    match wmo {
        ParsedWmo::Root(root) => {
            assert_eq!(root.visible_vertices.len(), 0);
            assert_eq!(root.visible_blocks.len(), 0);
        }
        _ => panic!("Expected root file"),
    }
}
