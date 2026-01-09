//! Simple ADT chunk structures with binrw derives
//!
//! These chunks use straightforward binrw derives without custom parsing.

use binrw::{BinRead, BinWrite};

/// MVER - Version chunk (always 18 for ADT files, Vanilla+)
///
/// All ADT versions from Vanilla through MoP use version 18.
/// Version detection uses chunk presence analysis instead.
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ✅ Introduced
/// - **TBC (2.4.3)**: ✅ Present
/// - **WotLK (3.3.5a)**: ✅ Present
/// - **Cataclysm (4.3.4)**: ✅ Present
/// - **MoP (5.4.8)**: ✅ Present
///
/// Reference: <https://wowdev.wiki/ADT/v18#MVER_chunk>
#[derive(Debug, Clone, Copy, PartialEq, Eq, BinRead, BinWrite)]
#[brw(little)]
pub struct MverChunk {
    /// Version number (always 18)
    pub version: u32,
}

impl Default for MverChunk {
    fn default() -> Self {
        Self { version: 18 }
    }
}

/// MHDR - Header chunk with offsets to all major chunks (64 bytes, Vanilla+)
///
/// Contains 16 u32 fields:
/// - flags (bits indicate which optional chunks are present)
/// - 15 chunk offset fields (relative to MHDR position)
///
/// Offsets are stored relative to the MHDR chunk location in the file.
/// A value of 0 means the chunk is not present.
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ✅ Introduced
/// - **TBC (2.4.3)**: ✅ Present
/// - **WotLK (3.3.5a)**: ✅ Present
/// - **Cataclysm (4.3.4)**: ✅ Present
/// - **MoP (5.4.8)**: ✅ Present
///
/// Reference: <https://wowdev.wiki/ADT/v18#MHDR_chunk>
#[derive(Debug, Clone, Copy, Default, BinRead, BinWrite)]
#[brw(little)]
pub struct MhdrChunk {
    /// Flags indicating optional features
    /// - 0x01: MFBO present (flight bounds)
    /// - 0x02: MH2O present (water/lava)
    pub flags: u32,

    /// Offset to MCIN chunk (chunk index)
    pub mcin_offset: u32,

    /// Offset to MTEX chunk (texture filenames)
    pub mtex_offset: u32,

    /// Offset to MMDX chunk (model filenames)
    pub mmdx_offset: u32,

    /// Offset to MMID chunk (model indices)
    pub mmid_offset: u32,

    /// Offset to MWMO chunk (WMO filenames)
    pub mwmo_offset: u32,

    /// Offset to MWID chunk (WMO indices)
    pub mwid_offset: u32,

    /// Offset to MDDF chunk (doodad placements)
    pub mddf_offset: u32,

    /// Offset to MODF chunk (WMO placements)
    pub modf_offset: u32,

    /// Offset to MFBO chunk (flight boundaries, TBC+)
    pub mfbo_offset: u32,

    /// Offset to MH2O chunk (water data, WotLK+)
    pub mh2o_offset: u32,

    /// Offset to MTXF chunk (texture flags)
    pub mtxf_offset: u32,

    /// Reserved/unused offsets (4 u32 fields for future expansion)
    pub unused1: u32,
    pub unused2: u32,
    pub unused3: u32,
    pub unused4: u32,
}

/// MCIN - MCNK chunk index entry (16 bytes per entry, Vanilla+)
///
/// Each entry provides location and metadata for one MCNK terrain chunk.
/// MCIN contains exactly 256 entries (16x16 grid).
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ✅ Introduced
/// - **TBC (2.4.3)**: ✅ Present
/// - **WotLK (3.3.5a)**: ✅ Present
/// - **Cataclysm (4.3.4)**: ❌ Removed (split files use direct MCNK access)
/// - **MoP (5.4.8)**: ❌ Not present (split file architecture)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, BinRead, BinWrite)]
#[brw(little)]
pub struct McinEntry {
    /// Absolute file offset to MCNK chunk
    pub offset: u32,

    /// Size of MCNK chunk in bytes
    pub size: u32,

