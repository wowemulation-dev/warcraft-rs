//! Comprehensive examples of creating MPQ archives
//!
//! This example demonstrates:
//! - Basic archive creation
//! - Files from disk
//! - Custom compression and encryption
//! - Attributes file generation
//! - Different MPQ versions

use std::fs;
use wow_mpq::{
    Archive, ArchiveBuilder, AttributesOption, FormatVersion, ListfileOption, compression::flags,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Simple archive creation
    println!("Creating simple archive...");
    ArchiveBuilder::new()
        .add_file_data(b"Hello, MPQ!".to_vec(), "readme.txt")
        .add_file_data(b"Some game data".to_vec(), "data/game.dat")
        .build("simple.mpq")?;

    println!("Created simple.mpq");

    // Example 2: Archive with files from disk
    println!("\nCreating archive from files...");

    // Create some test files
    fs::create_dir_all("test_files")?;
    fs::write("test_files/config.ini", "[Game]\nVersion=1.0")?;
    fs::write("test_files/data.bin", vec![0xDE, 0xAD, 0xBE, 0xEF])?;

    ArchiveBuilder::new()
        .version(FormatVersion::V2)
        .add_file("test_files/config.ini", "config.ini")
        .add_file("test_files/data.bin", "data.bin")
        .build("from_files.mpq")?;

    println!("Created from_files.mpq");

    // Example 3: Archive with custom options
    println!("\nCreating customized archive...");

    // Create external listfile
    fs::write(
        "custom_list.txt",
        "(listfile)\r\nreadme.txt\r\nsecret.dat\r\n",
    )?;

    ArchiveBuilder::new()
        .version(FormatVersion::V3)
        .block_size(4) // 8KB sectors
        .default_compression(flags::BZIP2)
        .listfile_option(ListfileOption::External("custom_list.txt".into()))
        .add_file_data_with_options(
            b"Not compressed".to_vec(),
            "readme.txt",
            0,      // No compression
            false,  // No encryption
            0x0409, // English (US) locale
        )
        .add_file_data_with_options(
            b"Super secret data".to_vec(),
            "secret.dat",
            flags::ZLIB,
            true, // Encrypted
            0,    // Default locale
        )
        .build("custom.mpq")?;

    println!("Created custom.mpq");

    // Example 4: Verify created archives
    println!("\nVerifying archives...");

    for archive_name in &["simple.mpq", "from_files.mpq", "custom.mpq"] {
        println!("\nChecking {archive_name}:");

        let mut archive = Archive::open(archive_name)?;

        // Try to list files
        match archive.list() {
            Ok(files) => {
                println!("  Files in archive:");
                for file in files {
                    println!(
                        "    - {} ({} bytes, compressed: {} bytes)",
                        file.name, file.size, file.compressed_size
                    );
                }
            }
            Err(_) => {
                println!("  No (listfile) found - cannot enumerate files");
            }
        }
    }

    // Example 5: Archive with attributes file
    println!("\nCreating archive with attributes file...");

    ArchiveBuilder::new()
        .version(FormatVersion::V2)
        .listfile_option(ListfileOption::Generate)
        .attributes_option(AttributesOption::GenerateFull) // CRC32 + MD5 + timestamp
        .default_compression(flags::ZLIB)
        .add_file_data(b"Test data with attributes".to_vec(), "test.txt")
        .add_file_data(b"Binary data".to_vec(), "data/binary.dat")
        .build("with_attributes.mpq")?;

    println!("Created with_attributes.mpq");

    // Example 6: Different versions comparison
    println!("\nCreating archives in different MPQ versions...");

    let test_data = vec![
        (
            "version_test.txt",
            b"Same content in all versions".as_slice(),
        ),
        ("data/test.bin", &[0x01, 0x02, 0x03, 0x04]),
    ];

    for (version, name) in [
        (FormatVersion::V1, "v1_archive.mpq"),
        (FormatVersion::V2, "v2_archive.mpq"),
        (FormatVersion::V3, "v3_archive.mpq"),
        (FormatVersion::V4, "v4_archive.mpq"),
    ] {
        let mut builder = ArchiveBuilder::new()
            .version(version)
            .listfile_option(ListfileOption::Generate);

        for (filename, content) in &test_data {
            builder = builder.add_file_data(content.to_vec(), filename);
        }

        builder.build(name)?;
        println!("  Created {name} (version {version:?})");
    }

    // Verification of all created archives
    let all_archives = [
        "simple.mpq",
        "from_files.mpq",
        "custom.mpq",
        "with_attributes.mpq",
        "v1_archive.mpq",
        "v2_archive.mpq",
        "v3_archive.mpq",
        "v4_archive.mpq",
    ];

    println!("\nFinal verification of all archives:");
    for archive_name in &all_archives {
        match Archive::open(archive_name) {
            Ok(mut archive) => match archive.get_info() {
                Ok(info) => {
                    println!(
                        "  {} - Format: {:?}, Files: {}, Size: {} KB",
                        archive_name,
                        info.format_version,
                        info.file_count,
                        info.file_size / 1024
                    );
                }
                Err(_) => {
                    println!("  {archive_name} - Could not get archive info");
                }
            },
            Err(e) => {
                println!("  {archive_name} - Error: {e}");
            }
        }
    }

    // Cleanup
    fs::remove_dir_all("test_files").ok();
    fs::remove_file("custom_list.txt").ok();

    println!(
        "\nExample complete! Created {} archives demonstrating different features.",
        all_archives.len()
    );
    println!("Archives can be cleaned up manually if desired.");

    Ok(())
}
