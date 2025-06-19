// chunk.rs - Chunk structures and parsing for ADT files

use std::io::{Read, Seek, SeekFrom};
use std::str;

use crate::ParserContext;
use crate::error::{AdtError, Result};
use crate::io_helpers::ReadLittleEndian;
use crate::mcnk_subchunks::{McnrSubchunk, McvtSubchunk};
use crate::version::AdtVersion;

/// Common chunk header structure for all chunk types
#[derive(Debug, Clone)]
pub struct ChunkHeader {
    /// Magic signature - 4 bytes identifying the chunk type
    pub magic: [u8; 4],
    /// Size of the chunk data in bytes (not including this header)
    pub size: u32,
}

impl ChunkHeader {
    /// Read a chunk header from a reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut magic = [0; 4];
        if reader.read_exact(&mut magic).is_err() {
            return Err(AdtError::UnexpectedEof);
        }

        // WoW files store magic bytes in reverse order, so we need to reverse them
        magic.reverse();

        let size = reader.read_u32_le()?;

        Ok(Self { magic, size })
    }

    /// Convert the magic bytes to a string
    pub fn magic_as_string(&self) -> String {
        str::from_utf8(&self.magic).unwrap_or("????").to_string()
    }

    /// Check if the magic matches the expected value
    pub fn expect_magic(&self, expected: &[u8; 4]) -> Result<()> {
        if self.magic != *expected {
            return Err(AdtError::InvalidMagic {
                expected: str::from_utf8(expected).unwrap_or("????").to_string(),
                found: self.magic_as_string(),
            });
        }
        Ok(())
    }
}

/// MVER chunk - file version information
#[derive(Debug, Clone)]
pub struct MverChunk {
    /// Version number (usually 18)
    pub version: u32,
}

impl MverChunk {
    /// Read a MVER chunk
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let header = ChunkHeader::read(reader)?;
        Self::read_with_header(
            header,
            &mut ParserContext {
                reader,
                version: AdtVersion::Vanilla, // Default, will be updated
                position: 0,
            },
        )
    }

    /// Read a MVER chunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MVER")?;

        if header.size != 4 {
            return Err(AdtError::InvalidChunkSize {
                chunk: "MVER".to_string(),
                size: header.size,
                expected: 4,
            });
        }

        let version = context.reader.read_u32_le()?;

        Ok(Self { version })
    }
}

/// MHDR chunk - header containing offsets to other chunks
#[derive(Debug, Clone, Default)]
pub struct MhdrChunk {
    /// Flags
    pub flags: u32,
    /// Offset to MCIN chunk
    pub mcin_offset: u32,
    /// Offset to MTEX chunk
    pub mtex_offset: u32,
    /// Offset to MMDX chunk
    pub mmdx_offset: u32,
    /// Offset to MMID chunk
    pub mmid_offset: u32,
    /// Offset to MWMO chunk
    pub mwmo_offset: u32,
    /// Offset to MWID chunk
    pub mwid_offset: u32,
    /// Offset to MDDF chunk
    pub mddf_offset: u32,
    /// Offset to MODF chunk
    pub modf_offset: u32,
    /// Offset to MFBO chunk (TBC+)
    pub mfbo_offset: Option<u32>,
    /// Offset to MH2O chunk (WotLK+)
    pub mh2o_offset: Option<u32>,
    /// Offset to MTFX chunk (Cataclysm+)
    pub mtfx_offset: Option<u32>,
}

impl MhdrChunk {
    /// Read an MHDR chunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MHDR")?;

        let flags = context.reader.read_u32_le()?;
        let mcin_offset = context.reader.read_u32_le()?;
        let mtex_offset = context.reader.read_u32_le()?;
        let mmdx_offset = context.reader.read_u32_le()?;
        let mmid_offset = context.reader.read_u32_le()?;
        let mwmo_offset = context.reader.read_u32_le()?;
        let mwid_offset = context.reader.read_u32_le()?;
        let mddf_offset = context.reader.read_u32_le()?;
        let modf_offset = context.reader.read_u32_le()?;

        // Version-specific fields
        let mut mfbo_offset = None;
        let mut mh2o_offset = None;
        let mut mtfx_offset = None;

        // Determine which version-specific fields to read based on chunk size
        // Each offset is 4 bytes, so base size is 36 (9 fields * 4 bytes)
        let base_size = 36;

        if header.size >= base_size + 4 {
            // TBC+ has MFBO offset
            mfbo_offset = Some(context.reader.read_u32_le()?);

            if header.size >= base_size + 8 {
                // WotLK+ has MH2O offset
                mh2o_offset = Some(context.reader.read_u32_le()?);

                if header.size >= base_size + 12 {
                    // Cataclysm+ has MTFX offset
                    mtfx_offset = Some(context.reader.read_u32_le()?);
                }
            }
        }

        // Skip any remaining bytes in the chunk
        let read_size = base_size
            + if mfbo_offset.is_some() { 4 } else { 0 }
            + if mh2o_offset.is_some() { 4 } else { 0 }
            + if mtfx_offset.is_some() { 4 } else { 0 };

        if header.size > read_size {
            context
                .reader
                .seek(SeekFrom::Current((header.size - read_size) as i64))?;
        }