    /// Flags (rarely used, usually 0)
    pub flags: u32,

    /// Async object ID (used by client for loading)
    pub async_id: u32,
}

/// MCIN - MCNK chunk index (4096 bytes = 256 entries × 16 bytes, Vanilla+)
///
/// Provides fast lookup table for all 256 terrain chunks (16×16 grid).
/// Each entry contains absolute file offset and size for one MCNK chunk.
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ✅ Introduced
/// - **TBC (2.4.3)**: ✅ Present
/// - **WotLK (3.3.5a)**: ✅ Present
/// - **Cataclysm (4.3.4)**: ❌ Removed (split files use direct MCNK access)
/// - **MoP (5.4.8)**: ❌ Not present (split file architecture)
///
/// Reference: <https://wowdev.wiki/ADT/v18#MCIN_chunk>
#[derive(Debug, Clone, BinRead, BinWrite)]
#[brw(little)]
pub struct McinChunk {
    /// 256 index entries (16x16 tile grid)
    #[br(count = 256)]
    pub entries: Vec<McinEntry>,
}

impl Default for McinChunk {
    fn default() -> Self {
        Self {
            entries: vec![McinEntry::default(); 256],
        }
    }
}

impl McinChunk {
    /// Get MCIN entry for tile at (x, y) coordinates
    ///
    /// Coordinates are 0-15 for both x and y.
    /// Returns None if coordinates are out of bounds.
    pub fn get_entry(&self, x: usize, y: usize) -> Option<&McinEntry> {
        if x >= 16 || y >= 16 {
            return None;
        }
        let index = y * 16 + x;
        self.entries.get(index)
    }

    /// Get mutable MCIN entry for tile at (x, y)
    pub fn get_entry_mut(&mut self, x: usize, y: usize) -> Option<&mut McinEntry> {
        if x >= 16 || y >= 16 {
            return None;
        }
        let index = y * 16 + x;
        self.entries.get_mut(index)
    }
}

/// MFBO - Flight boundaries for flying mounts (TBC+, 36 bytes)
///
/// Defines maximum and minimum height planes for flying mount restrictions.
/// Each plane has 9 vertices arranged in a 3x3 grid covering the ADT tile.
///
/// Heights are stored as i16 values representing game world coordinates.
/// The planes define the upper and lower bounds of the flight box.
///
/// Reference: <https://wowdev.wiki/ADT/v18#MFBO_chunk>
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ❌ Not present
/// - **TBC (2.4.3)**: ✅ Introduced (flying mounts added)
/// - **WotLK (3.3.5a)**: ✅ Present
/// - **Cataclysm (4.3.4)**: ✅ Present
/// - **MoP (5.4.8)**: ✅ Present
#[derive(Debug, Clone, Copy, Default, BinRead, BinWrite)]
#[brw(little)]
pub struct MfboChunk {
    /// Maximum height plane (9 i16 values in 3x3 grid)
    pub max_plane: [i16; 9],
    /// Minimum height plane (9 i16 values in 3x3 grid)
    pub min_plane: [i16; 9],
}

/// MAMP chunk - Texture size amplifier (Cataclysm+)
///
/// Defines a texture coordinate scaling factor for terrain textures.
/// Higher values increase texture detail density.
///
/// Reference: <https://wowdev.wiki/ADT/v18#MAMP_chunk>
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ❌ Not present
/// - **TBC (2.4.3)**: ❌ Not present
/// - **WotLK (3.3.5a)**: ❌ Not present
/// - **Cataclysm (4.3.4)**: ✅ Introduced
/// - **MoP (5.4.8)**: ✅ Present
#[derive(Debug, Clone, Copy, BinRead, BinWrite)]
#[brw(little)]
pub struct MampChunk {
    /// Texture amplification factor
    pub amplifier: u32,
}

