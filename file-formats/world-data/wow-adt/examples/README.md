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
use wow_adt::{Adt, AdtType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load terrain ADT
    let adt = Adt::load("path/to/world/maps/azeroth/azeroth_32_48.adt")?;

    // Display basic information
    println!("ADT Version: {:?}", adt.version);
    println!("Chunks: {}", adt.chunks.len());

    // Access terrain data
    for (idx, chunk) in adt.chunks.iter().enumerate() {
        println!("Chunk {}: {} vertices", idx, chunk.vertices.len());
    }

    Ok(())
}
```

## Working with Different ADT Types

```rust
// Load object placement ADT
let obj_adt = Adt::load_typed("map_32_48_obj0.adt", AdtType::Objects)?;

// Load texture information ADT
let tex_adt = Adt::load_typed("map_32_48_tex0.adt", AdtType::Textures)?;
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
