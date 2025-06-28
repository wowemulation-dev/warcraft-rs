use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use tracing::{debug, trace, warn};

use crate::chunk::{Chunk, ChunkHeader};
use crate::error::{Result, WmoError};
use crate::types::{BoundingBox, ChunkId, Color, Vec3};
use crate::version::{WmoFeature, WmoVersion};
use crate::wmo_group_types::WmoGroupFlags;
use crate::wmo_types::*;

/// Helper trait for reading little-endian values
#[allow(dead_code)]
trait ReadLittleEndian: Read {
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_u16_le(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    fn read_u32_le(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    fn read_i16_le(&mut self) -> Result<i16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }

    fn read_i32_le(&mut self) -> Result<i32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }

    fn read_f32_le(&mut self) -> Result<f32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }
}

impl<R: Read> ReadLittleEndian for R {}

/// WMO chunk identifiers
pub mod chunks {
    use crate::types::ChunkId;

    // Root file chunks
    pub const MVER: ChunkId = ChunkId::from_str("MVER");
    pub const MOHD: ChunkId = ChunkId::from_str("MOHD");
    pub const MOTX: ChunkId = ChunkId::from_str("MOTX");
    pub const MOMT: ChunkId = ChunkId::from_str("MOMT");
    pub const MOGN: ChunkId = ChunkId::from_str("MOGN");
    pub const MOGI: ChunkId = ChunkId::from_str("MOGI");
    pub const MOSB: ChunkId = ChunkId::from_str("MOSB");
    pub const MOPV: ChunkId = ChunkId::from_str("MOPV");
    pub const MOPT: ChunkId = ChunkId::from_str("MOPT");
    pub const MOPR: ChunkId = ChunkId::from_str("MOPR");
    pub const MOVV: ChunkId = ChunkId::from_str("MOVV");
    pub const MOVB: ChunkId = ChunkId::from_str("MOVB");
    pub const MOLT: ChunkId = ChunkId::from_str("MOLT");
    pub const MODS: ChunkId = ChunkId::from_str("MODS");
    pub const MODN: ChunkId = ChunkId::from_str("MODN");
    pub const MODD: ChunkId = ChunkId::from_str("MODD");
    pub const MFOG: ChunkId = ChunkId::from_str("MFOG");
    pub const MCVP: ChunkId = ChunkId::from_str("MCVP");

    // Group file chunks
    pub const MOGP: ChunkId = ChunkId::from_str("MOGP");
    pub const MOPY: ChunkId = ChunkId::from_str("MOPY");
    pub const MOVI: ChunkId = ChunkId::from_str("MOVI");
    pub const MOVT: ChunkId = ChunkId::from_str("MOVT");
    pub const MONR: ChunkId = ChunkId::from_str("MONR");
    pub const MOTV: ChunkId = ChunkId::from_str("MOTV");
    pub const MOBA: ChunkId = ChunkId::from_str("MOBA");
    pub const MOLR: ChunkId = ChunkId::from_str("MOLR");
    pub const MODR: ChunkId = ChunkId::from_str("MODR");
    pub const MOBN: ChunkId = ChunkId::from_str("MOBN");
    pub const MOBR: ChunkId = ChunkId::from_str("MOBR");
    pub const MOCV: ChunkId = ChunkId::from_str("MOCV");
    pub const MLIQ: ChunkId = ChunkId::from_str("MLIQ");
    pub const MORI: ChunkId = ChunkId::from_str("MORI");
    pub const MORB: ChunkId = ChunkId::from_str("MORB");
}

/// WMO parser for reading WMO files
pub struct WmoParser;

impl Default for WmoParser {
    fn default() -> Self {
        Self::new()
    }
}

impl WmoParser {
    /// Create a new WMO parser
    pub fn new() -> Self {
        Self
    }

