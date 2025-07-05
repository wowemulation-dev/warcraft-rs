//! Test that verifies Warcraft 3 MPQ file handling
//!
//! This test addresses GitHub issue #4 about file count discrepancies

use wow_mpq::Archive;

#[test]
#[ignore] // Requires Warcraft 3 MPQ files to be present
fn test_war3_file_count() {
    let war3_path = "/home/danielsreichenbach/Downloads/war3.mpq";

    // Skip test if file doesn't exist
    if !std::path::Path::new(war3_path).exists() {
        eprintln!("Skipping test: war3.mpq not found at {war3_path}");
        return;
    }

    let mut archive = Archive::open(war3_path).expect("Failed to open war3.mpq");

    // Get file count from info (total files in archive)
    let total_files = if let Some(bet) = archive.bet_table() {
        bet.header.file_count as usize
    } else if let Some(block_table) = archive.block_table() {
        // Count valid blocks
        block_table
            .entries()
            .iter()
            .filter(|entry| entry.exists())
            .count()
    } else {
        0
    };

    println!("Total files in archive: {total_files}");

    // Get files from listfile
    let listed_files = archive.list().expect("Failed to list files");
    println!("Files from (listfile): {}", listed_files.len());

    // Get all files including anonymous
    let all_files = archive.list_all().expect("Failed to list all files");
    println!("All files (including anonymous): {}", all_files.len());

    // Expected values based on StormLib analysis:
    // - Total files: 10681
    // - Listfile entries that exist: ~10644
    // - Some files exist but aren't in listfile

    assert!(
        total_files >= 10680,
        "Expected at least 10680 total files, got {total_files}"
    );
    assert!(
        listed_files.len() >= 10620,
        "Expected at least 10620 listed files, got {}",
        listed_files.len()
    );
    assert_eq!(
        all_files.len(),
        total_files,
        "list_all should return exactly the total number of files"
    );

    // Test that we can extract a specific file that was mentioned in the issue
    let test_file = "(10)DustwallowKeys.w3m";
    if let Ok(data) = archive.read_file(test_file) {
        println!("Successfully read {}: {} bytes", test_file, data.len());
        assert_eq!(data.len(), 298790, "File size mismatch");
    } else {
        // File might be in archive but not in listfile
        println!("Note: {test_file} not accessible via listfile");
    }
}

#[test]
#[ignore] // Requires Warcraft 3 expansion MPQ to be present
fn test_war3x_file_count() {
    let war3x_path = "/home/danielsreichenbach/Downloads/War3x.mpq";

    // Skip test if file doesn't exist
    if !std::path::Path::new(war3x_path).exists() {
        eprintln!("Skipping test: War3x.mpq not found at {war3x_path}");
        return;
    }

    let mut archive = Archive::open(war3x_path).expect("Failed to open War3x.mpq");

    let listed_files = archive.list().expect("Failed to list files");
    println!("War3x.mpq - Files from (listfile): {}", listed_files.len());

    // Warcraft 3 expansion should have several thousand files
    assert!(
        listed_files.len() >= 1000,
        "Expected at least 1000 files in expansion, got {}",
        listed_files.len()
    );
}
