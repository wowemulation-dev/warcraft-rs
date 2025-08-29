# WDL Format 🌍

WDL (World Data Low-resolution) files contain low-detail heightmap and water data
for entire continents in World of Warcraft. These files are part of the terrain
Level of Detail (LoD) system, providing efficient rendering of distant terrain
and supporting world map generation.

## Overview

- **Extension**: `.wdl` - ✅ Implemented
- **Purpose**: Low-resolution terrain for distant viewing and world maps - ✅ Implemented
- **Coverage**: Entire continent in one file - ✅ Implemented
- **Resolution**: 17x17 height values per ADT tile - ✅ Implemented
- **Use Case**: Map view, flight paths, distant terrain, minimap generation - ⚠️ Parsing Implemented, Rendering Not Implemented
- **Format**: Chunk-based binary format (similar to other Blizzard formats) - ✅ Implemented

## Version History

Based on analysis of WDL files from WoW versions 1.12.1 through 5.4.8:

| Version | WoW Versions | Notes |
|---------|--------------|-------|
| 18 | All versions (1.12.1 - 5.4.8) | Consistent across all tested versions |

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

## Chunk Evolution Timeline

Based on empirical analysis of WDL files across WoW versions:

### Classic (1.12.1)
- **Core chunks**: MVER, MAOF, MARE
- **MAOF size**: 16384 bytes (64×64×4 bytes)
- **MARE size**: 1090 bytes (when present)

### The Burning Crusade (2.4.3)
- **New chunk**: MAHO (height holes/occlusion)
- **MAHO size**: Variable (typically 32-2176 bytes)
- All other chunks unchanged

### Wrath of the Lich King (3.3.5a)
- No new chunks
- MAHO more commonly present
- Structure remains stable

### Cataclysm (4.3.4) - MAJOR CHANGES
- **New chunks**: MWID, MWMO, MODF
- **MWID**: WMO instance IDs (often 0 bytes)
- **MWMO**: WMO filenames (often 0 bytes)
- **MODF**: WMO placement data (often 0 bytes)
- Support for WMO references in low-res world

### Mists of Pandaria (5.4.8)
- No structural changes from Cataclysm
- Same chunk set as 4.3.4

## Main Chunks

| Chunk | Size | Description | First Seen | Required |
|-------|------|-------------|------------|----------|
| MVER | 4 | Version number (always 18) | 1.12.1 | ✅ |
| MAOF | 16384 | Map area offset table (64×64×4) | 1.12.1 | ✅ |
| MARE | 1090 | Map area terrain heights | 1.12.1 | ❌ |
| MAHO | 32-2176 | Map area height holes | 2.4.3 | ❌ |
| MWID | Variable | WMO instance IDs | 4.3.4 | ❌ |
| MWMO | Variable | WMO filenames | 4.3.4 | ❌ |
| MODF | Variable | WMO placement data | 4.3.4 | ❌ |

### MVER - Version Chunk

Always appears first in the file:

```rust
struct MverChunk {
    header: ChunkHeader,  // magic = b"MVER", size = 4
    version: u32,         // Always 18 in all tested versions (1.12.1-5.4.8)
}
```

### MAOF - Area Offset Chunk

Contains offset information for area data. Always 16384 bytes (64×64×4):

```rust
struct MaofChunk {
    header: ChunkHeader,  // magic = b"MAOF", size = 16384
    offsets: [[u32; 64]; 64],  // 64×64 grid of offsets
}
```

**Notes**:
- Most entries are zero (86-100% zeros in tested files)
- Non-zero values indicate tiles with terrain data
- Grid corresponds to ADT tile layout

### MARE - Area Information Chunk

Contains low-resolution heightmap data. Consistently 1090 bytes when present:

```rust
struct MareChunk {
    header: ChunkHeader,  // magic = b"MARE", size = 1090
    data: [u8; 1090],     // Height and area data
}
```

**Structure** (preliminary analysis):
- Contains 17×17 height grid per tile (545 int16 values = 1090 bytes)
- Provides low-resolution heightmap for distant terrain
- Not present in all WDL files (only maps with terrain)

### MAHO - Map Area Height Occlusion (TBC+)

Added in The Burning Crusade, contains height hole/occlusion data:

```rust
struct MahoChunk {
    header: ChunkHeader,  // magic = b"MAHO", size varies (32-2176 bytes)
    data: Vec<i16>,       // Height occlusion values
}
```

**Notes**:
- Size varies based on map complexity
- Often contains many zero values
- Used for occlusion culling optimization

### MWID - WMO Instance IDs (Cataclysm+)

Added in Cataclysm, maps WMO instances:

```rust
struct MwidChunk {
    header: ChunkHeader,  // magic = b"MWID", size varies (often 0)
    wmo_ids: Vec<u32>,    // WMO instance IDs
}
```

### MWMO - WMO Filenames (Cataclysm+)

Added in Cataclysm, contains WMO filename strings:

```rust
struct MwmoChunk {
    header: ChunkHeader,  // magic = b"MWMO", size varies (often 0)
    filenames: Vec<CString>,  // Null-terminated WMO filenames
}
```

### MODF - WMO Placement Data (Cataclysm+)

Added in Cataclysm, defines WMO positions:

```rust
struct ModfChunk {
    header: ChunkHeader,  // magic = b"MODF", size varies (often 0)
    entries: Vec<ModfEntry>,  // WMO placement entries (64 bytes each)
}

struct ModfEntry {
    id: u32,              // Index into MWMO
    unique_id: u32,       // Unique instance ID
    position: [f32; 3],   // X, Y, Z position
    rotation: [f32; 3],   // X, Y, Z rotation (radians)
    lower_bounds: [f32; 3],  // Bounding box min
    upper_bounds: [f32; 3],  // Bounding box max
    flags: u16,           // WMO flags
    doodad_set: u16,      // Doodad set index
    name_set: u16,        // Name set index
    scale: u16,           // Scale factor (1024 = 1.0)
}
```

**Notes**:
- Often empty (0 bytes) in WDL files
- When present, uses same structure as ADT MODF chunks
- Primarily for major landmarks visible from distance

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

## Usage Example - ✅ Implemented

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

## Implementation Considerations - ✅ Implemented

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
