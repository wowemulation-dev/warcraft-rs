# ADT Format 🏔️

ADT (Area Data Terrain) files contain the terrain and object information for a
single map tile in World of Warcraft. The world is divided into a grid of 64x64
maps, with each map consisting of 16x16 chunks. Each ADT file represents one map
tile, which is 533.33333 yards (1600 feet) on each side.

## Overview

- **Extension**: `.adt`
- **Purpose**: Terrain geometry, textures, water, and object placement
- **Grid Size**: 16x16 chunks per ADT, 64x64 ADTs per continent
- **Chunk Size**: 33.33333 yards (100 feet) per chunk
- **Format**: Chunk-based binary format

## File Types

Since Cataclysm (4.x), ADT data is split across multiple files:

| File Pattern | Description | Content |
|--------------|-------------|---------|
| `MapName_XX_YY.adt` | Root file | Terrain structure, texture list, MCNK data |
| `MapName_XX_YY_tex0.adt` | Texture file | Low-resolution texture maps |
| `MapName_XX_YY_obj0.adt` | Object file | M2 and WMO placement data |
| `MapName_XX_YY_obj1.adt` | Object file | Additional object data |
| `MapName_XX_YY_lod.adt` | LOD file | Level of detail information |

Where XX and YY are the tile coordinates (0-63).

## Coordinate System

World of Warcraft uses a right-handed coordinate system:

- **X-axis**: North (positive) to South (negative)
- **Y-axis**: West (positive) to East (negative)
- **Z-axis**: Up (positive) to Down (negative)

Map coordinates:

- One ADT tile = 533.33333 yards = 1600 feet
- One chunk = 33.33333 yards = 100 feet
- One unit = 0.33333 yards = 1 foot

The world origin (0,0) is at the center of map (32,32).

## Chunk Structure

All chunks follow this format:

```rust
#[repr(C, packed)]
struct ChunkHeader {
    /// Four-character chunk identifier
    magic: [u8; 4],

    /// Size of chunk data (excluding this header)
    size: u32,
}
```

## Main ADT File Structure

### MVER - Version

```rust
#[repr(C, packed)]
struct MVERChunk {
    header: ChunkHeader,  // Magic: "MVER"
    version: u32,         // ADT version number (usually 18)
}
```

Version history:

- 17 = The Burning Crusade
- 18 = WotLK through Legion (most common)

### MHDR - Header

Contains offsets to all other chunks in the file:

```rust
#[repr(C, packed)]
struct MHDRChunk {
    header: ChunkHeader,  // Magic: "MHDR"

    /// All offsets are relative to start of file (0 if chunk not present)
    flags: u32,           // Always 0 in ADT files
    mcin_offset: u32,     // Offset to MCIN chunk
    mtex_offset: u32,     // Offset to MTEX chunk
    mmdx_offset: u32,     // Offset to MMDX chunk
    mmid_offset: u32,     // Offset to MMID chunk
    mwmo_offset: u32,     // Offset to MWMO chunk
    mwid_offset: u32,     // Offset to MWID chunk
    mddf_offset: u32,     // Offset to MDDF chunk
    modf_offset: u32,     // Offset to MODF chunk
    mfbo_offset: u32,     // Offset to MFBO chunk (TBC+)
    mh2o_offset: u32,     // Offset to MH2O chunk (TBC+)
    mtxf_offset: u32,     // Offset to MTXF chunk (TBC+)
    reserved: [u32; 4],   // Padding
}
```

### MCIN - Chunk Information

Contains offsets and sizes for all 256 (16x16) map chunks:

```rust
#[repr(C, packed)]
struct MCINEntry {
    /// Absolute offset to MCNK chunk
    mcnk_offset: u32,

    /// Size of MCNK chunk including header
    size: u32,

    /// Flags (usually 0)
    flags: u32,

    /// Async object id (0 if none)
    async_id: u32,
}

#[repr(C, packed)]
struct MCINChunk {
    header: ChunkHeader,  // Magic: "MCIN"
    entries: [MCINEntry; 256],  // 16x16 grid
}
```

