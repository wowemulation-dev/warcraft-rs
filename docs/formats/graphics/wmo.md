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
- **Features**: Portal-based visibility, BSP trees, integrated lighting, multiple
  LODs
- **Use Cases**: Buildings, dungeons, caves, large structures, instances

## Key Characteristics

- **Chunk-based format**: Similar to other Blizzard formats, using 4-character
  chunk identifiers
- **Multi-file structure**: Root file + group files (numbered _000.wmo to_999.wmo)
- **Portal-based visibility**: Interior spaces use portals for occlusion culling
- **BSP trees**: Used for collision detection and ray casting
- **Multiple LOD support**: Different detail levels for rendering optimization
- **Integrated lighting**: Baked vertex lighting and dynamic light references

## Version History

| Version | Expansion | Notable Changes |
|---------|-----------|-----------------|
| 14 | Vanilla WoW (1.x) | Original format |
| 17 | The Burning Crusade (2.x) | Added MFOG chunk, enhanced materials |
| 17 | Wrath of the Lich King (3.x) | Minor updates |
| 17 | Cataclysm (4.x) | Added terrain cutting planes |
| 17 | Mists of Pandaria (5.x) | Enhanced liquid system |
| 17 | Warlords of Draenor (6.x) | Added GFID chunk |
| 17 | Legion (7.x) | Added MOP2, particle volumes |
| 17 | Battle for Azeroth (8.x) | Enhanced shadow mapping |
| 17 | Shadowlands (9.x) | Additional volume data types |

## File Structure Overview

WMO files follow a chunk-based format where each chunk has:

- 4-byte chunk identifier (reversed in file, e.g., "REVM" for MVER)
- 4-byte chunk size (not including the 8-byte header)
- Chunk data

```rust
#[repr(C, packed)]
struct ChunkHeader {
    /// Chunk identifier (4 characters, reversed)
    id: [u8; 4],
    /// Size of chunk data in bytes
    size: u32,
}

impl ChunkHeader {
    pub fn id_string(&self) -> String {
        // Reverse the bytes to get the readable ID
        String::from_utf8_lossy(&[self.id[3], self.id[2], self.id[1], self.id[0]]).to_string()
    }
}
```

## Chunk Specifications

### Root File Chunks

The root WMO file contains global information about the entire model.

#### MVER - Version

Always the first chunk in the file.

```rust
#[repr(C, packed)]
struct MVERChunk {
    version: u32,  // Usually 17 for modern WoW
}
```

#### MOHD - Header

Contains general information about the WMO.

```rust
#[repr(C, packed)]
struct MOHDChunk {
    /// Number of textures used
    texture_count: u32,

    /// Number of groups
    group_count: u32,

    /// Number of portals
    portal_count: u32,

    /// Number of lights
    light_count: u32,

    /// Number of doodad names
    doodad_name_count: u32,

    /// Number of doodad definitions
    doodad_def_count: u32,

    /// Number of doodad sets
    doodad_set_count: u32,

    /// Ambient color (BGRA)
    ambient_color: u32,

    /// WMO ID (used in ADT files)
    wmo_id: u32,

    /// Bounding box corners
    bounding_box_min: [f32; 3],
    bounding_box_max: [f32; 3],

    /// Flags
    flags: u32,

    /// Number of Level of Detail sets
    lod_count: u16,
}

impl MOHDChunk {
    // Flag constants
    pub const FLAG_DO_NOT_ATTENUATE_VERTICES: u32 = 0x01;
    pub const FLAG_USE_UNIFIED_RENDER_PATH: u32 = 0x02;
    pub const FLAG_USE_LIQUID_FROM_DBC: u32 = 0x04;
    pub const FLAG_DO_NOT_FIX_VERTEX_COLOR_ALPHA: u32 = 0x08;
    pub const FLAG_LOD: u32 = 0x10;
    pub const FLAG_DEFAULT_MAX_LOD: u32 = 0x20;
}
```

