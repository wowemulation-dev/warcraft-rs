# WDT Format 🗺️

WDT (World Definition Table) files define which ADT tiles exist for a map and
contain global map settings.

## Overview

- **Extension**: `.wdt`
- **Purpose**: Map metadata and ADT tile existence flags
- **Grid**: 64x64 possible ADT tiles
- **Versions**: Classic, TBC, WotLK variations
- **Related**: Works with ADT files to form complete maps

## Structure

### Main Chunks

| Chunk | Size | Description |
|-------|------|-------------|
| MVER | 4 | Version |
| MPHD | 32 | Map header with flags |
| MAIN | 32768 | ADT existence and flags |
| MWMO | Variable | WMO-only maps |
| MODF | Variable | WMO placement (if WMO-only) |

### Map Header (MPHD)

```rust
struct MapHeader {
    flags: u32,              // Global map flags
    unknown: u32,
    unused: [u32; 6],
}

// Map flags
const WDT_USE_GLOBAL_MAP_OBJ: u32 = 0x0001;
const WDT_NO_ADT_ALPHA: u32 = 0x0002;
const WDT_HEIGHT_TEXTURING: u32 = 0x0004;
const WDT_BIG_ALPHA: u32 = 0x0008;
```

### ADT Information (MAIN)

```rust
struct AdtInfo {
    exists: u32,    // 0 = doesn't exist, 1 = exists
    flags: u32,     // Per-ADT flags
}

struct MainChunk {
    adt_info: [[AdtInfo; 64]; 64], // 64x64 grid
}
```

## Usage Example

```rust
use warcraft_rs::wdt::{Wdt, MapFlags};

// Load WDT file
let wdt = Wdt::open("World/Maps/Azeroth/Azeroth.wdt")?;

// Check map properties
if wdt.has_flag(MapFlags::UseGlobalWmo) {
    println!("This is a WMO-only map");
}

// Iterate over existing ADT tiles
for (x, y) in wdt.existing_tiles() {
    println!("ADT exists at ({}, {})", x, y);

    // Construct ADT filename
    let adt_path = format!("World/Maps/Azeroth/Azeroth_{}_{}.adt", x, y);
    // Load ADT...
}

// Get map bounds
let bounds = wdt.get_map_bounds();
println!("Map spans from ({}, {}) to ({}, {})",
    bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y);

// Check if specific tile exists
if wdt.tile_exists(32, 48) {
    // Load that specific ADT
}
```

## Map Types

### Standard World Maps

Most common type with terrain ADT files:

```rust
fn load_world_map(map_name: &str) -> Result<WorldMap> {
    let wdt = Wdt::open(&format!("World/Maps/{0}/{0}.wdt", map_name))?;

    let mut adts = HashMap::new();
    for (x, y) in wdt.existing_tiles() {
        let adt = Adt::open(&format!("World/Maps/{0}/{0}_{1}_{2}.adt",
            map_name, x, y))?;
        adts.insert((x, y), adt);
    }

    Ok(WorldMap { wdt, adts })
}
```

### WMO-Only Maps

Instance maps with single WMO object:

```rust
if wdt.is_wmo_only() {
    let wmo_name = wdt.get_global_wmo()?;
    let wmo = Wmo::open(&format!("World/wmo/{}.wmo", wmo_name))?;

    // Instance map uses only this WMO
}
```

## Advanced Features

### Map Streaming

```rust
struct MapStreamer {
    wdt: Wdt,
    loaded_adts: LruCache<(u32, u32), Adt>,
    view_distance: u32,
}

impl MapStreamer {
    fn update(&mut self, player_tile: (u32, u32)) -> Result<()> {
        let (px, py) = player_tile;

        // Load ADTs within view distance
        for x in px.saturating_sub(self.view_distance)..=px + self.view_distance {
            for y in py.saturating_sub(self.view_distance)..=py + self.view_distance {
                if self.wdt.tile_exists(x, y) && !self.loaded_adts.contains(&(x, y)) {
                    self.load_adt(x, y)?;
                }
            }
        }

        // TODO: Unload distant ADTs
        Ok(())
    }
}
```

### Map Validation

```rust
impl Wdt {
    fn validate(&self) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();

        // Check for isolated tiles
        for (x, y) in self.existing_tiles() {
            let neighbors = self.count_neighbors(x, y);
            if neighbors == 0 {
                report.add_warning(format!("Isolated tile at ({}, {})", x, y));
            }
        }

        // Check map connectivity
        if !self.is_connected() {
            report.add_error("Map has disconnected regions");
        }

        Ok(report)
    }
}
```

## Common Patterns

### Minimap Generation

```rust
fn generate_minimap(wdt: &Wdt, size: u32) -> Image {
    let mut image = Image::new(size, size);
    let bounds = wdt.get_map_bounds();

    for (x, y) in wdt.existing_tiles() {
        let pixel_x = ((x - bounds.min_x) * size) / bounds.width();
        let pixel_y = ((y - bounds.min_y) * size) / bounds.height();

        image.set_pixel(pixel_x, pixel_y, Color::GREEN);
    }

    image
}
```

### Multi-Resolution Loading

```rust
struct MultiResMap {
    wdt: Wdt,
    wdl: Wdl,
    nearby_adts: HashMap<(u32, u32), Adt>,
}

impl MultiResMap {
    fn get_height(&self, world_x: f32, world_y: f32, distance: f32) -> f32 {
        if distance < 1000.0 {
            // Use high-res ADT data
            self.get_adt_height(world_x, world_y)
        } else {
            // Use low-res WDL data
            self.wdl.get_height(world_x, world_y)
        }
    }
}
```

## Performance Considerations

- WDT files are small (typically < 100KB)
- Keep in memory for entire map session
- Use to avoid loading non-existent ADTs
- Pre-calculate map bounds and connectivity

## Common Issues

### Missing Tiles

- Not all 64x64 grid positions have ADTs
- Ocean areas typically have no tiles
- Use `tile_exists()` before loading ADTs

### Version Differences

- Flag meanings vary between WoW versions
- Some chunks added in expansions
- Check version field for compatibility

## References

- [WDT Format (wowdev.wiki)](https://wowdev.wiki/WDT)
- [ADT/WDT Grid System](https://wowdev.wiki/ADT/WDT)

## See Also

- [ADT Format](adt.md) - Terrain tile format
- [WDL Format](wdl.md) - Low-resolution world data
- [Map Loading Guide](../../guides/map-loading.md)
