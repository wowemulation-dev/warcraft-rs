use wow_mpq::{Archive, OpenOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Enable debug logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    println!("=== Testing v3 archive open without table loading ===");

    let archive_path = "wowmpq_v3.mpq";
    println!("Opening archive: {}", archive_path);

    // Open without loading tables
    let options = OpenOptions::new().load_tables(false);
    let archive = Archive::open_with_options(archive_path, options)?;

    println!("Archive opened successfully!");
    println!("Header info:");
    println!("  Format version: {:?}", archive.header().format_version);
    println!("  Block size: {}", archive.header().block_size);
    println!("  Hash table size: {}", archive.header().hash_table_size);
    println!("  Block table size: {}", archive.header().block_table_size);

    if let Some(het_pos) = archive.header().het_table_pos {
        println!("  HET table position: 0x{:X}", het_pos);
    }
    if let Some(bet_pos) = archive.header().bet_table_pos {
        println!("  BET table position: 0x{:X}", bet_pos);
    }

    println!("\nNow attempting to load tables manually...");

    // Manual table loading would go here if we had public methods for it

    Ok(())
}