        Ok(Self {
            flags,
            mcin_offset,
            mtex_offset,
            mmdx_offset,
            mmid_offset,
            mwmo_offset,
            mwid_offset,
            mddf_offset,
            modf_offset,
            mfbo_offset,
            mh2o_offset,
            mtfx_offset,
        })
    }
}

/// MCIN chunk - map chunk index information
#[derive(Debug, Clone)]
pub struct McinChunk {
    /// Entries for each map chunk
    pub entries: Vec<McnkEntry>,
}

/// Entry in MCIN chunk for a map chunk
#[derive(Debug, Clone)]
pub struct McnkEntry {
    /// Offset to MCNK chunk
    pub offset: u32,
    /// Size of MCNK chunk
    pub size: u32,
    /// Flags
    pub flags: u32,
    /// Layer count
    pub layer_count: u32,
}

impl McinChunk {
    /// Read an MCIN chunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MCIN")?;

        // MCIN should have 256 entries (16x16 grid)
        // Each entry is 16 bytes (4 u32 values)
        if header.size != 256 * 16 {
            return Err(AdtError::InvalidChunkSize {
                chunk: "MCIN".to_string(),
                size: header.size,
                expected: 256 * 16,
            });
        }

        let mut entries = Vec::with_capacity(256);

        for _ in 0..256 {
            let offset = context.reader.read_u32_le()?;
            let size = context.reader.read_u32_le()?;
            let flags = context.reader.read_u32_le()?;
            let layer_count = context.reader.read_u32_le()?;

            entries.push(McnkEntry {
                offset,
                size,
                flags,
                layer_count,
            });
        }

        Ok(Self { entries })
    }
}

/// MTEX chunk - texture filenames
#[derive(Debug, Clone)]
pub struct MtexChunk {
    /// List of texture filenames
    pub filenames: Vec<String>,
}

impl MtexChunk {
    /// Read an MTEX chunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MTEX")?;

        // Read the entire chunk data
        let mut data = vec![0u8; header.size as usize];
        context.reader.read_exact(&mut data)?;

        // Parse null-terminated strings
        let mut filenames = Vec::new();
        let mut start = 0;

        for i in 0..data.len() {
            if data[i] == 0 {
                if i > start {
                    // Found a null terminator, extract the string
                    if let Ok(filename) = str::from_utf8(&data[start..i]) {
                        filenames.push(filename.to_string());
                    }
                }
                start = i + 1;
            }
        }

        Ok(Self { filenames })
    }
}

/// MMDX chunk - model filenames
#[derive(Debug, Clone)]
pub struct MmdxChunk {
    /// List of model filenames
    pub filenames: Vec<String>,
}

impl MmdxChunk {
    /// Read an MMDX chunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MMDX")?;

        // Read the entire chunk data
        let mut data = vec![0u8; header.size as usize];
        context.reader.read_exact(&mut data)?;

        // Parse null-terminated strings
        let mut filenames = Vec::new();
        let mut start = 0;

        for i in 0..data.len() {
            if data[i] == 0 {
                if i > start {
                    // Found a null terminator, extract the string
                    if let Ok(filename) = str::from_utf8(&data[start..i]) {
                        filenames.push(filename.to_string());
                    }
                }
                start = i + 1;
            }
        }

        Ok(Self { filenames })
    }
}

/// MMID chunk - model indices
#[derive(Debug, Clone)]
pub struct MmidChunk {
    /// List of offsets into the MMDX chunk
    pub offsets: Vec<u32>,
}

impl MmidChunk {
    /// Read an MMID chunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MMID")?;

        // Each offset is a u32 (4 bytes)
        let count = header.size / 4;
        let mut offsets = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let offset = context.reader.read_u32_le()?;
            offsets.push(offset);
        }

        Ok(Self { offsets })
    }
}

/// MWMO chunk - WMO filenames
#[derive(Debug, Clone)]
pub struct MwmoChunk {
    /// List of WMO filenames
    pub filenames: Vec<String>,
}

impl MwmoChunk {
    /// Read an MWMO chunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MWMO")?;

        // Read the entire chunk data
        let mut data = vec![0u8; header.size as usize];
        context.reader.read_exact(&mut data)?;

        // Parse null-terminated strings
        let mut filenames = Vec::new();
        let mut start = 0;

        for i in 0..data.len() {
            if data[i] == 0 {
                if i > start {
                    // Found a null terminator, extract the string
                    if let Ok(filename) = str::from_utf8(&data[start..i]) {
                        filenames.push(filename.to_string());
                    }
                }
                start = i + 1;
            }
        }

        Ok(Self { filenames })
    }
}

/// MWID chunk - WMO indices
#[derive(Debug, Clone)]
pub struct MwidChunk {
    /// List of offsets into the MWMO chunk
    pub offsets: Vec<u32>,
}

impl MwidChunk {
    /// Read an MWID chunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MWID")?;

        // Each offset is a u32 (4 bytes)
        let count = header.size / 4;
        let mut offsets = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let offset = context.reader.read_u32_le()?;
            offsets.push(offset);
        }

        Ok(Self { offsets })
    }
}

/// MDDF chunk - doodad placement information
#[derive(Debug, Clone)]
pub struct MddfChunk {
    /// List of doodad placements
    pub doodads: Vec<DoodadPlacement>,
}

