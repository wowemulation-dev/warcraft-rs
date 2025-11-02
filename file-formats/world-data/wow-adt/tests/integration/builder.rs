//! Integration tests for ADT builder and serialization.
//!
//! Tests the complete build → write → parse round-trip to ensure
//! serialization produces valid ADT files that can be parsed back.

use std::io::Cursor;

use wow_adt::api::{ParsedAdt, RootAdt};
use wow_adt::builder::AdtBuilder;
use wow_adt::{AdtVersion, DoodadPlacement, WmoPlacement, parse_adt};

/// Helper to extract RootAdt from ParsedAdt enum
fn extract_root(parsed: ParsedAdt) -> RootAdt {
    match parsed {
        ParsedAdt::Root(r) => *r,
        _ => panic!("Expected Root ADT"),
    }
}

#[test]
fn test_minimal_adt_round_trip() {
    // Build minimal ADT
    let built_adt = AdtBuilder::new()
        .with_version(AdtVersion::VanillaEarly)
        .add_texture("terrain/grass.blp")
        .build()
        .expect("Failed to build ADT");

    // Serialize to bytes
    let bytes = built_adt.to_bytes().expect("Failed to serialize ADT");

    // Verify bytes are non-empty
    assert!(!bytes.is_empty(), "Serialized ADT should have content");

    // Parse back
    let mut cursor = Cursor::new(bytes);
    let root = extract_root(parse_adt(&mut cursor).expect("Failed to parse serialized ADT"));

    // Verify version
    assert_eq!(root.version, AdtVersion::VanillaEarly);

    // Verify textures
    assert_eq!(root.textures.len(), 1);
    assert_eq!(root.textures[0], "terrain/grass.blp");

    // Verify empty collections
    assert_eq!(root.models.len(), 0);
    assert_eq!(root.wmos.len(), 0);
    assert_eq!(root.doodad_placements.len(), 0);
    assert_eq!(root.wmo_placements.len(), 0);
}

#[test]
fn test_multiple_textures_round_trip() {
    // Build ADT with multiple textures
    let built_adt = AdtBuilder::new()
        .with_version(AdtVersion::WotLK)
        .add_texture("terrain/grass_01.blp")
        .add_texture("terrain/dirt_01.blp")
        .add_texture("terrain/rock_01.blp")
        .build()
        .expect("Failed to build ADT");

    // Serialize and parse
    let bytes = built_adt.to_bytes().expect("Failed to serialize ADT");
    let mut cursor = Cursor::new(bytes);
    let root = extract_root(parse_adt(&mut cursor).expect("Failed to parse serialized ADT"));

    // Verify textures preserved
    assert_eq!(root.textures.len(), 3);
    assert_eq!(root.textures[0], "terrain/grass_01.blp");
    assert_eq!(root.textures[1], "terrain/dirt_01.blp");
    assert_eq!(root.textures[2], "terrain/rock_01.blp");
}

#[test]
fn test_models_and_wmos_round_trip() {
    // Build ADT with models and WMOs
    let built_adt = AdtBuilder::new()
        .with_version(AdtVersion::TBC)
        .add_texture("terrain/base.blp")
        .add_model("world/doodads/tree_01.m2")
        .add_model("world/doodads/rock_01.m2")
        .add_wmo("world/wmo/building_01.wmo")
        .build()
        .expect("Failed to build ADT");

    // Serialize and parse
    let bytes = built_adt.to_bytes().expect("Failed to serialize ADT");
    let mut cursor = Cursor::new(bytes);
    let root = extract_root(parse_adt(&mut cursor).expect("Failed to parse serialized ADT"));

    // Verify models
    assert_eq!(root.models.len(), 2);
    assert_eq!(root.models[0], "world/doodads/tree_01.m2");
    assert_eq!(root.models[1], "world/doodads/rock_01.m2");

    // Verify WMOs
    assert_eq!(root.wmos.len(), 1);
    assert_eq!(root.wmos[0], "world/wmo/building_01.wmo");
}

