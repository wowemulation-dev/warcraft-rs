//! Debug HET/BET table creation

use std::fs;
use wow_mpq::{ArchiveBuilder, AttributesOption, FormatVersion, ListfileOption};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Debug HET/BET Table Creation");
    println!("==============================");

    let test_archive = "debug_het_bet.mpq";

    // Clean up any existing archive
    fs::remove_file(test_archive).ok();

    // Create a simple V3 archive with just a few test files
    println!("📝 Creating V3 archive with test files...");

    ArchiveBuilder::new()
        .version(FormatVersion::V3)
        .listfile_option(ListfileOption::Generate)
        .attributes_option(AttributesOption::GenerateFull)
        .add_file_data(b"Test data 1".to_vec(), "test1.txt")
        .add_file_data(b"Test data 2 - longer content".to_vec(), "test2.txt")
        .add_file_data(b"Test data 3".to_vec(), "subdir/test3.txt")
        .build(test_archive)?;

    // Analyze the created archive
    println!("\n📊 Analyzing created archive...");
    let mut archive = wow_mpq::Archive::open(test_archive)?;
    let info = archive.get_info()?;

    println!("Archive format: {:?}", info.format_version);
    println!("File count: {}", info.file_count);
    println!("Max file count: {}", info.max_file_count);
    println!("Sector size: {} bytes", info.sector_size);
    println!(
        "Hash table: {} bytes",
        info.hash_table_info.size.unwrap_or(0)
    );
    println!(
        "Block table: {} bytes",
        info.block_table_info.size.unwrap_or(0)
    );

    if let Some(het_info) = &info.het_table_info {
        println!(
            "HET table: {} bytes at offset {}",
            het_info.size.unwrap_or(0),
            het_info.offset
        );
    } else {
        println!("HET table: None");
    }

    if let Some(bet_info) = &info.bet_table_info {
        println!(
            "BET table: {} bytes at offset {}",
            bet_info.size.unwrap_or(0),
            bet_info.offset
        );
    } else {
        println!("BET table: None");
    }

    // List files to verify
    println!("\n📁 Files in archive:");
    match archive.list() {
        Ok(files) => {
            for file in files {
                println!("  {} - {} bytes", file.name, file.size);
            }
        }
        Err(e) => println!("  Error listing files: {}", e),
    }

    // Check actual file size
    let file_size = fs::metadata(test_archive)?.len();
    println!("\n💾 Archive file size: {} bytes", file_size);

    // Clean up
    fs::remove_file(test_archive).ok();

    Ok(())
}
