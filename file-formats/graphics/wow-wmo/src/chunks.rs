use std::collections::HashMap;

use binrw::BinRead;

/// MOMT - Materials chunk
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MomtEntry {
    pub flags: u32,
    pub shader: u32,
    pub blend_mode: u32,
    pub texture_1: u32,
    pub emissive_color: [u8; 4],
    pub frame_emissive_color: [u8; 4],
    pub texture_2: u32,
    pub diff_color: [u8; 4],
    pub ground_type: u32,
    pub texture_3: u32,
    pub color_2: u32,
    pub flags_2: u32,
    #[br(count = 16)]
    pub runtime_data: Vec<u8>,
}

impl MomtEntry {
    pub fn get_texture1_index(&self, texture_offset_index_map: &HashMap<u32, u32>) -> u32 {
        texture_offset_index_map
            .get(&self.texture_1)
            .copied()
            .unwrap()
    }

    pub fn get_texture2_index(&self, texture_offset_index_map: &HashMap<u32, u32>) -> u32 {
        texture_offset_index_map
            .get(&self.texture_2)
            .copied()
            .unwrap()
    }

    pub fn get_texture3_index(&self, texture_offset_index_map: &HashMap<u32, u32>) -> u32 {
        texture_offset_index_map
            .get(&self.texture_3)
            .copied()
            .unwrap()
    }
}

/// MOGN - Group names chunk
#[derive(Debug, Clone)]
pub struct Mogn {
    pub names: Vec<String>,
}

impl Mogn {
    pub fn parse(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut names = Vec::new();
        let mut start = 0;

        for i in 0..data.len() {
            if data[i] == 0 {
                if i > start {
                    let name = String::from_utf8(data[start..i].to_vec())?;
                    names.push(name);
                }
                start = i + 1;
            }
        }

        Ok(Self { names })
    }
}

/// MOGI - Group information chunk
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MogiEntry {
    pub flags: u32,
    pub bounding_box_min: [f32; 3],
    pub bounding_box_max: [f32; 3],
    pub name_offset: i32,
}

/// MOLT - Lights chunk (48 bytes per entry)
///
/// On-disk layout matches SMOLight from BN/wiki:
///   +0x00 type (u8), +0x01 atten (u8), +0x02 pad (u8\[2\]),
///   +0x04 color (CImVector), +0x08 position (C3Vector),
///   +0x14 intensity (f32), +0x18 rotation (C4Quaternion),
///   +0x28 attenStart (f32), +0x2C attenEnd (f32)
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MoltEntry {
    pub light_type: u8,
    pub use_attenuation: u8,
    pub padding: [u8; 2],
    pub color: [u8; 4],
    pub position: [f32; 3],
    pub intensity: f32,
    pub rotation: [f32; 4],
    pub attenuation_start: f32,
    pub attenuation_end: f32,
}

/// MODS - Doodad sets chunk
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct ModsEntry {
    pub name: [u8; 20],
    pub start_index: u32,
    pub count: u32,
    pub padding: u32,
}

/// MODD - Doodad definitions chunk
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct ModdEntry {
    pub name_index_and_flags: u32, // nameIndex is bits 0-23, flags are bits 24-31
    pub position: [f32; 3],        // (X, Z, -Y)
    pub orientation: [f32; 4],     // Quaternion (X, Y, Z, W)
    pub scale: f32,
    pub color: [u8; 4], // (B, G, R, A)
}

impl ModdEntry {
    /// Extract the name index (bits 0-23)
    pub fn name_index(&self) -> u32 {
        self.name_index_and_flags & 0x00FFFFFF
    }

    /// Check if accepts projected textures (bit 24)
    pub fn accepts_proj_tex(&self) -> bool {
        (self.name_index_and_flags & 0x01000000) != 0
    }

    /// Check if uses interior lighting (bit 25)
    pub fn uses_interior_lighting(&self) -> bool {
        (self.name_index_and_flags & 0x02000000) != 0
    }
}