#[test]
fn test_doodad_placement_round_trip() {
    // Build ADT with doodad placement
    let placement = DoodadPlacement {
        name_id: 0,
        unique_id: 1,
        position: [1000.0, 1000.0, 100.0],
        rotation: [0.0, 45.0, 0.0],
        scale: 1024, // 1.0x scale
        flags: 0,
    };

    let built_adt = AdtBuilder::new()
        .with_version(AdtVersion::WotLK)
        .add_texture("terrain/grass.blp")
        .add_model("world/doodads/tree_01.m2")
        .add_doodad_placement(placement)
        .build()
        .expect("Failed to build ADT");

    // Serialize and parse
    let bytes = built_adt.to_bytes().expect("Failed to serialize ADT");
    let mut cursor = Cursor::new(bytes);
    let root = extract_root(parse_adt(&mut cursor).expect("Failed to parse serialized ADT"));

    // Verify placement preserved
    assert_eq!(root.doodad_placements.len(), 1);
    let parsed_placement = &root.doodad_placements[0];
    assert_eq!(parsed_placement.name_id, placement.name_id);
    assert_eq!(parsed_placement.unique_id, placement.unique_id);
    assert_eq!(parsed_placement.position, placement.position);
    assert_eq!(parsed_placement.rotation, placement.rotation);
    assert_eq!(parsed_placement.scale, placement.scale);
}

#[test]
fn test_wmo_placement_round_trip() {
    // Build ADT with WMO placement
    let placement = WmoPlacement {
        name_id: 0,
        unique_id: 100,
        position: [2000.0, 2000.0, 50.0],
        rotation: [0.0, 90.0, 0.0],
        extents_min: [-10.0, -10.0, -10.0],
        extents_max: [10.0, 10.0, 10.0],
        flags: 0,
        doodad_set: 0,
        name_set: 0,
        scale: 1024,
    };

    let built_adt = AdtBuilder::new()
        .with_version(AdtVersion::Cataclysm)
        .add_texture("terrain/grass.blp")
        .add_wmo("world/wmo/building_01.wmo")
        .add_wmo_placement(placement)
        .build()
        .expect("Failed to build ADT");

    // Serialize and parse
    let bytes = built_adt.to_bytes().expect("Failed to serialize ADT");
    let mut cursor = Cursor::new(bytes);
    let root = extract_root(parse_adt(&mut cursor).expect("Failed to parse serialized ADT"));

    // Verify placement preserved
    assert_eq!(root.wmo_placements.len(), 1);
    let parsed_placement = &root.wmo_placements[0];
    assert_eq!(parsed_placement.name_id, placement.name_id);
    assert_eq!(parsed_placement.unique_id, placement.unique_id);
    assert_eq!(parsed_placement.position, placement.position);
    assert_eq!(parsed_placement.rotation, placement.rotation);
    assert_eq!(parsed_placement.extents_min, placement.extents_min);
    assert_eq!(parsed_placement.extents_max, placement.extents_max);
    assert_eq!(parsed_placement.doodad_set, placement.doodad_set);
}

#[test]
fn test_chunk_ordering_after_serialization() {
    // Build ADT
    let built_adt = AdtBuilder::new()
        .add_texture("test.blp")
        .build()
        .expect("Failed to build ADT");

    // Serialize
    let bytes = built_adt.to_bytes().expect("Failed to serialize ADT");

    // Verify chunk order by examining magic bytes
    let mut offset = 0;

    // Helper to read chunk magic
    let read_magic = |offset: &mut usize| -> [u8; 4] {
        let magic = [
            bytes[*offset],
            bytes[*offset + 1],
            bytes[*offset + 2],
            bytes[*offset + 3],
        ];
        let size = u32::from_le_bytes([
            bytes[*offset + 4],
            bytes[*offset + 5],
            bytes[*offset + 6],
            bytes[*offset + 7],
        ]);
        *offset += 8 + size as usize;
        magic
    };

    // Verify chunk order
    assert_eq!(read_magic(&mut offset), [b'R', b'E', b'V', b'M']); // MVER
    assert_eq!(read_magic(&mut offset), [b'R', b'D', b'H', b'M']); // MHDR
    assert_eq!(read_magic(&mut offset), [b'N', b'I', b'C', b'M']); // MCIN
    assert_eq!(read_magic(&mut offset), [b'X', b'E', b'T', b'M']); // MTEX
    assert_eq!(read_magic(&mut offset), [b'X', b'D', b'M', b'M']); // MMDX
    assert_eq!(read_magic(&mut offset), [b'D', b'I', b'M', b'M']); // MMID
    assert_eq!(read_magic(&mut offset), [b'O', b'M', b'W', b'M']); // MWMO
    assert_eq!(read_magic(&mut offset), [b'D', b'I', b'W', b'M']); // MWID
    assert_eq!(read_magic(&mut offset), [b'F', b'D', b'D', b'M']); // MDDF
    assert_eq!(read_magic(&mut offset), [b'F', b'D', b'O', b'M']); // MODF
}

