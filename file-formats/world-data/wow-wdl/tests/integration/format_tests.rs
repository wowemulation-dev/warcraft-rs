//! Integration tests for WDL parser

use std::io::Cursor;

use wow_wdl::conversion::convert_wdl_file;
use wow_wdl::parser::WdlParser;
use wow_wdl::types::*;
use wow_wdl::validation::validate_wdl_file;
use wow_wdl::version::WdlVersion;

/// Creates a realistic WDL file for testing
fn create_test_wdl_file(version: WdlVersion) -> Vec<u8> {
    let mut file = WdlFile::with_version(version);

    // Add WMO filenames if the version supports it
    if version.has_wmo_chunks() {
        file.wmo_filenames
            .push("World/wmo/Azeroth/Buildings/Human_Farm/Farm.wmo".to_string());
        file.wmo_filenames
            .push("World/wmo/Azeroth/Buildings/Human_Tower/Human_Tower.wmo".to_string());
        file.wmo_indices.push(0);
        file.wmo_indices.push(1);

        // Add WMO placements
        for i in 0..2 {
            let placement = ModelPlacement {
                id: i + 1,
                wmo_id: i,
                position: Vec3d::new(100.0 * i as f32, 200.0, 50.0),
                rotation: Vec3d::new(0.0, 0.0, 0.0),
                bounds: BoundingBox {
                    min: Vec3d::new(-10.0, -10.0, -10.0),
                    max: Vec3d::new(10.0, 10.0, 10.0),
                },
                flags: 0,
                doodad_set: 0,
                name_set: 0,
                padding: 0,
            };
            file.wmo_placements.push(placement);
        }
    }

    // Add Legion model info if the version supports it
    if version.has_ml_chunks() {
        // Add M2 placements
        for i in 0..2 {
            let placement = M2Placement {
                id: i + 1,
                m2_id: 1000 + i,
                position: Vec3d::new(100.0 * i as f32, 200.0, 50.0),
                rotation: Vec3d::new(0.0, 0.0, 0.0),
                scale: 1.0,
                flags: 0,
            };
            file.m2_placements.push(placement);

            // Add visibility info
            let visibility = M2VisibilityInfo {
                bounds: BoundingBox {
                    min: Vec3d::new(-10.0, -10.0, -10.0),
                    max: Vec3d::new(10.0, 10.0, 10.0),
                },
                radius: 17.32,
            };
            file.m2_visibility.push(visibility);
        }

        // Add WMO Legion placements
        for i in 0..2 {
            let placement = M2Placement {
                id: i + 10,
                m2_id: 2000 + i,
                position: Vec3d::new(300.0 * i as f32, 400.0, 100.0),
                rotation: Vec3d::new(0.0, 0.0, 0.0),
                scale: 1.0,
                flags: 0,
            };
            file.wmo_legion_placements.push(placement);

            // Add visibility info
            let visibility = M2VisibilityInfo {
                bounds: BoundingBox {
                    min: Vec3d::new(-20.0, -20.0, -20.0),
                    max: Vec3d::new(20.0, 20.0, 20.0),
                },
                radius: 34.64,
            };
            file.wmo_legion_visibility.push(visibility);
        }
    }

    // Add heightmap tiles
    for y in 0..4 {
        for x in 0..4 {
            let mut heightmap = HeightMapTile::new();

            // Set some height values
            for i in 0..HeightMapTile::OUTER_COUNT {
                heightmap.outer_values[i] = ((x + y) * 10 + (i as u32 % 10)) as i16;
            }
            for i in 0..HeightMapTile::INNER_COUNT {
                heightmap.inner_values[i] = ((x + y) * 5 + (i as u32 % 10)) as i16;
            }

            file.heightmap_tiles.insert((x, y), heightmap);
            file.map_tile_offsets[(y * 64 + x) as usize] = 1; // Placeholder, will be calculated during write

            // Add holes if the version supports it
            if version.has_maho_chunk() {
                let mut holes = HolesData::new();

                // Create a checkerboard pattern of holes
                for hy in 0..16 {
                    for hx in 0..16 {
                        if (hx + hy) % 2 == 0 {
                            holes.set_hole(hx, hy, true);
                        }
                    }
                }

                file.holes_data.insert((x, y), holes);
            }
        }
    }

    // Write the file to a buffer
    let parser = WdlParser::with_version(version);
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    parser.write(&mut cursor, &file).unwrap();

    buffer
}

