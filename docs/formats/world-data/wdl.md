# WDL Format 🌍

WDL (World Data Low-resolution) files contain low-detail heightmap and water data for entire continents in World of Warcraft. These files are part of the terrain Level of Detail (LoD) system, providing efficient rendering of distant terrain and supporting world map generation.

## Overview

- **Extension**: `.wdl`
- **Purpose**: Low-resolution terrain for distant viewing and world maps
- **Coverage**: Entire continent in one file
- **Resolution**: 17x17 height values per ADT tile
- **Use Case**: Map view, flight paths, distant terrain, minimap generation
- **Format**: Chunk-based binary format (similar to other Blizzard formats)

## Version History

| Version | First Appearance | Notable Changes | WoW Version |
|---------|-----------------|-----------------|-------------|
| 18 | Classic WoW | Initial format | 1.x |
| 19 | The Burning Crusade | Enhanced data structure | 2.x |
| 20 | Wrath of the Lich King | Additional chunk types | 3.x |
| 21+ | Later Expansions | Format refinements | 4.x+ |

⚠️ **Note**: Version history requires validation against actual game files from different expansions.

## File Structure

WDL files follow the standard Blizzard chunk-based format:

```
[File Header]
[Chunk 1: Header + Data]
[Chunk 2: Header + Data]
[...]
[Chunk N: Header + Data]
```

### Chunk Header Structure

```rust
#[repr(C, packed)]
struct ChunkHeader {
    /// Four-character chunk identifier (e.g., b"MVER", b"MAOF")
    magic: [u8; 4],

    /// Size of the chunk data (not including this header)
    size: u32,
}
```

## Main Chunks

| Chunk | Size | Description | Required |
|-------|------|-------------|----------|
| MVER | 4 | Version number | ✅ |
| MAOF | Variable | Map area offset table | ✅ |
| MARE | Variable | Map area data (terrain) | ✅ |
| MAHO | Variable | Map area holes | ❌ |
| MWMO | Variable | WMO placement info | ❌ |
| MWID | Variable | WMO indices | ❌ |
| MODF | Variable | WMO placement data | ❌ |

### MVER - Version Chunk

Always appears first in the file:

```rust
struct MverChunk {
    header: ChunkHeader,  // magic = b"MVER", size = 4
    version: u32,         // Format version (18-21+)
}
```

### MAOF - Area Offset Chunk

Contains offset information for area data within the file:

```rust
struct AreaOffset {
    offset: u32,  // Offset to area data within the file
    size: u32,    // Size of the area data
}

struct MaofChunk {
    header: ChunkHeader,  // magic = b"MAOF"
    // Array of area offsets
    // Length = header.size / sizeof(AreaOffset)
}
```

### MARE - Area Information Chunk

Contains the actual low-resolution terrain data:

⚠️ **Note**: The internal structure of MARE chunks is not fully documented and requires reverse engineering.

```rust
struct MareChunk {
    header: ChunkHeader,  // magic = b"MARE"
    // Contains:
    // - Height map data at low resolution
    // - Texture information
    // - Possibly color/lighting data
    // - Area identification information
}
```

### Height Data Structure (Tentative)

```rust
struct WdlHeight {
    base_height: i16,           // Base terrain height
    height_map: [[i8; 17]; 17], // Height offsets from base
    unknown: [[i8; 16]; 16],    // Unknown data (possibly normals)
}

struct WdlWater {
    height_level: i16,          // Water surface height
    height_map: [[i8; 17]; 17], // Water depth values
}
```

## Coordinate System

WDL files use World of Warcraft's standard coordinate system:

- **World Units**: 1 yard = 1 coordinate unit
- **Coverage**: 64x64 ADT tiles per continent
- **Tile Size**: 533.33333 units (needs verification)
- **Origin Offset**: 17066.666 units (needs verification)
- **Total Resolution**: ~1024x1024 height values for entire continent

### Coordinate Conversion

```rust
/// Convert world coordinates to WDL tile coordinates
pub fn world_to_wdl_coords(world_x: f32, world_y: f32) -> (u32, u32) {
    const TILE_SIZE: f32 = 533.33333;
    const ORIGIN_OFFSET: f32 = 17066.666;

    let tile_x = ((ORIGIN_OFFSET - world_x) / TILE_SIZE) as u32;
    let tile_y = ((ORIGIN_OFFSET - world_y) / TILE_SIZE) as u32;

    (tile_x, tile_y)
}

/// Convert WDL tile coordinates back to world coordinates
pub fn wdl_to_world_coords(tile_x: u32, tile_y: u32) -> (f32, f32) {
    const TILE_SIZE: f32 = 533.33333;
    const ORIGIN_OFFSET: f32 = 17066.666;

    let world_x = ORIGIN_OFFSET - (tile_x as f32 * TILE_SIZE);
    let world_y = ORIGIN_OFFSET - (tile_y as f32 * TILE_SIZE);

    (world_x, world_y)
}
```

