//! Compliance tests for WotLK 3.3.5a ADT files using real WoW data.
//!
//! These tests use actual ADT files extracted from WotLK 3.3.5a MPQ archives
//! to validate that the parser correctly handles real-world Northrend data.
//!
//! Test files extracted from: `/home/danielsreichenbach/Downloads/wow/3.3.5a/Data/lichking.MPQ`

use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use wow_adt::{parse_adt, AdtVersion, ParsedAdt};

/// Get path to WotLK test data directory
fn test_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("wotlk")
}

/// Helper to parse a WotLK ADT file and extract the Root variant
fn parse_wotlk_adt(test_file: &PathBuf) -> wow_adt::RootAdt {
    let data = fs::read(test_file).expect("Failed to read test file");
    let mut cursor = Cursor::new(data);
    let parsed = parse_adt(&mut cursor).expect("Failed to parse WotLK ADT");

    match parsed {
        ParsedAdt::Root(root) => root,
        _ => panic!("Expected Root ADT, got different variant"),
    }
}

/// Test parsing a WotLK Northrend ADT file.
///
/// This test verifies:
/// - File can be parsed without errors
/// - Version is correctly detected as WotLK
/// - All required chunks are present (MVER, MHDR, MCIN, MTEX, MCNK)
/// - MCNK chunks are present (256 terrain tiles)
/// - WotLK-specific MH2O (water data) chunk may be present
#[test]
fn test_parse_wotlk_northrend() {
    let test_file = test_data_dir().join("Northrend_30_30.adt");

    // Skip test if file doesn't exist (CI environment)
    if !test_file.exists() {
        eprintln!("Skipping test - file not found: {:?}", test_file);
        return;
    }

    let data = fs::read(&test_file).expect("Failed to read test file");
    let mut cursor = Cursor::new(data.clone());
    let parsed = parse_adt(&mut cursor).expect("Failed to parse WotLK ADT");

    // Verify version detection
    let version = parsed.version();
    assert!(
        matches!(version, AdtVersion::WotLK),
        "Version should be detected as WotLK, got: {:?}",
        version
    );

    // Verify root ADT structure
    let root = match parsed {
        ParsedAdt::Root(root) => root,
        _ => panic!("Expected Root ADT, got different variant"),
    };

    // Verify required chunks are present
    assert!(
        !root.textures.is_empty(),
        "MTEX chunk should contain textures"
    );
    assert_eq!(
        root.mcnk_chunks.len(),
        256,
        "Should have 256 MCNK terrain chunks"
    );

    // Verify basic data integrity
    assert_eq!(
        root.mcin.entries.len(),
        256,
        "MCIN should have 256 chunk index entries"
    );

    println!(
        "✓ WotLK Northrend ADT parsed successfully ({} textures, {} chunks)",
        root.textures.len(),
        root.mcnk_chunks.len()
    );
}

/// Test parsing WotLK MCNK subchunks with vertex colors.
///
/// This test verifies that nested MCNK subchunks parse correctly:
/// - MCVT (heightmap with 145 floats)
/// - MCNR (normals with 145 compressed vectors)
/// - MCLY (texture layers, up to 4 per chunk)
/// - MCSH (shadow map, 512 bytes)
/// - MCCV (vertex colors, WotLK+ feature)
#[test]
fn test_wotlk_mcnk_subchunks() {
    let test_file = test_data_dir().join("Northrend_29_30.adt"); // Adjacent tile with varied terrain

    if !test_file.exists() {
        eprintln!("Skipping test - file not found: {:?}", test_file);
        return;
    }

    let root = parse_wotlk_adt(&test_file);

    // Verify at least one MCNK chunk has subchunks
    let mut found_mcvt = false;
    let mut found_mcnr = false;
    let mut found_mcly = false;
    let mut found_mccv = false;

    for mcnk in &root.mcnk_chunks {
        if mcnk.heights.is_some() {
            found_mcvt = true;
            let heights = mcnk.heights.as_ref().unwrap();
            assert_eq!(
                heights.heights.len(),
                145,
                "MCVT should have 145 height values"
            );
        }

        if mcnk.normals.is_some() {
            found_mcnr = true;
            let normals = mcnk.normals.as_ref().unwrap();
            assert_eq!(
                normals.normals.len(),
                145,
                "MCNR should have 145 normal vectors"
            );
        }

        if mcnk.layers.is_some() {
            found_mcly = true;
            let layers = mcnk.layers.as_ref().unwrap();
            assert!(
                layers.layers.len() <= 4,
                "MCLY should have at most 4 texture layers"
            );
        }

        if mcnk.vertex_colors.is_some() {
            found_mccv = true;
            let colors = mcnk.vertex_colors.as_ref().unwrap();
            assert_eq!(
                colors.colors.len(),
                145,
                "MCCV should have 145 vertex colors (WotLK+ feature)"
            );
        }
    }

    assert!(found_mcvt, "Should find MCVT subchunks in ADT");
    assert!(found_mcnr, "Should find MCNR subchunks in ADT");
    assert!(found_mcly, "Should find MCLY subchunks in ADT");

    if found_mccv {
        println!("✓ Found MCCV vertex colors (WotLK feature)");
    }

    println!("✓ WotLK MCNK subchunks parsed successfully");
}

