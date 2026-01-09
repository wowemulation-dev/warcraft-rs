use crate::chunk_discovery::ChunkDiscovery;
use crate::chunks::{
    GfidEntry, McvpEntry, MfogEntry, ModdEntry, ModiEntry, Modn, ModsEntry, MogiEntry, Mogn,
    MoltEntry, MolvEntry, Mom3Entry, MomtEntry, MopeEntry, MoprEntry, MoptEntry, MopvEntry, Mosb,
    Motx, MouvEntry, MovbEntry, MovvEntry,
};
use binrw::{BinRead, BinReaderExt};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};

/// WMO Root file structure with extended chunk support
#[derive(Debug, Clone)]
pub struct WmoRoot {
    /// Version (always 17 for supported versions)
    pub version: u32,
    /// Number of materials (from MOHD)
    pub n_materials: u32,
    /// Number of groups (from MOHD)
    pub n_groups: u32,
    /// Number of portals (from MOHD)
    pub n_portals: u32,
    /// Number of lights (from MOHD)
    pub n_lights: u32,
    /// Number of doodad names (from MOHD)
    pub n_doodad_names: u32,
    /// Number of doodad definitions (from MOHD)
    pub n_doodad_defs: u32,
    /// Number of doodad sets (from MOHD)
    pub n_doodad_sets: u32,
    /// Ambient color as BGRA (from MOHD) - base/ambient lighting color
    pub ambient_color: [u8; 4],
    /// WMO ID (foreign key to WMOAreaTable.dbc)
    pub wmo_id: u32,
    /// Bounding box minimum corner
    pub bounding_box_min: [f32; 3],
    /// Bounding box maximum corner
    pub bounding_box_max: [f32; 3],
    /// WMO flags
    pub flags: u16,
    /// Number of LOD levels
    pub num_lod: u16,

    // Extended chunk data
    /// Texture filenames (MOTX)
    pub textures: Vec<String>,
    pub texture_offset_index_map: HashMap<u32, u32>,
    /// Materials (MOMT)
    pub materials: Vec<MomtEntry>,
    /// Group names (MOGN)
    pub group_names: Vec<String>,
    /// Group information (MOGI)
    pub group_info: Vec<MogiEntry>,
    /// Skybox name (MOSB)
    pub skybox: Option<String>,
    /// Portal vertices (MOPV)
    pub portal_vertices: Vec<MopvEntry>,
    /// Portal information (MOPT)
    pub portals: Vec<MoptEntry>,
    /// Portal references (MOPR)
    pub portal_refs: Vec<MoprEntry>,
    /// Visible block vertices (MOVV)
    pub visible_vertices: Vec<MovvEntry>,
    /// Visible block list (MOVB)
    pub visible_blocks: Vec<MovbEntry>,
    /// Lights (MOLT)
    pub lights: Vec<MoltEntry>,
    /// Doodad sets (MODS)
    pub doodad_sets: Vec<ModsEntry>,
    /// Doodad names (MODN)
    pub doodad_names: Vec<String>,
    /// Doodad definitions (MODD)
    pub doodad_defs: Vec<ModdEntry>,
    /// Fog definitions (MFOG)
    pub fogs: Vec<MfogEntry>,
    /// Convex volume planes (MCVP - Cataclysm+)
    pub convex_volume_planes: Vec<McvpEntry>,
    /// UV transformations (MOUV - Legion+)
    pub uv_transforms: Vec<MouvEntry>,
    /// Portal extra information (MOPE - WarWithin+)
    pub portal_extras: Vec<MopeEntry>,
    /// Light extensions (MOLV - Shadowlands+)
    pub light_extensions: Vec<MolvEntry>,
    /// Doodad file IDs (MODI - Battle for Azeroth+)
    pub doodad_ids: Vec<ModiEntry>,
    /// New materials (MOM3 - WarWithin+)
    pub new_materials: Vec<Mom3Entry>,
    /// Group file IDs (GFID - modern WoW versions)
    pub group_file_ids: Vec<GfidEntry>,
}