/// Doodad placement information
#[derive(Debug, Clone)]
pub struct DoodadPlacement {
    /// Index into the MMID list
    pub name_id: u32,
    /// Unique ID
    pub unique_id: u32,
    /// Position (x, y, z)
    pub position: [f32; 3],
    /// Rotation (x, y, z)
    pub rotation: [f32; 3],
    /// Scale (usually 1.0)
    pub scale: f32,
    /// Flags
    pub flags: u16,
}

impl MddfChunk {
    /// Read an MDDF chunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MDDF")?;

        // Each doodad entry is 36 bytes
        if header.size % 36 != 0 {
            return Err(AdtError::InvalidChunkSize {
                chunk: "MDDF".to_string(),
                size: header.size,
                expected: header.size - (header.size % 36),
            });
        }

        let count = header.size / 36;
        let mut doodads = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let name_id = context.reader.read_u32_le()?;
            let unique_id = context.reader.read_u32_le()?;

            let mut position = [0.0; 3];
            for item in &mut position {
                *item = context.reader.read_f32_le()?;
            }

            let mut rotation = [0.0; 3];
            for item in &mut rotation {
                *item = context.reader.read_f32_le()?;
            }

            let scale = context.reader.read_f32_le()?;
            let flags = context.reader.read_u16_le()?;

            // Skip padding (2 bytes)
            context.reader.seek(SeekFrom::Current(2))?;

            doodads.push(DoodadPlacement {
                name_id,
                unique_id,
                position,
                rotation,
                scale,
                flags,
            });
        }

        Ok(Self { doodads })
    }
}

/// MODF chunk - model placement information
#[derive(Debug, Clone)]
pub struct ModfChunk {
    /// List of model placements
    pub models: Vec<ModelPlacement>,
}

/// Model placement information
#[derive(Debug, Clone)]
pub struct ModelPlacement {
    /// Index into the MWID list
    pub name_id: u32,
    /// Unique ID
    pub unique_id: u32,
    /// Position (x, y, z)
    pub position: [f32; 3],
    /// Rotation (x, y, z)
    pub rotation: [f32; 3],
    /// Bounds minimum (x, y, z)
    pub bounds_min: [f32; 3],
    /// Bounds maximum (x, y, z)
    pub bounds_max: [f32; 3],
    /// Flags
    pub flags: u16,
    /// Doodad set
    pub doodad_set: u16,
    /// Name set
    pub name_set: u16,
    /// Padding
    pub padding: u16,
}

impl ModfChunk {
    /// Read a MODF chunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MODF")?;

        // Each model entry is 64 bytes
        if header.size % 64 != 0 {
            return Err(AdtError::InvalidChunkSize {
                chunk: "MODF".to_string(),
                size: header.size,
                expected: header.size - (header.size % 64),
            });
        }

        let count = header.size / 64;
        let mut models = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let name_id = context.reader.read_u32_le()?;
            let unique_id = context.reader.read_u32_le()?;

            let mut position = [0.0; 3];
            for item in &mut position {
                *item = context.reader.read_f32_le()?;
            }

            let mut rotation = [0.0; 3];
            for item in &mut rotation {
                *item = context.reader.read_f32_le()?;
            }

            let mut bounds_min = [0.0; 3];
            for item in &mut bounds_min {
                *item = context.reader.read_f32_le()?;
            }

            let mut bounds_max = [0.0; 3];
            for item in &mut bounds_max {
                *item = context.reader.read_f32_le()?;
            }

            let flags = context.reader.read_u16_le()?;
            let doodad_set = context.reader.read_u16_le()?;
            let name_set = context.reader.read_u16_le()?;
            let padding = context.reader.read_u16_le()?;

            models.push(ModelPlacement {
                name_id,
                unique_id,
                position,
                rotation,
                bounds_min,
                bounds_max,
                flags,
                doodad_set,
                name_set,
                padding,
            });
        }

        Ok(Self { models })
    }
}

/// MCNK chunk - map chunk data
#[derive(Debug, Clone)]
pub struct McnkChunk {
    /// Flags
    pub flags: u32,
    /// Index X
    pub ix: u32,
    /// Index Y
    pub iy: u32,
    /// Number of layers
    pub n_layers: u32,
    /// Number of doodad references
    pub n_doodad_refs: u32,
    /// Offset to MCVT (height map)
    pub mcvt_offset: u32,
    /// Offset to MCNR (normal data)
    pub mcnr_offset: u32,
    /// Offset to MCLY (texture layers)
    pub mcly_offset: u32,
    /// Offset to MCRF (doodad references)
    pub mcrf_offset: u32,
    /// Offset to MCAL (alpha maps)
    pub mcal_offset: u32,
    /// Size of alpha maps
    pub mcal_size: u32,
    /// Offset to MCSH (shadow map)
    pub mcsh_offset: u32,
    /// Size of shadow map
    pub mcsh_size: u32,
    /// Area ID
    pub area_id: u32,
    /// Number of map object references
    pub n_map_obj_refs: u32,
    /// Holes (CMaNGOS: uint32)
    pub holes: u32,
    /// CMaNGOS: s1, s2 fields
    pub s1: u16,
    pub s2: u16,
    /// CMaNGOS: d1, d2, d3 fields
    pub d1: u32,
    pub d2: u32,
    pub d3: u32,
    /// CMaNGOS: predTex field
    pub pred_tex: u32,
    /// CMaNGOS: nEffectDoodad field
    pub n_effect_doodad: u32,
    /// Offset to MCSE (sound emitters)
    pub mcse_offset: u32,
    /// Number of sound emitters
    pub n_sound_emitters: u32,
    /// Offset to MCLQ (liquid data) - only used in pre-WotLK versions
    /// In WotLK+, water data is stored in the root MH2O chunk instead
    pub liquid_offset: u32,
    /// Size of liquid data
    pub liquid_size: u32,
    /// Position (x, y, z)
    pub position: [f32; 3],
    /// Offset to MCCV (vertex colors) / CMaNGOS: textureId in Vanilla
    pub mccv_offset: u32,
    /// Offset to MCLV (vertex lighting) / CMaNGOS: props in Vanilla
    pub mclv_offset: u32,
    /// CMaNGOS: additional fields
    pub texture_id: u32,
    pub props: u32,
    pub effect_id: u32,

