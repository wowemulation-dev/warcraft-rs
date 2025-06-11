// mh2o.rs - Enhanced water data parsing for WotLK+ ADT files

use crate::ParserContext;
use crate::error::Result;
use crate::io_helpers::ReadLittleEndian;
use std::io::{Read, Seek, SeekFrom};

/// MH2O chunk - water data (WotLK+)
#[derive(Debug, Clone)]
pub struct Mh2oChunk {
    /// Water data for each map chunk (256 entries)
    pub chunks: Vec<Mh2oEntry>,
}

/// Water data for a single chunk
#[derive(Debug, Clone)]
pub struct Mh2oEntry {
    /// Header for this entry
    pub header: Mh2oHeader,
    /// Water instances (layers) for this chunk
    pub instances: Vec<Mh2oInstance>,
    /// Render mask for liquid rendering
    pub render_mask: Option<Mh2oRenderMask>,
}

/// MH2O header for a single map chunk
#[derive(Debug, Clone)]
pub struct Mh2oHeader {
    /// Offset to instance data, relative to the start of the MH2O chunk
    pub offset_instances: u32,
    /// Number of water layers in this chunk
    pub layer_count: u32,
    /// Offset to render mask, relative to the start of the MH2O chunk
    pub offset_render_mask: u32,
}

/// MH2O water instance (layer)
#[derive(Debug, Clone)]
pub struct Mh2oInstance {
    /// Liquid type ID
    pub liquid_type: u16,
    /// Liquid object ID
    pub liquid_object: u16,
    /// Water level (height) values
    pub level_data: WaterLevelData,
    /// Vertex data for water surface
    pub vertex_data: Option<WaterVertexData>,
    /// Attributes for this water layer
    pub attributes: Vec<u64>,
}

/// Water height information
#[derive(Debug, Clone)]
pub enum WaterLevelData {
    /// Single water level for the entire chunk
    Uniform {
        /// Minimum height of water level
        min_height: f32,
        /// Maximum height of water level
        max_height: f32,
    },
    /// Variable water heights
    Variable {
        /// Minimum height of water level
        min_height: f32,
        /// Maximum height of water level
        max_height: f32,
        /// Offset to height map, relative to MH2O chunk
        offset_height_map: u32,
        /// Height values for each vertex
        heights: Option<Vec<f32>>,
    },
}

/// Water vertex information
#[derive(Debug, Clone)]
pub struct WaterVertexData {
    /// Offset to vertex data, relative to MH2O chunk
    pub offset_vertex_data: u32,
    /// Number of vertices in x direction
    pub x_vertices: u8,
    /// Number of vertices in y direction
    pub y_vertices: u8,
    /// Actual vertex data
    pub vertices: Option<Vec<WaterVertex>>,
}

/// Individual water vertex
#[derive(Debug, Clone)]
pub struct WaterVertex {
    /// Depth at this vertex
    pub depth: f32,
    /// Flow direction and velocity
    pub flow: [u8; 2],
}

/// Render mask for water
#[derive(Debug, Clone)]
pub struct Mh2oRenderMask {
    /// The mask is an 8x8 grid of bits, stored as 8 bytes
    pub mask: [u8; 8],
}

