//! Integration tests for the WDT library

use std::io::Cursor;
use wow_wdt::chunks::maid::MaidSection;
use wow_wdt::{
    WdtFile, WdtReader, WdtWriter,
    chunks::{MaidChunk, ModfEntry, MphdFlags, mphd::FileDataIds},
    conversion::{convert_wdt, get_conversion_summary},
    tile_to_world,
    version::WowVersion,
    world_to_tile,
};

/// Create a minimal valid WDT file in memory
fn create_minimal_wdt() -> Vec<u8> {
    let mut buffer = Vec::new();

    // MVER chunk
    buffer.extend(b"REVM"); // Magic
    buffer.extend(&4u32.to_le_bytes()); // Size
    buffer.extend(&18u32.to_le_bytes()); // Version

    // MPHD chunk
    buffer.extend(b"DHPM"); // Magic
    buffer.extend(&32u32.to_le_bytes()); // Size
    buffer.extend(&0u32.to_le_bytes()); // Flags
    buffer.extend(&[0u8; 28]); // Rest of MPHD

    // MAIN chunk
    buffer.extend(b"NIAM"); // Magic
    buffer.extend(&((64 * 64 * 8) as u32).to_le_bytes()); // Size
    buffer.extend(&vec![0u8; 64 * 64 * 8]); // Empty tile data

    buffer
}

#[test]
fn test_read_minimal_wdt() {
    let data = create_minimal_wdt();
    let mut reader = WdtReader::new(Cursor::new(data), WowVersion::Classic);
    let wdt = reader.read().unwrap();

    assert_eq!(wdt.mver.version, 18);
    assert_eq!(wdt.mphd.flags.bits(), 0);
    assert_eq!(wdt.count_existing_tiles(), 0);
    assert!(!wdt.is_wmo_only());
}

#[test]
fn test_write_and_read_terrain_map() {
    let mut wdt = WdtFile::new(WowVersion::WotLK);

    // Set some tiles as existing
    wdt.main.get_mut(10, 20).unwrap().set_has_adt(true);
    wdt.main.get_mut(10, 20).unwrap().area_id = 1519; // Stormwind
    wdt.main.get_mut(30, 40).unwrap().set_has_adt(true);
    wdt.main.get_mut(30, 40).unwrap().area_id = 1637; // Orgrimmar

    // Add empty MWMO (pre-Cataclysm terrain maps have this)
    wdt.mwmo = Some(wow_wdt::chunks::MwmoChunk::new());

    // Set some flags
    wdt.mphd.flags |= MphdFlags::ADT_HAS_MCCV | MphdFlags::ADT_HAS_BIG_ALPHA;

    // Write to buffer
    let mut buffer = Vec::new();
    let mut writer = WdtWriter::new(&mut buffer);
    writer.write(&wdt).unwrap();

    // Read back
    let mut reader = WdtReader::new(Cursor::new(buffer), WowVersion::WotLK);
    let read_wdt = reader.read().unwrap();

    // Verify
    assert_eq!(read_wdt.count_existing_tiles(), 2);
    assert_eq!(read_wdt.main.get(10, 20).unwrap().area_id, 1519);
    assert_eq!(read_wdt.main.get(30, 40).unwrap().area_id, 1637);
    assert!(read_wdt.mphd.flags.contains(MphdFlags::ADT_HAS_MCCV));
    assert!(read_wdt.mphd.flags.contains(MphdFlags::ADT_HAS_BIG_ALPHA));
    assert!(read_wdt.mwmo.is_some());
}

