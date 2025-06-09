use wow_mpq::{Archive, OpenOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Enable debug logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    println!("=== Testing v3 archive debug ===");

    let archive_path = "wowmpq_v4.mpq";
    println!("Opening archive: {}", archive_path);

    let mut archive = Archive::open_with_options(archive_path, OpenOptions::default())?;

    // Get archive info
    let info = archive.get_info()?;
    println!("Archive info:");
    println!("  Version: {:?}", info.format_version);
    println!("  Files: {}", info.file_count);

    // Get table info from the archive info
    println!("Table info:");
    println!("  Has HET: {}", info.het_table_info.is_some());
    println!("  Has BET: {}", info.bet_table_info.is_some());
    println!(
        "  Hash table: size={:?}, offset=0x{:X}, failed={}",
        info.hash_table_info.size, info.hash_table_info.offset, info.hash_table_info.failed_to_load
    );
    println!(
        "  Block table: size={:?}, offset=0x{:X}, failed={}",
        info.block_table_info.size,
        info.block_table_info.offset,
        info.block_table_info.failed_to_load
    );

    if let Some(het) = &info.het_table_info {
        println!(
            "  HET: size={:?}, offset=0x{:X}, failed={}",
            het.size, het.offset, het.failed_to_load
        );
    }
    if let Some(bet) = &info.bet_table_info {
        println!(
            "  BET: size={:?}, offset=0x{:X}, failed={}",
            bet.size, bet.offset, bet.failed_to_load
        );
    }

    // Try to list files
    println!("\nListing files:");
    match archive.list() {
        Ok(files) => {
            for (i, entry) in files.iter().enumerate() {
                println!("  {}: {} ({} bytes)", i, entry.name, entry.size);
            }
        }
        Err(e) => {
            println!("  Error listing files: {}", e);
        }
    }

    // Try to read attributes directly
    println!("\nTrying to read (attributes) file:");
    match archive.read_file("(attributes)") {
        Ok(data) => {
            println!("  Success! Read {} bytes", data.len());
            println!("  First 32 bytes: {:?}", &data[..32.min(data.len())]);
        }
        Err(e) => {
            println!("  Error: {}", e);
        }
    }

    Ok(())
}
