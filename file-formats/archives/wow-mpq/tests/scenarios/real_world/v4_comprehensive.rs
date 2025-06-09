use std::fs;
use tempfile::{NamedTempFile, TempDir};
use wow_mpq::{Archive, ArchiveBuilder, FormatVersion, ListfileOption, compression};

#[test]
fn test_v4_comprehensive() {
    // Create temporary directory for test files
    let temp_dir = TempDir::new().unwrap();

    // Create test files with different content
    let file1_path = temp_dir.path().join("test1.txt");
    let file2_path = temp_dir.path().join("test2.bin");
    let file3_path = temp_dir.path().join("subdir/test3.dat");

    fs::write(&file1_path, b"Hello from test file 1!\n").unwrap();
    fs::write(&file2_path, vec![0xFF; 1024]).unwrap(); // Binary data
    fs::create_dir_all(file3_path.parent().unwrap()).unwrap();
    fs::write(&file3_path, b"Nested file content").unwrap();

    // Create v4 archive with various options
    let archive_file = NamedTempFile::new().unwrap();
    let archive_path = archive_file.path();

    ArchiveBuilder::new()
        .version(FormatVersion::V4)
        .block_size(4) // 8KB sectors
        .default_compression(compression::flags::ZLIB)
        .generate_crcs(true) // Enable CRC generation
        .compress_tables(true) // Enable table compression
        .add_file(&file1_path, "files/test1.txt")
        .add_file_with_options(
            &file2_path,
            "files/test2.bin",
            compression::flags::BZIP2,
            false,
            0,
        )
        .add_file_data(b"Direct data content".to_vec(), "direct.txt")
        .add_file_data_with_encryption(
            b"Encrypted content".to_vec(),
            "encrypted.txt",
            compression::flags::ZLIB,
            true, // use_fix_key
            0,
        )
        .build(archive_path)
        .unwrap();

    // Verify archive creation
    assert!(archive_path.exists());
    let file_size = fs::metadata(archive_path).unwrap().len();
    assert!(file_size > 0);

    // Open and verify the archive
    let mut archive = Archive::open(archive_path).unwrap();

    // Check format version
    assert_eq!(archive.header().format_version, FormatVersion::V4);

    // Get archive info
    let info = archive.get_info().unwrap();
    println!("Archive info: {:?}", info);

    // Verify MD5 checksums
    assert!(info.md5_status.is_some());
    let md5_status = info.md5_status.unwrap();
    assert!(md5_status.header_valid);
    assert!(md5_status.hash_table_valid);
    assert!(md5_status.block_table_valid);
    assert!(md5_status.hi_block_table_valid);
    assert!(md5_status.het_table_valid);
    assert!(md5_status.bet_table_valid);

    // List files
    let files = archive.list().unwrap();
    assert_eq!(files.len(), 5); // 4 files + listfile

    // Verify file contents
    let content1 = archive.read_file("files/test1.txt").unwrap();
    assert_eq!(content1, b"Hello from test file 1!\n");

    let content2 = archive.read_file("files/test2.bin").unwrap();
    assert_eq!(content2.len(), 1024);
    assert!(content2.iter().all(|&b| b == 0xFF));

    let content3 = archive.read_file("direct.txt").unwrap();
    assert_eq!(content3, b"Direct data content");

    let content4 = archive.read_file("encrypted.txt").unwrap();
    assert_eq!(content4, b"Encrypted content");

    // Test file exists by checking list
    let file_entries = archive.list_with_hashes().unwrap();

    // MPQ paths use backslashes internally
    let file_entry = file_entries
        .iter()
        .find(|e| e.name == "files\\test1.txt")
        .unwrap_or_else(|| {
            panic!(
                "Could not find 'files\\test1.txt' in file list. Available files: {:?}",
                file_entries.iter().map(|e| &e.name).collect::<Vec<_>>()
            );
        });
    assert_eq!(file_entry.name, "files\\test1.txt");

    // Test listfile
    let listfile = archive.read_file("(listfile)").unwrap();
    let listfile_str = String::from_utf8_lossy(&listfile);
    // MPQ paths use backslashes in the listfile
    assert!(listfile_str.contains("files\\test1.txt"));
    assert!(listfile_str.contains("encrypted.txt"));
}