#[test]
fn test_write_and_read_wmo_only_map() {
    let mut wdt = WdtFile::new(WowVersion::Classic);

    // Set as WMO-only
    wdt.mphd.flags |= MphdFlags::WDT_USES_GLOBAL_MAP_OBJ;

    // Add WMO filename
    let mut mwmo = wow_wdt::chunks::MwmoChunk::new();
    mwmo.add_filename("World\\wmo\\Dungeon\\KL_Orgrimmar\\KL_Orgrimmar.wmo".to_string());
    wdt.mwmo = Some(mwmo);

    // Add MODF entry
    let mut modf = wow_wdt::chunks::ModfChunk::new();
    modf.add_entry(ModfEntry {
        id: 0,
        unique_id: 0xFFFFFFFF,
        position: [100.0, 200.0, 50.0],
        rotation: [0.0, 1.57, 0.0], // 90 degrees Y rotation
        lower_bounds: [-500.0, -500.0, -100.0],
        upper_bounds: [500.0, 500.0, 300.0],
        flags: 0,
        doodad_set: 0,
        name_set: 0,
        scale: 0, // Pre-Cataclysm scale
    });
    wdt.modf = Some(modf);

    // Write and read back
    let mut buffer = Vec::new();
    let mut writer = WdtWriter::new(&mut buffer);
    writer.write(&wdt).unwrap();

    let mut reader = WdtReader::new(Cursor::new(buffer), WowVersion::Classic);
    let read_wdt = reader.read().unwrap();

    // Verify
    assert!(read_wdt.is_wmo_only());
    assert!(read_wdt.mwmo.is_some());
    assert!(read_wdt.modf.is_some());

    let wmo_name = &read_wdt.mwmo.unwrap().filenames[0];
    assert!(wmo_name.contains("KL_Orgrimmar.wmo"));

    let modf_entry = &read_wdt.modf.unwrap().entries[0];
    assert_eq!(modf_entry.position[0], 100.0);
    assert_eq!(modf_entry.unique_id, 0xFFFFFFFF);
}

#[test]
fn test_bfa_format_with_maid() {
    let mut wdt = WdtFile::new(WowVersion::BfA);

    // Enable MAID
    wdt.mphd.flags |= MphdFlags::WDT_HAS_MAID;
    wdt.mphd.set_file_data_ids(FileDataIds {
        lgt: 1000,
        occ: 1001,
        fogs: 1002,
        mpv: 1003,
        tex: 1004,
        wdl: 1005,
        pd4: 1006,
    });

    // Create MAID chunk
    let mut maid = MaidChunk::new();
    maid.set(MaidSection::RootAdt, 10, 20, 777332).unwrap();
    maid.set(MaidSection::Obj0Adt, 10, 20, 777333).unwrap();
    maid.set(MaidSection::Obj1Adt, 10, 20, 777334).unwrap();
    maid.set(MaidSection::Tex0Adt, 10, 20, 777335).unwrap();
    wdt.maid = Some(maid);

    // Also set the tile in MAIN
    wdt.main.get_mut(10, 20).unwrap().set_has_adt(true);

    // Write and read back
    let mut buffer = Vec::new();
    let mut writer = WdtWriter::new(&mut buffer);
    writer.write(&wdt).unwrap();

    let mut reader = WdtReader::new(Cursor::new(buffer), WowVersion::BfA);
    let read_wdt = reader.read().unwrap();

    // Verify
    assert!(read_wdt.mphd.has_maid());
    assert_eq!(read_wdt.mphd.lgt_file_data_id, Some(1000));
    assert_eq!(read_wdt.mphd.pd4_file_data_id, Some(1006));

    let maid = read_wdt.maid.unwrap();
    assert_eq!(maid.get(MaidSection::RootAdt, 10, 20), Some(777332));
    assert_eq!(maid.get(MaidSection::Tex0Adt, 10, 20), Some(777335));
}

#[test]
fn test_version_conversion_wotlk_to_cata() {
    let mut wdt = WdtFile::new(WowVersion::WotLK);

    // Pre-Cata terrain map with empty MWMO
    wdt.mwmo = Some(wow_wdt::chunks::MwmoChunk::new());
    wdt.main.get_mut(5, 5).unwrap().set_has_adt(true);

    // Convert to Cataclysm
    convert_wdt(&mut wdt, WowVersion::WotLK, WowVersion::Cataclysm).unwrap();

    // Verify changes
    assert!(wdt.mwmo.is_none()); // MWMO removed for terrain maps
    assert!(wdt.mphd.flags.contains(MphdFlags::UNK_FIRELANDS)); // Universal flag added
}

