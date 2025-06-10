//! Analyze the Huffman compression issue by extracting the problematic file with wow-mpq

use std::fs;
use wow_mpq::Archive;

const PROBLEMATIC_FILE: &str = "World\\Maps\\Azeroth\\Azeroth_28_51_tex1.adt";
const CATA_ARCHIVE: &str = "/home/danielsreichenbach/Downloads/wow/4.3.4/4.3.4/Data/world.MPQ";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 wow-mpq Huffman Compression Analysis");
    println!("======================================");

    println!("\n📂 Opening Cataclysm archive: {}", CATA_ARCHIVE);
    let mut archive = Archive::open(CATA_ARCHIVE)?;

    println!("\n🔍 Analyzing archive and file...");

    // Get archive info
    let info = archive.get_info()?;
    println!("  Archive format: {:?}", info.format_version);
    println!("  Archive size: {} bytes", info.file_size);

    // Check if file exists
    match archive.find_file(PROBLEMATIC_FILE)? {
        Some(file_info) => {
            println!("\n📄 Found file: {}", PROBLEMATIC_FILE);
            println!("  File size: {} bytes", file_info.file_size);
            println!("  Compressed size: {} bytes", file_info.compressed_size);
            println!("  Flags: 0x{:08x}", file_info.flags);

            // Decode flags
            if file_info.flags & 0x00000200 != 0 {
                println!("  File is compressed");
            }
            if file_info.flags & 0x00010000 != 0 {
                println!("  File is encrypted");
            }
            if file_info.flags & 0x04000000 != 0 {
                println!("  File has sector CRCs");
            }
        }
        None => {
            println!("❌ File not found in archive");
            return Ok(());
        }
    }

    println!("\n📥 Attempting to extract file with wow-mpq...");

    match archive.read_file(PROBLEMATIC_FILE) {
        Ok(data) => {
            println!("  ✅ wow-mpq extraction successful!");
            println!("  Extracted {} bytes", data.len());

            // Save extracted data
            fs::write("huffman_test_wowmpq.dat", &data)?;
            println!("  💾 Saved extracted data to huffman_test_wowmpq.dat");

            // Analyze first few bytes
            println!("\n🔍 First 32 bytes of extracted data:");
            print!("  ");
            for (i, byte) in data.iter().take(32).enumerate() {
                print!("{:02x} ", byte);
                if (i + 1) % 16 == 0 {
                    print!("\n  ");
                }
            }
            println!();

            // Compare with StormLib data if available
            if let Ok(stormlib_data) = fs::read("huffman_test_stormlib.dat") {
                println!("\n🔍 Comparing with StormLib extraction...");
                if data == stormlib_data {
                    println!("  ✅ Data matches StormLib extraction perfectly!");
                } else {
                    println!("  ❌ Data differs from StormLib extraction");
                    println!("  wow-mpq size: {} bytes", data.len());
                    println!("  StormLib size: {} bytes", stormlib_data.len());

                    if data.len() == stormlib_data.len() {
                        // Find first difference
                        for (i, (a, b)) in data.iter().zip(stormlib_data.iter()).enumerate() {
                            if a != b {
                                println!(
                                    "  First difference at byte {}: 0x{:02x} vs 0x{:02x}",
                                    i, a, b
                                );
                                break;
                            }
                        }
                    }
                }
            } else {
                println!("\n⚠️ StormLib reference data not found (run StormLib test first)");
            }
        }
        Err(e) => {
            println!("  ❌ wow-mpq extraction failed: {}", e);
            println!("  Error type: {:?}", e);

            // Try to get more detailed error information
            if let wow_mpq::Error::Compression(comp_err) = &e {
                println!("  Compression error details: {}", comp_err);
            }
        }
    }

    println!("\n📊 Analysis Summary:");
    println!("===================");
    println!("File: {}", PROBLEMATIC_FILE);
    println!("Archive: {}", CATA_ARCHIVE);

    Ok(())
}
