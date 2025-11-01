// Test reading a patch file
use wow_mpq::Archive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    let mpq_path = "/home/danielsreichenbach/Downloads/wow/4.3.4/4.3.4/Data/wow-update-base-15211.MPQ";
    let mut archive = Archive::open(mpq_path)?;

    let filename = "Creature/DrakeMount/FelDrakeMount01.skin";

    println!("Attempting to read patch file: {}", filename);
    println!();

    match archive.read_file(filename) {
        Ok(data) => {
            println!("✓ Successfully read {} bytes", data.len());
            println!("First 16 bytes: {:02X?}", &data[..16.min(data.len())]);

            if data.len() >= 4 {
                let sig = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                println!("Signature: 0x{:08X}", sig);
                if sig == 0x48435450 {
                    println!("  → PTCH format (raw patch data)");
                } else if sig == 0x4E494B53 {
                    println!("  → SKIN format (patched result)");
                } else {
                    println!("  → Unknown format");
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to read: {}", e);
        }
    }

    Ok(())
}
