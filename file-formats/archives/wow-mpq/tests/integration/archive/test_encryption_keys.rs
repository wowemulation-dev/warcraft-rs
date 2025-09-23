//! Test to diagnose encryption key generation differences

use std::io::{Read, Seek};
use tempfile::TempDir;
use wow_mpq::compression::CompressionMethod;
use wow_mpq::crypto::{hash_string, hash_type};
use wow_mpq::{AddFileOptions, Archive, ArchiveBuilder, MutableArchive};

#[test]
fn test_encryption_key_diagnosis() {
    let _ = env_logger::try_init();
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("test_encryption.mpq");

    let filename = "memory\\file.dat";
    let content = b"Memory file content";

    println!("=== Encryption Key Diagnosis ===");
    println!("Filename: {filename}");
    println!("Content: {content:?}");
    println!("Content length: {}", content.len());

    // Create a simple test archive
    let builder = ArchiveBuilder::new().add_file_data(b"Test content 1".to_vec(), "file1.txt");
    builder.build(&archive_path).unwrap();

    // Calculate encryption key as our modification code does
    let modification_key = hash_string(filename, hash_type::FILE_KEY);
    println!("Modification code key: 0x{modification_key:08X}");

    // Open for modification and add encrypted file
    {
        let mut mutable_archive = MutableArchive::open(&archive_path).unwrap();

        let options = AddFileOptions::new()
            .compression(CompressionMethod::None) // No compression, just encryption
            .encrypt();

        mutable_archive
            .add_file_data(content, filename, options)
            .unwrap();
        mutable_archive.flush().unwrap();
    }

    // Now read back and check key calculation
    {
        let archive = Archive::open(&archive_path).unwrap();

        // Calculate key as reading code does
        if let Some(file_info) = archive.find_file(filename).unwrap() {
            println!("File info:");
            println!("  Flags: 0x{:08X}", file_info.flags);
            println!("  Compressed size: {}", file_info.compressed_size);
            println!("  File size: {}", file_info.file_size);
            println!("  File pos: 0x{:08X}", file_info.file_pos);
            println!("  Archive offset: 0x{:08X}", archive.archive_offset());
            println!("  Effective file pos: 0x{:08X}", file_info.file_pos);
            println!("  Is encrypted: {}", file_info.is_encrypted());
            println!("  Has fix key: {}", file_info.has_fix_key());

            // Recreate the key calculation from archive.rs:1689-1701
            let reading_key = if file_info.is_encrypted() {
                let base_key = hash_string(filename, hash_type::FILE_KEY);
                if file_info.has_fix_key() {
                    // Apply FIX_KEY modification
                    let file_pos = (file_info.file_pos - archive.archive_offset()) as u32;
                    let file_size_for_key = file_info.file_size as u32;
                    (base_key.wrapping_add(file_pos)) ^ file_size_for_key
                } else {
                    base_key
                }
            } else {
                0
            };

            println!("Reading code key: 0x{reading_key:08X}");

            if modification_key == reading_key {
                println!("✅ Keys match!");
            } else {
                println!("❌ Keys do NOT match!");
                println!(
                    "  Difference: modification uses base key, reading might use modified key"
                );
            }

            // Let's manually test the decryption process
            println!("\n=== Manual Decryption Test ===");

            // Read raw encrypted data
            let mut file = std::fs::File::open(&archive_path).unwrap();
            file.seek(std::io::SeekFrom::Start(file_info.file_pos))
                .unwrap();
            let mut raw_data = vec![0u8; file_info.compressed_size as usize];
            file.read_exact(&mut raw_data).unwrap();

            println!("Raw encrypted data: {raw_data:?}");

            // Test our decryption function directly
            let mut test_data = raw_data.clone();
            wow_mpq::decrypt_file_data(&mut test_data, reading_key);
            test_data.truncate(file_info.file_size as usize);

            println!("After decryption: {test_data:?}");

            if test_data == content {
                println!("✅ Manual decryption works!");
            } else {
                println!("❌ Manual decryption failed!");
            }

            // Try to read the file through normal API
            match archive.read_file(filename) {
                Ok(read_content) => {
                    println!("\nFile read through API: {} bytes", read_content.len());
                    println!("API content: {read_content:?}");
                    if read_content == content {
                        println!("✅ API content matches!");
                    } else {
                        println!("❌ API content does NOT match!");
                        println!("Expected: {content:?}");
                        println!("Got:      {read_content:?}");
                    }
                }
                Err(e) => {
                    println!("❌ Failed to read file: {e}");
                }
            }
        } else {
            println!("❌ File not found in archive");
        }
    }
}
