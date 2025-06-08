# WDT Format 🗺️

WDT (World Data Table) files are fundamental components of World of Warcraft's
world rendering system. They serve as master indexes that define which map tiles
(ADT files) exist in a world and can optionally reference a global World Map
Object (WMO) for WMO-only maps like instances.

## Overview

- **Extension**: `.wdt` (main file) and multiple auxiliary files
- **Magic**: Chunk-based format with standard IFF structure
- **Purpose**: Map tile presence, global WMO reference, map properties, lighting,
  fog, and occlusion data
- **Grid**: 64×64 possible ADT tiles
- **Version**: Always 18 for WDT files
- **Related**: Works with ADT files to form complete maps

## Key Features

1. **Map Tile Presence**: Define which of the potential 64×64 ADT tiles actually
   exist for a given map
2. **Global WMO Reference**: For indoor/instance maps, reference a single global
   WMO
3. **Map Properties**: Store various flags and properties that affect how the
   entire map is rendered
4. **Lighting Information**: Define global lighting properties and light sources
   (_lgt.wdt)
5. **Fog Effects**: Control volumetric fog and atmospheric effects (_fogs.wdt)
6. **Occlusion Data**: Provide low-resolution occlusion information for improved
   rendering performance (_occ.wdt)

## Coordinate Systems

World of Warcraft uses different coordinate systems for different purposes:

### ADT/WDT Terrain Coordinate System

- **Right-handed coordinate system**
- **X-axis**: Points North (decreasing tile Y)
- **Y-axis**: Points West (decreasing tile X)
- **Z-axis**: Points Up (vertical height)
- **Origin**: Map center at tile [32, 32]
- **Range**: ±17066.666 world units on X and Y axes

### MODF Placement Coordinate System

Placement chunks use a different orientation:

- **X-axis**: Points West
- **Y-axis**: Points Up (vertical)
- **Z-axis**: Points North

**Important transformations for MODF placement**:

```rust
// Convert from placement coordinates to world coordinates
let world_x = 32.0 * TILESIZE - placement_x;
let world_z = 32.0 * TILESIZE - placement_z;
let world_y = placement_y; // Y (up) remains the same
```

## File Structure

WDT files follow the standard chunked format:

```rust
struct IffChunk {
    magic: [u8; 4],    // Chunk identifier (e.g., "MVER", "MPHD")
    size: u32,         // Size of chunk data in bytes
    data: Vec<u8>,     // Chunk-specific data
}
```

### Typical Chunk Order

For main WDT files:

1. **MVER** - Version information (always first)
2. **MPHD** - Map header with flags and file references
3. **MAIN** - Map tile presence information
4. **MAID** - FileDataIDs for ADT files (post-8.1)
5. **MWMO** - Global WMO filename (WMO-only maps, or empty in pre-4.x terrain maps)
6. **MODF** - Global WMO placement (WMO-only maps)

## Chunk Specifications

### MVER - Version

The version chunk is always the first chunk in the file.

```rust
struct MVER {
    version: u32,  // Always 18 for WDT files
}
```

### MPHD - Map Header

Contains global flags and references to other map-related files.

```rust
struct MPHD {
    flags: u32,              // See flag definitions below

    // Pre-8.1.0 (Classic through Legion):
    something: u32,          // Unknown purpose
    unused: [u32; 6],        // Reserved (always 0)
    // Total size: 32 bytes

    // Post-8.1.0 (BfA+):
    // The 7 uint32 fields above are repurposed as FileDataIDs:
    lgt_file_data_id: u32,   // _lgt.wdt lighting file
    occ_file_data_id: u32,   // _occ.wdt occlusion file
    fogs_file_data_id: u32,  // _fogs.wdt fog file
    mpv_file_data_id: u32,   // _mpv.wdt particulate volume file
    tex_file_data_id: u32,   // _tex.wdt texture file
    wdl_file_data_id: u32,   // _wdl low-resolution heightmap
    pd4_file_data_id: u32,   // _pd4.wdt file
}
```