/// MTXP - Texture height parameters (MoP+)
///
/// Defines height blending parameters for each texture in the MTEX chunk.
/// Each entry contains height scale/offset values for height-based texture blending.
/// Variable length - contains one `TextureHeightParams` per texture.
///
/// Height blending uses `_h.blp` height textures to create more natural transitions
/// between terrain texture layers based on height values rather than simple alpha blending.
///
/// Reference: <https://wowdev.wiki/ADT/v18#MTXP_chunk>
///
/// ## Height Texture Loading
///
/// If `height_scale == 0.0` and `height_offset == 1.0`, no `_h.blp` texture is loaded.
/// Otherwise, the corresponding `<texture>_h.blp` file should be loaded.
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ❌ Not present
/// - **TBC (2.4.3)**: ❌ Not present
/// - **WotLK (3.3.5a)**: ❌ Not present
/// - **Cataclysm (4.3.4)**: ❌ Not present
/// - **MoP (5.4.8)**: ✅ Introduced
#[derive(Debug, Clone, BinRead, BinWrite)]
#[brw(little)]
pub struct MtxpChunk {
    /// Height parameters for each texture (one per MTEX entry)
    #[br(parse_with = parse_texture_height_params)]
    #[bw(write_with = write_texture_height_params)]
    pub entries: Vec<TextureHeightParams>,
}

/// Height blending parameters for a single texture (SMTextureParams)
///
/// Controls how height textures (`_h.blp`) affect terrain blending.
#[derive(Debug, Clone, Copy, BinRead, BinWrite)]
#[brw(little)]
pub struct TextureHeightParams {
    /// Texture flags (same format as MTXF flags)
    pub flags: u32,
    /// Height scale factor - `_h.blp` values are scaled to [0, height_scale)
    /// Default: 0.0 (no height blending)
    pub height_scale: f32,
    /// Height offset for blending calculations
    /// Default: 1.0
    pub height_offset: f32,
    /// Reserved/padding
    pub padding: u32,
}

impl TextureHeightParams {
    /// Check if this texture should load a height texture (`_h.blp`)
    ///
    /// Returns false if height_scale == 0.0 and height_offset == 1.0 (default values)
    pub fn uses_height_texture(&self) -> bool {
        self.height_scale != 0.0 || self.height_offset != 1.0
    }
}

/// Parse texture height parameters until end of stream
#[binrw::parser(reader, endian)]
#[allow(clippy::while_let_loop)]
fn parse_texture_height_params() -> binrw::BinResult<Vec<TextureHeightParams>> {
    let mut params = Vec::new();
    loop {
        match TextureHeightParams::read_options(reader, endian, ()) {
            Ok(entry) => params.push(entry),
            Err(_) => break,
        }
    }
    Ok(params)
}

/// Write texture height parameters
#[binrw::writer(writer, endian)]
fn write_texture_height_params(params: &Vec<TextureHeightParams>) -> binrw::BinResult<()> {
    for param in params {
        param.write_options(writer, endian, ())?;
    }
    Ok(())
}

