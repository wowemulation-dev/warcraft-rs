//! MPHD chunk - Map Header

use crate::error::{Error, Result};
use bitflags::bitflags;
use std::io::{Read, Write};

bitflags! {
    /// MPHD flags controlling map behaviors and features
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MphdFlags: u32 {
        /// Map is WMO-only (no terrain)
        const WDT_USES_GLOBAL_MAP_OBJ              = 0x0001;
        /// ADTs have vertex colors (MCCV chunks)
        const ADT_HAS_MCCV                         = 0x0002;
        /// ADTs use alternative terrain shader (big alpha)
        const ADT_HAS_BIG_ALPHA                    = 0x0004;
        /// Doodads are sorted by size category
        const ADT_HAS_DOODADREFS_SORTED_BY_SIZE_CAT = 0x0008;
        /// ADTs have lighting vertices (deprecated in BfA)
        const ADT_HAS_LIGHTING_VERTICES            = 0x0010;
        /// Flip ground display
        const ADT_HAS_UPSIDE_DOWN_GROUND           = 0x0020;
        /// Universal flag in 4.3.4+
        const UNK_FIRELANDS                        = 0x0040;
        /// Use _h textures for height-based blending
        const ADT_HAS_HEIGHT_TEXTURING             = 0x0080;
        /// Load _lod.adt files
        const UNK_LOAD_LOD                         = 0x0100;
        /// Has MAID chunk with FileDataIDs (8.1.0+)
        const WDT_HAS_MAID                         = 0x0200;
        /// Unknown flag
        const UNK_FLAG_0x0400                      = 0x0400;
        /// Unknown flag
        const UNK_FLAG_0x0800                      = 0x0800;
        /// Unknown flag
        const UNK_FLAG_0x1000                      = 0x1000;
        /// Unknown flag
        const UNK_FLAG_0x2000                      = 0x2000;
        /// Unknown flag
        const UNK_FLAG_0x4000                      = 0x4000;
        /// Unknown flag (continent-related?)
        const UNK_FLAG_0x8000                      = 0x8000;
    }
}

/// MPHD chunk - Map header with global properties
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MphdChunk {
    pub flags: MphdFlags,

    // Pre-8.1.0 fields
    pub something: u32,
    pub unused: [u32; 6],

    // Post-8.1.0 fields (reinterpret unused array)
    // These are only valid when WDT_HAS_MAID flag is set
    pub lgt_file_data_id: Option<u32>,  // _lgt.wdt
    pub occ_file_data_id: Option<u32>,  // _occ.wdt
    pub fogs_file_data_id: Option<u32>, // _fogs.wdt
    pub mpv_file_data_id: Option<u32>,  // _mpv.wdt
    pub tex_file_data_id: Option<u32>,  // _tex.wdt
    pub wdl_file_data_id: Option<u32>,  // .wdl
    pub pd4_file_data_id: Option<u32>,  // _pd4.wdt
}

/// FileDataIDs for BfA+ format
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDataIds {
    pub lgt: u32,
    pub occ: u32,
    pub fogs: u32,
    pub mpv: u32,
    pub tex: u32,
    pub wdl: u32,
    pub pd4: u32,
}

impl MphdChunk {
    pub fn new() -> Self {
        Self {
            flags: MphdFlags::empty(),
            something: 0,
            unused: [0; 6],
            lgt_file_data_id: None,
            occ_file_data_id: None,
            fogs_file_data_id: None,
            mpv_file_data_id: None,
            tex_file_data_id: None,
            wdl_file_data_id: None,
            pd4_file_data_id: None,
        }
    }

    /// Check if this is a WMO-only map
    pub fn is_wmo_only(&self) -> bool {
        self.flags.contains(MphdFlags::WDT_USES_GLOBAL_MAP_OBJ)
    }

    /// Check if MAID chunk should be present
    pub fn has_maid(&self) -> bool {
        self.flags.contains(MphdFlags::WDT_HAS_MAID)
    }

    /// Set FileDataIDs (for BfA+ format)
    pub fn set_file_data_ids(&mut self, ids: FileDataIds) {
        self.flags |= MphdFlags::WDT_HAS_MAID;
        self.lgt_file_data_id = Some(ids.lgt);
        self.occ_file_data_id = Some(ids.occ);
        self.fogs_file_data_id = Some(ids.fogs);
        self.mpv_file_data_id = Some(ids.mpv);
        self.tex_file_data_id = Some(ids.tex);
        self.wdl_file_data_id = Some(ids.wdl);
        self.pd4_file_data_id = Some(ids.pd4);
    }

    /// Clear FileDataIDs and revert to pre-BfA format
    pub fn clear_file_data_ids(&mut self) {
        self.flags.remove(MphdFlags::WDT_HAS_MAID);
        self.lgt_file_data_id = None;
        self.occ_file_data_id = None;
        self.fogs_file_data_id = None;
        self.mpv_file_data_id = None;
        self.tex_file_data_id = None;
        self.wdl_file_data_id = None;
        self.pd4_file_data_id = None;
    }
}

impl Default for MphdChunk {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Chunk for MphdChunk {
    fn magic() -> &'static [u8; 4] {
        b"DHPM" // 'MPHD' reversed
    }

