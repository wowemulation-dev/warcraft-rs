// Check what flags we see for patch files
use wow_mpq::Archive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mpq_path = "/home/danielsreichenbach/Downloads/wow/4.3.4/4.3.4/Data/wow-update-base-15211.MPQ";
    let mut archive = Archive::open(mpq_path)?;

    let filename = "Creature/DrakeMount/FelDrakeMount01.skin";

    let file_info = archive.find_file(filename)?
        .ok_or("File not found")?;

    println!("=== Rust MPQ File Info ===");
    println!("File: {}", filename);
    println!("File position: 0x{:X}", file_info.file_pos);
    println!("File size: {} bytes", file_info.file_size);
    println!("Compressed size: {} bytes", file_info.compressed_size);
    println!("Flags: 0x{:08X}", file_info.flags);
    println!("\nFlag checks:");
    println!("  Is single unit: {}", file_info.is_single_unit());
    println!("  Is compressed: {}", file_info.is_compressed());
    println!("  Is encrypted: {}", file_info.is_encrypted());
    println!("  Is patch file: {}", file_info.is_patch_file());
    println!("  Has sector CRC: {}", file_info.has_sector_crc());

    println!("\nComparison with StormLib:");
    println!("  StormLib file_size: 26,464 bytes (patched SKIN)");
    println!("  StormLib compressed: 13,457 bytes");
    println!("  StormLib flags: 0x84000200 (NO PATCH_FILE flag!)");
    println!();
    println!("  Our file_size: {} bytes", file_info.file_size);
    println!("  Our compressed: {} bytes", file_info.compressed_size);
    println!("  Our flags: 0x{:08X}", file_info.flags);

    if file_info.is_patch_file() {
        println!("\n✓ We detect PATCH_FILE flag");
        println!("✗ StormLib does NOT have PATCH_FILE flag");
        println!("\nThis means StormLib already processed this as a patch!");
    }

    Ok(())
}