/// MTXF - Texture flags (WotLK 3.x+)
///
/// Defines rendering flags for each texture in the MTEX chunk.
/// Each flag value controls texture rendering behavior like specularity,
/// environment mapping, and animation.
///
/// Variable length - contains one u32 per texture in MTEX.
///
/// ## References
///
/// - **wowdev.wiki**: <https://wowdev.wiki/ADT/v18#MTXF_chunk>
/// - **WoWFormatLib**: `WoWFormatLib/Structs/ADT.Struct.cs:435-438` (struct MTXF)
/// - **noggit-red**: Not directly implemented (parses via MHDR offset)
/// - **wow.export**: `src/js/3D/loaders/ADTLoader.js:125` (MHDR.ofsMTXF field)
///
/// ## Structure
///
/// Each texture flag is a 32-bit bitfield:
/// - Flags control rendering properties (specularity, cubemap, animation)
/// - Corresponds to MTEX entries by index
/// - Zero value means default rendering behavior
///
/// ## Deviations
///
/// None - matches WoWFormatLib struct layout exactly.
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: Not present
/// - **TBC (2.4.3)**: Not present
/// - **WotLK (3.3.5a)**: ✅ Introduced in 3.x
/// - **Cataclysm (4.3.4)**: ✅ Present
/// - **MoP (5.4.8)**: ✅ Present
///
/// ## Example
///
/// ```no_run
/// use wow_adt::chunks::simple::MtxfChunk;
/// use binrw::BinRead;
/// use std::io::Cursor;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Parse MTXF chunk with 3 texture flags
/// let data = vec![
///     0x01, 0x00, 0x00, 0x00, // texture[0] flags = 0x01
///     0x02, 0x00, 0x00, 0x00, // texture[1] flags = 0x02
///     0x00, 0x00, 0x00, 0x00, // texture[2] flags = 0x00 (default)
/// ];
/// let mut cursor = Cursor::new(data);
/// let mtxf = MtxfChunk::read_le(&mut cursor)?;
///
/// assert_eq!(mtxf.flags.len(), 3);
/// assert_eq!(mtxf.flags[0], 0x01);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, BinRead, BinWrite)]
#[brw(little)]
pub struct MtxfChunk {
    /// Rendering flags for each texture (one per MTEX entry)
    #[br(parse_with = parse_texture_flags)]
    #[bw(write_with = write_texture_flags)]
    pub flags: Vec<u32>,
}

/// Parse texture flags until end of stream
#[binrw::parser(reader, endian)]
#[allow(clippy::while_let_loop)]
fn parse_texture_flags() -> binrw::BinResult<Vec<u32>> {
    let mut flags = Vec::new();
    loop {
        match u32::read_options(reader, endian, ()) {
            Ok(value) => flags.push(value),
            Err(_) => break,
        }
    }
    Ok(flags)
}