impl Mh2oChunk {
    /// Parse a MH2O chunk from a reader
    pub(crate) fn read_full<R: Read + Seek>(
        context: &mut ParserContext<R>,
        chunk_start: u64,
        chunk_size: u32,
    ) -> Result<Self> {
        // Start position for calculating relative offsets
        let start_pos = chunk_start;

        // MH2O has up to 256 headers (one for each map chunk)
        // Each header is 3 integers (12 bytes) according to WoWDev wiki
        // Some files may have fewer headers if not all chunks have water
        let max_possible_headers = chunk_size / 12;
        let header_count = std::cmp::min(256, max_possible_headers) as usize;

        // Read the headers first (only what's available)
        let mut headers = Vec::with_capacity(256);

        // Read available headers
        for _ in 0..header_count {
            let offset_instances = context.reader.read_u32_le()?;
            let layer_count = context.reader.read_u32_le()?;
            let offset_render_mask = context.reader.read_u32_le()?;

            // Note: offset_render_mask is actually offset_attributes in newer versions
            // but we keep the old name for compatibility

            headers.push(Mh2oHeader {
                offset_instances,
                layer_count,
                offset_render_mask,
            });
        }

        // Fill remaining headers with empty entries
        for _ in header_count..256 {
            headers.push(Mh2oHeader {
                offset_instances: 0,
                layer_count: 0,
                offset_render_mask: 0,
            });
        }

        // Process each header to read its data
        let mut chunks = Vec::with_capacity(256);

        for header in headers {
            let mut instances = Vec::new();
            let mut render_mask = None;

            // Read water instances
            if header.offset_instances > 0 && header.layer_count > 0 {
                // Validate that the offset is within the chunk bounds
                if header.offset_instances >= chunk_size {
                    // Skip this header if offset is invalid
                    chunks.push(Mh2oEntry {
                        header,
                        instances: Vec::new(),
                        render_mask: None,
                    });
                    continue;
                }

                context
                    .reader
                    .seek(SeekFrom::Start(start_pos + header.offset_instances as u64))?;

                for _layer_idx in 0..header.layer_count {
                    // Try to read instance header, break on EOF
                    let instance_result = (|| -> Result<Mh2oInstance> {
                        let liquid_type = context.reader.read_u16_le()?;
                        let liquid_object = context.reader.read_u16_le()?;
                        let min_height = context.reader.read_f32_le()?;
                        let max_height = context.reader.read_f32_le()?;

                        let _x_offset = context.reader.read_u8()?;
                        let _y_offset = context.reader.read_u8()?;
                        let _width = context.reader.read_u8()?;
                        let _height = context.reader.read_u8()?;

                        let _offset_mask_2bit = context.reader.read_u32_le()?;
                        let offset_height_map = context.reader.read_u32_le()?;

                        // Vertex data
                        let mut vertex_data = None;
                        let offset_vertex_data = context.reader.read_u32_le()?;
                        if offset_vertex_data > 0 {
                            let x_vertices = context.reader.read_u8()?;
                            let y_vertices = context.reader.read_u8()?;
                            let _liquid_flags = context.reader.read_u16_le()?;

                            vertex_data = Some(WaterVertexData {
                                offset_vertex_data,
                                x_vertices,
                                y_vertices,
                                vertices: None, // Will be read later
                            });
                        } else {
                            // Skip the 4 bytes if no vertex data
                            context.reader.seek(SeekFrom::Current(4))?;
                        }

                        // Read attribute data - these are 64-bit flags since MoP
                        // In Cataclysm they were different, but we'll keep it simple
                        let attribute_count = if context.version >= crate::version::AdtVersion::MoP
                        {
                            context.reader.read_u32_le()?
                        } else {
                            0
                        };

                        let mut attributes = Vec::new();
                        for _ in 0..attribute_count {
                            let attribute = context.reader.read_u64_le()?;
                            attributes.push(attribute);
                        }

                        // Determine level data type
                        let level_data = if offset_height_map > 0 {
                            // Variable height water
                            WaterLevelData::Variable {
                                min_height,
                                max_height,
                                offset_height_map,
                                heights: None, // Will be read later
                            }
                        } else {
                            // Uniform height water
                            WaterLevelData::Uniform {
                                min_height,
                                max_height,
                            }
                        };

                        Ok(Mh2oInstance {
                            liquid_type,
                            liquid_object,
                            level_data,
                            vertex_data,
                            attributes,
                        })
                    })();

                    match instance_result {
                        Ok(instance) => {
                            instances.push(instance);
                        }
                        Err(_) => {
                            // EOF or other read error - stop reading instances
                            // This is normal for some ADT files with incomplete water data
                            break;
                        }
                    }
                }

                // Now go back and read all the variable data
                // (height maps, vertex data, etc.)
                for instance in &mut instances {
                    // Read height map data if needed
                    if let WaterLevelData::Variable {
                        offset_height_map,
                        heights,
                        ..
                    } = &mut instance.level_data
                    {
                        if *offset_height_map > 0 && *offset_height_map < chunk_size {
                            // Try to read height map data, skip on error
                            let height_result = (|| -> Result<Vec<f32>> {
                                context
                                    .reader
                                    .seek(SeekFrom::Start(start_pos + *offset_height_map as u64))?;

                                // Typically a 9x9 grid for water heights
                                let mut height_data = Vec::with_capacity(9 * 9);
                                for _ in 0..9 * 9 {
                                    let height = context.reader.read_f32_le()?;
                                    height_data.push(height);
                                }

                                Ok(height_data)
                            })();

                            if let Ok(height_data) = height_result {
                                *heights = Some(height_data);
                            }
                        }
                    }

                    // Read vertex data if needed
                    if let Some(vertex_data) = &mut instance.vertex_data {
                        if vertex_data.offset_vertex_data > 0
                            && vertex_data.offset_vertex_data < chunk_size
                        {
                            // Try to read vertex data, skip on error
                            let vertex_result = (|| -> Result<Vec<WaterVertex>> {
                                context.reader.seek(SeekFrom::Start(
                                    start_pos + vertex_data.offset_vertex_data as u64,
                                ))?;

                                let x_verts = vertex_data.x_vertices as usize;
                                let y_verts = vertex_data.y_vertices as usize;
                                let vert_count = x_verts * y_verts;

                                let mut verts = Vec::with_capacity(vert_count);
                                for _ in 0..vert_count {
                                    let depth = context.reader.read_f32_le()?;
                                    let mut flow = [0u8; 2];
                                    context.reader.read_exact(&mut flow)?;

                                    verts.push(WaterVertex { depth, flow });
                                }

                                Ok(verts)
                            })();

                            if let Ok(verts) = vertex_result {
                                vertex_data.vertices = Some(verts);
                            }
                        }
                    }
                }
            }

            // Read render mask
            if header.offset_render_mask > 0 && header.offset_render_mask < chunk_size {
                // Try to read render mask, skip on error
                let mask_result = (|| -> Result<Mh2oRenderMask> {
                    context.reader.seek(SeekFrom::Start(
                        start_pos + header.offset_render_mask as u64,
                    ))?;

                    let mut mask = [0u8; 8];
                    context.reader.read_exact(&mut mask)?;

                    Ok(Mh2oRenderMask { mask })
                })();

                if let Ok(mask) = mask_result {
                    render_mask = Some(mask);
                }
            }

            chunks.push(Mh2oEntry {
                header,
                instances,
                render_mask,
            });
        }

        Ok(Self { chunks })
    }
}
