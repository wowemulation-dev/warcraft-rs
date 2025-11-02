use crate::chunk_discovery::ChunkDiscovery;
use crate::chunk_header::ChunkHeader;
use crate::chunks::{
    MliqHeader, MobaEntry, MobnEntry, MobsEntry, MocvEntry, MonrEntry, MopyEntry, MorbEntry,
    MotaEntry, MotvEntry, MovtEntry, Mpy2Entry,
};
use crate::error::{Result, WmoError};
use crate::wmo_group_types::WmoGroup as LegacyWmoGroup;
use binrw::{BinRead, BinReaderExt};
use std::io::{Read, Seek, SeekFrom};

/// WMO Group file structure with extended chunk support
#[derive(Debug, Clone)]
pub struct WmoGroup {
    /// Version (always 17 for supported versions)
    pub version: u32,
    /// Group index (calculated from MOGN)
    pub group_index: u32,
    /// Group name index (offset into MOGN)
    pub group_name_index: u32,
    /// Descriptive group name index (offset into MOGN)
    pub descriptive_name_index: u32,
    /// Group flags
    pub flags: u32,
    /// Bounding box [min_x, min_y, min_z, max_x, max_y, max_z]
    pub bounding_box: Vec<f32>,
    /// Portal information
    pub portal_start: u16,
    pub portal_count: u16,
    /// Batch counts (trans, int, ext)
    pub trans_batch_count: u16,
    pub int_batch_count: u16,
    pub ext_batch_count: u16,
    /// Unknown batch type (padding or batch_type_d)
    pub batch_type_d: u16,
    /// Fog indices (up to 4)
    pub fog_ids: Vec<u8>,
    /// Liquid group ID
    pub group_liquid: u32,
    /// WMOAreaTable ID
    pub area_table_id: u32,
    /// Additional flags (Cataclysm+)
    pub flags2: u32,
    /// Parent or first child split group index
    pub parent_split_group: i16,
    /// Next split child group index
    pub next_split_child: i16,
    /// Number of triangles in this group (calculated)
    pub n_triangles: u32,
    /// Number of vertices in this group (calculated)
    pub n_vertices: u32,

    // Extended chunk data
    /// Material info per triangle (MOPY)
    pub material_info: Vec<MopyEntry>,
    /// Vertex indices (MOVI)
    pub vertex_indices: Vec<u16>,
    /// Vertex positions (MOVT)
    pub vertex_positions: Vec<MovtEntry>,
    /// Vertex normals (MONR)
    pub vertex_normals: Vec<MonrEntry>,
    /// Texture coordinates (MOTV)
    pub texture_coords: Vec<MotvEntry>,
    /// Render batches (MOBA)
    pub render_batches: Vec<MobaEntry>,
    /// Vertex colors (MOCV)
    pub vertex_colors: Vec<MocvEntry>,
    /// Light references (MOLR)
    pub light_refs: Vec<u16>,
    /// Doodad references (MODR)
    pub doodad_refs: Vec<u16>,
    /// BSP tree nodes (MOBN)
    pub bsp_nodes: Vec<MobnEntry>,
    /// BSP face indices (MOBR)
    pub bsp_face_indices: Vec<u16>,
    /// Liquid data (MLIQ)
    pub liquid_header: Option<MliqHeader>,
    /// Query face start (MOGX - Dragonflight+)
    pub query_face_start: Option<u32>,
    /// Extended material info (MPY2 - Dragonflight+)
    pub extended_materials: Vec<Mpy2Entry>,
    /// Extended vertex indices (MOVX - Shadowlands+)
    pub extended_vertex_indices: Vec<u32>,
    /// Query faces (MOQG - Dragonflight+)
    pub query_faces: Vec<u32>,
    /// Triangle strip indices (MORI)
    pub triangle_strip_indices: Vec<u16>,
    /// Additional render batches (MORB)
    pub additional_render_batches: Vec<MorbEntry>,
    /// Tangent arrays (MOTA)
    pub tangent_arrays: Vec<MotaEntry>,
    /// Shadow batches (MOBS)
    pub shadow_batches: Vec<MobsEntry>,
}

