#[cfg(test)]
mod tests {
    use crate::chunk_discovery::discover_chunks;
    use crate::chunk_header::ChunkHeader;
    use crate::chunk_id::ChunkId;
    use crate::root_parser::parse_root_file;
    use binrw::BinWrite;
    use std::io::{Cursor, Seek, Write};

    #[test]
    fn test_motx_chunk_parsing() {
        // Create MOTX chunk with texture filenames
        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);

        // Write MVER chunk (always first)
        let mver_header = ChunkHeader {
            id: ChunkId::from_bytes([b'M', b'V', b'E', b'R']),
            size: 4,
        };
        mver_header.write(&mut cursor).unwrap();
        cursor.write_all(&17u32.to_le_bytes()).unwrap(); // Version 17

        // Write MOHD chunk (required)
        let mohd_header = ChunkHeader {
            id: ChunkId::from_bytes([b'M', b'O', b'H', b'D']),
            size: 64,
        };
        mohd_header.write(&mut cursor).unwrap();
        cursor.write_all(&[0u8; 64]).unwrap(); // Empty MOHD

        // Write MOTX chunk with texture data
        let textures = b"texture1.blp\0texture2.blp\0";
        let motx_header = ChunkHeader {
            id: ChunkId::from_bytes([b'M', b'O', b'T', b'X']),
            size: textures.len() as u32,
        };
        motx_header.write(&mut cursor).unwrap();
        cursor.write_all(textures).unwrap();

