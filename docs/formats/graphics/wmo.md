# WMO Format 🏰

WMO (World Map Object) files are used in World of Warcraft to represent large
static objects such as buildings, caves, and other structures that are too
complex to be represented as M2 (doodad) models. WMOs consist of a root file and
zero or more group files that contain geometry data.

## Overview

- **Extension**: `.wmo` (root file), `_000.wmo` to `_999.wmo` (group files)
- **Magic**: Chunk-based format with 4-character identifiers (reversed in file)
- **Purpose**: Large static world geometry with interior/exterior areas
- **Components**: Root file + multiple group files
- **Features**: Portal data ⚠️ **Parsing Only**, BSP trees ⚠️ **Parsing Only**, lighting data ⚠️ **Parsing Only**, multiple LODs ⚠️ **Format Specification**
- **Use Cases**: Buildings, dungeons, caves, large structures, instances

## Key Characteristics

- **Chunk-based format**: Similar to other Blizzard formats, using 4-character
  chunk identifiers
- **Multi-file structure**: Root file + group files (numbered _000.wmo to_999.wmo)
- **Portal data**: ⚠️ **Format Specification Only** - Portal vertices and normals parsed, visibility culling not implemented
- **BSP trees**: ⚠️ **Format Specification Only** - Node structure parsed, collision detection not implemented  
- **Multiple LOD support**: ⚠️ **Format Specification Only** - LOD data parsed, rendering optimization not implemented
- **Lighting data**: ⚠️ **Format Specification Only** - Light parameters parsed, lighting calculations not implemented

## Version History

Based on empirical analysis of WMO files from original MPQ archives:

| Version | Expansion | Core Chunks | Notable Changes |
|---------|-----------|-------------|-----------------|
| 17 | Vanilla WoW (1.12.1) | MVER, MOHD, MOTX, MOMT, MOGN, MOGI, MOSB, MOPV, MOPT, MOPR, MOVV, MOVB, MOLT, MODS, MODN, MODD, MFOG | Original format with 17 core chunks |
| 17 | The Burning Crusade (2.4.3) | Same as 1.12.1 | No new chunks detected in samples |
| 17 | Wrath of the Lich King (3.3.5a) | Same as 1.12.1 | No new chunks detected in samples |
| 17 | Cataclysm (4.3.4) | Core + MCVP | Added MCVP (Convex Volume Planes, 496 bytes in transport WMOs) |
| 17 | Mists of Pandaria (5.4.8) | Core + MCVP | No additional chunks detected |
| 17 | Warlords of Draenor (6.x) | Core + GFID | Added GFID chunk for file IDs |
| 17 | Legion (7.x) | Core + MOP2, MPVD | Added MOP2 (Portal Info 2), MPVD (particle volumes) |
| 17 | Battle for Azeroth (8.x) | Core + shadow chunks | Enhanced shadow mapping (MLSP, MLSS, MLSK) |
| 17 | Shadowlands (9.x) | Core + volume chunks | Additional volume data types (MAVD, MBVD) |

**Note**: Version number (17) remained constant from Vanilla through modern WoW, with functionality added through new optional chunks rather than version changes.

## File Structure Overview

WMO files follow a chunk-based format where each chunk has:

- 4-byte chunk identifier (reversed in file, e.g., "REVM" for MVER)
- 4-byte chunk size (not including the 8-byte header)
- Chunk data

```rust
use wow_wmo::{ChunkHeader, ChunkId};

// Example of reading a chunk header
let header = ChunkHeader {
    id: ChunkId::from_str("MVER"),
    size: 4,
};

println!("Chunk ID: {}", header.id);
println!("Size: {}", header.size);
```

## Empirical Analysis Results

Based on analysis of WMO files from WoW versions 1.12.1 through 3.3.5a:

### Core Chunk Structure (All Versions)
All analyzed WMO root files consistently contain these 17 chunks in order:
1. **MVER** (4 bytes) - Version, always value 17
2. **MOHD** (64 bytes) - Header with counts and flags
3. **MOTX** (variable) - Texture filenames, null-terminated strings
4. **MOMT** (variable) - Materials, 64 bytes per material
5. **MOGN** (variable) - Group names, null-terminated strings
6. **MOGI** (variable) - Group information, 32 bytes per group
7. **MOSB** (4 bytes) - Skybox filename offset or empty
8. **MOPV** (variable) - Portal vertices, 12 bytes per vertex
9. **MOPT** (variable) - Portal information, 20 bytes per portal
10. **MOPR** (variable) - Portal references, 8 bytes per reference
11. **MOVV** (0 bytes typically) - Visible block vertices (often empty)
12. **MOVB** (0 bytes typically) - Visible block list (often empty)
13. **MOLT** (variable) - Lighting, 48 bytes per light
14. **MODS** (32 bytes typically) - Doodad sets, single default set common
15. **MODN** (variable) - Doodad names, null-terminated M2 filenames
16. **MODD** (variable) - Doodad definitions, 40 bytes per doodad
17. **MFOG** (variable) - Fog parameters, 48 bytes per fog entry

### Group File Structure
Group files (e.g., `*_000.wmo`) contain a single large MOGP chunk:
- **MVER** (4 bytes) - Version 17
- **MOGP** (entire remaining file) - Contains all group geometry sub-chunks

## Chunk Specifications

### Root File Chunks

The root WMO file contains global information about the entire model.

#### MVER - Version

Always the first chunk in the file. ✅ **Implemented**

```rust
// Version information is part of WmoRoot
use wow_wmo::{WmoVersion, WmoRoot};

// Example accessing version from parsed WMO
fn check_version(wmo: &WmoRoot) {
    println!("WMO Version: {}", wmo.version.to_raw());
    println!("Expansion: {}", wmo.version.expansion_name());
}
```