/// MOGP chunk header structure (corrected based on wowdev.wiki)
#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MogpHeader {
    group_name: u32,             // offset into MOGN
    descriptive_group_name: u32, // offset into MOGN
    flags: u32,                  // group flags
    #[br(count = 6)]
    bounding_box: Vec<f32>, // min xyz, max xyz
    portal_start: u16,           // index into portal references
    portal_count: u16,           // number of portal items
    trans_batch_count: u16,
    int_batch_count: u16,
    ext_batch_count: u16,
    padding_or_batch_type_d: u16,
    #[br(count = 4)]
    fog_ids: Vec<u8>, // fog IDs from MFOG
    group_liquid: u32, // liquid-related
    unique_id: u32,    // WMOAreaTable reference
    flags2: u32,
    parent_or_first_child_split_group_index: i16,
    next_split_child_group_index: i16,
}

/// Parse a WMO group file using discovered chunks
pub fn parse_group_file<R: Read + Seek>(
    reader: &mut R,
    discovery: ChunkDiscovery,
) -> std::result::Result<WmoGroup, Box<dyn std::error::Error>> {
    let mut group = WmoGroup {
        version: 0,
        group_index: 0,
        group_name_index: 0,
        descriptive_name_index: 0,
        flags: 0,
        bounding_box: Vec::new(),
        portal_start: 0,
        portal_count: 0,
        trans_batch_count: 0,
        int_batch_count: 0,
        ext_batch_count: 0,
        batch_type_d: 0,
        fog_ids: Vec::new(),
        group_liquid: 0,
        area_table_id: 0,
        flags2: 0,
        parent_split_group: 0,
        next_split_child: 0,
        n_triangles: 0,
        n_vertices: 0,
        material_info: Vec::new(),
        vertex_indices: Vec::new(),
        vertex_positions: Vec::new(),
        vertex_normals: Vec::new(),
        texture_coords: Vec::new(),
        render_batches: Vec::new(),
        vertex_colors: Vec::new(),
        light_refs: Vec::new(),
        doodad_refs: Vec::new(),
        bsp_nodes: Vec::new(),
        bsp_face_indices: Vec::new(),
        liquid_header: None,
        query_face_start: None,
        extended_materials: Vec::new(),
        extended_vertex_indices: Vec::new(),
        query_faces: Vec::new(),
        triangle_strip_indices: Vec::new(),
        additional_render_batches: Vec::new(),
        tangent_arrays: Vec::new(),
        shadow_batches: Vec::new(),
    };

    // Process chunks in order
    for chunk_info in &discovery.chunks {
        // Seek to chunk data (skip header)
        reader.seek(SeekFrom::Start(chunk_info.offset + 8))?;

        match chunk_info.id.as_str() {
            "MVER" => {
                // Read version
                group.version = reader.read_le()?;
            }
            "MOGP" => {
                // Read group header
                let header = MogpHeader::read(reader)?;

                // Populate all header fields into the group structure
                group.group_index = 0; // Will be set from filename or external context
                group.group_name_index = header.group_name;
                group.descriptive_name_index = header.descriptive_group_name;
                group.flags = header.flags;
                group.bounding_box = header.bounding_box;
                group.portal_start = header.portal_start;
                group.portal_count = header.portal_count;
                group.trans_batch_count = header.trans_batch_count;
                group.int_batch_count = header.int_batch_count;
                group.ext_batch_count = header.ext_batch_count;
                group.batch_type_d = header.padding_or_batch_type_d;
                group.fog_ids = header.fog_ids;
                group.group_liquid = header.group_liquid;
                group.area_table_id = header.unique_id;
                group.flags2 = header.flags2;
                group.parent_split_group = header.parent_or_first_child_split_group_index;
                group.next_split_child = header.next_split_child_group_index;

                // MOGP contains sub-chunks - we need to parse them
                // The remaining chunk data contains nested chunks
                // MogpHeader is 68 bytes when serialized (not std::mem::size_of due to Vec fields)
                let data_size = chunk_info.size - 68;
                let mut data_reader = std::io::Cursor::new(read_chunk_data(reader, data_size)?);

                // Parse nested chunks within MOGP
                parse_nested_chunks(&mut data_reader, &mut group)?;
            }
            "MOPY" => {
                // Read material info
                let count = chunk_info.size / 2; // Each entry is 2 bytes
                for _ in 0..count {
                    group.material_info.push(MopyEntry::read(reader)?);
                }
            }
            "MOVI" => {
                // Read vertex indices
                let count = chunk_info.size / 2; // Each index is 2 bytes
                for _ in 0..count {
                    group.vertex_indices.push(reader.read_le()?);
                }
            }
            "MOVT" => {
                // Read vertex positions
                let count = chunk_info.size / 12; // Each position is 3 floats (12 bytes)
                for _ in 0..count {
                    group.vertex_positions.push(MovtEntry::read(reader)?);
                }
            }
            "MONR" => {
                // Read vertex normals
                let count = chunk_info.size / 12; // Each normal is 3 floats (12 bytes)
                for _ in 0..count {
                    group.vertex_normals.push(MonrEntry::read(reader)?);
                }
            }
            "MOTV" => {
                // Read texture coordinates
                let count = chunk_info.size / 8; // Each coord is 2 floats (8 bytes)
                for _ in 0..count {
                    group.texture_coords.push(MotvEntry::read(reader)?);
                }
            }
            "MOBA" => {
                // Read render batches
                let count = chunk_info.size / 16; // Each batch is 16 bytes
                for _ in 0..count {
                    group.render_batches.push(MobaEntry::read(reader)?);
                }
            }
            "MOCV" => {
                // Read vertex colors
                let count = chunk_info.size / 4; // Each color is 4 bytes
                for _ in 0..count {
                    group.vertex_colors.push(MocvEntry::read(reader)?);
                }
            }
            "MOLR" => {
                // Read light references
                let count = chunk_info.size / 2; // Each ref is 2 bytes
                for _ in 0..count {
                    group.light_refs.push(reader.read_le()?);
                }
            }
            "MODR" => {
                // Read doodad references
                let count = chunk_info.size / 2; // Each ref is 2 bytes
                for _ in 0..count {
                    group.doodad_refs.push(reader.read_le()?);
                }
            }
            "MOBN" => {
                // Read BSP tree nodes
                let count = chunk_info.size / 16; // Each node is 16 bytes
                for _ in 0..count {
                    group.bsp_nodes.push(MobnEntry::read(reader)?);
                }
            }
            "MOBR" => {
                // Read BSP face indices
                let count = chunk_info.size / 2; // Each index is 2 bytes
                for _ in 0..count {
                    group.bsp_face_indices.push(reader.read_le()?);
                }
            }
            "MLIQ" => {
                // Read liquid header (just the header for now)
                if chunk_info.size >= 32 {
                    group.liquid_header = Some(MliqHeader::read(reader)?);
                }
            }
            "MOGX" => {
                // Read query face start (Dragonflight+)
                if chunk_info.size >= 4 {
                    group.query_face_start = Some(reader.read_le()?);
                }
            }
            "MPY2" => {
                // Read extended material info (Dragonflight+)
                let count = chunk_info.size / 4; // Each entry is 4 bytes (2 u16s)
                for _ in 0..count {
                    group.extended_materials.push(Mpy2Entry::read(reader)?);
                }
            }
            "MOVX" => {
                // Read extended vertex indices (Shadowlands+)
                let count = chunk_info.size / 4; // Each index is 4 bytes (u32)
                for _ in 0..count {
                    group.extended_vertex_indices.push(reader.read_le()?);
                }
            }
            "MOQG" => {
                // Read query faces (Dragonflight+)
                let count = chunk_info.size / 4; // Each face is 4 bytes (u32)
                for _ in 0..count {
                    group.query_faces.push(reader.read_le()?);
                }
            }
            "MORI" => {
                // Read triangle strip indices
                let count = chunk_info.size / 2; // Each index is 2 bytes (u16)
                for _ in 0..count {
                    group.triangle_strip_indices.push(reader.read_le()?);
                }
            }
            "MORB" => {
                // Read additional render batches
                let count = chunk_info.size / 10; // Each batch is 10 bytes
                for _ in 0..count {
                    group
                        .additional_render_batches
                        .push(MorbEntry::read(reader)?);
                }
            }
            "MOTA" => {
                // Read tangent arrays
                let count = chunk_info.size / 8; // Each tangent is 8 bytes (4 i16)
                for _ in 0..count {
                    group.tangent_arrays.push(MotaEntry::read(reader)?);
                }
            }
            "MOBS" => {
                // Read shadow batches
                let count = chunk_info.size / 10; // Each batch is 10 bytes (same as MORB)
                for _ in 0..count {
                    group.shadow_batches.push(MobsEntry::read(reader)?);
                }
            }
            _ => {
                // Skip unknown/unimplemented chunks
            }
        }
    }

    // Calculate triangle and vertex counts from actual data
    group.n_triangles = (group.vertex_indices.len() / 3) as u32;
    group.n_vertices = group.vertex_positions.len() as u32;

    Ok(group)
}