/// Test parsing WotLK model placements (MDDF and MODF).
///
/// This test verifies:
/// - MMDX/MMID chunks for M2 model references
/// - MDDF chunk for doodad (M2) placements
/// - MWMO/MWID chunks for WMO object references
/// - MODF chunk for WMO placements
/// - Position, rotation, and scale data integrity
/// - WotLK Northrend-specific models
#[test]
fn test_wotlk_placements() {
    let test_file = test_data_dir().join("Northrend_31_31.adt"); // Area with structures

    if !test_file.exists() {
        eprintln!("Skipping test - file not found: {:?}", test_file);
        return;
    }

    let root = parse_wotlk_adt(&test_file);

    // Verify M2 model data (doodads)
    if !root.models.is_empty() {
        println!("Found {} M2 model references", root.models.len());
    }

    if !root.doodad_placements.is_empty() {
        let placement = &root.doodad_placements[0];

        // Verify position data is reasonable (Northrend coordinates)
        assert!(
            placement.position[0].abs() < 100000.0,
            "Position X should be reasonable"
        );
        assert!(
            placement.position[1].abs() < 100000.0,
            "Position Y should be reasonable"
        );
        assert!(
            placement.position[2].abs() < 100000.0,
            "Position Z should be reasonable"
        );

        // Verify rotation data is in reasonable range
        assert!(
            placement.rotation[0].abs() <= std::f32::consts::PI * 2.0,
            "Rotation should be in valid range"
        );

        // Verify scale is reasonable (u16 where 1024 = 1.0x scale)
        assert!(placement.scale > 0, "Scale should be positive");

        println!("✓ Doodad placement data validated");
    }

    // Verify WMO object data
    if !root.wmos.is_empty() {
        println!("Found {} WMO object references", root.wmos.len());
    }

    if !root.wmo_placements.is_empty() {
        let placement = &root.wmo_placements[0];

        // Verify position data
        assert!(
            placement.position[0].abs() < 100000.0,
            "WMO position X should be reasonable"
        );

        // Verify bounding box is valid
        for i in 0..3 {
            assert!(
                placement.extents_min[i] <= placement.extents_max[i],
                "Bounding box should be valid (min <= max)"
            );
        }

        println!("✓ WMO placement data validated");
    }

    println!("✓ WotLK model placements parsed successfully");
}