#### MPHD Flags

```rust
enum MphdFlags {
    WdtUsesGlobalMapObj              = 0x0001,  // Map is WMO-only (instance/indoor)
    AdtHasMccv                       = 0x0002,  // ADTs have vertex colors
    AdtHasBigAlpha                   = 0x0004,  // Alternative terrain shader
    AdtHasDoodadrefsSortedBySizeCat  = 0x0008,  // Doodads sorted by size
    AdtHasLightingVertices           = 0x0010,  // ADTs have MCLV chunk (deprecated in 8.x)
    AdtHasUpsideDownGround           = 0x0020,  // Flip ground display
    UnkFirelands                     = 0x0040,  // Universal in 4.3.4+ (all maps have this)
    AdtHasHeightTexturing            = 0x0080,  // Use _h textures
    UnkLoadLod                       = 0x0100,  // Load _lod.adt files
    WdtHasMaid                       = 0x0200,  // Has MAID chunk with FileDataIDs (8.1.0+)
    // Flags 0x0400 through 0x8000 are unknown/reserved
}
```

### MAIN - Map Area Information

Defines which ADT tiles exist in the 64×64 grid.

```rust
struct MAIN {
    entries: [[MainEntry; 64]; 64],  // [y][x] ordering (row-major)
}

struct MainEntry {
    flags: u32,     // See flag definitions below
    area_id: u32,   // AreaTable.dbc ID (async loading in 0.5.3+)
}

enum MainFlags {
    HasAdt      = 0x0001,  // ADT file exists for this tile
    IsLoaded    = 0x0002,  // Set at runtime when ADT is loaded (never in file)
    AllWater    = 0x0002,  // Special flag for all-water tiles (runtime only)
    IsImported  = 0x0004,  // Marks imported tiles (runtime only)
}
```

### MAID - Map Area ID

Introduced in 8.1.0, contains FileDataIDs for all map files.

```rust
struct MAID {
    // Each section contains 64x64 entries (4096 uint32 values)
    // Stored in [y][x] order (row-major)

    root_adt: [[u32; 64]; 64],        // FileDataIDs for root ADT files
    obj0_adt: [[u32; 64]; 64],        // FileDataIDs for _obj0.adt files
    obj1_adt: [[u32; 64]; 64],        // FileDataIDs for _obj1.adt files
    tex0_adt: [[u32; 64]; 64],        // FileDataIDs for _tex0.adt files
    lod_adt: [[u32; 64]; 64],         // FileDataIDs for _lod.adt files
    map_texture: [[u32; 64]; 64],     // FileDataIDs for map textures
    map_texture_n: [[u32; 64]; 64],   // FileDataIDs for normal map textures
    minimap_texture: [[u32; 64]; 64], // FileDataIDs for minimap textures
}
```

### MWMO - World Map Object

For WMO-only maps, contains the filename of the global WMO.

```rust
struct MWMO {
    filename: CString,  // Zero-terminated string, max 256 bytes
}
```

**Important Notes**:

- In MOP, this chunk is limited to 0x100 bytes due to stack allocation
- **Pre-4.x**: Terrain maps include empty MWMO chunks (0 bytes)
- **4.x+**: Terrain maps have NO MWMO chunk at all (breaking change!)

### MODF - Map Object Definition

Placement information for the global WMO (if present).

