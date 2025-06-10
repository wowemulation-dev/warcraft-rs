//! Comprehensive StormLib compatibility test

use std::process::Command;
use tempfile::TempDir;
use wow_mpq::{
    AddFileOptions, Archive, ArchiveBuilder, AttributesOption, FormatVersion, ListfileOption,
    MutableArchive, compression::CompressionMethod,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let archive_path = temp_dir.path().join("stormlib_test.mpq");

    println!("=== Creating initial archive ===");
    ArchiveBuilder::new()
        .version(FormatVersion::V1)
        .listfile_option(ListfileOption::Generate)
        .attributes_option(AttributesOption::GenerateCrc32)
        .add_file_data(b"File 1 content".to_vec(), "data\\file1.txt")
        .add_file_data(b"File 2 content".to_vec(), "data\\file2.txt")
        .add_file_data(b"Readme content".to_vec(), "readme.txt")
        .build(&archive_path)?;

    // List initial files
    {
        let mut archive = Archive::open(&archive_path)?;
        println!("\nInitial files:");
        for file in archive.list()? {
            println!("  - {}", file.name);
        }
    }

    println!("\n=== Modifying archive ===");
    {
        let mut mutable = MutableArchive::open(&archive_path)?;

        // Add files
        println!("Adding data\\file3.txt...");
        mutable.add_file_data(
            b"File 3 - added by wow-mpq".as_ref(),
            "data\\file3.txt",
            AddFileOptions::default(),
        )?;

        println!("Adding data\\compressed.bin with zlib...");
        mutable.add_file_data(
            b"This file is compressed with zlib".as_ref(),
            "data\\compressed.bin",
            AddFileOptions::new().compression(CompressionMethod::Zlib),
        )?;

        println!("Adding encrypted.dat with encryption...");
        mutable.add_file_data(
            b"This file is encrypted".as_ref(),
            "encrypted.dat",
            AddFileOptions::new().encrypt(),
        )?;

        // Remove file
        println!("Removing readme.txt...");
        mutable.remove_file("readme.txt")?;

        // Rename file
        println!("Renaming data\\file1.txt to data\\renamed.txt...");
        mutable.rename_file("data\\file1.txt", "data\\renamed.txt")?;

        // Replace file
        println!("Replacing data\\file2.txt...");
        mutable.add_file_data(
            b"File 2 - replaced content".as_ref(),
            "data\\file2.txt",
            AddFileOptions::default(),
        )?;

        mutable.flush()?;
    }

    // Verify with wow-mpq
    println!("\n=== Verifying with wow-mpq ===");
    {
        let mut archive = Archive::open(&archive_path)?;
        println!("Final files:");
        for file in archive.list()? {
            println!("  - {} ({} bytes)", file.name, file.size);
        }

        // Check listfile
        let listfile = archive.read_file("(listfile)")?;
        let listfile_str = String::from_utf8_lossy(&listfile);
        println!("\nListfile contents:");
        for line in listfile_str.lines() {
            println!("  {}", line);
        }

        // Check attributes
        if let Ok(attrs) = archive.read_file("(attributes)") {
            println!("\nAttributes file present: {} bytes", attrs.len());
        }
    }

    // Run StormLib test
    println!("\n=== Running StormLib test ===");
    let stormlib_exe = "test_modification_stormlib_compat";

    if std::path::Path::new(stormlib_exe).exists() {
        let output = Command::new(format!("./{}", stormlib_exe))
            .arg(&archive_path)
            .output()?;

        println!("{}", String::from_utf8_lossy(&output.stdout));
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }

        if output.status.success() {
            println!("\n✅ StormLib compatibility test PASSED!");
        } else {
            eprintln!("\n❌ StormLib compatibility test FAILED!");
            return Err("StormLib test failed".into());
        }
    } else {
        println!("StormLib test binary not found in current directory");
        println!("Expected: ./{}", stormlib_exe);
        println!("Current dir: {}", std::env::current_dir()?.display());
    }

    Ok(())
}
