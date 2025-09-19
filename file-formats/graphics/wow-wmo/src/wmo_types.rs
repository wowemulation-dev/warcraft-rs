use std::collections::HashMap;

use crate::types::{BoundingBox, Color, Vec3};
use crate::version::WmoVersion;
use crate::wmo_group_types::WmoGroupFlags;
use bitflags::bitflags;

/// Represents a WMO root file
#[derive(Debug)]
pub struct WmoRoot {
    /// WMO version
    pub version: WmoVersion,

    /// List of materials
    pub materials: Vec<WmoMaterial>,

    /// List of groups
    pub groups: Vec<WmoGroupInfo>,

    /// List of portals
    pub portals: Vec<WmoPortal>,

    /// List of portal references
    pub portal_references: Vec<WmoPortalReference>,

    /// List of visible block lists
    pub visible_block_lists: Vec<Vec<u16>>,

    /// List of lights
    pub lights: Vec<WmoLight>,

    /// List of doodad definitions
    pub doodad_defs: Vec<WmoDoodadDef>,

    /// List of doodad sets
    pub doodad_sets: Vec<WmoDoodadSet>,

    /// Global bounding box
    pub bounding_box: BoundingBox,

    /// List of textures
    pub textures: Vec<String>,

    /// Map of texture offsets to indices in the textures vector
    pub texture_offset_index_map: HashMap<u32, u32>,

    /// Model header info
    pub header: WmoHeader,

    /// Skybox model path, if any
    pub skybox: Option<String>,

    /// Convex volume planes (Cataclysm+, transport WMOs)
    /// Contains collision geometry for advanced physics interactions
    pub convex_volume_planes: Option<WmoConvexVolumePlanes>,
}

/// WMO header information
#[derive(Debug, Clone)]
pub struct WmoHeader {
    /// Number of materials
    pub n_materials: u32,

    /// Number of groups
    pub n_groups: u32,

    /// Number of portals
    pub n_portals: u32,

    /// Number of lights
    pub n_lights: u32,

    /// Number of doodad names
    pub n_doodad_names: u32,

    /// Number of doodad defs
    pub n_doodad_defs: u32,

    /// Number of doodad sets
    pub n_doodad_sets: u32,

    /// Global WMO flags
    pub flags: WmoFlags,

    /// Ambient color
    pub ambient_color: Color,
}

bitflags! {
    /// Global WMO flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct WmoFlags: u32 {
        /// Contains vertex colors
        const HAS_VERTEX_COLORS = 0x01;
        /// Outdoor
        const OUTDOOR = 0x02;
        /// Does not cast shadows on terrain
        const NO_TERRAIN_SHADOWS = 0x04;
        /// Contains groups with liquids
        const HAS_LIQUIDS = 0x08;
        /// Indoor map
        const INDOOR_MAP = 0x10;
        /// Contains skybox (introduced in WotLK)
        const HAS_SKYBOX = 0x20;
        /// Needs rendering during a special pass
        const SPECIAL_PASS = 0x40;
        /// Uses scene graph for rendering
        const USE_SCENE_GRAPH = 0x80;
        /// Shows minimap in outdoor
        const SHOW_MINIMAP_OUTDOOR = 0x100;
        /// Mount allowed inside this WMO
        const MOUNT_ALLOWED = 0x200;
    }
}

/// Represents a WMO material
#[derive(Debug, Clone)]
pub struct WmoMaterial {
    /// Material flags
    pub flags: WmoMaterialFlags,

    /// Texture 1 index in MOTX chunk
    pub shader: u32,

    /// Blend mode
    pub blend_mode: u32,

    /// Texture 1 offset in MOTX chunk
    pub texture1: u32,

    /// Emissive color
    pub emissive_color: Color,

    /// Sidn color
    pub sidn_color: Color,

    /// Framebuffer blend
    pub framebuffer_blend: Color,

    /// Texture 2 offset in MOTX chunk
    pub texture2: u32,

    /// Diffuse color
    pub diffuse_color: Color,

    /// Ground type
    pub ground_type: u32,
}

bitflags! {
    /// WMO material flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct WmoMaterialFlags: u32 {
        /// Unlit
        const UNLIT = 0x01;
        /// Unfogged
        const UNFOGGED = 0x02;
        /// Two-sided
        const TWO_SIDED = 0x04;
        /// Exterior light
        const EXTERIOR_LIGHT = 0x08;
        /// Window light
        const WINDOW_LIGHT = 0x10;
        /// Use clamp s addressing
        const CLAMP_S = 0x20;
        /// Use clamp t addressing
        const CLAMP_T = 0x40;
        /// Unused 1
        const UNUSED1 = 0x80;
        /// Shadow batch 1
        const SHADOW_BATCH_1 = 0x100;
        /// Shadow batch 2
        const SHADOW_BATCH_2 = 0x200;
        /// Unused 2
        const UNUSED2 = 0x400;
        /// Unused 3
        const UNUSED3 = 0x800;
    }
}

