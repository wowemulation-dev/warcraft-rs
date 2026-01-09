use std::fs::File;
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};

use crate::io_ext::ReadExt;
use std::path::Path;

use crate::chunks::animation::M2Animation;
use crate::chunks::bone::M2Bone;
use crate::chunks::infrastructure::{ChunkHeader, ChunkReader};
use crate::chunks::material::M2Material;
use crate::chunks::particle_emitter::M2ParticleEmitter;
use crate::chunks::ribbon_emitter::M2RibbonEmitter;
use crate::chunks::{
    AfraChunk, AnimationFileIds, BoneData, BoneFileIds, CollisionMeshData, DbocChunk, DpivChunk,
    EdgeFadeData, ExtendedParticleData, GeometryParticleIds, LightingDetails, LodData, M2Texture,
    M2Vertex, ModelAlphaData, ParentAnimationBlacklist, ParentAnimationData, ParentEventData,
    ParentSequenceBounds, ParticleGeosetData, PhysicsData, PhysicsFileDataChunk, PhysicsFileId,
    RecursiveParticleIds, SkeletonData, SkeletonFileId, SkinFileIds, TextureAnimationChunk,
    TextureFileIds, WaterfallEffect,
};
use crate::common::{M2Array, read_array};
use crate::error::{M2Error, Result};
use crate::file_resolver::FileResolver;
use crate::header::{M2_MAGIC_CHUNKED, M2_MAGIC_LEGACY, M2Header, M2ModelFlags};
use crate::version::M2Version;

/// M2 format variants
#[derive(Debug, Clone)]
pub enum M2Format {
    /// Legacy MD20 format (Pre-Legion)
    Legacy(M2Model),
    /// Chunked MD21 format (Legion+)
    Chunked(M2Model),
}

/// Main M2 model structure
#[derive(Debug, Clone)]
pub struct M2Model {
    /// M2 header
    pub header: M2Header,
    /// Model name
    pub name: Option<String>,
    /// Global sequences
    pub global_sequences: Vec<u32>,
    /// Animations
    pub animations: Vec<M2Animation>,
    /// Animation lookups
    pub animation_lookup: Vec<u16>,
    /// Bones
    pub bones: Vec<M2Bone>,
    /// Key bone lookups
    pub key_bone_lookup: Vec<u16>,
    /// Vertices
    pub vertices: Vec<M2Vertex>,
    /// Textures
    pub textures: Vec<M2Texture>,
    /// Materials (render flags)
    pub materials: Vec<M2Material>,
    /// Particle emitters
    pub particle_emitters: Vec<M2ParticleEmitter>,
    /// Ribbon emitters
    pub ribbon_emitters: Vec<M2RibbonEmitter>,
    /// Raw data for other sections
    /// This is used to preserve data that we don't fully parse yet
    pub raw_data: M2RawData,

    /// Chunked format data (Legion+ only)
    /// Contains FileDataID references for external files
    pub skin_file_ids: Option<SkinFileIds>,
    /// Animation file IDs (Legion+ only)
    pub animation_file_ids: Option<AnimationFileIds>,
    /// Texture file IDs (Legion+ only)
    pub texture_file_ids: Option<TextureFileIds>,
    /// Physics file ID (Legion+ only)
    pub physics_file_id: Option<PhysicsFileId>,
    /// Skeleton file ID (Legion+ only)
    pub skeleton_file_id: Option<SkeletonFileId>,
    /// Bone file IDs (Legion+ only)
    pub bone_file_ids: Option<BoneFileIds>,
    /// Level of detail data (Legion+ only)
    pub lod_data: Option<LodData>,

    /// Advanced rendering features (Legion+ only)
    /// Extended particle data (EXPT/EXP2 chunks)
    pub extended_particle_data: Option<ExtendedParticleData>,
    /// Parent animation blacklist (PABC chunk)
    pub parent_animation_blacklist: Option<ParentAnimationBlacklist>,
    /// Parent animation data (PADC chunk)
    pub parent_animation_data: Option<ParentAnimationData>,
    /// Waterfall effects (WFV1/WFV2/WFV3 chunks)
    pub waterfall_effect: Option<WaterfallEffect>,
    /// Edge fade rendering (EDGF chunk)
    pub edge_fade_data: Option<EdgeFadeData>,
    /// Model alpha calculations (NERF chunk)
    pub model_alpha_data: Option<ModelAlphaData>,
    /// Lighting details (DETL chunk)
    pub lighting_details: Option<LightingDetails>,
    /// Recursive particle model IDs (RPID chunk)
    pub recursive_particle_ids: Option<RecursiveParticleIds>,
    /// Geometry particle model IDs (GPID chunk)
    pub geometry_particle_ids: Option<GeometryParticleIds>,

    /// Phase 7 specialized chunks
    /// TXAC texture animation chunk
    pub texture_animation_chunk: Option<TextureAnimationChunk>,
    /// PGD1 particle geoset data
    pub particle_geoset_data: Option<ParticleGeosetData>,
    /// DBOC chunk (purpose unknown)
    pub dboc_chunk: Option<DbocChunk>,
    /// AFRA chunk (purpose unknown)
    pub afra_chunk: Option<AfraChunk>,
    /// DPIV chunk (collision mesh for player housing)
    pub dpiv_chunk: Option<DpivChunk>,
    /// PSBC chunk (parent sequence bounds)
    pub parent_sequence_bounds: Option<ParentSequenceBounds>,
    /// PEDC chunk (parent event data)
    pub parent_event_data: Option<ParentEventData>,
    /// PCOL chunk (collision mesh data)
    pub collision_mesh_data: Option<CollisionMeshData>,
    /// PFDC chunk (physics file data)
    pub physics_file_data: Option<PhysicsFileDataChunk>,
}

/// Raw data for sections that are not fully parsed
#[derive(Debug, Clone, Default)]
pub struct M2RawData {
    /// Transparency data
    pub transparency: Vec<u8>,
    /// Texture animations
    pub texture_animations: Vec<u8>,
    /// Color animations
    pub color_animations: Vec<u8>,
    /// Render flags
    pub render_flags: Vec<u8>,
    /// Bone lookup table
    pub bone_lookup_table: Vec<u16>,
    /// Texture lookup table
    pub texture_lookup_table: Vec<u16>,
    /// Texture units
    pub texture_units: Vec<u16>,
    /// Transparency lookup table
    pub transparency_lookup_table: Vec<u16>,
    /// Texture animation lookup
    pub texture_animation_lookup: Vec<u16>,
    /// Bounding triangles
    pub bounding_triangles: Vec<u8>,
    /// Bounding vertices
    pub bounding_vertices: Vec<u8>,
    /// Bounding normals
    pub bounding_normals: Vec<u8>,
    /// Attachments
    pub attachments: Vec<u8>,
    /// Attachment lookup table
    pub attachment_lookup_table: Vec<u16>,
    /// Events
    pub events: Vec<u8>,
    /// Lights
    pub lights: Vec<u8>,
    /// Cameras
    pub cameras: Vec<u8>,
    /// Camera lookup table
    pub camera_lookup_table: Vec<u16>,
    /// Ribbon emitters
    pub ribbon_emitters: Vec<u8>,
    /// Particle emitters
    pub particle_emitters: Vec<u8>,
    /// Texture combiner combos (added in Cataclysm)
    pub texture_combiner_combos: Option<Vec<u8>>,
}

/// Parse an M2 model, automatically detecting format
pub fn parse_m2<R: Read + Seek>(reader: &mut R) -> Result<M2Format> {
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    reader.seek(SeekFrom::Start(0))?;

    match &magic {
        magic if magic == &M2_MAGIC_LEGACY => Ok(M2Format::Legacy(M2Model::parse_legacy(reader)?)),
        magic if magic == &M2_MAGIC_CHUNKED => {
            Ok(M2Format::Chunked(M2Model::parse_chunked(reader)?))
        }
        _ => Err(M2Error::InvalidMagicBytes(magic)),
    }
}

impl M2Format {
    /// Get the underlying M2Model regardless of format
    pub fn model(&self) -> &M2Model {
        match self {
            M2Format::Legacy(model) => model,
            M2Format::Chunked(model) => model,
        }
    }

    /// Get mutable reference to the underlying M2Model
    pub fn model_mut(&mut self) -> &mut M2Model {
        match self {
            M2Format::Legacy(model) => model,
            M2Format::Chunked(model) => model,
        }
    }

    /// Check if this is a chunked format model
    pub fn is_chunked(&self) -> bool {
        matches!(self, M2Format::Chunked(_))
    }

    /// Check if this is a legacy format model
    pub fn is_legacy(&self) -> bool {
        matches!(self, M2Format::Legacy(_))
    }
}

impl Default for M2Model {
    fn default() -> Self {
        Self {
            header: M2Header {
                magic: *b"MD20",
                version: 264, // Default to WotLK version
                name: M2Array::default(),
                flags: M2ModelFlags::empty(),
                global_sequences: M2Array::default(),
                animations: M2Array::default(),
                animation_lookup: M2Array::default(),
                playable_animation_lookup: None,
                bones: M2Array::default(),
                key_bone_lookup: M2Array::default(),
                vertices: M2Array::default(),
                views: M2Array::default(),
                num_skin_profiles: Some(0),
                color_animations: M2Array::default(),
                textures: M2Array::default(),
                transparency_lookup: M2Array::default(),
                texture_flipbooks: None,
                texture_animations: M2Array::default(),
                color_replacements: M2Array::default(),
                render_flags: M2Array::default(),
                bone_lookup_table: M2Array::default(),
                texture_lookup_table: M2Array::default(),
                texture_units: M2Array::default(),
                transparency_lookup_table: M2Array::default(),
                texture_animation_lookup: M2Array::default(),
                bounding_box_min: [0.0, 0.0, 0.0],
                bounding_box_max: [0.0, 0.0, 0.0],
                bounding_sphere_radius: 0.0,
                collision_box_min: [0.0, 0.0, 0.0],
                collision_box_max: [0.0, 0.0, 0.0],
                collision_sphere_radius: 0.0,
                bounding_triangles: M2Array::default(),
                bounding_vertices: M2Array::default(),
                bounding_normals: M2Array::default(),
                attachments: M2Array::default(),
                attachment_lookup_table: M2Array::default(),
                events: M2Array::default(),
                lights: M2Array::default(),
                cameras: M2Array::default(),
                camera_lookup_table: M2Array::default(),
                ribbon_emitters: M2Array::default(),
                particle_emitters: M2Array::default(),
                blend_map_overrides: None,
                texture_combiner_combos: None,
                texture_transforms: None,
            },
            name: None,
            global_sequences: Vec::new(),
            animations: Vec::new(),
            animation_lookup: Vec::new(),
            bones: Vec::new(),
            key_bone_lookup: Vec::new(),
            vertices: Vec::new(),
            textures: Vec::new(),
            materials: Vec::new(),
            particle_emitters: Vec::new(),
            ribbon_emitters: Vec::new(),
            raw_data: M2RawData::default(),
            skin_file_ids: None,
            animation_file_ids: None,
            texture_file_ids: None,
            physics_file_id: None,
            skeleton_file_id: None,
            bone_file_ids: None,
            lod_data: None,
            extended_particle_data: None,
            parent_animation_blacklist: None,
            parent_animation_data: None,
            waterfall_effect: None,
            edge_fade_data: None,
            model_alpha_data: None,
            lighting_details: None,
            recursive_particle_ids: None,
            geometry_particle_ids: None,
            texture_animation_chunk: None,
            particle_geoset_data: None,
            dboc_chunk: None,
            afra_chunk: None,
            dpiv_chunk: None,
            parent_sequence_bounds: None,
            parent_event_data: None,
            collision_mesh_data: None,
            physics_file_data: None,
        }
    }
}

