// mcnk_subchunks.rs - Detailed parsing for MCNK subchunks

use crate::ParserContext;
use crate::chunk::ChunkHeader;
use crate::error::{AdtError, Result};
use crate::io_helpers::ReadLittleEndian;
use std::io::{Read, Seek, SeekFrom};

/// MCVT subchunk - height map vertices
#[derive(Debug, Clone)]
pub struct McvtSubchunk {
    /// Height values for each vertex (145 vertices, 9x9 grid + extra control points)
    pub heights: [f32; 145],
}

impl McvtSubchunk {
    /// Parse a MCVT subchunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MCVT")?;

        // MCVT should be 145 vertices * 4 bytes each = 580 bytes
        if header.size != 580 {
            return Err(AdtError::InvalidChunkSize {
                chunk: "MCVT".to_string(),
                size: header.size,
                expected: 580,
            });
        }

        let mut heights = [0.0; 145];
        for item in &mut heights {
            *item = context.reader.read_f32_le()?;
        }

        Ok(Self { heights })
    }
}

/// MCNR subchunk - normal vectors
#[derive(Debug, Clone)]
pub struct McnrSubchunk {
    /// Normal vectors for each vertex (145 vertices, each normal is 3 bytes)
    /// The normals are stored as signed bytes (-127 to 127) and need to be normalized
    pub normals: [[i8; 3]; 145],
}

impl McnrSubchunk {
    /// Parse a MCNR subchunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MCNR")?;

        // MCNR should be 145 vertices * 3 bytes each = 435 bytes
        // There might be padding at the end to align to 4 bytes
        if header.size < 435 {
            return Err(AdtError::InvalidChunkSize {
                chunk: "MCNR".to_string(),
                size: header.size,
                expected: 435,
            });
        }

        let mut normals = [[0; 3]; 145];
        for item in &mut normals {
            for value in item.iter_mut() {
                *value = context.reader.read_i8()?;
            }
        }

        // Skip any padding
        if header.size > 435 {
            context
                .reader
                .seek(SeekFrom::Current((header.size - 435) as i64))?;
        }

        Ok(Self { normals })
    }

    /// Convert the normals to floating point values normalized to [-1, 1]
    pub fn to_float_normals(&self) -> [[f32; 3]; 145] {
        let mut result = [[0.0; 3]; 145];
        for (i, normal) in self.normals.iter().enumerate() {
            for (j, &component) in normal.iter().enumerate() {
                // Convert from signed byte (-127 to 127) to float (-1 to 1)
                result[i][j] = (component as f32) / 127.0;
            }
        }
        result
    }
}

/// MCLY subchunk - texture layer information
#[derive(Debug, Clone)]
pub struct MclySubchunk {
    /// Texture layers
    pub layers: Vec<TextureLayer>,
}

/// Texture layer information
#[derive(Debug, Clone)]
pub struct TextureLayer {
    /// Texture ID (index into MTEX)
    pub texture_id: u32,
    /// Flags
    pub flags: u32,
    /// Offset to alpha map for this layer in MCAL
    pub alpha_map_offset: u32,
    /// Effect ID
    pub effect_id: u32,
}

/// MCLY texture layer flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MclyFlags {
    /// 0x001: Animation: Rotate 45° clockwise
    Animate1 = 0x001,
    /// 0x002: Animation: Rotate 90° clockwise
    Animate2 = 0x002,
    /// 0x004: Animation: Rotate 180° clockwise
    Animate3 = 0x004,
    /// 0x008: Animation: Animate faster (wrath+)
    AnimateFaster = 0x008,
    /// 0x010: Animation: Animate fastest (wrath+)
    AnimateFastest = 0x010,
    /// 0x020: Animation: Fixed time offset
    AnimateFixedTime = 0x020,
    /// 0x040: Animation: Use animation from previous layer (cata+)
    AnimateUseOtherLayer = 0x040,
    /// 0x080: Use alpha map compressed with ADPCM
    CompressedAlpha = 0x080,
    /// 0x100: Use big alpha (64x64 instead of 32x32 for MCAL)
    UseBigAlpha = 0x100,
}

impl MclySubchunk {
    /// Parse a MCLY subchunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MCLY")?;

        // Each texture layer is 16 bytes (4 u32s)
        if header.size % 16 != 0 {
            return Err(AdtError::InvalidChunkSize {
                chunk: "MCLY".to_string(),
                size: header.size,
                expected: header.size - (header.size % 16),
            });
        }

        let layer_count = header.size / 16;
        let mut layers = Vec::with_capacity(layer_count as usize);

        for _ in 0..layer_count {
            let texture_id = context.reader.read_u32_le()?;
            let flags = context.reader.read_u32_le()?;
            let alpha_map_offset = context.reader.read_u32_le()?;
            let effect_id = context.reader.read_u32_le()?;

            layers.push(TextureLayer {
                texture_id,
                flags,
                alpha_map_offset,
                effect_id,
            });
        }

        Ok(Self { layers })
    }
}

/// MCRF subchunk - doodad references
#[derive(Debug, Clone)]
pub struct McrfSubchunk {
    /// Indices into MMID array
    pub indices: Vec<u32>,
}

impl McrfSubchunk {
    /// Parse a MCRF subchunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MCRF")?;

        // Each index is 4 bytes
        let count = header.size / 4;
        let mut indices = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let index = context.reader.read_u32_le()?;
            indices.push(index);
        }

        Ok(Self { indices })
    }
}

