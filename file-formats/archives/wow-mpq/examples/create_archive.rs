//! Example of creating MPQ archives

use std::fs;
use wow_mpq::{Archive, ArchiveBuilder, FormatVersion, ListfileOption, compression::flags};

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
        println!("\nChecking {}:", archive_name);

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

    // Cleanup
    fs::remove_dir_all("test_files").ok();
    fs::remove_file("custom_list.txt").ok();

    Ok(())
}
