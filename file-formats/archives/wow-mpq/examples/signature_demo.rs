//! Example demonstrating MPQ digital signature operations
//!
//! This example shows how to:
//! - Generate weak signatures for MPQ archives
//! - Verify existing signatures
//! - Work with signature information

use std::fs;
use std::io::Cursor;
use std::path::Path;
use wow_mpq::crypto::{
    SignatureInfo, WEAK_SIGNATURE_FILE_SIZE, generate_weak_signature, parse_weak_signature,
    verify_weak_signature_stormlib,
};
use wow_mpq::{Archive, Result};

/// Demonstrate signature operations on a test archive
fn main() -> Result<()> {
    env_logger::init();

    // Create a simple test MPQ archive
    let test_data = create_test_archive()?;

    println!("Created test archive ({} bytes)", test_data.len());

    // Demonstrate weak signature generation
    demonstrate_weak_signature(&test_data)?;

    // Try to verify an existing archive with signature
    if let Some(path) = find_signed_archive() {
        println!("\nVerifying existing archive: {}", path.display());
        verify_existing_archive(&path)?;
    }

    Ok(())
}

/// Create a simple test MPQ archive
fn create_test_archive() -> Result<Vec<u8>> {
    use tempfile::NamedTempFile;
    use wow_mpq::ArchiveBuilder;

    // Create a temporary file for the archive
    let temp_file = NamedTempFile::new().map_err(wow_mpq::Error::Io)?;
    let temp_path = temp_file.path().to_path_buf();

    // Build the archive
    ArchiveBuilder::new()
        .add_file_data(b"Hello, World!".to_vec(), "test.txt")
        .build(&temp_path)?;

    // Read the archive back into memory
    let buffer = fs::read(&temp_path)?;

    Ok(buffer)
}

/// Demonstrate weak signature generation and verification
fn demonstrate_weak_signature(archive_data: &[u8]) -> Result<()> {
    println!("\n=== Weak Signature Demo ===");

    let archive_size = archive_data.len() as u64;

    // Create signature info (signature would be stored at end of archive)
    let sig_info = SignatureInfo::new_weak(
        0,                               // Archive starts at offset 0
        archive_size,                    // Archive size
        archive_size,                    // Signature file position (at end)
        WEAK_SIGNATURE_FILE_SIZE as u64, // Signature file size
        vec![],                          // Empty signature initially
    );

    println!("Archive size: {} bytes", archive_size);
    println!("Signature position: 0x{:X}", sig_info.begin_exclude);

    // Generate the weak signature
    let cursor = Cursor::new(archive_data);
    let signature_file = generate_weak_signature(cursor, &sig_info)?;

    println!("Generated signature file: {} bytes", signature_file.len());
    println!("Signature header: {:02X?}", &signature_file[0..8]);

    // Extract the actual signature from the file
    let signature = parse_weak_signature(&signature_file)?;
    println!("Extracted signature: {} bytes", signature.len());
    println!(
        "First 16 bytes: {:02X?}",
        &signature[0..16.min(signature.len())]
    );

    // Verify the signature
    let cursor = Cursor::new(archive_data);
    let mut verify_info = sig_info.clone();
    verify_info.signature = signature_file.clone();

    match verify_weak_signature_stormlib(cursor, &signature, &verify_info) {
        Ok(true) => println!("✓ Signature verified successfully!"),
        Ok(false) => println!("✗ Signature verification failed"),
        Err(e) => println!("✗ Error verifying signature: {}", e),
    }

    // Demonstrate what would happen with a tampered archive
    let mut tampered_data = archive_data.to_vec();
    if tampered_data.len() > 100 {
        tampered_data[100] ^= 0xFF; // Flip some bits
    }

    println!("\nVerifying tampered archive...");
    let cursor = Cursor::new(&tampered_data);
    match verify_weak_signature_stormlib(cursor, &signature, &verify_info) {
        Ok(true) => println!("✗ Unexpected: Tampered archive verified!"),
        Ok(false) => println!("✓ Correctly rejected tampered archive"),
        Err(e) => println!("Error: {}", e),
    }

    Ok(())
}

/// Find a signed MPQ archive in the test data directories
fn find_signed_archive() -> Option<std::path::PathBuf> {
    // Check common locations for signed MPQs
    let test_paths = [
        "/home/danielsreichenbach/Downloads/wow/1.12.1/Data/patch.MPQ",
        "/home/danielsreichenbach/Downloads/wow/2.4.3/Data/patch.mpq",
        "/home/danielsreichenbach/Downloads/wow/3.3.5a/Data/patch.MPQ",
    ];

    for path in &test_paths {
        let path = Path::new(path);
        if path.exists() {
            // Quick check if it might have a signature
            if let Ok(metadata) = fs::metadata(path) {
                // Check if file size suggests it might have a signature
                // (This is just a heuristic)
                println!(
                    "Found potential archive: {} ({} bytes)",
                    path.display(),
                    metadata.len()
                );
                return Some(path.to_path_buf());
            }
        }
    }

    None
}

/// Verify an existing archive with signature
fn verify_existing_archive(path: &Path) -> Result<()> {
    // Open the archive
    let mut archive = Archive::open(path)?;

    // Get list of files to check for signature
    let files = archive.list()?;

    // Check if it has a (signature) file
    let has_signature = files.iter().any(|entry| entry.name == "(signature)");

    if has_signature {
        println!("Archive contains (signature) file");

        // Read the signature file
        let sig_data = archive.read_file("(signature)")?;

        println!("Signature file size: {} bytes", sig_data.len());

        // Parse the signature
        match parse_weak_signature(&sig_data) {
            Ok(signature) => {
                println!("Parsed weak signature: {} bytes", signature.len());
                // Note: Full verification would require reading the entire archive
                // and calculating proper offsets
            }
            Err(e) => {
                println!("Failed to parse signature: {}", e);
            }
        }
    } else {
        println!("Archive does not contain (signature) file");
    }

    Ok(())
}