### MTEX - Texture List

Contains null-terminated texture filenames:

```rust
struct MTEXChunk {
    header: ChunkHeader,  // Magic: "MTEX"
    /// Concatenated null-terminated strings
    /// Example: "Tileset\\Elwynn\\ElwynnGrass01.blp\0"
    texture_names: Vec<u8>,
}
```

### MMDX/MMID - Model List

- **MMDX**: Contains null-terminated M2 model filenames
- **MMID**: Maps model instances to their filename offsets in MMDX

### MWMO/MWID - WMO List

- **MWMO**: Contains null-terminated WMO filenames
- **MWID**: Maps WMO instances to their filename offsets in MWMO

### MDDF - Model (Doodad) Placement

```rust
#[repr(C, packed)]
struct MDDFEntry {
    /// Index into MMID
    mmid_entry: u32,

    /// Unique instance ID
    unique_id: u32,

    /// Position in world coordinates
    position: [f32; 3],  // x, y, z

    /// Rotation in degrees
    rotation: [f32; 3],  // x, y, z

    /// Scale factor (1024 = 1.0)
    scale: u16,

    /// Flags
    flags: u16,
}

// MDDF flags
const MDDF_BIODOME: u16 = 0x0001;     // Use for biodome in WMO
const MDDF_SHRUBBERY: u16 = 0x0002;   // Shrubbery scale factor
```

### MODF - WMO Placement

```rust
#[repr(C, packed)]
struct MODFEntry {
    /// Index into MWID
    mwid_entry: u32,

    /// Unique instance ID
    unique_id: u32,

    /// Position in world coordinates
    position: [f32; 3],

    /// Rotation in degrees
    rotation: [f32; 3],

    /// Bounding box
    extent_lower: [f32; 3],
    extent_upper: [f32; 3],

    /// Flags (same as MDDF)
    flags: u16,

    /// Doodad set index
    doodad_set: u16,

    /// Name set index
    name_set: u16,

    /// Scale (Legion+)
    scale: u16,
}
```

### MH2O - Water Information

Contains water levels and types:

```rust
#[repr(C, packed)]
struct MH2OHeader {
    /// Offset to MH2OInformation for this chunk
    offset_information: u32,

    /// Number of water layers
    layer_count: u32,

    /// Offset to render mask
    offset_render_mask: u32,
}

#[repr(C, packed)]
struct MH2OInformation {
    /// Water type (ocean, lake, etc)
    liquid_type: u16,

    /// Flags
    flags: u16,

    /// Height levels
    height_level1: f32,
    height_level2: f32,

    /// Position within chunk
    x_offset: u8,
    y_offset: u8,
    width: u8,
    height: u8,

    /// Offset to vertex data
    offset_vertex_data: u32,
}

// MH2O Flags
const MH2O_OCEAN: u16 = 0x0002;
const MH2O_DEEP: u16 = 0x0004;
const MH2O_FISHABLE: u16 = 0x0008;
```

### MFBO - Flight Bounds

Contains flight ceiling information:

```rust
#[repr(C, packed)]
struct MFBOChunk {
    header: ChunkHeader,  // Magic: "MFBO"

    /// Plane equation: ax + by + cz + d = 0
    plane: [f32; 4],  // a, b, c, d

    /// Flight bounds
    bounds: [[f32; 3]; 3],  // 3 points defining bounds
}
```

### MTXF - Texture Flags

```rust
#[repr(C, packed)]
struct MTXFChunk {
    header: ChunkHeader,  // Magic: "MTXF"
    flags: Vec<u32>,      // One per texture
}

// Texture flags
const MTXF_DISABLE_ALPHA: u32 = 0x0001;
const MTXF_USE_CUBE_MAP: u32 = 0x0002;
```

## Map Chunk (MCNK) Structure

Each MCNK chunk represents a 33.33x33.33 yard square of terrain:

### MCNK Header

