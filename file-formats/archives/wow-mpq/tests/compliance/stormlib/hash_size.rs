//! Test with StormLib-compatible hash table size

use std::fs;
use wow_mpq::{ArchiveBuilder, AttributesOption, FormatVersion, ListfileOption};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Testing StormLib-compatible hash table size");
    println!("=============================================");

    let test_archive = "hash_size_test.mpq";
    fs::remove_file(test_archive).ok();

    // Create archive with custom builder
    let mut builder = ArchiveBuilder::new();

    // Add the same test file as StormLib test
    let test_data = b"This is test data for file 1";
    let test_data2 = b"This is test data for file 2 - a bit longer to test different sizes";

    // Extract the problematic file (simplified for this test)
    println!("📝 Creating V3 archive with exact StormLib test setup...");

    builder = builder
        .version(FormatVersion::V3)
        .listfile_option(ListfileOption::Generate)
        .attributes_option(AttributesOption::GenerateFull)
        .add_file_data(test_data.to_vec(), "test_file_1.txt")
        .add_file_data(test_data2.to_vec(), "test_file_2.txt")
        .add_file_data(
            b"fake adt data".to_vec(),
            "World\\Maps\\Azeroth\\Azeroth_28_51_tex1.adt",
        );

    builder.build(test_archive)?;

    // Analyze the result
    println!("\n📊 Archive Analysis:");
    let mut archive = wow_mpq::Archive::open(test_archive)?;
    let info = archive.get_info()?;

    println!("File count: {}", info.file_count);
    println!("Max file count: {}", info.max_file_count);
    println!(
        "Hash table: {} bytes",
        info.hash_table_info.size.unwrap_or(0)
    );
    println!(
        "Block table: {} bytes",
        info.block_table_info.size.unwrap_or(0)
    );

    if let Some(het_info) = &info.het_table_info {
        println!("HET table: {} bytes", het_info.size.unwrap_or(0));
    }

    if let Some(bet_info) = &info.bet_table_info {
        println!("BET table: {} bytes", bet_info.size.unwrap_or(0));
    }

    // Calculate expected hash table size
    // StormLib uses 32 bytes = 2 hash entries (16 bytes each)
    // But wait, that doesn't make sense for 5 files...

    println!("\n📁 Files in archive:");
    let files = archive.list()?;
    for file in files {
        println!("  {} - {} bytes", file.name, file.size);
    }

    println!(
        "\n💾 File size: {} bytes",
        fs::metadata(test_archive)?.len()
    );

    // Clean up
    fs::remove_file(test_archive).ok();

    Ok(())
}