#[test]
fn test_mhdr_offsets_are_valid() {
    // Build ADT
    let built_adt = AdtBuilder::new()
        .add_texture("test.blp")
        .add_model("test.m2")
        .build()
        .expect("Failed to build ADT");

    // Serialize and parse
    let bytes = built_adt.to_bytes().expect("Failed to serialize ADT");
    let mut cursor = Cursor::new(bytes);
    let root = extract_root(parse_adt(&mut cursor).expect("Failed to parse serialized ADT"));

    // If parsing succeeded, MHDR offsets were valid
    // (parser would fail if offsets were incorrect)
    assert_eq!(root.textures.len(), 1);
    assert_eq!(root.models.len(), 1);
}

#[test]
fn test_write_to_file_and_read_back() {
    use std::fs::File;
    use std::io::Read;

    // Build ADT
    let built_adt = AdtBuilder::new()
        .with_version(AdtVersion::WotLK)
        .add_texture("terrain/grass.blp")
        .add_texture("terrain/dirt.blp")
        .build()
        .expect("Failed to build ADT");

    // Write to temporary file
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join("test_adt_round_trip.adt");

    built_adt
        .write_to_file(&temp_path)
        .expect("Failed to write ADT to file");

    // Read file back
    let mut file = File::open(&temp_path).expect("Failed to open written ADT file");
    let mut file_bytes = Vec::new();
    file.read_to_end(&mut file_bytes)
        .expect("Failed to read ADT file");

    // Parse from file
    let mut cursor = Cursor::new(file_bytes);
    let root = extract_root(parse_adt(&mut cursor).expect("Failed to parse ADT from file"));

    // Verify content
    assert_eq!(root.version, AdtVersion::WotLK);
    assert_eq!(root.textures.len(), 2);
    assert_eq!(root.textures[0], "terrain/grass.blp");
    assert_eq!(root.textures[1], "terrain/dirt.blp");

    // Cleanup
    let _ = std::fs::remove_file(&temp_path);
}

#[test]
fn test_complex_adt_round_trip() {
    // Build complex ADT with multiple features
    let doodad_placement = DoodadPlacement {
        name_id: 0,
        unique_id: 1,
        position: [1000.0, 1000.0, 100.0],
        rotation: [0.0, 45.0, 0.0],
        scale: 1024,
        flags: 0,
    };

    let wmo_placement = WmoPlacement {
        name_id: 0,
        unique_id: 100,
        position: [2000.0, 2000.0, 50.0],
        rotation: [0.0, 90.0, 0.0],
        extents_min: [-10.0, -10.0, -10.0],
        extents_max: [10.0, 10.0, 10.0],
        flags: 0,
        doodad_set: 0,
        name_set: 0,
        scale: 1024,
    };

    let built_adt = AdtBuilder::new()
        .with_version(AdtVersion::WotLK)
        .add_texture("terrain/grass_01.blp")
        .add_texture("terrain/dirt_01.blp")
        .add_texture("terrain/rock_01.blp")
        .add_model("world/doodads/tree_01.m2")
        .add_model("world/doodads/rock_01.m2")
        .add_wmo("world/wmo/building_01.wmo")
        .add_doodad_placement(doodad_placement)
        .add_wmo_placement(wmo_placement)
        .build()
        .expect("Failed to build complex ADT");

    // Serialize and parse
    let bytes = built_adt
        .to_bytes()
        .expect("Failed to serialize complex ADT");
    let mut cursor = Cursor::new(bytes);
    let root = extract_root(parse_adt(&mut cursor).expect("Failed to parse complex ADT"));

    // Verify all content preserved
    assert_eq!(root.version, AdtVersion::WotLK);
    assert_eq!(root.textures.len(), 3);
    assert_eq!(root.models.len(), 2);
    assert_eq!(root.wmos.len(), 1);
    assert_eq!(root.doodad_placements.len(), 1);
    assert_eq!(root.wmo_placements.len(), 1);
}

