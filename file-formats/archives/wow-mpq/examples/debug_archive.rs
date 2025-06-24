//! Example demonstrating the debug utilities for MPQ archives
//!
//! This example shows how to use the debug-utils feature to analyze
//! MPQ archive internals for debugging purposes.

use std::env;
use wow_mpq::debug::*;
use wow_mpq::{Archive, ArchiveInfo};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <mpq_file> [options]", args[0]);
        eprintln!("\nOptions:");
        eprintln!("  --hash-table     Display hash table entries");
        eprintln!("  --block-table    Display block table entries");
        eprintln!("  --structure      Show archive structure visualization");
        eprintln!("  --header         Dump header details");
        eprintln!("  --hex <file>     Hex dump a specific file");
        eprintln!("  --trace <file>   Trace extraction of a specific file");
        eprintln!("  --compression    Analyze compression methods");
        eprintln!("  --all            Show all debug information");
        return Ok(());
    }

    let mpq_path = &args[1];
    let options: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();

    println!("🔍 Debug Analysis: {}", mpq_path);
    println!("=====================================\n");

    // Open the archive
    let mut archive = Archive::open(mpq_path)?;
    let info = archive.get_info()?;

    // Display based on options
    if options.is_empty() || options.contains(&"--all") {
        // Show basic info if no specific options
        show_basic_info(&info);
    }

    if options.contains(&"--header") || options.contains(&"--all") {
        show_header_debug(&mut archive)?;
    }

    if options.contains(&"--structure") || options.contains(&"--all") {
        show_structure_visualization(&info);
    }

    if options.contains(&"--hash-table") || options.contains(&"--all") {
        show_hash_table_debug(&mut archive)?;
    }

    if options.contains(&"--block-table") || options.contains(&"--all") {
        show_block_table_debug(&mut archive)?;
    }

    if options.contains(&"--compression") || options.contains(&"--all") {
        analyze_compression(&mut archive)?;
    }

    // Handle specific file operations
    for (i, option) in options.iter().enumerate() {
        match *option {
            "--hex" => {
                if i + 1 < options.len() {
                    hex_dump_file(&mut archive, options[i + 1])?;
                }
            }
            "--trace" => {
                if i + 1 < options.len() {
                    trace_file_extraction(&mut archive, options[i + 1])?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn show_basic_info(info: &ArchiveInfo) {
    println!("📊 Archive Overview");
    println!("-------------------");
    println!("Format Version: {:?}", info.format_version);
    println!(
        "File Count: {}/{} (max capacity)",
        info.file_count, info.max_file_count
    );
    println!("Sector Size: {}", format_size(info.sector_size as u64));
    println!("Archive Offset: 0x{:X}", info.archive_offset);
    println!("Total Size: {}", format_size(info.file_size));
    println!(
        "Encrypted: {}",
        if info.is_encrypted { "Yes" } else { "No" }
    );
    println!(
        "Has Signature: {}",
        if info.has_signature { "Yes" } else { "No" }
    );
    println!();
}

fn show_header_debug(archive: &mut Archive) -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 Header Debug Information");
    println!("---------------------------");

    // This would require exposing the header through Archive API
    // For now, we'll show what we can from ArchiveInfo
    let _info = archive.get_info()?;

    println!("Header analysis would go here...");
    println!("(Note: Full header debug requires access to internal header structure)");
    println!();

    Ok(())
}

fn show_structure_visualization(info: &ArchiveInfo) {
    println!("🏗️  Archive Structure Visualization");
    println!("------------------------------------");
    let viz = visualize_archive_structure(info);
    println!("{}", viz);
}

fn show_hash_table_debug(archive: &mut Archive) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔑 Hash Table Analysis");
    println!("----------------------");

    if let Some(hash_table) = archive.hash_table() {
        let entries = hash_table.entries();
        println!("Total entries: {}", entries.len());

        let mut used_entries = 0;
        let mut deleted_entries = 0;
        let mut free_entries = 0;

        for entry in entries {
            if entry.name_1 == 0xFFFFFFFF && entry.name_2 == 0xFFFFFFFF {
                free_entries += 1;
            } else if entry.name_1 == 0xFFFFFFFE && entry.name_2 == 0xFFFFFFFE {
                deleted_entries += 1;
            } else {
                used_entries += 1;
            }
        }

        println!(
            "Used entries: {} ({:.1}%)",
            used_entries,
            (used_entries as f64 / entries.len() as f64) * 100.0
        );
        println!("Deleted entries: {}", deleted_entries);
        println!("Free entries: {}", free_entries);

        // Show first few valid entries as examples
        println!("\nFirst few entries:");
        let mut shown = 0;
        for (i, entry) in entries.iter().enumerate() {
            if entry.name_1 != 0xFFFFFFFF && entry.name_2 != 0xFFFFFFFF && shown < 5 {
                println!(
                    "  [{}] Hash1: 0x{:08X}, Hash2: 0x{:08X}, Locale: 0x{:04X}, Platform: 0x{:04X}, Block: {}",
                    i, entry.name_1, entry.name_2, entry.locale, entry.platform, entry.block_index
                );
                shown += 1;
            }
        }
    } else {
        println!("No hash table found (archive may use HET/BET tables)");
    }
    println!();

    Ok(())
}

fn show_block_table_debug(archive: &mut Archive) -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Block Table Analysis");
    println!("-----------------------");

    if let Some(block_table) = archive.block_table() {
        let entries = block_table.entries();
        println!("Total entries: {}", entries.len());

        let mut total_uncompressed = 0u64;
        let mut total_compressed = 0u64;
        let mut encrypted_count = 0;
        let mut compressed_count = 0;
        let mut single_unit_count = 0;

        for entry in entries {
            total_uncompressed += entry.file_size as u64;
            total_compressed += entry.compressed_size as u64;

            if entry.flags & 0x00010000 != 0 {
                // Encrypted
                encrypted_count += 1;
            }
            if entry.flags & 0x00000200 != 0 {
                // Compressed
                compressed_count += 1;
            }
            if entry.flags & 0x01000000 != 0 {
                // Single unit
                single_unit_count += 1;
            }
        }

        println!(
            "Total uncompressed size: {}",
            format_size(total_uncompressed)
        );
        println!("Total compressed size: {}", format_size(total_compressed));
        if total_uncompressed > 0 {
            println!(
                "Compression ratio: {:.1}%",
                (total_compressed as f64 / total_uncompressed as f64) * 100.0
            );
        }
        println!("Encrypted files: {}", encrypted_count);
        println!("Compressed files: {}", compressed_count);
        println!("Single unit files: {}", single_unit_count);

        // Show first few entries as examples
        println!("\nFirst few entries:");
        for (i, entry) in entries.iter().enumerate().take(5) {
            println!(
                "  [{}] Offset: 0x{:08X}, Size: {}, Compressed: {}, Flags: 0x{:08X}",
                i, entry.file_pos, entry.file_size, entry.compressed_size, entry.flags
            );
        }
    } else {
        println!("No block table found");
    }
    println!();

    Ok(())
}

fn hex_dump_file(archive: &mut Archive, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔢 Hex Dump: {}", filename);
    println!("--------------------");

    match archive.read_file(filename) {
        Ok(data) => {
            let config = HexDumpConfig {
                bytes_per_line: 16,
                show_ascii: true,
                show_offset: true,
                max_bytes: 256, // Show first 256 bytes
            };

            println!("{}", hex_dump(&data, &config));

            if data.len() > 256 {
                println!("\n... showing first 256 bytes of {} total", data.len());
            }
        }
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
        }
    }

    println!();
    Ok(())
}

