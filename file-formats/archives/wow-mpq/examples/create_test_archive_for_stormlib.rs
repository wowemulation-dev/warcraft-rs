//! Create a test archive and save path for StormLib testing

use wow_mpq::{
    AddFileOptions, Archive, ArchiveBuilder, AttributesOption, FormatVersion, ListfileOption,
    MutableArchive,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let archive_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "test_archive.mpq".to_string());

    println!("Creating test archive at: {}", archive_path);

    // Create initial archive
    ArchiveBuilder::new()
        .version(FormatVersion::V1)
        .listfile_option(ListfileOption::Generate)
        .attributes_option(AttributesOption::GenerateCrc32)
        .add_file_data(b"Initial file 1 content".to_vec(), "file1.txt")
        .add_file_data(b"Initial file 2 content".to_vec(), "file2.txt")
        .build(&archive_path)?;

    // Modify it
    {
        let mut mutable = MutableArchive::open(&archive_path)?;

        mutable.add_file_data(
            b"New file added via modification".as_ref(),
            "added_file.txt",
            AddFileOptions::default(),
        )?;

        mutable.rename_file("file1.txt", "renamed_file1.txt")?;

        mutable.flush()?;
    }

    // Verify
    let mut archive = Archive::open(&archive_path)?;
    println!("\nArchive contents:");
    for file in archive.list()? {
        println!("  - {} ({} bytes)", file.name, file.size);
    }

    println!("\nArchive ready for StormLib testing: {}", archive_path);

    Ok(())
}