/// Write texture flags
#[binrw::writer(writer, endian)]
fn write_texture_flags(flags: &Vec<u32>) -> binrw::BinResult<()> {
    for flag in flags {
        flag.write_options(writer, endian, ())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_mver_default() {
        let mver = MverChunk::default();
        assert_eq!(mver.version, 18);
    }

    #[test]
    fn test_mver_round_trip() {
        let original = MverChunk { version: 18 };
        let mut buffer = Cursor::new(Vec::new());
        original.write_le(&mut buffer).unwrap();

        let data = buffer.into_inner();
        let mut cursor = Cursor::new(data);
        let parsed = MverChunk::read_le(&mut cursor).unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_mhdr_size() {
        let mhdr = MhdrChunk::default();
        let mut buffer = Cursor::new(Vec::new());
        mhdr.write_le(&mut buffer).unwrap();
        assert_eq!(buffer.position(), 64);
    }

    #[test]
    fn test_mcin_entry_count() {
        let mcin = McinChunk::default();
        assert_eq!(mcin.entries.len(), 256);
    }

    #[test]
    fn test_mcin_get_entry() {
        let mut mcin = McinChunk::default();
        mcin.entries[0].offset = 1000;
        mcin.entries[255].offset = 2000;

        assert_eq!(mcin.get_entry(0, 0).unwrap().offset, 1000);
        assert_eq!(mcin.get_entry(15, 15).unwrap().offset, 2000);
        assert!(mcin.get_entry(16, 0).is_none());
    }

    #[test]
    fn test_mfbo_chunk_size() {
        assert_eq!(std::mem::size_of::<MfboChunk>(), 36);
    }

    #[test]
    fn test_mfbo_chunk_parse() {
        // Create test data: 9 max values (100-108), 9 min values (0-8)
        let mut data = Vec::new();
        for i in 0..9 {
            data.extend_from_slice(&(100 + i as i16).to_le_bytes());
        }
        for i in 0..9 {
            data.extend_from_slice(&(i as i16).to_le_bytes());
        }

        let mut cursor = Cursor::new(data);
        let mfbo = MfboChunk::read_le(&mut cursor).unwrap();

        // Verify max plane values
        for i in 0..9 {
            assert_eq!(mfbo.max_plane[i], 100 + i as i16);
        }

        // Verify min plane values
        for i in 0..9 {
            assert_eq!(mfbo.min_plane[i], i as i16);
        }
    }

    #[test]
    fn test_mfbo_chunk_round_trip() {
        let original = MfboChunk {
            max_plane: [500, 510, 520, 530, 540, 550, 560, 570, 580],
            min_plane: [10, 20, 30, 40, 50, 60, 70, 80, 90],
        };

        let mut buffer = Cursor::new(Vec::new());
        original.write_le(&mut buffer).unwrap();
        assert_eq!(buffer.position(), 36);

        let data = buffer.into_inner();
        let mut cursor = Cursor::new(data);
        let parsed = MfboChunk::read_le(&mut cursor).unwrap();

        assert_eq!(original.max_plane, parsed.max_plane);
        assert_eq!(original.min_plane, parsed.min_plane);
    }

    #[test]
    fn test_mfbo_chunk_default() {
        let mfbo = MfboChunk::default();
        assert_eq!(mfbo.max_plane, [0; 9]);
        assert_eq!(mfbo.min_plane, [0; 9]);
    }

    #[test]
    fn test_mamp_chunk_size() {
        assert_eq!(std::mem::size_of::<MampChunk>(), 4);
    }

    #[test]
    fn test_mamp_chunk_parse() {
        let data = vec![0x02, 0x00, 0x00, 0x00]; // amplifier = 2
        let mut cursor = Cursor::new(data);
        let mamp = MampChunk::read_le(&mut cursor).unwrap();
        assert_eq!(mamp.amplifier, 2);
    }

    #[test]
    fn test_mtxp_chunk_parse() {
        // 2 texture entries (16 bytes each = 32 bytes total)
        // Entry format: flags (u32), height_scale (f32), height_offset (f32), padding (u32)
        let data = vec![
            // Entry 0: flags=1, height_scale=0.5, height_offset=1.0, padding=0
            0x01, 0x00, 0x00, 0x00, // flags = 1
            0x00, 0x00, 0x00, 0x3F, // height_scale = 0.5 (0x3F000000)
            0x00, 0x00, 0x80, 0x3F, // height_offset = 1.0 (0x3F800000)
            0x00, 0x00, 0x00, 0x00, // padding = 0
            // Entry 1: flags=2, height_scale=1.0, height_offset=0.5, padding=0
            0x02, 0x00, 0x00, 0x00, // flags = 2
            0x00, 0x00, 0x80, 0x3F, // height_scale = 1.0 (0x3F800000)
            0x00, 0x00, 0x00, 0x3F, // height_offset = 0.5 (0x3F000000)
            0x00, 0x00, 0x00, 0x00, // padding = 0
        ];
        let mut cursor = Cursor::new(data);
        let mtxp = MtxpChunk::read_le(&mut cursor).unwrap();
        assert_eq!(mtxp.entries.len(), 2);
        assert_eq!(mtxp.entries[0].flags, 1);
        assert!((mtxp.entries[0].height_scale - 0.5).abs() < 0.001);
        assert!((mtxp.entries[0].height_offset - 1.0).abs() < 0.001);
        assert_eq!(mtxp.entries[1].flags, 2);
        assert!((mtxp.entries[1].height_scale - 1.0).abs() < 0.001);
        assert!((mtxp.entries[1].height_offset - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_mtxp_chunk_empty() {
        let data = vec![];
        let mut cursor = Cursor::new(data);
        let mtxp = MtxpChunk::read_le(&mut cursor).unwrap();
        assert_eq!(mtxp.entries.len(), 0);
    }

    #[test]
    fn test_mtxp_chunk_round_trip() {
        let original = MtxpChunk {
            entries: vec![
                TextureHeightParams {
                    flags: 1,
                    height_scale: 0.5,
                    height_offset: 1.0,
                    padding: 0,
                },
                TextureHeightParams {
                    flags: 2,
                    height_scale: 2.0,
                    height_offset: 0.25,
                    padding: 0,
                },
            ],
        };
        let mut buffer = Cursor::new(Vec::new());
        original.write_le(&mut buffer).unwrap();

        let data = buffer.into_inner();
        let mut cursor = Cursor::new(data);
        let parsed = MtxpChunk::read_le(&mut cursor).unwrap();

        assert_eq!(original.entries.len(), parsed.entries.len());
        for (orig, parsed) in original.entries.iter().zip(parsed.entries.iter()) {
            assert_eq!(orig.flags, parsed.flags);
            assert!((orig.height_scale - parsed.height_scale).abs() < 0.001);
            assert!((orig.height_offset - parsed.height_offset).abs() < 0.001);
        }
    }

    #[test]
    fn test_texture_height_params_uses_height_texture() {
        // Default values (0.0, 1.0) should NOT use height texture
        let default_params = TextureHeightParams {
            flags: 0,
            height_scale: 0.0,
            height_offset: 1.0,
            padding: 0,
        };
        assert!(!default_params.uses_height_texture());

        // Non-default height_scale should use height texture
        let with_scale = TextureHeightParams {
            flags: 0,
            height_scale: 0.5,
            height_offset: 1.0,
            padding: 0,
        };
        assert!(with_scale.uses_height_texture());

        // Non-default height_offset should use height texture
        let with_offset = TextureHeightParams {
            flags: 0,
            height_scale: 0.0,
            height_offset: 0.5,
            padding: 0,
        };
        assert!(with_offset.uses_height_texture());
    }

    #[test]
    fn test_mtxf_chunk_parse() {
        // 3 textures with flags 0x01, 0x02, 0x00
        let data = vec![
            0x01, 0x00, 0x00, 0x00, // flags[0] = 0x01
            0x02, 0x00, 0x00, 0x00, // flags[1] = 0x02
            0x00, 0x00, 0x00, 0x00, // flags[2] = 0x00
        ];
        let mut cursor = Cursor::new(data);
        let mtxf = MtxfChunk::read_le(&mut cursor).unwrap();
        assert_eq!(mtxf.flags.len(), 3);
        assert_eq!(mtxf.flags[0], 0x01);
        assert_eq!(mtxf.flags[1], 0x02);
        assert_eq!(mtxf.flags[2], 0x00);
    }

    #[test]
    fn test_mtxf_chunk_empty() {
        let data = vec![];
        let mut cursor = Cursor::new(data);
        let mtxf = MtxfChunk::read_le(&mut cursor).unwrap();
        assert_eq!(mtxf.flags.len(), 0);
    }

    #[test]
    fn test_mtxf_chunk_round_trip() {
        let original = MtxfChunk {
            flags: vec![0x01, 0x02, 0x04, 0x08],
        };
        let mut buffer = Cursor::new(Vec::new());
        original.write_le(&mut buffer).unwrap();

        let data = buffer.into_inner();
        let mut cursor = Cursor::new(data);
        let parsed = MtxfChunk::read_le(&mut cursor).unwrap();

        assert_eq!(original.flags.len(), parsed.flags.len());
        assert_eq!(original.flags, parsed.flags);
    }

    #[test]
    fn test_mtxf_chunk_single_flag() {
        // Single texture with flag 0x100 (specularity)
        let data = vec![0x00, 0x01, 0x00, 0x00];
        let mut cursor = Cursor::new(data);
        let mtxf = MtxfChunk::read_le(&mut cursor).unwrap();
        assert_eq!(mtxf.flags.len(), 1);
        assert_eq!(mtxf.flags[0], 0x100);
    }

    #[test]
    fn test_mtxf_chunk_all_zeros() {
        // All textures use default rendering (flags = 0)
        let data = vec![
            0x00, 0x00, 0x00, 0x00, // flags[0] = 0
            0x00, 0x00, 0x00, 0x00, // flags[1] = 0
            0x00, 0x00, 0x00, 0x00, // flags[2] = 0
        ];
        let mut cursor = Cursor::new(data);
        let mtxf = MtxfChunk::read_le(&mut cursor).unwrap();
        assert_eq!(mtxf.flags.len(), 3);
        assert!(mtxf.flags.iter().all(|&f| f == 0));
    }
}
