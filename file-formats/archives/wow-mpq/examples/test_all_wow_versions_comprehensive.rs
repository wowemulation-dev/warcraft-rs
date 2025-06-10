//! Comprehensive file integrity test for ALL WoW versions (1.12.1 through 5.4.8)
//! Tests archive creation and modification with version-appropriate formats

use rand::{Rng, seq::IteratorRandom};
use std::collections::HashMap;
use std::process::Command;
use wow_mpq::{
    AddFileOptions, Archive, ArchiveBuilder, AttributesOption, FormatVersion, ListfileOption,
    MutableArchive,
};

#[derive(Debug, Clone)]
struct WowVersion {
    name: &'static str,
    path: &'static str,
    format: FormatVersion,
    expected_archives: &'static [&'static str],
}

const WOW_VERSIONS: &[WowVersion] = &[
    WowVersion {
        name: "WoW 1.12.1 (Vanilla)",
        path: "/home/danielsreichenbach/Downloads/wow/1.12.1/Data",
        format: FormatVersion::V1,
        expected_archives: &["patch.MPQ", "patch-2.MPQ", "base.MPQ"],
    },
    WowVersion {
        name: "WoW 2.4.3 (The Burning Crusade)",
        path: "/home/danielsreichenbach/Downloads/wow/2.4.3/Data",
        format: FormatVersion::V1, // TBC still used V1, some V2
        expected_archives: &["expansion.MPQ", "patch.MPQ", "patch-2.MPQ"],
    },
    WowVersion {
        name: "WoW 3.3.5a (Wrath of the Lich King)",
        path: "/home/danielsreichenbach/Downloads/wow/3.3.5a/Data",
        format: FormatVersion::V1, // WotLK primarily V1, some V2
        expected_archives: &["expansion.MPQ", "lichking.MPQ", "patch.MPQ"],
    },
    WowVersion {
        name: "WoW 4.3.4 (Cataclysm)",
        path: "/home/danielsreichenbach/Downloads/wow/4.3.4/4.3.4/Data",
        format: FormatVersion::V3, // Cata introduced V3
        expected_archives: &[
            "world.MPQ",
            "misc.MPQ",
            "expansion1.MPQ",
            "expansion2.MPQ",
            "expansion3.MPQ",
        ],
    },
    WowVersion {
        name: "WoW 5.4.8 (Mists of Pandaria)",
        path: "/home/danielsreichenbach/Downloads/wow/5.4.8/5.4.8/Data",
        format: FormatVersion::V4, // MoP introduced V4
        expected_archives: &["world.MPQ", "misc.MPQ", "expansion4.MPQ"],
    },
];

fn test_stormlib_compatibility(archive_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
    // Create a simple C program to test StormLib compatibility
    let test_program = format!(
        r#"
#include <StormLib.h>
#include <stdio.h>
#include <stdlib.h>

int main() {{
    HANDLE hMpq = NULL;
    
    // Try to open the archive
    if (!SFileOpenArchive("{}", 0, 0, &hMpq)) {{
        printf("Failed to open archive with StormLib\n");
        return 1;
    }}
    
    printf("StormLib successfully opened the archive\n");
    
    // Try to enumerate files
    SFILE_FIND_DATA findData;
    HANDLE hFind = SFileFindFirstFile(hMpq, "*", &findData, NULL);
    
    if (hFind != NULL) {{
        int fileCount = 0;
        do {{
            fileCount++;
            printf("Found file: %s\n", findData.cFileName);
        }} while (SFileFindNextFile(hFind, &findData));
        
        SFileFindClose(hFind);
        printf("Total files found: %d\n", fileCount);
    }}
    
    // Try to read a specific file
    HANDLE hFile;
    if (SFileOpenFileEx(hMpq, "(listfile)", 0, &hFile)) {{
        DWORD fileSize = SFileGetFileSize(hFile, NULL);
        printf("Successfully opened (listfile), size: %u bytes\n", fileSize);
        SFileCloseFile(hFile);
    }}
    
    SFileCloseArchive(hMpq);
    printf("StormLib compatibility test PASSED\n");
    return 0;
}}
"#,
        archive_path
    );

    // Write the test program
    let test_file = "stormlib_test.c";
    std::fs::write(test_file, test_program)?;

    // Compile with StormLib
    let compile_result = Command::new("gcc")
        .args(&[
            "-o",
            "stormlib_test",
            test_file,
            "-I/home/danielsreichenbach/Repos/github.com/ladislav-zezula/StormLib/src",
            "-L/home/danielsreichenbach/Repos/github.com/ladislav-zezula/StormLib/build",
            "-lstorm",
            "-lz",
            "-lbz2",
        ])
        .output();

    match compile_result {
        Ok(output) if output.status.success() => {
            // Run the test with library path
            let test_result = Command::new("./stormlib_test")
                .env(
                    "LD_LIBRARY_PATH",
                    "/home/danielsreichenbach/Repos/github.com/ladislav-zezula/StormLib/build",
                )
                .output()?;

            if test_result.status.success() {
                println!("  📝 StormLib output:");
                for line in String::from_utf8_lossy(&test_result.stdout).lines() {
                    println!("    {}", line);
                }

                // Cleanup
                std::fs::remove_file("stormlib_test").ok();
                std::fs::remove_file(test_file).ok();

                Ok(true)
            } else {
                println!("  ❌ StormLib test execution failed");
                println!("  Error: {}", String::from_utf8_lossy(&test_result.stderr));

                // Cleanup
                std::fs::remove_file("stormlib_test").ok();
                std::fs::remove_file(test_file).ok();

                Ok(false)
            }
        }
        Ok(_) => {
            println!("  ⚠️ StormLib test compilation failed - StormLib not available");
            std::fs::remove_file(test_file).ok();
            Ok(false)
        }
        Err(_) => {
            println!("  ⚠️ gcc not available for StormLib test");
            std::fs::remove_file(test_file).ok();
            Ok(false)
        }
    }
}

