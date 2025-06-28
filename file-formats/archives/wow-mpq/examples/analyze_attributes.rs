use wow_mpq::Archive;

fn analyze_attributes(path: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Analyzing {name} attributes file ===");

    let mut archive = Archive::open(path)?;

    match archive.read_file("(attributes)") {
        Ok(data) => {
            println!("Attributes file size: {} bytes", data.len());

            // Dump raw bytes in hex
            println!("Raw bytes (hex):");
            for (i, chunk) in data.chunks(16).enumerate() {
                print!("{:04X}: ", i * 16);
                for &byte in chunk {
                    print!("{byte:02X} ");
                }
                // Pad if less than 16 bytes
                for _ in chunk.len()..16 {
                    print!("   ");
                }
                print!(" | ");
                for &byte in chunk {
                    if byte.is_ascii_graphic() || byte == b' ' {
                        print!("{}", char::from(byte));
                    } else {
                        print!(".");
                    }
                }
                println!();
            }

            // Try to interpret structure
            if data.len() >= 8 {
                let version = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                let flags = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                println!("\nParsed structure:");
                println!("  Version: {version} (0x{version:X})");
                println!("  Flags: {flags} (0x{flags:X})");

                // Check what flags mean
                println!("  Flag bits:");
                if flags & 0x01 != 0 {
                    println!("    0x01: CRC32");
                }
                if flags & 0x02 != 0 {
                    println!("    0x02: Timestamp");
                }
                if flags & 0x04 != 0 {
                    println!("    0x04: MD5");
                }
                if flags & 0x08 != 0 {
                    println!("    0x08: Unknown");
                }
                if flags & 0x10 != 0 {
                    println!("    0x10: Unknown");
                }

                // Calculate expected file entry count
                let header_size = 8;
                let remaining = data.len() - header_size;

                // Each file entry contains: CRC32 (4) + timestamp (8) + MD5 (16) = 28 bytes per file
                if flags & 0x07 == 0x07 {
                    // CRC32 + timestamp + MD5
                    let entry_size = 4 + 8 + 16; // 28 bytes
                    let file_count = remaining / entry_size;
                    println!(
                        "  Estimated file count: {file_count} (assuming {entry_size} bytes per entry)"
                    );
                } else if flags & 0x01 != 0 {
                    // Just CRC32
                    let entry_size = 4;
                    let file_count = remaining / entry_size;
                    println!(
                        "  Estimated file count: {file_count} (assuming {entry_size} bytes per entry)"
                    );
                }
            }
        }
        Err(e) => {
            println!("Failed to read (attributes) file: {e}");
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Attributes File Format Analysis ===");

    // Analyze our files
    analyze_attributes("wowmpq_v1.mpq", "wow-mpq V1")?;
    analyze_attributes("wowmpq_v2.mpq", "wow-mpq V2")?;
    analyze_attributes("wowmpq_v3.mpq", "wow-mpq V3")?;

    // Analyze StormLib files
    analyze_attributes("tests/stormlib_comparison/stormlib_v1.mpq", "StormLib V1")?;
    analyze_attributes("tests/stormlib_comparison/stormlib_v2.mpq", "StormLib V2")?;
    analyze_attributes("tests/stormlib_comparison/stormlib_v3.mpq", "StormLib V3")?;

    Ok(())
}
