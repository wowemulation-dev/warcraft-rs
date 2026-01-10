use std::fs::File;
use std::io::BufReader;
use wow_wmo::chunk_discovery::discover_chunks;
use wow_wmo::file_type::{WmoFileType, detect_file_type};

#[test]
#[ignore = "Requires WMO test files not available in CI"]
fn test_root_file_detection() {
    // Test with a root WMO file
    let test_file = "tests/data/vanilla/World/wmo/Azeroth/Buildings/Stormwind/Stormwind.wmo";
    let file =
        File::open(test_file).unwrap_or_else(|_| panic!("Test file not found: {}", test_file));
    let mut reader = BufReader::new(file);

    let discovery = discover_chunks(&mut reader).expect("Failed to discover chunks");
    let file_type = detect_file_type(&discovery);

    assert_eq!(file_type, WmoFileType::Root, "Expected root file type");
}

#[test]
#[ignore = "Requires WMO test files not available in CI"]
fn test_group_file_detection() {
    // Test with a group WMO file
    let test_file = "tests/data/vanilla/World/wmo/Azeroth/Buildings/Stormwind/Stormwind_000.wmo";
    let file =
        File::open(test_file).unwrap_or_else(|_| panic!("Test file not found: {}", test_file));
    let mut reader = BufReader::new(file);

    let discovery = discover_chunks(&mut reader).expect("Failed to discover chunks");
    let file_type = detect_file_type(&discovery);

    assert_eq!(file_type, WmoFileType::Group, "Expected group file type");
}

#[test]
#[ignore = "Requires WMO test files not available in CI"]
fn test_multiple_file_types() {
    // Test multiple files to ensure consistent detection
    let test_files = vec![
        (
            "tests/data/vanilla/World/wmo/Dungeon/KZ_Uldaman/KZ_Uldaman_A.wmo",
            WmoFileType::Root,
        ),
        (
            "tests/data/vanilla/World/wmo/Dungeon/KL_DireMaul/KL_Diremaul_Instance_030.wmo",
            WmoFileType::Group,
        ),
        (
            "tests/data/wotlk/World/wmo/Northrend/Dragonblight/NewHearthglen/NH_Cathedral.wmo",
            WmoFileType::Root,
        ),
        (
            "tests/data/wotlk/World/wmo/Dungeon/Valgarde/Valgarde_003.wmo",
            WmoFileType::Group,
        ),
    ];

    for (file_path, expected_type) in test_files {
        let file =
            File::open(file_path).unwrap_or_else(|_| panic!("Test file not found: {}", file_path));
        let mut reader = BufReader::new(file);

        let discovery = discover_chunks(&mut reader).expect("Failed to discover chunks");
        let file_type = detect_file_type(&discovery);

        assert_eq!(file_type, expected_type, "Incorrect type for {}", file_path);
    }
}
