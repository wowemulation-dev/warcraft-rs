use std::error::Error;
use wow_mpq::Archive;

fn main() -> Result<(), Box<dyn Error>> {
    let archives = ["stormlib_v1.mpq", "wowmpq_v1.mpq"];

    for archive_path in &archives {
        println!("\n=== Contents of {archive_path} ===");

        let archive = Archive::open(archive_path)?;
        let files = archive.list_all()?;

        println!("Total files: {}", files.len());
        for (i, entry) in files.iter().enumerate() {
            println!("{}: {} ({} bytes)", i, entry.name, entry.size);
        }

        // Try to read the listfile
        match archive.read_file("(listfile)") {
            Ok(data) => {
                let listfile_content = String::from_utf8_lossy(&data);
                println!("\n(listfile) content:");
                println!("{listfile_content}");
            }
            Err(e) => {
                println!("\nNo (listfile) or error reading: {e}");
            }
        }
    }

    Ok(())
}