## Level of Detail System

WDL files are part of WoW's LoD hierarchy:

```
WDT (World Directory Table)
├── ADT (Area Data Table) - High detail, close terrain
└── WDL (World LoD) - Low detail, distant terrain
```

### Usage Patterns

1. **Distance-Based Switching**: Engine switches between ADT and WDL based on camera distance
2. **World Map Generation**: WDL data generates world map imagery
3. **Minimap Support**: Low-resolution data for minimap rendering
4. **Memory Optimization**: Allows unloading high-detail ADT data when not needed

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

### LoD Management

```rust
pub struct LodManager {
    wdt_data: WdtFile,
    loaded_adts: HashMap<(u32, u32), AdtFile>,
    wdl_data: WdlFile,
}

impl LodManager {
    /// Determine which data to use based on distance
    pub fn get_terrain_data(&self, world_pos: (f32, f32), camera_distance: f32) -> TerrainData {
        const LOD_SWITCH_DISTANCE: f32 = 1000.0;

        if camera_distance < LOD_SWITCH_DISTANCE {
            // Use high-detail ADT data
            self.get_adt_data(world_pos)
        } else {
            // Use low-detail WDL data
            self.get_wdl_data(world_pos)
        }
    }
}
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

## Implementation Considerations

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum WdlError {
    #[error("Invalid chunk magic: {0:?}")]
    InvalidChunkMagic([u8; 4]),

    #[error("Invalid chunk size: {0}")]
    InvalidChunkSize(u32),

    #[error("Unsupported version: {0}")]
    UnsupportedVersion(u32),

    #[error("Chunk not found: {0:?}")]
    ChunkNotFound([u8; 4]),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid file format")]
    InvalidFormat,
}
```

### Memory Management

```rust
/// WDL file representation optimized for memory usage
pub struct WdlFile {
    version: u32,
    chunks: HashMap<[u8; 4], ChunkData>,
}

/// Chunk data with lazy loading support
pub enum ChunkData {
    /// Raw chunk data (not parsed)
    Raw(Vec<u8>),
    /// Parsed chunk data
    Parsed(Box<dyn ChunkContent>),
}
```

## Performance Tips

- WDL files are relatively small (~1-2 MB)
- Can be kept in memory for entire session
- Use for distant terrain LOD switching
- Ideal for minimap rendering
- Cache frequently accessed height data
- Use bilinear interpolation for smooth transitions

## Common Issues

### Height Precision

- WDL uses 8-bit height offsets
- Less precise than ADT heightmaps
- Suitable for distant viewing only
- May show stepping artifacts up close

### Water Detection

- Not all water bodies are represented
- Small ponds/streams may be missing
- Ocean height is typically 0.0
- Water data may be incomplete

### Coordinate System

- Exact tile size and origin offset need verification
- Coordinate conversion formulas may vary by continent
- Edge cases at continent boundaries

## Known Limitations and Research Gaps

### Critical Research Areas

1. **MARE Chunk Structure**: Internal format needs reverse engineering
2. **Height Data Format**: Exact encoding of height information unknown
3. **Texture Mapping**: How low-resolution textures are stored/referenced
4. **Version Differences**: Changes between game versions not fully documented
5. **Coordinate Precision**: Exact parameters need verification

### Implementation Challenges

1. **Reverse Engineering Required**: Most technical details need verification
2. **Version Compatibility**: Supporting multiple game versions
3. **Performance Requirements**: Real-time terrain rendering demands
4. **Memory Constraints**: Efficient loading and caching strategies

## References

- [WDL Format (wowdev.wiki)](https://wowdev.wiki/WDL)
- [Map Coordinates System](https://wowdev.wiki/Map_coordinates)
- [WowDevTools libwarcraft](https://github.com/WowDevTools/libwarcraft) - .NET implementation
- [pywowlib](https://github.com/wowdev/pywowlib) - Python implementation

## See Also

- [ADT Format](adt.md) - High-detail terrain data
- [WDT Format](wdt.md) - World definition tables
- [Coordinate System](../../resources/coordinates.md) - Detailed coordinate information
- [LoD System Guide](../../guides/lod-system.md) - Level of Detail implementation