/// Legacy parser for compatibility
pub struct WmoGroupParser;

impl Default for WmoGroupParser {
    fn default() -> Self {
        Self
    }
}

impl WmoGroupParser {
    /// Create a new WMO group parser
    pub fn new() -> Self {
        Self
    }

    /// Parse a WMO group file (legacy interface)
    pub fn parse_group<R: Read + Seek>(
        &self,
        _reader: &mut R,
        _group_index: u32,
    ) -> Result<LegacyWmoGroup> {
        // This is a stub for now - the real implementation will use binrw
        Err(WmoError::InvalidFormat(
            "Legacy parser not yet migrated".into(),
        ))
    }
}

/// Read chunk data as bytes
fn read_chunk_data<R: Read>(
    reader: &mut R,
    size: u32,
) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut data = vec![0u8; size as usize];
    reader.read_exact(&mut data)?;
    Ok(data)
}

/// Parse nested chunks within MOGP data
fn parse_nested_chunks<R: Read + Seek>(
    reader: &mut R,
    group: &mut WmoGroup,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let end_pos = reader.seek(SeekFrom::End(0))?;
    reader.seek(SeekFrom::Start(0))?;

    // Parse chunks within the MOGP data
    while reader.stream_position()? < end_pos {
        let chunk_start = reader.stream_position()?;
        let remaining = end_pos - chunk_start;

        if remaining < 8 {
            break; // Not enough bytes for chunk header
        }

        let header = match ChunkHeader::read(reader) {
            Ok(h) => h,
            Err(_) => break, // Malformed chunk
        };

        let chunk_id = header.id.as_str();
        let chunk_size = header.size;

        // Check if we have enough remaining data for this chunk
        let remaining_data = end_pos - reader.stream_position()?;
        if chunk_size as u64 > remaining_data {
            // Chunk claims more data than available - truncated file or corrupted chunk
            // Skip parsing this chunk and stop
            break;
        }
        match chunk_id {
            "MOPY" => {
                // Read material info
                let count = chunk_size / 2; // Each entry is 2 bytes
                for _ in 0..count {
                    group.material_info.push(MopyEntry::read(reader)?);
                }
            }
            "MOVI" => {
                // Read vertex indices
                let count = chunk_size / 2; // Each index is 2 bytes
                for _ in 0..count {
                    group.vertex_indices.push(reader.read_le()?);
                }
            }
            "MOVT" => {
                // Read vertex positions
                let count = chunk_size / 12; // Each position is 3 floats (12 bytes)
                for _ in 0..count {
                    group.vertex_positions.push(MovtEntry::read(reader)?);
                }
            }
            "MONR" => {
                // Read vertex normals
                let count = chunk_size / 12; // Each normal is 3 floats (12 bytes)
                for _ in 0..count {
                    group.vertex_normals.push(MonrEntry::read(reader)?);
                }
            }
            "MOTV" => {
                // Read texture coordinates
                let count = chunk_size / 8; // Each coord is 2 floats (8 bytes)
                for _ in 0..count {
                    group.texture_coords.push(MotvEntry::read(reader)?);
                }
            }
            "MOBA" => {
                // Read render batches
                let count = chunk_size / 24; // Each batch is 24 bytes
                for _ in 0..count {
                    group.render_batches.push(MobaEntry::read(reader)?);
                }
            }
            "MOCV" => {
                // Read vertex colors
                let count = chunk_size / 4; // Each color is 4 bytes (BGRA)
                for _ in 0..count {
                    group.vertex_colors.push(MocvEntry::read(reader)?);
                }
            }
            "MOBN" => {
                // Read BSP tree nodes
                let count = chunk_size / 16; // Each node is 16 bytes
                for _ in 0..count {
                    group.bsp_nodes.push(MobnEntry::read(reader)?);
                }
            }
            "MOBR" => {
                // Read BSP face indices
                let count = chunk_size / 2; // Each index is 2 bytes
                for _ in 0..count {
                    group.bsp_face_indices.push(reader.read_le()?);
                }
            }
            "MLIQ" => {
                // Read liquid header if available
                if chunk_size >= 32 {
                    group.liquid_header = Some(MliqHeader::read(reader)?);
                }
            }
            "MOGX" => {
                // Read query face start (Dragonflight+)
                if chunk_size >= 4 {
                    group.query_face_start = Some(reader.read_le()?);
                }
            }
            "MPY2" => {
                // Read extended material info (Dragonflight+)
                let count = chunk_size / 4; // Each entry is 4 bytes (2 u16s)
                for _ in 0..count {
                    group.extended_materials.push(Mpy2Entry::read(reader)?);
                }
            }
            "MOVX" => {
                // Read extended vertex indices (Shadowlands+)
                let count = chunk_size / 4; // Each index is 4 bytes (u32)
                for _ in 0..count {
                    group.extended_vertex_indices.push(reader.read_le()?);
                }
            }
            "MOQG" => {
                // Read query faces (Dragonflight+)
                let count = chunk_size / 4; // Each face is 4 bytes (u32)
                for _ in 0..count {
                    group.query_faces.push(reader.read_le()?);
                }
            }
            "MORI" => {
                // Read triangle strip indices
                let count = chunk_size / 2; // Each index is 2 bytes (u16)
                for _ in 0..count {
                    group.triangle_strip_indices.push(reader.read_le()?);
                }
            }
            "MORB" => {
                // Read additional render batches
                let count = chunk_size / 10; // Each batch is 10 bytes
                for _ in 0..count {
                    group
                        .additional_render_batches
                        .push(MorbEntry::read(reader)?);
                }
            }
            "MOTA" => {
                // Read tangent arrays
                let count = chunk_size / 8; // Each tangent is 8 bytes (4 i16)
                for _ in 0..count {
                    group.tangent_arrays.push(MotaEntry::read(reader)?);
                }
            }
            "MOBS" => {
                // Read shadow batches
                let count = chunk_size / 10; // Each batch is 10 bytes (same as MORB)
                for _ in 0..count {
                    group.shadow_batches.push(MobsEntry::read(reader)?);
                }
            }
            _ => {
                // Skip unknown chunk
                reader.seek(SeekFrom::Current(chunk_size as i64))?;
            }
        }
    }

    Ok(())
}
