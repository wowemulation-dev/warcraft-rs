//! Integration tests for archive creation

use std::fs;
use tempfile::TempDir;
use wow_mpq::{Archive, ArchiveBuilder, FormatVersion, ListfileOption, OpenOptions};

#[test]
fn test_create_empty_archive() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("empty.mpq");

    // Create empty archive
    let result = OpenOptions::new()
        .version(FormatVersion::V1)
        .create(&archive_path);

    assert!(result.is_ok());
    assert!(archive_path.exists());

    // Verify we can open it
    let archive = Archive::open(&archive_path).unwrap();
    assert_eq!(archive.header().format_version, FormatVersion::V1);
}

#[test]
fn test_create_archive_with_files() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("test.mpq");

    // Create test files
    let file1_path = temp_dir.path().join("file1.txt");
    let file2_path = temp_dir.path().join("file2.dat");
    fs::write(&file1_path, b"Hello, MPQ!").unwrap();
    fs::write(&file2_path, b"Binary data here").unwrap();

    // Build archive
    ArchiveBuilder::new()
        .version(FormatVersion::V1)
        .add_file(&file1_path, "test/file1.txt")
        .add_file(&file2_path, "test/file2.dat")
        .build(&archive_path)
        .unwrap();

    // Verify archive contents
    let mut archive = Archive::open(&archive_path).unwrap();

    // Check that files exist
    assert!(archive.find_file("test/file1.txt").unwrap().is_some());
    assert!(archive.find_file("test/file2.dat").unwrap().is_some());

    // Read and verify file contents
    let data1 = archive.read_file("test/file1.txt").unwrap();
    assert_eq!(data1, b"Hello, MPQ!");

    let data2 = archive.read_file("test/file2.dat").unwrap();
    assert_eq!(data2, b"Binary data here");
}

#[test]
fn test_create_archive_with_memory_files() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("memory.mpq");

    // Build archive with in-memory files
    ArchiveBuilder::new()
        .add_file_data(b"Memory file 1".to_vec(), "mem1.txt")
        .add_file_data(b"Memory file 2".to_vec(), "mem2.txt")
        .build(&archive_path)
        .unwrap();

    // Verify contents
    let mut archive = Archive::open(&archive_path).unwrap();

    let data1 = archive.read_file("mem1.txt").unwrap();
    assert_eq!(data1, b"Memory file 1");

    let data2 = archive.read_file("mem2.txt").unwrap();
    assert_eq!(data2, b"Memory file 2");
}

#[test]
fn test_listfile_generation() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("listfile.mpq");

    // Build archive with automatic listfile
    ArchiveBuilder::new()
        .add_file_data(b"File 1".to_vec(), "file1.txt")
        .add_file_data(b"File 2".to_vec(), "folder/file2.txt")
        .listfile_option(ListfileOption::Generate)
        .build(&archive_path)
        .unwrap();

    // Verify listfile exists and contains expected entries
    let mut archive = Archive::open(&archive_path).unwrap();

    let listfile_data = archive.read_file("(listfile)").unwrap();
    let listfile_content = String::from_utf8(listfile_data).unwrap();

    assert!(listfile_content.contains("file1.txt"));
    // MPQ paths use backslashes internally, so check for the normalized path
    assert!(listfile_content.contains("folder\\file2.txt"));
    assert!(listfile_content.contains("(listfile)"));
}

#[test]
fn test_external_listfile() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("external_list.mpq");
    let listfile_path = temp_dir.path().join("external.txt");

    // Create external listfile
    fs::write(
        &listfile_path,
        "file1.txt\r\nfile2.txt\r\ncustom_entry.txt\r\n",
    )
    .unwrap();

    // Build archive with external listfile
    ArchiveBuilder::new()
        .add_file_data(b"File 1".to_vec(), "file1.txt")
        .add_file_data(b"File 2".to_vec(), "file2.txt")
        .listfile_option(ListfileOption::External(listfile_path))
        .build(&archive_path)
        .unwrap();

    // Verify listfile contains external content
    let mut archive = Archive::open(&archive_path).unwrap();

    let listfile_data = archive.read_file("(listfile)").unwrap();
    let listfile_content = String::from_utf8(listfile_data).unwrap();

    assert!(listfile_content.contains("custom_entry.txt"));
}

