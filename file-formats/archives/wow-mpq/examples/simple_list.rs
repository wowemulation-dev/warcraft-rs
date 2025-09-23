use std::env;
use wow_mpq::Archive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <archive.mpq>", args[0]);
        std::process::exit(1);
    }

    let archive_path = &args[1];
    println!("=== Simple Archive Listing ===");
    println!("Archive: {archive_path}");

    let archive = Archive::open(archive_path)?;
    let files = archive.list()?;

    println!("Found {} files:", files.len());
    for (i, file) in files.iter().take(20).enumerate() {
        println!("  {}: {}", i + 1, file.name);
    }
    if files.len() > 20 {
        println!("  ... and {} more files", files.len() - 20);
    }

    Ok(())
}