#[test]
fn test_v4_empty_archive() {
    // Test creating an empty v4 archive
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    ArchiveBuilder::new()
        .version(FormatVersion::V4)
        .listfile_option(ListfileOption::None)
        .build(path)
        .unwrap();

    // Open and verify
    let mut archive = Archive::open(path).unwrap();
    assert_eq!(archive.header().format_version, FormatVersion::V4);

    // Should have no files - empty archives may not have block tables
    let files = archive.list().unwrap_or_else(|e| {
        // Empty archives might not have block tables, which is expected
        if e.to_string().contains("No block table loaded") {
            Vec::new()
        } else {
            panic!("Unexpected error listing files: {}", e);
        }
    });
    assert_eq!(files.len(), 0);

    // MD5 checksums should still be valid
    let info = archive.get_info().unwrap();
    let md5_status = info.md5_status.unwrap();
    assert!(md5_status.header_valid);
    assert!(md5_status.hash_table_valid);
    assert!(md5_status.block_table_valid);
}

#[test]
fn test_v4_large_file_support() {
    // Test v4's ability to handle large file offsets
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    // Test v4's ability to handle large file offsets
    let mut builder = ArchiveBuilder::new()
        .version(FormatVersion::V4)
        .listfile_option(ListfileOption::None);

    // Add multiple 1MB files
    for i in 0..10 {
        let data = vec![i as u8; 1024 * 1024]; // 1MB each
        builder = builder.add_file_data(data, &format!("large_{}.dat", i));
    }

    builder.build(path).unwrap();

    // Verify all files can be read
    let mut archive = Archive::open(path).unwrap();

    // First, let's examine the archive structure
    println!("Archive info:");
    let info = archive.get_info().unwrap();
    println!("  Version: {:?}", info.format_version);
    println!("  Files: {}", info.file_count);
    println!("  Archive size: {} bytes", info.file_size);

    // List all files and their info
    println!("File information in archive:");
    for i in 0..10 {
        let filename = format!("large_{}.dat", i);
        if let Some(file_info) = archive.find_file(&filename).unwrap() {
            println!(
                "  {}: size={}, compressed_size={}, flags=0x{:08X}, pos=0x{:X}",
                filename,
                file_info.file_size,
                file_info.compressed_size,
                file_info.flags,
                file_info.file_pos
            );
        } else {
            println!("  {}: NOT FOUND", filename);
        }
    }
    println!();

    for i in 0..10 {
        let filename = format!("large_{}.dat", i);
        println!("Testing file {}: {}", i, filename);

        let data = archive.read_file(&filename).unwrap();
        assert_eq!(data.len(), 1024 * 1024, "File {} has wrong size", i);

        // Check data integrity with detailed error reporting
        let mut corruption_found = false;
        let mut first_corruption_offset = None;
        let mut corruption_count = 0;

        for (offset, &byte) in data.iter().enumerate() {
            if byte != i as u8 {
                if !corruption_found {
                    first_corruption_offset = Some(offset);
                    corruption_found = true;
                }
                corruption_count += 1;

                // Show first few corruptions for analysis
                if corruption_count <= 10 {
                    println!(
                        "  Corruption at offset {}: expected 0x{:02X}, got 0x{:02X}",
                        offset, i as u8, byte
                    );
                }
            }
        }

        if corruption_found {
            println!(
                "File {} corrupted: {} out of {} bytes incorrect",
                i,
                corruption_count,
                data.len()
            );
            println!("First corruption at offset: {:?}", first_corruption_offset);

            // Show pattern around first corruption
            if let Some(offset) = first_corruption_offset {
                let start = offset.saturating_sub(16);
                let end = (offset + 32).min(data.len());
                println!("Data around first corruption (offset {}):", offset);
                print!("  ");
                for j in start..end {
                    if j == offset {
                        print!("[{:02X}] ", data[j]);
                    } else {
                        print!("{:02X} ", data[j]);
                    }
                }
                println!();
            }

            panic!("File {} data integrity check failed", i);
        } else {
            println!("File {} integrity check passed", i);
        }
    }
}