/// MOHD chunk structure (WMO Header)
/// Reference: https://wowdev.wiki/WMO#MOHD_chunk
#[derive(Debug, Clone, BinRead)]
#[br(little)]
struct Mohd {
    /// Number of textures (0x00)
    n_materials: u32,
    /// Number of groups (0x04)
    n_groups: u32,
    /// Number of portals (0x08)
    n_portals: u32,
    /// Number of lights (0x0C)
    n_lights: u32,
    /// Number of doodad names (0x10)
    n_doodad_names: u32,
    /// Number of doodad definitions (0x14)
    n_doodad_defs: u32,
    /// Number of doodad sets (0x18)
    n_doodad_sets: u32,
    /// Ambient color as BGRA (0x1C) - base/ambient lighting color for the WMO
    ambient_color: [u8; 4],
    /// WMO ID (foreign key to WMOAreaTable.dbc) (0x20)
    wmo_id: u32,
    /// Bounding box min (0x24)
    bounding_box_min: [f32; 3],
    /// Bounding box max (0x30)
    bounding_box_max: [f32; 3],
    /// Flags (0x3C)
    flags: u16,
    /// Number of LOD levels (0x3E)
    num_lod: u16,
}

/// Parse a WMO root file using discovered chunks
pub fn parse_root_file<R: Read + Seek>(
    reader: &mut R,
    discovery: ChunkDiscovery,
) -> Result<WmoRoot, Box<dyn std::error::Error>> {
    let mut root = WmoRoot {
        version: 0,
        n_materials: 0,
        n_groups: 0,
        n_portals: 0,
        n_lights: 0,
        n_doodad_names: 0,
        n_doodad_defs: 0,
        n_doodad_sets: 0,
        ambient_color: [128, 128, 128, 255], // Default to mid-gray
        wmo_id: 0,
        bounding_box_min: [0.0; 3],
        bounding_box_max: [0.0; 3],
        flags: 0,
        num_lod: 0,
        textures: Vec::new(),
        texture_offset_index_map: HashMap::new(),
        materials: Vec::new(),
        group_names: Vec::new(),
        group_info: Vec::new(),
        skybox: None,
        portal_vertices: Vec::new(),
        portals: Vec::new(),
        portal_refs: Vec::new(),
        visible_vertices: Vec::new(),
        visible_blocks: Vec::new(),
        lights: Vec::new(),
        doodad_sets: Vec::new(),
        doodad_names: Vec::new(),
        doodad_defs: Vec::new(),
        fogs: Vec::new(),
        convex_volume_planes: Vec::new(),
        uv_transforms: Vec::new(),
        portal_extras: Vec::new(),
        light_extensions: Vec::new(),
        doodad_ids: Vec::new(),
        new_materials: Vec::new(),
        group_file_ids: Vec::new(),
    };

    // Process chunks in order
    for chunk_info in &discovery.chunks {
        // Seek to chunk data (skip header)
        reader.seek(SeekFrom::Start(chunk_info.offset + 8))?;

        match chunk_info.id.as_str() {
            "MVER" => {
                // Read version
                root.version = reader.read_le()?;
            }
            "MOHD" => {
                // Read header and populate all fields
                let mohd = Mohd::read(reader)?;
                root.n_materials = mohd.n_materials;
                root.n_groups = mohd.n_groups;
                root.n_portals = mohd.n_portals;
                root.n_lights = mohd.n_lights;
                root.n_doodad_names = mohd.n_doodad_names;
                root.n_doodad_defs = mohd.n_doodad_defs;
                root.n_doodad_sets = mohd.n_doodad_sets;
                root.ambient_color = mohd.ambient_color;
                root.wmo_id = mohd.wmo_id;
                root.bounding_box_min = mohd.bounding_box_min;
                root.bounding_box_max = mohd.bounding_box_max;
                root.flags = mohd.flags;
                root.num_lod = mohd.num_lod;
            }
            "MOTX" => {
                // Read texture filenames
                let mut data = vec![0u8; chunk_info.size as usize];
                reader.read_exact(&mut data)?;
                let motx = Motx::parse(&data)?;
                root.textures = motx.textures;
                root.texture_offset_index_map = motx.texture_offset_index_map;
            }
            "MOMT" => {
                // Read materials
                let count = chunk_info.size / 64; // Each material is 64 bytes
                for _ in 0..count {
                    root.materials.push(MomtEntry::read(reader)?);
                }
            }
            "MOGN" => {
                // Read group names
                let mut data = vec![0u8; chunk_info.size as usize];
                reader.read_exact(&mut data)?;
                let mogn = Mogn::parse(&data)?;
                root.group_names = mogn.names;
            }
            "MOGI" => {
                // Read group information
                let count = chunk_info.size / 32; // Each entry is 32 bytes
                for _ in 0..count {
                    root.group_info.push(MogiEntry::read(reader)?);
                }
            }
            "MOSB" => {
                // Read skybox name
                let mut data = vec![0u8; chunk_info.size as usize];
                reader.read_exact(&mut data)?;
                let mosb = Mosb::parse(&data)?;
                root.skybox = mosb.skybox;
            }
            "MOPV" => {
                // Read portal vertices
                let count = chunk_info.size / 12; // Each vertex is 3 floats (12 bytes)
                for _ in 0..count {
                    root.portal_vertices.push(MopvEntry::read(reader)?);
                }
            }
            "MOPT" => {
                // Read portal information
                let count = chunk_info.size / 20; // Each portal is 20 bytes
                for _ in 0..count {
                    root.portals.push(MoptEntry::read(reader)?);
                }
            }
            "MOPR" => {
                // Read portal references
                let count = chunk_info.size / 8; // Each reference is 8 bytes
                for _ in 0..count {
                    root.portal_refs.push(MoprEntry::read(reader)?);
                }
            }
            "MOVV" => {
                // Read visible block vertices
                let count = chunk_info.size / 12; // Each vertex is 3 floats (12 bytes)
                for _ in 0..count {
                    root.visible_vertices.push(MovvEntry::read(reader)?);
                }
            }
            "MOVB" => {
                // Read visible block list
                let count = chunk_info.size / 4; // Each entry is 4 bytes
                for _ in 0..count {
                    root.visible_blocks.push(MovbEntry::read(reader)?);
                }
            }
            "MOLT" => {
                // Read lights
                let count = chunk_info.size / 48; // Each light is 48 bytes
                for _ in 0..count {
                    root.lights.push(MoltEntry::read(reader)?);
                }
            }
            "MODS" => {
                // Read doodad sets
                let count = chunk_info.size / 32; // Each set is 32 bytes
                for _ in 0..count {
                    root.doodad_sets.push(ModsEntry::read(reader)?);
                }
            }
            "MODN" => {
                // Read doodad names
                let mut data = vec![0u8; chunk_info.size as usize];
                reader.read_exact(&mut data)?;
                let modn = Modn::parse(&data)?;
                root.doodad_names = modn.names;
            }
            "MODD" => {
                // Read doodad definitions
                let count = chunk_info.size / 40; // Each def is 40 bytes
                for _ in 0..count {
                    root.doodad_defs.push(ModdEntry::read(reader)?);
                }
            }
            "MFOG" => {
                // Read fog definitions
                let count = chunk_info.size / 48; // Each fog is 48 bytes
                for _ in 0..count {
                    root.fogs.push(MfogEntry::read(reader)?);
                }
            }
            "MCVP" => {
                // Read convex volume planes (Cataclysm+)
                let count = chunk_info.size / 16; // Each plane is 16 bytes (4 floats)
                for _ in 0..count {
                    root.convex_volume_planes.push(McvpEntry::read(reader)?);
                }
            }
            "MOUV" => {
                // Read UV transformations (Legion+)
                let count = chunk_info.size / 16; // Each UV transform is 16 bytes (4 floats)
                for _ in 0..count {
                    root.uv_transforms.push(MouvEntry::read(reader)?);
                }
            }
            "MOPE" => {
                // Read portal extra information (WarWithin+)
                let count = chunk_info.size / 16; // Each entry is 16 bytes (4 u32s)
                for _ in 0..count {
                    root.portal_extras.push(MopeEntry::read(reader)?);
                }
            }
            "MOLV" => {
                // Read light extensions (Shadowlands+)
                let count = chunk_info.size / 100; // Each entry is 100 bytes
                for _ in 0..count {
                    root.light_extensions.push(MolvEntry::read(reader)?);
                }
            }
            "MODI" => {
                // Read doodad file IDs (Battle for Azeroth+)
                let count = chunk_info.size / 4; // Each ID is 4 bytes (u32)
                for _ in 0..count {
                    root.doodad_ids.push(reader.read_le()?);
                }
            }
            "MOM3" => {
                // Read new materials (WarWithin+)
                // Structure is variable, read as opaque data for now
                let mut data = vec![0u8; chunk_info.size as usize];
                reader.read_exact(&mut data)?;
                root.new_materials.push(Mom3Entry { data });
            }
            "MOMO" => {
                // Alpha version container chunk (version 14 only)
                // Skip - this is a container chunk with no data
            }
            "GFID" => {
                // Read group file IDs (modern WoW versions)
                let count = chunk_info.size / 4; // Each ID is 4 bytes (u32)
                for _ in 0..count {
                    root.group_file_ids.push(reader.read_le()?);
                }
            }
            _ => {
                // Skip unknown/unimplemented chunks
            }
        }
    }

    Ok(root)
}
