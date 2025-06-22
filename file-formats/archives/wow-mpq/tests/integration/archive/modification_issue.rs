//! Test the exact modification scenario that causes the error

use std::collections::HashMap;
use wow_mpq::{
    AddFileOptions, Archive, ArchiveBuilder, FormatVersion, ListfileOption, MutableArchive,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Modification Scenario");
    println!("=============================\n");

    // Read the problematic files
    let mut test_files = HashMap::new();

    // File 1: TrollBanner4.blp
    {
        let mut archive =
            Archive::open("/home/danielsreichenbach/Downloads/wow/1.12.1/Data/interface.MPQ")?;
        let data = archive.read_file("Interface\\Glues\\Credits\\TrollBanner4.blp")?;
        test_files.insert(
            "Interface\\Glues\\Credits\\TrollBanner4.blp".to_string(),
            data,
        );
        println!(
            "✓ Read TrollBanner4.blp: {} bytes",
            test_files["Interface\\Glues\\Credits\\TrollBanner4.blp"].len()
        );
    }

    // File 2: Some other file from the same archive
    {
        let mut archive =
            Archive::open("/home/danielsreichenbach/Downloads/wow/1.12.1/Data/fonts.MPQ")?;
        let data = archive.read_file("Fonts\\MORPHEUS.TTF")?;
        test_files.insert("Fonts\\MORPHEUS.TTF".to_string(), data);
        println!(
            "✓ Read MORPHEUS.TTF: {} bytes",
            test_files["Fonts\\MORPHEUS.TTF"].len()
        );
    }

    // Step 1: Create archive with first file (simulating first batch)
    println!("\nStep 1: Creating archive with first file...");
    let test_archive = "test_modification_issue.mpq";
    std::fs::remove_file(test_archive).ok();

    let mut builder = ArchiveBuilder::new()
        .version(FormatVersion::V1)
        .listfile_option(ListfileOption::Generate);

    // Add only MORPHEUS.TTF first
    builder = builder.add_file_data(
        test_files["Fonts\\MORPHEUS.TTF"].clone(),
        "Fonts\\MORPHEUS.TTF",
    );

    builder.build(test_archive)?;
    println!("✓ Created archive with MORPHEUS.TTF");

    // Verify it works
    {
        let mut archive = Archive::open(test_archive)?;
        let data = archive.read_file("Fonts\\MORPHEUS.TTF")?;
        assert_eq!(data.len(), test_files["Fonts\\MORPHEUS.TTF"].len());
        println!("✓ Verified MORPHEUS.TTF in new archive");
    }

    // Step 2: Open for modification and add the problematic file
    println!("\nStep 2: Opening archive for modification...");
    let mut mutable = MutableArchive::open(test_archive)?;

    println!("Adding TrollBanner4.blp via modification...");
    let options = AddFileOptions::default();
    mutable.add_file_data(
        &test_files["Interface\\Glues\\Credits\\TrollBanner4.blp"],
        "Interface\\Glues\\Credits\\TrollBanner4.blp",
        options,
    )?;

    println!("✓ Added file, now flushing changes...");
    mutable.flush()?;
    drop(mutable);
    println!("✓ Modification complete");

    // Step 3: Try to read both files
    println!("\nStep 3: Verifying both files in modified archive...");
    let mut final_archive = Archive::open(test_archive)?;

    // Try to read MORPHEUS.TTF
    match final_archive.read_file("Fonts\\MORPHEUS.TTF") {
        Ok(data) => {
            if data == test_files["Fonts\\MORPHEUS.TTF"] {
                println!("✅ MORPHEUS.TTF: Verified successfully");
            } else {
                println!("❌ MORPHEUS.TTF: Data mismatch");
            }
        }
        Err(e) => {
            println!("❌ MORPHEUS.TTF: Read error - {}", e);
        }
    }

    // Try to read TrollBanner4.blp
    match final_archive.read_file("Interface\\Glues\\Credits\\TrollBanner4.blp") {
        Ok(data) => {
            if data == test_files["Interface\\Glues\\Credits\\TrollBanner4.blp"] {
                println!("✅ TrollBanner4.blp: Verified successfully");
            } else {
                println!("❌ TrollBanner4.blp: Data mismatch");
            }
        }
        Err(e) => {
            println!("❌ TrollBanner4.blp: Read error - {}", e);
        }
    }

    // Test with StormLib
    println!("\nStep 4: Testing with StormLib...");
    test_with_stormlib(test_archive)?;

    // Cleanup
    std::fs::remove_file(test_archive).ok();

    Ok(())
}

fn test_with_stormlib(archive_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    let test_code = format!(
        r#"
#include <StormLib.h>
#include <stdio.h>

int main() {{
    HANDLE hMpq;
    if (!SFileOpenArchive("{}", 0, 0, &hMpq)) {{
        printf("Failed to open archive\n");
        return 1;
    }}

    printf("StormLib opened archive successfully\n");

    const char* files[] = {{
        "Fonts\\MORPHEUS.TTF",
        "Interface\\Glues\\Credits\\TrollBanner4.blp"
    }};

    for (int i = 0; i < 2; i++) {{
        HANDLE hFile;
        if (SFileOpenFileEx(hMpq, files[i], 0, &hFile)) {{
            DWORD size = SFileGetFileSize(hFile, NULL);
            printf("  ✓ %s: %u bytes\n", files[i], size);
            SFileCloseFile(hFile);
        }} else {{
            printf("  ❌ %s: Failed to open\n", files[i]);
        }}
    }}

    SFileCloseArchive(hMpq);
    return 0;
}}
"#,
        archive_path
    );

    std::fs::write("test_mod.c", test_code)?;

    let compile = Command::new("gcc")
        .args([
            "-o",
            "test_mod",
            "test_mod.c",
            "-I/home/danielsreichenbach/Repos/github.com/ladislav-zezula/StormLib/src",
            "-L/home/danielsreichenbach/Repos/github.com/ladislav-zezula/StormLib/build",
            "-lstorm",
            "-lz",
            "-lbz2",
        ])
        .output();

    if let Ok(output) = compile {
        if output.status.success() {
            let result = Command::new("./test_mod")
                .env(
                    "LD_LIBRARY_PATH",
                    "/home/danielsreichenbach/Repos/github.com/ladislav-zezula/StormLib/build",
                )
                .output()?;

            println!("{}", String::from_utf8_lossy(&result.stdout));

            std::fs::remove_file("test_mod").ok();
        }
    }

    std::fs::remove_file("test_mod.c").ok();
    Ok(())
}