    /// Parse a WMO root file
    pub fn parse_root<R: Read + Seek>(&self, reader: &mut R) -> Result<WmoRoot> {
        // Read all chunks in the file
        let chunks = self.read_chunks(reader)?;

        // Parse version
        let version = self.parse_version(&chunks, reader)?;
        debug!("WMO version: {:?}", version);

        // Parse header
        let header = self.parse_header(&chunks, reader, version)?;
        debug!("WMO header: {:?}", header);

        // Parse textures
        let textures = self.parse_textures(&chunks, reader)?;
        debug!("Found {} textures", textures.len());

        // Parse materials
        let materials = self.parse_materials(&chunks, reader, version, header.n_materials)?;
        debug!("Found {} materials", materials.len());

        // Parse group info
        let groups = self.parse_group_info(&chunks, reader, version, header.n_groups)?;
        debug!("Found {} groups", groups.len());

        // Parse portals
        let portals = self.parse_portals(&chunks, reader, header.n_portals)?;
        debug!("Found {} portals", portals.len());

        // Parse portal references
        let portal_references = self.parse_portal_references(&chunks, reader)?;
        debug!("Found {} portal references", portal_references.len());

        // Parse visible block lists
        let visible_block_lists = self.parse_visible_block_lists(&chunks, reader)?;
        debug!("Found {} visible block lists", visible_block_lists.len());

        // Parse lights
        let lights = self.parse_lights(&chunks, reader, version, header.n_lights)?;
        debug!("Found {} lights", lights.len());

        // Parse doodad names
        let doodad_names = self.parse_doodad_names(&chunks, reader)?;
        debug!("Found {} doodad names", doodad_names.len());

        // Parse doodad definitions
        let doodad_defs = self.parse_doodad_defs(&chunks, reader, version, header.n_doodad_defs)?;
        debug!("Found {} doodad definitions", doodad_defs.len());

        // Parse doodad sets
        let doodad_sets =
            self.parse_doodad_sets(&chunks, reader, doodad_names, header.n_doodad_sets)?;
        debug!("Found {} doodad sets", doodad_sets.len());

        // Parse skybox
        let skybox = self.parse_skybox(&chunks, reader, version, &header)?;
        debug!("Skybox: {:?}", skybox);

        // Create global bounding box from all groups
        let bounding_box = self.calculate_global_bounding_box(&groups);

        Ok(WmoRoot {
            version,
            materials,
            groups,
            portals,
            portal_references,
            visible_block_lists,
            lights,
            doodad_defs,
            doodad_sets,
            bounding_box,
            textures,
            header,
            skybox,
        })
    }

    /// Read all chunks in a file
    fn read_chunks<R: Read + Seek>(&self, reader: &mut R) -> Result<HashMap<ChunkId, Chunk>> {
        let mut chunks = HashMap::new();
        let start_pos = reader.stream_position()?;
        reader.seek(SeekFrom::Start(start_pos))?;

        // Read chunks until end of file
        loop {
            match ChunkHeader::read(reader) {
                Ok(header) => {
                    trace!("Found chunk: {}, size: {}", header.id, header.size);
                    let data_pos = reader.stream_position()?;

                    chunks.insert(
                        header.id,
                        Chunk {
                            header,
                            data_position: data_pos,
                        },
                    );

                    reader.seek(SeekFrom::Current(header.size as i64))?;
                }
                Err(WmoError::UnexpectedEof) => {
                    // End of file reached
                    break;
                }
                Err(e) => return Err(e),
            }
        }

        // Reset position to start
        reader.seek(SeekFrom::Start(start_pos))?;

        Ok(chunks)
    }

