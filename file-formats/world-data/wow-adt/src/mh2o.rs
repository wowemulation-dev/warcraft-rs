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
    /// Attributes for liquid properties (fishable, deep water zones)
    pub attributes: Option<Mh2oAttributes>,
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
    /// Offset to attributes, relative to the start of the MH2O chunk
    pub offset_attributes: u32,
}

/// MH2O chunk attributes (8x8 bitmasks for liquid properties)
#[derive(Debug, Clone)]
pub struct Mh2oAttributes {
    /// Fishable/visibility flags (8x8 grid of bits, stored as 64-bit value)
    pub fishable: u64,
    /// Deep water/fatigue zone flags (8x8 grid of bits, stored as 64-bit value)
    pub deep: u64,
}

/// MH2O water instance (layer)
#[derive(Debug, Clone)]
pub struct Mh2oInstance {
    /// Liquid type ID
    pub liquid_type: u16,
    /// Liquid object ID
    pub liquid_object: u16,
    /// X offset within chunk (0-8)
    pub x_offset: u8,
    /// Y offset within chunk (0-8)
    pub y_offset: u8,
    /// Width in cells (not vertices)
    pub width: u8,
    /// Height in cells (not vertices)
    pub height: u8,
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
            let offset_attributes = context.reader.read_u32_le()?;

            headers.push(Mh2oHeader {
                offset_instances,
                layer_count,
                offset_attributes,
            });
        }

        // Fill remaining headers with empty entries
        for _ in header_count..256 {
            headers.push(Mh2oHeader {
                offset_instances: 0,
                layer_count: 0,
                offset_attributes: 0,
            });
        }

        // Process each header to read its data
        let mut chunks = Vec::with_capacity(256);

        for header in headers {
            let mut instances = Vec::new();
            let mut attributes = None;
            let render_mask = None;

            // Read water instances
            if header.offset_instances > 0 && header.layer_count > 0 {
                // Validate that the offset is within the chunk bounds
                if header.offset_instances >= chunk_size {
                    // Skip this header if offset is invalid
                    chunks.push(Mh2oEntry {
                        header,
                        instances: Vec::new(),
                        attributes: None,
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
                        // Read the 24-byte SMLiquidInstance structure per wowdev.wiki spec
                        let liquid_type = context.reader.read_u16_le()?; // 0x00
                        let liquid_object = context.reader.read_u16_le()?; // 0x02
                        let min_height = context.reader.read_f32_le()?; // 0x04
                        let max_height = context.reader.read_f32_le()?; // 0x08

                        let x_offset = context.reader.read_u8()?; // 0x0C
                        let y_offset = context.reader.read_u8()?; // 0x0D
                        let width = context.reader.read_u8()?; // 0x0E
                        let height = context.reader.read_u8()?; // 0x0F

                        let offset_exists_bitmap = context.reader.read_u32_le()?; // 0x10
                        let offset_vertex_data = context.reader.read_u32_le()?; // 0x14
                        // Instance header ends at 0x18 (24 bytes)

                        // Determine liquid vertex format (LVF)
                        // If liquid_object < 42, it encodes LVF directly:
                        //   0 = height + depth (has heightmap)
                        //   1 = height + UV (has heightmap)
                        //   2 = depth only (NO heightmap)
                        //   3 = height + UV + depth (has heightmap)
                        // If liquid_object >= 42, it's a LiquidObjectID (need DB lookup)
                        let has_heightmap = if liquid_object < 42 {
                            // liquid_object is LVF directly
                            liquid_object != 2 // LVF 2 (depth-only) has no heightmap
                        } else {
                            // For LiquidObjectID, assume has heightmap if vertex data present
                            // TODO: Proper DB lookup when implementing full liquid support
                            offset_vertex_data > 0
                        };

                        // Vertex data will be read later using offset_vertex_data
                        // The vertex data section contains both height map AND vertex attributes
                        let vertex_data = if offset_vertex_data > 0 {
                            Some(WaterVertexData {
                                offset_vertex_data,
                                x_vertices: width + 1, // Grid has width+1 vertices
                                y_vertices: height + 1, // Grid has height+1 vertices
                                vertices: None,        // Will be read later
                            })
                        } else {
                            None
                        };

                        // Determine level data type based on whether heightmap exists
                        let level_data = if has_heightmap && offset_vertex_data > 0 {
                            // Variable height water - heightmap in vertex data section
                            WaterLevelData::Variable {
                                min_height,
                                max_height,
                                offset_height_map: offset_vertex_data, // Height data is first in vertex section
                                heights: None,                         // Will be read later
                            }
                        } else {
                            // Uniform height water (flat surface at min_height)
                            WaterLevelData::Uniform {
                                min_height,
                                max_height,
                            }
                        };

                        Ok(Mh2oInstance {
                            liquid_type,
                            liquid_object,
                            x_offset,
                            y_offset,
                            width,
                            height,
                            level_data,
                            vertex_data,
                            attributes: Vec::new(), // Attributes are in separate chunk header section
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

                                // Calculate actual vertex count based on water dimensions
                                // IMPORTANT: width/height in MH2O are cell counts, vertices = cells + 1
                                // SPECIAL CASE: width=0 or height=0 means full chunk coverage (8 cells = 9 vertices)
                                let width_vertices = if instance.width == 0 {
                                    9
                                } else {
                                    instance.width + 1
                                };
                                let height_vertices = if instance.height == 0 {
                                    9
                                } else {
                                    instance.height + 1
                                };
                                let vertex_count =
                                    (width_vertices as usize) * (height_vertices as usize);

                                let mut height_data = Vec::with_capacity(vertex_count);
                                for _ in 0..vertex_count {
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

            // Read attributes
            if header.offset_attributes > 0 && header.offset_attributes < chunk_size {
                // Try to read attributes, skip on error
                let attr_result = (|| -> Result<Mh2oAttributes> {
                    context
                        .reader
                        .seek(SeekFrom::Start(start_pos + header.offset_attributes as u64))?;

                    let fishable = context.reader.read_u64_le()?;
                    let deep = context.reader.read_u64_le()?;

                    Ok(Mh2oAttributes { fishable, deep })
                })();

                if let Ok(attr) = attr_result {
                    attributes = Some(attr);
                }
            }

            // Note: Render mask is deprecated in WotLK+ and replaced by attributes
            // We keep this for backward compatibility with pre-WotLK data
            // In WotLK+, offset_attributes points to the new 16-byte structure

            chunks.push(Mh2oEntry {
                header,
                instances,
                attributes,
                render_mask,
            });
        }

        Ok(Self { chunks })
    }
}
