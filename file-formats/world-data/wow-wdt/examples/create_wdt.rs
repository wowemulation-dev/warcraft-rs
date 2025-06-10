//! Example showing how to create a new WDT file

use std::fs::File;
use std::io::BufWriter;
use wow_wdt::{
    WdtFile, WdtWriter,
    chunks::{ModfChunk, ModfEntry, MphdFlags, MwmoChunk},
    version::WowVersion,
};

fn main() -> anyhow::Result<()> {
    // Example 1: Create a simple terrain map
    create_terrain_map()?;

    // Example 2: Create a WMO-only map (instance)
    create_wmo_only_map()?;

    Ok(())
}

fn create_terrain_map() -> anyhow::Result<()> {
    println!("Creating terrain map...");

    // Create a new WDT for WotLK
    let mut wdt = WdtFile::new(WowVersion::WotLK);

    // Set some map properties
    wdt.mphd.flags |= MphdFlags::ADT_HAS_MCCV | MphdFlags::ADT_HAS_BIG_ALPHA;

    // Add empty MWMO (required for pre-Cataclysm terrain maps)
    wdt.mwmo = Some(MwmoChunk::new());

    // Mark some tiles as existing
    // Create a small island in the center
    for y in 30..35 {
        for x in 30..35 {
            if let Some(tile) = wdt.main.get_mut(x, y) {
                tile.set_has_adt(true);
                tile.area_id = 1; // Dun Morogh
            }
        }
    }

    // Write to file
    let file = File::create("terrain_example.wdt")?;
    let mut writer = WdtWriter::new(BufWriter::new(file));
    writer.write(&wdt)?;

    println!(
        "Created terrain_example.wdt with {} tiles",
        wdt.count_existing_tiles()
    );

    Ok(())
}

fn create_wmo_only_map() -> anyhow::Result<()> {
    println!("\nCreating WMO-only map...");

    // Create a new WDT for Classic
    let mut wdt = WdtFile::new(WowVersion::Classic);

    // Mark as WMO-only
    wdt.mphd.flags |= MphdFlags::WDT_USES_GLOBAL_MAP_OBJ;

    // Set the WMO filename
    let mut mwmo = MwmoChunk::new();
    mwmo.add_filename("World\\wmo\\Dungeon\\KL_Stockades\\KL_Stockades.wmo".to_string());
    wdt.mwmo = Some(mwmo);

    // Add WMO placement information
    let mut modf = ModfChunk::new();
    modf.add_entry(ModfEntry {
        id: 0,
        unique_id: 0xFFFFFFFF,     // Classic uses -1
        position: [0.0, 0.0, 0.0], // Center of map
        rotation: [0.0, 0.0, 0.0], // No rotation
        lower_bounds: [-200.0, -200.0, -50.0],
        upper_bounds: [200.0, 200.0, 100.0],
        flags: 0,
        doodad_set: 0,
        name_set: 0,
        scale: 0, // Classic uses 0
    });
    wdt.modf = Some(modf);

    // Write to file
    let file = File::create("instance_example.wdt")?;
    let mut writer = WdtWriter::new(BufWriter::new(file));
    writer.write(&wdt)?;

    println!("Created instance_example.wdt (WMO-only map)");

    Ok(())
}