#[test]
fn test_no_listfile() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("no_list.mpq");

    // Build archive without listfile
    ArchiveBuilder::new()
        .add_file_data(b"File 1".to_vec(), "file1.txt")
        .listfile_option(ListfileOption::None)
        .build(&archive_path)
        .unwrap();

    // Verify no listfile exists
    let archive = Archive::open(&archive_path).unwrap();
    assert!(archive.find_file("(listfile)").unwrap().is_none());
}

#[test]
fn test_compression_options() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("compressed.mpq");

    // Create some compressible data
    let data = "This is a test string that should compress well. ".repeat(100);

    // Build archive with compressed file
    ArchiveBuilder::new()
        .default_compression(wow_mpq::compression::flags::ZLIB)
        .add_file_data(data.as_bytes().to_vec(), "compressed.txt")
        .build(&archive_path)
        .unwrap();

    // Verify file is compressed
    let archive = Archive::open(&archive_path).unwrap();

    if let Some(file_info) = archive.find_file("compressed.txt").unwrap() {
        assert!(file_info.is_compressed());
        assert!(file_info.compressed_size < file_info.file_size);

        // Verify we can still read it correctly
        let mut archive = Archive::open(&archive_path).unwrap();
        let read_data = archive.read_file("compressed.txt").unwrap();
        assert_eq!(read_data, data.as_bytes());
    } else {
        panic!("File not found in archive");
    }
}

#[test]
fn test_uncompressed_file() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("uncompressed.mpq");

    // Build archive with uncompressed file
    ArchiveBuilder::new()
        .add_file_data_with_options(
            b"Uncompressed data".to_vec(),
            "uncompressed.txt",
            0,     // No compression
            false, // No encryption
            0,     // Default locale
        )
        .build(&archive_path)
        .unwrap();

    // Verify file is not compressed
    let archive = Archive::open(&archive_path).unwrap();

    if let Some(file_info) = archive.find_file("uncompressed.txt").unwrap() {
        assert!(!file_info.is_compressed());
        assert_eq!(file_info.compressed_size, file_info.file_size);
    } else {
        panic!("File not found in archive");
    }
}

#[test]
fn test_large_file_sectors() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("large.mpq");

    // Create a large file that will span multiple sectors
    let large_data = vec![0xAB; 16 * 1024]; // 16KB should span multiple 4KB sectors

    // Build archive
    ArchiveBuilder::new()
        .block_size(3) // 4KB sectors
        .add_file_data(large_data.clone(), "large.dat")
        .build(&archive_path)
        .unwrap();

    // Verify we can read it back correctly
    let mut archive = Archive::open(&archive_path).unwrap();
    let read_data = archive.read_file("large.dat").unwrap();
    assert_eq!(read_data, large_data);
}

#[test]
fn test_hash_table_sizing() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("sized.mpq");

    // Add many files to test hash table sizing
    let mut builder = ArchiveBuilder::new();
    for i in 0..50 {
        builder = builder.add_file_data(
            format!("File {}", i).into_bytes(),
            &format!("file_{:03}.txt", i),
        );
    }

    builder.build(&archive_path).unwrap();

    // Verify all files can be found
    let mut archive = Archive::open(&archive_path).unwrap();
    for i in 0..50 {
        let filename = format!("file_{:03}.txt", i);
        assert!(archive.find_file(&filename).unwrap().is_some());

        let data = archive.read_file(&filename).unwrap();
        assert_eq!(data, format!("File {}", i).as_bytes());
    }
}

#[test]
fn test_path_normalization() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("paths.mpq");

    // Build archive with forward slashes
    ArchiveBuilder::new()
        .add_file_data(b"Test file".to_vec(), "folder/subfolder/file.txt")
        .build(&archive_path)
        .unwrap();

    // Verify we can find it with backslashes
    let archive = Archive::open(&archive_path).unwrap();
    assert!(
        archive
            .find_file("folder\\subfolder\\file.txt")
            .unwrap()
            .is_some()
    );

    // And with forward slashes
    assert!(
        archive
            .find_file("folder/subfolder/file.txt")
            .unwrap()
            .is_some()
    );
}

