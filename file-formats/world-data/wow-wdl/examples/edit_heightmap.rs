use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::PathBuf;

use wow_wdl::parser::WdlParser;
use wow_wdl::types::HeightMapTile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This example demonstrates how to modify heightmap data in a WDL file

    // First, let's check if our example file exists from the previous example
    let input_path = PathBuf::from("example_wotlk.wdl");

    if !input_path.exists() {
        // If not, suggest running the create_and_convert example first
        println!("Example WDL file not found.");
        println!("Please run the create_and_convert example first to generate the file.");
        return Ok(());
    }

    println!("Reading WDL file...");

    // Open and parse the file
    let input_file = File::open(&input_path)?;
    let mut reader = BufReader::new(input_file);

    let parser = WdlParser::new();
    let mut wdl_file = parser.parse(&mut reader)?;

    println!(
        "Successfully parsed WDL file with version: {}",
        wdl_file.version
    );

    // Let's find and modify a heightmap tile
    if let Some(heightmap) = wdl_file.heightmap_tiles.get_mut(&(10, 20)) {
        println!("Found and modifying heightmap tile at (10, 20)");

        // Apply a hill in the center of the heightmap
        apply_hill_to_heightmap(heightmap, 50.0);
    } else {
        println!("No heightmap tile found at (10, 20)");
        // Let's create a new one
        println!("Creating a new heightmap tile at (5, 5)");

        let mut heightmap = HeightMapTile::new();

        // Apply a hill to the new heightmap
        apply_hill_to_heightmap(&mut heightmap, 100.0);

        // Add the new heightmap tile to the file
        wdl_file.heightmap_tiles.insert((5, 5), heightmap);

        // Set the offset for this tile (actual value will be calculated during write)
        wdl_file.map_tile_offsets[5 * 64 + 5] = 1;
    }

    // Save the modified file
    let output_path = PathBuf::from("example_modified.wdl");

    // Write the file with the same parser
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    parser.write(&mut cursor, &wdl_file)?;

    // Write buffer to file
    std::fs::write(&output_path, buffer)?;

    println!("Modified WDL file saved to {output_path:?}");
    println!("\nSummary of modifications:");
    println!("- Added a hill to the terrain");
    println!("- The hill should be visible when viewing the terrain from a distance in-game");
    println!("- This demonstrates how to create terrain features like mountains and valleys");

    Ok(())
}

/// Apply a hill (circular heightmap feature) to the given heightmap
///
/// # Arguments
///
/// * `heightmap` - The heightmap to modify
/// * `height` - The height of the hill
fn apply_hill_to_heightmap(heightmap: &mut HeightMapTile, height: f32) {
    let center_x = 8.0; // Center of the 17x17 grid
    let center_y = 8.0;
    let radius = 5.0; // Radius of the hill

    // Modify outer values (17x17 grid)
    for y in 0..17 {
        for x in 0..17 {
            let index = y * 17 + x;

            // Calculate distance from center
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let distance = (dx * dx + dy * dy).sqrt();

            // Apply a bell curve based on distance
            if distance < radius {
                let factor = 1.0 - (distance / radius).powi(2);
                let height_addition = (height * factor) as i16;

                // Add to the existing height
                heightmap.outer_values[index] += height_addition;
            }
        }
    }

    // Modify inner values (16x16 grid)
    let center_x = 7.5; // Center of the 16x16 grid
    let center_y = 7.5;

    for y in 0..16 {
        for x in 0..16 {
            let index = y * 16 + x;

            // Calculate distance from center
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let distance = (dx * dx + dy * dy).sqrt();

            // Apply a bell curve based on distance
            if distance < radius {
                let factor = 1.0 - (distance / radius).powi(2);
                let height_addition = (height * factor) as i16;

                // Add to the existing height
                heightmap.inner_values[index] += height_addition;
            }
        }
    }
}