/// Test MoP blend mesh system builder and serialization (MBMH, MBBB, MBNV, MBMI).
///
/// Tests T192: Builder test for MoP blend mesh creation
///
/// Note: Full round-trip parsing requires MTXP chunk for version detection.
/// This test verifies builder accepts blend mesh chunks and serialization completes.
/// Parsing round-trip deferred until version detection includes MBMH as MoP indicator.
#[test]
fn test_mop_blend_mesh_builder() {
    use wow_adt::chunks::blend_mesh::{
        MbbbChunk, MbbbEntry, MbmhChunk, MbmhEntry, MbmiChunk, MbnvChunk, MbnvVertex,
    };

    // Create blend mesh header with one entry
    let mbmh = MbmhChunk {
        entries: vec![MbmhEntry {
            map_object_id: 1,
            texture_id: 42,
            unknown: 0,
            mbmi_count: 3, // One triangle (3 indices)
            mbnv_count: 3, // 3 vertices
            mbmi_start: 0,
            mbnv_start: 0,
        }],
    };

    // Create corresponding bounding box
    let mbbb = MbbbChunk {
        entries: vec![MbbbEntry {
            map_object_id: 1,
            min: [-10.0, -10.0, 0.0],
            max: [10.0, 10.0, 5.0],
        }],
    };

    // Create 3 vertices forming a triangle
    let mbnv = MbnvChunk {
        vertices: vec![
            MbnvVertex {
                position: [0.0, 0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [0.0, 0.0],
                color: [[255, 255, 255, 255]; 3],
            },
            MbnvVertex {
                position: [10.0, 0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [1.0, 0.0],
                color: [[255, 255, 255, 255]; 3],
            },
            MbnvVertex {
                position: [5.0, 10.0, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [0.5, 1.0],
                color: [[255, 255, 255, 255]; 3],
            },
        ],
    };

    // Create indices for the triangle
    let mbmi = MbmiChunk {
        indices: vec![0, 1, 2],
    };

    // Build MoP ADT with blend mesh chunks
    let built_adt = AdtBuilder::new()
        .with_version(AdtVersion::MoP)
        .add_texture("terrain/grass.blp")
        .add_blend_mesh_headers(mbmh.clone())
        .add_blend_mesh_bounds(mbbb.clone())
        .add_blend_mesh_vertices(mbnv.clone())
        .add_blend_mesh_indices(mbmi.clone())
        .build()
        .expect("Failed to build MoP ADT with blend mesh");

    // Verify builder stored blend mesh data correctly
    assert!(built_adt.blend_mesh_headers().is_some());
    assert!(built_adt.blend_mesh_bounds().is_some());
    assert!(built_adt.blend_mesh_vertices().is_some());
    assert!(built_adt.blend_mesh_indices().is_some());

    let stored_mbmh = built_adt.blend_mesh_headers().unwrap();
    assert_eq!(stored_mbmh.entries.len(), 1);
    assert_eq!(stored_mbmh.entries[0].map_object_id, 1);
    assert_eq!(stored_mbmh.entries[0].texture_id, 42);

    let stored_mbbb = built_adt.blend_mesh_bounds().unwrap();
    assert_eq!(stored_mbbb.entries.len(), 1);
    assert_eq!(stored_mbbb.entries[0].min, [-10.0, -10.0, 0.0]);

    let stored_mbnv = built_adt.blend_mesh_vertices().unwrap();
    assert_eq!(stored_mbnv.vertices.len(), 3);
    assert_eq!(stored_mbnv.vertices[0].position, [0.0, 0.0, 0.0]);

    let stored_mbmi = built_adt.blend_mesh_indices().unwrap();
    assert_eq!(stored_mbmi.indices.len(), 3);
    assert_eq!(stored_mbmi.indices, vec![0, 1, 2]);

    // Serialize to bytes - verifies serialization doesn't crash
    let bytes = built_adt
        .to_bytes()
        .expect("Failed to serialize MoP ADT with blend mesh");

    // Verify serialized data is non-empty and reasonable size
    // Minimal ADT with 256 MCNK chunks + blend mesh is ~200KB
    assert!(
        bytes.len() > 1000,
        "Serialized ADT should have substantial size"
    );
    assert!(
        bytes.len() < 1_000_000,
        "Serialized ADT should not be unreasonably large (got {} bytes)",
        bytes.len()
    );
}

/// Test MCNK serialization with subchunk offset verification.
///
/// Tests T094e: MCNK serialization integration test verifying subchunk offsets
///
/// This test verifies that when an ADT with MCNK chunks is serialized and parsed back,
/// all MCNK subchunks are correctly written and parsed, which proves that the offset
/// calculation in the MCNK header is working correctly.
#[test]
fn test_mcnk_subchunk_offset_verification() {
    // Build minimal ADT - serializer generates 256 minimal MCNK chunks automatically
    let built_adt = AdtBuilder::new()
        .with_version(AdtVersion::WotLK)
        .add_texture("terrain/grass.blp")
        .build()
        .expect("Failed to build ADT");

    // Serialize to bytes
    let bytes = built_adt.to_bytes().expect("Failed to serialize ADT");

    // Parse back
    let mut cursor = Cursor::new(bytes);
    let parsed = parse_adt(&mut cursor).expect("Failed to parse serialized ADT");
    let root = extract_root(parsed);

    // Verify MCNK chunks were written (256 chunks total)
    assert_eq!(
        root.mcnk_chunks.len(),
        256,
        "Should have 256 MCNK chunks (16x16 grid)"
    );

    // Verify all MCNK chunks have required subchunks
    // If parsing succeeded, offset calculation worked correctly
    for (idx, mcnk) in root.mcnk_chunks.iter().enumerate() {
        // All minimal MCNK chunks should have these subchunks
        assert!(
            mcnk.heights.is_some(),
            "MCNK chunk {} should have MCVT heights",
            idx
        );
        assert!(
            mcnk.normals.is_some(),
            "MCNK chunk {} should have MCNR normals",
            idx
        );
        assert!(
            mcnk.layers.is_some(),
            "MCNK chunk {} should have MCLY layers",
            idx
        );

        // Verify header offset fields are non-zero (proving they were calculated)
        assert!(
            mcnk.header.ofs_layer > 0,
            "MCNK chunk {} ofs_layer should be non-zero",
            idx
        );

        // Verify heights have correct count
        if let Some(heights) = &mcnk.heights {
            assert_eq!(
                heights.heights.len(),
                145,
                "MCNK chunk {} should have 145 heights",
                idx
            );
        }

        // Verify normals have correct count
        if let Some(normals) = &mcnk.normals {
            assert_eq!(
                normals.normals.len(),
                145,
                "MCNK chunk {} should have 145 normals",
                idx
            );
        }

        // Verify layers have at least one layer
        if let Some(layers) = &mcnk.layers {
            assert!(
                !layers.layers.is_empty(),
                "MCNK chunk {} should have at least one layer",
                idx
            );
        }
    }

    // Spot check first MCNK chunk for detailed verification
    let first_mcnk = &root.mcnk_chunks[0];

    // Verify index coordinates
    assert_eq!(first_mcnk.header.index_x, 0, "First MCNK should be at X=0");
    assert_eq!(first_mcnk.header.index_y, 0, "First MCNK should be at Y=0");

    // Verify last MCNK chunk
    let last_mcnk = &root.mcnk_chunks[255];
    assert_eq!(last_mcnk.header.index_x, 15, "Last MCNK should be at X=15");
    assert_eq!(last_mcnk.header.index_y, 15, "Last MCNK should be at Y=15");

    // If all of the above succeeded, MCNK serialization with correct offset calculation is working.
    // The parser successfully used the offset fields in the MCNK headers to locate and parse
    // all subchunks, proving that the serializer correctly calculated those offsets.
}

/// Test MH2O vertex data round-trip for all LVF formats.
///
/// Tests T098a-3: MH2O vertex data serialization and parsing
///
/// This test verifies that MH2O vertex data arrays and exists bitmaps are correctly
/// serialized and can be parsed back with data integrity preserved. It tests all four
/// Liquid Vertex Formats (LVF 0-3) to ensure complete format coverage.
#[test]
fn test_mh2o_vertex_data_round_trip() {
    use wow_adt::chunks::mh2o::{
        DepthOnlyVertex, HeightDepthVertex, HeightUvDepthVertex, HeightUvVertex, Mh2oChunk,
        Mh2oEntry, Mh2oHeader, Mh2oInstance, UvMapEntry, VertexDataArray,
    };

    // Create MH2O chunk with 4 entries, each using a different LVF format
    let mut entries = Vec::new();

    // Entry 0: LVF 0 (HeightDepth) - 3x3 liquid (16 vertices)
    {
        let header = Mh2oHeader {
            offset_instances: 0, // Will be calculated during serialization
            layer_count: 1,
            offset_attributes: 0,
        };

        let instance = Mh2oInstance {
            liquid_type: 0,
            liquid_object_or_lvf: 0, // LVF 0
            min_height_level: 100.0,
            max_height_level: 110.0,
            x_offset: 0,
            y_offset: 0,
            width: 3,
            height: 3,
            offset_exists_bitmap: 0,
            offset_vertex_data: 0,
        };

        // Create 4x4 = 16 vertices (width+1) * (height+1)
        let mut vertices = Vec::new();
        for i in 0..16 {
            vertices.push(HeightDepthVertex {
                height: 100.0 + i as f32,
                depth: i as u8,
            });
        }

        let vertex_data = vec![Some(VertexDataArray::HeightDepth(vertices))];
        let exists_bitmap: u64 = 0xFFFF; // All tiles exist
        let exists_bitmaps = vec![Some(exists_bitmap)];

        entries.push(Mh2oEntry {
            header,
            instances: vec![instance],
            vertex_data,
            exists_bitmaps,
            attributes: None,
        });
    }

    // Entry 1: LVF 1 (HeightUv) - 2x2 liquid (9 vertices)
    {
        let header = Mh2oHeader {
            offset_instances: 0,
            layer_count: 1,
            offset_attributes: 0,
        };

        let instance = Mh2oInstance {
            liquid_type: 1,
            liquid_object_or_lvf: 1, // LVF 1
            min_height_level: 50.0,
            max_height_level: 60.0,
            x_offset: 0,
            y_offset: 0,
            width: 2,
            height: 2,
            offset_exists_bitmap: 0,
            offset_vertex_data: 0,
        };

        // Create 3x3 = 9 vertices
        let mut vertices = Vec::new();
        for i in 0..9 {
            vertices.push(HeightUvVertex {
                height: 50.0 + i as f32,
                uv: UvMapEntry {
                    u: (i % 3) as u16 * 100,
                    v: (i / 3) as u16 * 100,
                },
            });
        }

        let vertex_data = vec![Some(VertexDataArray::HeightUv(vertices))];
        let exists_bitmap: u64 = 0xFF; // Partial tiles
        let exists_bitmaps = vec![Some(exists_bitmap)];

        entries.push(Mh2oEntry {
            header,
            instances: vec![instance],
            vertex_data,
            exists_bitmaps,
            attributes: None,
        });
    }

    // Entry 2: LVF 2 (DepthOnly) - 1x1 liquid (4 vertices)
    {
        let header = Mh2oHeader {
            offset_instances: 0,
            layer_count: 1,
            offset_attributes: 0,
        };

        let instance = Mh2oInstance {
            liquid_type: 2,
            liquid_object_or_lvf: 2, // LVF 2
            min_height_level: 200.0,
            max_height_level: 200.0, // Flat surface
            x_offset: 0,
            y_offset: 0,
            width: 1,
            height: 1,
            offset_exists_bitmap: 0,
            offset_vertex_data: 0,
        };

        // Create 2x2 = 4 vertices
        let vertices = vec![
            DepthOnlyVertex { depth: 10 },
            DepthOnlyVertex { depth: 20 },
            DepthOnlyVertex { depth: 30 },
            DepthOnlyVertex { depth: 40 },
        ];

        let vertex_data = vec![Some(VertexDataArray::DepthOnly(vertices))];
        let exists_bitmap: u64 = 0x1; // Single tile
        let exists_bitmaps = vec![Some(exists_bitmap)];

        entries.push(Mh2oEntry {
            header,
            instances: vec![instance],
            vertex_data,
            exists_bitmaps,
            attributes: None,
        });
    }

    // Entry 3: LVF 3 (HeightUvDepth) - 2x1 liquid (6 vertices)
    {
        let header = Mh2oHeader {
            offset_instances: 0,
            layer_count: 1,
            offset_attributes: 0,
        };

        let instance = Mh2oInstance {
            liquid_type: 3,
            liquid_object_or_lvf: 3, // LVF 3
            min_height_level: 150.0,
            max_height_level: 160.0,
            x_offset: 0,
            y_offset: 0,
            width: 2,
            height: 1,
            offset_exists_bitmap: 0,
            offset_vertex_data: 0,
        };

        // Create 3x2 = 6 vertices
        let mut vertices = Vec::new();
        for i in 0..6 {
            vertices.push(HeightUvDepthVertex {
                height: 150.0 + i as f32,
                uv: UvMapEntry {
                    u: i as u16 * 50,
                    v: i as u16 * 25,
                },
                depth: i as u8 * 10,
            });
        }

        let vertex_data = vec![Some(VertexDataArray::HeightUvDepth(vertices))];
        let exists_bitmap: u64 = 0x3; // Two tiles
        let exists_bitmaps = vec![Some(exists_bitmap)];

        entries.push(Mh2oEntry {
            header,
            instances: vec![instance],
            vertex_data,
            exists_bitmaps,
            attributes: None,
        });
    }

    // Fill remaining entries with empty data (no liquid)
    while entries.len() < 256 {
        entries.push(Mh2oEntry::default());
    }

    let mh2o = Mh2oChunk { entries };

    // Build WotLK ADT with MH2O water data
    let built_adt = AdtBuilder::new()
        .with_version(AdtVersion::WotLK)
        .add_texture("terrain/water.blp")
        .add_water_data(mh2o)
        .build()
        .expect("Failed to build ADT with MH2O data");

    // Serialize to bytes
    let bytes = built_adt
        .to_bytes()
        .expect("Failed to serialize ADT with MH2O vertex data");

    // Parse back
    let mut cursor = Cursor::new(bytes);
    let root = extract_root(parse_adt(&mut cursor).expect("Failed to parse ADT with MH2O data"));

    // Verify MH2O chunk was parsed
    assert!(
        root.water_data.is_some(),
        "Parsed ADT should have MH2O water data"
    );
    let parsed_mh2o = root.water_data.unwrap();

    // Verify entry count
    assert_eq!(
        parsed_mh2o.entries.len(),
        256,
        "MH2O should have 256 entries"
    );

    // Verify Entry 0 (LVF 0 - HeightDepth)
    {
        let entry = &parsed_mh2o.entries[0];
        assert_eq!(entry.instances.len(), 1, "Entry 0 should have 1 instance");

        let instance = &entry.instances[0];
        assert_eq!(instance.liquid_type, 0);
        assert_eq!(instance.liquid_object_or_lvf, 0); // LVF 0
        assert_eq!(instance.width, 3);
        assert_eq!(instance.height, 3);

        // Verify exists bitmap
        assert_eq!(entry.exists_bitmaps.len(), 1);
        assert_eq!(entry.exists_bitmaps[0], Some(0xFFFF));

        // Verify vertex data
        assert_eq!(entry.vertex_data.len(), 1);
        let vertex_data = entry.vertex_data[0]
            .as_ref()
            .expect("Entry 0 should have vertex data");

        match vertex_data {
            VertexDataArray::HeightDepth(vertices) => {
                assert_eq!(vertices.len(), 16, "Entry 0 should have 16 vertices");
                // Spot check first and last vertices
                assert_eq!(vertices[0].height, 100.0);
                assert_eq!(vertices[0].depth, 0);
                assert_eq!(vertices[15].height, 115.0);
                assert_eq!(vertices[15].depth, 15);
            }
            _ => panic!("Entry 0 should have HeightDepth vertex format"),
        }
    }

    // Verify Entry 1 (LVF 1 - HeightUv)
    {
        let entry = &parsed_mh2o.entries[1];
        assert_eq!(entry.instances.len(), 1, "Entry 1 should have 1 instance");

        let instance = &entry.instances[0];
        assert_eq!(instance.liquid_type, 1);
        assert_eq!(instance.liquid_object_or_lvf, 1); // LVF 1
        assert_eq!(instance.width, 2);
        assert_eq!(instance.height, 2);

        // Verify exists bitmap
        assert_eq!(entry.exists_bitmaps[0], Some(0xFF));

        // Verify vertex data
        let vertex_data = entry.vertex_data[0]
            .as_ref()
            .expect("Entry 1 should have vertex data");

        match vertex_data {
            VertexDataArray::HeightUv(vertices) => {
                assert_eq!(vertices.len(), 9, "Entry 1 should have 9 vertices");
                // Spot check vertices
                assert_eq!(vertices[0].height, 50.0);
                assert_eq!(vertices[0].uv.u, 0);
                assert_eq!(vertices[8].height, 58.0);
                assert_eq!(vertices[8].uv.u, 200);
            }
            _ => panic!("Entry 1 should have HeightUv vertex format"),
        }
    }

    // Verify Entry 2 (LVF 2 - DepthOnly)
    {
        let entry = &parsed_mh2o.entries[2];
        assert_eq!(entry.instances.len(), 1, "Entry 2 should have 1 instance");

        let instance = &entry.instances[0];
        assert_eq!(instance.liquid_type, 2);
        assert_eq!(instance.liquid_object_or_lvf, 2); // LVF 2
        assert_eq!(instance.width, 1);
        assert_eq!(instance.height, 1);

        // Verify exists bitmap
        assert_eq!(entry.exists_bitmaps[0], Some(0x1));

        // Verify vertex data
        let vertex_data = entry.vertex_data[0]
            .as_ref()
            .expect("Entry 2 should have vertex data");

        match vertex_data {
            VertexDataArray::DepthOnly(vertices) => {
                assert_eq!(vertices.len(), 4, "Entry 2 should have 4 vertices");
                assert_eq!(vertices[0].depth, 10);
                assert_eq!(vertices[1].depth, 20);
                assert_eq!(vertices[2].depth, 30);
                assert_eq!(vertices[3].depth, 40);
            }
            _ => panic!("Entry 2 should have DepthOnly vertex format"),
        }
    }

    // Verify Entry 3 (LVF 3 - HeightUvDepth)
    {
        let entry = &parsed_mh2o.entries[3];
        assert_eq!(entry.instances.len(), 1, "Entry 3 should have 1 instance");

        let instance = &entry.instances[0];
        assert_eq!(instance.liquid_type, 3);
        assert_eq!(instance.liquid_object_or_lvf, 3); // LVF 3
        assert_eq!(instance.width, 2);
        assert_eq!(instance.height, 1);

        // Verify exists bitmap
        assert_eq!(entry.exists_bitmaps[0], Some(0x3));

        // Verify vertex data
        let vertex_data = entry.vertex_data[0]
            .as_ref()
            .expect("Entry 3 should have vertex data");

        match vertex_data {
            VertexDataArray::HeightUvDepth(vertices) => {
                assert_eq!(vertices.len(), 6, "Entry 3 should have 6 vertices");
                // Spot check vertices
                assert_eq!(vertices[0].height, 150.0);
                assert_eq!(vertices[0].uv.u, 0);
                assert_eq!(vertices[0].depth, 0);
                assert_eq!(vertices[5].height, 155.0);
                assert_eq!(vertices[5].uv.u, 250);
                assert_eq!(vertices[5].depth, 50);
            }
            _ => panic!("Entry 3 should have HeightUvDepth vertex format"),
        }
    }

    // Verify remaining entries have no liquid data
    for i in 4..256 {
        let entry = &parsed_mh2o.entries[i];
        assert!(
            !entry.has_liquid(),
            "Entry {} should have no liquid data",
            i
        );
    }
}

/// Test MH2O entry without vertex data (uses min/max height).
///
/// Tests T098a-3: MH2O serialization without vertex data
///
/// This test verifies that MH2O instances without vertex data (which rely on
/// min_height_level and max_height_level for flat surfaces) serialize correctly.
#[test]
fn test_mh2o_without_vertex_data() {
    use wow_adt::chunks::mh2o::{Mh2oChunk, Mh2oEntry, Mh2oHeader, Mh2oInstance};

    let mut entries = Vec::new();

    // Create entry with instance but no vertex data
    let header = Mh2oHeader {
        offset_instances: 0,
        layer_count: 1,
        offset_attributes: 0,
    };

    let instance = Mh2oInstance {
        liquid_type: 0,
        liquid_object_or_lvf: 0,
        min_height_level: 100.0,
        max_height_level: 100.0, // Flat surface
        x_offset: 0,
        y_offset: 0,
        width: 8, // Full tile coverage
        height: 8,
        offset_exists_bitmap: 0, // No bitmap
        offset_vertex_data: 0,   // No vertex data
    };

    entries.push(Mh2oEntry {
        header,
        instances: vec![instance],
        vertex_data: vec![None],    // No vertex data
        exists_bitmaps: vec![None], // No exists bitmap
        attributes: None,
    });

    // Fill remaining entries
    while entries.len() < 256 {
        entries.push(Mh2oEntry::default());
    }

    let mh2o = Mh2oChunk { entries };

    // Build and serialize
    let built_adt = AdtBuilder::new()
        .with_version(AdtVersion::WotLK)
        .add_texture("terrain/water.blp")
        .add_water_data(mh2o)
        .build()
        .expect("Failed to build ADT");

    let bytes = built_adt.to_bytes().expect("Failed to serialize ADT");

    // Parse back
    let mut cursor = Cursor::new(bytes);
    let root = extract_root(parse_adt(&mut cursor).expect("Failed to parse ADT"));

    // Verify MH2O parsed correctly
    let parsed_mh2o = root.water_data.expect("Should have MH2O data");
    let entry = &parsed_mh2o.entries[0];

    assert_eq!(entry.instances.len(), 1);
    let instance = &entry.instances[0];
    assert_eq!(instance.min_height_level, 100.0);
    assert_eq!(instance.max_height_level, 100.0);
    assert_eq!(instance.width, 8);
    assert_eq!(instance.height, 8);

    // Verify no vertex data or exists bitmap
    assert_eq!(entry.vertex_data.len(), 1);
    assert!(entry.vertex_data[0].is_none(), "Should have no vertex data");
    assert_eq!(entry.exists_bitmaps.len(), 1);
    assert!(
        entry.exists_bitmaps[0].is_none(),
        "Should have no exists bitmap"
    );
}
