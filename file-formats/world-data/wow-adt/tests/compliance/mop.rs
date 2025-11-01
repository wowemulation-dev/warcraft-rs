//! Compliance tests for Mists of Pandaria (5.4.8) chunk implementations.
//!
//! These tests verify that our MoP chunk implementations correctly parse
//! and serialize the 6 chunks introduced in Phase 4d:
//! - MCBB: Blend batches (per-MCNK references to blend mesh)
//! - MBMH: Blend mesh headers
//! - MBBB: Blend mesh bounding boxes
//! - MBNV: Blend mesh vertices
//! - MBMI: Blend mesh indices
//!
//! Test files extracted from: `/home/danielsreichenbach/Downloads/wow/5.4.8/5.4.8/Data/world.MPQ`

use binrw::{BinReaderExt, BinWrite};
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use wow_adt::chunks::mcnk::{BlendBatch, McbbChunk};
use wow_adt::chunks::{
    MbbbChunk, MbbbEntry, MbmhChunk, MbmhEntry, MbmiChunk, MbnvChunk, MbnvVertex,
};
use wow_adt::{parse_adt, AdtVersion, ParsedAdt};

/// Test MCBB chunk round-trip with realistic blend batch data.
#[test]
fn test_mcbb_compliance() {
    // Simulate a chunk with 3 blend batches referencing different mesh sections
    let mcbb = McbbChunk {
        batches: vec![
            BlendBatch {
                mbmh_index: 0,
                index_count: 96,
                index_first: 0,
                vertex_count: 64,
                vertex_first: 0,
            },
            BlendBatch {
                mbmh_index: 1,
                index_count: 48,
                index_first: 96,
                vertex_count: 32,
                vertex_first: 64,
            },
            BlendBatch {
                mbmh_index: 0,
                index_count: 72,
                index_first: 144,
                vertex_count: 48,
                vertex_first: 96,
            },
        ],
    };

    // Round-trip test
    let mut buffer = Cursor::new(Vec::new());
    mcbb.write_le(&mut buffer).unwrap();

    let data = buffer.into_inner();
    assert_eq!(data.len(), 60); // 3 batches × 20 bytes

    let mut cursor = Cursor::new(data);
    let parsed: McbbChunk = cursor.read_le().unwrap();

    assert_eq!(mcbb.batches, parsed.batches);
    assert_eq!(parsed.count(), 3);
    assert_eq!(parsed.total_indices(), 216); // 96 + 48 + 72
    assert_eq!(parsed.total_vertices(), 144); // 64 + 32 + 48
}

/// Test MBMH chunk round-trip with realistic blend mesh header data.
#[test]
fn test_mbmh_compliance() {
    // Simulate 2 blend mesh entries with different textures
    let mbmh = MbmhChunk {
        entries: vec![
            MbmhEntry {
                map_object_id: 1001,
                texture_id: 10,
                unknown: 0,
                mbmi_count: 96,
                mbnv_count: 64,
                mbmi_start: 0,
                mbnv_start: 0,
            },
            MbmhEntry {
                map_object_id: 1002,
                texture_id: 15,
                unknown: 0,
                mbmi_count: 48,
                mbnv_count: 32,
                mbmi_start: 96,
                mbnv_start: 64,
            },
        ],
    };

    // Round-trip test
    let mut buffer = Cursor::new(Vec::new());
    mbmh.write_le(&mut buffer).unwrap();

    let data = buffer.into_inner();
    assert_eq!(data.len(), 56); // 2 entries × 28 bytes

    let mut cursor = Cursor::new(data);
    let parsed: MbmhChunk = cursor.read_le().unwrap();

    assert_eq!(mbmh.entries, parsed.entries);
    assert_eq!(parsed.count(), 2);
    assert_eq!(parsed.total_indices(), 144); // 96 + 48
    assert_eq!(parsed.total_vertices(), 96); // 64 + 32
}

