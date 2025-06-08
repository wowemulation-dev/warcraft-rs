use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::PathBuf;

use wow_wdl::conversion::convert_wdl_file;
use wow_wdl::parser::WdlParser;
use wow_wdl::types::{BoundingBox, HeightMapTile, HolesData, ModelPlacement, Vec3d, WdlFile};
use wow_wdl::version::WdlVersion;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Create a new WDL file and save it
    println!("Creating a new WDL file...");

    // Create a new WDL file with WotLK version
    let mut file = WdlFile::with_version(WdlVersion::Wotlk);

    // Add a WMO filename and index
    file.wmo_filenames
        .push("World/wmo/Azeroth/Buildings/Human_Farm/Farm.wmo".to_string());
    file.wmo_indices.push(0);

    // Add a WMO placement
    let placement = ModelPlacement {
        id: 1,
        wmo_id: 0,
        position: Vec3d::new(100.0, 200.0, 50.0),
        rotation: Vec3d::new(0.0, 0.0, 0.0),
        bounds: BoundingBox {
            min: Vec3d::new(-10.0, -10.0, -10.0),
            max: Vec3d::new(10.0, 10.0, 10.0),
        },
        flags: 0,
        doodad_set: 0,
        name_set: 0,
        padding: 0,
    };
    file.wmo_placements.push(placement);

    // Add a heightmap tile
    let mut heightmap = HeightMapTile::new();
    for i in 0..HeightMapTile::OUTER_COUNT {
        heightmap.outer_values[i] = (i as i16) % 100;
    }
    for i in 0..HeightMapTile::INNER_COUNT {
        heightmap.inner_values[i] = ((i + 100) as i16) % 100;
    }
    file.heightmap_tiles.insert((10, 20), heightmap);

    // Set the offset for this tile (actual value will be calculated during write)
    file.map_tile_offsets[20 * 64 + 10] = 1;

    // Add holes data
    let mut holes = HolesData::new();
    holes.set_hole(5, 7, true);
    holes.set_hole(8, 9, true);
    file.holes_data.insert((10, 20), holes);

    // Save to a file
    let output_path = PathBuf::from("example_wotlk.wdl");

    // Write the file
    let parser = WdlParser::with_version(WdlVersion::Wotlk);
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    parser.write(&mut cursor, &file)?;

    // Write buffer to file
    std::fs::write(&output_path, buffer)?;

    println!("WDL file saved to {:?}", output_path);

    // Example 2: Read a WDL file
    println!("\nReading WDL file...");

    let input_file = File::open(&output_path)?;
    let mut reader = BufReader::new(input_file);

    // Parse the file
    let parser = WdlParser::new();
    let parsed_file = parser.parse(&mut reader)?;

    println!("Read WDL file with version: {}", parsed_file.version);
    println!("Number of map tiles: {}", parsed_file.heightmap_tiles.len());
    println!(
        "Number of WMO placements: {}",
        parsed_file.wmo_placements.len()
    );

    // Example 3: Convert to Legion version
    println!("\nConverting to Legion version...");

    let legion_file = convert_wdl_file(&parsed_file, WdlVersion::Legion)?;

    println!("Converted to version: {}", legion_file.version);
    println!(
        "Number of Legion WMO placements: {}",
        legion_file.wmo_legion_placements.len()
    );

    // Save Legion file
    let legion_path = PathBuf::from("example_legion.wdl");

    // Write the file using the Legion parser
    let legion_parser = WdlParser::with_version(WdlVersion::Legion);
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    legion_parser.write(&mut cursor, &legion_file)?;

    // Write buffer to file
    std::fs::write(&legion_path, buffer)?;

    println!("Legion WDL file saved to {:?}", legion_path);

    Ok(())
}