#### MOHD - Header

Contains general information about the WMO. ✅ **Implemented**

```rust
use wow_wmo::{WmoHeader, WmoFlags, Color};

// Example accessing WMO header information
fn analyze_wmo_header(header: &WmoHeader) {
    println!("Materials: {}", header.n_materials);
    println!("Groups: {}", header.n_groups);
    println!("Portals: {}", header.n_portals);
    println!("Lights: {}", header.n_lights);
    println!("Doodad Defs: {}", header.n_doodad_defs);
    println!("Doodad Sets: {}", header.n_doodad_sets);

    if header.flags.contains(WmoFlags::HAS_SKYBOX) {
        println!("WMO has skybox");
    }

    if header.flags.contains(WmoFlags::INDOOR_MAP) {
        println!("WMO is an indoor map");
    }
}
```

#### MOTX - Textures

Null-terminated texture filenames used by this WMO. ✅ **Implemented**

```rust
// Textures are automatically parsed and available in WmoRoot
use wow_wmo::WmoRoot;

fn list_textures(wmo: &WmoRoot) {
    for (i, texture) in wmo.textures.iter().enumerate() {
        println!("Texture {}: {}", i, texture);
    }
}
```

#### MOMT - Materials

Material definitions for all textures. ✅ **Implemented**

```rust
use wow_wmo::{WmoMaterial, WmoMaterialFlags};

// Example analyzing WMO materials
fn analyze_material(material: &WmoMaterial) {
    if material.flags.contains(WmoMaterialFlags::UNLIT) {
        println!("Material is unlit");
    }

    if material.flags.contains(WmoMaterialFlags::TWO_SIDED) {
        println!("Material is two-sided");
    }

    if material.flags.contains(WmoMaterialFlags::UNFOGGED) {
        println!("Material is unfogged");
    }

    println!("Texture 1 index: {}", material.texture1);
    println!("Texture 2 index: {}", material.texture2);
    println!("Blend mode: {}", material.blend_mode);
    println!("Ground type: {}", material.ground_type);
}
```

#### MOGN - Group Names

Null-terminated strings for group names (primarily for debugging).

```rust
// Group names are automatically parsed and available in WmoGroupInfo
use wow_wmo::{WmoRoot, WmoGroupInfo};

fn list_group_names(wmo: &WmoRoot) {
    for (i, group_info) in wmo.groups.iter().enumerate() {
        println!("Group {}: {}", i, group_info.name);
    }
}
```

#### MOGI - Group Information

Information about each group in the WMO.

```rust
use wow_wmo::{WmoGroupInfo, WmoGroupFlags};

// Example analyzing group information
fn analyze_group_info(group_info: &WmoGroupInfo) {
    println!("Group name: {}", group_info.name);
    println!("Bounding box: {:?}", group_info.bounding_box);

    if group_info.flags.contains(WmoGroupFlags::INDOOR) {
        println!("Group is indoor");
    }

    if group_info.flags.contains(WmoGroupFlags::HAS_VERTEX_COLORS) {
        println!("Group has vertex colors");
    }

    if group_info.flags.contains(WmoGroupFlags::HAS_DOODADS) {
        println!("Group has doodads");
    }

    if group_info.flags.contains(WmoGroupFlags::HAS_WATER) {
        println!("Group has liquid");
    }
}
```

#### MOSB - Skybox

Skybox model filename (if present).

```rust
// Skybox information is available in WmoRoot
use wow_wmo::WmoRoot;

fn check_skybox(wmo: &WmoRoot) {
    if let Some(skybox) = &wmo.skybox {
        println!("Skybox model: {}", skybox);
    } else {
        println!("No skybox");
    }
}
```

#### MOPV - Portal Vertices

Vertices used to define portal geometry.

```rust
use wow_wmo::{WmoPortal, Vec3};

// Portal vertices are part of WmoPortal structure
fn analyze_portal(portal: &WmoPortal) {
    println!("Portal has {} vertices", portal.vertices.len());
    println!("Portal normal: {:?}", portal.normal);

    for (i, vertex) in portal.vertices.iter().enumerate() {
        println!("Vertex {}: ({}, {}, {})", i, vertex.x, vertex.y, vertex.z);
    }
}
```

#### MOPT - Portal Information

Portal definitions connecting groups.

```rust
#[repr(C, packed)]
struct MOPTEntry {
    /// Index into MOPV for start of vertices
    start_vertex: u16,

    /// Number of vertices in this portal
    vertex_count: u16,

    /// Portal plane (normal xyz, distance w)
    plane: [f32; 4],
}
```

#### MOPR - Portal References

Links portals to groups.

```rust
#[repr(C, packed)]
struct MOPREntry {
    /// Portal index
    portal_index: u16,

    /// Group index
    group_index: u16,

    /// 1 = portal is on the interior side of the group
    side: i16,

    /// Padding
    _padding: u16,
}
```

#### MOVV - Visible Block Vertices

Vertices for visibility blocking volumes.

```rust
#[repr(C, packed)]
struct MOVVEntry {
    position: [f32; 3],
}
```

#### MOVB - Visible Block List

Defines visibility blocking volumes.

```rust
#[repr(C, packed)]
struct MOVBEntry {
    /// Index into MOVV
    start_vertex: u16,

    /// Number of vertices
    vertex_count: u16,
}
```

#### MOLT - Lighting

Light definitions for the WMO.

