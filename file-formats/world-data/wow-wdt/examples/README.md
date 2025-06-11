# wow-wdt Examples

This directory contains examples demonstrating wow-wdt functionality.

## Running Examples

```bash
cargo run --example <example_name> [arguments]
```

## Available Examples

### Basic Operations

- **`parse_wdt.rs`** - Parse and display WDT file information

  ```bash
  cargo run --example parse_wdt path/to/world/maps/azeroth/azeroth.wdt
  ```

- **`create_wdt.rs`** - Create a new WDT file programmatically

  ```bash
  cargo run --example create_wdt
  ```

## WDT File Structure

WDT (World Data Table) files are map definition files that:

- Define which ADT tiles exist for a map
- Store global map properties and flags
- Reference world map objects (WMOs)
- Link to texture and model files

## Example Usage

```rust
use wow_wdt::{Wdt, WdtFlags};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load a WDT file
    let wdt = Wdt::load("path/to/azeroth.wdt")?;

    // Display map information
    println!("Map Flags: {:?}", wdt.flags);

    // Check which tiles exist
    for y in 0..64 {
        for x in 0..64 {
            if wdt.tile_exists(x, y) {
                println!("Tile ({}, {}) exists", x, y);
            }
        }
    }

    // List map objects
    for obj in &wdt.objects {
        println!("WMO: {} at position {:?}", obj.filename, obj.position);
    }

    Ok(())
}
```

## Creating a WDT

```rust
use wow_wdt::{Wdt, WdtBuilder, WdtFlags};

let mut builder = WdtBuilder::new();

// Set map properties
builder.set_flags(WdtFlags::GLOBAL_WMO);

// Define which tiles exist
builder.set_tile(32, 48, true); // Tile at (32, 48) exists
builder.set_area_id(32, 48, 1519); // Stormwind City

// Add a world map object
builder.add_wmo(
    "world/wmo/azeroth/buildings/stormwind/stormwind.wmo",
    Vector3::new(100.0, 200.0, 50.0),
    Vector3::new(0.0, 0.0, 0.0),
);

let wdt = builder.build()?;
wdt.save("custom_map.wdt")?;
```

## Map Flags

Common WDT flags:

- `GLOBAL_WMO` - Map uses a global WMO
- `VERTEX_SHADING` - Use vertex shading
- `BIG_ALPHA` - Big alpha maps
- `TERRAIN_SHADERS` - Use terrain shaders

## CLI Integration

The wow-wdt functionality is also available through the warcraft-rs CLI:

```bash
warcraft-rs wdt info azeroth.wdt
warcraft-rs wdt tiles azeroth.wdt
warcraft-rs wdt objects azeroth.wdt
warcraft-rs wdt tree azeroth.wdt
```

## Test Data

WDT files are located in map directories:

```
world/maps/[mapname]/[mapname].wdt
```

Common maps:

- `azeroth` - Eastern Kingdoms
- `kalimdor` - Kalimdor
- `northrend` - Northrend
- `pandaria` - Pandaria