#[test]
fn test_case_insensitive_lookup() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("case.mpq");

    // Build archive
    ArchiveBuilder::new()
        .add_file_data(b"Test".to_vec(), "TestFile.TXT")
        .build(&archive_path)
        .unwrap();

    // Verify case-insensitive lookup works
    let archive = Archive::open(&archive_path).unwrap();
    assert!(archive.find_file("testfile.txt").unwrap().is_some());
    assert!(archive.find_file("TESTFILE.TXT").unwrap().is_some());
    assert!(archive.find_file("TestFile.TXT").unwrap().is_some());
}

#[test]
fn test_duplicate_file_error() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("duplicate.mpq");

    // Try to build archive with duplicate files
    let result = ArchiveBuilder::new()
        .add_file_data(b"File 1".to_vec(), "test.txt")
        .add_file_data(b"File 2".to_vec(), "test.txt") // Duplicate name
        .build(&archive_path);

    // Should fail with duplicate file error
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Duplicate file"));
}

#[test]
fn test_atomic_write_on_failure() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("atomic.mpq");

    // Create a file at the target location
    fs::write(&archive_path, b"Original content").unwrap();

    // Try to create an archive with an invalid configuration that will fail
    // For example, try to read a non-existent file
    let result = ArchiveBuilder::new()
        .add_file("/non/existent/file.txt", "test.txt")
        .build(&archive_path);

    // Build should fail
    assert!(result.is_err());

    // Original file should be unchanged
    let content = fs::read(&archive_path).unwrap();
    assert_eq!(content, b"Original content");
}

#[test]
fn test_different_format_versions() {
    let temp_dir = TempDir::new().unwrap();

    // Test V1 format
    let v1_path = temp_dir.path().join("v1.mpq");
    ArchiveBuilder::new()
        .version(FormatVersion::V1)
        .add_file_data(b"V1 data".to_vec(), "test.txt")
        .build(&v1_path)
        .unwrap();

    let archive = Archive::open(&v1_path).unwrap();
    assert_eq!(archive.header().format_version, FormatVersion::V1);
    assert_eq!(archive.header().header_size, 32);

    // Test V2 format
    let v2_path = temp_dir.path().join("v2.mpq");
    ArchiveBuilder::new()
        .version(FormatVersion::V2)
        .add_file_data(b"V2 data".to_vec(), "test.txt")
        .build(&v2_path)
        .unwrap();

    let archive = Archive::open(&v2_path).unwrap();
    assert_eq!(archive.header().format_version, FormatVersion::V2);
    assert_eq!(archive.header().header_size, 44);

    // Test V3 format
    let v3_path = temp_dir.path().join("v3.mpq");
    ArchiveBuilder::new()
        .version(FormatVersion::V3)
        .add_file_data(b"V3 data".to_vec(), "test.txt")
        .build(&v3_path)
        .unwrap();

    let archive = Archive::open(&v3_path).unwrap();
    assert_eq!(archive.header().format_version, FormatVersion::V3);
    assert_eq!(archive.header().header_size, 68);
}

#[test]
fn test_encrypted_file_round_trip() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("encrypted.mpq");

    // Test data
    let test_data = b"This is secret encrypted data that should be protected!";

    // Build archive with encrypted file
    ArchiveBuilder::new()
        .add_file_data_with_options(
            test_data.to_vec(),
            "secret.txt",
            wow_mpq::compression::flags::ZLIB,
            true, // Enable encryption
            0,    // Default locale
        )
        .build(&archive_path)
        .unwrap();

    // Verify file is encrypted
    let archive = Archive::open(&archive_path).unwrap();
    if let Some(file_info) = archive.find_file("secret.txt").unwrap() {
        assert!(file_info.is_encrypted());
    } else {
        panic!("Encrypted file not found");
    }

    // Verify we can decrypt and read it correctly
    let mut archive = Archive::open(&archive_path).unwrap();
    let decrypted_data = archive.read_file("secret.txt").unwrap();
    assert_eq!(decrypted_data, test_data);
}

