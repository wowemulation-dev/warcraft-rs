use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use tracing::{debug, trace};

use crate::chunk::{Chunk, ChunkHeader};
use crate::error::{Result, WmoError};
use crate::parser::chunks;
use crate::types::{BoundingBox, ChunkId, Color, Vec3};
use crate::version::{WmoFeature, WmoVersion};
use crate::wmo_group_types::*;

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

/// Parser for WMO group files
pub struct WmoGroupParser;

impl Default for WmoGroupParser {
    fn default() -> Self {
        Self::new()
    }
}

impl WmoGroupParser {
    /// Create a new WMO group parser
    pub fn new() -> Self {
        Self
    }

    /// Parse a WMO group file
    pub fn parse_group<R: Read + Seek>(
        &self,
        reader: &mut R,
        group_index: u32,
    ) -> Result<WmoGroup> {
        // Read all chunks in the file
        let chunks = self.read_chunks(reader)?;

        // Parse version
        let version = self.parse_version(&chunks, reader)?;
        debug!("WMO version: {:?}", version);

        // Parse group header
        let header = self.parse_group_header(&chunks, reader, version, group_index)?;
        debug!("WMO group header: {:?}", header);

        // Parse materials
        let materials = self.parse_materials(&chunks, reader)?;
        debug!("Found {} materials", materials.len());

        // Parse vertices
        let vertices = self.parse_vertices(&chunks, reader)?;
        debug!("Found {} vertices", vertices.len());

        // Parse normals
        let normals = if header.flags.contains(WmoGroupFlags::HAS_NORMALS) {
            self.parse_normals(&chunks, reader)?
        } else {
            Vec::new()
        };
        debug!("Found {} normals", normals.len());

        // Parse texture coordinates
        let tex_coords = self.parse_texture_coords(&chunks, reader)?;
        debug!("Found {} texture coordinates", tex_coords.len());

        // Parse batches
        let batches = self.parse_batches(&chunks, reader)?;
        debug!("Found {} batches", batches.len());

        // Parse indices
        let indices = self.parse_indices(&chunks, reader)?;
        debug!("Found {} indices", indices.len());

        // Parse vertex colors (if present)
        let vertex_colors = if header.flags.contains(WmoGroupFlags::HAS_VERTEX_COLORS) {
            Some(self.parse_vertex_colors(&chunks, reader)?)
        } else {
            None
        };
        debug!("Vertex colors: {}", vertex_colors.is_some());

        // Parse BSP nodes (if present)
        let bsp_nodes = self.parse_bsp_nodes(&chunks, reader)?;
        debug!("BSP nodes: {}", bsp_nodes.is_some());

        // Parse liquid data (if present)
        let liquid = if header.flags.contains(WmoGroupFlags::HAS_WATER) {
            self.parse_liquid(&chunks, reader, version)?
        } else {
            None
        };
        debug!("Liquid data: {}", liquid.is_some());

        // Parse doodad references (if present)
        let doodad_refs = if header.flags.contains(WmoGroupFlags::HAS_DOODADS) {
            Some(self.parse_doodad_refs(&chunks, reader)?)
        } else {
            None
        };
        debug!("Doodad references: {}", doodad_refs.is_some());

        Ok(WmoGroup {
            header,
            materials,
            vertices,
            normals,
            tex_coords,
            batches,
            indices,
            vertex_colors,
            bsp_nodes,
            liquid,
            doodad_refs,
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

    /// Parse the group header
    fn parse_group_header<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
        _version: WmoVersion,
        group_index: u32,
    ) -> Result<WmoGroupHeader> {
        let mogp_chunk = chunks
            .get(&chunks::MOGP)
            .ok_or_else(|| WmoError::MissingRequiredChunk("MOGP".to_string()))?;

        mogp_chunk.seek_to_data(reader)?;

        // First 4 bytes are group name offset
        let name_offset = reader.read_u32_le()?;

        // Next 4 bytes are group flags
        let flags = WmoGroupFlags::from_bits_truncate(reader.read_u32_le()?);

        // Next 24 bytes are bounding box
        let min_x = reader.read_f32_le()?;
        let min_y = reader.read_f32_le()?;
        let min_z = reader.read_f32_le()?;

        let max_x = reader.read_f32_le()?;
        let max_y = reader.read_f32_le()?;
        let max_z = reader.read_f32_le()?;

        // Skip the rest of the header (varies by version)
        reader.seek(SeekFrom::Current(8))?; // Skip 8 bytes

        Ok(WmoGroupHeader {
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
            name_offset,
            group_index,
        })
    }

    /// Parse materials used in the group
    fn parse_materials<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
    ) -> Result<Vec<u16>> {
        let moba_chunk = match chunks.get(&chunks::MOBA) {
            Some(chunk) => chunk,
            None => return Ok(Vec::new()), // No batches
        };

        let batch_data = moba_chunk.read_data(reader)?;
        let mut materials = Vec::new();

        // Each batch is 24 bytes
        let batch_count = batch_data.len() / 24;

        for i in 0..batch_count {
            let offset = i * 24 + 2; // Material ID is 2 bytes in

            let material_id = u16::from_le_bytes([batch_data[offset], batch_data[offset + 1]]);

            if !materials.contains(&material_id) {
                materials.push(material_id);
            }
        }

        Ok(materials)
    }

    /// Parse vertices
    fn parse_vertices<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
    ) -> Result<Vec<Vec3>> {
        let movt_chunk = match chunks.get(&chunks::MOVT) {
            Some(chunk) => chunk,
            None => return Ok(Vec::new()), // No vertices
        };

        let movt_data = movt_chunk.read_data(reader)?;
        let vertex_count = movt_data.len() / 12; // 12 bytes per vertex (3 floats)
        let mut vertices = Vec::with_capacity(vertex_count);

        for i in 0..vertex_count {
            let offset = i * 12;

            let x = f32::from_le_bytes([
                movt_data[offset],
                movt_data[offset + 1],
                movt_data[offset + 2],
                movt_data[offset + 3],
            ]);

            let y = f32::from_le_bytes([
                movt_data[offset + 4],
                movt_data[offset + 5],
                movt_data[offset + 6],
                movt_data[offset + 7],
            ]);

            let z = f32::from_le_bytes([
                movt_data[offset + 8],
                movt_data[offset + 9],
                movt_data[offset + 10],
                movt_data[offset + 11],
            ]);

            vertices.push(Vec3 { x, y, z });
        }

        Ok(vertices)
    }

