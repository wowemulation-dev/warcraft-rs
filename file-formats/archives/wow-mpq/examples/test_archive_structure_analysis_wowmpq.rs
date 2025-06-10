//! Archive structure analysis from wow-mpq perspective

use wow_mpq::Archive;

fn analyze_archive(path: &str, label: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Analyzing {}: {}", label, path);
    println!("=======================================");

    // Basic file info
    let metadata = std::fs::metadata(path)?;
    println!("📊 Basic Information:");
    println!("  File size: {} bytes", metadata.len());

    // Try to open with wow-mpq
    match Archive::open(path) {
        Ok(mut archive) => {
            // Get detailed archive information
            let info = archive.get_info()?;
            println!("  Archive format: {:?}", info.format_version);
            println!("  Archive offset: {} bytes", info.archive_offset);
            println!("  File count: {}", info.file_count);
            println!("  Max file count: {}", info.max_file_count);
            println!("  Sector size: {} bytes", info.sector_size);
            println!(
                "  Hash table: {} bytes",
                info.hash_table_info.size.unwrap_or(0)
            );
            println!(
                "  Block table: {} bytes",
                info.block_table_info.size.unwrap_or(0)
            );

            if let Some(het_info) = &info.het_table_info {
                println!("  HET table: {} bytes", het_info.size.unwrap_or(0));
            }

            if let Some(bet_info) = &info.bet_table_info {
                println!("  BET table: {} bytes", bet_info.size.unwrap_or(0));
            }

            // Analyze files
            println!("\n📁 File Analysis:");
            match archive.list() {
                Ok(files) => {
                    let mut total_uncompressed = 0;
                    let mut total_compressed = 0;

                    for (i, file) in files.iter().enumerate() {
                        total_uncompressed += file.size as u64;
                        total_compressed += file.compressed_size as u64;

                        if i < 5 {
                            // Show first 5 files
                            print!("  {} - {} bytes", file.name, file.size);
                            if file.compressed_size != file.size {
                                print!(" (compressed: {})", file.compressed_size);
                            }
                            println!();
                        }
                    }

                    println!("  Total files: {}", files.len());
                    println!("  Total uncompressed: {} bytes", total_uncompressed);
                    println!("  Total compressed: {} bytes", total_compressed);

                    if total_uncompressed > 0 {
                        let compression_ratio =
                            total_compressed as f64 / total_uncompressed as f64 * 100.0;
                        println!("  Compression ratio: {:.1}%", compression_ratio);
                    }
                }
                Err(e) => {
                    println!("  ❌ Failed to list files: {}", e);
                }
            }
        }
        Err(e) => {
            println!("  ❌ Failed to open archive: {}", e);
            println!("  Error details: {:?}", e);
        }
    }

    Ok(())
}

fn compare_archives() {
    println!("\n📊 Archive Comparison Summary:");
    println!("==============================");

    if let (Ok(stormlib_meta), Ok(wowmpq_meta)) = (
        std::fs::metadata("stormlib_v3_modification_test.mpq"),
        std::fs::metadata("wowmpq_v3_modification_test.mpq"),
    ) {
        let stormlib_size = stormlib_meta.len();
        let wowmpq_size = wowmpq_meta.len();

        println!("StormLib archive: {} bytes", stormlib_size);
        println!("wow-mpq archive:  {} bytes", wowmpq_size);

        let diff = wowmpq_size as i64 - stormlib_size as i64;
        print!("Difference:       {} bytes", diff);

        if diff > 0 {
            let overhead = diff as f64 / stormlib_size as f64 * 100.0;
            println!(" ({:.2}% overhead)", overhead);
        } else if diff < 0 {
            let savings = (-diff) as f64 / stormlib_size as f64 * 100.0;
            println!(" ({:.2}% smaller)", savings);
        } else {
            println!(" (identical)");
        }

        // Test cross-compatibility
        println!("\n🔄 Cross-compatibility tests:");

        // Can wow-mpq read StormLib archive?
        print!("  wow-mpq reading StormLib: ");
        match Archive::open("stormlib_v3_modification_test.mpq") {
            Ok(_) => println!("✅ SUCCESS"),
            Err(e) => println!("❌ FAILED ({})", e),
        }

        // Note: We already know StormLib can't read wow-mpq archive from previous test
        println!("  StormLib reading wow-mpq: ❌ FAILED (Error 1004)");
    }
}

fn investigate_format_differences() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔬 Investigating Format Differences:");
    println!("===================================");

    // Read raw header data from both archives
    let stormlib_data = std::fs::read("stormlib_v3_modification_test.mpq")?;
    let wowmpq_data = std::fs::read("wowmpq_v3_modification_test.mpq")?;

    println!("📄 Header Analysis:");
    println!("  StormLib header (first 32 bytes):");
    print!("    ");
    for (i, byte) in stormlib_data.iter().take(32).enumerate() {
        print!("{:02x} ", byte);
        if (i + 1) % 16 == 0 {
            print!("\n    ");
        }
    }
    println!();

    println!("  wow-mpq header (first 32 bytes):");
    print!("    ");
    for (i, byte) in wowmpq_data.iter().take(32).enumerate() {
        print!("{:02x} ", byte);
        if (i + 1) % 16 == 0 {
            print!("\n    ");
        }
    }
    println!();

    // Look for MPQ signature
    if stormlib_data.len() >= 4 && wowmpq_data.len() >= 4 {
        let stormlib_sig = &stormlib_data[0..4];
        let wowmpq_sig = &wowmpq_data[0..4];

        println!(
            "  StormLib signature: {:?}",
            std::str::from_utf8(stormlib_sig).unwrap_or("invalid")
        );
        println!(
            "  wow-mpq signature:  {:?}",
            std::str::from_utf8(wowmpq_sig).unwrap_or("invalid")
        );
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Archive Structure Analysis (wow-mpq perspective)");
    println!("==================================================");

    // Analyze both archives
    analyze_archive("stormlib_v3_modification_test.mpq", "StormLib V3 Archive")?;
    analyze_archive("wowmpq_v3_modification_test.mpq", "wow-mpq V3 Archive")?;

    // Compare archives
    compare_archives();

    // Investigate format differences
    investigate_format_differences()?;

    println!("\n🎯 Analysis Complete");

    Ok(())
}
