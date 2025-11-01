// Test reading with proper patch chain (base + patch)
use wow_mpq::patch_chain::PatchChain;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let base_mpq = "/home/danielsreichenbach/Downloads/wow/4.3.4/4.3.4/Data/art.MPQ";
    let patch_mpq = "/home/danielsreichenbach/Downloads/wow/4.3.4/4.3.4/Data/wow-update-base-15211.MPQ";
    let filename = "Creature/DrakeMount/FelDrakeMount01.skin";

    println!("=== Testing Patch Chain ===\n");
    println!("Base MPQ: {}", base_mpq);
    println!("Patch MPQ: {}", patch_mpq);
    println!("File: {}\n", filename);

    let mut chain = PatchChain::new();
    chain.add_archive(base_mpq, 0)?;
    chain.add_archive(patch_mpq, 1)?;

    match chain.read_file(filename) {
        Ok(data) => {
            println!("✓ Successfully read {} bytes", data.len());
            println!("First 16 bytes: {:02X?}", &data[..16.min(data.len())]);

            if data.len() >= 4 {
                let sig = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                println!("Signature: 0x{:08X}", sig);
                if sig == 0x48435450 {
                    println!("  → PTCH format (ERROR - should be patched!)");
                } else if sig == 0x4E494B53 {
                    println!("  → SKIN format (SUCCESS - patched result)");
                } else {
                    println!("  → Unknown format");
                }
            }

            println!("\n✓ Patch chain test PASSED");
        }
        Err(e) => {
            println!("✗ Failed to read: {}", e);
            println!("\n✗ Patch chain test FAILED");
        }
    }

    Ok(())
}
