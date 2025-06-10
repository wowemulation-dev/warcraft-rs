//! Test the round-trip process: extract file, create archive, read back

use std::fs;
use wow_mpq::{
    Archive, ArchiveBuilder, AttributesOption, FormatVersion, ListfileOption,
    compression::CompressionMethod,
};

const PROBLEMATIC_FILE: &str = "World\\Maps\\Azeroth\\Azeroth_28_51_tex1.adt";
const CATA_ARCHIVE: &str = "/home/danielsreichenbach/Downloads/wow/4.3.4/4.3.4/Data/world.MPQ";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 wow-mpq Huffman Round-trip Test");
    println!("=================================");

    // Step 1: Extract from original archive
    println!("\n📥 Step 1: Extracting from original Cataclysm archive...");
    let mut original_archive = Archive::open(CATA_ARCHIVE)?;
    let original_data = original_archive.read_file(PROBLEMATIC_FILE)?;
    println!("  ✅ Extracted {} bytes from original", original_data.len());

    // Step 2: Create V3 archive with this file (no compression first)
    println!("\n🔨 Step 2: Creating V3 archive with no compression...");
    let test_archive_uncompressed = "huffman_test_uncompressed.mpq";

    ArchiveBuilder::new()
        .version(FormatVersion::V3)
        .listfile_option(ListfileOption::Generate)
        .attributes_option(AttributesOption::GenerateFull)
        .add_file_data(original_data.clone(), PROBLEMATIC_FILE)
        .build(test_archive_uncompressed)?;

    println!("  ✅ Created uncompressed V3 archive");

    // Step 3: Read back from uncompressed archive
    println!("\n📤 Step 3: Reading from uncompressed V3 archive...");
    let mut test_archive_uncomp = Archive::open(test_archive_uncompressed)?;
    match test_archive_uncomp.read_file(PROBLEMATIC_FILE) {
        Ok(data) => {
            println!(
                "  ✅ Successfully read {} bytes from uncompressed archive",
                data.len()
            );
            if data == original_data {
                println!("  ✅ Data matches original perfectly");
            } else {
                println!("  ❌ Data differs from original");
            }
        }
        Err(e) => {
            println!("  ❌ Failed to read from uncompressed archive: {}", e);
        }
    }

    // Step 4: Create V3 archive with default compression
    println!("\n🔨 Step 4: Creating V3 archive with default compression...");
    let test_archive_compressed = "huffman_test_compressed.mpq";

    ArchiveBuilder::new()
        .version(FormatVersion::V3)
        .listfile_option(ListfileOption::Generate)
        .attributes_option(AttributesOption::GenerateFull)
        .add_file_data_with_options(
            original_data.clone(),
            PROBLEMATIC_FILE,
            0x02,  // Zlib compression
            false, // Not encrypted
            0,     // Default locale
        )
        .build(test_archive_compressed)?;

    println!("  ✅ Created compressed V3 archive");

    // Step 5: Read back from compressed archive
    println!("\n📤 Step 5: Reading from compressed V3 archive...");
    let mut test_archive_comp = Archive::open(test_archive_compressed)?;
    match test_archive_comp.read_file(PROBLEMATIC_FILE) {
        Ok(data) => {
            println!(
                "  ✅ Successfully read {} bytes from compressed archive",
                data.len()
            );
            if data == original_data {
                println!("  ✅ Data matches original perfectly");
            } else {
                println!("  ❌ Data differs from original");
            }
        }
        Err(e) => {
            println!("  ❌ Failed to read from compressed archive: {}", e);
            println!("  Error details: {:?}", e);
        }
    }

    // Step 6: Try with no compression
    println!("\n🔨 Step 6: Creating V3 archive with no compression...");
    let test_archive_zlib = "huffman_test_none.mpq";

    ArchiveBuilder::new()
        .version(FormatVersion::V3)
        .listfile_option(ListfileOption::Generate)
        .attributes_option(AttributesOption::GenerateFull)
        .add_file_data_with_options(
            original_data.clone(),
            PROBLEMATIC_FILE,
            0x00,  // No compression
            false, // Not encrypted
            0,     // Default locale
        )
        .build(test_archive_zlib)?;

    println!("  ✅ Created uncompressed V3 archive");

    // Step 7: Read back from second uncompressed archive
    println!("\n📤 Step 7: Reading from second uncompressed V3 archive...");
    let mut test_archive_zlib_arch = Archive::open(test_archive_zlib)?;
    match test_archive_zlib_arch.read_file(PROBLEMATIC_FILE) {
        Ok(data) => {
            println!(
                "  ✅ Successfully read {} bytes from second archive",
                data.len()
            );
            if data == original_data {
                println!("  ✅ Data matches original perfectly");
            } else {
                println!("  ❌ Data differs from original");
            }
        }
        Err(e) => {
            println!("  ❌ Failed to read from second archive: {}", e);
            println!("  Error details: {:?}", e);
        }
    }

    println!("\n📊 Round-trip Test Summary:");
    println!("===========================");
    println!("Original file size: {} bytes", original_data.len());
    println!("Test files created:");
    println!("  - {} (uncompressed)", test_archive_uncompressed);
    println!("  - {} (default compression)", test_archive_compressed);
    println!("  - {} (no compression)", test_archive_zlib);

    // Cleanup
    fs::remove_file(test_archive_uncompressed).ok();
    fs::remove_file(test_archive_compressed).ok();
    fs::remove_file(test_archive_zlib).ok();

    Ok(())
}