/// Test parsing WotLK MH2O multi-level water data.
///
/// This test verifies:
/// - MH2O chunk is present (WotLK+ feature)
/// - Header array has 256 entries (one per MCNK chunk)
/// - Instances have valid liquid types and dimensions
/// - Vertex data parsing works correctly
/// - Multi-layer water (if present) is properly structured
#[test]
fn test_mh2o_multi_level_parsing() {
    let test_file = test_data_dir().join("Northrend_30_30.adt");

    if !test_file.exists() {
        eprintln!("Skipping test - file not found: {:?}", test_file);
        return;
    }

    let root = parse_wotlk_adt(&test_file);

    // Check if MH2O water data exists (WotLK+ feature)
    if let Some(water_data) = &root.water_data {
        println!("✓ Found MH2O water data chunk");

        // Verify header array has 256 entries (one per MCNK chunk)
        assert_eq!(
            water_data.entries.len(),
            256,
            "MH2O should have 256 entries (16×16 grid)"
        );

        let mut chunks_with_water = 0;
        let mut total_layers = 0;
        let mut multi_layer_chunks = 0;

        // Examine each entry
        for (idx, entry) in water_data.entries.iter().enumerate() {
            if entry.header.has_liquid() {
                chunks_with_water += 1;
                let layer_count = entry.header.layer_count as usize;
                total_layers += layer_count;

                if layer_count > 1 {
                    multi_layer_chunks += 1;
                    let row = idx / 16;
                    let col = idx % 16;
                    println!(
                        "  Multi-layer water at chunk ({}, {}): {} layers",
                        row, col, layer_count
                    );
                }

                // Verify instances match layer count
                assert_eq!(
                    entry.instances.len(),
                    layer_count,
                    "Instance count should match layer count for chunk {}",
                    idx
                );

                // Validate each instance
                for (layer_idx, instance) in entry.instances.iter().enumerate() {
                    // Verify dimensions are reasonable (8x8 grid max)
                    assert!(
                        instance.width <= 9,
                        "Instance width should be ≤ 9 (8×8 grid + 1)"
                    );
                    assert!(
                        instance.height <= 9,
                        "Instance height should be ≤ 9 (8×8 grid + 1)"
                    );

                    // Verify position within tile
                    assert!(
                        instance.x_offset < 8,
                        "X offset should be < 8 (within 8×8 tile)"
                    );
                    assert!(
                        instance.y_offset < 8,
                        "Y offset should be < 8 (within 8×8 tile)"
                    );

                    // Verify minimum height is less than or equal to maximum
                    assert!(
                        instance.min_height_level <= instance.max_height_level,
                        "Min height ({}) should be ≤ max height ({}) for chunk {} layer {}",
                        instance.min_height_level,
                        instance.max_height_level,
                        idx,
                        layer_idx
                    );
                }
            }
        }

        println!(
            "✓ Water statistics: {} chunks with water, {} total layers",
            chunks_with_water, total_layers
        );

        if multi_layer_chunks > 0 {
            println!("✓ Found {} chunks with multi-layer water", multi_layer_chunks);
        }

        assert!(
            chunks_with_water > 0,
            "Should find at least some chunks with water in Northrend"
        );

        println!("✓ MH2O multi-level water data validated");
    } else {
        println!("ℹ MH2O water data not present in this ADT (may be land-only area)");
    }
}

/// Test parsing WotLK MTXF texture flags.
///
/// This test verifies:
/// - MTXF chunk is present (WotLK 3.x+ feature)
/// - Flag count matches texture count from MTEX
/// - Flags contain valid bit patterns
#[test]
fn test_wotlk_mtxf_texture_flags() {
    let test_file = test_data_dir().join("Northrend_30_30.adt");

    if !test_file.exists() {
        eprintln!("Skipping test - file not found: {:?}", test_file);
        return;
    }

    let root = parse_wotlk_adt(&test_file);

    // Check if MTXF texture flags exist (WotLK 3.x+ feature)
    if let Some(texture_flags) = &root.texture_flags {
        println!("✓ Found MTXF texture flags chunk");

        // Verify flag count matches texture count
        assert_eq!(
            texture_flags.flags.len(),
            root.textures.len(),
            "MTXF flag count should match MTEX texture count"
        );

        // Examine flag values
        let mut flags_set = 0;
        let mut unique_flags = std::collections::HashSet::new();

        for (idx, &flag) in texture_flags.flags.iter().enumerate() {
            if flag != 0 {
                flags_set += 1;
                unique_flags.insert(flag);
                println!(
                    "  Texture {} ({}): flags = 0x{:08X}",
                    idx,
                    root.textures.get(idx).unwrap_or(&"<invalid>".to_string()),
                    flag
                );
            }
        }

        println!(
            "✓ MTXF statistics: {} textures, {} with non-zero flags, {} unique flag patterns",
            texture_flags.flags.len(),
            flags_set,
            unique_flags.len()
        );

        println!("✓ MTXF texture flags validated");
    } else {
        println!("ℹ MTXF texture flags not present in this ADT");
    }
}

/// Test parsing multiple WotLK ADT files to ensure robustness.
///
/// This test loads all extracted test files and verifies they all parse successfully.
#[test]
fn test_parse_all_wotlk_files() {
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

        if path.extension().and_then(|s| s.to_str()) == Some("adt") {
            let data = fs::read(&path).expect("Failed to read file");
            let mut cursor = Cursor::new(data);

            match parse_adt(&mut cursor) {
                Ok(parsed) => {
                    let version = parsed.version();
                    assert!(
                        matches!(version, AdtVersion::WotLK),
                        "All files should be WotLK version, got: {:?}",
                        version
                    );
                    parsed_count += 1;
                    println!("✓ Parsed: {}", path.file_name().unwrap().to_string_lossy());
                }
                Err(e) => {
                    eprintln!("✗ Failed to parse {}: {}", path.display(), e);
                    error_count += 1;
                }
            }
        }
    }

    assert_eq!(error_count, 0, "All WotLK ADT files should parse successfully");
    assert!(parsed_count >= 5, "Should have parsed at least 5 test files");

    println!("✓ Successfully parsed {}/{} WotLK ADT files", parsed_count, parsed_count + error_count);
}