```rust
#[repr(C, packed)]
struct MCNKHeader {
    /// Flags for this chunk
    flags: u32,

    /// Index of this chunk
    index_x: u32,
    index_y: u32,

    /// Number of texture layers (max 4)
    n_layers: u32,

    /// Number of doodad references
    n_doodad_refs: u32,

    /// High-res holes (8x8 grid)
    holes_high_res: u64,

    /// Offsets to sub-chunks (relative to MCNK data start)
    offset_mcly: u32,  // Texture layers
    offset_mcrf: u32,  // References
    offset_mcal: u32,  // Alpha maps
    size_mcal: u32,    // Alpha map size
    offset_mcsh: u32,  // Shadow map
    size_mcsh: u32,    // Shadow map size

    /// Area ID
    area_id: u32,

    /// Number of WMO references
    n_map_obj_refs: u32,

    /// Low-res holes (4x4 grid)
    holes_low_res: u16,

    /// Unknown
    unknown_0x3C: u16,

    /// Low-res texture map (8x8 grid)
    low_res_texture_map: [u16; 8],

    /// No effect doodad
    no_effect_doodad: u32,

    /// Sound emitters
    offset_mcse: u32,
    n_sound_emitters: u32,

    /// Liquid
    offset_mclq: u32,
    size_mclq: u32,

    /// Position of chunk
    position: [f32; 3],

    /// Vertex colors
    offset_mccv: u32,

    /// Vertex lighting (unused)
    offset_mclv: u32,

    /// Unused
    unused: u32,
}

// MCNK flags
const MCNK_HAS_MCSH: u32 = 0x0001;          // Has shadow map
const MCNK_IMPASS: u32 = 0x0002;            // Impassable
const MCNK_LQ_RIVER: u32 = 0x0004;          // River
const MCNK_LQ_OCEAN: u32 = 0x0008;          // Ocean
const MCNK_LQ_MAGMA: u32 = 0x0010;          // Magma
const MCNK_LQ_SLIME: u32 = 0x0020;          // Slime
const MCNK_HAS_MCCV: u32 = 0x0040;          // Has vertex colors
const MCNK_DO_NOT_FIX_ALPHA_MAP: u32 = 0x8000;   // Don't fix alpha map
const MCNK_HIGH_RES_HOLES: u32 = 0x10000;   // Use high-res holes
```

### MCVT - Vertex Heights

Contains 145 height values (9x9 outer + 8x8 inner vertices):

```rust
#[repr(C, packed)]
struct MCVTChunk {
    header: ChunkHeader,  // Magic: "MCVT"
    /// Height values relative to MCNK position
    /// Order: 9x9 outer vertices, then 8x8 inner vertices
    heights: [f32; 145],  // 9*9 + 8*8 = 145
}
```

Vertex layout:

```text
Outer vertices (9x9): Grid corners
Inner vertices (8x8): Center of each quad
```

### MCCV - Vertex Colors

Optional vertex colors for terrain shading:

```rust
#[repr(C, packed)]
struct MCCVColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,  // Usually 127 or 255
}

#[repr(C, packed)]
struct MCCVChunk {
    header: ChunkHeader,  // Magic: "MCCV"
    colors: [MCCVColor; 145],  // Same layout as MCVT
}
```

### MCNR - Normals

Normal vectors for lighting:

```rust
#[repr(C, packed)]
struct MCNREntry {
    normal: [i8; 3],    // X, Y, Z components (-127 to 127)
}

#[repr(C, packed)]
struct MCNRChunk {
    header: ChunkHeader,  // Magic: "MCNR"
    normals: [MCNREntry; 145],  // Same layout as MCVT
    padding: [u8; 13],   // Unknown padding
}
```

### MCLY - Texture Layers

Defines up to 4 texture layers:

```rust
#[repr(C, packed)]
struct MCLYEntry {
    /// Texture index in MTEX
    texture_id: u32,

    /// Flags for this layer
    flags: u32,

    /// Offset to alpha map in MCAL
    offset_in_mcal: u32,

    /// Effect ID (usually 0)
    effect_id: u32,
}

// MCLY flags
const MCLY_ANIMATION_ROTATION: u32 = 0x040;      // 45° rotation
const MCLY_ANIMATION_SPEED_FAST: u32 = 0x080;    // Faster animation
const MCLY_ANIMATION_SPEED_FASTER: u32 = 0x100;  // Even faster
const MCLY_ANIMATION_SPEED_FASTEST: u32 = 0x200; // Fastest
const MCLY_ANIMATE: u32 = 0x400;                 // Enable animation
const MCLY_USE_ALPHA_MAP: u32 = 0x800;           // Use alpha map
const MCLY_USE_CUBE_MAP_REFLECTION: u32 = 0x1000; // Skybox reflection
```

### MCRF - References

Lists doodad and object references for this chunk:

```rust
#[repr(C, packed)]
struct MCRFChunk {
    header: ChunkHeader,  // Magic: "MCRF"
    /// Indices into MDDF and MODF
    doodad_refs: Vec<u32>,
    object_refs: Vec<u32>,
}
```

### MCSH - Shadow Map

64x64 bit shadow map:

```rust
#[repr(C, packed)]
struct MCSHChunk {
    header: ChunkHeader,  // Magic: "MCSH"
    /// 1 bit per terrain quad
    shadow_map: [u8; 512],  // 64 * 64 / 8
}
```

### MCAL - Alpha Maps

