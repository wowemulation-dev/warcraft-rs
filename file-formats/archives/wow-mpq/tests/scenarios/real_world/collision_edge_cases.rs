use tempfile::NamedTempFile;
use wow_mpq::{Archive, ArchiveBuilder, FormatVersion, ListfileOption};

#[test]
fn test_collision_edge_cases() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    // Create files with various naming patterns likely to cause collisions
    let mut builder = ArchiveBuilder::new()
        .version(FormatVersion::V4)
        .listfile_option(ListfileOption::None);

    let collision_files = [
        "file1.bin",
        "file2.bin",
        "file3.bin", // Sequential
        "a.txt",
        "b.txt",
        "c.txt", // Short names
        "test_1.dat",
        "test_2.dat",
        "test_3.dat", // Similar patterns
        "very_long_filename_that_might_cause_issues.data",
        "another_very_long_filename_variant.data", // Long names
    ];

    // Add files with unique content
    for (i, filename) in collision_files.iter().enumerate() {
        let data = vec![i as u8; 1024];
        builder = builder.add_file_data(data, filename);
    }

    builder.build(path).unwrap();

    // Verify all files maintain integrity
    let mut archive = Archive::open(path).unwrap();
    for (i, filename) in collision_files.iter().enumerate() {
        let data = archive.read_file(filename).unwrap();
        assert_eq!(data.len(), 1024, "File {} has wrong size", filename);
        assert!(
            data.iter().all(|&b| b == i as u8),
            "File {} has wrong content",
            filename
        );
    }
}