```rust
use wow_wmo::{WmoLight, WmoLightType, WmoLightProperties};

// Example analyzing WMO lights
fn analyze_light(light: &WmoLight) {
    println!("Light type: {:?}", light.light_type);
    println!("Position: ({}, {}, {})", light.position.x, light.position.y, light.position.z);
    println!("Color: {:?}", light.color);
    println!("Intensity: {}", light.intensity);

    if light.use_attenuation {
        println!("Attenuation: {} to {}", light.attenuation_start, light.attenuation_end);
    }

    match &light.properties {
        WmoLightProperties::Spot { direction, hotspot, falloff } => {
            println!("Spot light: direction {:?}, hotspot {}, falloff {}", direction, hotspot, falloff);
        }
        WmoLightProperties::Directional { direction } => {
            println!("Directional light: direction {:?}", direction);
        }
        WmoLightProperties::Omni => {
            println!("Omni light");
        }
        WmoLightProperties::Ambient => {
            println!("Ambient light");
        }
    }
}
```

#### MODS - Doodad Sets

Doodad set definitions (e.g., "furniture", "decorations").

```rust
use wow_wmo::WmoDoodadSet;

// Example analyzing doodad sets
fn analyze_doodad_set(doodad_set: &WmoDoodadSet) {
    println!("Doodad set: {}", doodad_set.name);
    println!("Start doodad: {}", doodad_set.start_doodad);
    println!("Number of doodads: {}", doodad_set.n_doodads);
}
```

#### MODN - Doodad Names

List of null-terminated doodad filenames (M2 models).

```rust
// Doodad names are automatically parsed and available
// They would typically be referenced by doodad definitions
use wow_wmo::WmoRoot;

fn show_doodad_info(wmo: &WmoRoot) {
    for (i, doodad_def) in wmo.doodad_defs.iter().enumerate() {
        println!("Doodad {}: position ({}, {}, {})",
            i, doodad_def.position.x, doodad_def.position.y, doodad_def.position.z);
        println!("  Scale: {}", doodad_def.scale);
        println!("  Color: {:?}", doodad_def.color);
    }
}
```

#### MODD - Doodad Definitions

Placement information for doodads.

```rust
use wow_wmo::WmoDoodadDef;

// Example analyzing doodad definitions
fn analyze_doodad_def(doodad_def: &WmoDoodadDef) {
    println!("Name offset: {}", doodad_def.name_offset);
    println!("Position: ({}, {}, {})",
        doodad_def.position.x, doodad_def.position.y, doodad_def.position.z);
    println!("Orientation: [{}, {}, {}, {}]",
        doodad_def.orientation[0], doodad_def.orientation[1],
        doodad_def.orientation[2], doodad_def.orientation[3]);
    println!("Scale: {}", doodad_def.scale);
    println!("Color: {:?}", doodad_def.color);
    println!("Set index: {}", doodad_def.set_index);
}
```

#### MFOG - Fog

Fog settings for groups.

```rust
#[repr(C, packed)]
struct MFOGEntry {
    /// Flags
    flags: u32,

    /// Position
    position: [f32; 3],

    /// Small radius
    radius_small: f32,

    /// Large radius
    radius_large: f32,

    /// Fog end distance
    fog_end: f32,

    /// Fog start multiplier
    fog_start_multiplier: f32,

    /// Fog color (BGRA)
    color: u32,

    /// Underwater fog end
    underwater_end: f32,

    /// Underwater fog start multiplier
    underwater_start_multiplier: f32,

    /// Underwater color (BGRA)
    underwater_color: u32,
}
```

#### MCVP - Convex Volume Planes

Convex volume planes for advanced collision or effects.

```rust
#[repr(C, packed)]
struct MCVPEntry {
    /// Plane equation (normal xyz, distance w)
    plane: [f32; 4],
}
```

#### MOUV - UV Transformations

UV transformations for animated textures (Legion+). ✅ **Implemented**

```rust
#[repr(C, packed)]
struct MOUVEntry {
    translation_speed: [[f32; 2]; 2], // 2 C2Vectors per material
}
```

#### MOPE - Portal Extra Information

Additional portal information (WarWithin+). ✅ **Implemented**

```rust
#[repr(C, packed)]
struct MOPEEntry {
    portal_index: u32, // index into MOPT
    unk1: u32,
    unk2: u32,
    unk3: u32,
}
```

#### MOLV - Light Extensions

Extended light information (Shadowlands+). ✅ **Implemented**

```rust
#[repr(C, packed)]
struct MOLVEntry {
    directions: [[f32; 4]; 6], // 6 sets of C3Vector + float value
    unknown: [u8; 3],
    molt_index: u8,
}
```

#### MODI - Doodad File IDs

Doodad file IDs for modern file reference system (Battle for Azeroth+). ✅ **Implemented**

```rust
/// MODI contains an array of u32 doodad IDs, same count as SMOHeader.nDoodadNames
fn parse_modi(data: &[u8]) -> Vec<u32> {
    data.chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}
```

#### MOM3 - New Materials

New material system for modern WoW versions (WarWithin+). ✅ **Implemented**

```rust
#[repr(C, packed)]
struct MOM3Entry {
    // m3SI structure - defines new materials
    // Structure details may vary, treated as opaque data
    data: Vec<u8>,
}
```

#### MOMO - Alpha Version Container

Container chunk for alpha WoW versions (version 14 only). ✅ **Implemented**

```rust
// MOMO is a container chunk with no additional data
// It wraps other chunks in early WoW alpha versions
```

#### GFID - Group File IDs

File IDs for group files (modern WoW versions). ⚠️ **Format Specification Only**

```rust
/// GFID contains an array of u32 file IDs, one per group
fn parse_gfid(data: &[u8]) -> Vec<u32> {
    data.chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}
```

### Group File Chunks