```rust
struct MODF {
    entries: Vec<ModfEntry>,  // Only one entry for WDT files
}

struct ModfEntry {
    id: u32,                  // Index into MWMO, unused in WDT
    unique_id: u32,           // Unique instance ID (0xFFFFFFFF in 1.12.1, 0 in 3.3.5a)
    position: [f32; 3],       // Position in MODF coordinate system
    rotation: [f32; 3],       // Euler angles in radians (X, Y, Z order)
                              // Note: Some 2.4.3 files incorrectly store degrees
    lower_bounds: [f32; 3],   // Bounding box minimum corner
    upper_bounds: [f32; 3],   // Bounding box maximum corner
    flags: u16,               // WMO flags
    doodad_set: u16,          // Doodad set index
    name_set: u16,            // Name set index
    scale: u16,               // Scale factor (0 in 1.12.1, 1024 = 1.0 in later versions)
}

enum ModfFlags {
    Destructible = 0x0001,    // WMO is destructible
    UseLod       = 0x0002,    // WMO has LOD levels
    Unknown      = 0x0004,    // Unknown flag
}
```

## Additional WDT Files

Modern WoW uses multiple WDT files per map, each serving specific purposes:

### _lgt.wdt - Lighting (Legion 7.0+)

Contains global lighting information and light sources.

- **Chunks**: MPL2/MPL3 (point lights), MSLT (spotlights), MTEX (textures), MLTA (animations)
- **Purpose**: Enhanced lighting system with dynamic lights

### _occ.wdt - Occlusion

Low-resolution occlusion data for visibility culling.

- **Chunks**: MAOI (occlusion index), MAOH (occlusion heightmap)
- **Purpose**: Optimize rendering by culling non-visible areas

### _fogs.wdt - Fog Effects (Legion 7.0+, functional in BfA 8.0+)

Volumetric fog definitions and atmospheric effects.

- **Chunks**: MVFX (fog effects), VFOG (fog volumes), VFEX (extended fog data)
- **Purpose**: Atmospheric and weather effects

### _mpv.wdt - Particulate Volume (BfA 8.0.1+)

Particulate effects and volume data.

- **Purpose**: Volumetric particle effects like dust, smoke, etc.

### _tex.wdt - Textures

Texture-related data and references.

- **Chunks**: MTXF (texture flags), MTXP (texture parameters)
- **Purpose**: Additional texture properties and parameters

### _pd4.wdt

Purpose not fully documented (possibly physics or collision data).

## Version Evolution

### Classic (1.x) - Foundation

- **Format**: Basic structure with MVER, MPHD, MAIN chunks
- **Content Split**: ~60% WMO-only, ~40% terrain maps
- **Flags**: Minimal usage (only 0x0001 for WMO-only)
- **MODF Values**: UniqueID=0xFFFFFFFF, Scale=0

### Burning Crusade (2.x) - Expansion

- **No format changes** - Complete compatibility
- **Content**: Added Outland with massive terrain maps
- **Known Issues**: DireMaul rotation bug (degrees vs radians)

### Wrath of the Lich King (3.x) - Feature Enhancement

- **Major Flag Adoption** (while maintaining format compatibility):
  - 0x0002 (MCCV): 60% of maps
  - 0x0004 (Big Alpha): 60% of maps
  - 0x0008 (Sorted Doodads): 35% of maps
- **Content Evolution**: 70% terrain maps (shift from WMO-heavy design)

### Cataclysm (4.x) - BREAKING CHANGE

- **Format Change**: Terrain maps NO LONGER have MWMO chunks
- **Universal Flag**: 0x0040 in 100% of maps
- **Near-Universal Features**: 0x0008 (Sorted) in 95% of maps
- **Phasing Technology**: 1-2 tile maps for seamless world updates

### Mists of Pandaria (5.x) - Refinement

- **No structural changes** - Stable format
- **Flag 0x0080 Active**: Height texturing in 20% of maps
- **New Content Systems**: Scenarios, Pet Battles

### Legion (7.x)

- Added _lgt.wdt files with MPL2, MSLT, MTEX, MLTA chunks
- Enhanced fog system with MVFX/VFOG/VFEX chunks
- _fogs.wdt files added (initially empty)

### Battle for Azeroth (8.x)

