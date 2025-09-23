use std::env;
use wow_mpq::Archive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <archive.mpq>", args[0]);
        std::process::exit(1);
    }

    let archive_path = &args[1];
    println!("=== Archive Listing with Proper Names ===");
    println!("Archive: {archive_path}");

    let archive = Archive::open(archive_path)?;

    // First try to list files using the listfile (gets proper names)
    match archive.list() {
        Ok(files) => {
            println!("\nFiles from (listfile) - {} entries:", files.len());
            for (i, file) in files.iter().take(20).enumerate() {
                println!("  {}: {}", i + 1, file.name);
            }
            if files.len() > 20 {
                println!("  ... and {} more files", files.len() - 20);
            }
        }
        Err(e) => {
            println!("Failed to read listfile: {e}");
            println!("Falling back to anonymous enumeration...");

            // Fall back to list_all() which gives generic names
            let files = archive.list_all()?;
            println!("\nAll files (anonymous) - {} entries:", files.len());
            for (i, file) in files.iter().take(10).enumerate() {
                println!("  {}: {}", i + 1, file.name);
            }
            if files.len() > 10 {
                println!("  ... and {} more files", files.len() - 10);
            }
        }
    }

    Ok(())
}