#### MOTX - Textures

Null-terminated texture filenames used by this WMO.

```rust
/// Parse MOTX chunk data
fn parse_motx(data: &[u8]) -> Vec<String> {
    let mut textures = Vec::new();
    let mut start = 0;

    for (i, &byte) in data.iter().enumerate() {
        if byte == 0 {
            if i > start {
                if let Ok(s) = std::str::from_utf8(&data[start..i]) {
                    textures.push(s.to_string());
                }
            }
            start = i + 1;
        }
    }

    textures
}
```

#### MOMT - Materials

Material definitions for all textures.

```rust
#[repr(C, packed)]
struct MOMTEntry {
    /// Flags
    flags: u32,

    /// Specular mode
    specular_mode: u32,

    /// Blend mode
    blend_mode: u32,

    /// First texture index
    texture_1: u32,

    /// Emissive color (BGRA)
    side_dn: u32,

    /// Frame blend alpha
    frame_blend_alpha: u32,

    /// Second texture index
    texture_2: u32,

    /// Diffuse color (BGRA)
    diff_color: u32,

    /// Terrain type for footsteps
    ground_type: u32,

    /// Third texture index
    texture_3: u32,

    /// Two colors (BGRA each)
    color_2: u32,
    unknown_3: u32,

    /// Runtime data
    runtime_data: [u32; 4],
}

impl MOMTEntry {
    // Shader flags
    pub const SHADER_DIFFUSE: u32 = 0x00000001;
    pub const SHADER_SPECULAR: u32 = 0x00000002;
    pub const SHADER_METAL: u32 = 0x00000004;
    pub const SHADER_ENV: u32 = 0x00000008;
    pub const SHADER_OPAQUE: u32 = 0x00000010;
    pub const SHADER_ENV_METAL: u32 = 0x00000020;
    pub const SHADER_TWO_SIDED: u32 = 0x00000040;
    pub const SHADER_DARKENED: u32 = 0x00000080;
    pub const SHADER_UNSHADED: u32 = 0x00000100;
    pub const SHADER_NO_FADE: u32 = 0x00000200;
    pub const SHADER_UNFOGGED: u32 = 0x00000400;
    pub const SHADER_IGNORE_VERTEX_ALPHA: u32 = 0x00000800;
    pub const SHADER_IGNORE_VERTEX_COLOR: u32 = 0x00001000;
    pub const SHADER_CLAMP_S: u32 = 0x00004000;
    pub const SHADER_CLAMP_T: u32 = 0x00008000;
}
```

#### MOGN - Group Names

Null-terminated strings for group names (primarily for debugging).

```rust
/// Parse MOGN chunk data (same format as MOTX)
fn parse_mogn(data: &[u8]) -> Vec<String> {
    parse_motx(data) // Same null-terminated string format
}
```

#### MOGI - Group Information

Information about each group in the WMO.

```rust
#[repr(C, packed)]
struct MOGIEntry {
    /// Flags
    flags: u32,

    /// Bounding box for this group
    bounding_box_min: [f32; 3],
    bounding_box_max: [f32; 3],

    /// Name offset in MOGN chunk (-1 if none)
    name_offset: i32,
}

impl MOGIEntry {
    // Group flags
    pub const FLAG_HAS_BSP: u32 = 0x00000001;
    pub const FLAG_HAS_LIGHT_MAP: u32 = 0x00000002;
    pub const FLAG_HAS_VERTEX_COLORS: u32 = 0x00000004;
    pub const FLAG_OUTDOOR: u32 = 0x00000008;
    pub const FLAG_DO_NOT_USE_LOCAL_LIGHTING: u32 = 0x00000040;
    pub const FLAG_HAS_LIGHTS: u32 = 0x00000200;
    pub const FLAG_HAS_LOD: u32 = 0x00000400;
    pub const FLAG_HAS_DOODADS: u32 = 0x00000800;
    pub const FLAG_HAS_LIQUID: u32 = 0x00001000;
    pub const FLAG_INDOOR: u32 = 0x00002000;
    pub const FLAG_ALWAYS_DRAW: u32 = 0x00010000;
    pub const FLAG_HAS_THREE_MOTV: u32 = 0x00040000;
    pub const FLAG_SHOW_SKYBOX: u32 = 0x00080000;
    pub const FLAG_OCEANIC: u32 = 0x00100000;
    pub const FLAG_UNDERWATER: u32 = 0x00200000;
    pub const FLAG_HAS_TWO_MOCV: u32 = 0x01000000;
    pub const FLAG_HAS_TWO_MOTV: u32 = 0x02000000;
    pub const FLAG_FORCE_CLAMP_S_OUTDOOR: u32 = 0x04000000;
    pub const FLAG_FORCE_CLAMP_T_OUTDOOR: u32 = 0x08000000;
}
```

