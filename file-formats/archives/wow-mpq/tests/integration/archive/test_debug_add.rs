//! Debug test for file addition

use tempfile::TempDir;
use wow_mpq::{Archive, ArchiveBuilder, MutableArchive};

#[test]
fn test_debug_add_file() {
    let _ = env_logger::try_init();
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("test.mpq");

    // Create a simple test archive
    let builder = ArchiveBuilder::new().add_file_data(b"Test content 1".to_vec(), "file1.txt");
    builder.build(&archive_path).unwrap();

    println!("Created archive at: {archive_path:?}");

    // Check initial state and print header info
    {
        let mut archive = Archive::open(&archive_path).unwrap();
        let header = archive.header();
        println!("Initial header info:");
        println!("  Format version: {:?}", header.format_version);
        println!("  Hash table size: {}", header.hash_table_size);
        println!("  Block table size: {}", header.block_table_size);
        println!("  Hash table pos: 0x{:X}", header.hash_table_pos);
        println!("  Block table pos: 0x{:X}", header.block_table_pos);

        let files = archive.list().unwrap();
        println!("Initial files: {}", files.len());
        for file in &files {
            println!("  - {}", file.name);
        }
    }

    // Open for modification and add a file
    {
        let mut mutable_archive = MutableArchive::open(&archive_path).unwrap();
        println!("\nOpened archive for modification");

        // Debug: Check tables before adding
        let header_before = mutable_archive.archive().header();
        println!(
            "Before adding - Block table size: {}",
            header_before.block_table_size
        );

        // Add a file with no compression
        let options = wow_mpq::AddFileOptions::new()
            .compression(wow_mpq::compression::CompressionMethod::None);
        mutable_archive
            .add_file_data(b"New file content", "new_file.txt", options)
            .unwrap();
        println!("Added new_file.txt");

        // Debug: Print internal state before flush
        println!("\nBefore flush:");
        let (block_count, hash_count) = mutable_archive.debug_state();
        if let Some(count) = block_count {
            println!("  Block table entries: {count}");
        }
        if let Some(count) = hash_count {
            println!("  Hash table size: {count}");
        }

        // Explicitly flush
        mutable_archive.flush().unwrap();
        println!("Flushed changes");
    }

    // Check final state
    {
        println!("\nReopening archive to check changes...");
        let mut archive = Archive::open(&archive_path).unwrap();
        let header = archive.header();
        println!("Final header info:");
        println!("  Format version: {:?}", header.format_version);
        println!("  Hash table size: {}", header.hash_table_size);
        println!("  Block table size: {}", header.block_table_size);
        println!("  Hash table pos: 0x{:X}", header.hash_table_pos);
        println!("  Block table pos: 0x{:X}", header.block_table_pos);

        let files = archive.list().unwrap();
        println!("Final files: {}", files.len());
        for file in &files {
            println!("  - {}", file.name);
        }

        // Try to read the new file
        match archive.read_file("new_file.txt") {
            Ok(content) => {
                println!("Successfully read new_file.txt: {} bytes", content.len());
                assert_eq!(content, b"New file content");
            }
            Err(e) => {
                println!("Failed to read new_file.txt: {e}");

                // Debug: Try to find the file in hash table
                if let Some(entry) = archive.find_file("new_file.txt").unwrap() {
                    println!("Found hash entry for new_file.txt:");
                    println!("  Block index: {}", entry.block_index);
                    println!("  File position: {}", entry.file_pos);
                    println!("  Compressed size: {}", entry.compressed_size);
                }
            }
        }
    }
}
