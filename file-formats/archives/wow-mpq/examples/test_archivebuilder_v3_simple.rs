//! Test ArchiveBuilder V3 compatibility with StormLib

use wow_mpq::{ArchiveBuilder, AttributesOption, FormatVersion, ListfileOption};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔨 Testing ArchiveBuilder V3 for StormLib compatibility");
    println!("===================================================");

    let test_archive = "archivebuilder_v3_test.mpq";
    std::fs::remove_file(test_archive).ok();

    // Create a simple V3 archive using only ArchiveBuilder
    println!("📝 Creating V3 archive with ArchiveBuilder only...");

    ArchiveBuilder::new()
        .version(FormatVersion::V3)
        .listfile_option(ListfileOption::Generate)
        .attributes_option(AttributesOption::GenerateFull)
        .add_file_data(b"Test data 1".to_vec(), "test1.txt")
        .add_file_data(b"Test data 2".to_vec(), "test2.txt")
        .build(test_archive)?;

    println!("✅ ArchiveBuilder V3 archive created: {}", test_archive);

    // Verify it works with wow-mpq
    println!("\n🔍 Verifying with wow-mpq...");
    let mut archive = wow_mpq::Archive::open(test_archive)?;
    let files = archive.list()?;

    for file in files {
        println!("  📄 {} - {} bytes", file.name, file.size);

        // Try to read the file
        if let Ok(data) = archive.read_file(&file.name) {
            println!("    ✅ Read {} bytes successfully", data.len());
        } else {
            println!("    ❌ Failed to read file");
        }
    }

    println!("\n💾 File saved as {} for StormLib testing", test_archive);

    Ok(())
}