/// MFOG - Fog chunk
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MfogEntry {
    pub flags: u32,
    pub position: [f32; 3],
    pub smaller_radius: f32,
    pub larger_radius: f32,
    pub fog_end: f32,
    pub fog_start_multiplier: f32,
    pub color_1: [u8; 4],
    pub color_2: [u8; 4],
}

/// MCVP - Convex volume planes (Cataclysm+)
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct McvpEntry {
    pub plane: [f32; 4], // Ax + By + Cz + D = 0
}

// Group file chunks

/// MOPY - Material info for triangles
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MopyEntry {
    pub flags: u8,
    pub material_id: u8,
}

/// MOVI - Vertex indices for triangles
pub type MoviEntry = u16; // Each index is a u16

/// MOVT - Vertex positions
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MovtEntry {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// MONR - Normals
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MonrEntry {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// MOTV - Texture coordinates
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MotvEntry {
    pub u: f32,
    pub v: f32,
}

/// MOBA - Render batches
/// Structure for WoW versions ≥ Vanilla (our supported range: 1.12.1 to 5.4.8)
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MobaEntry {
    /// Bounding box for culling (bx, by, bz)
    pub bounding_box_min: [i16; 3],
    /// Bounding box for culling (tx, ty, tz)
    pub bounding_box_max: [i16; 3],
    /// Index of the first face index used in MOVI
    pub start_index: u32,
    /// Number of MOVI indices used
    pub count: u16,
    /// Index of the first vertex used in MOVT
    pub min_index: u16,
    /// Index of the last vertex used (batch includes this one)
    pub max_index: u16,
    /// Batch flags
    pub flags: u8,
    /// Material index in MOMT
    pub material_id: u8,
}

/// MOLR - Light references
pub type MolrEntry = u16; // Light index reference

/// MODR - Doodad references
pub type ModrEntry = u16; // Doodad index reference

/// MOBN - BSP tree nodes
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MobnEntry {
    pub flags: u16,
    pub neg_child: i16,
    pub pos_child: i16,
    pub n_faces: u16,
    pub face_start: u32,
    pub plane_distance: f32,
}

/// MOBR - BSP tree face indices
pub type MobrEntry = u16; // Face index

/// MOCV - Vertex colors
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MocvEntry {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8,
}

/// MLIQ - Liquid data
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MliqHeader {
    pub x_tiles: u32,
    pub y_tiles: u32,
    pub x_corner: f32,
    pub y_corner: f32,
    pub liquid_type: u32,
    pub material_id: u32,
}

/// MLIQ vertex entry
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MliqVertex {
    pub position: [f32; 3],
    pub texture_coord: [f32; 2],
}

/// MOTX - Texture names (parsed as string list)
#[derive(Debug, Clone)]
pub struct Motx {
    pub textures: Vec<String>,
    pub texture_offset_index_map: HashMap<u32, u32>,
}

impl Motx {
    pub fn parse(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut textures = Vec::new();
        let mut start = 0;
        let mut texture_offset_index_map = HashMap::new();

        for i in 0..data.len() {
            if data[i] == 0 {
                if i > start {
                    let texture = String::from_utf8(data[start..i].to_vec())?;
                    textures.push(texture);
                    texture_offset_index_map.insert(start as u32, textures.len() as u32 - 1);
                }
                start = i + 1;
            }
        }

        Ok(Self {
            textures,
            texture_offset_index_map,
        })
    }
}

/// MOSB - Skybox name
#[derive(Debug, Clone)]
pub struct Mosb {
    pub skybox: Option<String>,
}

impl Mosb {
    pub fn parse(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if data.is_empty() {
            return Ok(Self { skybox: None });
        }

        // Find null terminator
        let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
        if end > 0 {
            let skybox = String::from_utf8(data[..end].to_vec())?;
            Ok(Self {
                skybox: Some(skybox),
            })
        } else {
            Ok(Self { skybox: None })
        }
    }
}

/// MOPV - Portal vertices
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MopvEntry {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// MOPT - Portal information
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MoptEntry {
    pub start_vertex: u16,
    pub n_vertices: u16,
    pub normal: MopvEntry, // Reusing Vec3 structure
    pub distance: f32,
}

/// MOPR - Portal references
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MoprEntry {
    pub portal_index: u16,
    pub group_index: u16,
    pub side: i16,
    pub padding: u16,
}

/// MOVV - Visible block vertices
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MovvEntry {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// MOVB - Visible block list
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MovbEntry {
    pub start_vertex: u16,
    pub vertex_count: u16,
}

/// MODN - Doodad names (parsed as string list)
#[derive(Debug, Clone)]
pub struct Modn {
    pub names: Vec<String>,
}

/// MOMO - Alpha version container chunk (version 14 only)
/// Container chunk that wraps other chunks in early WoW versions
#[derive(Debug, Clone)]
pub struct MomoEntry {
    // No additional data - acts as container for other chunks
}

/// MOM3 - New materials (WarWithin+)
#[derive(Debug, Clone)]
pub struct Mom3Entry {
    // m3SI structure - defines new materials
    // Structure details may vary, treated as opaque for now
    pub data: Vec<u8>,
}

/// MOUV - UV transformations (Legion+)
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MouvEntry {
    pub translation_speed: [[f32; 2]; 2], // 2 C2Vectors per material
}

/// MOPE - Portal extra information (WarWithin+)
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MopeEntry {
    pub portal_index: u32, // index into MOPT
    pub unk1: u32,
    pub unk2: u32,
    pub unk3: u32,
}

/// MOLV - Light extensions (Shadowlands+)
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MolvEntry {
    pub directions: [[f32; 4]; 6], // 6 sets of C3Vector + float value
    pub unknown: [u8; 3],
    pub molt_index: u8,
}

/// MODI - Doodad file IDs (Battle for Azeroth+)
pub type ModiEntry = u32; // Doodad ID, same count as SMOHeader.nDoodadNames

/// MOGX - Query face start (Dragonflight+)
pub type MogxEntry = u32; // Query face start index

/// MPY2 - Extended material info (Dragonflight+)
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct Mpy2Entry {
    pub flags: u16,
    pub material_id: u16,
}

/// MOVX - Extended vertex indices (Shadowlands+)
/// Possible replacement for MOVI chunk allowing larger indices
pub type MovxEntry = u32; // Extended vertex index (u32 instead of u16)

/// MOQG - Query faces (Dragonflight+)
pub type MoqgEntry = u32; // Ground type values

/// GFID - Group file IDs (modern WoW versions)
pub type GfidEntry = u32; // File ID for group files

/// MORI - Triangle strip indices (optimized rendering)
pub type MoriEntry = u16; // Triangle strip index

/// MORB - Additional render batches (extended batching)
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MorbEntry {
    pub start_index: u16,
    pub index_count: u16,
    pub min_index: u16,
    pub max_index: u16,
    pub flags: u8,
    pub material_id: u8,
}

/// MOTA - Tangent array (normal mapping)
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MotaEntry {
    pub tangent: [i16; 4], // Packed tangent vector (x, y, z, w)
}

impl MotaEntry {
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

/// MOBS - Shadow batches (shadow rendering)
/// Note: Based on real-world data analysis, index_count appears to use signed values
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MobsEntry {
    pub start_index: u16,
    pub index_count: i16, // Changed to signed based on data analysis
    pub min_index: u16,
    pub max_index: u16,
    pub flags: u8,
    pub material_id: u8,
}

impl Modn {
    pub fn parse(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut names = Vec::new();
        let mut start = 0;

        for i in 0..data.len() {
            if data[i] == 0 {
                if i > start {
                    let name = String::from_utf8(data[start..i].to_vec())?;
                    names.push(name);
                }
                start = i + 1;
            }
        }

        Ok(Self { names })
    }
}