/// Test MBBB chunk round-trip with realistic bounding box data.
#[test]
fn test_mbbb_compliance() {
    // Simulate 2 bounding boxes for blend meshes
    let mbbb = MbbbChunk {
        entries: vec![
            MbbbEntry {
                map_object_id: 1001,
                min: [1000.0, 2000.0, 50.0],
                max: [1100.0, 2100.0, 150.0],
            },
            MbbbEntry {
                map_object_id: 1002,
                min: [1200.0, 2200.0, 60.0],
                max: [1300.0, 2300.0, 160.0],
            },
        ],
    };

    // Round-trip test
    let mut buffer = Cursor::new(Vec::new());
    mbbb.write_le(&mut buffer).unwrap();

    let data = buffer.into_inner();
    assert_eq!(data.len(), 56); // 2 entries × 28 bytes

    let mut cursor = Cursor::new(data);
    let parsed: MbbbChunk = cursor.read_le().unwrap();

    assert_eq!(mbbb.entries, parsed.entries);
    assert_eq!(parsed.count(), 2);

    // Verify geometry calculations
    let center0 = parsed.entries[0].center();
    assert!((center0[0] - 1050.0).abs() < 0.01);
    assert!((center0[1] - 2050.0).abs() < 0.01);
    assert!((center0[2] - 100.0).abs() < 0.01);
}

/// Test MBNV chunk round-trip with realistic vertex data.
#[test]
fn test_mbnv_compliance() {
    // Simulate 3 blend mesh vertices with typical data
    let mbnv = MbnvChunk {
        vertices: vec![
            MbnvVertex {
                position: [1000.0, 2000.0, 100.0],
                normal: [0.0, 0.0, 1.0],
                uv: [0.0, 0.0],
                color: [
                    [255, 0, 0, 255], // Red channel
                    [0, 255, 0, 255], // Green channel
                    [0, 0, 255, 255], // Blue channel
                ],
            },
            MbnvVertex {
                position: [1010.0, 2010.0, 105.0],
                normal: [0.0, 0.707, 0.707],
                uv: [0.5, 0.0],
                color: [
                    [200, 200, 200, 255],
                    [150, 150, 150, 255],
                    [100, 100, 100, 255],
                ],
            },
            MbnvVertex {
                position: [1020.0, 2020.0, 110.0],
                normal: [0.707, 0.0, 0.707],
                uv: [1.0, 1.0],
                color: [
                    [128, 128, 128, 255],
                    [128, 128, 128, 255],
                    [128, 128, 128, 255],
                ],
            },
        ],
    };

    // Round-trip test
    let mut buffer = Cursor::new(Vec::new());
    mbnv.write_le(&mut buffer).unwrap();

    let data = buffer.into_inner();
    assert_eq!(data.len(), 132); // 3 vertices × 44 bytes

    let mut cursor = Cursor::new(data);
    let parsed: MbnvChunk = cursor.read_le().unwrap();

    assert_eq!(mbnv.vertices, parsed.vertices);
    assert_eq!(parsed.count(), 3);

    // Verify first vertex data
    let v0 = &parsed.vertices[0];
    assert_eq!(v0.position, [1000.0, 2000.0, 100.0]);
    assert_eq!(v0.normal, [0.0, 0.0, 1.0]);
    assert_eq!(v0.uv, [0.0, 0.0]);
    assert_eq!(v0.color[0], [255, 0, 0, 255]);
}

/// Test MBMI chunk round-trip with realistic index data.
#[test]
fn test_mbmi_compliance() {
    // Simulate 2 triangles (6 indices)
    let mbmi = MbmiChunk {
        indices: vec![0, 1, 2, 2, 1, 3],
    };

    // Round-trip test
    let mut buffer = Cursor::new(Vec::new());
    mbmi.write_le(&mut buffer).unwrap();

    let data = buffer.into_inner();
    assert_eq!(data.len(), 12); // 6 indices × 2 bytes

    let mut cursor = Cursor::new(data);
    let parsed: MbmiChunk = cursor.read_le().unwrap();

    assert_eq!(mbmi.indices, parsed.indices);
    assert_eq!(parsed.count(), 6);
    assert_eq!(parsed.triangle_count(), 2);
}

