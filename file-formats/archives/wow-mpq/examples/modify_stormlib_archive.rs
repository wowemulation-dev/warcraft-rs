//! Modify a StormLib-created archive with wow-mpq

use wow_mpq::{AddFileOptions, Archive, MutableArchive};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let archive_path = std::env::args()
        .nth(1)
        .expect("Usage: modify_stormlib_archive <archive.mpq>");

    println!("Opening StormLib archive: {}", archive_path);

    // First, list original contents
    {
        let mut archive = Archive::open(&archive_path)?;
        println!("\nOriginal contents:");
        for file in archive.list()? {
            println!("  - {} ({} bytes)", file.name, file.size);
        }
    }

    // Modify the archive
    {
        let mut mutable = MutableArchive::open(&archive_path)?;

        println!("\nModifying archive...");

        // Add a new file
        println!("  Adding wow_mpq_added.txt");
        mutable.add_file_data(
            b"This file was added by wow-mpq to a StormLib archive".as_ref(),
            "wow_mpq_added.txt",
            AddFileOptions::default(),
        )?;

        // Add another with compression
        println!("  Adding compressed_by_wowmpq.dat");
        mutable.add_file_data(
            b"Compressed file added by wow-mpq".repeat(10).as_ref(),
            "compressed_by_wowmpq.dat",
            AddFileOptions::new().compression(wow_mpq::compression::CompressionMethod::Zlib),
        )?;

        mutable.flush()?;
        println!("Modifications saved!");
    }

    // Verify the changes
    {
        let mut archive = Archive::open(&archive_path)?;
        println!("\nModified contents:");
        for file in archive.list()? {
            println!("  - {} ({} bytes)", file.name, file.size);
        }

        // Check listfile
        if let Ok(listfile_data) = archive.read_file("(listfile)") {
            let listfile = String::from_utf8_lossy(&listfile_data);
            println!(
                "\nListfile updated: {}",
                if listfile.contains("wow_mpq_added.txt") {
                    "✓"
                } else {
                    "✗"
                }
            );
        }
    }

    println!("\nArchive successfully modified by wow-mpq!");
    println!(
        "Test with: ./test_modification_stormlib_compat {}",
        archive_path
    );

    Ok(())
}
