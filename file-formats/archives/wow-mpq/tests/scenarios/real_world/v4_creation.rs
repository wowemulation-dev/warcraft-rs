use std::fs;
use std::io::Read;
use tempfile::NamedTempFile;
use wow_mpq::{Archive, ArchiveBuilder, FormatVersion};

#[test]
fn test_v4_archive_creation() {
    // Create a temporary file for the archive
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    // Create a v4 archive
    ArchiveBuilder::new()
        .version(FormatVersion::V4)
        .add_file_data(b"Hello, V4 MPQ!".to_vec(), "test.txt")
        .build(path)
        .unwrap();

    // Debug: Check what's at the start of the file AND look for MPQ signature
    let mut file = fs::File::open(path).unwrap();
    let mut all_data = Vec::new();
    file.read_to_end(&mut all_data).unwrap();

    println!("File size: {} bytes", all_data.len());
    println!("First 64 bytes of archive:");
    for (i, chunk) in all_data[..64.min(all_data.len())].chunks(16).enumerate() {
        print!("{:04X}: ", i * 16);
        for b in chunk {
            print!("{:02X} ", b);
        }
        println!();
    }

    // Look for MPQ signature
    for i in 0..all_data.len().saturating_sub(4) {
        if all_data[i..i + 4] == [0x4D, 0x50, 0x51, 0x1A] {
            println!("\nFound MPQ signature at offset 0x{:X}", i);
            break;
        }
    }

    // Open and verify the archive
    let mut archive = Archive::open(path).unwrap();

    // Check that it's v4
    assert_eq!(archive.header().format_version, FormatVersion::V4);

    // Read the file back
    let data = archive.read_file("test.txt").unwrap();
    assert_eq!(data, b"Hello, V4 MPQ!");

    // Get archive info to check MD5 status
    let info = archive.get_info().unwrap();

    // For v4 archives, MD5 status should be present
    assert!(info.md5_status.is_some());

    if let Some(md5_status) = info.md5_status {
        println!("MD5 Status:");
        println!("  Header: {}", md5_status.header_valid);
        println!("  Hash table: {}", md5_status.hash_table_valid);
        println!("  Block table: {}", md5_status.block_table_valid);
        println!("  HET table: {}", md5_status.het_table_valid);
        println!("  BET table: {}", md5_status.bet_table_valid);
    }
}