impl WmoMaterial {
    pub fn get_texture1_index(&self, texture_offset_index_map: &HashMap<u32, u32>) -> u32 {
        texture_offset_index_map
            .get(&self.texture1)
            .copied()
            .unwrap()
    }

    pub fn get_texture2_index(&self, texture_offset_index_map: &HashMap<u32, u32>) -> u32 {
        texture_offset_index_map
            .get(&self.texture2)
            .copied()
            .unwrap()
    }
}

/// Represents information about a WMO group
#[derive(Debug, Clone)]
pub struct WmoGroupInfo {
    /// Group flags
    pub flags: WmoGroupFlags,

    /// Bounding box
    pub bounding_box: BoundingBox,

    /// Group name (often contains a numeric index)
    pub name: String,
}

/// Represents a WMO portal
#[derive(Debug, Clone)]
pub struct WmoPortal {
    /// Portal vertices
    pub vertices: Vec<Vec3>,

    /// Normal vector
    pub normal: Vec3,
}

/// Represents a WMO portal reference
#[derive(Debug, Clone)]
pub struct WmoPortalReference {
    /// Portal index
    pub portal_index: u16,

    /// Group index
    pub group_index: u16,

    /// Side (0 or 1)
    pub side: u16,
}

/// Represents a WMO light
#[derive(Debug, Clone)]
pub struct WmoLight {
    /// Light type
    pub light_type: WmoLightType,

    /// Light position
    pub position: Vec3,

    /// Light color
    pub color: Color,

    /// Light intensity
    pub intensity: f32,

    /// Attenuation start
    pub attenuation_start: f32,

    /// Attenuation end
    pub attenuation_end: f32,

    /// Use attenuation
    pub use_attenuation: bool,

    /// Additional properties based on light type
    pub properties: WmoLightProperties,
}

/// Type of WMO light
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WmoLightType {
    /// Omnidirectional point light
    Omni = 0,

    /// Spotlight
    Spot = 1,

    /// Directional light
    Directional = 2,

    /// Ambient light
    Ambient = 3,
}

impl WmoLightType {
    /// Convert a raw value to a WmoLightType
    pub fn from_raw(raw: u8) -> Option<Self> {
        match raw {
            0 => Some(Self::Omni),
            1 => Some(Self::Spot),
            2 => Some(Self::Directional),
            3 => Some(Self::Ambient),
            // Unknown light types default to Omni
            _ => {
                // Log warning but continue with default
                tracing::warn!("Unknown light type {}, defaulting to Omni", raw);
                Some(Self::Omni)
            }
        }
    }
}

/// Additional light properties depending on light type
#[derive(Debug, Clone)]
pub enum WmoLightProperties {
    /// Omni light properties (none)
    Omni,

    /// Spotlight properties
    Spot {
        /// Direction
        direction: Vec3,

        /// Hotspot angle (inner cone)
        hotspot: f32,

        /// Falloff angle (outer cone)
        falloff: f32,
    },

    /// Directional light properties
    Directional {
        /// Direction
        direction: Vec3,
    },

    /// Ambient light properties (none)
    Ambient,
}

/// Represents a WMO doodad definition
#[derive(Debug, Clone)]
pub struct WmoDoodadDef {
    /// Doodad name offset in MODN chunk
    pub name_offset: u32,

    /// Doodad position
    pub position: Vec3,

    /// Doodad orientation as quaternion
    pub orientation: [f32; 4],

    /// Doodad scale factor
    pub scale: f32,

    /// Doodad color
    pub color: Color,

    /// Doodad set index
    pub set_index: u16,
}

/// Represents a WMO doodad set
#[derive(Debug, Clone)]
pub struct WmoDoodadSet {
    /// Set name
    pub name: String,

    /// Start doodad index
    pub start_doodad: u32,

    /// Number of doodads in this set
    pub n_doodads: u32,
}

/// Represents a convex volume plane (MCVP chunk)
/// Added in Cataclysm for transport WMOs and world objects requiring collision
/// Based on empirical analysis: typically 496 bytes in transport WMOs
#[derive(Debug, Clone)]
pub struct WmoConvexVolumePlane {
    /// Plane normal vector
    pub normal: Vec3,

    /// Distance from origin along the normal
    pub distance: f32,

    /// Plane flags (usage unknown)
    pub flags: u32,
}

/// Container for MCVP chunk data
/// Found in Cataclysm+ WMOs, particularly transport objects like ships
#[derive(Debug, Clone)]
pub struct WmoConvexVolumePlanes {
    /// List of convex volume planes
    /// Each plane defines a clipping boundary for the WMO collision system
    pub planes: Vec<WmoConvexVolumePlane>,
}
