//! Example of creating an MPQ archive with encrypted files

use wow_mpq::{ArchiveBuilder, Result, compression::flags as CompressionFlags};

fn main() -> Result<()> {
    println!("Creating MPQ archive with encrypted files...");

    // Create an archive with various encryption options
    ArchiveBuilder::new()
        // Add a plain file (no encryption)
        .add_file_data(
            b"This is a plain text file that anyone can read.".to_vec(),
            "plain.txt",
        )
        // Add an encrypted file
        .add_file_data_with_options(
            b"This is a secret encrypted file!".to_vec(),
            "secret.txt",
            CompressionFlags::ZLIB,
            true, // Enable encryption
            0,    // Default locale
        )
        // Add an encrypted file with FIX_KEY flag
        // FIX_KEY adjusts the encryption key based on the file's position in the archive
        .add_file_data_with_encryption(
            b"This file uses FIX_KEY encryption for extra security.".to_vec(),
            "fix_key_secret.dat",
            CompressionFlags::ZLIB,
            true, // Use FIX_KEY
            0,    // Default locale
        )
        // Add a large encrypted file that spans multiple sectors
        .add_file_data_with_options(
            vec![0x42; 20000], // 20KB of data
            "large_encrypted.bin",
            CompressionFlags::ZLIB,
            true, // Enable encryption
            0,    // Default locale
        )
        .build("encrypted_example.mpq")?;

    println!("Archive created successfully!");

    // Verify the archive by reading it back
    println!("\nVerifying archive contents...");
    let mut archive = wow_mpq::Archive::open("encrypted_example.mpq")?;

    // List all files
    let files = archive.list()?;
    for file in &files {
        println!(
            "- {} ({}{}{})",
            file.name,
            if file.is_encrypted() {
                "encrypted"
            } else {
                "plain"
            },
            if file.is_compressed() {
                ", compressed"
            } else {
                ""
            },
            if file.has_fix_key() { ", FIX_KEY" } else { "" }
        );
    }

    // Read and verify each file
    println!("\nReading files:");

    let plain = archive.read_file("plain.txt")?;
    println!("plain.txt: {}", String::from_utf8_lossy(&plain));

    let secret = archive.read_file("secret.txt")?;
    println!("secret.txt: {}", String::from_utf8_lossy(&secret));

    let fix_key = archive.read_file("fix_key_secret.dat")?;
    println!("fix_key_secret.dat: {}", String::from_utf8_lossy(&fix_key));

    let large = archive.read_file("large_encrypted.bin")?;
    println!("large_encrypted.bin: {} bytes", large.len());

    // Don't clean up - keep the file for testing
    println!("\nArchive saved as: encrypted_example.mpq");

    Ok(())
}