Each group file contains the geometry and rendering data for a portion of the WMO.

#### MOGP - Group Header

The main header for a group file, contains all other chunks.

```rust
#[repr(C, packed)]
struct MOGPHeader {
    /// Group name offset in MOGN
    group_name: u32,

    /// Descriptive name offset in MOGN
    descriptive_name: u32,

    /// Flags (same as MOGI flags)
    flags: u32,

    /// Bounding box
    bounding_box_min: [f32; 3],
    bounding_box_max: [f32; 3],

    /// Portal index offset
    portal_start: u16,

    /// Number of portals
    portal_count: u16,

    /// Number of batches A
    batch_count_a: u16,

    /// Number of batches B
    batch_count_b: u16,

    /// Number of batches C
    batch_count_c: u16,

    /// Number of batches D
    batch_count_d: u16,

    /// Fog indices
    fog_indices: [u8; 4],

    /// Liquid type
    liquid_type: u32,

    /// Group ID
    group_id: u32,

    /// Unknown fields
    unknown_1: u32,
    unknown_2: u32,
}
```

#### MOPY - Material Info

Material information for each triangle. ✅ **Implemented**

```rust
#[repr(C, packed)]
struct MOPYEntry {
    /// Flags
    flags: u8,

    /// Material ID
    material_id: u8,
}

impl MOPYEntry {
    // Triangle flags
    pub const FLAG_UNK_0X01: u8 = 0x01;
    pub const FLAG_NO_COLLISION: u8 = 0x02;
    pub const FLAG_NO_CAMERA_COLLISION: u8 = 0x04;
    pub const FLAG_NO_RENDER: u8 = 0x08;
    pub const FLAG_IS_WATER: u8 = 0x10;
}
```

#### MOVI - Vertex Indices

Triangle vertex indices.

```rust
/// MOVI contains u16 indices, 3 per triangle
fn parse_movi(data: &[u8]) -> Vec<[u16; 3]> {
    data.chunks_exact(6)
        .map(|chunk| {
            [
                u16::from_le_bytes([chunk[0], chunk[1]]),
                u16::from_le_bytes([chunk[2], chunk[3]]),
                u16::from_le_bytes([chunk[4], chunk[5]]),
            ]
        })
        .collect()
}
```

#### MOVT - Vertices

Vertex positions. ✅ **Implemented**

Vertices chunk with count = size / (sizeof(float) * 3). 3 floats per vertex.
**Important**: Coordinates are in (X,Z,-Y) order as WMOs use a coordinate system with Z-up and Y into screen,
while OpenGL uses Z toward viewer and Y up.

```rust
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct MOVTEntry {
    x: f32,  // X coordinate
    y: f32,  // Z coordinate (in WMO space)
    z: f32,  // -Y coordinate (in WMO space)
}
```

#### MONR - Normals

Vertex normals. ✅ **Implemented**

```rust
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct MONREntry {
    x: f32,
    y: f32,
    z: f32,
}
```

#### MOTV - Texture Coordinates

Texture coordinates (can have up to 3 sets). ✅ **Implemented**

```rust
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct MOTVEntry {
    u: f32,
    v: f32,
}
```

#### MOBA - Render Batches

Defines how triangles are grouped for rendering. ✅ **Implemented**

```rust
#[repr(C, packed)]
struct MOBAEntry {
    /// Bounding box for culling (bx, by, bz)
    bounding_box_min: [i16; 3],

    /// Bounding box for culling (tx, ty, tz)
    bounding_box_max: [i16; 3],

    /// Index of the first face index used in MOVI
    start_index: u32,

    /// Number of MOVI indices used
    count: u16,

    /// Index of the first vertex used in MOVT
    min_index: u16,

    /// Index of the last vertex used (batch includes this one)
    max_index: u16,

    /// Batch flags
    flags: u8,

    /// Material index in MOMT
    material_id: u8,
}
```

#### MOLR - Light References

References to lights that affect this group.

```rust
/// MOLR contains u16 light indices
fn parse_molr(data: &[u8]) -> Vec<u16> {
    data.chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect()
}
```

#### MODR - Doodad References

References to doodads in this group.

```rust
/// MODR contains u16 doodad indices
fn parse_modr(data: &[u8]) -> Vec<u16> {
    data.chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect()
}
```

#### MOBN - BSP Nodes

Binary Space Partition tree nodes for collision detection.

```rust
#[repr(C, packed)]
struct MOBNNode {
    /// Flags
    flags: u16,

    /// Negative child index
    neg_child: i16,

    /// Positive child index
    pos_child: i16,

    /// Number of faces
    face_count: u16,

    /// Index of first face
    face_start: u32,

    /// Plane distance
    plane_dist: f32,
}

impl MOBNNode {
    pub const FLAG_AXIS_X: u16 = 0x00;
    pub const FLAG_AXIS_Y: u16 = 0x01;
    pub const FLAG_AXIS_Z: u16 = 0x02;
    pub const FLAG_AXIS_MASK: u16 = 0x03;
    pub const FLAG_LEAF: u16 = 0x04;
}
```

#### MOBR - BSP Face Indices

Face indices referenced by BSP leaf nodes.

```rust
/// MOBR contains u16 face indices
fn parse_mobr(data: &[u8]) -> Vec<u16> {
    data.chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect()
}
```

#### MOCV - Vertex Colors

Vertex colors for lighting.