    /// Height map (145 vertices, 9x9 grid + extra control points, stored as f32)
    pub height_map: Vec<f32>,
    /// Normal data (145 vertices, 9x9 grid + extra control points, stored as [u8; 3])
    pub normals: Vec<[u8; 3]>,
    /// Texture layers
    pub texture_layers: Vec<McnkTextureLayer>,
    /// Doodad references (indices into MMID)
    pub doodad_refs: Vec<u32>,
    /// Map object references (indices into MWID)
    pub map_obj_refs: Vec<u32>,
    /// Alpha maps (texture blending)
    pub alpha_maps: Vec<Vec<u8>>,
    /// Legacy liquid data (pre-WotLK)
    pub mclq: Option<crate::mcnk_subchunks::MclqSubchunk>,
}

/// MCNK texture layer information
#[derive(Debug, Clone)]
pub struct McnkTextureLayer {
    /// Texture ID (index into MTEX)
    pub texture_id: u32,
    /// Flags
    pub flags: u32,
    /// Offset to alpha map for this layer in MCAL
    pub alpha_map_offset: u32,
    /// Effect ID
    pub effect_id: u32,
}

impl McnkChunk {
    /// Read a MCNK chunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MCNK")?;

        // The MCNK chunk was found at some offset in the file
        // We need to know where the MCNK chunk started to calculate subchunk positions
        // Since we've already read the 8-byte header, we're 8 bytes past the chunk start
        let chunk_data_start = context.reader.stream_position()?;
        let chunk_start = chunk_data_start - 8;

        // Check if we have enough data for the MCNK header
        // Based on CMaNGOS reference, the MCNK header is 128 bytes in all versions
        if header.size < 128 {
            return Err(AdtError::InvalidChunkSize {
                chunk: "MCNK".to_string(),
                size: header.size,
                expected: 128,
            });
        }

        // Read the MCNK header fields
        let flags = context.reader.read_u32_le()?;
        let ix = context.reader.read_u32_le()?;
        let iy = context.reader.read_u32_le()?;
        let n_layers = context.reader.read_u32_le()?;
        let n_doodad_refs = context.reader.read_u32_le()?;
        let mcvt_offset = context.reader.read_u32_le()?;
        let mcnr_offset = context.reader.read_u32_le()?;
        let mcly_offset = context.reader.read_u32_le()?;
        let mcrf_offset = context.reader.read_u32_le()?;
        let mcal_offset = context.reader.read_u32_le()?;
        let mcal_size = context.reader.read_u32_le()?;
        let mcsh_offset = context.reader.read_u32_le()?;
        let mcsh_size = context.reader.read_u32_le()?;
        let area_id = context.reader.read_u32_le()?;
        let n_map_obj_refs = context.reader.read_u32_le()?;
        let holes = context.reader.read_u32_le()?; // CMaNGOS: uint32, not uint16
        let s1 = context.reader.read_u16_le()?; // CMaNGOS: s1
        let s2 = context.reader.read_u16_le()?; // CMaNGOS: s2
        let d1 = context.reader.read_u32_le()?; // CMaNGOS: d1
        let d2 = context.reader.read_u32_le()?; // CMaNGOS: d2
        let d3 = context.reader.read_u32_le()?; // CMaNGOS: d3
        let pred_tex = context.reader.read_u32_le()?; // CMaNGOS: predTex
        let n_effect_doodad = context.reader.read_u32_le()?; // CMaNGOS: nEffectDoodad

        let mcse_offset = context.reader.read_u32_le()?; // CMaNGOS: ofsSndEmitters
        let n_sound_emitters = context.reader.read_u32_le()?; // CMaNGOS: nSndEmitters
        let liquid_offset = context.reader.read_u32_le()?; // CMaNGOS: ofsLiquid
        let liquid_size = context.reader.read_u32_le()?; // CMaNGOS: sizeLiquid

        // CMaNGOS: position fields (zpos, xpos, ypos)
        let z_pos = context.reader.read_f32_le()?;
        let x_pos = context.reader.read_f32_le()?;
        let y_pos = context.reader.read_f32_le()?;
        let position = [x_pos, y_pos, z_pos]; // Convert to our format

        // CMaNGOS: additional fields at the end
        let texture_id = context.reader.read_u32_le()?; // CMaNGOS: textureId
        let props = context.reader.read_u32_le()?; // CMaNGOS: props
        let effect_id = context.reader.read_u32_le()?; // CMaNGOS: effectId

