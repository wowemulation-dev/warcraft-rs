//! Test hybrid approach: Start with ArchiveBuilder V3, then modify with MutableArchive

use wow_mpq::{
    AddFileOptions, ArchiveBuilder, AttributesOption, FormatVersion, ListfileOption, MutableArchive,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Testing Hybrid V3 Approach");
    println!("=============================");

    let test_archive = "hybrid_v3_test.mpq";
    std::fs::remove_file(test_archive).ok();

    // Step 1: Create initial V3 archive using ArchiveBuilder (this works with StormLib)
    println!("📝 Step 1: Creating initial V3 archive with ArchiveBuilder...");

    ArchiveBuilder::new()
        .version(FormatVersion::V3)
        .listfile_option(ListfileOption::Generate)
        .attributes_option(AttributesOption::GenerateFull)
        .add_file_data(b"Initial test data".to_vec(), "initial.txt")
        .build(test_archive)?;

    println!("  ✅ ArchiveBuilder V3 archive created");

    // Step 2: Modify the archive using MutableArchive
    println!("\n➕ Step 2: Adding files using MutableArchive...");

    let mut mutable = MutableArchive::open(test_archive)?;

    // Add test files
    let options = AddFileOptions::default();
    mutable.add_file_data(
        b"Added with MutableArchive 1".as_ref(),
        "added1.txt",
        options,
    )?;
    mutable.add_file_data(
        b"Added with MutableArchive 2".as_ref(),
        "added2.txt",
        AddFileOptions::default(),
    )?;

    // Flush the changes
    mutable.flush()?;
    drop(mutable); // Ensure it's fully written

    println!("  ✅ Files added with MutableArchive");

    // Step 3: Verify all files can be read
    println!("\n🔍 Step 3: Verifying all files...");

    let mut archive = wow_mpq::Archive::open(test_archive)?;
    let files = archive.list()?;

    println!("  📁 Files in archive:");
    for file in files {
        println!("    📄 {} - {} bytes", file.name, file.size);

        // Try to read each file
        match archive.read_file(&file.name) {
            Ok(data) => {
                println!("      ✅ Read {} bytes successfully", data.len());

                // Show content for text files
                if file.name.ends_with(".txt") && data.len() < 100 {
                    if let Ok(content) = std::str::from_utf8(&data) {
                        println!("      Content: \"{}\"", content);
                    }
                }
            }
            Err(e) => {
                println!("      ❌ Failed to read: {}", e);
            }
        }
    }

    println!(
        "\n💾 Hybrid archive saved as {} for StormLib testing",
        test_archive
    );

    Ok(())
}
