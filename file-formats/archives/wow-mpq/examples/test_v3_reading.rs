use std::env;
use wow_mpq::Archive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <archive.mpq>", args[0]);
        std::process::exit(1);
    }

    let archive_path = &args[1];
    println!("=== Testing wow-mpq reading its own V3 archive ===");
    println!("Archive: {}", archive_path);

    // Open archive
    let mut archive = Archive::open(archive_path)?;
    println!("Archive opened successfully!");

    // List files
    println!("\nListing files:");
    let files = archive.list_all()?;
    for (i, file_entry) in files.iter().enumerate() {
        println!("  {}: {}", i + 1, file_entry.name);
    }
    println!("Total files: {}", files.len());

    // Try to read specific files
    let test_files = ["readme.txt", "(listfile)", "(attributes)"];
    for filename in &test_files {
        println!("\nTrying to read: {}", filename);
        match archive.read_file(filename) {
            Ok(data) => {
                println!("  Successfully read {} bytes", data.len());
                if data.len() > 0 {
                    let preview = &data[..data.len().min(32)];
                    print!("  First bytes: ");
                    for &byte in preview {
                        print!("{:02x} ", byte);
                    }
                    println!();
                }
            }
            Err(e) => {
                println!("  Failed to read: {}", e);
            }
        }
    }

    Ok(())
}
