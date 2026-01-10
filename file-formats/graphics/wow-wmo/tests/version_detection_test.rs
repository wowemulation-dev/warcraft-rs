use std::fs::File;
use std::io::BufReader;
use wow_wmo::chunk_discovery::discover_chunks;
use wow_wmo::version_detection::{WmoVersion, detect_version};

#[test]
#[ignore = "Requires WMO test files not available in CI"]
fn test_classic_version_detection() {
    // Test with Classic WMO files
    let test_file = "tests/data/vanilla/World/wmo/Azeroth/Buildings/Stormwind/Stormwind.wmo";
    let file =
        File::open(test_file).unwrap_or_else(|_| panic!("Test file not found: {}", test_file));
    let mut reader = BufReader::new(file);

    let discovery = discover_chunks(&mut reader).expect("Failed to discover chunks");
    let version = detect_version(&discovery);

    // Classic should be detected based on absence of newer chunks
    assert_eq!(version, WmoVersion::Classic, "Expected Classic version");
}

#[test]
#[ignore = "Requires WMO test files not available in CI"]
fn test_wotlk_version_detection() {
    // Test with WotLK WMO files
    let test_file =
        "tests/data/wotlk/World/wmo/Northrend/Dragonblight/NewHearthglen/NH_Cathedral.wmo";
    let file =
        File::open(test_file).unwrap_or_else(|_| panic!("Test file not found: {}", test_file));
    let mut reader = BufReader::new(file);

    let discovery = discover_chunks(&mut reader).expect("Failed to discover chunks");
    let version = detect_version(&discovery);

    // WotLK should be detected (might be same as Classic/TBC for version 17)
    assert!(
        version == WmoVersion::Classic || version == WmoVersion::WotLK,
        "Expected Classic or WotLK version"
    );
}

#[test]
#[ignore = "Requires WMO test files not available in CI"]
fn test_cataclysm_version_detection() {
    // Test with Cataclysm transport WMO (should have MCVP chunk)
    let test_file = "tests/data/cataclysm/World/wmo/Dungeon/MD_Shipwreck/Transport_Shipwreck.wmo";
    let file =
        File::open(test_file).unwrap_or_else(|_| panic!("Test file not found: {}", test_file));
    let mut reader = BufReader::new(file);

    let discovery = discover_chunks(&mut reader).expect("Failed to discover chunks");
    let version = detect_version(&discovery);

    // Check if MCVP chunk is present
    let has_mcvp = discovery.chunks.iter().any(|c| c.id.as_str() == "MCVP");

    if has_mcvp {
        assert_eq!(
            version,
            WmoVersion::Cataclysm,
            "Expected Cataclysm version for MCVP chunk"
        );
    } else {
        // File might not have MCVP, could still be Cata without it
        assert!(
            version == WmoVersion::Classic || version == WmoVersion::Cataclysm,
            "Expected Classic or Cataclysm version"
        );
    }
}

#[test]
fn test_version_detection_by_chunk_patterns() {
    use binrw::io::Cursor;

    // Test data with MCVP chunk (Cataclysm+)
    let mut data = Vec::new();

    // MVER chunk
    data.extend_from_slice(b"REVM"); // MVER reversed
    data.extend_from_slice(&4u32.to_le_bytes()); // Size: 4
    data.extend_from_slice(&17u32.to_le_bytes()); // Version 17

    // MOHD chunk
    data.extend_from_slice(b"DHOM"); // MOHD reversed
    data.extend_from_slice(&64u32.to_le_bytes()); // Size: 64
    data.extend_from_slice(&[0u8; 64]); // Dummy data

    // MCVP chunk (Cataclysm+ indicator)
    data.extend_from_slice(b"PVCM"); // MCVP reversed
    data.extend_from_slice(&16u32.to_le_bytes()); // Size: 16
    data.extend_from_slice(&[0u8; 16]); // Dummy data

    let mut cursor = Cursor::new(&data);
    let discovery = discover_chunks(&mut cursor).expect("Failed to discover chunks");
    let version = detect_version(&discovery);

    assert_eq!(
        version,
        WmoVersion::Cataclysm,
        "MCVP chunk should indicate Cataclysm"
    );
}

#[test]
fn test_version_detection_with_gfid() {
    use binrw::io::Cursor;

    // Test data with GFID chunk (Warlords+)
    let mut data = Vec::new();

    // MVER chunk
    data.extend_from_slice(b"REVM"); // MVER reversed
    data.extend_from_slice(&4u32.to_le_bytes()); // Size: 4
    data.extend_from_slice(&17u32.to_le_bytes()); // Version 17

    // GFID chunk (Warlords+ indicator)
    data.extend_from_slice(b"DIFG"); // GFID reversed
    data.extend_from_slice(&16u32.to_le_bytes()); // Size: 16
    data.extend_from_slice(&[0u8; 16]); // Dummy data

    let mut cursor = Cursor::new(&data);
    let discovery = discover_chunks(&mut cursor).expect("Failed to discover chunks");
    let version = detect_version(&discovery);

    assert_eq!(
        version,
        WmoVersion::Warlords,
        "GFID chunk should indicate Warlords or later"
    );
}
