use wow_mpq::Archive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing wow-mpq reading its own V3 archive ===");

    let mut archive = Archive::open("wowmpq_v3.mpq")?;

    println!("Archive opened successfully!");

    // List all files
    if let Ok(listfile) = archive.read_file("(listfile)") {
        let listfile_content = String::from_utf8_lossy(&listfile);
        println!("Files in archive:");
        for line in listfile_content.lines() {
            if !line.trim().is_empty() {
                println!("  {}", line.trim());
            }
        }
    }

    // Try to read attributes file directly
    match archive.read_file("(attributes)") {
        Ok(data) => {
            println!("Successfully read (attributes) file: {} bytes", data.len());
            println!("First 24 bytes: {:?}", &data[..data.len().min(24)]);
        }
        Err(e) => {
            println!("Failed to read (attributes) file: {}", e);
        }
    }

    Ok(())
}
