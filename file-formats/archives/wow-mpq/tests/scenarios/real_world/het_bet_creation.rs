//! Test HET/BET table creation in ArchiveBuilder

use tempfile::tempdir;
use wow_mpq::{Archive, ArchiveBuilder, FormatVersion};

#[test]
fn test_het_bet_table_creation() {
    let _ = env_logger::builder().is_test(true).try_init();
    let dir = tempdir().unwrap();
    let archive_path = dir.path().join("test_v3.mpq");

    // Create test files
    let test_data1 = b"Hello from file 1";
    let test_data2 = b"Hello from file 2";
    let test_data3 = b"This is a test file with more content than the others";

    // Build archive with v3 format (which uses HET/BET tables)
    ArchiveBuilder::new()
        .version(FormatVersion::V3)
        .add_file_data(test_data1.to_vec(), "file1.txt")
        .add_file_data(test_data2.to_vec(), "file2.txt")
        .add_file_data(test_data3.to_vec(), "test/file3.dat")
        .build(&archive_path)
        .expect("Failed to create archive");

    // Open the archive and verify files can be read
    let mut archive = Archive::open(&archive_path).expect("Failed to open archive");

    // Verify header has HET/BET table positions
    assert_eq!(
        archive.header().format_version,
        FormatVersion::V3,
        "Should be v3 format"
    );

    // Test reading files
    let data = archive
        .read_file("file1.txt")
        .expect("Failed to read file1.txt");
    assert_eq!(data, test_data1);

    let data = archive
        .read_file("file2.txt")
        .expect("Failed to read file2.txt");
    assert_eq!(data, test_data2);

    let data = archive
        .read_file("test/file3.dat")
        .expect("Failed to read file3.dat");
    assert_eq!(data, test_data3);
}

#[test]
fn test_het_bet_with_compression() {
    let dir = tempdir().unwrap();
    let archive_path = dir.path().join("test_v3_compressed.mpq");

    // Create a larger test file that will benefit from compression
    let test_data = "This is a test string that will be repeated many times. ".repeat(100);

    // Build archive with v3 format and compression
    ArchiveBuilder::new()
        .version(FormatVersion::V3)
        .default_compression(wow_mpq::compression::flags::ZLIB)
        .add_file_data(test_data.as_bytes().to_vec(), "compressed_file.txt")
        .build(&archive_path)
        .expect("Failed to create archive");

    // Open the archive and verify file can be read
    let mut archive = Archive::open(&archive_path).expect("Failed to open archive");

    let data = archive
        .read_file("compressed_file.txt")
        .expect("Failed to read compressed file");
    assert_eq!(data, test_data.as_bytes());
}

#[test]
fn test_het_bet_with_many_files() {
    let dir = tempdir().unwrap();
    let archive_path = dir.path().join("test_v3_many_files.mpq");

    // Create many files to test hash collision handling
    let mut builder = ArchiveBuilder::new().version(FormatVersion::V3);

    for i in 0..50 {
        let filename = format!("file_{i:03}.txt");
        let content = format!("This is file number {i}");
        builder = builder.add_file_data(content.into_bytes(), &filename);
    }

    builder
        .build(&archive_path)
        .expect("Failed to create archive");

    // Open the archive and verify a few files
    let mut archive = Archive::open(&archive_path).expect("Failed to open archive");

    // Check first, middle, and last files
    for i in [0, 25, 49] {
        let filename = format!("file_{i:03}.txt");
        let expected_content = format!("This is file number {i}");

        let data = archive
            .read_file(&filename)
            .unwrap_or_else(|_| panic!("Failed to read {filename}"));
        assert_eq!(data, expected_content.as_bytes());
    }
}