```rust
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct MOCVEntry {
    /// BGRA color
    color: u32,
}

impl MOCVEntry {
    pub fn from_bgra(b: u8, g: u8, r: u8, a: u8) -> Self {
        Self {
            color: (a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | (b as u32)
        }
    }

    pub fn to_rgba_f32(&self) -> [f32; 4] {
        [
            ((self.color >> 16) & 0xFF) as f32 / 255.0, // R
            ((self.color >> 8) & 0xFF) as f32 / 255.0,  // G
            (self.color & 0xFF) as f32 / 255.0,         // B
            ((self.color >> 24) & 0xFF) as f32 / 255.0, // A
        ]
    }
}
```

#### MLIQ - Liquids

Liquid (water/lava/slime) data for this group.

```rust
#[repr(C, packed)]
struct MLIQHeader {
    /// Number of vertices in X direction
    x_verts: u32,

    /// Number of vertices in Y direction
    y_verts: u32,

    /// Number of tiles in X direction
    x_tiles: u32,

    /// Number of tiles in Y direction
    y_tiles: u32,

    /// Base coordinates
    base_coords: [f32; 3],

    /// Material ID (0 = water, 1 = ocean, 2 = magma, 3 = slime)
    material_id: u16,
}

#[repr(C, packed)]
struct MLIQVertex {
    /// Height or depth value
    height: f32,
}

#[repr(C, packed)]
struct MLIQTile {
    /// 0 = no liquid, 1 = has liquid
    liquid: u8,
}
```

#### MORI - Triangle Strip Indices

Triangle strip indices for optimized rendering.

```rust
/// MORI contains u16 indices for triangle strips
fn parse_mori(data: &[u8]) -> Vec<u16> {
    data.chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect()
}
```

#### MORB - Additional Render Batches

Additional render batch information.

```rust
#[repr(C, packed)]
struct MORBEntry {
    start_index: u16,
    index_count: u16,
    min_index: u16,
    max_index: u16,
    flags: u8,
    material_id: u8,
}
```

#### MOTA - Map Object Tangent Array

Tangent data for normal mapping.

```rust
#[repr(C, packed)]
struct MOTAEntry {
    /// Tangent vector
    tangent: [i16; 4], // Packed as 16-bit signed integers
}

impl MOTAEntry {
    /// Convert packed tangent to normalized float vector
    pub fn to_float_tangent(&self) -> [f32; 4] {
        [
            self.tangent[0] as f32 / 32767.0,
            self.tangent[1] as f32 / 32767.0,
            self.tangent[2] as f32 / 32767.0,
            self.tangent[3] as f32 / 32767.0,
        ]
    }
}
```

#### MOGX - Query Face Start

Query face start index for modern collision (Dragonflight+). ✅ **Implemented**

```rust
/// MOGX contains a single u32 query face start index
fn parse_mogx(data: &[u8]) -> u32 {
    u32::from_le_bytes([data[0], data[1], data[2], data[3]])
}
```

#### MPY2 - Extended Material Info

Extended material information for modern rendering (Dragonflight+). ✅ **Implemented**

```rust
#[repr(C, packed)]
struct MPY2Entry {
    flags: u16,
    material_id: u16,
}
```

#### MOVX - Extended Vertex Indices

Extended vertex indices allowing larger index values (Shadowlands+). ✅ **Implemented**

```rust
/// MOVX contains u32 indices instead of u16, allowing larger meshes
fn parse_movx(data: &[u8]) -> Vec<u32> {
    data.chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}
```

#### MOQG - Query Faces

Query face ground type values for collision detection (Dragonflight+). ✅ **Implemented**

```rust
/// MOQG contains an array of u32 ground type values
fn parse_moqg(data: &[u8]) -> Vec<u32> {
    data.chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}
```

#### MOBS - Map Object Shadow Batches

Shadow batch information for shadow rendering. ⚠️ **Format Specification Only**

```rust
#[repr(C, packed)]
struct MOBSEntry {
    /// Same structure as MOBA
    start_index: u16,
    index_count: u16,
    min_index: u16,
    max_index: u16,
    flags: u8,
    material_id: u8,
}
```

#### Additional Group Chunks (Modern Versions)

- **MDAL** - Unknown chunk
- **MOPL** - Terrain Cutting Planes (4.x+)
- **MOPB** - Prepass Batches
- **MOLS** - Spot Lights
- **MOLP** - Light Page
- **MLSP** - Shadowmap LSP
- **MLSS** - Shadowmap Shadows
- **MLSK** - Shadowmap LSSK
- **MOP2** - Portal Information 2 (7.x+)
- **MOS2** - Skybox 2
- **MPVD** - Particle Volume Data (7.x+)
- **MAVD** - Ambient Volume Data
- **MBVD** - Baked Volume Data

## Coordinate System

World of Warcraft uses a right-handed coordinate system:

- **X-axis**: North (positive) to South (negative)
- **Y-axis**: West (positive) to East (negative)
- **Z-axis**: Up (positive) to Down (negative)

WMO local coordinates are transformed to world coordinates using placement
information from ADT files.

## Material System

Materials in WMOs control how surfaces are rendered:

```rust
pub enum BlendMode {
    Opaque = 0,
    AlphaKey = 1,
    Alpha = 2,
    NoAlphaAdd = 3,
    Add = 4,
    Mod = 5,
    Mod2x = 6,
    ModAdd = 7,
    InvSrcAlphaAdd = 8,
    InvSrcAlphaOpaque = 9,
    SrcAlphaOpaque = 10,
    NoAlphaAddAlpha = 11,
    ConstantAlpha = 12,
}

pub fn apply_blend_mode(blend_mode: BlendMode) {
    match blend_mode {
        BlendMode::Opaque => {
            // src = 1, dst = 0
        }
        BlendMode::AlphaKey => {
            // src = 1, dst = 0 (with alpha test)
        }
        BlendMode::Alpha => {
            // src = srcAlpha, dst = invSrcAlpha
        }
        BlendMode::Add => {
            // src = 1, dst = 1
        }
        // ... etc
    }
}
```