        // Parse the data - cursor goes out of scope
        // Create new cursor to read the data
        let mut read_cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut read_cursor).unwrap();
        read_cursor.seek(std::io::SeekFrom::Start(0)).unwrap();
        let root = parse_root_file(&mut read_cursor, discovery).unwrap();

        // Verify textures were parsed correctly
        assert_eq!(root.textures.len(), 2);
        assert_eq!(root.textures[0], "texture1.blp");
        assert_eq!(root.textures[1], "texture2.blp");
    }

    #[test]
    fn test_mosb_chunk_parsing() {
        // Create MOSB chunk with skybox name
        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);

        // Write MVER chunk
        let mver_header = ChunkHeader {
            id: ChunkId::from_bytes([b'M', b'V', b'E', b'R']),
            size: 4,
        };
        mver_header.write(&mut cursor).unwrap();
        cursor.write_all(&17u32.to_le_bytes()).unwrap();

        // Write MOHD chunk
        let mohd_header = ChunkHeader {
            id: ChunkId::from_bytes([b'M', b'O', b'H', b'D']),
            size: 64,
        };
        mohd_header.write(&mut cursor).unwrap();
        cursor.write_all(&[0u8; 64]).unwrap();

        // Write MOSB chunk with skybox data
        let skybox = b"skybox.blp\0";
        let mosb_header = ChunkHeader {
            id: ChunkId::from_bytes([b'M', b'O', b'S', b'B']),
            size: skybox.len() as u32,
        };
        mosb_header.write(&mut cursor).unwrap();
        cursor.write_all(skybox).unwrap();

        // Parse the data - cursor goes out of scope
        // Create new cursor to read the data
        let mut read_cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut read_cursor).unwrap();
        read_cursor.seek(std::io::SeekFrom::Start(0)).unwrap();
        let root = parse_root_file(&mut read_cursor, discovery).unwrap();

        // Verify skybox was parsed correctly
        assert_eq!(root.skybox, Some("skybox.blp".to_string()));
    }

    #[test]
    fn test_mopv_chunk_parsing() {
        // Create MOPV chunk with portal vertices
        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);

        // Write MVER chunk
        let mver_header = ChunkHeader {
            id: ChunkId::from_bytes([b'M', b'V', b'E', b'R']),
            size: 4,
        };
        mver_header.write(&mut cursor).unwrap();
        cursor.write_all(&17u32.to_le_bytes()).unwrap();

        // Write MOHD chunk
        let mohd_header = ChunkHeader {
            id: ChunkId::from_bytes([b'M', b'O', b'H', b'D']),
            size: 64,
        };
        mohd_header.write(&mut cursor).unwrap();
        cursor.write_all(&[0u8; 64]).unwrap();

        // Write MOPV chunk with vertices
        let mopv_header = ChunkHeader {
            id: ChunkId::from_bytes([b'M', b'O', b'P', b'V']),
            size: 24, // 2 vertices * 12 bytes each
        };
        mopv_header.write(&mut cursor).unwrap();
        // Vertex 1: (1.0, 2.0, 3.0)
        cursor.write_all(&1.0f32.to_le_bytes()).unwrap();
        cursor.write_all(&2.0f32.to_le_bytes()).unwrap();
        cursor.write_all(&3.0f32.to_le_bytes()).unwrap();
        // Vertex 2: (4.0, 5.0, 6.0)
        cursor.write_all(&4.0f32.to_le_bytes()).unwrap();
        cursor.write_all(&5.0f32.to_le_bytes()).unwrap();
        cursor.write_all(&6.0f32.to_le_bytes()).unwrap();

        // Parse the data - cursor goes out of scope
        // Create new cursor to read the data
        let mut read_cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut read_cursor).unwrap();
        read_cursor.seek(std::io::SeekFrom::Start(0)).unwrap();
        let root = parse_root_file(&mut read_cursor, discovery).unwrap();

        // Verify portal vertices were parsed correctly
        assert_eq!(root.portal_vertices.len(), 2);
        assert_eq!(root.portal_vertices[0].x, 1.0);
        assert_eq!(root.portal_vertices[0].y, 2.0);
        assert_eq!(root.portal_vertices[0].z, 3.0);
        assert_eq!(root.portal_vertices[1].x, 4.0);
        assert_eq!(root.portal_vertices[1].y, 5.0);
        assert_eq!(root.portal_vertices[1].z, 6.0);
    }

    #[test]
    fn test_mopt_chunk_parsing() {
        // Create MOPT chunk with portal information
        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);

        // Write MVER chunk
        let mver_header = ChunkHeader {
            id: ChunkId::from_bytes([b'M', b'V', b'E', b'R']),
            size: 4,
        };
        mver_header.write(&mut cursor).unwrap();
        cursor.write_all(&17u32.to_le_bytes()).unwrap();

        // Write MOHD chunk
        let mohd_header = ChunkHeader {
            id: ChunkId::from_bytes([b'M', b'O', b'H', b'D']),
            size: 64,
        };
        mohd_header.write(&mut cursor).unwrap();
        cursor.write_all(&[0u8; 64]).unwrap();

        // Write MOPT chunk with portal data
        let mopt_header = ChunkHeader {
            id: ChunkId::from_bytes([b'M', b'O', b'P', b'T']),
            size: 20, // 1 portal * 20 bytes
        };
        mopt_header.write(&mut cursor).unwrap();
        // Portal: start_vertex=0, n_vertices=4, normal=(0,1,0), distance=10.0
        cursor.write_all(&0u16.to_le_bytes()).unwrap(); // start_vertex
        cursor.write_all(&4u16.to_le_bytes()).unwrap(); // n_vertices
        cursor.write_all(&0.0f32.to_le_bytes()).unwrap(); // normal.x
        cursor.write_all(&1.0f32.to_le_bytes()).unwrap(); // normal.y
        cursor.write_all(&0.0f32.to_le_bytes()).unwrap(); // normal.z
        cursor.write_all(&10.0f32.to_le_bytes()).unwrap(); // distance

        // Parse the data - cursor goes out of scope
        // Create new cursor to read the data
        let mut read_cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut read_cursor).unwrap();
        read_cursor.seek(std::io::SeekFrom::Start(0)).unwrap();
        let root = parse_root_file(&mut read_cursor, discovery).unwrap();

        // Verify portal was parsed correctly
        assert_eq!(root.portals.len(), 1);
        assert_eq!(root.portals[0].start_vertex, 0);
        assert_eq!(root.portals[0].n_vertices, 4);
        assert_eq!(root.portals[0].normal.y, 1.0);
        assert_eq!(root.portals[0].distance, 10.0);
    }

    #[test]
    fn test_mopr_chunk_parsing() {
        // Create MOPR chunk with portal references
        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);

        // Write MVER chunk
        let mver_header = ChunkHeader {
            id: ChunkId::from_bytes([b'M', b'V', b'E', b'R']),
            size: 4,
        };
        mver_header.write(&mut cursor).unwrap();
        cursor.write_all(&17u32.to_le_bytes()).unwrap();

        // Write MOHD chunk
        let mohd_header = ChunkHeader {
            id: ChunkId::from_bytes([b'M', b'O', b'H', b'D']),
            size: 64,
        };
        mohd_header.write(&mut cursor).unwrap();
        cursor.write_all(&[0u8; 64]).unwrap();

        // Write MOPR chunk with references
        let mopr_header = ChunkHeader {
            id: ChunkId::from_bytes([b'M', b'O', b'P', b'R']),
            size: 16, // 2 references * 8 bytes each
        };
        mopr_header.write(&mut cursor).unwrap();
        // Reference 1: portal_index=0, group_index=1, side=1
        cursor.write_all(&0u16.to_le_bytes()).unwrap(); // portal_index
        cursor.write_all(&1u16.to_le_bytes()).unwrap(); // group_index
        cursor.write_all(&1i16.to_le_bytes()).unwrap(); // side
        cursor.write_all(&0u16.to_le_bytes()).unwrap(); // padding
        // Reference 2: portal_index=1, group_index=2, side=-1
        cursor.write_all(&1u16.to_le_bytes()).unwrap(); // portal_index
        cursor.write_all(&2u16.to_le_bytes()).unwrap(); // group_index
        cursor.write_all(&(-1i16).to_le_bytes()).unwrap(); // side
        cursor.write_all(&0u16.to_le_bytes()).unwrap(); // padding

        // Parse the data - cursor goes out of scope
        // Create new cursor to read the data
        let mut read_cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut read_cursor).unwrap();
        read_cursor.seek(std::io::SeekFrom::Start(0)).unwrap();
        let root = parse_root_file(&mut read_cursor, discovery).unwrap();

        // Verify portal references were parsed correctly
        assert_eq!(root.portal_refs.len(), 2);
        assert_eq!(root.portal_refs[0].portal_index, 0);
        assert_eq!(root.portal_refs[0].group_index, 1);
        assert_eq!(root.portal_refs[0].side, 1);
        assert_eq!(root.portal_refs[1].portal_index, 1);
        assert_eq!(root.portal_refs[1].group_index, 2);
        assert_eq!(root.portal_refs[1].side, -1);
    }

    #[test]
    fn test_modn_chunk_parsing() {
        // Create MODN chunk with doodad names
        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);

        // Write MVER chunk
        let mver_header = ChunkHeader {
            id: ChunkId::from_bytes([b'M', b'V', b'E', b'R']),
            size: 4,
        };
        mver_header.write(&mut cursor).unwrap();
        cursor.write_all(&17u32.to_le_bytes()).unwrap();

        // Write MOHD chunk
        let mohd_header = ChunkHeader {
            id: ChunkId::from_bytes([b'M', b'O', b'H', b'D']),
            size: 64,
        };
        mohd_header.write(&mut cursor).unwrap();
        cursor.write_all(&[0u8; 64]).unwrap();

        // Write MODN chunk with doodad names
        let names = b"doodad1.m2\0doodad2.m2\0";
        let modn_header = ChunkHeader {
            id: ChunkId::from_bytes([b'M', b'O', b'D', b'N']),
            size: names.len() as u32,
        };
        modn_header.write(&mut cursor).unwrap();
        cursor.write_all(names).unwrap();

        // Parse the data - cursor goes out of scope
        // Create new cursor to read the data
        let mut read_cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut read_cursor).unwrap();
        read_cursor.seek(std::io::SeekFrom::Start(0)).unwrap();
        let root = parse_root_file(&mut read_cursor, discovery).unwrap();

        // Verify doodad names were parsed correctly
        assert_eq!(root.doodad_names.len(), 2);
        assert_eq!(root.doodad_names[0], "doodad1.m2");
        assert_eq!(root.doodad_names[1], "doodad2.m2");
    }
}
