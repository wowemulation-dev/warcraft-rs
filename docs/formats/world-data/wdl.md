# WDL Format 🌍

WDL (World Data Low-resolution) files contain low-detail heightmap and water data
for entire continents in World of Warcraft. These files are part of the terrain
Level of Detail (LoD) system, providing efficient rendering of distant terrain
and supporting world map generation.

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

⚠️ **Note**: Version history requires validation against actual game files from
different expansions.

## File Structure

WDL files follow the standard Blizzard chunk-based format:

```text
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

⚠️ **Note**: The internal structure of MARE chunks is not fully documented and
requires reverse engineering.

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

```text
WDT (World Directory Table)
├── ADT (Area Data Table) - High detail, close terrain
└── WDL (World LoD) - Low detail, distant terrain
```

### Usage Patterns

1. **Distance-Based Switching**: Engine switches between ADT and WDL based on
   camera distance
2. **World Map Generation**: WDL data generates world map imagery
3. **Minimap Support**: Low-resolution data for minimap rendering
4. **Memory Optimization**: Allows unloading high-detail ADT data when not needed

## Usage Example

```rust
use std::fs::File;
use std::io::BufReader;
use wow_wdl::parser::WdlParser;

// Open a WDL file
let file = File::open("World/Maps/Azeroth/Azeroth.wdl")?;
let mut reader = BufReader::new(file);

// Parse the file
let parser = WdlParser::new();
let wdl_file = parser.parse(&mut reader)?;

// Use the data
println!("WDL version: {}", wdl_file.version);
println!("Map tiles: {}", wdl_file.heightmap_tiles.len());

// Get heightmap for a specific tile
if let Some(tile) = wdl_file.heightmap_tiles.get(&(32, 32)) {
    println!("Tile 32,32 has {} outer height values", tile.outer_values.len());
    println!("Tile 32,32 has {} inner height values", tile.inner_values.len());
}

// Check which ADT tiles have data
for ((x, y), _) in &wdl_file.heightmap_tiles {
    println!("ADT {}_{} has heightmap data", x, y);
}

// Check for holes data
if let Some(holes) = wdl_file.holes_data.get(&(32, 32)) {
    for row in 0..16 {
        for col in 0..16 {
            if holes.has_hole(row, col) {
                println!("Hole at tile 32,32 position ({}, {})", row, col);
            }
        }
    }
}
```

## Advanced Features

### Version Conversion

```rust
use wow_wdl::parser::WdlParser;
use wow_wdl::version::WdlVersion;
use wow_wdl::conversion::convert_wdl_file;
use std::fs::File;
use std::io::{BufReader, BufWriter};

// Parse an existing file
let file = File::open("input.wdl")?;
let mut reader = BufReader::new(file);
let parser = WdlParser::new();
let wdl_file = parser.parse(&mut reader)?;

// Convert to Legion version
let legion_file = convert_wdl_file(&wdl_file, WdlVersion::Legion)?;

// Save the converted file
let output = File::create("output.wdl")?;
let mut writer = BufWriter::new(output);
let legion_parser = WdlParser::with_version(WdlVersion::Legion);
legion_parser.write(&mut writer, &legion_file)?;
```

### Creating New WDL Files

```rust
use wow_wdl::types::{WdlFile, HeightMapTile, HolesData};
use wow_wdl::version::WdlVersion;
use wow_wdl::parser::WdlParser;
use std::io::Cursor;

// Create a new WDL file with WotLK version
let mut file = WdlFile::with_version(WdlVersion::Wotlk);

// Add a heightmap tile
let mut heightmap = HeightMapTile::new();
for i in 0..HeightMapTile::OUTER_COUNT {
    heightmap.outer_values[i] = (i as i16) % 100;
}
for i in 0..HeightMapTile::INNER_COUNT {
    heightmap.inner_values[i] = ((i + 100) as i16) % 100;
}
file.heightmap_tiles.insert((10, 20), heightmap);

// Add holes data
let mut holes = HolesData::new();
holes.set_hole(5, 7, true);
holes.set_hole(8, 9, true);
file.holes_data.insert((10, 20), holes);