    /// Parse normals
    fn parse_normals<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
    ) -> Result<Vec<Vec3>> {
        let monr_chunk = match chunks.get(&chunks::MONR) {
            Some(chunk) => chunk,
            None => return Ok(Vec::new()), // No normals
        };

        let monr_data = monr_chunk.read_data(reader)?;
        let normal_count = monr_data.len() / 12; // 12 bytes per normal (3 floats)
        let mut normals = Vec::with_capacity(normal_count);

        for i in 0..normal_count {
            let offset = i * 12;

            let x = f32::from_le_bytes([
                monr_data[offset],
                monr_data[offset + 1],
                monr_data[offset + 2],
                monr_data[offset + 3],
            ]);

            let y = f32::from_le_bytes([
                monr_data[offset + 4],
                monr_data[offset + 5],
                monr_data[offset + 6],
                monr_data[offset + 7],
            ]);

            let z = f32::from_le_bytes([
                monr_data[offset + 8],
                monr_data[offset + 9],
                monr_data[offset + 10],
                monr_data[offset + 11],
            ]);

            normals.push(Vec3 { x, y, z });
        }

        Ok(normals)
    }

    /// Parse texture coordinates
    fn parse_texture_coords<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
    ) -> Result<Vec<TexCoord>> {
        let motv_chunk = match chunks.get(&chunks::MOTV) {
            Some(chunk) => chunk,
            None => return Ok(Vec::new()), // No texture coordinates
        };

        let motv_data = motv_chunk.read_data(reader)?;
        let tex_coord_count = motv_data.len() / 8; // 8 bytes per texture coordinate (2 floats)
        let mut tex_coords = Vec::with_capacity(tex_coord_count);

        for i in 0..tex_coord_count {
            let offset = i * 8;

            let u = f32::from_le_bytes([
                motv_data[offset],
                motv_data[offset + 1],
                motv_data[offset + 2],
                motv_data[offset + 3],
            ]);

            let v = f32::from_le_bytes([
                motv_data[offset + 4],
                motv_data[offset + 5],
                motv_data[offset + 6],
                motv_data[offset + 7],
            ]);

            tex_coords.push(TexCoord { u, v });
        }

        Ok(tex_coords)
    }

    /// Parse batches
    fn parse_batches<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
    ) -> Result<Vec<WmoBatch>> {
        let moba_chunk = match chunks.get(&chunks::MOBA) {
            Some(chunk) => chunk,
            None => return Ok(Vec::new()), // No batches
        };

        let moba_data = moba_chunk.read_data(reader)?;
        let batch_count = moba_data.len() / 24; // 24 bytes per batch
        let mut moba_cursor = std::io::Cursor::new(moba_data);
        let mut batches = Vec::with_capacity(batch_count);

        for _ in 0..batch_count {
            let mut flags = [0u8; 10];
            moba_cursor.read_exact(&mut flags)?;
            let large_material_id = moba_cursor.read_u16_le()?;
            let start_index = moba_cursor.read_u32_le()?;
            let count = moba_cursor.read_u16_le()?;
            let start_vertex = moba_cursor.read_u16_le()?;
            let end_vertex = moba_cursor.read_u16_le()?;
            let use_large_material_id = moba_cursor.read_u8()? != 0;
            let mut material_id = moba_cursor.read_u8()? as u16;

            // TODO: Apply this check only if version >= Legion
            if use_large_material_id {
                material_id = large_material_id;
            }

            batches.push(WmoBatch {
                flags,
                material_id,
                start_index,
                count,
                start_vertex,
                end_vertex,
                use_large_material_id,
            });
        }

        Ok(batches)
    }

    /// Parse indices
    fn parse_indices<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
    ) -> Result<Vec<u16>> {
        let movi_chunk = match chunks.get(&chunks::MOVI) {
            Some(chunk) => chunk,
            None => return Ok(Vec::new()), // No indices
        };

        let movi_data = movi_chunk.read_data(reader)?;
        let index_count = movi_data.len() / 2; // 2 bytes per index
        let mut indices = Vec::with_capacity(index_count);

        for i in 0..index_count {
            let offset = i * 2;

            let index = u16::from_le_bytes([movi_data[offset], movi_data[offset + 1]]);

            indices.push(index);
        }

        Ok(indices)
    }

    /// Parse vertex colors
    fn parse_vertex_colors<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
    ) -> Result<Vec<Color>> {
        let mocv_chunk = match chunks.get(&chunks::MOCV) {
            Some(chunk) => chunk,
            None => return Ok(Vec::new()), // No vertex colors
        };

        let mocv_data = mocv_chunk.read_data(reader)?;
        let color_count = mocv_data.len() / 4; // 4 bytes per color
        let mut colors = Vec::with_capacity(color_count);

        for i in 0..color_count {
            let offset = i * 4;

            let color = Color {
                b: mocv_data[offset],
                g: mocv_data[offset + 1],
                r: mocv_data[offset + 2],
                a: mocv_data[offset + 3],
            };

            colors.push(color);
        }

        Ok(colors)
    }

    /// Parse BSP nodes
    fn parse_bsp_nodes<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
    ) -> Result<Option<Vec<WmoBspNode>>> {
        let mobn_chunk = match chunks.get(&chunks::MOBN) {
            Some(chunk) => chunk,
            None => return Ok(None), // No BSP nodes
        };

        let mobn_data = mobn_chunk.read_data(reader)?;
        let node_count = mobn_data.len() / 16; // 16 bytes per node
        let mut nodes = Vec::with_capacity(node_count);

        for i in 0..node_count {
            let offset = i * 16;

            let plane_normal_x = f32::from_le_bytes([
                mobn_data[offset],
                mobn_data[offset + 1],
                mobn_data[offset + 2],
                mobn_data[offset + 3],
            ]);

            let plane_distance = f32::from_le_bytes([
                mobn_data[offset + 4],
                mobn_data[offset + 5],
                mobn_data[offset + 6],
                mobn_data[offset + 7],
            ]);

            let child0 = i16::from_le_bytes([mobn_data[offset + 8], mobn_data[offset + 9]]);

            let child1 = i16::from_le_bytes([mobn_data[offset + 10], mobn_data[offset + 11]]);

            let first_face = u16::from_le_bytes([mobn_data[offset + 12], mobn_data[offset + 13]]);

            let num_faces = u16::from_le_bytes([mobn_data[offset + 14], mobn_data[offset + 15]]);

            // Extract plane normal components (x is stored, y and z are packed)
            let plane_flags = (plane_normal_x as u32) & 0x3;
            let normal = match plane_flags {
                0 => Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
                1 => Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
                2 => Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                },
                3 => {
                    // Custom normal
                    let x = ((plane_normal_x as u32) >> 2) as f32 / 32767.0;

                    // Extract y and z from child0 and child1 if they are flagged
                    // This is approximate since the exact encoding is complex
                    let y = if child0 < 0 { -0.5 } else { 0.5 };
                    let z = if child1 < 0 { -0.5 } else { 0.5 };

                    Vec3 { x, y, z }
                }
                _ => unreachable!(),
            };

            nodes.push(WmoBspNode {
                plane: WmoPlane {
                    normal,
                    distance: plane_distance,
                },
                children: [child0, child1],
                first_face,
                num_faces,
            });
        }

        Ok(Some(nodes))
    }

    /// Parse liquid data
    fn parse_liquid<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
        version: WmoVersion,
    ) -> Result<Option<WmoLiquid>> {
        let mliq_chunk = match chunks.get(&chunks::MLIQ) {
            Some(chunk) => chunk,
            None => return Ok(None), // No liquid data
        };

        mliq_chunk.seek_to_data(reader)?;

        // Parse liquid header
        let liquid_type = reader.read_u32_le()?;
        let flags = reader.read_u32_le()?;

        let width = reader.read_u32_le()? + 1; // Grid is 1 larger than stored value
        let height = reader.read_u32_le()? + 1;

        // Skip bounding box for now
        reader.seek(SeekFrom::Current(24))?;

        // Determine format based on version
        // In older versions, liquid data was simpler
        let vertex_count = (width * height) as usize;
        let mut vertices = Vec::with_capacity(vertex_count);

        if version >= WmoVersion::Wod && version.supports_feature(WmoFeature::LiquidV2) {
            // WoD and later - more complex liquid format
            // Read vertices and heights
            for y in 0..height {
                for x in 0..width {
                    let base_x = reader.read_f32_le()?;
                    let base_y = reader.read_f32_le()?;
                    let base_z = reader.read_f32_le()?;

                    let height = reader.read_f32_le()?;

                    vertices.push(WmoLiquidVertex {
                        position: Vec3 {
                            x: base_x + x as f32,
                            y: base_y + y as f32,
                            z: base_z,
                        },
                        height,
                    });
                }
            }

            // Read tile flags if available
            let has_tile_flags = flags & 2 != 0;
            let tile_flags = if has_tile_flags {
                let tile_count = ((width - 1) * (height - 1)) as usize;
                let mut flags = Vec::with_capacity(tile_count);

                for _ in 0..tile_count {
                    flags.push(reader.read_u8()?);
                }

                Some(flags)
            } else {
                None
            };

            Ok(Some(WmoLiquid {
                liquid_type,
                flags,
                width,
                height,
                vertices,
                tile_flags,
            }))
        } else {
            // Classic through MoP - simpler liquid format
            // Just read heights
            for y in 0..height {
                for x in 0..width {
                    let depth = reader.read_f32_le()?;

                    vertices.push(WmoLiquidVertex {
                        position: Vec3 {
                            x: x as f32,
                            y: y as f32,
                            z: 0.0, // Base height set to 0, adjusted by depth
                        },
                        height: depth,
                    });
                }
            }

            // Read tile flags
            let tile_count = ((width - 1) * (height - 1)) as usize;
            let mut tile_flags = Vec::with_capacity(tile_count);

            for _ in 0..tile_count {
                tile_flags.push(reader.read_u8()?);
            }

            Ok(Some(WmoLiquid {
                liquid_type,
                flags,
                width,
                height,
                vertices,
                tile_flags: Some(tile_flags),
            }))
        }
    }

    /// Parse doodad references
    fn parse_doodad_refs<R: Read + Seek>(
        &self,
        chunks: &HashMap<ChunkId, Chunk>,
        reader: &mut R,
    ) -> Result<Vec<u16>> {
        let modr_chunk = match chunks.get(&chunks::MODR) {
            Some(chunk) => chunk,
            None => return Ok(Vec::new()), // No doodad refs
        };

        let modr_data = modr_chunk.read_data(reader)?;
        let doodad_count = modr_data.len() / 2; // 2 bytes per doodad reference
        let mut doodad_refs = Vec::with_capacity(doodad_count);

        for i in 0..doodad_count {
            let offset = i * 2;

            let doodad_ref = u16::from_le_bytes([modr_data[offset], modr_data[offset + 1]]);

            doodad_refs.push(doodad_ref);
        }

        Ok(doodad_refs)
    }
}