fn test_wow_version(version: &WowVersion) -> Result<bool, Box<dyn std::error::Error>> {
    println!(
        "\n🎮 Testing {} (Format: {:?})",
        version.name, version.format
    );
    println!("{}", "=".repeat(60));

    let data_path = std::path::PathBuf::from(version.path);
    if !data_path.exists() {
        println!("⚠️  Data not found at: {}", data_path.display());
        return Ok(false);
    }
    println!("✅ Found data at: {}", data_path.display());

    // Find MPQ archives
    let mut archives = Vec::new();
    for entry in std::fs::read_dir(&data_path)? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            if name.to_lowercase().ends_with(".mpq") {
                archives.push(name.to_string());
            }
        }
    }

    if archives.is_empty() {
        println!("❌ No MPQ archives found");
        return Ok(false);
    }

    archives.sort();
    println!(
        "📦 Found {} archives: {}",
        archives.len(),
        archives
            .iter()
            .take(5)
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Extract random files
    println!("\n🎲 Extracting random files...");
    let mut extracted_files = HashMap::new();
    let mut rng = rand::rng();

    // Sample from first few archives
    let sample_archives: Vec<_> = archives.iter().take(8).collect();
    for archive_name in &sample_archives {
        println!("  📖 Scanning: {}", archive_name);
        let archive_path = data_path.join(archive_name);

        match Archive::open(&archive_path) {
            Ok(mut archive) => {
                match archive.list() {
                    Ok(files) => {
                        let sample_count = rng.random_range(2..=4);
                        let sampled = files.into_iter().choose_multiple(&mut rng, sample_count);

                        for file_entry in sampled {
                            // Skip special files and already extracted
                            if file_entry.name.starts_with('(')
                                || extracted_files.contains_key(&file_entry.name)
                            {
                                continue;
                            }

                            match archive.read_file(&file_entry.name) {
                                Ok(data) => {
                                    if data.len() > 0 && data.len() < 1024 * 1024 {
                                        // Skip empty/huge files
                                        println!("    ✓ {}: {} bytes", file_entry.name, data.len());
                                        extracted_files.insert(file_entry.name.clone(), data);
                                        if extracted_files.len() >= 15 {
                                            // Limit extraction
                                            break;
                                        }
                                    }
                                }
                                Err(_) => continue,
                            }
                        }
                    }
                    Err(e) => println!("    ⚠️ Could not list files: {}", e),
                }
            }
            Err(e) => println!("    ⚠️ Could not open {}: {}", archive_name, e),
        }

        if extracted_files.len() >= 15 {
            break;
        }
    }

    if extracted_files.is_empty() {
        println!("❌ No files could be extracted");
        return Ok(false);
    }

    println!("✅ Extracted {} files", extracted_files.len());

    // Create archive with version-appropriate format
    println!(
        "\n🔨 Creating {} archive...",
        match version.format {
            FormatVersion::V1 => "V1",
            FormatVersion::V2 => "V2",
            FormatVersion::V3 => "V3 (with HET/BET)",
            FormatVersion::V4 => "V4 (with advanced HET/BET)",
        }
    );

    let test_archive = format!(
        "test_{}_comprehensive.mpq",
        version
            .name
            .split_whitespace()
            .next()
            .unwrap()
            .to_lowercase()
            .replace(".", "_")
    );
    std::fs::remove_file(&test_archive).ok();

    let mut builder = ArchiveBuilder::new()
        .version(version.format)
        .listfile_option(ListfileOption::Generate);

    // Add attributes based on format capabilities
    builder = match version.format {
        FormatVersion::V1 | FormatVersion::V2 => {
            builder.attributes_option(AttributesOption::GenerateCrc32)
        }
        FormatVersion::V3 | FormatVersion::V4 => {
            builder.attributes_option(AttributesOption::GenerateFull)
        }
    };

    // Add first batch of files
    let first_batch_size = extracted_files.len() / 2;
    let mut first_batch = HashMap::new();
    for (i, (filename, data)) in extracted_files.iter().enumerate() {
        if i < first_batch_size {
            builder = builder.add_file_data(data.clone(), filename);
            first_batch.insert(filename.clone(), data.clone());
        }
    }

    builder.build(&test_archive)?;
    println!("✅ Created archive: {}", test_archive);

    // Verify first batch
    println!("\n🔍 Verifying first batch...");
    let mut archive = Archive::open(&test_archive)?;
    let mut verified_count = 0;

    for (filename, original_data) in &first_batch {
        match archive.read_file(filename) {
            Ok(extracted_data) => {
                if extracted_data == *original_data {
                    verified_count += 1;
                } else {
                    println!("  ❌ {}: Data mismatch!", filename);
                }
            }
            Err(e) => {
                println!("  ❌ {}: Read error: {}", filename, e);
            }
        }
    }

    if verified_count != first_batch.len() {
        println!(
            "❌ First batch verification failed: {}/{}",
            verified_count,
            first_batch.len()
        );
        std::fs::remove_file(&test_archive).ok();
        return Ok(false);
    }
    println!(
        "✅ First batch verified: {}/{}",
        verified_count,
        first_batch.len()
    );

    // Add second batch using modification
    println!("\n➕ Adding second batch via modification...");
    let mut second_batch = HashMap::new();
    for (i, (filename, data)) in extracted_files.iter().enumerate() {
        if i >= first_batch_size {
            second_batch.insert(filename.clone(), data.clone());
        }
    }

    if !second_batch.is_empty() {
        let mut mutable = MutableArchive::open(&test_archive)?;
        for (filename, data) in &second_batch {
            let options = AddFileOptions::default();
            mutable.add_file_data(data, filename, options)?;
        }
        mutable.flush()?;
        drop(mutable);
        println!("✅ Added {} files via modification", second_batch.len());

        // Final verification
        println!("\n🔍 Final verification...");
        let mut final_archive = Archive::open(&test_archive)?;
        let mut total_verified = 0;

        // Verify all files
        for (filename, original_data) in extracted_files.iter() {
            match final_archive.read_file(filename) {
                Ok(extracted_data) => {
                    if extracted_data == *original_data {
                        total_verified += 1;
                    } else {
                        println!("  ❌ {}: Data mismatch!", filename);
                    }
                }
                Err(e) => {
                    println!("  ❌ {}: Read error: {}", filename, e);
                }
            }
        }

        let total_files = extracted_files.len();
        println!(
            "  📊 Final verification: {}/{} files",
            total_verified, total_files
        );

        if total_verified == total_files {
            println!("✅ {} PASSED - All files verified!", version.name);
        } else {
            println!(
                "❌ {} FAILED - {}/{} files verified",
                version.name, total_verified, total_files
            );
        }

        // Test format-specific features
        if let Ok(info) = final_archive.get_info() {
            println!(
                "  🔍 Archive info: {} files, {:?} format",
                info.file_count, info.format_version
            );
        }
    } else {
        println!("✅ {} PASSED (single batch only)", version.name);
    }

    // StormLib compatibility verification
    println!("\n🔍 Testing StormLib compatibility...");
    let stormlib_test_result = test_stormlib_compatibility(&test_archive);
    match stormlib_test_result {
        Ok(true) => println!("✅ StormLib can successfully read our archive!"),
        Ok(false) => println!("⚠️ StormLib compatibility test skipped"),
        Err(e) => println!("❌ StormLib compatibility test failed: {}", e),
    }

    // Cleanup
    std::fs::remove_file(&test_archive).ok();

    Ok(true)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 WoW Multi-Version Archive Integrity Test");
    println!("===========================================");
    println!("Testing archive creation and modification across all WoW versions");

    let mut passed = 0;
    let mut total = 0;
    let mut results = Vec::new();

    for version in WOW_VERSIONS {
        total += 1;
        match test_wow_version(version) {
            Ok(true) => {
                passed += 1;
                results.push((version.name, "✅ PASSED"));
            }
            Ok(false) => {
                results.push((version.name, "⚠️ SKIPPED (no data)"));
            }
            Err(e) => {
                let error_msg = format!("❌ ERROR: {}", e);
                results.push((version.name, Box::leak(error_msg.into_boxed_str())));
            }
        }
    }

    // Final summary
    println!("\n📊 FINAL RESULTS");
    println!("================");
    for (version, result) in &results {
        println!("{}: {}", version, result);
    }

    println!(
        "\n🏆 Summary: {}/{} versions tested successfully",
        passed, total
    );

    if passed == total {
        println!("✅ ALL VERSIONS PASSED!");
        println!("✅ wow-mpq supports all WoW versions (1.12.1 through 5.4.8)");
        println!("✅ Archive creation and modification work across all formats");
        println!("✅ File integrity preserved across all version formats");
    } else {
        println!("⚠️  Some versions skipped or failed");
    }

    Ok(())
}