## Portal System

Portals connect indoor groups for visibility culling:

```rust
pub struct Portal {
    pub vertices: Vec<[f32; 3]>,
    pub plane: [f32; 4],
    pub groups: [u16; 2], // Groups on each side
}

impl Portal {
    /// Check if a point is on the positive side of the portal
    pub fn is_point_on_positive_side(&self, point: &[f32; 3]) -> bool {
        let dot = point[0] * self.plane[0]
                + point[1] * self.plane[1]
                + point[2] * self.plane[2];
        dot >= self.plane[3]
    }

    /// Check if portal is visible from a viewpoint
    pub fn is_visible_from(&self, viewpoint: &[f32; 3], view_dir: &[f32; 3]) -> bool {
        // Check if viewpoint is on positive side
        if !self.is_point_on_positive_side(viewpoint) {
            return false;
        }

        // Check if portal faces viewpoint
        let normal = [self.plane[0], self.plane[1], self.plane[2]];
        let dot = normal[0] * view_dir[0]
                + normal[1] * view_dir[1]
                + normal[2] * view_dir[2];
        dot < 0.0
    }
}
```

## Lighting System

WMO lighting combines several elements:

1. **Vertex Colors**: Baked lighting stored per-vertex
2. **Dynamic Lights**: Point and spot lights that affect nearby geometry
3. **Ambient Color**: Global ambient light color

```rust
pub fn calculate_vertex_lighting(
    vertex_pos: &[f32; 3],
    vertex_normal: &[f32; 3],
    vertex_color: &[f32; 4],
    lights: &[MOLTEntry],
    ambient: &[f32; 3],
) -> [f32; 3] {
    let mut final_color = [
        ambient[0] * vertex_color[0],
        ambient[1] * vertex_color[1],
        ambient[2] * vertex_color[2],
    ];

    for light in lights {
        match light.light_type {
            0 => {
                // Ambient light
                final_color[0] += light.intensity * ((light.color >> 16) & 0xFF) as f32 / 255.0;
                final_color[1] += light.intensity * ((light.color >> 8) & 0xFF) as f32 / 255.0;
                final_color[2] += light.intensity * (light.color & 0xFF) as f32 / 255.0;
            }
            1 => {
                // Directional light
                let light_dir = normalize(&light.position);
                let n_dot_l = dot_product(vertex_normal, &light_dir).max(0.0);
                final_color[0] += n_dot_l * light.intensity * ((light.color >> 16) & 0xFF) as f32 / 255.0;
                final_color[1] += n_dot_l * light.intensity * ((light.color >> 8) & 0xFF) as f32 / 255.0;
                final_color[2] += n_dot_l * light.intensity * (light.color & 0xFF) as f32 / 255.0;
            }
            2 | 3 => {
                // Point or spot light
                let light_vec = sub_vec3(&light.position, vertex_pos);
                let dist = length(&light_vec);

                if dist < light.attenuation_end {
                    let light_dir = normalize(&light_vec);
                    let n_dot_l = dot_product(vertex_normal, &light_dir).max(0.0);

                    let attenuation = if dist < light.attenuation_start {
                        1.0
                    } else {
                        1.0 - (dist - light.attenuation_start)
                            / (light.attenuation_end - light.attenuation_start)
                    };

                    final_color[0] += attenuation * n_dot_l * light.intensity
                        * ((light.color >> 16) & 0xFF) as f32 / 255.0;
                    final_color[1] += attenuation * n_dot_l * light.intensity
                        * ((light.color >> 8) & 0xFF) as f32 / 255.0;
                    final_color[2] += attenuation * n_dot_l * light.intensity
                        * (light.color & 0xFF) as f32 / 255.0;
                }
            }
            _ => {}
        }
    }

    final_color
}
```

## References

