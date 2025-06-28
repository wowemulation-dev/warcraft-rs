//! Example showing how to parse and analyze a WDT file

use std::fs::File;
use std::io::BufReader;
use wow_wdt::{WdtReader, version::WowVersion};

fn main() -> anyhow::Result<()> {
    // Example: Parse a WDT file
    let _wdt_path = "path/to/your/map.wdt";

    // You can also read from command line arguments
    let args: Vec<String> = std::env::args().collect();
    let path = if args.len() > 1 {
        &args[1]
    } else {
        println!("Usage: {} <path_to_wdt_file>", args[0]);
        println!("\nExample paths:");
        println!("  Classic: World/Maps/Azeroth/Azeroth.wdt");
        println!("  TBC: World/Maps/Expansion01/Expansion01.wdt");
        println!("  WotLK: World/Maps/Northrend/Northrend.wdt");
        return Ok(());
    };

    // Open the WDT file
    let file = File::open(path)?;
    let mut reader = WdtReader::new(BufReader::new(file), WowVersion::WotLK);

    // Parse the WDT
    let wdt = reader.read()?;

    // Display basic information
    println!("WDT File: {path}");
    println!("Version: {}", wdt.mver.version);
    println!(
        "Type: {}",
        if wdt.is_wmo_only() {
            "WMO-only map"
        } else {
            "Terrain map"
        }
    );

    // Display flags
    println!("\nMPHD Flags: 0x{:08X}", wdt.mphd.flags.bits());

    // Count existing tiles
    let tile_count = wdt.count_existing_tiles();
    println!("\nExisting tiles: {tile_count} / 4096");

    // List first 10 tiles
    println!("\nFirst 10 tiles:");
    let mut count = 0;
    for y in 0..64 {
        for x in 0..64 {
            if let Some(tile) = wdt.get_tile(x, y) {
                if tile.has_adt {
                    println!("  [{:2},{:2}] - Area ID: {}", x, y, tile.area_id);
                    count += 1;
                    if count >= 10 {
                        break;
                    }
                }
            }
        }
        if count >= 10 {
            break;
        }
    }

    // Display WMO information if present
    if let Some(ref wmo) = wdt.mwmo {
        if !wmo.is_empty() {
            println!("\nGlobal WMO: {}", wmo.filenames[0]);

            if let Some(ref modf) = wdt.modf {
                let entry = &modf.entries[0];
                println!(
                    "  Position: [{:.2}, {:.2}, {:.2}]",
                    entry.position[0], entry.position[1], entry.position[2]
                );
                println!(
                    "  Rotation: [{:.2}°, {:.2}°, {:.2}°]",
                    entry.rotation[0].to_degrees(),
                    entry.rotation[1].to_degrees(),
                    entry.rotation[2].to_degrees()
                );
            }
        }
    }

    // Validate the file
    let warnings = wdt.validate();
    if !warnings.is_empty() {
        println!("\nValidation warnings:");
        for warning in warnings {
            println!("  - {warning}");
        }
    } else {
        println!("\nFile validation: OK");
    }

    Ok(())
}
