# WDL Format 🌍

WDL (World Data Low-resolution) files contain low-detail heightmap and water data
for entire continents.

## Overview

- **Extension**: `.wdl`
- **Purpose**: Low-resolution terrain for distant viewing
- **Coverage**: Entire continent in one file
- **Resolution**: 17x17 height values per ADT tile
- **Use Case**: Map view, flight paths, distant terrain

## Structure

### Main Chunks

| Chunk | Size | Description |
|-------|------|-------------|
| MVER | 4 | Version (always 18) |
| MWMO | Variable | WMO placement info |
| MWID | Variable | WMO indices |
| MODF | Variable | WMO placement data |
| MAOF | 4096*4 | Map area offset |
| MARE | Variable | Map area data |
| MAHO | Variable | Map area holes |

### Height Data

```rust
struct WdlHeight {
    base_height: i16,        // Base terrain height
    height_map: [[i8; 17]; 17], // Height offsets from base
    unknown: [[i8; 16]; 16],    // Unknown data (possibly normals)
}

struct WdlWater {
    height_level: i16,       // Water surface height
    height_map: [[i8; 17]; 17], // Water depth values
}
```

## Usage Example

```rust
use warcraft_rs::wdl::{Wdl, HeightQuery};

// Load WDL file
let wdl = Wdl::open("World/Maps/Azeroth/Azeroth.wdl")?;

// Query height at world coordinates
let height = wdl.get_height(1234.5, 5678.9)?;

// Get water level
if let Some(water_height) = wdl.get_water_height(1234.5, 5678.9)? {
    println!("Water at height: {}", water_height);
}

// Export low-res heightmap for entire continent
let heightmap = wdl.export_heightmap();
heightmap.save("continent_heightmap.png")?;

// Check which ADT tiles have data
for x in 0..64 {
    for y in 0..64 {
        if wdl.has_tile(x, y) {
            println!("ADT {}_{} exists", x, y);
        }
    }
}
```

## Coordinate System

- Covers 64x64 ADT tiles
- Each ADT represented by 17x17 height points
- Total resolution: 1024x1024 height values
- Height scale factor: 2.0 units

## Advanced Features

### Minimap Generation

```rust
use warcraft_rs::wdl::MinimapGenerator;

let generator = MinimapGenerator::new(&wdl);
generator.set_water_color(Color::rgba(0, 100, 200, 128));
generator.set_terrain_gradient(TerrainGradient::realistic());

let minimap = generator.generate(2048, 2048)?;
minimap.save("world_minimap.png")?;
```

### Height Interpolation

```rust
// Bilinear interpolation for smooth height queries
let smooth_height = wdl.get_height_interpolated(x, y)?;

// Get terrain normal at position
let normal = wdl.get_normal(x, y)?;
```

## Common Patterns

### Map View Implementation

```rust
struct MapView {
    wdl: Wdl,
    zoom_level: f32,
    center: Vec2,
}

impl MapView {
    fn render(&self, viewport: &Viewport) -> Image {
        let bounds = self.calculate_visible_bounds(viewport);

        for x in bounds.min_x..bounds.max_x {
            for y in bounds.min_y..bounds.max_y {
                let height = self.wdl.get_height(x, y)?;
                let color = self.height_to_color(height);
                // Draw pixel
            }
        }
    }
}
```

### Flight Path Validation

```rust
// Ensure flight path stays above terrain
fn validate_flight_path(wdl: &Wdl, path: &[Vec3]) -> Result<()> {
    let min_clearance = 50.0;

    for point in path {
        let terrain_height = wdl.get_height(point.x, point.y)?;
        if point.z < terrain_height + min_clearance {
            return Err("Flight path too low");
        }
    }
    Ok(())
}
```

## Performance Tips

- WDL files are relatively small (~1-2 MB)
- Can be kept in memory for entire session
- Use for distant terrain LOD switching
- Ideal for minimap rendering

## Common Issues

### Height Precision

- WDL uses 8-bit height offsets
- Less precise than ADT heightmaps
- Suitable for distant viewing only

### Water Detection

- Not all water bodies are represented
- Small ponds/streams may be missing
- Ocean height is typically 0.0

## References

- [WDL Format (wowdev.wiki)](https://wowdev.wiki/WDL)
- [Map Coordinates System](https://wowdev.wiki/Map_coordinates)

## See Also

- [ADT Format](adt.md) - High-detail terrain data
- [WDT Format](wdt.md) - World definition tables
- [Minimap Generation Guide](../../guides/minimap-generation.md)