#[test]
fn test_wotlk_file_roundtrip() {
    // Create a WotLK test file
    let buffer = create_test_wdl_file(WdlVersion::Wotlk);

    // Parse the file
    let mut cursor = Cursor::new(&buffer);
    let parser = WdlParser::with_version(WdlVersion::Wotlk);
    let file = parser.parse(&mut cursor).unwrap();

    // Verify the parsed file
    assert_eq!(file.version, WdlVersion::Wotlk);
    assert_eq!(file.wmo_filenames.len(), 2);
    assert_eq!(file.wmo_placements.len(), 2);
    assert_eq!(file.heightmap_tiles.len(), 16); // 4x4 grid
    assert_eq!(file.holes_data.len(), 16); // 4x4 grid

    // Validate the file
    assert!(validate_wdl_file(&file).is_ok());

    // Write the file again
    let mut output = Vec::new();
    let mut cursor = Cursor::new(&mut output);
    parser.write(&mut cursor, &file).unwrap();

    // Sizes should be similar (may not be identical due to padding/alignment)
    let size_diff = (buffer.len() as i64 - output.len() as i64).abs();
    assert!(size_diff < 1024, "Size difference too large: {}", size_diff);
}

#[test]
fn test_legion_file_roundtrip() {
    // Create a Legion test file
    let buffer = create_test_wdl_file(WdlVersion::Legion);

    // Parse the file
    let mut cursor = Cursor::new(&buffer);
    let parser = WdlParser::with_version(WdlVersion::Legion);
    let file = parser.parse(&mut cursor).unwrap();

    // Verify the parsed file
    assert_eq!(file.version, WdlVersion::Legion);
    assert_eq!(file.wmo_filenames.len(), 0); // Legion doesn't use these
    assert_eq!(file.m2_placements.len(), 2);
    assert_eq!(file.wmo_legion_placements.len(), 2);
    assert_eq!(file.heightmap_tiles.len(), 16); // 4x4 grid
    assert_eq!(file.holes_data.len(), 16); // 4x4 grid

    // Validate the file
    assert!(validate_wdl_file(&file).is_ok());

    // Write the file again
    let mut output = Vec::new();
    let mut cursor = Cursor::new(&mut output);
    parser.write(&mut cursor, &file).unwrap();

    // Sizes should be similar
    let size_diff = (buffer.len() as i64 - output.len() as i64).abs();
    assert!(size_diff < 1024, "Size difference too large: {}", size_diff);
}

#[test]
fn test_version_conversion() {
    // Create a WotLK test file
    let buffer = create_test_wdl_file(WdlVersion::Wotlk);

    // Parse the file
    let mut cursor = Cursor::new(&buffer);
    let parser = WdlParser::with_version(WdlVersion::Wotlk);
    let wotlk_file = parser.parse(&mut cursor).unwrap();

    // Convert to Legion
    let legion_file = convert_wdl_file(&wotlk_file, WdlVersion::Legion).unwrap();

    // Verify the conversion
    assert_eq!(legion_file.version, WdlVersion::Legion);
    assert_eq!(legion_file.wmo_filenames.len(), 0); // Legion doesn't use these
    assert_eq!(legion_file.wmo_placements.len(), 0); // Legion doesn't use these

    // Should have converted WMO placements to Legion format
    assert_eq!(legion_file.wmo_legion_placements.len(), 2);
    assert_eq!(legion_file.wmo_legion_visibility.len(), 2);

    // Heightmap data should be preserved
    assert_eq!(legion_file.heightmap_tiles.len(), 16);
    assert_eq!(legion_file.holes_data.len(), 16);

    // Validate the converted file
    assert!(validate_wdl_file(&legion_file).is_ok());

    // Now let's go back from Legion to WotLK
    let back_to_wotlk = convert_wdl_file(&legion_file, WdlVersion::Wotlk).unwrap();

    // Verify the conversion
    assert_eq!(back_to_wotlk.version, WdlVersion::Wotlk);
    assert!(!back_to_wotlk.wmo_filenames.is_empty());
    assert_eq!(back_to_wotlk.wmo_placements.len(), 2);

    // Validate the converted file
    assert!(validate_wdl_file(&back_to_wotlk).is_ok());
}
