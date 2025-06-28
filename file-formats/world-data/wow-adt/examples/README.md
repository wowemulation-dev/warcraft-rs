# wow-adt Examples

This directory contains examples demonstrating the wow-adt library functionality.

## Running Examples

```bash
cargo run --example <example_name> [arguments]
```

## Available Examples

### Basic Operations

- **`parse_adt.rs`** - Parse and display ADT file information

  ```bash
  cargo run --example parse_adt path/to/map_x_y.adt
  ```

- **`validate_adt.rs`** - Validate ADT file structure and integrity

  ```bash
  cargo run --example validate_adt path/to/map_x_y.adt
  ```

- **`version_info.rs`** - Display version information and compatibility

  ```bash
  cargo run --example version_info
  ```

### Planned Examples

- **`extract_heightmap.rs`** - Extract terrain height data
- **`list_textures.rs`** - List all textures used in the ADT
- **`export_obj.rs`** - Export terrain to OBJ format
- **`merge_tiles.rs`** - Merge multiple ADT tiles
- **`split_chunks.rs`** - Split ADT into individual chunks

## ADT File Structure

ADT (Area Data Table) files contain terrain data for World of Warcraft:

- 16x16 chunks per tile
- Height maps, texture layers, water, shadows
- Model and WMO placement information
- Multiple file types: .adt (terrain), _obj0.adt,_obj1.adt (objects), _tex0.adt (textures)

## Example Usage

```rust
use wow_adt::Adt;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse terrain ADT
    let adt = Adt::from_path("path/to/world/maps/azeroth/azeroth_32_48.adt")?;

    // Display basic information
    println!("ADT Version: {:?}", adt.version());
    println!("Terrain chunks: {}", adt.mcnk_chunks().len());

    // Access terrain data
    for (idx, chunk) in adt.mcnk_chunks().iter().enumerate() {
        println!("Chunk {}: {} heights", idx, chunk.height_map.len());
    }

    // Check for water data (WotLK+)
    if let Some(water) = adt.mh2o() {
        println!("Contains water data with {} chunks", water.chunks.len());
    }

    Ok(())
}
```

## Working with Different ADT Files

```rust
// Parse object placement ADT (Cataclysm+ split format)
let obj_adt = Adt::from_path("map_32_48_obj0.adt")?;
println!("Doodads: {}", obj_adt.mddf.as_ref().map_or(0, |d| d.doodads.len()));

// Parse texture information ADT (Cataclysm+ split format)  
let tex_adt = Adt::from_path("map_32_48_tex0.adt")?;
println!("Textures: {}", tex_adt.mtex.as_ref().map_or(0, |t| t.filenames.len()));
```

## Coordinate System

ADT files use a specific coordinate system:

- X increases eastward
- Y increases northward
- Z is height
- Each ADT covers 533.33333 world units
- Each chunk is 33.33333 world units

## Test Data

ADT files must be extracted from MPQ archives.
Use the wow-mpq crate to extract:

```bash
warcraft-rs mpq extract wow-update-13329.MPQ world/maps/azeroth/azeroth_32_48.adt
```
