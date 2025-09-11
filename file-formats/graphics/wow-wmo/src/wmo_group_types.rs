use crate::types::{BoundingBox, Color, Vec3};
use bitflags::bitflags;

/// Represents a WMO group file
#[derive(Debug)]
pub struct WmoGroup {
    /// Group header
    pub header: WmoGroupHeader,

    /// Materials used in this group
    pub materials: Vec<u16>,

    /// Vertices that make up the geometry
    pub vertices: Vec<Vec3>,

    /// Normal vectors for vertices
    pub normals: Vec<Vec3>,

    /// Texture coordinates
    pub tex_coords: Vec<TexCoord>,

    /// Batch information for rendering
    pub batches: Vec<WmoBatch>,

    /// Triangle indices
    pub indices: Vec<u16>,

    /// Vertices colors (if present)
    pub vertex_colors: Option<Vec<Color>>,

    /// BSP tree nodes (if present)
    pub bsp_nodes: Option<Vec<WmoBspNode>>,

    /// Liquid data (if present)
    pub liquid: Option<WmoLiquid>,

    /// Doodad references (if present)
    pub doodad_refs: Option<Vec<u16>>,
}

/// Header for a WMO group
#[derive(Debug, Clone)]
pub struct WmoGroupHeader {
    /// Group flags
    pub flags: WmoGroupFlags,

    /// Bounding box
    pub bounding_box: BoundingBox,

    /// Name offset in MOGN chunk of root file
    pub name_offset: u32,

    /// Index of this group
    pub group_index: u32,
}

bitflags! {
    /// WMO group flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct WmoGroupFlags: u32 {
        /// Has base vertices
        const HAS_BASE_VERTICES = 0x01;
        /// Has vertex lighting information
        const HAS_VERTEX_LIGHTING = 0x02;
        /// Has normals
        const HAS_NORMALS = 0x04;
        /// Has some light
        const HAS_LIGHT = 0x08;
        /// Has doodads
        const HAS_DOODADS = 0x10;
        /// Has liquids
        const HAS_WATER = 0x20;
        /// Indoor
        const INDOOR = 0x40;
        /// Show skybox
        const SHOW_SKYBOX = 0x80;
        /// Has exterior lights
        const EXTERIOR_LIGHTS = 0x100;
        /// Unused 1
        const UNUSED1 = 0x200;
        /// Unreachable
        const UNREACHABLE = 0x400;
        /// For custom use
        const CUSTOM = 0x800;
        /// Use ambient lighting
        const USE_AMBIENT = 0x1000;
        /// Has vertex colors
        const HAS_VERTEX_COLORS = 0x2000;
        /// Uses scene graph for rendering
        const USE_SCENE_GRAPH = 0x4000;
        /// Has more motion types
        const HAS_MORE_MOTION_TYPES = 0x8000;
        /// Mount allowed
        const MOUNT_ALLOWED = 0x10000;
        /// Has exterior BSP
        const EXTERIOR_BSP = 0x20000;
    }
}

/// Texture coordinates
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TexCoord {
    pub u: f32,
    pub v: f32,
}

/// Represents a rendering batch in a WMO group
#[derive(Debug, Clone)]
pub struct WmoBatch {
    /// Flags for the batch
    pub flags: [u8; 10],

    /// Material ID
    pub material_id: u16,

    /// Start index in the indices array
    pub start_index: u32,

    /// Number of indices
    pub count: u16,

    /// Start vertex
    pub start_vertex: u16,

    /// End vertex
    pub end_vertex: u16,

    /// Whether a large material ID is used
    pub use_large_material_id: bool,
}

/// BSP tree node for collision and visibility
#[derive(Debug, Clone)]
pub struct WmoBspNode {
    /// Plane split information
    pub plane: WmoPlane,

    /// Child nodes or triangles
    pub children: [i16; 2],

    /// First triangle index
    pub first_face: u16,

    /// Number of triangles
    pub num_faces: u16,
}

/// Plane used in BSP node calculations
#[derive(Debug, Clone, Copy)]
pub struct WmoPlane {
    /// Normal vector
    pub normal: Vec3,

    /// Distance from origin
    pub distance: f32,
}

/// Liquid data in a WMO group
#[derive(Debug, Clone)]
pub struct WmoLiquid {
    /// Liquid type
    pub liquid_type: u32,

    /// Liquid flags
    pub flags: u32,

    /// Grid dimensions
    pub width: u32,
    pub height: u32,

    /// Vertices for liquid surface
    pub vertices: Vec<WmoLiquidVertex>,

    /// Flags for each liquid cell
    pub tile_flags: Option<Vec<u8>>,
}

/// Vertex in a liquid surface
#[derive(Debug, Clone, Copy)]
pub struct WmoLiquidVertex {
    /// Position
    pub position: Vec3,

    /// Height of the liquid
    pub height: f32,
}

/// Material information for a group
#[derive(Debug, Clone)]
pub struct WmoMaterialInfo {
    /// Material ID in the root file
    pub material_id: u16,

    /// Flags specific to this usage
    pub flags: u8,
}