#[test]
fn test_version_conversion_legion_to_bfa() {
    let mut wdt = WdtFile::new(WowVersion::Legion);

    // Pre-BfA format
    wdt.mphd.flags |= MphdFlags::ADT_HAS_LIGHTING_VERTICES;

    // Convert to BfA
    convert_wdt(&mut wdt, WowVersion::Legion, WowVersion::BfA).unwrap();

    // Verify changes
    assert!(wdt.maid.is_some()); // MAID added
    assert!(wdt.mphd.has_maid());
    assert!(
        !wdt.mphd
            .flags
            .contains(MphdFlags::ADT_HAS_LIGHTING_VERTICES)
    ); // Deprecated flag removed
}

#[test]
fn test_coordinate_conversions() {
    // Test multiple coordinate pairs
    let test_cases = vec![
        ((0, 0), (17066.66, 17066.66)),     // Top-left
        ((63, 63), (-16533.33, -16533.33)), // Bottom-right
        ((32, 32), (0.0, 0.0)),             // Center
        ((16, 48), (-8533.33, 8533.33)),    // Random positions
    ];

    for ((tile_x, tile_y), (expected_x, expected_y)) in test_cases {
        let (world_x, world_y) = tile_to_world(tile_x, tile_y);

        // Check conversion with some tolerance for float precision
        assert!(
            (world_x - expected_x).abs() < 0.1,
            "World X mismatch for tile ({}, {}): expected {}, got {}",
            tile_x,
            tile_y,
            expected_x,
            world_x
        );
        assert!(
            (world_y - expected_y).abs() < 0.1,
            "World Y mismatch for tile ({}, {}): expected {}, got {}",
            tile_x,
            tile_y,
            expected_y,
            world_y
        );

        // Test reverse conversion
        let (calc_x, calc_y) = world_to_tile(world_x, world_y);
        assert_eq!(calc_x, tile_x);
        assert_eq!(calc_y, tile_y);
    }
}

#[test]
fn test_wdt_validation() {
    // Test valid WDT - Pre-Cataclysm terrain maps should have MWMO
    let mut valid_wdt = WdtFile::new(WowVersion::Classic);
    valid_wdt.mwmo = Some(wow_wdt::chunks::MwmoChunk::new()); // Add empty MWMO for pre-Cata
    let warnings = valid_wdt.validate();
    assert!(warnings.is_empty());

    // Test WMO-only without MWMO
    let mut invalid_wdt = WdtFile::new(WowVersion::Classic);
    invalid_wdt.mphd.flags |= MphdFlags::WDT_USES_GLOBAL_MAP_OBJ;
    let warnings = invalid_wdt.validate();
    assert!(warnings.iter().any(|w| w.contains("missing MWMO")));

    // Test MAID flag without MAID chunk
    let mut invalid_wdt2 = WdtFile::new(WowVersion::BfA);
    invalid_wdt2.mphd.flags |= MphdFlags::WDT_HAS_MAID;
    let warnings = invalid_wdt2.validate();
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("MAID chunk should be present"))
    );
}

#[test]
fn test_conversion_summary() {
    // Test conversion summary generation
    let summary = get_conversion_summary(WowVersion::WotLK, WowVersion::Cataclysm, false);
    assert!(summary.iter().any(|s| s.contains("Remove empty MWMO")));
    assert!(summary.iter().any(|s| s.contains("Add universal flag")));

    let summary2 = get_conversion_summary(WowVersion::Legion, WowVersion::BfA, false);
    assert!(summary2.iter().any(|s| s.contains("Add MAID chunk")));

    let summary3 = get_conversion_summary(WowVersion::Classic, WowVersion::Classic, false);
    assert!(summary3.iter().any(|s| s.contains("No conversion needed")));
}
