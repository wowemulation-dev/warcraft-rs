use std::env;
use std::fs::File;
use std::io::BufReader;
use wow_wmo::discover_wmo_chunks;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <wmo-file>", args[0]);
        std::process::exit(1);
    }

    let file = File::open(&args[1])?;
    let mut reader = BufReader::new(file);

    let discovery = discover_wmo_chunks(&mut reader)?;

    println!("Total chunks: {}", discovery.chunks.len());
    println!("\nChunk List:");
    println!("{:<8} {:<12} {:<12}", "ID", "Offset", "Size");
    println!("{:-<32}", "");

    for chunk in &discovery.chunks {
        println!(
            "{:<8} 0x{:08X}   {:<8}",
            chunk.id.as_str(),
            chunk.offset,
            chunk.size
        );

        // Show MODD details
        if chunk.id.as_str() == "MODD" {
            println!("  -> MODD chunk size: {} bytes", chunk.size);
            if chunk.size > 0 {
                let entry_size_40 = chunk.size / 40;
                let entry_size_48 = chunk.size / 48;
                println!(
                    "  -> If 40 bytes per entry: {} entries (remainder: {})",
                    entry_size_40,
                    chunk.size % 40
                );
                println!(
                    "  -> If 48 bytes per entry: {} entries (remainder: {})",
                    entry_size_48,
                    chunk.size % 48
                );
            }
        }
    }

    if discovery.has_unknown_chunks() {
        println!("\nUnknown chunks: {}", discovery.unknown_count());
    }
    if discovery.has_malformed_chunks() {
        println!("Malformed chunks: {}", discovery.malformed_count());
    }

    Ok(())
}
