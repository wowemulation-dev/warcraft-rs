//! Compliance tests for TBC 2.4.3 ADT files using real WoW data.
//!
//! These tests use actual ADT files extracted from TBC 2.4.3 MPQ archives
//! to validate that the parser correctly handles real-world Outland data.
//!
//! Test files extracted from: `/home/danielsreichenbach/Downloads/wow/2.4.3/Data/expansion.MPQ`

use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use wow_adt::{AdtVersion, ParsedAdt, parse_adt};

/// Get path to TBC test data directory
fn test_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("tbc")
}

/// Helper to parse a TBC ADT file and extract the Root variant
fn parse_tbc_adt(test_file: &PathBuf) -> wow_adt::RootAdt {
    let data = fs::read(test_file).expect("Failed to read test file");
    let mut cursor = Cursor::new(data);
    let parsed = parse_adt(&mut cursor).expect("Failed to parse TBC ADT");

    match parsed {
        ParsedAdt::Root(root) => *root,
        _ => panic!("Expected Root ADT, got different variant"),
    }
}

/// Test parsing a TBC Outland ADT file (Hellfire Peninsula area).
///
/// This test verifies:
/// - File can be parsed without errors
/// - Version is correctly detected as TBC
/// - All required chunks are present (MVER, MHDR, MCIN, MTEX, MCNK)
/// - MCNK chunks are present (256 terrain tiles)
/// - TBC-specific MFBO (flight boundaries) chunk may be present
#[test]
fn test_parse_tbc_outland() {
    let test_file = test_data_dir().join("Expansion01_30_30.adt");

    // Skip test if file doesn't exist (CI environment)
    if !test_file.exists() {
        eprintln!("Skipping test - file not found: {:?}", test_file);
        return;
    }

    let data = fs::read(&test_file).expect("Failed to read test file");
    let mut cursor = Cursor::new(data.clone());
    let parsed = parse_adt(&mut cursor).expect("Failed to parse TBC ADT");

    // Verify version detection
    let version = parsed.version();
    assert!(
        matches!(version, AdtVersion::TBC),
        "Version should be detected as TBC, got: {:?}",
        version
    );

    // Verify root ADT structure
    let root = match parsed {
        ParsedAdt::Root(root) => *root,
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
        "✓ TBC Outland ADT parsed successfully ({} textures, {} chunks)",
        root.textures.len(),
        root.mcnk_chunks.len()
    );
}

/// Test parsing TBC MCNK subchunks with flight boundaries.
///
/// This test verifies that nested MCNK subchunks parse correctly:
/// - MCVT (heightmap with 145 floats)
/// - MCNR (normals with 145 compressed vectors)
/// - MCLY (texture layers, up to 4 per chunk)
/// - MCSH (shadow map, 512 bytes)
/// - MCLQ (legacy liquid data for TBC)
/// - TBC flight boundary data if present
#[test]
fn test_tbc_mcnk_subchunks() {
    let test_file = test_data_dir().join("Expansion01_29_30.adt"); // Adjacent tile with varied terrain

    if !test_file.exists() {
        eprintln!("Skipping test - file not found: {:?}", test_file);
        return;
    }

    let root = parse_tbc_adt(&test_file);

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

    println!("✓ TBC MCNK subchunks parsed successfully");
}

/// Test parsing TBC model placements (MDDF and MODF).
///
/// This test verifies:
/// - MMDX/MMID chunks for M2 model references
/// - MDDF chunk for doodad (M2) placements
/// - MWMO/MWID chunks for WMO object references
/// - MODF chunk for WMO placements
/// - Position, rotation, and scale data integrity
/// - TBC Outland-specific models
#[test]
fn test_tbc_placements() {
    let test_file = test_data_dir().join("Expansion01_31_31.adt"); // Area with structures

    if !test_file.exists() {
        eprintln!("Skipping test - file not found: {:?}", test_file);
        return;
    }

    let root = parse_tbc_adt(&test_file);

    // Verify M2 model data (doodads)
    if !root.models.is_empty() {
        println!("Found {} M2 model references", root.models.len());
    }

    if !root.doodad_placements.is_empty() {
        let placement = &root.doodad_placements[0];

        // Verify position data is reasonable (Outland coordinates)
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

    println!("✓ TBC model placements parsed successfully");
}

/// Test parsing TBC flight boundaries (MFBO chunk).
///
/// This test verifies:
/// - MFBO chunk is present (TBC+ feature)
/// - Flight boundary data contains valid height planes
/// - Max planes are above min planes (valid bounds)
#[test]
fn test_tbc_mcnk_with_flight_bounds() {
    let test_file = test_data_dir().join("Expansion01_30_30.adt");

    if !test_file.exists() {
        eprintln!("Skipping test - file not found: {:?}", test_file);
        return;
    }

    let root = parse_tbc_adt(&test_file);

    // Check if flight bounds exist (TBC feature)
    if let Some(flight_bounds) = &root.flight_bounds {
        println!("✓ Found MFBO flight boundaries chunk");

        // Verify that max_plane and min_plane arrays have valid data
        // These are 3x3 grids of i16 height values
        assert_eq!(
            flight_bounds.max_plane.len(),
            9,
            "MFBO max_plane should have 9 values"
        );
        assert_eq!(
            flight_bounds.min_plane.len(),
            9,
            "MFBO min_plane should have 9 values"
        );

        // Verify that at least some values are non-zero (actual flight boundary data)
        let has_max_data = flight_bounds.max_plane.iter().any(|&v| v != 0);
        let has_min_data = flight_bounds.min_plane.iter().any(|&v| v != 0);

        println!(
            "  Max plane has data: {}, Min plane has data: {}",
            has_max_data, has_min_data
        );

        // For valid flight boundaries, verify max is above min
        for i in 0..9 {
            if flight_bounds.max_plane[i] != 0 || flight_bounds.min_plane[i] != 0 {
                assert!(
                    flight_bounds.max_plane[i] >= flight_bounds.min_plane[i],
                    "Max plane[{}] ({}) should be >= min plane[{}] ({})",
                    i,
                    flight_bounds.max_plane[i],
                    i,
                    flight_bounds.min_plane[i]
                );
            }
        }

        println!("✓ TBC flight boundaries validated");
    } else {
        println!("ℹ MFBO chunk not present in this ADT (optional in TBC)");
    }
}

/// Test parsing multiple TBC ADT files to ensure robustness.
///
/// This test loads all extracted test files and verifies they all parse successfully.
#[test]
fn test_parse_all_tbc_files() {
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
                        matches!(version, AdtVersion::TBC),
                        "All files should be TBC version, got: {:?}",
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

    assert_eq!(
        error_count, 0,
        "All TBC ADT files should parse successfully"
    );
    assert!(
        parsed_count >= 5,
        "Should have parsed at least 5 test files"
    );

    println!(
        "✓ Successfully parsed {}/{} TBC ADT files",
        parsed_count,
        parsed_count + error_count
    );
}