#[test]
fn test_encrypted_file_with_fix_key() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("fix_key.mpq");

    // Test data
    let test_data = b"FIX_KEY encrypted data";

    // Build archive with FIX_KEY encrypted file
    ArchiveBuilder::new()
        .add_file_data_with_encryption(
            test_data.to_vec(),
            "fix_key.dat",
            0,    // No compression
            true, // Use FIX_KEY
            0,    // Default locale
        )
        .build(&archive_path)
        .unwrap();

    // Verify file has both encrypted and fix_key flags
    let archive = Archive::open(&archive_path).unwrap();
    if let Some(file_info) = archive.find_file("fix_key.dat").unwrap() {
        assert!(file_info.is_encrypted());
        assert!(file_info.has_fix_key());
    } else {
        panic!("FIX_KEY encrypted file not found");
    }

    // Verify we can decrypt and read it correctly
    let mut archive = Archive::open(&archive_path).unwrap();
    let decrypted_data = archive.read_file("fix_key.dat").unwrap();
    assert_eq!(decrypted_data, test_data);
}

#[test]
fn test_encrypted_large_file() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("encrypted_large.mpq");

    // Create large data that spans multiple sectors
    let large_data: Vec<u8> = (0..20000).map(|i| (i % 256) as u8).collect();

    // Build archive with encrypted large file
    ArchiveBuilder::new()
        .block_size(3) // 4KB sectors
        .add_file_data_with_options(
            large_data.clone(),
            "large_encrypted.bin",
            wow_mpq::compression::flags::ZLIB,
            true, // Enable encryption
            0,    // Default locale
        )
        .build(&archive_path)
        .unwrap();

    // Verify file is encrypted and compressed
    let archive = Archive::open(&archive_path).unwrap();
    if let Some(file_info) = archive.find_file("large_encrypted.bin").unwrap() {
        assert!(file_info.is_encrypted());
        assert!(file_info.is_compressed());
        assert!(!file_info.is_single_unit()); // Should be multi-sector
    } else {
        panic!("Large encrypted file not found");
    }

    // Verify we can decrypt and read it correctly
    let mut archive = Archive::open(&archive_path).unwrap();
    let decrypted_data = archive.read_file("large_encrypted.bin").unwrap();
    assert_eq!(decrypted_data, large_data);
}

#[test]
fn test_mixed_encrypted_and_plain_files() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("mixed.mpq");

    // Build archive with mix of encrypted and plain files
    ArchiveBuilder::new()
        .add_file_data(b"Plain text file".to_vec(), "plain.txt")
        .add_file_data_with_options(
            b"Encrypted file".to_vec(),
            "encrypted.txt",
            0,
            true, // Encrypted
            0,
        )
        .add_file_data_with_encryption(
            b"FIX_KEY encrypted".to_vec(),
            "fix_key.txt",
            0,
            true, // Use FIX_KEY
            0,
        )
        .build(&archive_path)
        .unwrap();

    // Open archive and verify all files
    let mut archive = Archive::open(&archive_path).unwrap();

    // Check plain file
    let plain_info = archive.find_file("plain.txt").unwrap().unwrap();
    assert!(!plain_info.is_encrypted());
    let plain_data = archive.read_file("plain.txt").unwrap();
    assert_eq!(plain_data, b"Plain text file");

    // Check encrypted file
    let enc_info = archive.find_file("encrypted.txt").unwrap().unwrap();
    assert!(enc_info.is_encrypted());
    assert!(!enc_info.has_fix_key());
    let enc_data = archive.read_file("encrypted.txt").unwrap();
    assert_eq!(enc_data, b"Encrypted file");

    // Check FIX_KEY encrypted file
    let fix_info = archive.find_file("fix_key.txt").unwrap().unwrap();
    assert!(fix_info.is_encrypted());
    assert!(fix_info.has_fix_key());
    let fix_data = archive.read_file("fix_key.txt").unwrap();
    assert_eq!(fix_data, b"FIX_KEY encrypted");
}