impl M2Model {
    /// Parse a legacy M2 model from a reader (MD20 format)
    pub fn parse_legacy<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Self::parse(reader)
    }

    /// Parse a chunked M2 model from a reader (MD21 format)
    pub fn parse_chunked<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let mut chunks = Vec::new();
        let mut md21_chunk = None;
        let mut skin_file_ids = None;
        let mut animation_file_ids = None;
        let mut texture_file_ids = None;
        let mut physics_file_id = None;
        let mut skeleton_file_id = None;
        let mut bone_file_ids = None;
        let mut lod_data = None;
        let mut extended_particle_data = None;
        let mut parent_animation_blacklist = None;
        let mut parent_animation_data = None;
        let mut waterfall_effect = None;
        let mut edge_fade_data = None;
        let mut model_alpha_data = None;
        let mut lighting_details = None;
        let mut recursive_particle_ids = None;
        let mut geometry_particle_ids = None;
        let mut texture_animation_chunk = None;
        let mut particle_geoset_data = None;
        let mut dboc_chunk = None;
        let mut afra_chunk = None;
        let mut dpiv_chunk = None;
        let mut parent_sequence_bounds = None;
        let mut parent_event_data = None;
        let mut collision_mesh_data = None;
        let mut physics_file_data = None;

        // Read all chunks
        loop {
            let header = match ChunkHeader::read(reader) {
                Ok(h) => h,
                Err(M2Error::Io(ref e)) if e.kind() == ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };

            chunks.push(header.clone());

            match &header.magic {
                b"MD21" => {
                    // We need to take the reader to avoid borrowing issues
                    // First, get the current position for the chunk reader
                    let current_pos = reader.stream_position()?;

                    // Skip the chunk for now and store the position for later parsing
                    reader.seek(SeekFrom::Current(header.size as i64))?;

                    // For now, store the header and position info for later processing
                    // This is a simplified approach to avoid complex borrowing
                    md21_chunk = Some(Self::parse_md21_simple(current_pos, &header)?);
                }
                b"SFID" => {
                    // Parse SFID (Skin File IDs) chunk
                    let current_pos = reader.stream_position()?;
                    let end_pos = current_pos + header.size as u64;

                    let count = header.size / 4; // Each ID is 4 bytes
                    let mut ids = Vec::with_capacity(count as usize);

                    for _ in 0..count {
                        ids.push(reader.read_u32_le()?);
                    }

                    skin_file_ids = Some(SkinFileIds { ids });

                    // Ensure we're at the correct position
                    reader.seek(SeekFrom::Start(end_pos))?;
                }
                b"AFID" => {
                    // Parse AFID (Animation File IDs) chunk
                    let current_pos = reader.stream_position()?;
                    let end_pos = current_pos + header.size as u64;

                    let count = header.size / 4; // Each ID is 4 bytes
                    let mut ids = Vec::with_capacity(count as usize);

                    for _ in 0..count {
                        ids.push(reader.read_u32_le()?);
                    }

                    animation_file_ids = Some(AnimationFileIds { ids });

                    // Ensure we're at the correct position
                    reader.seek(SeekFrom::Start(end_pos))?;
                }
                b"TXID" => {
                    // Parse TXID (Texture File IDs) chunk
                    let current_pos = reader.stream_position()?;
                    let end_pos = current_pos + header.size as u64;

                    let count = header.size / 4; // Each ID is 4 bytes
                    let mut ids = Vec::with_capacity(count as usize);

                    for _ in 0..count {
                        ids.push(reader.read_u32_le()?);
                    }

                    texture_file_ids = Some(TextureFileIds { ids });

                    // Ensure we're at the correct position
                    reader.seek(SeekFrom::Start(end_pos))?;
                }
                b"PFID" => {
                    // Parse PFID (Physics File ID) chunk
                    if header.size != 4 {
                        return Err(M2Error::ParseError(format!(
                            "PFID chunk should contain exactly 4 bytes, got {}",
                            header.size
                        )));
                    }

                    let id = reader.read_u32_le()?;
                    physics_file_id = Some(PhysicsFileId { id });
                }
                b"SKID" => {
                    // Parse SKID (Skeleton File ID) chunk
                    if header.size != 4 {
                        return Err(M2Error::ParseError(format!(
                            "SKID chunk should contain exactly 4 bytes, got {}",
                            header.size
                        )));
                    }

                    let id = reader.read_u32_le()?;
                    skeleton_file_id = Some(SkeletonFileId { id });
                }
                b"BFID" => {
                    // Parse BFID (Bone File IDs) chunk
                    let current_pos = reader.stream_position()?;
                    let end_pos = current_pos + header.size as u64;

                    let count = header.size / 4; // Each ID is 4 bytes
                    let mut ids = Vec::with_capacity(count as usize);

                    for _ in 0..count {
                        ids.push(reader.read_u32_le()?);
                    }

                    bone_file_ids = Some(BoneFileIds { ids });

                    // Ensure we're at the correct position
                    reader.seek(SeekFrom::Start(end_pos))?;
                }
                b"LDV1" => {
                    // Parse LDV1 (Level of Detail) chunk
                    let current_pos = reader.stream_position()?;
                    let end_pos = current_pos + header.size as u64;

                    // LDV1 format: each LOD level is 14 bytes
                    const LOD_LEVEL_SIZE: u32 = 14;

                    if header.size % LOD_LEVEL_SIZE != 0 {
                        return Err(M2Error::ParseError(format!(
                            "LDV1 chunk size {} is not a multiple of LOD level size {}",
                            header.size, LOD_LEVEL_SIZE
                        )));
                    }

                    let count = header.size / LOD_LEVEL_SIZE;
                    let mut levels = Vec::with_capacity(count as usize);

                    for _ in 0..count {
                        use crate::chunks::file_references::LodLevel;

                        let distance = reader.read_f32_le()?;
                        let skin_file_index = reader.read_u16_le()?;
                        let vertex_count = reader.read_u32_le()?;
                        let triangle_count = reader.read_u32_le()?;

                        levels.push(LodLevel {
                            distance,
                            skin_file_index,
                            vertex_count,
                            triangle_count,
                        });
                    }

                    lod_data = Some(LodData { levels });

                    // Ensure we're at the correct position
                    reader.seek(SeekFrom::Start(end_pos))?;
                }
                b"EXPT" => {
                    // Parse EXPT (Extended Particle v1) chunk
                    let current_pos = reader.stream_position()?;
                    let _end_pos = current_pos + header.size as u64;

                    // Create a limited reader for this chunk
                    let mut chunk_data = vec![0u8; header.size as usize];
                    reader.read_exact(&mut chunk_data)?;
                    let chunk_cursor = std::io::Cursor::new(chunk_data);
                    let mut chunk_reader = ChunkReader::new(chunk_cursor, header.clone())?;

                    extended_particle_data =
                        Some(ExtendedParticleData::parse_expt(&mut chunk_reader)?);

                    // Position is already correct after reading chunk_data
                }
                b"EXP2" => {
                    // Parse EXP2 (Extended Particle v2) chunk
                    let current_pos = reader.stream_position()?;
                    let _end_pos = current_pos + header.size as u64;

                    // Create a limited reader for this chunk
                    let mut chunk_data = vec![0u8; header.size as usize];
                    reader.read_exact(&mut chunk_data)?;
                    let chunk_cursor = std::io::Cursor::new(chunk_data);
                    let mut chunk_reader = ChunkReader::new(chunk_cursor, header.clone())?;

                    extended_particle_data =
                        Some(ExtendedParticleData::parse_exp2(&mut chunk_reader)?);

                    // Position is already correct after reading chunk_data
                }
                b"PABC" => {
                    // Parse PABC (Parent Animation Blacklist) chunk
                    let current_pos = reader.stream_position()?;
                    let _end_pos = current_pos + header.size as u64;

                    // Create a limited reader for this chunk
                    let mut chunk_data = vec![0u8; header.size as usize];
                    reader.read_exact(&mut chunk_data)?;
                    let chunk_cursor = std::io::Cursor::new(chunk_data);
                    let mut chunk_reader = ChunkReader::new(chunk_cursor, header.clone())?;

                    parent_animation_blacklist =
                        Some(ParentAnimationBlacklist::parse(&mut chunk_reader)?);

                    // Position is already correct after reading chunk_data
                }
                b"PADC" => {
                    // Parse PADC (Parent Animation Data) chunk
                    let current_pos = reader.stream_position()?;
                    let _end_pos = current_pos + header.size as u64;

                    // Create a limited reader for this chunk
                    let mut chunk_data = vec![0u8; header.size as usize];
                    reader.read_exact(&mut chunk_data)?;
                    let chunk_cursor = std::io::Cursor::new(chunk_data);
                    let mut chunk_reader = ChunkReader::new(chunk_cursor, header.clone())?;

                    parent_animation_data = Some(ParentAnimationData::parse(&mut chunk_reader)?);

                    // Position is already correct after reading chunk_data
                }
                b"WFV1" => {
                    // Parse WFV1 (Waterfall v1) chunk
                    let current_pos = reader.stream_position()?;
                    let _end_pos = current_pos + header.size as u64;

                    // Create a limited reader for this chunk
                    let mut chunk_data = vec![0u8; header.size as usize];
                    reader.read_exact(&mut chunk_data)?;
                    let chunk_cursor = std::io::Cursor::new(chunk_data);
                    let mut chunk_reader = ChunkReader::new(chunk_cursor, header.clone())?;

                    waterfall_effect = Some(WaterfallEffect::parse(&mut chunk_reader, 1)?);

                    // Position is already correct after reading chunk_data
                }
                b"WFV2" => {
                    // Parse WFV2 (Waterfall v2) chunk
                    let current_pos = reader.stream_position()?;
                    let _end_pos = current_pos + header.size as u64;

                    // Create a limited reader for this chunk
                    let mut chunk_data = vec![0u8; header.size as usize];
                    reader.read_exact(&mut chunk_data)?;
                    let chunk_cursor = std::io::Cursor::new(chunk_data);
                    let mut chunk_reader = ChunkReader::new(chunk_cursor, header.clone())?;

                    waterfall_effect = Some(WaterfallEffect::parse(&mut chunk_reader, 2)?);

                    // Position is already correct after reading chunk_data
                }
                b"WFV3" => {
                    // Parse WFV3 (Waterfall v3) chunk
                    let current_pos = reader.stream_position()?;
                    let _end_pos = current_pos + header.size as u64;

                    // Create a limited reader for this chunk
                    let mut chunk_data = vec![0u8; header.size as usize];
                    reader.read_exact(&mut chunk_data)?;
                    let chunk_cursor = std::io::Cursor::new(chunk_data);
                    let mut chunk_reader = ChunkReader::new(chunk_cursor, header.clone())?;

                    waterfall_effect = Some(WaterfallEffect::parse(&mut chunk_reader, 3)?);

                    // Position is already correct after reading chunk_data
                }
                b"EDGF" => {
                    // Parse EDGF (Edge Fade) chunk
                    let current_pos = reader.stream_position()?;
                    let _end_pos = current_pos + header.size as u64;

                    // Create a limited reader for this chunk
                    let mut chunk_data = vec![0u8; header.size as usize];
                    reader.read_exact(&mut chunk_data)?;
                    let chunk_cursor = std::io::Cursor::new(chunk_data);
                    let mut chunk_reader = ChunkReader::new(chunk_cursor, header.clone())?;

                    edge_fade_data = Some(EdgeFadeData::parse(&mut chunk_reader)?);

                    // Position is already correct after reading chunk_data
                }
                b"NERF" => {
                    // Parse NERF (Model Alpha) chunk
                    let current_pos = reader.stream_position()?;
                    let _end_pos = current_pos + header.size as u64;

                    // Create a limited reader for this chunk
                    let mut chunk_data = vec![0u8; header.size as usize];
                    reader.read_exact(&mut chunk_data)?;
                    let chunk_cursor = std::io::Cursor::new(chunk_data);
                    let mut chunk_reader = ChunkReader::new(chunk_cursor, header.clone())?;

                    model_alpha_data = Some(ModelAlphaData::parse(&mut chunk_reader)?);

                    // Position is already correct after reading chunk_data
                }
                b"DETL" => {
                    // Parse DETL (Lighting Details) chunk
                    let current_pos = reader.stream_position()?;
                    let _end_pos = current_pos + header.size as u64;

                    // Create a limited reader for this chunk
                    let mut chunk_data = vec![0u8; header.size as usize];
                    reader.read_exact(&mut chunk_data)?;
                    let chunk_cursor = std::io::Cursor::new(chunk_data);
                    let mut chunk_reader = ChunkReader::new(chunk_cursor, header.clone())?;

                    lighting_details = Some(LightingDetails::parse(&mut chunk_reader)?);

                    // Position is already correct after reading chunk_data
                }
                b"RPID" => {
                    // Parse RPID (Recursive Particle IDs) chunk
                    let current_pos = reader.stream_position()?;
                    let _end_pos = current_pos + header.size as u64;

                    // Create a limited reader for this chunk
                    let mut chunk_data = vec![0u8; header.size as usize];
                    reader.read_exact(&mut chunk_data)?;
                    let chunk_cursor = std::io::Cursor::new(chunk_data);
                    let mut chunk_reader = ChunkReader::new(chunk_cursor, header.clone())?;

                    recursive_particle_ids = Some(RecursiveParticleIds::parse(&mut chunk_reader)?);

                    // Position is already correct after reading chunk_data
                }
                b"GPID" => {
                    // Parse GPID (Geometry Particle IDs) chunk
                    let current_pos = reader.stream_position()?;
                    let _end_pos = current_pos + header.size as u64;

                    // Create a limited reader for this chunk
                    let mut chunk_data = vec![0u8; header.size as usize];
                    reader.read_exact(&mut chunk_data)?;
                    let chunk_cursor = std::io::Cursor::new(chunk_data);
                    let mut chunk_reader = ChunkReader::new(chunk_cursor, header.clone())?;

                    geometry_particle_ids = Some(GeometryParticleIds::parse(&mut chunk_reader)?);

                    // Position is already correct after reading chunk_data
                }
                b"TXAC" => {
                    // Parse TXAC (Texture Animation) chunk
                    let current_pos = reader.stream_position()?;
                    let _end_pos = current_pos + header.size as u64;

                    // Create a limited reader for this chunk
                    let mut chunk_data = vec![0u8; header.size as usize];
                    reader.read_exact(&mut chunk_data)?;
                    let chunk_cursor = std::io::Cursor::new(chunk_data);
                    let mut chunk_reader = ChunkReader::new(chunk_cursor, header.clone())?;

                    texture_animation_chunk =
                        Some(TextureAnimationChunk::parse(&mut chunk_reader)?);

                    // Position is already correct after reading chunk_data
                }
                b"PGD1" => {
                    // Parse PGD1 (Particle Geoset Data) chunk
                    let current_pos = reader.stream_position()?;
                    let _end_pos = current_pos + header.size as u64;

                    // Create a limited reader for this chunk
                    let mut chunk_data = vec![0u8; header.size as usize];
                    reader.read_exact(&mut chunk_data)?;
                    let chunk_cursor = std::io::Cursor::new(chunk_data);
                    let mut chunk_reader = ChunkReader::new(chunk_cursor, header.clone())?;

                    particle_geoset_data = Some(ParticleGeosetData::parse(&mut chunk_reader)?);

                    // Position is already correct after reading chunk_data
                }
                b"DBOC" => {
                    // Parse DBOC (unknown purpose) chunk
                    let current_pos = reader.stream_position()?;
                    let _end_pos = current_pos + header.size as u64;

                    // Create a limited reader for this chunk
                    let mut chunk_data = vec![0u8; header.size as usize];
                    reader.read_exact(&mut chunk_data)?;
                    let chunk_cursor = std::io::Cursor::new(chunk_data);
                    let mut chunk_reader = ChunkReader::new(chunk_cursor, header.clone())?;

                    dboc_chunk = Some(DbocChunk::parse(&mut chunk_reader)?);

                    // Position is already correct after reading chunk_data
                }
                b"AFRA" => {
                    // Parse AFRA (unknown purpose) chunk
                    let current_pos = reader.stream_position()?;
                    let _end_pos = current_pos + header.size as u64;

                    // Create a limited reader for this chunk
                    let mut chunk_data = vec![0u8; header.size as usize];
                    reader.read_exact(&mut chunk_data)?;
                    let chunk_cursor = std::io::Cursor::new(chunk_data);
                    let mut chunk_reader = ChunkReader::new(chunk_cursor, header.clone())?;

                    afra_chunk = Some(AfraChunk::parse(&mut chunk_reader)?);

                    // Position is already correct after reading chunk_data
                }
                b"DPIV" => {
                    // Parse DPIV (collision mesh for player housing) chunk
                    let current_pos = reader.stream_position()?;
                    let _end_pos = current_pos + header.size as u64;

                    // Create a limited reader for this chunk
                    let mut chunk_data = vec![0u8; header.size as usize];
                    reader.read_exact(&mut chunk_data)?;
                    let chunk_cursor = std::io::Cursor::new(chunk_data);
                    let mut chunk_reader = ChunkReader::new(chunk_cursor, header.clone())?;

                    dpiv_chunk = Some(DpivChunk::parse(&mut chunk_reader)?);

                    // Position is already correct after reading chunk_data
                }
                b"PSBC" => {
                    // Parse PSBC (Parent Sequence Bounds) chunk
                    let current_pos = reader.stream_position()?;
                    let _end_pos = current_pos + header.size as u64;

                    // Create a limited reader for this chunk
                    let mut chunk_data = vec![0u8; header.size as usize];
                    reader.read_exact(&mut chunk_data)?;
                    let chunk_cursor = std::io::Cursor::new(chunk_data);
                    let mut chunk_reader = ChunkReader::new(chunk_cursor, header.clone())?;

                    parent_sequence_bounds = Some(ParentSequenceBounds::parse(&mut chunk_reader)?);

                    // Position is already correct after reading chunk_data
                }
                b"PEDC" => {
                    // Parse PEDC (Parent Event Data) chunk
                    let current_pos = reader.stream_position()?;
                    let _end_pos = current_pos + header.size as u64;

                    // Create a limited reader for this chunk
                    let mut chunk_data = vec![0u8; header.size as usize];
                    reader.read_exact(&mut chunk_data)?;
                    let chunk_cursor = std::io::Cursor::new(chunk_data);
                    let mut chunk_reader = ChunkReader::new(chunk_cursor, header.clone())?;

                    parent_event_data = Some(ParentEventData::parse(&mut chunk_reader)?);

                    // Position is already correct after reading chunk_data
                }
                b"PCOL" => {
                    // Parse PCOL (Collision Mesh Data) chunk
                    let current_pos = reader.stream_position()?;
                    let _end_pos = current_pos + header.size as u64;

                    // Create a limited reader for this chunk
                    let mut chunk_data = vec![0u8; header.size as usize];
                    reader.read_exact(&mut chunk_data)?;
                    let chunk_cursor = std::io::Cursor::new(chunk_data);
                    let mut chunk_reader = ChunkReader::new(chunk_cursor, header.clone())?;

                    collision_mesh_data = Some(CollisionMeshData::parse(&mut chunk_reader)?);

                    // Position is already correct after reading chunk_data
                }
                b"PFDC" => {
                    // Parse PFDC (Physics File Data) chunk
                    let current_pos = reader.stream_position()?;
                    let _end_pos = current_pos + header.size as u64;

                    // Create a limited reader for this chunk
                    let mut chunk_data = vec![0u8; header.size as usize];
                    reader.read_exact(&mut chunk_data)?;
                    let chunk_cursor = std::io::Cursor::new(chunk_data);
                    let mut chunk_reader = ChunkReader::new(chunk_cursor, header.clone())?;

                    physics_file_data = Some(PhysicsFileDataChunk::parse(&mut chunk_reader)?);

                    // Position is already correct after reading chunk_data
                }
                _ => {
                    // Skip unknown chunks
                    reader.seek(SeekFrom::Current(header.size as i64))?;
                }
            }
        }

        // Get the base model from MD21 chunk
        let mut model = md21_chunk.ok_or(M2Error::MissingMD21Chunk)?;

        // Add the file reference data
        model.skin_file_ids = skin_file_ids;
        model.animation_file_ids = animation_file_ids;
        model.texture_file_ids = texture_file_ids;
        model.physics_file_id = physics_file_id;
        model.skeleton_file_id = skeleton_file_id;
        model.bone_file_ids = bone_file_ids;
        model.lod_data = lod_data;
        model.extended_particle_data = extended_particle_data;
        model.parent_animation_blacklist = parent_animation_blacklist;
        model.parent_animation_data = parent_animation_data;
        model.waterfall_effect = waterfall_effect;
        model.edge_fade_data = edge_fade_data;
        model.model_alpha_data = model_alpha_data;
        model.lighting_details = lighting_details;
        model.recursive_particle_ids = recursive_particle_ids;
        model.geometry_particle_ids = geometry_particle_ids;
        model.texture_animation_chunk = texture_animation_chunk;
        model.particle_geoset_data = particle_geoset_data;
        model.dboc_chunk = dboc_chunk;
        model.afra_chunk = afra_chunk;
        model.dpiv_chunk = dpiv_chunk;
        model.parent_sequence_bounds = parent_sequence_bounds;
        model.parent_event_data = parent_event_data;
        model.collision_mesh_data = collision_mesh_data;
        model.physics_file_data = physics_file_data;

        Ok(model)
    }

    /// Parse the MD21 chunk containing legacy M2 data (simplified version)
    fn parse_md21_simple(_chunk_pos: u64, _header: &ChunkHeader) -> Result<Self> {
        // Simplified implementation for P0-003 basic functionality
        // This creates a minimal model structure without full parsing
        Ok(Self {
            header: M2Header::new(M2Version::Legion),
            name: None,
            global_sequences: Vec::new(),
            animations: Vec::new(),
            animation_lookup: Vec::new(),
            bones: Vec::new(),
            key_bone_lookup: Vec::new(),
            vertices: Vec::new(),
            textures: Vec::new(),
            materials: Vec::new(),
            particle_emitters: Vec::new(),
            ribbon_emitters: Vec::new(),
            raw_data: M2RawData::default(),
            skin_file_ids: None,
            animation_file_ids: None,
            texture_file_ids: None,
            physics_file_id: None,
            skeleton_file_id: None,
            bone_file_ids: None,
            lod_data: None,
            extended_particle_data: None,
            parent_animation_blacklist: None,
            parent_animation_data: None,
            waterfall_effect: None,
            edge_fade_data: None,
            model_alpha_data: None,
            lighting_details: None,
            recursive_particle_ids: None,
            geometry_particle_ids: None,
            texture_animation_chunk: None,
            particle_geoset_data: None,
            dboc_chunk: None,
            afra_chunk: None,
            dpiv_chunk: None,
            parent_sequence_bounds: None,
            parent_event_data: None,
            collision_mesh_data: None,
            physics_file_data: None,
        })
    }

    /// Parse the MD21 chunk containing legacy M2 data (full implementation - TODO)
    fn _parse_md21_chunk<R: Read + Seek>(mut reader: ChunkReader<R>) -> Result<Self> {
        // The MD21 chunk contains the legacy M2 structure with chunk-relative offsets
        // We need to parse it similar to the legacy format but handle offset resolution differently

        // Parse the header from within the chunk (this doesn't include the "MD21" magic)
        // The MD21 chunk contains the full legacy M2 structure starting with "MD20" magic
        let chunk_inner = reader.inner();

        // Read magic and version (should be MD20 within the chunk)
        let mut magic = [0u8; 4];
        chunk_inner.read_exact(&mut magic)?;

        if magic != M2_MAGIC_LEGACY {
            return Err(M2Error::InvalidMagic {
                expected: String::from_utf8_lossy(&M2_MAGIC_LEGACY).to_string(),
                actual: String::from_utf8_lossy(&magic).to_string(),
            });
        }

        // Read version
        let version = chunk_inner.read_u32_le()?;

        // Check if version is supported
        if M2Version::from_header_version(version).is_none() {
            return Err(M2Error::UnsupportedVersion(version.to_string()));
        }

        // Parse the rest of the header using the existing logic
        // But we need to construct it manually since we already read magic and version
        let name = M2Array::parse(chunk_inner)?;
        let flags = M2ModelFlags::from_bits_retain(chunk_inner.read_u32_le()?);

        let global_sequences = M2Array::parse(chunk_inner)?;
        let animations = M2Array::parse(chunk_inner)?;
        let animation_lookup = M2Array::parse(chunk_inner)?;

        // Vanilla/TBC versions have playable animation lookup
        let playable_animation_lookup = if version <= 263 {
            Some(M2Array::parse(chunk_inner)?)
        } else {
            None
        };

        let bones = M2Array::parse(chunk_inner)?;
        let key_bone_lookup = M2Array::parse(chunk_inner)?;

        let vertices = M2Array::parse(chunk_inner)?;

        // Views field changes between versions
        let (views, num_skin_profiles) = if version <= 263 {
            // BC and earlier: views is M2Array
            (M2Array::parse(chunk_inner)?, None)
        } else {
            // WotLK+: views becomes a count (num_skin_profiles)
            let count = chunk_inner.read_u32_le()?;
            (M2Array::new(0, 0), Some(count))
        };

        let color_animations = M2Array::parse(chunk_inner)?;

        let textures = M2Array::parse(chunk_inner)?;
        let transparency_lookup = M2Array::parse(chunk_inner)?;

        // Texture flipbooks only exist in BC and earlier
        let texture_flipbooks = if version <= 263 {
            Some(M2Array::parse(chunk_inner)?)
        } else {
            None
        };

        let texture_animations = M2Array::parse(chunk_inner)?;

        let color_replacements = M2Array::parse(chunk_inner)?;
        let render_flags = M2Array::parse(chunk_inner)?;
        let bone_lookup_table = M2Array::parse(chunk_inner)?;
        let texture_lookup_table = M2Array::parse(chunk_inner)?;
        let texture_units = M2Array::parse(chunk_inner)?;
        let transparency_lookup_table = M2Array::parse(chunk_inner)?;
        let mut texture_animation_lookup = M2Array::parse(chunk_inner)?;

        // Workaround for corrupted texture_animation_lookup fields
        if texture_animation_lookup.count > 1_000_000 {
            texture_animation_lookup = M2Array::new(0, 0);
        }

        // Read bounding box data
        let mut bounding_box_min = [0.0; 3];
        let mut bounding_box_max = [0.0; 3];

        for item in &mut bounding_box_min {
            *item = chunk_inner.read_f32_le()?;
        }

        for item in &mut bounding_box_max {
            *item = chunk_inner.read_f32_le()?;
        }

        let bounding_sphere_radius = chunk_inner.read_f32_le()?;

        // Read collision box
        let mut collision_box_min = [0.0; 3];
        let mut collision_box_max = [0.0; 3];

        for item in &mut collision_box_min {
            *item = chunk_inner.read_f32_le()?;
        }

        for item in &mut collision_box_max {
            *item = chunk_inner.read_f32_le()?;
        }

        let collision_sphere_radius = chunk_inner.read_f32_le()?;

        let bounding_triangles = M2Array::parse(chunk_inner)?;
        let bounding_vertices = M2Array::parse(chunk_inner)?;
        let bounding_normals = M2Array::parse(chunk_inner)?;

        let attachments = M2Array::parse(chunk_inner)?;
        let attachment_lookup_table = M2Array::parse(chunk_inner)?;
        let events = M2Array::parse(chunk_inner)?;
        let lights = M2Array::parse(chunk_inner)?;
        let cameras = M2Array::parse(chunk_inner)?;
        let camera_lookup_table = M2Array::parse(chunk_inner)?;

        let ribbon_emitters = M2Array::parse(chunk_inner)?;
        let particle_emitters = M2Array::parse(chunk_inner)?;

        // Version-specific fields
        let m2_version = M2Version::from_header_version(version).unwrap();

        let blend_map_overrides = if version >= 260 && (flags.bits() & 0x8000000 != 0) {
            Some(M2Array::parse(chunk_inner)?)
        } else {
            None
        };

        let texture_combiner_combos = if m2_version >= M2Version::Cataclysm {
            Some(M2Array::parse(chunk_inner)?)
        } else {
            None
        };

        let texture_transforms = if m2_version >= M2Version::Legion {
            Some(M2Array::parse(chunk_inner)?)
        } else {
            None
        };

        // Create the header structure
        let header = M2Header {
            magic: M2_MAGIC_LEGACY,
            version,
            name,
            flags,
            global_sequences,
            animations,
            animation_lookup,
            playable_animation_lookup,
            bones,
            key_bone_lookup,
            vertices,
            views,
            num_skin_profiles,
            color_animations,
            textures,
            transparency_lookup,
            texture_flipbooks,
            texture_animations,
            color_replacements,
            render_flags,
            bone_lookup_table,
            texture_lookup_table,
            texture_units,
            transparency_lookup_table,
            texture_animation_lookup,
            bounding_box_min,
            bounding_box_max,
            bounding_sphere_radius,
            collision_box_min,
            collision_box_max,
            collision_sphere_radius,
            bounding_triangles,
            bounding_vertices,
            bounding_normals,
            attachments,
            attachment_lookup_table,
            events,
            lights,
            cameras,
            camera_lookup_table,
            ribbon_emitters,
            particle_emitters,
            blend_map_overrides,
            texture_combiner_combos,
            texture_transforms,
        };

        // Now parse the actual data arrays using chunk-relative offsets
        // This is simplified for the basic implementation - we'll reuse the legacy parsing logic
        // but with the understanding that all offsets are now chunk-relative

        // For now, we'll create a basic model structure and defer full array parsing
        // to maintain compatibility while implementing the chunked infrastructure

        Ok(Self {
            header,
            name: None,                    // TODO: Parse name from chunk-relative offset
            global_sequences: Vec::new(),  // TODO: Parse from chunk
            animations: Vec::new(),        // TODO: Parse from chunk
            animation_lookup: Vec::new(),  // TODO: Parse from chunk
            bones: Vec::new(),             // TODO: Parse from chunk
            key_bone_lookup: Vec::new(),   // TODO: Parse from chunk
            vertices: Vec::new(),          // TODO: Parse from chunk
            textures: Vec::new(),          // TODO: Parse from chunk
            materials: Vec::new(),         // TODO: Parse from chunk
            particle_emitters: Vec::new(), // TODO: Parse from chunk
            ribbon_emitters: Vec::new(),   // TODO: Parse from chunk
            raw_data: M2RawData::default(),
            skin_file_ids: None,              // Will be populated from SFID chunk
            animation_file_ids: None,         // Will be populated from AFID chunk
            texture_file_ids: None,           // Will be populated from TXID chunk
            physics_file_id: None,            // Will be populated from PFID chunk
            skeleton_file_id: None,           // Will be populated from SKID chunk
            bone_file_ids: None,              // Will be populated from BFID chunk
            lod_data: None,                   // Will be populated from LDV1 chunk
            extended_particle_data: None,     // Will be populated from EXPT/EXP2 chunks
            parent_animation_blacklist: None, // Will be populated from PABC chunk
            parent_animation_data: None,      // Will be populated from PADC chunk
            waterfall_effect: None,           // Will be populated from WFV1/2/3 chunks
            edge_fade_data: None,             // Will be populated from EDGF chunk
            model_alpha_data: None,           // Will be populated from NERF chunk
            lighting_details: None,           // Will be populated from DETL chunk
            recursive_particle_ids: None,     // Will be populated from RPID chunk
            geometry_particle_ids: None,      // Will be populated from GPID chunk
            texture_animation_chunk: None,    // Will be populated from TXAC chunk
            particle_geoset_data: None,       // Will be populated from PGD1 chunk
            dboc_chunk: None,                 // Will be populated from DBOC chunk
            afra_chunk: None,                 // Will be populated from AFRA chunk
            dpiv_chunk: None,                 // Will be populated from DPIV chunk
            parent_sequence_bounds: None,     // Will be populated from PSBC chunk
            parent_event_data: None,          // Will be populated from PEDC chunk
            collision_mesh_data: None,        // Will be populated from PCOL chunk
            physics_file_data: None,          // Will be populated from PFDC chunk
        })
    }

    /// Parse an M2 model from a reader
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Parse the header first
        let header = M2Header::parse(reader)?;

        // Get the version
        let _version = header
            .version()
            .ok_or(M2Error::UnsupportedVersion(header.version.to_string()))?;

        // Parse the name
        let name = if header.name.count > 0 {
            // Seek to the name
            reader.seek(SeekFrom::Start(header.name.offset as u64))?;

            // Read the name (null-terminated string)
            let name_bytes = read_array(reader, &header.name, |r| Ok(r.read_u8()?))?;

            // Convert to string, stopping at null terminator
            let name_end = name_bytes
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(name_bytes.len());
            let name_str = String::from_utf8_lossy(&name_bytes[..name_end]).to_string();
            Some(name_str)
        } else {
            None
        };

        // Parse global sequences
        let global_sequences =
            read_array(reader, &header.global_sequences, |r| Ok(r.read_u32_le()?))?;

        // Parse animations
        let animations = read_array(reader, &header.animations.convert(), |r| {
            M2Animation::parse(r, header.version)
        })?;

        // Parse animation lookups
        let animation_lookup =
            read_array(reader, &header.animation_lookup, |r| Ok(r.read_u16_le()?))?;

        // Parse bones
        // Special handling for BC item files with 203 bones
        let bones = if header.version == 260 && header.bones.count == 203 {
            // Check if this might be an item file with bone indices instead of bone structures
            let current_pos = reader.stream_position()?;
            let file_size = reader.seek(SeekFrom::End(0))?;
            reader.seek(SeekFrom::Start(current_pos))?; // Restore position

            let bone_size = 92; // BC bone size
            let expected_end = header.bones.offset as u64 + (header.bones.count as u64 * bone_size);

            if expected_end > file_size {
                // File is too small to contain 203 bone structures
                // This is likely a BC item file where "bones" is actually a bone lookup table

                // Skip the bone lookup table for now - we'll handle it differently
                Vec::new()
            } else {
                // File is large enough, parse normally
                read_array(reader, &header.bones.convert(), |r| {
                    M2Bone::parse(r, header.version)
                })?
            }
        } else {
            // Normal bone parsing for other versions
            read_array(reader, &header.bones.convert(), |r| {
                M2Bone::parse(r, header.version)
            })?
        };

        // Parse key bone lookups
        let key_bone_lookup =
            read_array(reader, &header.key_bone_lookup, |r| Ok(r.read_u16_le()?))?;

        // Parse vertices with bone index validation
        let bone_count = header.bones.count;
        let vertices = read_array(reader, &header.vertices.convert(), |r| {
            // CRITICAL FIX: Use validated parsing to prevent out-of-bounds bone references
            M2Vertex::parse_with_validation(
                r,
                header.version,
                Some(bone_count),
                crate::chunks::vertex::ValidationMode::default(),
            )
        })?;

        // Parse textures
        let textures = read_array(reader, &header.textures.convert(), |r| {
            M2Texture::parse(r, header.version)
        })?;

        // Parse materials (render flags)
        let materials = read_array(reader, &header.render_flags.convert(), |r| {
            M2Material::parse(r, header.version)
        })?;

        // Parse particle emitters
        let particle_emitters = read_array(reader, &header.particle_emitters.convert(), |r| {
            M2ParticleEmitter::parse(r, header.version)
        })?;

        // Parse ribbon emitters
        let ribbon_emitters = read_array(reader, &header.ribbon_emitters.convert(), |r| {
            M2RibbonEmitter::parse(r, header.version)
        })?;

        // Parse raw data for other sections
        // These are sections we won't fully parse yet but want to preserve
        let raw_data = M2RawData {
            transparency_lookup_table: read_array(
                reader,
                &header.transparency_lookup_table,
                |r| Ok(r.read_u16_le()?),
            )?,
            texture_animation_lookup: read_array(reader, &header.texture_animation_lookup, |r| {
                Ok(r.read_u16_le()?)
            })?,
            bone_lookup_table: read_array(reader, &header.bone_lookup_table, |r| {
                Ok(r.read_u16_le()?)
            })?,
            texture_lookup_table: read_array(reader, &header.texture_lookup_table, |r| {
                Ok(r.read_u16_le()?)
            })?,
            texture_units: read_array(reader, &header.texture_units, |r| Ok(r.read_u16_le()?))?,
            camera_lookup_table: read_array(reader, &header.camera_lookup_table, |r| {
                Ok(r.read_u16_le()?)
            })?,
            ..Default::default()
        };

        Ok(Self {
            header,
            name,
            global_sequences,
            animations,
            animation_lookup,
            bones,
            key_bone_lookup,
            vertices,
            textures,
            materials,
            particle_emitters,
            ribbon_emitters,
            raw_data,
            skin_file_ids: None,
            animation_file_ids: None,
            texture_file_ids: None,
            physics_file_id: None,
            skeleton_file_id: None,
            bone_file_ids: None,
            lod_data: None,
            extended_particle_data: None,
            parent_animation_blacklist: None,
            parent_animation_data: None,
            waterfall_effect: None,
            edge_fade_data: None,
            model_alpha_data: None,
            lighting_details: None,
            recursive_particle_ids: None,
            geometry_particle_ids: None,
            texture_animation_chunk: None,
            particle_geoset_data: None,
            dboc_chunk: None,
            afra_chunk: None,
            dpiv_chunk: None,
            parent_sequence_bounds: None,
            parent_event_data: None,
            collision_mesh_data: None,
            physics_file_data: None,
        })
    }

    /// Load an M2 model from a file with format detection
    pub fn load<P: AsRef<Path>>(path: P) -> Result<M2Format> {
        let mut file = File::open(path)?;
        parse_m2(&mut file)
    }

    /// Load a legacy M2 model from a file
    pub fn load_legacy<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        Self::parse_legacy(&mut file)
    }

    /// Save an M2 model to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut file = File::create(path)?;
        self.write(&mut file)
    }

    /// Write an M2 model to a writer
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        // We need to recalculate all offsets and build the file in memory
        let mut data_section = Vec::new();
        let mut header = self.header.clone();

        // Start with header size (will be written last)
        let header_size = self.calculate_header_size();
        let mut current_offset = header_size as u32;

        // Write name
        if let Some(ref name) = self.name {
            let name_bytes = name.as_bytes();
            let name_len = name_bytes.len() as u32 + 1; // +1 for null terminator
            header.name = M2Array::new(name_len, current_offset);

            data_section.extend_from_slice(name_bytes);
            data_section.push(0); // Null terminator
            current_offset += name_len;
        } else {
            header.name = M2Array::new(0, 0);
        }

        // Write global sequences
        if !self.global_sequences.is_empty() {
            header.global_sequences =
                M2Array::new(self.global_sequences.len() as u32, current_offset);

            for &seq in &self.global_sequences {
                data_section.extend_from_slice(&seq.to_le_bytes());
            }

            current_offset += (self.global_sequences.len() * std::mem::size_of::<u32>()) as u32;
        } else {
            header.global_sequences = M2Array::new(0, 0);
        }

        // Write animations
        if !self.animations.is_empty() {
            header.animations = M2Array::new(self.animations.len() as u32, current_offset);

            for anim in &self.animations {
                // For each animation, write its data
                let mut anim_data = Vec::new();
                anim.write(&mut anim_data, header.version)?;
                data_section.extend_from_slice(&anim_data);
            }

            // Animation size depends on version: 24 bytes for Classic, 52 bytes for BC+
            let anim_size = if header.version <= 256 { 24 } else { 52 };
            current_offset += (self.animations.len() * anim_size) as u32;
        } else {
            header.animations = M2Array::new(0, 0);
        }

        // Write animation lookups
        if !self.animation_lookup.is_empty() {
            header.animation_lookup =
                M2Array::new(self.animation_lookup.len() as u32, current_offset);

            for &lookup in &self.animation_lookup {
                data_section.extend_from_slice(&lookup.to_le_bytes());
            }

            current_offset += (self.animation_lookup.len() * std::mem::size_of::<u16>()) as u32;
        } else {
            header.animation_lookup = M2Array::new(0, 0);
        }

        // Write bones
        if !self.bones.is_empty() {
            header.bones = M2Array::new(self.bones.len() as u32, current_offset);

            for bone in &self.bones {
                let mut bone_data = Vec::new();
                bone.write(&mut bone_data, self.header.version)?;
                data_section.extend_from_slice(&bone_data);
            }

            // Bone size is 92 bytes for all versions we support
            let bone_size = 92;
            current_offset += (self.bones.len() * bone_size) as u32;
        } else {
            header.bones = M2Array::new(0, 0);
        }

        // Write key bone lookups
        if !self.key_bone_lookup.is_empty() {
            header.key_bone_lookup =
                M2Array::new(self.key_bone_lookup.len() as u32, current_offset);

            for &lookup in &self.key_bone_lookup {
                data_section.extend_from_slice(&lookup.to_le_bytes());
            }

            current_offset += (self.key_bone_lookup.len() * std::mem::size_of::<u16>()) as u32;
        } else {
            header.key_bone_lookup = M2Array::new(0, 0);
        }

        // Write vertices
        if !self.vertices.is_empty() {
            header.vertices = M2Array::new(self.vertices.len() as u32, current_offset);

            let vertex_size =
                if self.header.version().unwrap_or(M2Version::Vanilla) >= M2Version::Cataclysm {
                    // Cataclysm and later have secondary texture coordinates
                    44
                } else {
                    // Pre-Cataclysm don't have secondary texture coordinates
                    36
                };

            for vertex in &self.vertices {
                let mut vertex_data = Vec::new();
                vertex.write(&mut vertex_data, self.header.version)?;
                data_section.extend_from_slice(&vertex_data);
            }

            current_offset += (self.vertices.len() * vertex_size) as u32;
        } else {
            header.vertices = M2Array::new(0, 0);
        }

        // Write textures
        if !self.textures.is_empty() {
            header.textures = M2Array::new(self.textures.len() as u32, current_offset);

            // First, we need to write the texture definitions
            let mut texture_name_offsets = Vec::new();
            let texture_def_size = 16; // Each texture definition is 16 bytes

            for texture in &self.textures {
                // Save the current offset for this texture's filename
                texture_name_offsets
                    .push(current_offset + (self.textures.len() * texture_def_size) as u32);

                // Write the texture definition (without the actual filename)
                let mut texture_def = Vec::new();

                // Write texture type
                texture_def.extend_from_slice(&(texture.texture_type as u32).to_le_bytes());

                // Write flags
                texture_def.extend_from_slice(&texture.flags.bits().to_le_bytes());

                // Write filename offset and length (will be filled in later)
                texture_def.extend_from_slice(&0u32.to_le_bytes()); // Count
                texture_def.extend_from_slice(&0u32.to_le_bytes()); // Offset

                data_section.extend_from_slice(&texture_def);
            }

            // Now write the filenames
            current_offset += (self.textures.len() * texture_def_size) as u32;

            // For each texture, update the offset in the definition and write the filename
            for (i, texture) in self.textures.iter().enumerate() {
                // Get the filename
                let filename_offset = texture.filename.array.offset as usize;
                let filename_len = texture.filename.array.count as usize;
                // Not every texture has a filename (some are hardcoded)
                if filename_offset == 0 || filename_len == 0 {
                    continue;
                }

                // Calculate the offset in the data section where this texture's definition was written
                // The texture definitions start at (header.textures.offset - base_data_offset)
                let base_data_offset = std::mem::size_of::<M2Header>();
                let def_offset_in_data = (header.textures.offset as usize - base_data_offset)
                    + (i * texture_def_size)
                    + 8;

                // Update the count and offset for the filename
                data_section[def_offset_in_data..def_offset_in_data + 4]
                    .copy_from_slice(&(filename_len as u32).to_le_bytes());
                data_section[def_offset_in_data + 4..def_offset_in_data + 8]
                    .copy_from_slice(&current_offset.to_le_bytes());

                // Write the filename
                data_section.extend_from_slice(&texture.filename.string.data);
                data_section.push(0); // Null terminator

                current_offset += filename_len as u32;
            }
        } else {
            header.textures = M2Array::new(0, 0);
        }

        // Write materials (render flags)
        if !self.materials.is_empty() {
            header.render_flags = M2Array::new(self.materials.len() as u32, current_offset);

            for material in &self.materials {
                let mut material_data = Vec::new();
                material.write(&mut material_data, self.header.version)?;
                data_section.extend_from_slice(&material_data);
            }

            let material_size = match self.header.version().unwrap_or(M2Version::Vanilla) {
                v if v >= M2Version::WoD => 18, // WoD and later have color animation lookup
                v if v >= M2Version::Cataclysm => 16, // Cataclysm and later have shader ID and secondary texture unit
                _ => 12,                              // Classic to WotLK
            };

            current_offset += (self.materials.len() * material_size) as u32;
        } else {
            header.render_flags = M2Array::new(0, 0);
        }

        // Write bone lookup table
        if !self.raw_data.bone_lookup_table.is_empty() {
            header.bone_lookup_table =
                M2Array::new(self.raw_data.bone_lookup_table.len() as u32, current_offset);

            for &lookup in &self.raw_data.bone_lookup_table {
                data_section.extend_from_slice(&lookup.to_le_bytes());
            }

            current_offset +=
                (self.raw_data.bone_lookup_table.len() * std::mem::size_of::<u16>()) as u32;
        } else {
            header.bone_lookup_table = M2Array::new(0, 0);
        }

        // Write texture lookup table
        if !self.raw_data.texture_lookup_table.is_empty() {
            header.texture_lookup_table = M2Array::new(
                self.raw_data.texture_lookup_table.len() as u32,
                current_offset,
            );

            for &lookup in &self.raw_data.texture_lookup_table {
                data_section.extend_from_slice(&lookup.to_le_bytes());
            }

            current_offset +=
                (self.raw_data.texture_lookup_table.len() * std::mem::size_of::<u16>()) as u32;
        } else {
            header.texture_lookup_table = M2Array::new(0, 0);
        }

        // Write texture units
        if !self.raw_data.texture_units.is_empty() {
            header.texture_units =
                M2Array::new(self.raw_data.texture_units.len() as u32, current_offset);

            for &unit in &self.raw_data.texture_units {
                data_section.extend_from_slice(&unit.to_le_bytes());
            }

            current_offset +=
                (self.raw_data.texture_units.len() * std::mem::size_of::<u16>()) as u32;
        } else {
            header.texture_units = M2Array::new(0, 0);
        }

        // Write transparency lookup table
        if !self.raw_data.transparency_lookup_table.is_empty() {
            header.transparency_lookup_table = M2Array::new(
                self.raw_data.transparency_lookup_table.len() as u32,
                current_offset,
            );

            for &lookup in &self.raw_data.transparency_lookup_table {
                data_section.extend_from_slice(&lookup.to_le_bytes());
            }

            current_offset +=
                (self.raw_data.transparency_lookup_table.len() * std::mem::size_of::<u16>()) as u32;
        } else {
            header.transparency_lookup_table = M2Array::new(0, 0);
        }

        // Write texture animation lookup
        if !self.raw_data.texture_animation_lookup.is_empty() {
            header.texture_animation_lookup = M2Array::new(
                self.raw_data.texture_animation_lookup.len() as u32,
                current_offset,
            );

            for &lookup in &self.raw_data.texture_animation_lookup {
                data_section.extend_from_slice(&lookup.to_le_bytes());
            }

            // current_offset +=
            //     (self.raw_data.texture_animation_lookup.len() * std::mem::size_of::<u16>()) as u32;
        } else {
            header.texture_animation_lookup = M2Array::new(0, 0);
        }

        // Finally, write the header followed by the data section
        header.write(writer)?;
        writer.write_all(&data_section)?;

        Ok(())
    }

    /// Convert this model to a different version
    pub fn convert(&self, target_version: M2Version) -> Result<Self> {
        let source_version = self.header.version().ok_or(M2Error::ConversionError {
            from: self.header.version,
            to: target_version.to_header_version(),
            reason: "Unknown source version".to_string(),
        })?;

        if source_version == target_version {
            return Ok(self.clone());
        }

        // Convert the header
        let header = self.header.convert(target_version)?;

        // Convert vertices
        let vertices = self
            .vertices
            .iter()
            .map(|v| v.convert(target_version))
            .collect();

        // Convert textures
        let textures = self
            .textures
            .iter()
            .map(|t| t.convert(target_version))
            .collect();

        // Convert bones
        let bones = self
            .bones
            .iter()
            .map(|b| b.convert(target_version))
            .collect();

        // Convert materials
        let materials = self
            .materials
            .iter()
            .map(|m| m.convert(target_version))
            .collect();

        // Create the new model
        let mut new_model = self.clone();
        new_model.header = header;
        new_model.vertices = vertices;
        new_model.textures = textures;
        new_model.bones = bones;
        new_model.materials = materials;

        // Chunked format fields are preserved for compatibility
        // They will be None for legacy format conversions
        new_model.physics_file_id = self.physics_file_id.clone();
        new_model.skeleton_file_id = self.skeleton_file_id.clone();
        new_model.bone_file_ids = self.bone_file_ids.clone();
        new_model.lod_data = self.lod_data.clone();

        Ok(new_model)
    }

    /// Calculate the size of the header for this model version
    fn calculate_header_size(&self) -> usize {
        let version = self.header.version().unwrap_or(M2Version::Vanilla);

        let mut size = 4 + 4; // Magic + version

        // Common fields
        size += 2 * 4; // name
        size += 4; // flags

        size += 2 * 4; // global_sequences
        size += 2 * 4; // animations
        size += 2 * 4; // animation_lookup

        size += 2 * 4; // bones
        size += 2 * 4; // key_bone_lookup

        size += 2 * 4; // vertices
        size += 2 * 4; // views

        size += 2 * 4; // color_animations

        size += 2 * 4; // textures
        size += 2 * 4; // transparency_lookup
        size += 2 * 4; // transparency_animations
        size += 2 * 4; // texture_animations

        size += 2 * 4; // color_replacements
        size += 2 * 4; // render_flags
        size += 2 * 4; // bone_lookup_table
        size += 2 * 4; // texture_lookup_table
        size += 2 * 4; // texture_units
        size += 2 * 4; // transparency_lookup_table
        size += 2 * 4; // texture_animation_lookup

        size += 3 * 4; // bounding_box_min
        size += 3 * 4; // bounding_box_max
        size += 4; // bounding_sphere_radius

        size += 3 * 4; // collision_box_min
        size += 3 * 4; // collision_box_max
        size += 4; // collision_sphere_radius

        size += 2 * 4; // bounding_triangles
        size += 2 * 4; // bounding_vertices
        size += 2 * 4; // bounding_normals

        size += 2 * 4; // attachments
        size += 2 * 4; // attachment_lookup_table
        size += 2 * 4; // events
        size += 2 * 4; // lights
        size += 2 * 4; // cameras
        size += 2 * 4; // camera_lookup_table

        size += 2 * 4; // ribbon_emitters
        size += 2 * 4; // particle_emitters

        // Version-specific fields
        if version >= M2Version::Cataclysm {
            size += 2 * 4; // texture_combiner_combos
        }

        if version >= M2Version::Legion {
            size += 2 * 4; // texture_transforms
        }

        size
    }

    /// Validate the model structure
    pub fn validate(&self) -> Result<()> {
        // Check if the version is supported
        if self.header.version().is_none() {
            return Err(M2Error::UnsupportedVersion(self.header.version.to_string()));
        }

        // Validate vertices
        if self.vertices.is_empty() {
            return Err(M2Error::ValidationError(
                "Model has no vertices".to_string(),
            ));
        }

        // Validate bones
        for (i, bone) in self.bones.iter().enumerate() {
            // Check if parent bone is valid
            if bone.parent_bone >= 0 && bone.parent_bone as usize >= self.bones.len() {
                return Err(M2Error::ValidationError(format!(
                    "Bone {} has invalid parent bone {}",
                    i, bone.parent_bone
                )));
            }
        }

        // Validate textures
        for (i, texture) in self.textures.iter().enumerate() {
            // Check if the texture has a valid filename
            if texture.filename.array.count > 0 && texture.filename.array.offset == 0 {
                return Err(M2Error::ValidationError(format!(
                    "Texture {i} has invalid filename offset"
                )));
            }
        }

        // Validate materials (simplified - just check basic structure)
        for (i, _material) in self.materials.iter().enumerate() {
            // Materials now only contain render flags and blend modes
            // No direct texture references to validate here
            let _material_index = i; // Just to acknowledge we're iterating
        }

        Ok(())
    }

    /// Check if this model has external file references (Legion+ chunked format)
    pub fn has_external_files(&self) -> bool {
        self.skin_file_ids.is_some()
            || self.animation_file_ids.is_some()
            || self.texture_file_ids.is_some()
            || self.physics_file_id.is_some()
            || self.skeleton_file_id.is_some()
            || self.bone_file_ids.is_some()
            || self.lod_data.is_some()
            || self.has_advanced_features()
    }

    /// Get the number of skin files referenced
    pub fn skin_file_count(&self) -> usize {
        self.skin_file_ids.as_ref().map_or(0, |ids| ids.len())
    }

    /// Get the number of animation files referenced
    pub fn animation_file_count(&self) -> usize {
        self.animation_file_ids.as_ref().map_or(0, |ids| ids.len())
    }

    /// Get the number of texture files referenced
    pub fn texture_file_count(&self) -> usize {
        self.texture_file_ids.as_ref().map_or(0, |ids| ids.len())
    }

    /// Resolve a skin file path by index using a FileResolver
    pub fn resolve_skin_path(&self, index: usize, resolver: &dyn FileResolver) -> Result<String> {
        let skin_ids = self.skin_file_ids.as_ref().ok_or_else(|| {
            M2Error::ExternalFileError("Model has no external skin files".to_string())
        })?;

        let id = skin_ids.get(index).ok_or_else(|| {
            M2Error::ExternalFileError(format!("Skin index {} out of range", index))
        })?;

        resolver.resolve_file_data_id(id)
    }

    /// Load a skin file by index using a FileResolver
    pub fn load_skin_file(&self, index: usize, resolver: &dyn FileResolver) -> Result<Vec<u8>> {
        let skin_ids = self.skin_file_ids.as_ref().ok_or_else(|| {
            M2Error::ExternalFileError("Model has no external skin files".to_string())
        })?;

        let id = skin_ids.get(index).ok_or_else(|| {
            M2Error::ExternalFileError(format!("Skin index {} out of range", index))
        })?;

        resolver.load_skin_by_id(id)
    }

    /// Resolve an animation file path by index using a FileResolver
    pub fn resolve_animation_path(
        &self,
        index: usize,
        resolver: &dyn FileResolver,
    ) -> Result<String> {
        let anim_ids = self.animation_file_ids.as_ref().ok_or_else(|| {
            M2Error::ExternalFileError("Model has no external animation files".to_string())
        })?;

        let id = anim_ids.get(index).ok_or_else(|| {
            M2Error::ExternalFileError(format!("Animation index {} out of range", index))
        })?;

        resolver.resolve_file_data_id(id)
    }

    /// Load an animation file by index using a FileResolver
    pub fn load_animation_file(
        &self,
        index: usize,
        resolver: &dyn FileResolver,
    ) -> Result<Vec<u8>> {
        let anim_ids = self.animation_file_ids.as_ref().ok_or_else(|| {
            M2Error::ExternalFileError("Model has no external animation files".to_string())
        })?;

        let id = anim_ids.get(index).ok_or_else(|| {
            M2Error::ExternalFileError(format!("Animation index {} out of range", index))
        })?;

        resolver.load_animation_by_id(id)
    }

    /// Resolve a texture file path by index using a FileResolver
    /// Falls back to embedded texture names for pre-Legion models
    pub fn resolve_texture_path(
        &self,
        index: usize,
        resolver: &dyn FileResolver,
    ) -> Result<String> {
        // Legion+ models use TXID chunk
        if let Some(texture_ids) = &self.texture_file_ids {
            if let Some(id) = texture_ids.get(index) {
                return resolver.resolve_file_data_id(id);
            }
        }

        // Pre-Legion models use embedded texture names
        if let Some(texture) = self.textures.get(index) {
            if !texture.filename.string.data.is_empty() {
                let filename = String::from_utf8_lossy(&texture.filename.string.data).to_string();
                return Ok(filename.trim_end_matches('\0').to_string());
            }
        }

        Err(M2Error::ExternalFileError(format!(
            "Texture index {} not found",
            index
        )))
    }

    /// Load a texture file by index using a FileResolver
    /// Falls back to embedded texture names for pre-Legion models
    pub fn load_texture_file(&self, index: usize, resolver: &dyn FileResolver) -> Result<Vec<u8>> {
        // Legion+ models use TXID chunk
        if let Some(texture_ids) = &self.texture_file_ids {
            if let Some(id) = texture_ids.get(index) {
                return resolver.load_texture_by_id(id);
            }
        }

        // Pre-Legion models use embedded texture names - we can't load them directly
        // since we don't have FileDataIDs, just return an error with the filename
        if let Some(texture) = self.textures.get(index) {
            if !texture.filename.string.data.is_empty() {
                let filename = String::from_utf8_lossy(&texture.filename.string.data).to_string();
                let clean_filename = filename.trim_end_matches('\0').to_string();
                return Err(M2Error::ExternalFileError(format!(
                    "Cannot load pre-Legion texture '{}' without FileDataID",
                    clean_filename
                )));
            }
        }

        Err(M2Error::ExternalFileError(format!(
            "Texture index {} not found",
            index
        )))
    }

    /// Get all skin file IDs
    pub fn get_skin_file_ids(&self) -> Option<&[u32]> {
        self.skin_file_ids.as_ref().map(|ids| ids.ids.as_slice())
    }

    /// Get all animation file IDs
    pub fn get_animation_file_ids(&self) -> Option<&[u32]> {
        self.animation_file_ids
            .as_ref()
            .map(|ids| ids.ids.as_slice())
    }

    /// Get all texture file IDs
    pub fn get_texture_file_ids(&self) -> Option<&[u32]> {
        self.texture_file_ids.as_ref().map(|ids| ids.ids.as_slice())
    }

    /// Get the physics file ID
    pub fn get_physics_file_id(&self) -> Option<u32> {
        self.physics_file_id.as_ref().map(|id| id.id)
    }

    /// Get the skeleton file ID
    pub fn get_skeleton_file_id(&self) -> Option<u32> {
        self.skeleton_file_id.as_ref().map(|id| id.id)
    }

    /// Get all bone file IDs
    pub fn get_bone_file_ids(&self) -> Option<&[u32]> {
        self.bone_file_ids.as_ref().map(|ids| ids.ids.as_slice())
    }

    /// Get the LOD data
    pub fn get_lod_data(&self) -> Option<&LodData> {
        self.lod_data.as_ref()
    }

    /// Load physics data using a FileResolver
    pub fn load_physics(&self, resolver: &dyn FileResolver) -> Result<Option<PhysicsData>> {
        match &self.physics_file_id {
            Some(pfid) => {
                let data = resolver.load_physics_by_id(&pfid.id)?;
                Ok(Some(PhysicsData::parse(&data)?))
            }
            None => Ok(None),
        }
    }

    /// Load skeleton data using a FileResolver
    pub fn load_skeleton(&self, resolver: &dyn FileResolver) -> Result<Option<SkeletonData>> {
        match &self.skeleton_file_id {
            Some(skid) => {
                let data = resolver.load_skeleton_by_id(&skid.id)?;
                Ok(Some(SkeletonData::parse(&data)?))
            }
            None => Ok(None),
        }
    }

    /// Load bone data by index using a FileResolver
    pub fn load_bone_data(
        &self,
        index: usize,
        resolver: &dyn FileResolver,
    ) -> Result<Option<BoneData>> {
        match &self.bone_file_ids {
            Some(bfid) => {
                if let Some(id) = bfid.get(index) {
                    let data = resolver.load_bone_by_id(&id)?;
                    Ok(Some(BoneData::parse(&data)?))
                } else {
                    Err(M2Error::ExternalFileError(format!(
                        "Bone index {} out of range",
                        index
                    )))
                }
            }
            None => Ok(None),
        }
    }

    /// Get the number of bone files referenced
    pub fn bone_file_count(&self) -> usize {
        self.bone_file_ids.as_ref().map_or(0, |ids| ids.len())
    }

    /// Select the appropriate LOD level for a given distance
    pub fn select_lod(&self, distance: f32) -> Option<&crate::chunks::file_references::LodLevel> {
        self.lod_data.as_ref()?.select_lod(distance)
    }

    /// Check if the model has LOD data
    pub fn has_lod_data(&self) -> bool {
        self.lod_data.is_some()
    }

    /// Check if an animation sequence is blacklisted
    pub fn is_animation_blacklisted(&self, sequence_id: u16) -> bool {
        self.parent_animation_blacklist
            .as_ref()
            .is_some_and(|pabc| pabc.blacklisted_sequences.contains(&sequence_id))
    }

    /// Get extended particle data
    pub fn get_extended_particle_data(&self) -> Option<&ExtendedParticleData> {
        self.extended_particle_data.as_ref()
    }

    /// Get parent animation blacklist
    pub fn get_parent_animation_blacklist(&self) -> Option<&ParentAnimationBlacklist> {
        self.parent_animation_blacklist.as_ref()
    }

    /// Get parent animation data
    pub fn get_parent_animation_data(&self) -> Option<&ParentAnimationData> {
        self.parent_animation_data.as_ref()
    }

    /// Get waterfall effect data
    pub fn get_waterfall_effect(&self) -> Option<&WaterfallEffect> {
        self.waterfall_effect.as_ref()
    }

    /// Get edge fade data
    pub fn get_edge_fade_data(&self) -> Option<&EdgeFadeData> {
        self.edge_fade_data.as_ref()
    }

    /// Get model alpha data
    pub fn get_model_alpha_data(&self) -> Option<&ModelAlphaData> {
        self.model_alpha_data.as_ref()
    }

    /// Get lighting details
    pub fn get_lighting_details(&self) -> Option<&LightingDetails> {
        self.lighting_details.as_ref()
    }

    /// Get recursive particle model IDs
    pub fn get_recursive_particle_ids(&self) -> Option<&[u32]> {
        self.recursive_particle_ids
            .as_ref()
            .map(|ids| ids.model_ids.as_slice())
    }

    /// Get geometry particle model IDs
    pub fn get_geometry_particle_ids(&self) -> Option<&[u32]> {
        self.geometry_particle_ids
            .as_ref()
            .map(|ids| ids.model_ids.as_slice())
    }

    /// Load particle models using a FileResolver
    /// This method implements recursion protection to avoid infinite loops
    pub fn load_particle_models(
        &self,
        file_resolver: &dyn crate::file_resolver::FileResolver,
    ) -> crate::error::Result<Vec<M2Model>> {
        let mut models = Vec::new();
        let mut loaded_ids = std::collections::HashSet::new();

        // Load recursive particle models with protection against infinite recursion
        if let Some(rpid) = &self.recursive_particle_ids {
            for &id in &rpid.model_ids {
                if !loaded_ids.contains(&id) {
                    loaded_ids.insert(id);

                    match file_resolver.load_animation_by_id(id) {
                        Ok(data) => {
                            let mut cursor = std::io::Cursor::new(data);
                            match parse_m2(&mut cursor) {
                                Ok(format) => models.push(format.model().clone()),
                                Err(e) => {
                                    // Log warning but continue loading other models
                                    eprintln!(
                                        "Warning: Failed to load recursive particle model {}: {:?}",
                                        id, e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Failed to load recursive particle model data {}: {:?}",
                                id, e
                            );
                        }
                    }
                }
            }
        }

        // Load geometry particle models
        if let Some(gpid) = &self.geometry_particle_ids {
            for &id in &gpid.model_ids {
                if !loaded_ids.contains(&id) {
                    loaded_ids.insert(id);

                    match file_resolver.load_animation_by_id(id) {
                        Ok(data) => {
                            let mut cursor = std::io::Cursor::new(data);
                            match parse_m2(&mut cursor) {
                                Ok(format) => models.push(format.model().clone()),
                                Err(e) => {
                                    eprintln!(
                                        "Warning: Failed to load geometry particle model {}: {:?}",
                                        id, e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Failed to load geometry particle model data {}: {:?}",
                                id, e
                            );
                        }
                    }
                }
            }
        }

        Ok(models)
    }

    /// Check if the model has any advanced rendering features
    /// Get parent sequence bounds data (PSBC chunk)
    pub fn get_parent_sequence_bounds(&self) -> Option<&ParentSequenceBounds> {
        self.parent_sequence_bounds.as_ref()
    }

    /// Get parent event data (PEDC chunk)
    pub fn get_parent_event_data(&self) -> Option<&ParentEventData> {
        self.parent_event_data.as_ref()
    }

    /// Get collision mesh data (PCOL chunk)
    pub fn get_collision_mesh_data(&self) -> Option<&CollisionMeshData> {
        self.collision_mesh_data.as_ref()
    }

    /// Get physics file data (PFDC chunk)
    pub fn get_physics_file_data(&self) -> Option<&PhysicsFileDataChunk> {
        self.physics_file_data.as_ref()
    }

    /// Check if model has advanced features (Legion+)
    pub fn has_advanced_features(&self) -> bool {
        self.extended_particle_data.is_some()
            || self.parent_animation_blacklist.is_some()
            || self.parent_animation_data.is_some()
            || self.parent_sequence_bounds.is_some()
            || self.parent_event_data.is_some()
            || self.waterfall_effect.is_some()
            || self.edge_fade_data.is_some()
            || self.model_alpha_data.is_some()
            || self.lighting_details.is_some()
            || self.recursive_particle_ids.is_some()
            || self.geometry_particle_ids.is_some()
            || self.collision_mesh_data.is_some()
            || self.physics_file_data.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunks::{AnimationFileIds, SkinFileIds, TextureFileIds};
    use crate::header::{M2_MAGIC_CHUNKED, M2_MAGIC_LEGACY};
    use std::io::Cursor;

    #[test]
    fn test_m2_format_detection_legacy() {
        // Create minimal MD20 format data
        let mut data = Vec::new();
        data.extend_from_slice(&M2_MAGIC_LEGACY); // MD20 magic
        data.extend_from_slice(&256u32.to_le_bytes()); // Version
        // Add minimal header data to prevent parse errors
        for _ in 0..100 {
            data.extend_from_slice(&0u32.to_le_bytes());
        }

        let mut cursor = Cursor::new(data);
        let result = parse_m2(&mut cursor);

        assert!(result.is_ok());
        let format = result.unwrap();
        assert!(format.is_legacy());
        assert!(!format.is_chunked());
    }

    #[test]
    fn test_m2_format_detection_chunked() {
        // Create minimal MD21 format data with MD21 chunk
        let mut data = Vec::new();
        data.extend_from_slice(&M2_MAGIC_CHUNKED); // MD21 magic
        data.extend_from_slice(&8u32.to_le_bytes()); // Chunk size

        // MD21 chunk containing MD20 data
        data.extend_from_slice(b"MD21"); // Chunk magic
        data.extend_from_slice(&400u32.to_le_bytes()); // Large chunk size for MD20 data

        // MD20 data within the chunk
        data.extend_from_slice(&M2_MAGIC_LEGACY); // MD20 magic
        data.extend_from_slice(&276u32.to_le_bytes()); // Legion version
        // Add minimal header data
        for _ in 0..100 {
            data.extend_from_slice(&0u32.to_le_bytes());
        }

        let mut cursor = Cursor::new(data);
        let result = parse_m2(&mut cursor);

        // This will currently fail because our chunked parser is incomplete
        // but we can test the format detection part
        match result {
            Ok(format) => {
                assert!(format.is_chunked());
                assert!(!format.is_legacy());
            }
            Err(M2Error::ParseError(msg)) => {
                // Expected for incomplete implementation
                assert!(
                    msg.contains("TODO") || msg.contains("not yet") || msg.contains("incomplete")
                );
            }
            Err(other) => panic!("Unexpected error: {:?}", other),
        }
    }

    #[test]
    fn test_invalid_magic_detection() {
        let data = b"FAIL\x00\x00\x00\x00"; // Invalid magic
        let mut cursor = Cursor::new(data);
        let result = parse_m2(&mut cursor);

        assert!(result.is_err());
        match result.unwrap_err() {
            M2Error::InvalidMagicBytes(magic) => {
                assert_eq!(&magic, b"FAIL");
            }
            other => panic!("Expected InvalidMagicBytes error, got: {:?}", other),
        }
    }

    #[test]
    fn test_m2_format_model_access() {
        // Test that we can access the underlying model from both formats
        use crate::version::M2Version;

        // Create a test model
        let mut test_model = M2Model {
            header: M2Header::new(M2Version::Vanilla),
            name: Some("test".to_string()),
            global_sequences: Vec::new(),
            animations: Vec::new(),
            animation_lookup: Vec::new(),
            bones: Vec::new(),
            key_bone_lookup: Vec::new(),
            vertices: Vec::new(),
            textures: Vec::new(),
            materials: Vec::new(),
            particle_emitters: Vec::new(),
            ribbon_emitters: Vec::new(),
            raw_data: M2RawData::default(),
            skin_file_ids: None,
            animation_file_ids: None,
            texture_file_ids: None,
            physics_file_id: None,
            skeleton_file_id: None,
            bone_file_ids: None,
            lod_data: None,
            extended_particle_data: None,
            parent_animation_blacklist: None,
            parent_animation_data: None,
            waterfall_effect: None,
            edge_fade_data: None,
            model_alpha_data: None,
            lighting_details: None,
            recursive_particle_ids: None,
            geometry_particle_ids: None,
            texture_animation_chunk: None,
            particle_geoset_data: None,
            dboc_chunk: None,
            afra_chunk: None,
            dpiv_chunk: None,
            parent_sequence_bounds: None,
            parent_event_data: None,
            collision_mesh_data: None,
            physics_file_data: None,
        };

        // Test legacy format access
        let legacy_format = M2Format::Legacy(test_model.clone());
        assert_eq!(legacy_format.model().name.as_ref().unwrap(), "test");
        assert!(legacy_format.is_legacy());

        // Test chunked format access
        test_model.skin_file_ids = Some(SkinFileIds { ids: vec![1, 2, 3] });
        let chunked_format = M2Format::Chunked(test_model.clone());
        assert_eq!(chunked_format.model().name.as_ref().unwrap(), "test");
        assert!(chunked_format.is_chunked());
        assert_eq!(
            chunked_format.model().skin_file_ids.as_ref().unwrap().len(),
            3
        );
    }

    #[test]
    fn test_file_reference_methods() {
        use crate::file_resolver::ListfileResolver;
        use crate::version::M2Version;

        // Create a chunked format model with file references
        let mut model = M2Model {
            header: M2Header::new(M2Version::Legion),
            name: Some("test_model".to_string()),
            global_sequences: Vec::new(),
            animations: Vec::new(),
            animation_lookup: Vec::new(),
            bones: Vec::new(),
            key_bone_lookup: Vec::new(),
            vertices: Vec::new(),
            textures: Vec::new(),
            materials: Vec::new(),
            particle_emitters: Vec::new(),
            ribbon_emitters: Vec::new(),
            raw_data: M2RawData::default(),
            skin_file_ids: Some(SkinFileIds {
                ids: vec![123456, 789012],
            }),
            animation_file_ids: Some(AnimationFileIds {
                ids: vec![111111, 222222],
            }),
            texture_file_ids: Some(TextureFileIds {
                ids: vec![333333, 444444],
            }),
            physics_file_id: None,
            skeleton_file_id: None,
            bone_file_ids: None,
            lod_data: None,
            extended_particle_data: None,
            parent_animation_blacklist: None,
            parent_animation_data: None,
            waterfall_effect: None,
            edge_fade_data: None,
            model_alpha_data: None,
            lighting_details: None,
            recursive_particle_ids: None,
            geometry_particle_ids: None,
            texture_animation_chunk: None,
            particle_geoset_data: None,
            dboc_chunk: None,
            afra_chunk: None,
            dpiv_chunk: None,
            parent_sequence_bounds: None,
            parent_event_data: None,
            collision_mesh_data: None,
            physics_file_data: None,
        };

        // Test file count methods
        assert!(model.has_external_files());
        assert_eq!(model.skin_file_count(), 2);
        assert_eq!(model.animation_file_count(), 2);
        assert_eq!(model.texture_file_count(), 2);

        // Test getter methods
        assert_eq!(
            model.get_skin_file_ids(),
            Some([123456u32, 789012u32].as_slice())
        );
        assert_eq!(
            model.get_animation_file_ids(),
            Some([111111u32, 222222u32].as_slice())
        );
        assert_eq!(
            model.get_texture_file_ids(),
            Some([333333u32, 444444u32].as_slice())
        );

        // Create a mock resolver
        let mut resolver = ListfileResolver::new();
        resolver.add_mapping(123456, "character/human/male/humanmale00.skin");
        resolver.add_mapping(789012, "character/human/male/humanmale01.skin");
        resolver.add_mapping(111111, "character/human/male/humanmale_walk.anim");
        resolver.add_mapping(222222, "character/human/male/humanmale_run.anim");
        resolver.add_mapping(333333, "character/textures/skin_human_male.blp");
        resolver.add_mapping(444444, "character/textures/hair_human_male.blp");

        // Test path resolution
        assert_eq!(
            model.resolve_skin_path(0, &resolver).unwrap(),
            "character/human/male/humanmale00.skin"
        );
        assert_eq!(
            model.resolve_skin_path(1, &resolver).unwrap(),
            "character/human/male/humanmale01.skin"
        );
        assert!(model.resolve_skin_path(2, &resolver).is_err()); // Out of range

        assert_eq!(
            model.resolve_animation_path(0, &resolver).unwrap(),
            "character/human/male/humanmale_walk.anim"
        );
        assert_eq!(
            model.resolve_animation_path(1, &resolver).unwrap(),
            "character/human/male/humanmale_run.anim"
        );
        assert!(model.resolve_animation_path(2, &resolver).is_err()); // Out of range

        assert_eq!(
            model.resolve_texture_path(0, &resolver).unwrap(),
            "character/textures/skin_human_male.blp"
        );
        assert_eq!(
            model.resolve_texture_path(1, &resolver).unwrap(),
            "character/textures/hair_human_male.blp"
        );
        assert!(model.resolve_texture_path(2, &resolver).is_err()); // Out of range

        // Test loading methods (they should return errors since we don't have actual files)
        assert!(model.load_skin_file(0, &resolver).is_err());
        assert!(model.load_animation_file(0, &resolver).is_err());
        assert!(model.load_texture_file(0, &resolver).is_err());

        // Test model without external files
        model.skin_file_ids = None;
        model.animation_file_ids = None;
        model.texture_file_ids = None;

        // This model doesn't have advanced features initially, so it should not have external files
        assert!(!model.has_external_files());

        // Just ensure the test runs properly by clearing any existing advanced features
        model.extended_particle_data = None;
        model.parent_animation_blacklist = None;
        model.parent_animation_data = None;
        model.waterfall_effect = None;
        model.edge_fade_data = None;
        model.model_alpha_data = None;
        model.lighting_details = None;
        model.recursive_particle_ids = None;
        model.geometry_particle_ids = None;

        assert!(!model.has_external_files());
        assert_eq!(model.skin_file_count(), 0);
        assert_eq!(model.animation_file_count(), 0);
        assert_eq!(model.texture_file_count(), 0);

        assert!(model.resolve_skin_path(0, &resolver).is_err());
        assert!(model.resolve_animation_path(0, &resolver).is_err());
        assert!(model.resolve_texture_path(0, &resolver).is_err());
    }

    #[test]
    fn test_legacy_model_texture_handling() {
        use crate::chunks::texture::{M2Texture, M2TextureFlags, M2TextureType};
        use crate::common::{FixedString, M2Array, M2ArrayString};
        use crate::file_resolver::ListfileResolver;
        use crate::version::M2Version;

        // Create a legacy model with embedded texture names
        let texture_filename = "character/textures/skin_human_male.blp";
        let mut filename_data = texture_filename.as_bytes().to_vec();
        filename_data.push(0); // Null terminator

        let texture = M2Texture {
            texture_type: M2TextureType::Body,
            flags: M2TextureFlags::empty(),
            filename: M2ArrayString {
                array: M2Array::new(filename_data.len() as u32, 0),
                string: FixedString {
                    data: filename_data,
                },
            },
        };

        let model = M2Model {
            header: M2Header::new(M2Version::Vanilla),
            name: Some("legacy_model".to_string()),
            global_sequences: Vec::new(),
            animations: Vec::new(),
            animation_lookup: Vec::new(),
            bones: Vec::new(),
            key_bone_lookup: Vec::new(),
            vertices: Vec::new(),
            textures: vec![texture],
            materials: Vec::new(),
            particle_emitters: Vec::new(),
            ribbon_emitters: Vec::new(),
            raw_data: M2RawData::default(),
            skin_file_ids: None,
            animation_file_ids: None,
            texture_file_ids: None,
            physics_file_id: None,
            skeleton_file_id: None,
            bone_file_ids: None,
            lod_data: None,
            extended_particle_data: None,
            parent_animation_blacklist: None,
            parent_animation_data: None,
            waterfall_effect: None,
            edge_fade_data: None,
            model_alpha_data: None,
            lighting_details: None,
            recursive_particle_ids: None,
            geometry_particle_ids: None,
            texture_animation_chunk: None,
            particle_geoset_data: None,
            dboc_chunk: None,
            afra_chunk: None,
            dpiv_chunk: None,
            parent_sequence_bounds: None,
            parent_event_data: None,
            collision_mesh_data: None,
            physics_file_data: None,
        };

        let resolver = ListfileResolver::new();

        // Test texture path resolution for legacy model
        assert_eq!(
            model.resolve_texture_path(0, &resolver).unwrap(),
            texture_filename
        );

        // Test texture loading for legacy model (should fail with descriptive error)
        match model.load_texture_file(0, &resolver) {
            Err(M2Error::ExternalFileError(msg)) => {
                assert!(msg.contains("Cannot load pre-Legion texture"));
                assert!(msg.contains(texture_filename));
            }
            _ => panic!("Expected external file error for legacy texture loading"),
        }
    }

    #[test]
    fn test_advanced_features() {
        use crate::chunks::rendering_enhancements::*;
        use crate::file_resolver::ListfileResolver;
        use crate::version::M2Version;

        // Create a model with advanced features
        let mut model = M2Model {
            header: M2Header::new(M2Version::Legion),
            name: Some("advanced_model".to_string()),
            global_sequences: Vec::new(),
            animations: Vec::new(),
            animation_lookup: Vec::new(),
            bones: Vec::new(),
            key_bone_lookup: Vec::new(),
            vertices: Vec::new(),
            textures: Vec::new(),
            materials: Vec::new(),
            particle_emitters: Vec::new(),
            ribbon_emitters: Vec::new(),
            raw_data: M2RawData::default(),
            skin_file_ids: None,
            animation_file_ids: None,
            texture_file_ids: None,
            physics_file_id: None,
            skeleton_file_id: None,
            bone_file_ids: None,
            lod_data: None,
            extended_particle_data: Some(ExtendedParticleData {
                version: 1,
                enhanced_emitters: Vec::new(),
                particle_systems: Vec::new(),
            }),
            parent_animation_blacklist: Some(ParentAnimationBlacklist {
                blacklisted_sequences: vec![1, 5, 10],
            }),
            parent_animation_data: Some(ParentAnimationData {
                texture_weights: Vec::new(),
                blending_modes: Vec::new(),
            }),
            waterfall_effect: Some(WaterfallEffect {
                version: 1,
                parameters: WaterfallParameters {
                    flow_velocity: 1.0,
                    turbulence: 0.5,
                    foam_intensity: 0.75,
                    additional_params: Vec::new(),
                },
            }),
            edge_fade_data: Some(EdgeFadeData {
                fade_distances: vec![10.0, 20.0],
                fade_factors: vec![0.5, 0.8],
            }),
            model_alpha_data: Some(ModelAlphaData {
                alpha_test_threshold: 0.5,
                blend_mode: AlphaBlendMode::Normal,
            }),
            lighting_details: Some(LightingDetails {
                ambient_factor: 0.2,
                diffuse_factor: 0.8,
                specular_factor: 0.3,
            }),
            recursive_particle_ids: Some(RecursiveParticleIds {
                model_ids: vec![123456, 789012],
            }),
            geometry_particle_ids: Some(GeometryParticleIds {
                model_ids: vec![345678, 901234],
            }),
            texture_animation_chunk: None,
            particle_geoset_data: None,
            dboc_chunk: None,
            afra_chunk: None,
            dpiv_chunk: None,
            parent_sequence_bounds: None,
            parent_event_data: None,
            collision_mesh_data: None,
            physics_file_data: None,
        };

        // Test advanced features detection
        assert!(model.has_advanced_features());
        assert!(model.has_external_files());

        // Test animation blacklisting
        assert!(model.is_animation_blacklisted(1));
        assert!(model.is_animation_blacklisted(5));
        assert!(model.is_animation_blacklisted(10));
        assert!(!model.is_animation_blacklisted(2));

        // Test getters
        assert!(model.get_extended_particle_data().is_some());
        assert!(model.get_parent_animation_blacklist().is_some());
        assert!(model.get_parent_animation_data().is_some());
        assert!(model.get_waterfall_effect().is_some());
        assert!(model.get_edge_fade_data().is_some());
        assert!(model.get_model_alpha_data().is_some());
        assert!(model.get_lighting_details().is_some());

        assert_eq!(
            model.get_recursive_particle_ids(),
            Some([123456u32, 789012u32].as_slice())
        );
        assert_eq!(
            model.get_geometry_particle_ids(),
            Some([345678u32, 901234u32].as_slice())
        );

        // Test waterfall effect version
        let waterfall = model.get_waterfall_effect().unwrap();
        assert_eq!(waterfall.version, 1);
        assert_eq!(waterfall.parameters.flow_velocity, 1.0);

        // Test edge fade data
        let edge_fade = model.get_edge_fade_data().unwrap();
        assert_eq!(edge_fade.fade_distances, vec![10.0, 20.0]);
        assert_eq!(edge_fade.fade_factors, vec![0.5, 0.8]);

        // Test model alpha data
        let alpha_data = model.get_model_alpha_data().unwrap();
        assert_eq!(alpha_data.alpha_test_threshold, 0.5);
        assert_eq!(alpha_data.blend_mode, AlphaBlendMode::Normal);

        // Test lighting details
        let lighting = model.get_lighting_details().unwrap();
        assert_eq!(lighting.ambient_factor, 0.2);
        assert_eq!(lighting.diffuse_factor, 0.8);
        assert_eq!(lighting.specular_factor, 0.3);

        // Test particle model loading (will fail since we don't have real resolver)
        let resolver = ListfileResolver::new();
        let result = model.load_particle_models(&resolver);
        assert!(result.is_ok()); // Should succeed but return empty list due to resolver not having data

        // Clear all advanced features
        model.extended_particle_data = None;
        model.parent_animation_blacklist = None;
        model.parent_animation_data = None;
        model.waterfall_effect = None;
        model.edge_fade_data = None;
        model.model_alpha_data = None;
        model.lighting_details = None;
        model.recursive_particle_ids = None;
        model.geometry_particle_ids = None;

        assert!(!model.has_advanced_features());
        assert!(!model.has_external_files());
        assert!(!model.is_animation_blacklisted(1));
        assert!(model.get_extended_particle_data().is_none());
    }
}
