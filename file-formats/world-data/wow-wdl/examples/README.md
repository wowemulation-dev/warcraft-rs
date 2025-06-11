# wow-wdl Examples

This directory contains examples for working with WDL (World Data Low-resolution) files.

## Running Examples

```bash
cargo run --example <example_name> [arguments]
```

## Available Examples

### Basic Operations

- **`create_and_convert.rs`** - Create WDL data and convert formats

  ```bash
  cargo run --example create_and_convert
  ```

- **`edit_heightmap.rs`** - Modify WDL heightmap data

  ```bash
  cargo run --example edit_heightmap
  ```

### Planned Examples

- **`parse_wdl.rs`** - Parse and display WDL information
- **`export_heightmap.rs`** - Export height data to image
- **`validate_wdl.rs`** - Validate WDL structure
- **`compare_with_adt.rs`** - Compare WDL with high-res ADT data

## WDL File Structure

WDL files provide low-resolution terrain data for distant viewing:

- 64x64 height points (vs 16x16 chunks in ADT)
- Low-resolution water data
- Used for distant terrain rendering
- One WDL file per map

## Example Usage

```rust
use wow_wdl::Wdl;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load a WDL file
    let wdl = Wdl::load("path/to/world/maps/azeroth/azeroth.wdl")?;

    // Access height data
    println!("Map size: {}x{}", wdl.width, wdl.height);
    println!("Height samples: {}", wdl.heights.len());

    // Get height at specific coordinate
    let height = wdl.get_height(32, 48)?;
    println!("Height at (32,48): {}", height);

    Ok(())
}
```

## Coordinate Mapping

WDL coordinates map to ADT tiles:

- Each WDL point represents one ADT tile
- WDL (32, 48) = ADT file azeroth_32_48.adt
- Height is average/representative of the ADT area

## Creating WDL from ADT

```rust
use wow_wdl::{Wdl, WdlBuilder};
use wow_adt::Adt;

// Build WDL from multiple ADT files
let mut builder = WdlBuilder::new();

for y in 0..64 {
    for x in 0..64 {
        let adt = Adt::load(format!("azeroth_{}_{}.adt", x, y))?;
        builder.add_adt_data(x, y, &adt)?;
    }
}

let wdl = builder.build()?;
wdl.save("azeroth.wdl")?;
```

## Test Data

WDL files are found in the same directories as WDT files:

```
world/maps/[mapname]/[mapname].wdl
```

Extract from MPQ archives using:

```bash
warcraft-rs mpq extract "World of Warcraft/Data/common.MPQ" world/maps/azeroth/azeroth.wdl
```