- **8.0.1**: Added _mpv.wdt files for particulate volume effects
- **8.1.0**: Major change - introduction of MAID chunk
- Transition from filename-based to FileDataID system
- _fogs.wdt files became functional with fog data
- Deprecated MPHD flag 0x0010 (AdtHasLightingVertices)

### Shadowlands (9.x)

- Enhanced lighting system with MPL3 replacing MPL2
- Additional light properties for shadows and rendering
- Extended MAID structure variations

## Implementation Examples

### Reading a WDT File

```rust
use std::io::{Read, Seek, SeekFrom};
use std::fs::File;

pub struct WdtReader {
    file: File,
    version: u32,
    has_maid: bool,
}

impl WdtReader {
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        Ok(WdtReader {
            file,
            version: 0,
            has_maid: false,
        })
    }

    pub fn read(&mut self) -> Result<WdtFile, Box<dyn std::error::Error>> {
        let mut wdt = WdtFile::default();

        // Read chunks until EOF
        while let Ok(chunk) = self.read_chunk() {
            match &chunk.magic {
                b"MVER" => self.parse_mver(&chunk, &mut wdt)?,
                b"MPHD" => self.parse_mphd(&chunk, &mut wdt)?,
                b"MAIN" => self.parse_main(&chunk, &mut wdt)?,
                b"MAID" => self.parse_maid(&chunk, &mut wdt)?,
                b"MWMO" => self.parse_mwmo(&chunk, &mut wdt)?,
                b"MODF" => self.parse_modf(&chunk, &mut wdt)?,
                _ => {
                    // Skip unknown chunks
                    println!("Unknown chunk: {:?}",
                        std::str::from_utf8(&chunk.magic));
                }
            }
        }

        Ok(wdt)
    }

    fn read_chunk(&mut self) -> Result<Chunk, std::io::Error> {
        let mut magic = [0u8; 4];
        self.file.read_exact(&mut magic)?;

        let mut size_bytes = [0u8; 4];
        self.file.read_exact(&mut size_bytes)?;
        let size = u32::from_le_bytes(size_bytes);

        let mut data = vec![0u8; size as usize];
        self.file.read_exact(&mut data)?;

        Ok(Chunk { magic, size, data })
    }
}
```

### Coordinate System Conversion

```rust
/// Convert ADT tile coordinates to world coordinates (terrain system)
pub fn tile_to_world(tile_x: u32, tile_y: u32) -> (f32, f32, f32) {
    const TILE_SIZE: f32 = 533.33333;
    const MAP_CENTER: f32 = 32.0 * TILE_SIZE;

    // World coordinates use X=North, Y=West, Z=Up
    // Tile [0,0] is at the northwest corner
    let world_x = MAP_CENTER - (tile_y as f32 * TILE_SIZE);
    let world_y = MAP_CENTER - (tile_x as f32 * TILE_SIZE);
    let world_z = 0.0; // Height determined by terrain data

    (world_x, world_y, world_z)
}

/// Convert MODF placement coordinates to world coordinates
pub fn modf_to_world(modf_pos: [f32; 3]) -> [f32; 3] {
    const TILE_SIZE: f32 = 533.33333;
    const MAP_CENTER: f32 = 32.0 * TILE_SIZE;

    // MODF uses X=West, Y=Up, Z=North
    // World uses X=North, Y=West, Z=Up
    [
        MAP_CENTER - modf_pos[2],  // world_x (North) = center - modf_z
        MAP_CENTER - modf_pos[0],  // world_y (West) = center - modf_x
        modf_pos[1],               // world_z (Up) = modf_y
    ]
}
```

### Extracting ADT Tile Information