    /// Parse the WMO version
    fn parse_version<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
    ) -> Result<WmoVersion> {
        let version_chunk = chunks
            .get(&chunks::MVER)
            .ok_or_else(|| WmoError::MissingRequiredChunk("MVER".to_string()))?;

        version_chunk.seek_to_data(reader)?;
        let raw_version = reader.read_u32_le()?;

        WmoVersion::from_raw(raw_version).ok_or(WmoError::InvalidVersion(raw_version))
    }

    /// Parse the WMO header
    fn parse_header<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
        _version: WmoVersion,
    ) -> Result<WmoHeader> {
        let header_chunk = chunks
            .get(&chunks::MOHD)
            .ok_or_else(|| WmoError::MissingRequiredChunk("MOHD".to_string()))?;

        header_chunk.seek_to_data(reader)?;

        // Parse basic header fields
        let n_materials = reader.read_u32_le()?;
        let n_groups = reader.read_u32_le()?;
        let n_portals = reader.read_u32_le()?;
        let n_lights = reader.read_u32_le()?;
        let n_doodad_names = reader.read_u32_le()?;
        let n_doodad_defs = reader.read_u32_le()?;
        let n_doodad_sets = reader.read_u32_le()?;
        let color_bytes = reader.read_u32_le()?;
        let flags = WmoFlags::from_bits_truncate(reader.read_u32_le()?);

        // Skip some fields (depending on version)
        reader.seek(SeekFrom::Current(8))?; // Skip bounding box - we'll calculate this from groups

        // Create color from bytes
        let ambient_color = Color {
            r: ((color_bytes >> 16) & 0xFF) as u8,
            g: ((color_bytes >> 8) & 0xFF) as u8,
            b: (color_bytes & 0xFF) as u8,
            a: ((color_bytes >> 24) & 0xFF) as u8,
        };

        Ok(WmoHeader {
            n_materials,
            n_groups,
            n_portals,
            n_lights,
            n_doodad_names,
            n_doodad_defs,
            n_doodad_sets,
            flags,
            ambient_color,
        })
    }

    /// Parse texture filenames
    fn parse_textures<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
    ) -> Result<Vec<String>> {
        let motx_chunk = match chunks.get(&chunks::MOTX) {
            Some(chunk) => chunk,
            None => return Ok(Vec::new()), // No textures
        };

        let motx_data = motx_chunk.read_data(reader)?;
        let mut textures = Vec::new();

        // MOTX chunk is a list of null-terminated strings
        let mut current_string = String::new();

        for &byte in &motx_data {
            if byte == 0 {
                // End of string
                if !current_string.is_empty() {
                    textures.push(current_string);
                    current_string = String::new();
                }
            } else {
                // Add to current string
                current_string.push(byte as char);
            }
        }

        Ok(textures)
    }

    /// Parse materials
    fn parse_materials<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
        version: WmoVersion,
        n_materials: u32,
    ) -> Result<Vec<WmoMaterial>> {
        let momt_chunk = match chunks.get(&chunks::MOMT) {
            Some(chunk) => chunk,
            None => return Ok(Vec::new()), // No materials
        };

        momt_chunk.seek_to_data(reader)?;
        let mut materials = Vec::with_capacity(n_materials as usize);

        // Material size depends on version
        let material_size = if version >= WmoVersion::Mop { 64 } else { 40 };

        for _ in 0..n_materials {
            let flags = WmoMaterialFlags::from_bits_truncate(reader.read_u32_le()?);
            let shader = reader.read_u32_le()?;
            let blend_mode = reader.read_u32_le()?;
            let texture1 = reader.read_u32_le()?;

            let emissive_color = Color {
                r: reader.read_u8()?,
                g: reader.read_u8()?,
                b: reader.read_u8()?,
                a: reader.read_u8()?,
            };

            let sidn_color = Color {
                r: reader.read_u8()?,
                g: reader.read_u8()?,
                b: reader.read_u8()?,
                a: reader.read_u8()?,
            };

            let framebuffer_blend = Color {
                r: reader.read_u8()?,
                g: reader.read_u8()?,
                b: reader.read_u8()?,
                a: reader.read_u8()?,
            };

            let texture2 = reader.read_u32_le()?;

            let diffuse_color = Color {
                r: reader.read_u8()?,
                g: reader.read_u8()?,
                b: reader.read_u8()?,
                a: reader.read_u8()?,
            };

            let ground_type = reader.read_u32_le()?;

            // Skip remaining fields depending on version
            let remaining_size = material_size - 40;
            if remaining_size > 0 {
                reader.seek(SeekFrom::Current(remaining_size as i64))?;
            }

            materials.push(WmoMaterial {
                flags,
                shader,
                blend_mode,
                texture1,
                emissive_color,
                sidn_color,
                framebuffer_blend,
                texture2,
                diffuse_color,
                ground_type,
            });
        }

        Ok(materials)
    }

    /// Parse group info
    fn parse_group_info<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
        _version: WmoVersion,
        n_groups: u32,
    ) -> Result<Vec<WmoGroupInfo>> {
        // Parse group names first
        let mogn_chunk = match chunks.get(&chunks::MOGN) {
            Some(chunk) => chunk,
            None => return Ok(Vec::new()), // No group names
        };

        let group_names_data = mogn_chunk.read_data(reader)?;

        // Now parse group info
        let mogi_chunk = match chunks.get(&chunks::MOGI) {
            Some(chunk) => chunk,
            None => return Ok(Vec::new()), // No group info
        };

        mogi_chunk.seek_to_data(reader)?;
        let mut groups = Vec::with_capacity(n_groups as usize);

        for i in 0..n_groups {
            let flags = WmoGroupFlags::from_bits_truncate(reader.read_u32_le()?);

            let min_x = reader.read_f32_le()?;
            let min_y = reader.read_f32_le()?;
            let min_z = reader.read_f32_le()?;
            let max_x = reader.read_f32_le()?;
            let max_y = reader.read_f32_le()?;
            let max_z = reader.read_f32_le()?;

            let name_offset = reader.read_u32_le()?;

            // Get group name from offset
            let name = if name_offset < group_names_data.len() as u32 {
                let name = self.get_string_at_offset(&group_names_data, name_offset as usize);
                if name.is_empty() {
                    format!("Group_{i}")
                } else {
                    name
                }
            } else {
                format!("Group_{i}")
            };

            groups.push(WmoGroupInfo {
                flags,
                bounding_box: BoundingBox {
                    min: Vec3 {
                        x: min_x,
                        y: min_y,
                        z: min_z,
                    },
                    max: Vec3 {
                        x: max_x,
                        y: max_y,
                        z: max_z,
                    },
                },
                name,
            });
        }

        Ok(groups)
    }

    /// Parse portals
    fn parse_portals<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
        n_portals: u32,
    ) -> Result<Vec<WmoPortal>> {
        // Parse portal vertices first
        let mopv_chunk = match chunks.get(&chunks::MOPV) {
            Some(chunk) => chunk,
            None => return Ok(Vec::new()), // No portal vertices
        };

        let mopv_data = mopv_chunk.read_data(reader)?;
        let n_vertices = mopv_data.len() / 12; // 3 floats per vertex (x, y, z)
        let mut portal_vertices = Vec::with_capacity(n_vertices);

        for i in 0..n_vertices {
            let offset = i * 12;

            // Use byteorder to read from the data buffer
            let x = f32::from_le_bytes([
                mopv_data[offset],
                mopv_data[offset + 1],
                mopv_data[offset + 2],
                mopv_data[offset + 3],
            ]);

            let y = f32::from_le_bytes([
                mopv_data[offset + 4],
                mopv_data[offset + 5],
                mopv_data[offset + 6],
                mopv_data[offset + 7],
            ]);

            let z = f32::from_le_bytes([
                mopv_data[offset + 8],
                mopv_data[offset + 9],
                mopv_data[offset + 10],
                mopv_data[offset + 11],
            ]);

            portal_vertices.push(Vec3 { x, y, z });
        }

        // Now parse portal data
        let mopt_chunk = match chunks.get(&chunks::MOPT) {
            Some(chunk) => chunk,
            None => return Ok(Vec::new()), // No portal data
        };

        mopt_chunk.seek_to_data(reader)?;
        let mut portals = Vec::with_capacity(n_portals as usize);

        for _ in 0..n_portals {
            let vertex_index = reader.read_u16_le()? as usize;
            let n_vertices = reader.read_u16_le()? as usize;

            let normal_x = reader.read_f32_le()?;
            let normal_y = reader.read_f32_le()?;
            let normal_z = reader.read_f32_le()?;

            // Skip plane distance
            reader.seek(SeekFrom::Current(4))?;

            // Get portal vertices
            let mut vertices = Vec::with_capacity(n_vertices);
            for i in 0..n_vertices {
                let vertex_idx = vertex_index + i;
                if vertex_idx < portal_vertices.len() {
                    vertices.push(portal_vertices[vertex_idx]);
                } else {
                    warn!("Portal vertex index out of bounds: {}", vertex_idx);
                }
            }

            portals.push(WmoPortal {
                vertices,
                normal: Vec3 {
                    x: normal_x,
                    y: normal_y,
                    z: normal_z,
                },
            });
        }

        Ok(portals)
    }

    /// Parse portal references
    fn parse_portal_references<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
    ) -> Result<Vec<WmoPortalReference>> {
        let mopr_chunk = match chunks.get(&chunks::MOPR) {
            Some(chunk) => chunk,
            None => return Ok(Vec::new()), // No portal references
        };

        let mopr_data = mopr_chunk.read_data(reader)?;
        let n_refs = mopr_data.len() / 8; // 4 u16 values per reference
        let mut refs = Vec::with_capacity(n_refs);

        for i in 0..n_refs {
            let offset = i * 8;

            let portal_index = u16::from_le_bytes([mopr_data[offset], mopr_data[offset + 1]]);

            let group_index = u16::from_le_bytes([mopr_data[offset + 2], mopr_data[offset + 3]]);

            let side = u16::from_le_bytes([mopr_data[offset + 4], mopr_data[offset + 5]]);

            // Skip unused field

            refs.push(WmoPortalReference {
                portal_index,
                group_index,
                side,
            });
        }

        Ok(refs)
    }

    /// Parse visible block lists
    fn parse_visible_block_lists<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
    ) -> Result<Vec<Vec<u16>>> {
        // First get the visible block offsets
        let movv_chunk = match chunks.get(&chunks::MOVV) {
            Some(chunk) => chunk,
            None => return Ok(Vec::new()), // No visible blocks
        };

        let movv_data = movv_chunk.read_data(reader)?;
        let n_entries = movv_data.len() / 4; // u32 offset per entry
        let mut offsets = Vec::with_capacity(n_entries);

        for i in 0..n_entries {
            let offset = u32::from_le_bytes([
                movv_data[i * 4],
                movv_data[i * 4 + 1],
                movv_data[i * 4 + 2],
                movv_data[i * 4 + 3],
            ]);

            offsets.push(offset);
        }

        // Now get the visible block data
        let movb_chunk = match chunks.get(&chunks::MOVB) {
            Some(chunk) => chunk,
            None => return Ok(Vec::new()), // No visible block data
        };

        let movb_data = movb_chunk.read_data(reader)?;
        let mut visible_lists = Vec::with_capacity(offsets.len());

        for &offset in &offsets {
            let mut index = offset as usize;
            let mut list = Vec::new();

            // Read until we hit a 0xFFFF marker or end of data
            while index + 1 < movb_data.len() {
                let value = u16::from_le_bytes([movb_data[index], movb_data[index + 1]]);

                if value == 0xFFFF {
                    // End of list marker
                    break;
                }

                list.push(value);
                index += 2;
            }

            visible_lists.push(list);
        }

        Ok(visible_lists)
    }

    /// Parse lights
    fn parse_lights<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
        _version: WmoVersion,
        n_lights: u32,
    ) -> Result<Vec<WmoLight>> {
        let molt_chunk = match chunks.get(&chunks::MOLT) {
            Some(chunk) => chunk,
            None => return Ok(Vec::new()), // No lights
        };

        molt_chunk.seek_to_data(reader)?;
        let mut lights = Vec::with_capacity(n_lights as usize);

        for _ in 0..n_lights {
            let light_type_raw = reader.read_u8()?;
            let light_type = WmoLightType::from_raw(light_type_raw).ok_or_else(|| {
                WmoError::InvalidFormat(format!("Invalid light type: {light_type_raw}"))
            })?;

            // Read 3 flag bytes
            let use_attenuation = reader.read_u8()? != 0;
            let _use_unknown1 = reader.read_u8()?; // Unknown flag
            let _use_unknown2 = reader.read_u8()?; // Unknown flag

            // Read BGRA color
            let color_bytes = reader.read_u32_le()?;
            let color = Color {
                b: (color_bytes & 0xFF) as u8,
                g: ((color_bytes >> 8) & 0xFF) as u8,
                r: ((color_bytes >> 16) & 0xFF) as u8,
                a: ((color_bytes >> 24) & 0xFF) as u8,
            };

            let pos_x = reader.read_f32_le()?;
            let pos_y = reader.read_f32_le()?;
            let pos_z = reader.read_f32_le()?;

            let position = Vec3 {
                x: pos_x,
                y: pos_y,
                z: pos_z,
            };

            let intensity = reader.read_f32_le()?;

            let attenuation_start = reader.read_f32_le()?;
            let attenuation_end = reader.read_f32_le()?;

            // Skip the remaining 16 bytes (unknown radius values)
            // These might be used for spot/directional lights but we'll keep it simple for now
            reader.seek(SeekFrom::Current(16))?;

            // For now, use simple properties without directional info
            let properties = match light_type {
                WmoLightType::Spot => WmoLightProperties::Spot {
                    direction: Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: -1.0,
                    }, // Default down
                    hotspot: 0.0,
                    falloff: 0.0,
                },
                WmoLightType::Directional => WmoLightProperties::Directional {
                    direction: Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: -1.0,
                    }, // Default down
                },
                WmoLightType::Omni => WmoLightProperties::Omni,
                WmoLightType::Ambient => WmoLightProperties::Ambient,
            };

            lights.push(WmoLight {
                light_type,
                position,
                color,
                intensity,
                attenuation_start,
                attenuation_end,
                use_attenuation,
                properties,
            });
        }

        Ok(lights)
    }

    /// Parse doodad names
    fn parse_doodad_names<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
    ) -> Result<Vec<String>> {
        let modn_chunk = match chunks.get(&chunks::MODN) {
            Some(chunk) => chunk,
            None => return Ok(Vec::new()), // No doodad names
        };

        let modn_data = modn_chunk.read_data(reader)?;
        Ok(self.parse_string_list(&modn_data))
    }

    /// Parse doodad definitions
    fn parse_doodad_defs<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
        _version: WmoVersion,
        n_doodad_defs: u32,
    ) -> Result<Vec<WmoDoodadDef>> {
        let modd_chunk = match chunks.get(&chunks::MODD) {
            Some(chunk) => chunk,
            None => return Ok(Vec::new()), // No doodad definitions
        };

        modd_chunk.seek_to_data(reader)?;

        // Calculate actual number of doodad defs based on chunk size
        // Each doodad def is 40 bytes
        let actual_doodad_count = modd_chunk.header.size / 40;
        if actual_doodad_count != n_doodad_defs {
            warn!(
                "MODD chunk size indicates {} doodads, but header says {}. Using chunk size.",
                actual_doodad_count, n_doodad_defs
            );
        }

        let mut doodads = Vec::with_capacity(actual_doodad_count as usize);

        for _ in 0..actual_doodad_count {
            let name_index_raw = reader.read_u32_le()?;
            // Only the lower 24 bits are used for the name index
            let name_offset = name_index_raw & 0x00FFFFFF;

            let pos_x = reader.read_f32_le()?;
            let pos_y = reader.read_f32_le()?;
            let pos_z = reader.read_f32_le()?;

            let quat_x = reader.read_f32_le()?;
            let quat_y = reader.read_f32_le()?;
            let quat_z = reader.read_f32_le()?;
            let quat_w = reader.read_f32_le()?;

            let scale = reader.read_f32_le()?;

            let color_bytes = reader.read_u32_le()?;
            let color = Color {
                r: ((color_bytes >> 16) & 0xFF) as u8,
                g: ((color_bytes >> 8) & 0xFF) as u8,
                b: (color_bytes & 0xFF) as u8,
                a: ((color_bytes >> 24) & 0xFF) as u8,
            };

            // The set_index field doesn't exist in Classic/TBC/WotLK
            // It might be part of the upper bits of name_index_raw or added in later versions
            let set_index = 0; // Default to 0 for now

            doodads.push(WmoDoodadDef {
                name_offset,
                position: Vec3 {
                    x: pos_x,
                    y: pos_y,
                    z: pos_z,
                },
                orientation: [quat_x, quat_y, quat_z, quat_w],
                scale,
                color,
                set_index,
            });
        }

        Ok(doodads)
    }

    /// Parse doodad sets
    fn parse_doodad_sets<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
        _doodad_names: Vec<String>,
        n_doodad_sets: u32,
    ) -> Result<Vec<WmoDoodadSet>> {
        let mods_chunk = match chunks.get(&chunks::MODS) {
            Some(chunk) => chunk,
            None => return Ok(Vec::new()), // No doodad sets
        };

        mods_chunk.seek_to_data(reader)?;
        let mut sets = Vec::with_capacity(n_doodad_sets as usize);

        for _i in 0..n_doodad_sets {
            // Read 20 bytes for the set name (including null terminator)
            let mut name_bytes = [0u8; 20];
            reader.read_exact(&mut name_bytes)?;

            // Find null terminator position
            let null_pos = name_bytes.iter().position(|&b| b == 0).unwrap_or(20);
            let name = String::from_utf8_lossy(&name_bytes[0..null_pos]).to_string();

            let start_doodad = reader.read_u32_le()?;
            let n_doodads = reader.read_u32_le()?;

            // Skip unused field
            reader.seek(SeekFrom::Current(4))?;

            sets.push(WmoDoodadSet {
                name,
                start_doodad,
                n_doodads,
            });
        }

        Ok(sets)
    }

    /// Parse skybox model path (if present)
    fn parse_skybox<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
        version: WmoVersion,
        header: &WmoHeader,
    ) -> Result<Option<String>> {
        // Skybox was introduced in WotLK
        if !version.supports_feature(WmoFeature::SkyboxReferences) {
            return Ok(None);
        }

        // Check if this WMO has a skybox
        if !header.flags.contains(WmoFlags::HAS_SKYBOX) {
            return Ok(None);
        }

        let mosb_chunk = match chunks.get(&chunks::MOSB) {
            Some(chunk) => chunk,
            None => return Ok(None), // No skybox
        };

        let mosb_data = mosb_chunk.read_data(reader)?;

        // Find null terminator position
        let null_pos = mosb_data
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(mosb_data.len());
        let skybox_path = String::from_utf8_lossy(&mosb_data[0..null_pos]).to_string();

        if skybox_path.is_empty() {
            Ok(None)
        } else {
            Ok(Some(skybox_path))
        }
    }

    /// Parse a list of null-terminated strings from a buffer
    fn parse_string_list(&self, buffer: &[u8]) -> Vec<String> {
        let mut strings = Vec::new();
        let mut start = 0;

        for i in 0..buffer.len() {
            if buffer[i] == 0 {
                if i > start {
                    if let Ok(s) = std::str::from_utf8(&buffer[start..i]) {
                        strings.push(s.to_string());
                    }
                }
                start = i + 1;
            }
        }

        strings
    }

    /// Get string at specific offset from buffer
    fn get_string_at_offset(&self, buffer: &[u8], offset: usize) -> String {
        if offset >= buffer.len() {
            return String::new();
        }

        // Find the null terminator
        let end = buffer[offset..]
            .iter()
            .position(|&b| b == 0)
            .map(|pos| offset + pos)
            .unwrap_or(buffer.len());

        if let Ok(s) = std::str::from_utf8(&buffer[offset..end]) {
            s.to_string()
        } else {
            String::new()
        }
    }

    /// Calculate a global bounding box from all groups
    fn calculate_global_bounding_box(&self, groups: &[WmoGroupInfo]) -> BoundingBox {
        if groups.is_empty() {
            return BoundingBox {
                min: Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                max: Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            };
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut min_z = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let mut max_z = f32::MIN;

        for group in groups {
            min_x = min_x.min(group.bounding_box.min.x);
            min_y = min_y.min(group.bounding_box.min.y);
            min_z = min_z.min(group.bounding_box.min.z);
            max_x = max_x.max(group.bounding_box.max.x);
            max_y = max_y.max(group.bounding_box.max.y);
            max_z = max_z.max(group.bounding_box.max.z);
        }

        BoundingBox {
            min: Vec3 {
                x: min_x,
                y: min_y,
                z: min_z,
            },
            max: Vec3 {
                x: max_x,
                y: max_y,
                z: max_z,
            },
        }
    }
}