#### MOSB - Skybox

Skybox model filename (if present).

```rust
/// Parse MOSB chunk - contains null-terminated string
fn parse_mosb(data: &[u8]) -> Option<String> {
    if data.is_empty() {
        return None;
    }

    // Find null terminator
    let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());

    std::str::from_utf8(&data[..end])
        .ok()
        .map(|s| s.to_string())
}
```

#### MOPV - Portal Vertices

Vertices used to define portal geometry.

```rust
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct MOPVEntry {
    position: [f32; 3],
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
#[repr(C, packed)]
struct MOLTEntry {
    /// Light type: 0 = ambient, 1 = directional, 2 = point, 3 = spot
    light_type: u8,

    /// Use attenuation
    use_attenuation: u8,

    /// Padding
    _padding: [u8; 2],

    /// Color (BGRA)
    color: u32,

    /// Position
    position: [f32; 3],

    /// Intensity
    intensity: f32,

    /// Attenuation start
    attenuation_start: f32,

    /// Attenuation end
    attenuation_end: f32,
}
```

#### MODS - Doodad Sets

Doodad set definitions (e.g., "furniture", "decorations").

```rust
#[repr(C, packed)]
struct MODSEntry {
    /// Name of the set (20 bytes, null-padded)
    name: [u8; 20],

    /// Index of first doodad in this set
    start_index: u32,

    /// Number of doodads in this set
    count: u32,

    /// Padding
    _padding: u32,
}
```

#### MODN - Doodad Names

List of null-terminated doodad filenames (M2 models).

```rust
/// Parse MODN chunk - same format as MOTX
fn parse_modn(data: &[u8]) -> Vec<String> {
    parse_motx(data)
}
```

#### MODD - Doodad Definitions

Placement information for doodads.

```rust
#[repr(C, packed)]
struct MODDEntry {
    /// Index in MODN of which model to use (or 0xFFFFFFFF to use file_id)
    name_index: u32,

    /// Placement information
    position: [f32; 3],

    /// Quaternion rotation (WXYZ)
    rotation: [f32; 4],

    /// Scale factor
    scale: f32,

    /// Color tinting (BGRA)
    color: u32,
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

#### GFID - Group File IDs

File IDs for group files (modern WoW versions).

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

Material information for each triangle.

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

Vertex positions.

```rust
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct MOVTEntry {
    position: [f32; 3],
}
```

#### MONR - Normals

Vertex normals.

```rust
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct MONREntry {
    normal: [f32; 3],
}
```

#### MOTV - Texture Coordinates

Texture coordinates (can have up to 3 sets).

```rust
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct MOTVEntry {
    u: f32,
    v: f32,
}
```

#### MOBA - Render Batches

Defines how triangles are grouped for rendering.

```rust
#[repr(C, packed)]
struct MOBAEntry {
    /// Start position for the first index
    start_index: u16,

    /// Number of indices
    index_count: u16,

    /// First vertex
    min_index: u16,

    /// Last vertex
    max_index: u16,