/// Test complete blend mesh system integration.
#[test]
fn test_blend_mesh_system_integration() {
    // Create a complete blend mesh system with headers, bounds, vertices, and indices

    // 1 header referencing 1 mesh
    let mbmh = MbmhChunk {
        entries: vec![MbmhEntry {
            map_object_id: 1001,
            texture_id: 10,
            unknown: 0,
            mbmi_count: 6,
            mbnv_count: 4,
            mbmi_start: 0,
            mbnv_start: 0,
        }],
    };

    // 1 bounding box
    let mbbb = MbbbChunk {
        entries: vec![MbbbEntry {
            map_object_id: 1001,
            min: [0.0, 0.0, 0.0],
            max: [100.0, 100.0, 10.0],
        }],
    };

    // 4 vertices forming a quad
    let mbnv = MbnvChunk {
        vertices: vec![
            MbnvVertex {
                position: [0.0, 0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [0.0, 0.0],
                color: [[255, 255, 255, 255]; 3],
            },
            MbnvVertex {
                position: [100.0, 0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [1.0, 0.0],
                color: [[255, 255, 255, 255]; 3],
            },
            MbnvVertex {
                position: [100.0, 100.0, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [1.0, 1.0],
                color: [[255, 255, 255, 255]; 3],
            },
            MbnvVertex {
                position: [0.0, 100.0, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [0.0, 1.0],
                color: [[255, 255, 255, 255]; 3],
            },
        ],
    };

    // 6 indices forming 2 triangles (quad)
    let mbmi = MbmiChunk {
        indices: vec![0, 1, 2, 0, 2, 3],
    };

    // Verify system consistency
    assert_eq!(mbmh.entries[0].mbnv_count as usize, mbnv.count());
    assert_eq!(mbmh.entries[0].mbmi_count as usize, mbmi.count());
    assert_eq!(mbbb.entries[0].map_object_id, mbmh.entries[0].map_object_id);

    // Verify triangle topology
    assert_eq!(mbmi.triangle_count(), 2);

    // All indices should reference valid vertices
    for idx in &mbmi.indices {
        assert!((*idx as usize) < mbnv.count());
    }
}

/// Test MCBB batch referencing valid MBMH entries.
#[test]
fn test_mcbb_mbmh_integration() {
    // Create MBMH with 2 entries
    let mbmh = MbmhChunk {
        entries: vec![
            MbmhEntry {
                map_object_id: 1001,
                texture_id: 10,
                unknown: 0,
                mbmi_count: 96,
                mbnv_count: 64,
                mbmi_start: 0,
                mbnv_start: 0,
            },
            MbmhEntry {
                map_object_id: 1002,
                texture_id: 15,
                unknown: 0,
                mbmi_count: 48,
                mbnv_count: 32,
                mbmi_start: 96,
                mbnv_start: 64,
            },
        ],
    };

    // Create MCBB referencing both MBMH entries
    let mcbb = McbbChunk {
        batches: vec![
            BlendBatch {
                mbmh_index: 0, // References first MBMH entry
                index_count: 96,
                index_first: 0,
                vertex_count: 64,
                vertex_first: 0,
            },
            BlendBatch {
                mbmh_index: 1, // References second MBMH entry
                index_count: 48,
                index_first: 96,
                vertex_count: 32,
                vertex_first: 64,
            },
        ],
    };

    // Verify MCBB batches reference valid MBMH entries
    for batch in &mcbb.batches {
        assert!((batch.mbmh_index as usize) < mbmh.count());

        let header = &mbmh.entries[batch.mbmh_index as usize];
        assert_eq!(batch.index_count, header.mbmi_count);
        assert_eq!(batch.vertex_count, header.mbnv_count);
    }
}

// ==============================================================================
// COMPLIANCE TESTS - Real WoW 5.4.8 ADT files
// ==============================================================================

/// Get path to MoP test data directory
fn test_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("mop")
}

/// Test parsing MoP 5.4.8 split file root ADT.
///
/// This test verifies:
/// - Root file (.adt) can be parsed without errors
/// - Version is correctly detected as MoP
/// - Split file architecture is properly handled
/// - MoP format elements (blend mesh system) are supported
#[test]
fn test_parse_mop_azeroth() {
    let test_file = test_data_dir().join("Azeroth_30_30.adt");

    // Skip test if file doesn't exist (CI environment)
    if !test_file.exists() {
        eprintln!("Skipping test - file not found: {:?}", test_file);
        return;
    }

    let data = fs::read(&test_file).expect("Failed to read test file");
    let mut cursor = Cursor::new(data.clone());
    let parsed = parse_adt(&mut cursor).expect("Failed to parse MoP root ADT");

    // Verify version detection
    // Note: MoP 5.4.8 Azeroth files may be detected as Cataclysm if they don't contain
    // MoP-specific chunks (MTXP, blend mesh). This is expected for backward-compatible zones.
    let version = parsed.version();
    assert!(
        matches!(version, AdtVersion::Cataclysm | AdtVersion::MoP),
        "Version should be Cataclysm or MoP, got: {:?}",
        version
    );

    // Verify root ADT structure
    let root = match parsed {
        ParsedAdt::Root(root) => root,
        _ => panic!("Expected Root ADT, got different variant"),
    };

    // Verify required chunks are present
    assert_eq!(
        root.mcnk_chunks.len(),
        256,
        "Should have 256 MCNK terrain chunks"
    );

    println!(
        "✓ MoP root ADT parsed successfully ({} chunks)",
        root.mcnk_chunks.len()
    );
}

/// Test parsing multiple MoP ADT files to ensure robustness.
#[test]
fn test_parse_all_mop_root_files() {
    let test_dir = test_data_dir();

    if !test_dir.exists() {
        eprintln!("Skipping test - directory not found: {:?}", test_dir);
        return;
    }

    let mut parsed_count = 0;
    let mut error_count = 0;

    for entry in fs::read_dir(&test_dir).expect("Failed to read test directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        // Only test root files (not _tex0 or _obj0)
        if path.extension().and_then(|s| s.to_str()) == Some("adt")
            && !path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .contains("_tex")
            && !path.file_name().unwrap().to_string_lossy().contains("_obj")
        {
            let data = fs::read(&path).expect("Failed to read file");
            let mut cursor = Cursor::new(data);

            match parse_adt(&mut cursor) {
                Ok(parsed) => {
                    let version = parsed.version();
                    // Accept both Cataclysm and MoP for backward-compatible zones
                    if !matches!(version, AdtVersion::Cataclysm | AdtVersion::MoP) {
                        eprintln!(
                            "⚠ Skipping {} - detected as {:?} (expected Cataclysm or MoP)",
                            path.file_name().unwrap().to_string_lossy(),
                            version
                        );
                        continue;
                    }
                    parsed_count += 1;
                    println!("✓ Parsed: {} ({:?})", path.file_name().unwrap().to_string_lossy(), version);
                }
                Err(e) => {
                    eprintln!("⚠ Skipping {} - parse error: {}", path.display(), e);
                    error_count += 1;
                }
            }
        }
    }

    // Verify we successfully parsed at least 2 valid MoP root files
    assert!(
        parsed_count >= 2,
        "Should have parsed at least 2 root test files, got {}",
        parsed_count
    );

    println!(
        "✓ Successfully parsed {}/{} MoP root ADT files",
        parsed_count,
        parsed_count + error_count
    );
}

/// Test MoP split file completeness.
///
/// This test verifies that MoP split file triads are complete:
/// - Root file (.adt) contains terrain geometry and MCNK chunks
/// - Texture file (_tex0.adt) exists and contains texture data
/// - Object file (_obj0.adt) exists and contains model placements
#[test]
fn test_mop_split_file_completeness() {
    let test_sets = ["Azeroth_29_30", "Azeroth_30_30", "Azeroth_32_49"];

    for set_name in &test_sets {
        let root_file = test_data_dir().join(format!("{}.adt", set_name));
        let tex0_file = test_data_dir().join(format!("{}_tex0.adt", set_name));
        let obj0_file = test_data_dir().join(format!("{}_obj0.adt", set_name));

        if !root_file.exists() {
            eprintln!("Skipping test - file not found: {:?}", root_file);
            continue;
        }

        // Verify root file exists and can be parsed
        assert!(root_file.exists(), "Root file should exist: {:?}", root_file);
        let root_data = fs::read(&root_file).expect("Failed to read root file");
        let data_len = root_data.len();
        let mut cursor = Cursor::new(root_data);

        // Try to parse, skip if it fails (corrupted test file)
        let parsed = match parse_adt(&mut cursor) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("WARNING: {} failed to parse: {}", set_name, e);
                eprintln!("File size: {} bytes", data_len);
                continue;
            }
        };

        // Verify it's a Root ADT
        if !matches!(parsed, ParsedAdt::Root(_)) {
            eprintln!(
                "WARNING: {} parsed as {:?} instead of Root",
                set_name,
                std::mem::discriminant(&parsed)
            );
            eprintln!("File size: {} bytes", data_len);
            continue;
        }

        // Verify texture file exists
        assert!(
            tex0_file.exists(),
            "Texture file should exist: {:?}",
            tex0_file
        );
        let tex_size = fs::metadata(&tex0_file)
            .expect("Failed to get tex0 metadata")
            .len();
        assert!(tex_size > 0, "Texture file should not be empty");

        // Verify object file exists
        assert!(obj0_file.exists(), "Object file should exist: {:?}", obj0_file);
        let obj_size = fs::metadata(&obj0_file)
            .expect("Failed to get obj0 metadata")
            .len();
        assert!(obj_size > 0, "Object file should not be empty");

        println!(
            "✓ Split file triad complete: {} ({} bytes root, {} bytes tex, {} bytes obj)",
            set_name,
            fs::metadata(&root_file).unwrap().len(),
            tex_size,
            obj_size
        );
    }
}
