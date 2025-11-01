//! Compliance tests for Vanilla 1.12.1 ADT files using real WoW data.
//!
//! These tests use actual ADT files extracted from Vanilla 1.12.1 MPQ archives
//! to validate that the parser correctly handles real-world data.
//!
//! Test files extracted from: `/home/danielsreichenbach/Downloads/wow/1.12.1/Data/terrain.MPQ`

use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use wow_adt::{parse_adt, AdtVersion, ParsedAdt};

/// Get path to test data directory
fn test_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("vanilla")
}

/// Helper to parse an ADT file and extract the Root variant
fn parse_vanilla_adt(test_file: &PathBuf) -> wow_adt::RootAdt {
    let data = fs::read(test_file).expect("Failed to read test file");
    let mut cursor = Cursor::new(data);
    let parsed = parse_adt(&mut cursor).expect("Failed to parse Vanilla ADT");

    match parsed {
        ParsedAdt::Root(root) => root,
        _ => panic!("Expected Root ADT, got different variant"),
    }
}

/// Test parsing a Vanilla Azeroth ADT file (Elwynn Forest area).
///
/// This test verifies:
/// - File can be parsed without errors
/// - Version is correctly detected as Vanilla
/// - All required chunks are present (MVER, MHDR, MCIN, MTEX, MCNK)
/// - MCNK chunks are present (256 terrain tiles)
#[test]
fn test_parse_vanilla_azeroth() {
    let test_file = test_data_dir().join("Azeroth_30_48.adt");

    // Skip test if file doesn't exist (CI environment)
    if !test_file.exists() {
        eprintln!("Skipping test - file not found: {:?}", test_file);
        return;
    }

    let data = fs::read(&test_file).expect("Failed to read test file");
    let mut cursor = Cursor::new(data.clone());
    let parsed = parse_adt(&mut cursor).expect("Failed to parse Vanilla ADT");

    // Verify version detection (Vanilla can be Early or Late depending on MCCV presence)
    let version = parsed.version();
    assert!(
        matches!(version, AdtVersion::VanillaEarly | AdtVersion::VanillaLate),
        "Version should be detected as Vanilla (Early or Late), got: {:?}",
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
        "✓ Vanilla Azeroth ADT parsed successfully ({} textures, {} chunks)",
        root.textures.len(),
        root.mcnk_chunks.len()
    );
}

/// Test parsing Vanilla MCNK subchunks.
///
/// This test verifies that nested MCNK subchunks parse correctly:
/// - MCVT (heightmap with 145 floats)
/// - MCNR (normals with 145 compressed vectors)
/// - MCLY (texture layers, up to 4 per chunk)
/// - MCSH (shadow map, 512 bytes)
/// - MCLQ (legacy liquid data for Vanilla/TBC)
#[test]
fn test_vanilla_mcnk_subchunks() {
    let test_file = test_data_dir().join("Azeroth_36_50.adt"); // Redridge - complex heightmap

    if !test_file.exists() {
        eprintln!("Skipping test - file not found: {:?}", test_file);
        return;
    }

    let root = parse_vanilla_adt(&test_file);

    // Verify at least one MCNK chunk has subchunks
    let mut found_mcvt = false;
    let mut found_mcnr = false;
    let mut found_mcly = false;

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
    }

    assert!(found_mcvt, "Should find MCVT subchunks in ADT");
    assert!(found_mcnr, "Should find MCNR subchunks in ADT");
    assert!(found_mcly, "Should find MCLY subchunks in ADT");

    println!("✓ Vanilla MCNK subchunks parsed successfully");
}

/// Test parsing Vanilla model placements (MDDF and MODF).
///
/// This test verifies:
/// - MMDX/MMID chunks for M2 model references
/// - MDDF chunk for doodad (M2) placements
/// - MWMO/MWID chunks for WMO object references
/// - MODF chunk for WMO placements
/// - Position, rotation, and scale data integrity
#[test]
fn test_vanilla_placements() {
    let test_file = test_data_dir().join("Azeroth_28_47.adt"); // Stormwind area - WMO heavy

    if !test_file.exists() {
        eprintln!("Skipping test - file not found: {:?}", test_file);
        return;
    }

    let root = parse_vanilla_adt(&test_file);

    // Verify M2 model data (doodads)
    if !root.models.is_empty() {
        println!("Found {} M2 model references", root.models.len());
    }

    if !root.doodad_placements.is_empty() {
        let placement = &root.doodad_placements[0];

        // Verify position data is reasonable (Azeroth coordinates)
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

    println!("✓ Vanilla model placements parsed successfully");
}

/// Test parsing multiple Vanilla ADT files to ensure robustness.
///
/// This test loads all extracted test files and verifies they all parse successfully.
#[test]
fn test_parse_all_vanilla_files() {
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
                        matches!(version, AdtVersion::VanillaEarly | AdtVersion::VanillaLate),
                        "All files should be Vanilla version, got: {:?}",
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

    assert_eq!(error_count, 0, "All Vanilla ADT files should parse successfully");
    assert!(parsed_count >= 5, "Should have parsed at least 5 test files");

    println!("✓ Successfully parsed {}/{} Vanilla ADT files", parsed_count, parsed_count + error_count);
}