fn trace_file_extraction(
    archive: &mut Archive,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Extraction Trace: {}", filename);
    println!("-------------------------");

    // Create a tracer
    let mut tracer = FileExtractionTracer::new(filename);

    // Simulate extraction steps (would need integration with actual extraction)
    tracer.record_step("Looking up file in hash table", None);
    tracer.record_step("Found file entry", Some("Block index: N/A".to_string()));

    match archive.read_file(filename) {
        Ok(data) => {
            tracer.record_step(
                "File read successfully",
                Some(format!("Size: {}", format_size(data.len() as u64))),
            );
            println!("{}", tracer.generate_report());
        }
        Err(e) => {
            tracer.record_step("Extraction failed", Some(e.to_string()));
            println!("{}", tracer.generate_report());
        }
    }

    Ok(())
}

fn analyze_compression(archive: &mut Archive) -> Result<(), Box<dyn std::error::Error>> {
    println!("🗜️  Compression Analysis");
    println!("------------------------");

    let mut analyzer = CompressionAnalyzer::new();
    let files = archive.list()?;
    let sample_size = files.len().min(10);

    println!(
        "Analyzing {} files (sample of {})...\n",
        sample_size,
        files.len()
    );

    // Analyze a sample of files
    for (i, entry) in files.iter().take(sample_size).enumerate() {
        // This is a simplified analysis - real implementation would need access to block data
        analyzer.add_result(
            &entry.name,
            i,
            0x02, // Assume ZLIB for demo
            entry.size,
            entry.compressed_size,
        );
    }

    println!("{}", analyzer.generate_report());

    Ok(())
}