Alpha maps control texture blending. The first texture layer has no alpha map
(it's the base):

```rust
enum AlphaMapFormat {
    /// Uncompressed 64x64 (4096 bytes)
    Uncompressed4096,

    /// Uncompressed 32x32 (2048 bytes)
    Uncompressed2048,

    /// Compressed (variable size)
    Compressed,
}
```

Compression algorithm:

```rust
fn decompress_alpha_map(compressed: &[u8], output: &mut [u8]) {
    let mut input_pos = 0;
    let mut output_pos = 0;

    while output_pos < output.len() && input_pos < compressed.len() {
        let count = compressed[input_pos] & 0x7F;
        let fill = (compressed[input_pos] & 0x80) != 0;
        input_pos += 1;

        for _ in 0..count {
            if fill {
                // Repeat single value
                output[output_pos] = compressed[input_pos];
                output_pos += 1;
            } else {
                // Copy values
                output[output_pos] = compressed[input_pos];
                input_pos += 1;
                output_pos += 1;
            }
        }

        if fill {
            input_pos += 1;
        }
    }
}
```

### MCLQ - Liquid (Legacy)

Pre-WotLK liquid data:

```rust
#[repr(C, packed)]
struct MCLQHeader {
    min_height: f32,
    max_height: f32,
}

#[repr(C, packed)]
struct MCLQVertex {
    /// Water height
    height: f32,

    /// Flow data
    flow: [u8; 4],
}

#[repr(C, packed)]
struct MCLQChunk {
    header: ChunkHeader,  // Magic: "MCLQ"
    liquid_header: MCLQHeader,

    /// 9x9 vertex heights
    vertices: [MCLQVertex; 81],

    /// 8x8 flags
    flags: [u8; 64],
}

// MCLQ flags
const MCLQ_HIDDEN: u8 = 0x01;
const MCLQ_FISHABLE: u8 = 0x02;
const MCLQ_SHARED: u8 = 0x04;
```

### MCSE - Sound Emitters

```rust
#[repr(C, packed)]
struct MCSEEntry {
    /// Sound ID
    sound_id: u32,

    /// Position relative to chunk
    position: [f32; 3],

    /// Size/radius
    size: [f32; 3],
}
```

## Height Calculation Algorithm

To calculate terrain height at any position:

```rust
/// Get interpolated height at position within chunk
pub fn get_height_at_position(
    mcvt: &MCVTChunk,
    x: f32,  // 0.0 to 33.33333
    y: f32,  // 0.0 to 33.33333
) -> f32 {
    // Convert to cell coordinates
    let cell_x = (x / (33.33333 / 8.0)).min(7.999);
    let cell_y = (y / (33.33333 / 8.0)).min(7.999);

    let ix = cell_x as usize;
    let iy = cell_y as usize;

    let fx = cell_x - ix as f32;
    let fy = cell_y - iy as f32;

    // Get the four corner heights
    let h00 = mcvt.get_outer(ix, iy);
    let h01 = mcvt.get_outer(ix, iy + 1);
    let h10 = mcvt.get_outer(ix + 1, iy);
    let h11 = mcvt.get_outer(ix + 1, iy + 1);

    // Get center height
    let hc = mcvt.get_inner(ix, iy);

    // Determine which triangle and interpolate
    if fx + fy < 1.0 {
        // Lower triangle
        if fx < fy {
            // Left triangle
            h00 * (1.0 - fx - fy) + h01 * fy + hc * fx
        } else {
            // Bottom triangle
            h00 * (1.0 - fx - fy) + h10 * fx + hc * fy
        }
    } else {
        // Upper triangle
        if fx > fy {
            // Right triangle
            h11 * (fx + fy - 1.0) + h10 * (1.0 - fy) + hc * (1.0 - fx)
        } else {
            // Top triangle
            h11 * (fx + fy - 1.0) + h01 * (1.0 - fx) + hc * (1.0 - fy)
        }
    }
}
```

## Coordinate Transformations

```rust
/// World position to ADT tile coordinates
pub fn world_to_adt(world_x: f32, world_y: f32) -> (i32, i32) {
    // World origin is at center of map (32, 32)
    let adt_x = 32 - (world_x / 533.33333);
    let adt_y = 32 - (world_y / 533.33333);

    (adt_x as i32, adt_y as i32)
}

/// ADT tile to world coordinates (tile corner)
pub fn adt_to_world(adt_x: i32, adt_y: i32) -> (f32, f32) {
    let world_x = (32 - adt_x) as f32 * 533.33333;
    let world_y = (32 - adt_y) as f32 * 533.33333;

    (world_x, world_y)
}

/// Position within ADT to chunk index
pub fn position_to_chunk(x: f32, y: f32) -> (usize, usize) {
    let chunk_x = (x / 33.33333) as usize;
    let chunk_y = (y / 33.33333) as usize;

    (chunk_x.min(15), chunk_y.min(15))
}
```

## Usage Examples

### Loading and Parsing ADT

```rust
use warcraft_rs::adt::{Adt, ChunkHeader};

// Load ADT file
let adt = Adt::open("World/Maps/Azeroth/Azeroth_32_48.adt")?;

// Access terrain chunks
for (index, chunk) in adt.chunks.iter().enumerate() {
    let x = index % 16;
    let y = index / 16;
    println!("Chunk ({}, {})", x, y);

    // Get height at specific position
    let height = chunk.get_height(16.67, 16.67);

    // Get texture layers
    for layer in &chunk.layers {
        println!("  Texture: {}", adt.textures[layer.texture_id as usize]);
    }
}

// Export heightmap
let heightmap = adt.export_heightmap();
heightmap.save("terrain_height.png")?;

// Find all doodads (M2 models)
for doodad in &adt.doodads {
    let model_name = &adt.models[doodad.model_id as usize];
    println!("Model: {} at {:?}", model_name, doodad.position);
}
```

### Texture Blending

```rust
/// Apply alpha maps to create final texture
pub fn blend_textures(
    textures: &[Texture],
    alpha_maps: &[AlphaMap],
    u: f32,  // Texture coordinate 0-1
    v: f32,  // Texture coordinate 0-1
) -> Color {
    // Start with base texture
    let mut color = textures[0].sample(u, v);

    // Blend additional layers
    for i in 1..textures.len() {
        let alpha = alpha_maps[i - 1].sample(u, v);
        let layer_color = textures[i].sample(u, v);

        // Blend using alpha
        color = color.lerp(layer_color, alpha);
    }

    color
}
```

### Liquid Rendering

```rust
// MH2O water (WotLK+)
if let Some(water_chunk) = &chunk.water {
    for layer in &water_chunk.layers {
        let water_type = layer.liquid_type;
        let height = layer.height_level1;

        // Render water surface
        render_water_layer(layer, water_type);
    }
}

// Legacy MCLQ water
if let Some(liquid) = &chunk.liquid {
    for x in 0..9 {
        for y in 0..9 {
            let vertex = &liquid.vertices[y * 9 + x];
            let depth = vertex.height;
            let flow = vertex.flow;

            // Render water vertex
        }
    }
}
```

### Streaming Large Worlds

```rust
use warcraft_rs::adt::AdtManager;

let mut manager = AdtManager::new("World/Maps/Azeroth")?;
manager.set_view_distance(3); // Load 3x3 ADTs around player

// Update based on player position
manager.update_position(player_x, player_y);

// Get loaded ADTs
for adt in manager.loaded_adts() {
    // Render terrain
    render_adt(&adt);
}

// Unload distant ADTs
manager.cleanup_distant_adts();
```

## Implementation Notes

### Reading ADT Files

```rust
pub struct ADTReader {
    data: Vec<u8>,
    position: usize,
}

impl ADTReader {
    pub fn read_chunk(&mut self) -> Result<(ChunkHeader, &[u8]), ADTError> {
        if self.position + 8 > self.data.len() {
            return Err(ADTError::UnexpectedEof);
        }

        // Read header
        let header = unsafe {
            *(self.data.as_ptr().add(self.position) as *const ChunkHeader)
        };

        self.position += 8;

        // Get chunk data
        let chunk_size = header.size as usize;
        if self.position + chunk_size > self.data.len() {
            return Err(ADTError::InvalidChunkSize);
        }

        let chunk_data = &self.data[self.position..self.position + chunk_size];
        self.position += chunk_size;

        Ok((header, chunk_data))
    }
}
```

### Performance Optimizations

1. **Memory mapping**: Use memory-mapped files for large ADT files
2. **Lazy loading**: Only parse chunks when needed
3. **Caching**: Cache frequently accessed data like height maps
4. **LOD**: Use WDL files for distant terrain
5. **Frustum culling**: Only render visible chunks
6. **Texture atlasing**: Combine texture layers to reduce draw calls

### Common Pitfalls

1. **Byte order**: All values are little-endian
2. **Chunk alignment**: Chunks are not always aligned to 4-byte boundaries
3. **String parsing**: Strings in MTEX/MMDX/MWMO are null-terminated
4. **Coordinate systems**: Y-axis is north/south (not up/down)
5. **Height interpolation**: Must use the center vertices for proper interpolation
6. **Alpha map compression**: Check MCLY flags to determine format
7. **Scale values**: MDDF scale is 1024 = 1.0, not a float

## Version History

| Version | Game | Major Changes |
|---------|------|---------------|
| 17 | The Burning Crusade | Initial documented version |
| 18 | Wrath of the Lich King | Added MH2O water, high-res holes |
| 18 | Cataclysm | Split files (_tex0,_obj0, _obj1) |
| 18 | Mists of Pandaria | Minor updates |
| 18 | Warlords of Draenor | Added _lod.adt files |
| 18 | Legion | Added scale to MODF |

## References

- [WoWDev Wiki - ADT Format](https://wowdev.wiki/ADT)
- [Map Coordinates System](https://wowdev.wiki/Map_coordinates)
- [Trinity Core Map Extractor](https://github.com/TrinityCore/TrinityCore/tree/master/src/tools/map_extractor)
- [libwarcraft ADT Implementation](https://github.com/WowDevTools/libwarcraft)

## See Also

- [Rendering ADT Terrain Guide](../../guides/adt-rendering.md)
- [WDT Format](wdt.md) - World tables that reference ADTs
- [WDL Format](wdl.md) - Low-detail world data
- [Coordinate System](../../resources/coordinates.md) - Detailed coordinate information