// Write the file
let parser = WdlParser::with_version(WdlVersion::Wotlk);
let mut buffer = Vec::new();
let mut cursor = Cursor::new(&mut buffer);
parser.write(&mut cursor, &file)?;
```

### Working with WMO Placements

```rust
use wow_wdl::types::{ModelPlacement, Vec3d, BoundingBox};

// Add WMO data to WDL file
let mut wdl_file = WdlFile::with_version(WdlVersion::Wotlk);

// Add a WMO filename and index
wdl_file.wmo_filenames.push("World/wmo/Azeroth/Buildings/Human_Farm/Farm.wmo".to_string());
wdl_file.wmo_indices.push(0);

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
wdl_file.wmo_placements.push(placement);
```

## Common Patterns

### Iterating Over Heightmap Data

```rust
use wow_wdl::types::WdlFile;

fn analyze_heightmap(wdl_file: &WdlFile) {
    // Iterate over all tiles with heightmap data
    for ((tile_x, tile_y), heightmap) in &wdl_file.heightmap_tiles {
        println!("Tile ({}, {}):", tile_x, tile_y);

        // Find min/max heights in the tile
        let mut min_height = i16::MAX;
        let mut max_height = i16::MIN;

        for &height in &heightmap.outer_values {
            min_height = min_height.min(height);
            max_height = max_height.max(height);
        }

        for &height in &heightmap.inner_values {
            min_height = min_height.min(height);
            max_height = max_height.max(height);
        }

        println!("  Height range: {} to {}", min_height, max_height);
    }
}
```

### Checking for Data Coverage

```rust
use wow_wdl::types::WdlFile;

fn check_continent_coverage(wdl_file: &WdlFile) {
    let mut covered_tiles = 0;
    let mut total_tiles = 0;

    // Check all possible tile positions (64x64 grid)
    for x in 0..64 {
        for y in 0..64 {
            total_tiles += 1;
            if wdl_file.heightmap_tiles.contains_key(&(x, y)) {
                covered_tiles += 1;
            }
        }
    }

    let coverage = (covered_tiles as f32 / total_tiles as f32) * 100.0;
    println!("Continent coverage: {:.1}% ({}/{} tiles)", coverage, covered_tiles, total_tiles);

    // Check which tiles have holes
    let mut tiles_with_holes = 0;
    for (coord, _) in &wdl_file.holes_data {
        if wdl_file.heightmap_tiles.contains_key(coord) {
            tiles_with_holes += 1;
        }
    }

    println!("Tiles with hole data: {}", tiles_with_holes);
}
```

## Implementation Considerations

### Error Handling

```rust
use wow_wdl::{WdlError, Result};
use std::fs::File;
use std::io::BufReader;
use wow_wdl::parser::WdlParser;

fn safe_wdl_load(path: &str) -> Result<wow_wdl::WdlFile> {
    match File::open(path) {
        Ok(file) => {
            let mut reader = BufReader::new(file);
            let parser = WdlParser::new();

            match parser.parse(&mut reader) {
                Ok(wdl) => {
                    println!("Successfully loaded WDL version {}", wdl.version);
                    Ok(wdl)
                }
                Err(WdlError::UnsupportedVersion(ver)) => {
                    eprintln!("WDL version {} is not supported", ver);
                    Err(WdlError::UnsupportedVersion(ver))
                }
                Err(e) => {
                    eprintln!("Failed to parse WDL: {}", e);
                    Err(e)
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open file: {}", e);
            Err(WdlError::Io(e))
        }
    }
}
```

### Validation

```rust
use wow_wdl::validation::{validate_wdl_file, ValidationError};
use wow_wdl::types::WdlFile;

fn validate_wdl_data(wdl_file: &WdlFile) -> std::result::Result<(), ValidationError> {
    // Use the built-in validation
    validate_wdl_file(wdl_file)?;

    // Additional custom validation
    if wdl_file.heightmap_tiles.is_empty() {
        return Err(ValidationError::MissingRequiredData("No heightmap tiles found".into()));
    }

    // Validate tile coordinates are within bounds
    for ((x, y), _) in &wdl_file.heightmap_tiles {
        if *x >= 64 || *y >= 64 {
            return Err(ValidationError::InvalidCoordinates(*x, *y));
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