/// MCRD subchunk - map object references
#[derive(Debug, Clone)]
pub struct McrdSubchunk {
    /// Indices into MWID array
    pub indices: Vec<u32>,
}

impl McrdSubchunk {
    /// Parse a MCRD subchunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MCRD")?;

        // Each index is 4 bytes
        let count = header.size / 4;
        let mut indices = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let index = context.reader.read_u32_le()?;
            indices.push(index);
        }

        Ok(Self { indices })
    }
}

/// MCSH subchunk - shadow map
#[derive(Debug, Clone)]
pub struct McshSubchunk {
    /// Shadow map data (8x8 values, 1 byte per value)
    pub shadow_map: Vec<u8>,
}

impl McshSubchunk {
    /// Parse a MCSH subchunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MCSH")?;

        // Shadow map should be 8x8 values = 64 bytes
        // But there might be padding or other formats

        let mut shadow_map = vec![0u8; header.size as usize];
        context.reader.read_exact(&mut shadow_map)?;

        Ok(Self { shadow_map })
    }
}

/// MCAL subchunk - alpha maps
#[derive(Debug, Clone)]
pub struct McalSubchunk {
    /// Raw alpha map data
    pub data: Vec<u8>,
}

impl McalSubchunk {
    /// Parse a MCAL subchunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MCAL")?;

        let mut data = vec![0u8; header.size as usize];
        context.reader.read_exact(&mut data)?;

        Ok(Self { data })
    }

    /// Extract alpha maps for each layer
    ///
    /// The first layer doesn't have an alpha map.
    /// The format depends on the flags in the MCLY chunk:
    /// - If MCLY_FLAGS_COMPRESSED_ALPHA is set, the alpha map is compressed with ADPCM
    /// - If MCLY_FLAGS_USE_BIG_ALPHA is set, the alpha map is 64x64, otherwise it's 32x32
    pub fn extract_alpha_maps(&self, layers: &[TextureLayer]) -> Vec<Vec<u8>> {
        let mut result = Vec::new();

        // Skip the first layer (base layer)
        for layer in layers.iter().skip(1) {
            let offset = layer.alpha_map_offset as usize;
            let compressed = (layer.flags & MclyFlags::CompressedAlpha as u32) != 0;
            let big_alpha = (layer.flags & MclyFlags::UseBigAlpha as u32) != 0;

            // Determine size of the alpha map
            let size = if big_alpha { 64 * 64 } else { 32 * 32 };

            if offset + size <= self.data.len() {
                if compressed {
                    // ADPCM decompression would be implemented here
                    // For now, just store the compressed data
                    let map_data = self.data[offset..offset + size].to_vec();
                    result.push(map_data);
                } else {
                    // Uncompressed alpha map
                    let map_data = self.data[offset..offset + size].to_vec();
                    result.push(map_data);
                }
            }
        }

        result
    }
}

/// MCLQ subchunk - legacy liquid data (pre-WotLK)
#[derive(Debug, Clone)]
pub struct MclqSubchunk {
    /// Number of vertices in x direction
    pub x_vertices: u32,
    /// Number of vertices in y direction
    pub y_vertices: u32,
    /// Base height of the liquid
    pub base_height: f32,
    /// Liquid vertex data
    pub vertices: Vec<LiquidVertex>,
}

/// Liquid vertex data for pre-WotLK
#[derive(Debug, Clone)]
pub struct LiquidVertex {
    /// Depth of the liquid at this point
    pub depth: f32,
    /// Liquid ID
    pub liquid_id: u16,
    /// Flags
    pub flags: u16,
}

impl MclqSubchunk {
    /// Parse a MCLQ subchunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MCLQ")?;

        let x_vertices = context.reader.read_u32_le()?;
        let y_vertices = context.reader.read_u32_le()?;
        let base_height = context.reader.read_f32_le()?;

        // Calculate number of vertices with overflow check
        let vertex_count = (x_vertices as usize)
            .checked_mul(y_vertices as usize)
            .ok_or_else(|| {
                AdtError::ParseError(format!(
                    "Vertex count overflow: {} * {}",
                    x_vertices, y_vertices
                ))
            })?;

        // Sanity check - liquid layers shouldn't have more than 9x9 vertices typically
        if vertex_count > 10000 {
            return Err(AdtError::ParseError(format!(
                "Unreasonable vertex count for liquid layer: {} ({}x{})",
                vertex_count, x_vertices, y_vertices
            )));
        }

        let mut vertices = Vec::with_capacity(vertex_count);

        for _ in 0..vertex_count {
            let depth = context.reader.read_f32_le()?;
            let liquid_id = context.reader.read_u16_le()?;
            let flags = context.reader.read_u16_le()?;

            vertices.push(LiquidVertex {
                depth,
                liquid_id,
                flags,
            });
        }

        Ok(Self {
            x_vertices,
            y_vertices,
            base_height,
            vertices,
        })
    }
}

/// MCCV subchunk - vertex colors
#[derive(Debug, Clone)]
pub struct MccvSubchunk {
    /// Vertex colors (BGRA format, one per vertex)
    pub colors: Vec<[u8; 4]>,
}

impl MccvSubchunk {
    /// Parse a MCCV subchunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MCCV")?;

        // Each color is 4 bytes (BGRA)
        let count = header.size / 4;
        let mut colors = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let mut color = [0; 4];
            context.reader.read_exact(&mut color)?;
            colors.push(color);
        }

        Ok(Self { colors })
    }
}