1. [WoWDev Wiki - WMO Format](https://wowdev.wiki/WMO)
2. [WoWDev Wiki - WMO/v17](https://wowdev.wiki/WMO/v17)
3. [Ladislav Zezula's WMO Documentation](http://www.zezula.net/en/wow/wmo.html)
4. [libwarcraft WMO Implementation](https://github.com/WowDevTools/libwarcraft)
5. [Neo (WoW Model Viewer) Source](https://bitbucket.org/siliconknight/neo)
6. [WoWMapViewer Source Code](https://github.com/Marlamin/WoWMapViewer)
7. [PyWoW WMO Module](https://github.com/wowdev/pywowlib)

## Key Findings from Empirical Analysis

### Format Stability
- **Version Consistency**: All WMO files from 1.12.1 through 3.3.5a use version 17
- **Chunk Order**: The 17 core chunks always appear in the same order
- **Backward Compatibility**: No breaking changes detected between versions
- **Extension Model**: New features added via optional chunks, not format changes

### Common Patterns
- **Empty Chunks**: MOVV and MOVB frequently have 0 size (no visibility blocking)
- **Single Doodad Set**: Most WMOs have just one doodad set (32 bytes)
- **Skybox**: Usually empty (4 bytes of zeros) for indoor WMOs
- **Consistent Sizes**: MOHD always 64 bytes, MVER always 4 bytes

### Implementation Priority
Based on chunk frequency and importance:
1. **Essential**: MVER, MOHD, MOTX, MOMT, MOGI, MOGN (basic structure)
2. **Important**: MODD, MODN, MODS (doodad placement)
3. **Lighting**: MOLT, MFOG (visual quality)
4. **Advanced**: MOPV, MOPT, MOPR (portal culling)
5. **Optional**: MOVV, MOVB (rarely used)

### File Size Distribution
- **Root files**: Typically 50KB - 500KB
- **Group files**: Typically 100KB - 2MB per group
- **Texture paths**: Average 500-5000 bytes
- **Doodad data**: Can be 80KB+ for complex WMOs

## See Also

- [BLP Format](blp.md) - Texture format used by WMO
- [M2 Format](m2.md) - Doodads placed in WMO
- [ADT Format](../world-data/adt.md) - Terrain that WMOs sit on
- [WMO Rendering Guide](../../guides/wmo-rendering.md)

        Ok(wmo)
    }
}

pub struct WMOGroup {
    pub header: MOGPHeader,
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tex_coords: Vec<Vec<[f32; 2]>>,
    pub vertex_colors: Option<Vec<[f32; 4]>>,
    pub triangles: Vec<[u16; 3]>,
    pub materials: Vec<u8>,
    pub render_batches: Vec<MOBAEntry>,
    pub bsp_tree: Option<BSPTree>,
    pub liquid: Option<LiquidData>,
}

impl WMOGroup {
    pub fn read<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
        let chunk = Chunk::read(reader)?;
        if chunk.header.id_string() != "MOGP" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Expected MOGP chunk",
            ));
        }

        let mut group_reader = io::Cursor::new(chunk.data);
        let header: MOGPHeader = read_struct(&mut group_reader)?;

        let mut group = WMOGroup {
            header,
            vertices: Vec::new(),
            normals: Vec::new(),
            tex_coords: Vec::new(),
            vertex_colors: None,
            triangles: Vec::new(),
            materials: Vec::new(),
            render_batches: Vec::new(),
            bsp_tree: None,
            liquid: None,
        };

        // Read sub-chunks
        while group_reader.position() < group_reader.get_ref().len() as u64 {
            let sub_chunk = Chunk::read(&mut group_reader)?;

            match sub_chunk.header.id_string().as_str() {
                "MOVT" => {
                    let verts: Vec<MOVTEntry> = read_array(&sub_chunk.data)?;
                    group.vertices = verts.iter().map(|v| v.position).collect();
                }
                "MONR" => {
                    let norms: Vec<MONREntry> = read_array(&sub_chunk.data)?;
                    group.normals = norms.iter().map(|n| n.normal).collect();
                }
                "MOTV" => {
                    let coords: Vec<MOTVEntry> = read_array(&sub_chunk.data)?;
                    let tex_coords = coords.iter().map(|tc| [tc.u, tc.v]).collect();
                    group.tex_coords.push(tex_coords);
                }
                "MOVI" => {
                    group.triangles = parse_movi(&sub_chunk.data);
                }
                "MOPY" => {
                    let mopy: Vec<MOPYEntry> = read_array(&sub_chunk.data)?;
                    group.materials = mopy.iter().map(|m| m.material_id).collect();
                }
                "MOBA" => {
                    group.render_batches = read_array(&sub_chunk.data)?;
                }
                "MOCV" => {
                    let colors: Vec<MOCVEntry> = read_array(&sub_chunk.data)?;
                    group.vertex_colors = Some(
                        colors.iter().map(|c| c.to_rgba_f32()).collect()
                    );
                }
                _ => {
                    // Unknown sub-chunk
                }
            }
        }

        Ok(group)
    }
}

/// Helper function to read a struct from bytes
fn read_struct<T>(data: &[u8]) -> io::Result<T> {
    if data.len() < std::mem::size_of::<T>() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Not enough data for struct",
        ));
    }

    unsafe {
        Ok(std::ptr::read_unaligned(data.as_ptr() as *const T))
    }
}