        // For compatibility, set these to 0 since they don't exist in Vanilla
        let mccv_offset = texture_id; // Reuse texture_id as mccv_offset for later versions
        let mclv_offset = 0;

        // Initialize collections for subchunks
        let mut height_map = Vec::new();
        let mut normals = Vec::new();
        let mut texture_layers = Vec::new();
        let mut doodad_refs = Vec::new();
        let mut map_obj_refs = Vec::new();
        let mut alpha_maps = Vec::new();

        // Read MCVT (height map) - with bounds checking
        if mcvt_offset > 0 {
            let mcvt_abs_pos = chunk_start + mcvt_offset as u64;
            let chunk_end = chunk_start + 8 + header.size as u64;

            // Check if MCVT position is within chunk bounds with sufficient space for header
            if mcvt_abs_pos + 8 <= chunk_end {
                match context.reader.seek(SeekFrom::Start(mcvt_abs_pos)) {
                    Ok(_) => {
                        // Check for MCVT header
                        match ChunkHeader::read(context.reader) {
                            Ok(subheader) => {
                                if subheader.magic == *b"MCVT" {
                                    // Check if the entire MCVT chunk is within bounds
                                    if mcvt_abs_pos + 8 + subheader.size as u64 <= chunk_end {
                                        // Use the proper subchunk reader
                                        match McvtSubchunk::read_with_header(subheader, context) {
                                            Ok(mcvt) => {
                                                height_map = mcvt.heights.to_vec();
                                            }
                                            Err(_) => {
                                                // Silently skip malformed MCVT
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                // Silently skip if we can't read the header
                            }
                        }
                    }
                    Err(_) => {
                        // Silently skip if we can't seek to position
                    }
                }
            }
        }

        // Read MCNR (normals) - with bounds checking
        if mcnr_offset > 0 {
            let mcnr_abs_pos = chunk_start + mcnr_offset as u64;
            let chunk_end = chunk_start + 8 + header.size as u64;

            // Check if MCNR position is within chunk bounds with sufficient space for header
            if mcnr_abs_pos + 8 <= chunk_end {
                match context.reader.seek(SeekFrom::Start(mcnr_abs_pos)) {
                    Ok(_) => {
                        // Check for MCNR header
                        match ChunkHeader::read(context.reader) {
                            Ok(subheader) => {
                                if subheader.magic == *b"MCNR" {
                                    // Check if the entire MCNR chunk is within bounds
                                    if mcnr_abs_pos + 8 + subheader.size as u64 <= chunk_end {
                                        // Use the proper subchunk reader that handles padding
                                        match McnrSubchunk::read_with_header(subheader, context) {
                                            Ok(mcnr) => {
                                                normals = mcnr
                                                    .normals
                                                    .iter()
                                                    .map(|&[x, y, z]| [x as u8, y as u8, z as u8])
                                                    .collect();
                                            }
                                            Err(_) => {
                                                // Silently skip malformed MCNR
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                // Silently skip if we can't read the header
                            }
                        }
                    }
                    Err(_) => {
                        // Silently skip if we can't seek to position
                    }
                }
            }
        }

        // Read MCLY (texture layers)
        if mcly_offset > 0 && n_layers > 0 {
            let mcly_abs_pos = chunk_start + mcly_offset as u64;
            let chunk_end = chunk_start + 8 + header.size as u64;

            if mcly_abs_pos + 8 <= chunk_end {
                match context.reader.seek(SeekFrom::Start(mcly_abs_pos)) {
                    Ok(_) => {
                        // Check for MCLY header
                        match ChunkHeader::read(context.reader) {
                            Ok(subheader) => {
                                if subheader.magic == *b"MCLY" {
                                    // Check if the entire MCLY chunk is within bounds
                                    let expected_size = n_layers * 16; // Each layer entry is 16 bytes
                                    if mcly_abs_pos + 8 + expected_size as u64 <= chunk_end {
                                        texture_layers.reserve(n_layers as usize);

                                        for _ in 0..n_layers {
                                            if let (
                                                Ok(texture_id),
                                                Ok(flags),
                                                Ok(alpha_map_offset),
                                                Ok(effect_id),
                                            ) = (
                                                context.reader.read_u32_le(),
                                                context.reader.read_u32_le(),
                                                context.reader.read_u32_le(),
                                                context.reader.read_u32_le(),
                                            ) {
                                                texture_layers.push(McnkTextureLayer {
                                                    texture_id,
                                                    flags,
                                                    alpha_map_offset,
                                                    effect_id,
                                                });
                                            } else {
                                                break; // Stop on read error
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                // Silently skip if we can't read the header
                            }
                        }
                    }
                    Err(_) => {
                        // Silently skip if we can't seek to position
                    }
                }
            }
        }

        // Read MCRF (doodad references)
        if mcrf_offset > 0 && n_doodad_refs > 0 {
            let mcrf_abs_pos = chunk_start + mcrf_offset as u64;
            let chunk_end = chunk_start + 8 + header.size as u64;

            if mcrf_abs_pos + 8 <= chunk_end {
                match context.reader.seek(SeekFrom::Start(mcrf_abs_pos)) {
                    Ok(_) => {
                        // Check for MCRF header
                        match ChunkHeader::read(context.reader) {
                            Ok(subheader) => {
                                if subheader.magic == *b"MCRF" {
                                    // Check if the entire MCRF chunk is within bounds
                                    let expected_size = n_doodad_refs * 4; // Each reference is a u32 (4 bytes)
                                    if mcrf_abs_pos + 8 + expected_size as u64 <= chunk_end {
                                        doodad_refs.reserve(n_doodad_refs as usize);

                                        for _ in 0..n_doodad_refs {
                                            if let Ok(doodad_ref) = context.reader.read_u32_le() {
                                                doodad_refs.push(doodad_ref);
                                            } else {
                                                break; // Stop on read error
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                // Silently skip if we can't read the header
                            }
                        }
                    }
                    Err(_) => {
                        // Silently skip if we can't seek to position
                    }
                }
            }
        }

        // Read MCRD (map object references) - comes after MCRF
        if n_map_obj_refs > 0 && mcrf_offset > 0 {
            let mcrf_abs_pos = chunk_start + mcrf_offset as u64;
            let chunk_end = chunk_start + 8 + header.size as u64;

            if mcrf_abs_pos + 8 <= chunk_end {
                match context.reader.seek(SeekFrom::Start(mcrf_abs_pos)) {
                    Ok(_) => {
                        // Skip MCRF header and data to get to MCRD
                        match ChunkHeader::read(context.reader) {
                            Ok(subheader) => {
                                if subheader.magic == *b"MCRF" {
                                    // Skip MCRF data to get to MCRD
                                    match context
                                        .reader
                                        .seek(SeekFrom::Current(subheader.size as i64))
                                    {
                                        Ok(_) => {
                                            // Now we should be at MCRD
                                            match ChunkHeader::read(context.reader) {
                                                Ok(mcrd_header) => {
                                                    if mcrd_header.magic == *b"MCRD" {
                                                        // Each reference is a u32
                                                        map_obj_refs
                                                            .reserve(n_map_obj_refs as usize);

                                                        for _ in 0..n_map_obj_refs {
                                                            if let Ok(map_obj_ref) =
                                                                context.reader.read_u32_le()
                                                            {
                                                                map_obj_refs.push(map_obj_ref);
                                                            } else {
                                                                break; // Stop on read error
                                                            }
                                                        }
                                                    }
                                                }
                                                Err(_) => {
                                                    // Silently skip if we can't read MCRD header
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            // Silently skip if we can't seek past MCRF data
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                // Silently skip if we can't read MCRF header
                            }
                        }
                    }
                    Err(_) => {
                        // Silently skip if we can't seek to MCRF position
                    }
                }
            }
        }

        // Read MCAL (alpha maps)
        if mcal_offset > 0 && mcal_size > 0 {
            let mcal_abs_pos = chunk_start + mcal_offset as u64;
            let chunk_end = chunk_start + 8 + header.size as u64;

            if mcal_abs_pos + 8 <= chunk_end {
                match context.reader.seek(SeekFrom::Start(mcal_abs_pos)) {
                    Ok(_) => {
                        // Check for MCAL header
                        match ChunkHeader::read(context.reader) {
                            Ok(subheader) => {
                                if subheader.magic == *b"MCAL" {
                                    // Check if the entire MCAL chunk is within bounds
                                    if mcal_abs_pos + 8 + subheader.size as u64 <= chunk_end {
                                        // For now, just read the whole chunk and store it
                                        let mut alpha_data = vec![0u8; subheader.size as usize];
                                        match context.reader.read_exact(&mut alpha_data) {
                                            Ok(_) => {
                                                // Process alpha maps based on texture layers
                                                // The first layer doesn't have an alpha map
                                                for layer in texture_layers.iter().skip(1) {
                                                    // Compute the size of this alpha map
                                                    // For now, assume 64x64 = 4096 bytes
                                                    // In reality, this depends on the flags
                                                    let alpha_size = 64 * 64;

                                                    if (layer.alpha_map_offset as usize)
                                                        < alpha_data.len()
                                                        && (layer.alpha_map_offset as usize
                                                            + alpha_size)
                                                            <= alpha_data.len()
                                                    {
                                                        let start = layer.alpha_map_offset as usize;
                                                        let end = start + alpha_size;
                                                        let map_data =
                                                            alpha_data[start..end].to_vec();
                                                        alpha_maps.push(map_data);
                                                    }
                                                }
                                            }
                                            Err(_) => {
                                                // Silently skip if we can't read alpha data
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                // Silently skip if we can't read the header
                            }
                        }
                    }
                    Err(_) => {
                        // Silently skip if we can't seek to position
                    }
                }
            }
        }

        // Read MCLQ (legacy liquid data) for pre-WotLK versions
        let mut mclq = None;
        if context.version < crate::version::AdtVersion::WotLK
            && liquid_offset > 0
            && liquid_size > 0
        {
            let mclq_abs_pos = chunk_start + liquid_offset as u64;
            let chunk_end = chunk_start + 8 + header.size as u64;

            if mclq_abs_pos + 8 <= chunk_end {
                match context.reader.seek(SeekFrom::Start(mclq_abs_pos)) {
                    Ok(_) => {
                        // Check for MCLQ header
                        match ChunkHeader::read(context.reader) {
                            Ok(subheader) => {
                                if subheader.magic == *b"MCLQ" {
                                    // Check if the entire MCLQ chunk is within bounds
                                    if mclq_abs_pos + 8 + subheader.size as u64 <= chunk_end {
                                        // Use the proper subchunk reader
                                        match crate::mcnk_subchunks::MclqSubchunk::read_with_header(
                                            subheader, context,
                                        ) {
                                            Ok(mclq_data) => {
                                                mclq = Some(mclq_data);
                                            }
                                            Err(_) => {
                                                // Silently skip malformed MCLQ
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                // Silently skip if we can't read the header
                            }
                        }
                    }
                    Err(_) => {
                        // Silently skip if we can't seek to position
                    }
                }
            }
        }

        // Try to seek to the end of this chunk
        // Some malformed chunks might have incorrect sizes, so we handle this gracefully
        let _ = context
            .reader
            .seek(SeekFrom::Start(chunk_start + 8 + header.size as u64));

        Ok(Self {
            flags,
            ix,
            iy,
            n_layers,
            n_doodad_refs,
            mcvt_offset,
            mcnr_offset,
            mcly_offset,
            mcrf_offset,
            mcal_offset,
            mcal_size,
            mcsh_offset,
            mcsh_size,
            area_id,
            n_map_obj_refs,
            holes,
            s1,
            s2,
            d1,
            d2,
            d3,
            pred_tex,
            n_effect_doodad,
            mcse_offset,
            n_sound_emitters,
            liquid_offset,
            liquid_size,
            position,
            mccv_offset,
            mclv_offset,
            texture_id,
            props,
            effect_id,
            height_map,
            normals,
            texture_layers,
            doodad_refs,
            map_obj_refs,
            alpha_maps,
            mclq,
        })
    }
}

/// MFBO chunk - flight boundaries (TBC+)
#[derive(Debug, Clone)]
pub struct MfboChunk {
    /// Maximum boundary (x, y)
    pub max: [u16; 2],
    /// Minimum boundary (x, y)
    pub min: [u16; 2],
    /// Additional data (Cataclysm+)
    pub additional_data: Vec<u8>,
}

impl MfboChunk {
    /// Read a MFBO chunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MFBO")?;

        // Minimum expected size is 8 bytes (4 u16 values)
        if header.size < 8 {
            return Err(AdtError::InvalidChunkSize {
                chunk: "MFBO".to_string(),
                size: header.size,
                expected: 8,
            });
        }

        // Read the core data (always present)
        let mut max = [0; 2];
        for item in &mut max {
            *item = context.reader.read_u16_le()?;
        }

        let mut min = [0; 2];
        for item in &mut min {
            *item = context.reader.read_u16_le()?;
        }

        // Read any additional data (Cataclysm+ has extended MFBO format)
        let mut additional_data = Vec::new();
        let remaining_bytes = header.size as usize - 8;
        if remaining_bytes > 0 {
            additional_data.resize(remaining_bytes, 0);
            context.reader.read_exact(&mut additional_data)?;
        }

        Ok(Self {
            max,
            min,
            additional_data,
        })
    }
}

/// MH2O chunk - water data (WotLK+)
#[derive(Debug, Clone)]
pub struct Mh2oChunk {
    /// Water data for each chunk (256 entries)
    pub chunks: Vec<Mh2oData>,
}

/// Water data for a single chunk
#[derive(Debug, Clone)]
pub struct Mh2oData {
    /// Water flags
    pub flags: u32,
    /// Minimum height of water level
    pub min_height: f32,
    /// Maximum height of water level
    pub max_height: f32,
    /// Water vertex data (if present)
    pub vertices: Option<Vec<Mh2oVertex>>,
    /// Render flags (if present)
    pub render_flags: Option<Vec<u8>>,
}

/// Water vertex data
#[derive(Debug, Clone)]
pub struct Mh2oVertex {
    /// Depth (height) at this point
    pub depth: f32,
    /// Flow direction and velocity
    pub flow: [u8; 2],
}

impl Mh2oChunk {
    /// Read a MH2O chunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MH2O")?;

        // MH2O starts with a header for each chunk (256 entries)
        // Each header is 6 integers (24 bytes)
        if header.size < 256 * 24 {
            return Err(AdtError::InvalidChunkSize {
                chunk: "MH2O".to_string(),
                size: header.size,
                expected: 256 * 24,
            });
        }

        // Read the headers first
        let start_pos = context.reader.stream_position()? as u32;
        let mut chunks = Vec::with_capacity(256);

        for _ in 0..256 {
            // Read header
            let offset_info = context.reader.read_u32_le()?;
            let layer_count = context.reader.read_u32_le()?;
            let offset_render = context.reader.read_u32_le()?;

            // Skip unused values
            context.reader.seek(SeekFrom::Current(12))?;

            // If this chunk has no water data, all values will be 0
            if offset_info == 0 && layer_count == 0 && offset_render == 0 {
                chunks.push(Mh2oData {
                    flags: 0,
                    min_height: 0.0,
                    max_height: 0.0,
                    vertices: None,
                    render_flags: None,
                });
                continue;
            }

            // Save current position
            let header_end = context.reader.stream_position()? as u32;

            // Read water instance data
            context
                .reader
                .seek(SeekFrom::Start((start_pos + offset_info) as u64))?;

            let flags = context.reader.read_u32_le()?;
            let min_height = context.reader.read_f32_le()?;
            let max_height = context.reader.read_f32_le()?;

            // Read render mask (typically a 8x8 grid of flags)
            let mut render_flags = None;
            if offset_render > 0 {
                context
                    .reader
                    .seek(SeekFrom::Start((start_pos + offset_render) as u64))?;

                let mut flags_data = vec![0u8; 8 * 8];
                context.reader.read_exact(&mut flags_data)?;
                render_flags = Some(flags_data);
            }

            // Read vertex data (if present)
            // This is more complex and would need additional parsing
            // For now, just add a placeholder
            let vertices = None;

            chunks.push(Mh2oData {
                flags,
                min_height,
                max_height,
                vertices,
                render_flags,
            });

            // Return to the next header
            context.reader.seek(SeekFrom::Start(header_end as u64))?;
        }

        // Seek to the end of this chunk
        context
            .reader
            .seek(SeekFrom::Start((start_pos + header.size) as u64))?;

        Ok(Self { chunks })
    }
}

/// MTFX chunk - texture effects (Cataclysm+)
#[derive(Debug, Clone)]
pub struct MtfxChunk {
    /// Texture effects
    pub effects: Vec<TextureEffect>,
}

/// Texture effect data
#[derive(Debug, Clone)]
pub struct TextureEffect {
    /// Effect ID
    pub effect_id: u32,
}

impl MtfxChunk {
    /// Read a MTFX chunk with an existing header
    #[allow(dead_code)]
    pub(crate) fn read_with_header<R: Read + Seek>(
        header: ChunkHeader,
        context: &mut ParserContext<R>,
    ) -> Result<Self> {
        header.expect_magic(b"MTFX")?;

        // Each effect is a u32 (4 bytes)
        let count = header.size / 4;
        let mut effects = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let effect_id = context.reader.read_u32_le()?;
            effects.push(TextureEffect { effect_id });
        }

        Ok(Self { effects })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_chunk_header_parsing() {
        // Create test data with magic "REVM" (MVER reversed) and size 4
        let data = vec![0x52, 0x45, 0x56, 0x4D, 0x04, 0x00, 0x00, 0x00];
        let mut cursor = Cursor::new(data);

        let header = ChunkHeader::read(&mut cursor).unwrap();
        assert_eq!(header.magic, [b'M', b'V', b'E', b'R']);
        assert_eq!(header.size, 4);
        assert_eq!(header.magic_as_string(), "MVER");
    }

    #[test]
    fn test_chunk_header_expect_magic() {
        let header = ChunkHeader {
            magic: [b'M', b'V', b'E', b'R'],
            size: 4,
        };

        // Should succeed with correct magic
        assert!(header.expect_magic(b"MVER").is_ok());

        // Should fail with incorrect magic
        assert!(header.expect_magic(b"MHDR").is_err());
    }

    #[test]
    fn test_mver_chunk_parsing() {
        // Create test data: magic "REVM" (reversed), size 4, version 18
        let data = vec![
            0x52, 0x45, 0x56, 0x4D, // REVM (MVER reversed)
            0x04, 0x00, 0x00, 0x00, // size = 4
            0x12, 0x00, 0x00, 0x00, // version = 18
        ];
        let mut cursor = Cursor::new(data);

        let mver = MverChunk::read(&mut cursor).unwrap();
        assert_eq!(mver.version, 18);
    }

    #[test]
    fn test_empty_adt_creation() {
        // Test that we can create basic chunk structures
        let header = ChunkHeader {
            magic: [b'M', b'V', b'E', b'R'],
            size: 4,
        };
        assert_eq!(header.magic_as_string(), "MVER");
        assert_eq!(header.size, 4);
    }

    #[test]
    fn test_version_to_string() {
        assert_eq!(AdtVersion::Vanilla.to_string(), "Vanilla (1.x)");
        assert_eq!(AdtVersion::TBC.to_string(), "The Burning Crusade (2.x)");
        assert_eq!(
            AdtVersion::WotLK.to_string(),
            "Wrath of the Lich King (3.x)"
        );
        assert_eq!(AdtVersion::Cataclysm.to_string(), "Cataclysm (4.x)");
        assert_eq!(AdtVersion::MoP.to_string(), "Mists of Pandaria (5.x)");
    }

    #[test]
    fn test_version_detection() {
        // Test version detection using static logic
        let version_vanilla = AdtVersion::detect_from_chunks(false, false, false, false);
        assert_eq!(version_vanilla, AdtVersion::Vanilla);

        let version_tbc = AdtVersion::detect_from_chunks(true, false, false, false);
        assert_eq!(version_tbc, AdtVersion::TBC);

        let version_wotlk = AdtVersion::detect_from_chunks(false, true, false, false);
        assert_eq!(version_wotlk, AdtVersion::WotLK);

        let version_cata = AdtVersion::detect_from_chunks(false, false, true, false);
        assert_eq!(version_cata, AdtVersion::Cataclysm);
    }

    #[test]
    fn test_version_comparison() {
        assert!(AdtVersion::Vanilla < AdtVersion::TBC);
        assert!(AdtVersion::TBC < AdtVersion::WotLK);
        assert!(AdtVersion::WotLK < AdtVersion::Cataclysm);
        assert!(AdtVersion::Cataclysm < AdtVersion::MoP);
        assert!(AdtVersion::MoP <= AdtVersion::MoP);
    }
}
