//! Example of bulk modifications to MPQ archives
//!
//! This demonstrates:
//! - Batch operations on archives
//! - Working with different compression methods
//! - Managing encrypted files
//! - Handling file conflicts

use std::error::Error;
use wow_mpq::compression::CompressionMethod;
use wow_mpq::{AddFileOptions, Archive, ArchiveBuilder, MutableArchive};

fn main() -> Result<(), Box<dyn Error>> {
    let archive_path = "bulk_test.mpq";

    // Create initial archive with various file types
    println!("📦 Creating test archive with mixed content...");
    create_test_archive(archive_path)?;

    // Demonstrate bulk modifications
    println!("\n🔧 Performing bulk modifications...");
    bulk_modify_archive(archive_path)?;

    // Show final state
    println!("\n📊 Final archive contents:");
    list_archive_details(archive_path)?;

    // Clean up
    std::fs::remove_file(archive_path)?;

    Ok(())
}

fn create_test_archive(path: &str) -> Result<(), Box<dyn Error>> {
    let mut builder = ArchiveBuilder::new();

    // Add various types of files
    let file_types = [
        ("docs/readme.txt", b"Documentation for the project".to_vec()),
        ("docs/license.txt", b"MIT License".to_vec()),
        (
            "src/main.cpp",
            b"#include <iostream>\nint main() { }".to_vec(),
        ),
        ("assets/texture.tga", vec![0xAA; 1024]), // Binary data
        ("data/config.ini", b"[Settings]\nvalue=42".to_vec()),
    ];

    let file_count = file_types.len();

    for (name, data) in file_types {
        builder = builder.add_file_data(data, name);
    }

    builder.build(path)?;
    println!("✅ Created archive with {} files", file_count);

    Ok(())
}

fn bulk_modify_archive(path: &str) -> Result<(), Box<dyn Error>> {
    let mut archive = MutableArchive::open(path)?;

    // 1. Update all text files with compression
    println!("\n📝 Updating text files with compression...");
    let text_files = [
        ("docs/changelog.txt", "Version 1.0 - Initial release"),
        ("docs/api.txt", "API Documentation"),
        (
            "src/utils.cpp",
            "#include \"utils.h\"\n// Utility functions",
        ),
    ];

    for (name, content) in &text_files {
        let options = AddFileOptions::new().compression(CompressionMethod::Zlib);

        archive.add_file_data(content.as_bytes(), name, options)?;
        println!("  ✅ Added: {}", name);
    }

    // 2. Add encrypted sensitive files
    println!("\n🔐 Adding encrypted files...");
    let sensitive_files = [
        ("keys/private.key", "-----BEGIN PRIVATE KEY-----"),
        ("config/database.conf", "password=secret123"),
    ];

    for (name, content) in &sensitive_files {
        let options = AddFileOptions::new()
            .compression(CompressionMethod::Zlib)
            .encrypt();

        archive.add_file_data(content.as_bytes(), name, options)?;
        println!("  🔒 Added encrypted: {}", name);
    }

    // 3. Add large binary files with optimal compression
    println!("\n📦 Adding large binary files...");
    let large_data = vec![0xFF; 100_000]; // 100KB of data

    let compression_tests = [
        ("binary/test_none.bin", CompressionMethod::None),
        ("binary/test_zlib.bin", CompressionMethod::Zlib),
        ("binary/test_bzip2.bin", CompressionMethod::BZip2),
    ];

    for (name, method) in &compression_tests {
        let options = AddFileOptions::new().compression(*method);

        archive.add_file_data(&large_data, name, options)?;
        println!("  ✅ Added with {:?}: {}", method, name);
    }

    // 4. Test file replacement behavior
    println!("\n🔄 Testing file replacement...");

    // Try to add existing file without replace flag (should fail)
    let no_replace = AddFileOptions::new().replace_existing(false);

    match archive.add_file_data(b"New content", "docs/readme.txt", no_replace) {
        Err(e) => println!("  ✅ Correctly rejected duplicate: {}", e),
        Ok(_) => println!("  ❌ Error: should have rejected duplicate!"),
    }

    // Now replace it
    let replace = AddFileOptions::new().replace_existing(true);

    archive.add_file_data(b"Updated readme content", "docs/readme.txt", replace)?;
    println!("  ✅ Successfully replaced docs/readme.txt");

    // 5. Batch rename operation
    println!("\n✏️  Performing batch renames...");
    let renames = [
        ("src/main.cpp", "source/main.cpp"),
        ("src/utils.cpp", "source/utils.cpp"),
    ];

    for (old, new) in &renames {
        archive.rename_file(old, new)?;
        println!("  ✅ Renamed: {} -> {}", old, new);
    }

    // Save all changes
    println!("\n💾 Flushing all changes...");
    archive.flush()?;

    Ok(())
}

fn list_archive_details(path: &str) -> Result<(), Box<dyn Error>> {
    let mut archive = Archive::open(path)?;
    let files = archive.list_all()?;

    println!("\nTotal files: {}", files.len());
    println!("\nFile listing:");
    println!(
        "{:<30} {:>10} {:>10} {:>10}",
        "Name", "Size", "Compressed", "Encrypted"
    );
    println!("{:-<60}", "");

    for entry in files {
        if let Some(info) = archive.find_file(&entry.name)? {
            let encrypted = if info.is_encrypted() { "Yes" } else { "No" };
            println!(
                "{:<30} {:>10} {:>10} {:>10}",
                entry.name, info.file_size, info.compressed_size, encrypted
            );
        }
    }

    Ok(())
}