```rust
use std::collections::HashSet;

pub fn extract_adt_info(wdt_path: &str) -> Result<AdtInfo, Box<dyn std::error::Error>> {
    let mut reader = WdtReader::new(wdt_path)?;
    let wdt = reader.read()?;

    let mut info = AdtInfo {
        map_name: extract_map_name(wdt_path),
        existing_tiles: HashSet::new(),
        is_wmo_only: (wdt.flags & 0x0001) != 0,
        has_maid: (wdt.flags & 0x0200) != 0,
        global_wmo: wdt.global_wmo,
    };

    // Extract existing tiles
    for y in 0..64 {
        for x in 0..64 {
            if (wdt.tiles[y][x].flags & 0x0001) != 0 {
                info.existing_tiles.insert((x, y));
            }
        }
    }

    Ok(info)
}
```

## Common Patterns

### FileDataID Migration

Converting pre-8.1 maps to use FileDataIDs:

```rust
pub fn generate_maid_chunk(map_name: &str, existing_tiles: &HashSet<(u32, u32)>)
    -> Vec<u8> {
    let mut maid_data = Vec::new();

    // Generate FileDataIDs based on naming convention
    for y in 0..64 {
        for x in 0..64 {
            let file_data_id = if existing_tiles.contains(&(x, y)) {
                // Look up FileDataID from listfile
                lookup_file_data_id(&format!("{}_{}_{}.adt", map_name, x, y))
            } else {
                0  // No file
            };

            maid_data.extend_from_slice(&file_data_id.to_le_bytes());
        }
    }

    maid_data
}
```

### Version Compatibility

Handling different WDT versions across expansions:

```rust
pub enum WdtVersion {
    Classic,      // Pre-8.1
    BfA,          // 8.1.0+
    Shadowlands,  // 9.0+
}

impl WdtReader {
    pub fn detect_version(&self) -> WdtVersion {
        if self.has_maid {
            if self.has_extended_chunks {
                WdtVersion::Shadowlands
            } else {
                WdtVersion::BfA
            }
        } else {
            WdtVersion::Classic
        }
    }
}
```

## Common Issues and Solutions

### Issue 1: Missing MWMO Chunk

**Problem**: Some map editing tools don't generate MWMO chunks for terrain-based
maps.

**Solution**: Make MWMO optional for terrain maps (required for WMO-only maps):

```rust
// MWMO is optional for terrain maps
if (wdt.flags & 0x0001) == 0 {  // Not WMO-only
    // MWMO is optional, don't require it
}
```

### Issue 2: Coordinate System Confusion

**Problem**: WoW uses different coordinate systems for different purposes.

**Solution**:

- ADT tiles use [Y][X] array ordering
- World coordinates use X=North, Y=West, Z=Up
- MODF placement uses X=West, Y=Up, Z=North
- Always document which coordinate system you're using
- Provide conversion utilities

### Issue 3: Version Differences

**Problem**: Format changes between expansions (especially Cataclysm).

**Solution**: Check for format-breaking changes:

```rust
// Pre-4.x terrain maps have empty MWMO chunks
// 4.x+ terrain maps have NO MWMO chunk
let has_mwmo = if is_terrain_map {
    version < WowVersion::Cataclysm
} else {
    true  // WMO-only maps always have MWMO
};
```

## Performance Considerations

- WDT files are small (typically < 100KB)
- Keep in memory for entire map session
- Use to avoid loading non-existent ADTs
- Pre-calculate map bounds and connectivity
- Cache coordinate transformations

## References

- [WDT Format (wowdev.wiki)](https://wowdev.wiki/WDT)
- [ADT/WDT Grid System](https://wowdev.wiki/ADT/WDT)
- [libwarcraft Implementation](https://github.com/WowDevTools/libwarcraft)
- [TrinityCore Map Extractor](https://github.com/TrinityCore/TrinityCore/tree/master/src/tools/map_extractor)

## See Also

- [ADT Format](adt.md) - Terrain tile format
- [WDL Format](wdl.md) - Low-resolution world data
- [Map Loading Guide](../../guides/map-loading.md)
- [Coordinate Systems Guide](../../guides/coordinates.md)
