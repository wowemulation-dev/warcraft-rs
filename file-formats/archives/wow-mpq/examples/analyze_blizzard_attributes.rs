use std::env;
use wow_mpq::Archive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <blizzard_archive.mpq>", args[0]);
        eprintln!(
            "Analyzes Blizzard's attributes file format to understand the -28 byte discrepancy"
        );
        std::process::exit(1);
    }

    let archive_path = &args[1];
    println!("=== Analyzing Blizzard Attributes File ===");
    println!("Archive: {}", archive_path);

    let mut archive = Archive::open(archive_path)?;
    let info = archive.get_info()?;

    println!("Archive info:");
    println!("  Format version: {:?}", info.format_version);
    println!("  File count: {}", info.file_count);
    println!("  Archive size: {} bytes", info.file_size);

    // Try to read the attributes file directly
    println!("\n=== Reading (attributes) file ===");
    match archive.read_file("(attributes)") {
        Ok(data) => {
            println!("Successfully read attributes file: {} bytes", data.len());

            if data.len() >= 8 {
                // Parse header manually
                let version = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                let flags = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);

                println!("Attributes header:");
                println!("  Version: {} (0x{:08X})", version, version);
                println!("  Flags: {} (0x{:08X})", flags, flags);

                // Decode flags
                let has_crc32 = (flags & 0x01) != 0;
                let has_filetime = (flags & 0x02) != 0;
                let has_md5 = (flags & 0x04) != 0;
                let has_patch_bit = (flags & 0x08) != 0;

                println!("  Flag breakdown:");
                println!("    CRC32: {}", has_crc32);
                println!("    File time: {}", has_filetime);
                println!("    MD5: {}", has_md5);
                println!("    Patch bit: {}", has_patch_bit);

                // Calculate expected size based on our understanding
                // The block count for attributes should exclude the attributes file itself
                let block_count = info.file_count.saturating_sub(1);
                let mut expected_size = 8; // header
                if has_crc32 {
                    expected_size += block_count * 4;
                }
                if has_filetime {
                    expected_size += block_count * 8;
                }
                if has_md5 {
                    expected_size += block_count * 16;
                }
                if has_patch_bit {
                    expected_size += block_count.div_ceil(8);
                }

                println!("\nSize analysis:");
                println!("  Block count: {}", block_count);
                println!("  Actual size: {} bytes", data.len());
                println!("  Expected size: {} bytes", expected_size);
                println!(
                    "  Difference: {} bytes",
                    data.len() as i32 - expected_size as i32
                );

                // Calculate individual component sizes
                println!("\nComponent size breakdown:");
                println!("  Header: 8 bytes");
                let mut position = 8;
                if has_crc32 {
                    let crc32_size = block_count * 4;
                    println!("  CRC32 array: {} bytes (at 0x{:X})", crc32_size, position);
                    position += crc32_size;
                }
                if has_filetime {
                    let filetime_size = block_count * 8;
                    println!(
                        "  Filetime array: {} bytes (at 0x{:X})",
                        filetime_size, position
                    );
                    position += filetime_size;
                }
                if has_md5 {
                    let md5_size = block_count * 16;
                    println!("  MD5 array: {} bytes (at 0x{:X})", md5_size, position);
                    position += md5_size;
                }
                if has_patch_bit {
                    let ideal_patch_bytes = block_count.div_ceil(8);
                    println!(
                        "  Patch bits (ideal): {} bytes (at 0x{:X})",
                        ideal_patch_bytes, position
                    );

                    // Calculate actual remaining bytes for patch bits
                    let used_bytes = 8
                        + if has_crc32 { block_count * 4 } else { 0 }
                        + if has_filetime { block_count * 8 } else { 0 }
                        + if has_md5 { block_count * 16 } else { 0 };
                    let remaining_bytes = if data.len() > used_bytes {
                        data.len() - used_bytes
                    } else {
                        0
                    };
                    println!("  Patch bits (actual): {} bytes", remaining_bytes);
                    println!(
                        "  Patch bit difference: {} bytes",
                        remaining_bytes as i32 - ideal_patch_bytes as i32
                    );
                } else {
                    let remaining_bytes = if data.len() > position {
                        data.len() - position
                    } else {
                        0
                    };
                    if remaining_bytes > 0 {
                        println!(
                            "  Extra data at end: {} bytes (at 0x{:X})",
                            remaining_bytes, position
                        );
                    }
                }

                // Look at the end of the file to see what might be different
                if data.len() >= 32 {
                    println!("\nLast 32 bytes of attributes file:");
                    let start = data.len().saturating_sub(32);
                    for (i, &byte) in data[start..].iter().enumerate() {
                        if i % 16 == 0 {
                            print!("\n  {:04X}: ", start + i);
                        }
                        print!("{:02X} ", byte);
                    }
                    println!();
                }

                // Calculate patch bit usage
                if has_patch_bit {
                    let used_bytes = 8
                        + if has_crc32 { block_count * 4 } else { 0 }
                        + if has_filetime { block_count * 8 } else { 0 }
                        + if has_md5 { block_count * 16 } else { 0 };

                    if data.len() > used_bytes {
                        let patch_bit_start = used_bytes;
                        let patch_bit_data = &data[patch_bit_start..];
                        println!("\nPatch bit analysis:");
                        println!("  Patch bit data starts at offset: {}", patch_bit_start);
                        println!("  Patch bit data length: {} bytes", patch_bit_data.len());

                        // Count how many files are marked as patches
                        let mut patch_count = 0;
                        for (file_index, &byte) in patch_bit_data.iter().enumerate() {
                            for bit_index in 0..8 {
                                let overall_index = file_index * 8 + bit_index;
                                if overall_index >= block_count {
                                    break;
                                }
                                if (byte & (1 << bit_index)) != 0 {
                                    patch_count += 1;
                                }
                            }
                        }
                        println!("  Files marked as patches: {}/{}", patch_count, block_count);
                    }
                }
            } else {
                println!("Attributes file too small to parse header");
            }
        }
        Err(e) => {
            println!("Failed to read attributes file: {}", e);
        }
    }

    Ok(())
}