    /// Flags
    flags: u8,

    /// Material ID from MOMT
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

#### MOBS - Map Object Shadow Batches

Shadow batch information for shadow rendering.

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

## Collision Detection

BSP trees enable efficient collision detection:

```rust
pub struct BSPTree {
    nodes: Vec<MOBNNode>,
    face_indices: Vec<u16>,
}

impl BSPTree {
    /// Ray-BSP intersection test
    pub fn ray_intersect(
        &self,
        ray_origin: &[f32; 3],
        ray_dir: &[f32; 3],
        faces: &[[u16; 3]],
        vertices: &[[f32; 3]],
    ) -> Option<f32> {
        self.ray_intersect_node(0, ray_origin, ray_dir, 0.0, f32::MAX, faces, vertices)
    }

    fn ray_intersect_node(
        &self,
        node_idx: usize,
        ray_origin: &[f32; 3],
        ray_dir: &[f32; 3],
        t_min: f32,
        t_max: f32,
        faces: &[[u16; 3]],
        vertices: &[[f32; 3]],
    ) -> Option<f32> {
        let node = &self.nodes[node_idx];

        if node.flags & MOBNNode::FLAG_LEAF != 0 {
            // Leaf node - test faces
            let mut closest_t = None;

            for i in 0..node.face_count as usize {
                let face_idx = self.face_indices[node.face_start as usize + i] as usize;
                let face = &faces[face_idx];

                if let Some(t) = ray_triangle_intersect(
                    ray_origin,
                    ray_dir,
                    &vertices[face[0] as usize],
                    &vertices[face[1] as usize],
                    &vertices[face[2] as usize],
                ) {
                    if t >= t_min && t <= t_max {
                        closest_t = Some(closest_t.map_or(t, |ct| ct.min(t)));
                    }
                }
            }

            closest_t
        } else {
            // Branch node - traverse children
            let axis = (node.flags & MOBNNode::FLAG_AXIS_MASK) as usize;
            let split_pos = node.plane_dist;

            let t_split = (split_pos - ray_origin[axis]) / ray_dir[axis];

            let first_child = if ray_dir[axis] >= 0.0 {
                node.neg_child
            } else {
                node.pos_child
            };
            let second_child = if ray_dir[axis] >= 0.0 {
                node.pos_child
            } else {
                node.neg_child
            };

            if t_split > t_max || t_split < 0.0 {
                // Only traverse near side
                if first_child >= 0 {
                    self.ray_intersect_node(
                        first_child as usize,
                        ray_origin,
                        ray_dir,
                        t_min,
                        t_max,
                        faces,
                        vertices,
                    )
                } else {
                    None
                }
            } else if t_split < t_min {
                // Only traverse far side
                if second_child >= 0 {
                    self.ray_intersect_node(
                        second_child as usize,
                        ray_origin,
                        ray_dir,
                        t_min,
                        t_max,
                        faces,
                        vertices,
                    )
                } else {
                    None
                }
            } else {
                // Traverse both sides
                let mut result = None;

                if first_child >= 0 {
                    result = self.ray_intersect_node(
                        first_child as usize,
                        ray_origin,
                        ray_dir,
                        t_min,
                        t_split,
                        faces,
                        vertices,
                    );
                }

                if second_child >= 0 {
                    let far_result = self.ray_intersect_node(
                        second_child as usize,
                        ray_origin,
                        ray_dir,
                        t_split,
                        result.unwrap_or(t_max),
                        faces,
                        vertices,
                    );

                    if let Some(t) = far_result {
                        result = Some(result.map_or(t, |rt| rt.min(t)));
                    }
                }

                result
            }
        }
    }
}

/// Ray-triangle intersection using Möller-Trumbore algorithm
fn ray_triangle_intersect(
    ray_origin: &[f32; 3],
    ray_dir: &[f32; 3],
    v0: &[f32; 3],
    v1: &[f32; 3],
    v2: &[f32; 3],
) -> Option<f32> {
    const EPSILON: f32 = 0.000001;

    let edge1 = sub_vec3(v1, v0);
    let edge2 = sub_vec3(v2, v0);

    let h = cross_product(ray_dir, &edge2);
    let a = dot_product(&edge1, &h);

    if a > -EPSILON && a < EPSILON {
        return None; // Ray parallel to triangle
    }

    let f = 1.0 / a;
    let s = sub_vec3(ray_origin, v0);
    let u = f * dot_product(&s, &h);

    if u < 0.0 || u > 1.0 {
        return None;
    }

    let q = cross_product(&s, &edge1);
    let v = f * dot_product(ray_dir, &q);

    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = f * dot_product(&edge2, &q);

    if t > EPSILON {
        Some(t)
    } else {
        None
    }
}
```

## Implementation Examples

### Complete WMO Reader

```rust
use std::io::{self, Read, Seek, SeekFrom};
use std::collections::HashMap;
use byteorder::{LittleEndian, ReadBytesExt};

pub struct WMORoot {
    pub version: u32,
    pub header: MOHDChunk,
    pub textures: Vec<String>,
    pub materials: Vec<MOMTEntry>,
    pub group_names: Vec<String>,
    pub group_info: Vec<MOGIEntry>,
    pub portals: Vec<Portal>,
    pub lights: Vec<MOLTEntry>,
    pub doodad_sets: Vec<MODSEntry>,
    pub doodad_names: Vec<String>,
    pub doodad_defs: Vec<MODDEntry>,
    pub fog: Vec<MFOGEntry>,
}

impl WMORoot {
    pub fn read<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
        let mut wmo = WMORoot {
            version: 0,
            header: unsafe { std::mem::zeroed() },
            textures: Vec::new(),
            materials: Vec::new(),
            group_names: Vec::new(),
            group_info: Vec::new(),
            portals: Vec::new(),
            lights: Vec::new(),
            doodad_sets: Vec::new(),
            doodad_names: Vec::new(),
            doodad_defs: Vec::new(),
            fog: Vec::new(),
        };

        // Read chunks until EOF
        loop {
            let chunk = match Chunk::read(reader) {
                Ok(c) => c,
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };

            match chunk.header.id_string().as_str() {
                "MVER" => {
                    wmo.version = (&chunk.data[..]).read_u32::<LittleEndian>()?;
                }
                "MOHD" => {
                    wmo.header = read_struct(&chunk.data)?;
                }
                "MOTX" => {
                    wmo.textures = parse_motx(&chunk.data);
                }
                "MOMT" => {
                    wmo.materials = read_array(&chunk.data)?;
                }
                "MOGN" => {
                    wmo.group_names = parse_mogn(&chunk.data);
                }
                "MOGI" => {
                    wmo.group_info = read_array(&chunk.data)?;
                }
                "MOPV" => {
                    let vertices: Vec<MOPVEntry> = read_array(&chunk.data)?;
                    // Process with MOPT to create portals
                }
                "MOLT" => {
                    wmo.lights = read_array(&chunk.data)?;
                }
                "MODS" => {
                    wmo.doodad_sets = read_array(&chunk.data)?;
                }
                "MODN" => {
                    wmo.doodad_names = parse_modn(&chunk.data);
                }
                "MODD" => {
                    wmo.doodad_defs = read_array(&chunk.data)?;
                }
                "MFOG" => {
                    wmo.fog = read_array(&chunk.data)?;
                }
                _ => {
                    // Unknown chunk, skip
                }
            }
        }

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

        let indices: Vec<u32> = (batch.start_index..batch.start_index + batch.index_count)
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

## See Also

- [BLP Format](blp.md) - Texture format used by WMO
- [M2 Format](m2.md) - Doodads placed in WMO
- [ADT Format](../world-data/adt.md) - Terrain that WMOs sit on
- [WMO Rendering Guide](../../guides/wmo-rendering.md)