/// Helper function to read an array of structs
fn read_array<T>(data: &[u8]) -> io::Result<Vec<T>> {
    let item_size = std::mem::size_of::<T>();
    let count = data.len() / item_size;

    let mut result = Vec::with_capacity(count);
    for i in 0..count {
        let start = i * item_size;
        let item = read_struct(&data[start..start + item_size])?;
        result.push(item);
    }

    Ok(result)
}
```

### Render Batch Processing

```rust
pub fn process_render_batches(
    group: &WMOGroup,
    materials: &[MOMTEntry],
) -> Vec<RenderBatch> {
    let mut batches = Vec::new();

    for batch in &group.render_batches {
        let material = &materials[batch.material_id as usize];

        let indices: Vec<u32> = (batch.start_index..batch.start_index + batch.count as u32)
            .map(|i| i as u32)
            .collect();

        let render_batch = RenderBatch {
            indices,
            material_id: batch.material_id,
            blend_mode: BlendMode::from_u32(material.blend_mode),
            texture_ids: [material.texture_1, material.texture_2, material.texture_3],
            shader_flags: material.flags,
        };

        batches.push(render_batch);
    }

    batches
}
```

## Test Vectors

### Chunk Header Parsing

```rust
#[test]
fn test_chunk_header_parsing() {
    // MVER chunk header (reversed in file)
    let data = vec![0x52, 0x45, 0x56, 0x4D, 0x04, 0x00, 0x00, 0x00];
    let header = ChunkHeader {
        id: [data[0], data[1], data[2], data[3]],
        size: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
    };

    assert_eq!(header.id_string(), "MVER");
    assert_eq!(header.size, 4);
}
```

### Material Flag Tests

```rust
#[test]
fn test_material_flags() {
    let material = MOMTEntry {
        flags: MOMTEntry::SHADER_TWO_SIDED | MOMTEntry::SHADER_UNFOGGED,
        // ... other fields
    };

    assert!(material.flags & MOMTEntry::SHADER_TWO_SIDED != 0);
    assert!(material.flags & MOMTEntry::SHADER_UNFOGGED != 0);
    assert!(material.flags & MOMTEntry::SHADER_METAL == 0);
}
```

### BSP Tree Traversal

```rust
#[test]
fn test_bsp_ray_intersection() {
    let vertices = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
    ];

    let faces = vec![[0, 1, 2]];

    let nodes = vec![
        MOBNNode {
            flags: MOBNNode::FLAG_LEAF,
            neg_child: -1,
            pos_child: -1,
            face_count: 1,
            face_start: 0,
            plane_dist: 0.0,
        },
    ];

    let face_indices = vec![0];

    let bsp = BSPTree { nodes, face_indices };

    // Ray pointing at triangle
    let t = bsp.ray_intersect(
        &[0.25, 0.25, 1.0],
        &[0.0, 0.0, -1.0],
        &faces,
        &vertices,
    );

    assert!(t.is_some());
    assert!((t.unwrap() - 1.0).abs() < 0.001);

    // Ray missing triangle
    let t = bsp.ray_intersect(
        &[2.0, 2.0, 1.0],
        &[0.0, 0.0, -1.0],
        &faces,
        &vertices,
    );

    assert!(t.is_none());
}
```

### Portal Visibility

```rust
#[test]
fn test_portal_visibility() {
    let portal = Portal {
        vertices: vec![
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
        ],
        plane: [0.0, 0.0, 1.0, 0.0], // Facing +Z
        groups: [0, 1],
    };

    // Viewpoint on positive side, looking at portal
    assert!(portal.is_visible_from(&[0.0, 0.0, 1.0], &[0.0, 0.0, -1.0]));

    // Viewpoint on negative side
    assert!(!portal.is_visible_from(&[0.0, 0.0, -1.0], &[0.0, 0.0, 1.0]));

    // Viewpoint on positive side but looking away
    assert!(!portal.is_visible_from(&[0.0, 0.0, 1.0], &[0.0, 0.0, 1.0]));
}
```

## Common Pitfalls

1. **Byte Order**: All multi-byte values are little-endian
2. **Chunk Alignment**: Some chunks may have padding to align to 4-byte boundaries
3. **String Parsing**: Strings in MOTX, MOGN, MODN are null-terminated and can be empty
4. **Group Numbering**: Group files are numbered from 000, not 001
5. **Coordinate System**: Remember WoW uses a right-handed system with Y pointing north
6. **Material IDs**: Material IDs in groups index into the root file's MOMT chunk
7. **BSP Face Indices**: BSP face indices refer to triangles, not vertices
8. **Portal Normals**: Portal plane normals point toward the positive side
9. **Vertex Colors**: MOCV can have 1 or 2 sets of colors (check MOGI flags)
10. **Texture Coordinates**: Groups can have up to 3 sets of texture coordinates

## References

1. [WoWDev Wiki - WMO Format](https://wowdev.wiki/WMO)
2. [WoWDev Wiki - WMO/v17](https://wowdev.wiki/WMO/v17)
3. [Ladislav Zezula's WMO Documentation](http://www.zezula.net/en/wow/wmo.html)
4. [libwarcraft WMO Implementation](https://github.com/WowDevTools/libwarcraft)
5. [Neo (WoW Model Viewer) Source](https://bitbucket.org/siliconknight/neo)
6. [WoWMapViewer Source Code](https://github.com/Marlamin/WoWMapViewer)
7. [PyWoW WMO Module](https://github.com/wowdev/pywowlib)

### Implementation References

- **C++**: [StormLib](https://github.com/ladislav-zezula/StormLib) for MPQ reading
- **C#**: [libwarcraft](https://github.com/WowDevTools/libwarcraft) for complete WMO support
- **Python**: [pywow](https://github.com/wowdev/pywowlib) for WMO parsing
- **JavaScript**: [tswow](https://github.com/tswow/tswow) for modding framework

This documentation is based on reverse engineering efforts by the WoW modding community and may contain inaccuracies. Always verify against known working implementations when developing WMO parsing code.

## Key Findings from Empirical Analysis

### Format Stability
- **Version Consistency**: All WMO files from 1.12.1 through 3.3.5a use version 17
- **Chunk Order**: The 17 core chunks always appear in the same order
- **Backward Compatibility**: No breaking changes detected between versions
- **Extension Model**: New features added via optional chunks, not format changes

### Common Patterns
- **Empty Chunks**: MOVV and MOVB frequently have 0 size (no visibility blocking)
- **Single Doodad Set**: Most WMOs have just one doodad set (32 bytes)
- **Skybox**: Usually empty (4 bytes of zeros) for indoor WMOs
- **Consistent Sizes**: MOHD always 64 bytes, MVER always 4 bytes

### Implementation Priority
Based on chunk frequency and importance:
1. **Essential**: MVER, MOHD, MOTX, MOMT, MOGI, MOGN (basic structure)
2. **Important**: MODD, MODN, MODS (doodad placement)
3. **Lighting**: MOLT, MFOG (visual quality)
4. **Advanced**: MOPV, MOPT, MOPR (portal culling)
5. **Optional**: MOVV, MOVB (rarely used)

### File Size Distribution
- **Root files**: Typically 50KB - 500KB
- **Group files**: Typically 100KB - 2MB per group
- **Texture paths**: Average 500-5000 bytes
- **Doodad data**: Can be 80KB+ for complex WMOs

## See Also

- [BLP Format](blp.md) - Texture format used by WMO
- [M2 Format](m2.md) - Doodads placed in WMO
- [ADT Format](../world-data/adt.md) - Terrain that WMOs sit on
- [WMO Rendering Guide](../../guides/wmo-rendering.md)