    fn expected_size() -> Option<usize> {
        Some(32) // Always 32 bytes
    }

    fn read(reader: &mut impl Read, size: usize) -> Result<Self> {
        if size != 32 {
            return Err(Error::InvalidChunkSize {
                chunk: "MPHD".to_string(),
                expected: 32,
                found: size,
            });
        }

        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        let flags_raw = u32::from_le_bytes(buf);
        let flags = MphdFlags::from_bits(flags_raw).ok_or_else(|| Error::InvalidChunkData {
            chunk: "MPHD".to_string(),
            message: format!("Invalid flags: 0x{flags_raw:08X}"),
        })?;

        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        let something = u32::from_le_bytes(buf);

        let mut unused = [0u32; 6];
        for item in &mut unused {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf)?;
            *item = u32::from_le_bytes(buf);
        }

        let mut chunk = Self {
            flags,
            something,
            unused,
            lgt_file_data_id: None,
            occ_file_data_id: None,
            fogs_file_data_id: None,
            mpv_file_data_id: None,
            tex_file_data_id: None,
            wdl_file_data_id: None,
            pd4_file_data_id: None,
        };

        // If MAID flag is set, interpret the fields as FileDataIDs
        if flags.contains(MphdFlags::WDT_HAS_MAID) {
            chunk.lgt_file_data_id = Some(chunk.something);
            chunk.occ_file_data_id = Some(chunk.unused[0]);
            chunk.fogs_file_data_id = Some(chunk.unused[1]);
            chunk.mpv_file_data_id = Some(chunk.unused[2]);
            chunk.tex_file_data_id = Some(chunk.unused[3]);
            chunk.wdl_file_data_id = Some(chunk.unused[4]);
            chunk.pd4_file_data_id = Some(chunk.unused[5]);
        }

        Ok(chunk)
    }

    fn write(&self, writer: &mut impl Write) -> Result<()> {
        writer.write_all(&self.flags.bits().to_le_bytes())?;

        // Write fields based on whether we have FileDataIDs
        if self.has_maid() {
            // Write FileDataIDs
            writer.write_all(&self.lgt_file_data_id.unwrap_or(0).to_le_bytes())?;
            writer.write_all(&self.occ_file_data_id.unwrap_or(0).to_le_bytes())?;
            writer.write_all(&self.fogs_file_data_id.unwrap_or(0).to_le_bytes())?;
            writer.write_all(&self.mpv_file_data_id.unwrap_or(0).to_le_bytes())?;
            writer.write_all(&self.tex_file_data_id.unwrap_or(0).to_le_bytes())?;
            writer.write_all(&self.wdl_file_data_id.unwrap_or(0).to_le_bytes())?;
            writer.write_all(&self.pd4_file_data_id.unwrap_or(0).to_le_bytes())?;
        } else {
            // Write legacy format
            writer.write_all(&self.something.to_le_bytes())?;
            for &val in &self.unused {
                writer.write_all(&val.to_le_bytes())?;
            }
        }

        Ok(())
    }

    fn size(&self) -> usize {
        32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunks::Chunk;

    #[test]
    fn test_mphd_flags() {
        let mut flags = MphdFlags::empty();
        assert!(!flags.contains(MphdFlags::WDT_USES_GLOBAL_MAP_OBJ));

        flags |= MphdFlags::WDT_USES_GLOBAL_MAP_OBJ;
        assert!(flags.contains(MphdFlags::WDT_USES_GLOBAL_MAP_OBJ));

        flags |= MphdFlags::ADT_HAS_MCCV | MphdFlags::ADT_HAS_BIG_ALPHA;
        assert_eq!(flags.bits(), 0x0007);
    }

    #[test]
    fn test_mphd_read_write() {
        let mut chunk = MphdChunk::new();
        chunk.flags = MphdFlags::WDT_USES_GLOBAL_MAP_OBJ | MphdFlags::ADT_HAS_HEIGHT_TEXTURING;
        chunk.something = 12345;

        let mut buffer = Vec::new();
        chunk.write(&mut buffer).unwrap();

        let mut reader = std::io::Cursor::new(buffer);
        let read_chunk = MphdChunk::read(&mut reader, 32).unwrap();

        assert_eq!(chunk.flags, read_chunk.flags);
        assert_eq!(chunk.something, read_chunk.something);
    }

    #[test]
    fn test_mphd_file_data_ids() {
        let mut chunk = MphdChunk::new();
        chunk.set_file_data_ids(FileDataIds {
            lgt: 100,
            occ: 101,
            fogs: 102,
            mpv: 103,
            tex: 104,
            wdl: 105,
            pd4: 106,
        });

        assert!(chunk.has_maid());
        assert_eq!(chunk.lgt_file_data_id, Some(100));
        assert_eq!(chunk.pd4_file_data_id, Some(106));

        let mut buffer = Vec::new();
        chunk.write(&mut buffer).unwrap();

        let mut reader = std::io::Cursor::new(buffer);
        let read_chunk = MphdChunk::read(&mut reader, 32).unwrap();

        assert_eq!(read_chunk.lgt_file_data_id, Some(100));
        assert_eq!(read_chunk.pd4_file_data_id, Some(106));
    }
}