#[test]
fn test_encrypted_compressed_file() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("enc_comp.mpq");

    // Create compressible data
    let data = "Compressible encrypted data! ".repeat(100);

    // Build archive with encrypted and compressed file
    ArchiveBuilder::new()
        .add_file_data_with_options(
            data.as_bytes().to_vec(),
            "encrypted_compressed.txt",
            wow_mpq::compression::flags::ZLIB,
            true, // Enable encryption
            0,    // Default locale
        )
        .build(&archive_path)
        .unwrap();

    // Verify file is both encrypted and compressed
    let archive = Archive::open(&archive_path).unwrap();
    if let Some(file_info) = archive.find_file("encrypted_compressed.txt").unwrap() {
        assert!(file_info.is_encrypted());
        assert!(file_info.is_compressed());
        assert!(file_info.compressed_size < file_info.file_size);
    } else {
        panic!("Encrypted compressed file not found");
    }

    // Verify we can decrypt and decompress correctly
    let mut archive = Archive::open(&archive_path).unwrap();
    let decrypted_data = archive.read_file("encrypted_compressed.txt").unwrap();
    assert_eq!(decrypted_data, data.as_bytes());
}

#[test]
fn test_hi_block_table_creation_v2() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("hi_block_v2.mpq");

    // Create archive with V2 format (supports Hi-block tables)
    ArchiveBuilder::new()
        .version(FormatVersion::V2)
        .add_file_data(b"Test data for V2".to_vec(), "test.txt")
        .add_file_data(b"Another file".to_vec(), "file2.txt")
        .build(&archive_path)
        .unwrap();

    // Verify archive can be opened and read
    let archive = Archive::open(&archive_path).unwrap();
    assert_eq!(archive.header().format_version, FormatVersion::V2);

    // Check that hi_block_table_pos is set in header (even if 0)
    assert!(archive.header().hi_block_table_pos.is_some());

    // Verify files can be read
    let mut archive = Archive::open(&archive_path).unwrap();
    let data1 = archive.read_file("test.txt").unwrap();
    assert_eq!(data1, b"Test data for V2");
    let data2 = archive.read_file("file2.txt").unwrap();
    assert_eq!(data2, b"Another file");
}

#[test]
fn test_hi_block_table_creation_v3() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("hi_block_v3.mpq");

    // Create archive with V3 format (also supports Hi-block tables)
    ArchiveBuilder::new()
        .version(FormatVersion::V3)
        .add_file_data(b"Test data for V3".to_vec(), "test.txt")
        .add_file_data(b"V3 supports HET/BET".to_vec(), "advanced.txt")
        .build(&archive_path)
        .unwrap();

    // Verify archive
    let archive = Archive::open(&archive_path).unwrap();
    assert_eq!(archive.header().format_version, FormatVersion::V3);

    // V3 should have hi_block_table_pos
    assert!(archive.header().hi_block_table_pos.is_some());

    // V3 should also have 64-bit archive size
    assert!(archive.header().archive_size_64.is_some());

    // Verify files
    let mut archive = Archive::open(&archive_path).unwrap();
    let data1 = archive.read_file("test.txt").unwrap();
    assert_eq!(data1, b"Test data for V3");
    let data2 = archive.read_file("advanced.txt").unwrap();
    assert_eq!(data2, b"V3 supports HET/BET");
}

#[test]
fn test_v1_no_hi_block_table() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("v1_no_hi_block.mpq");

    // V1 format should not have Hi-block table
    ArchiveBuilder::new()
        .version(FormatVersion::V1)
        .add_file_data(b"V1 data".to_vec(), "old.txt")
        .build(&archive_path)
        .unwrap();

    // Verify V1 format doesn't have hi-block table fields
    let archive = Archive::open(&archive_path).unwrap();
    assert_eq!(archive.header().format_version, FormatVersion::V1);
    assert!(archive.header().hi_block_table_pos.is_none());
    assert!(archive.header().hash_table_pos_hi.is_none());
    assert!(archive.header().block_table_pos_hi.is_none());
}
