use std::env;
use wow_mpq::Archive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <mpq_file>", args[0]);
        std::process::exit(1);
    }

    let mpq_path = &args[1];
    println!("Analyzing: {}", mpq_path);

    let mut archive = Archive::open(mpq_path)?;

    // Get BET file count
    let bet_file_count = if let Some(bet_table) = archive.bet_table() {
        bet_table.header.file_count
    } else {
        println!("No BET table found");
        return Ok(());
    };

    println!("BET file count: {}", bet_file_count);

    // Also check block table if available
    if let Some(block_table) = archive.block_table() {
        println!("Block table entries: {}", block_table.entries().len());
    } else {
        println!("No block table available");
    }

    // Try counting files by enumeration
    match archive.list_all() {
        Ok(files) => {
            println!("Enumerated file count: {}", files.len());
        }
        Err(e) => {
            println!("Failed to enumerate files: {}", e);
        }
    }

    // Try to read (attributes) file
    match archive.read_file("(attributes)") {
        Ok(data) => {
            println!("Attributes file size: {} bytes", data.len());

            if data.len() >= 8 {
                let version = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                let flags = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);

                println!("Version: {}", version);
                println!("Flags: 0x{:08X}", flags);

                // Calculate expected size based on flags and BET file count
                let mut expected_size = 8; // header
                let block_count = bet_file_count as usize;

                if flags & 0x00000001 != 0 {
                    // CRC32
                    expected_size += block_count * 4;
                    println!("Has CRC32: +{} bytes", block_count * 4);
                }
                if flags & 0x00000002 != 0 {
                    // FILETIME
                    expected_size += block_count * 8;
                    println!("Has FILETIME: +{} bytes", block_count * 8);
                }
                if flags & 0x00000004 != 0 {
                    // MD5
                    expected_size += block_count * 16;
                    println!("Has MD5: +{} bytes", block_count * 16);
                }
                if flags & 0x00000008 != 0 {
                    // PATCH_BIT
                    let patch_bytes = block_count.div_ceil(8);
                    expected_size += patch_bytes;
                    println!(
                        "Has PATCH_BIT: +{} bytes (153 files = {} bits = {} bytes)",
                        patch_bytes, block_count, patch_bytes
                    );
                    println!(
                        "PATCH_BIT calculation: {} / 8 = {}, ceil = {}",
                        block_count,
                        block_count as f64 / 8.0,
                        patch_bytes
                    );
                }

                println!("Expected size: {} bytes", expected_size);
                println!("Actual size: {} bytes", data.len());
                println!(
                    "Difference: {} bytes",
                    expected_size as i32 - data.len() as i32
                );

                // Show first 32 bytes for analysis
                println!("First 32 bytes:");
                for (i, &byte) in data.iter().take(32).enumerate() {
                    if i % 16 == 0 {
                        print!("{:04X}: ", i);
                    }
                    print!("{:02X} ", byte);
                    if i % 16 == 15 {
                        println!();
                    }
                }
                if data.len() > 0 && data.len() % 16 != 0 {
                    println!();
                }

                // Show last 32 bytes to see what's missing
                println!("Last 32 bytes:");
                let start = if data.len() >= 32 { data.len() - 32 } else { 0 };
                for (i, &byte) in data.iter().skip(start).enumerate() {
                    if i % 16 == 0 {
                        print!("{:04X}: ", start + i);
                    }
                    print!("{:02X} ", byte);
                    if i % 16 == 15 {
                        println!();
                    }
                }
                if data.len() > 0 && data.len() % 16 != 0 {
                    println!();
                }
            }
        }
        Err(e) => {
            println!("Failed to read (attributes): {}", e);
        }
    }

    Ok(())
}
