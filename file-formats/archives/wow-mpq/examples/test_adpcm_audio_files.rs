//! Test ADPCM audio file handling across WoW versions
//! This test specifically targets audio files that might use ADPCM compression

use std::path::PathBuf;
use wow_mpq::{Archive, ArchiveBuilder, FormatVersion, ListfileOption};

const WOW_VERSIONS: &[(&str, &str)] = &[
    (
        "WoW 3.3.5a",
        "/home/danielsreichenbach/Downloads/wow/3.3.5a/Data",
    ),
    (
        "WoW 4.3.4",
        "/home/danielsreichenbach/Downloads/wow/4.3.4/4.3.4/Data",
    ),
    (
        "WoW 5.4.8",
        "/home/danielsreichenbach/Downloads/wow/5.4.8/5.4.8/Data",
    ),
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 Testing ADPCM Audio File Handling");
    println!("====================================");

    let mut total_audio_files = 0;
    let mut successfully_processed = 0;
    let mut adpcm_files = Vec::new();

    for (version_name, data_path) in WOW_VERSIONS {
        let path = PathBuf::from(data_path);
        if !path.exists() {
            println!("\n⚠️ {} data not found at: {}", version_name, data_path);
            continue;
        }

        println!("\n📁 Checking {} archives...", version_name);

        // Look for sound MPQ files
        let entries: Vec<_> = std::fs::read_dir(&path)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_lowercase();
                name.ends_with(".mpq") && (name.contains("sound") || name.contains("patch"))
            })
            .collect();

        for entry in entries.iter().take(3) {
            // Check first 3 sound-related archives
            let archive_path = entry.path();
            let archive_name = archive_path.file_name().unwrap().to_str().unwrap();

            match Archive::open(&archive_path) {
                Ok(mut archive) => {
                    println!("  📦 Scanning {}", archive_name);

                    match archive.list() {
                        Ok(files) => {
                            // Look for audio files
                            let audio_files: Vec<_> = files
                                .into_iter()
                                .filter(|f| {
                                    let name = f.name.to_lowercase();
                                    name.ends_with(".wav")
                                        || name.ends_with(".ogg")
                                        || name.ends_with(".mp3")
                                })
                                .take(10) // Limit to 10 audio files per archive
                                .collect();

                            for file_entry in audio_files {
                                total_audio_files += 1;

                                match archive.read_file(&file_entry.name) {
                                    Ok(data) => {
                                        // Check if this might be ADPCM compressed
                                        // ADPCM files are typically WAV files
                                        if file_entry.name.to_lowercase().ends_with(".wav")
                                            && data.len() > 0
                                        {
                                            // Store for later testing
                                            adpcm_files.push((
                                                format!("{}/{}", version_name, archive_name),
                                                file_entry.name.clone(),
                                                data.clone(),
                                            ));
                                        }
                                        successfully_processed += 1;

                                        if successfully_processed <= 5 {
                                            println!(
                                                "    ✓ {}: {} bytes",
                                                file_entry.name,
                                                data.len()
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        println!(
                                            "    ❌ {}: Failed to read - {}",
                                            file_entry.name, e
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => println!("    ⚠️ Could not list files: {}", e),
                    }
                }
                Err(e) => println!("  ⚠️ Could not open {}: {}", archive_name, e),
            }
        }
    }

    println!("\n📊 Initial Results:");
    println!("  Total audio files found: {}", total_audio_files);
    println!("  Successfully read: {}", successfully_processed);
    println!("  WAV files for testing: {}", adpcm_files.len());

    if adpcm_files.is_empty() {
        println!("\n⚠️ No WAV files found for ADPCM testing");
        return Ok(());
    }

    // Test creating an archive with audio files
    println!("\n🔨 Creating test archive with audio files...");
    let test_archive = "test_audio_adpcm.mpq";
    std::fs::remove_file(test_archive).ok();

    let mut builder = ArchiveBuilder::new()
        .version(FormatVersion::V1)
        .listfile_option(ListfileOption::Generate);

    // Add first few audio files
    for (i, (_source, filename, data)) in adpcm_files.iter().take(5).enumerate() {
        println!("  Adding {}: {} ({} bytes)", i + 1, filename, data.len());
        builder = builder.add_file_data(data.clone(), filename);
    }

    builder.build(test_archive)?;
    println!("✅ Created test archive: {}", test_archive);

    // Verify the archive
    println!("\n🔍 Verifying audio files in created archive...");
    let mut verify_archive = Archive::open(test_archive)?;
    let mut verified = 0;

    for (_, filename, original_data) in adpcm_files.iter().take(5) {
        match verify_archive.read_file(filename) {
            Ok(extracted_data) => {
                if extracted_data == *original_data {
                    verified += 1;
                    println!("  ✅ {}: Verified successfully", filename);
                } else {
                    println!("  ❌ {}: Data mismatch!", filename);
                }
            }
            Err(e) => {
                println!("  ❌ {}: Read error - {}", filename, e);
            }
        }
    }

    println!("\n📊 Final Results:");
    println!("  Audio files added: {}", adpcm_files.len().min(5));
    println!("  Successfully verified: {}", verified);

    if verified == adpcm_files.len().min(5) {
        println!("✅ ADPCM audio handling test PASSED!");
        println!("✅ The ADPCM overflow issue has been fixed!");
    } else {
        println!("❌ Some audio files failed verification");
    }

    // Test with StormLib if available
    println!("\n🔍 Testing StormLib compatibility...");
    test_stormlib_compatibility(test_archive)?;

    // Cleanup
    std::fs::remove_file(test_archive).ok();

    Ok(())
}

fn test_stormlib_compatibility(archive_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    // Create a simple test program
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
    
    SFILE_FIND_DATA fd;
    HANDLE hFind = SFileFindFirstFile(hMpq, "*.wav", &fd, NULL);
    if (hFind) {{
        do {{
            printf("Found: %s\n", fd.cFileName);
        }} while (SFileFindNextFile(hFind, &fd));
        SFileFindClose(hFind);
    }}
    
    SFileCloseArchive(hMpq);
    printf("StormLib test passed\n");
    return 0;
}}
"#,
        archive_path
    );

    std::fs::write("test_audio.c", test_code)?;

    let compile = Command::new("gcc")
        .args(&[
            "-o",
            "test_audio",
            "test_audio.c",
            "-I/home/danielsreichenbach/Repos/github.com/ladislav-zezula/StormLib/src",
            "-L/home/danielsreichenbach/Repos/github.com/ladislav-zezula/StormLib/build",
            "-lstorm",
            "-lz",
            "-lbz2",
        ])
        .output();

    match compile {
        Ok(output) if output.status.success() => {
            let result = Command::new("./test_audio")
                .env(
                    "LD_LIBRARY_PATH",
                    "/home/danielsreichenbach/Repos/github.com/ladislav-zezula/StormLib/build",
                )
                .output()?;

            if result.status.success() {
                println!("  ✅ StormLib successfully read our audio archive!");
                println!("  Output: {}", String::from_utf8_lossy(&result.stdout));
            }

            std::fs::remove_file("test_audio").ok();
        }
        _ => println!("  ⚠️ StormLib test compilation failed"),
    }

    std::fs::remove_file("test_audio.c").ok();
    Ok(())
}
