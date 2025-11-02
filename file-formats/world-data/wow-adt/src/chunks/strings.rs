//! String chunk parsers for ADT format.
//!
//! String chunks (MTEX, MMDX, MWMO) store null-terminated filenames consecutively.
//! Index chunks (MMID, MWID) store offset tables referencing string data.
//!
//! String data is stored as consecutive null-terminated ASCII/UTF-8 strings with
//! no padding between entries. Some files contain non-ASCII characters requiring
//! lossy UTF-8 conversion for compatibility.

use binrw::{BinRead, BinWrite};
use std::io::{Read, Seek, Write};

/// MTEX chunk - Texture filenames (Vanilla+)
///
/// Contains null-terminated texture filenames (e.g., "Tileset\\Terrain.blp\0").
/// Referenced by texture layer indices in MCLY chunks.
///
/// Filenames are stored consecutively without padding. Parse until end of chunk.
/// Use lossy UTF-8 conversion to handle non-ASCII characters in some files.
///
/// Reference: <https://wowdev.wiki/ADT/v18#MTEX_chunk>
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ✅ Introduced
/// - **TBC (2.4.3)**: ✅ Present
/// - **WotLK (3.3.5a)**: ✅ Present
/// - **Cataclysm (4.3.4)**: ✅ Present (moved to _tex0.adt in split files)
/// - **MoP (5.4.8)**: ✅ Present (in _tex0.adt split files)
#[derive(Debug, Clone, Default)]
pub struct MtexChunk {
    /// Texture filenames (null-terminated)
    pub filenames: Vec<String>,
}

impl BinRead for MtexChunk {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let filenames = parse_null_terminated_strings(reader)?;
        Ok(Self { filenames })
    }
}

impl BinWrite for MtexChunk {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        for filename in &self.filenames {
            writer.write_all(filename.as_bytes())?;
            writer.write_all(&[0u8])?; // Null terminator
        }
        Ok(())
    }
}

/// MMDX chunk - M2 model filenames (Vanilla+)
///
/// Contains null-terminated M2 model filenames (e.g., "World\\Doodad\\Model.m2\0").
/// Referenced by MMID offset indices for model placement via MDDF chunks.
///
/// Reference: <https://wowdev.wiki/ADT/v18#MMDX_chunk>
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ✅ Introduced
/// - **TBC (2.4.3)**: ✅ Present
/// - **WotLK (3.3.5a)**: ✅ Present
/// - **Cataclysm (4.3.4)**: ✅ Present (moved to _obj0.adt in split files)
/// - **MoP (5.4.8)**: ✅ Present (in _obj0.adt split files)
#[derive(Debug, Clone, Default)]
pub struct MmdxChunk {
    /// M2 model filenames (null-terminated)
    pub filenames: Vec<String>,
}

impl BinRead for MmdxChunk {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let filenames = parse_null_terminated_strings(reader)?;
        Ok(Self { filenames })
    }
}

impl BinWrite for MmdxChunk {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        for filename in &self.filenames {
            writer.write_all(filename.as_bytes())?;
            writer.write_all(&[0u8])?; // Null terminator
        }
        Ok(())
    }
}

/// MWMO chunk - WMO filenames (Vanilla+)
///
/// Contains null-terminated World Map Object (WMO) filenames.
/// Referenced by MWID offset indices for WMO placement.
///
/// Reference: <https://wowdev.wiki/ADT/v18#MWMO_chunk>
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ✅ Introduced
/// - **TBC (2.4.3)**: ✅ Present
/// - **WotLK (3.3.5a)**: ✅ Present
/// - **Cataclysm (4.3.4)**: ✅ Present (moved to _obj0.adt in split files)
/// - **MoP (5.4.8)**: ✅ Present (in _obj0.adt split files)
#[derive(Debug, Clone, Default)]
pub struct MwmoChunk {
    /// WMO filenames (null-terminated)
    pub filenames: Vec<String>,
}

impl BinRead for MwmoChunk {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let filenames = parse_null_terminated_strings(reader)?;
        Ok(Self { filenames })
    }
}

impl BinWrite for MwmoChunk {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        for filename in &self.filenames {
            writer.write_all(filename.as_bytes())?;
            writer.write_all(&[0u8])?; // Null terminator
        }
        Ok(())
    }
}

/// MMID chunk - M2 model offset indices (Vanilla+)
///
/// Contains offset table into MMDX string data. Each u32 is a byte offset
/// pointing to a null-terminated filename in the MMDX chunk.
///
/// Used by MDDF (doodad placement) chunk to reference models.
///
/// Reference: <https://wowdev.wiki/ADT/v18#MMID_chunk>
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ✅ Introduced
/// - **TBC (2.4.3)**: ✅ Present
/// - **WotLK (3.3.5a)**: ✅ Present
/// - **Cataclysm (4.3.4)**: ✅ Present (moved to _obj0.adt in split files)
/// - **MoP (5.4.8)**: ✅ Present (in _obj0.adt split files)
#[derive(Debug, Clone, Default, BinRead, BinWrite)]
#[brw(little)]
pub struct MmidChunk {
    /// Byte offsets into MMDX chunk (one per model)
    #[br(parse_with = binrw::helpers::until_eof)]
    pub offsets: Vec<u32>,
}

impl MmidChunk {
    /// Validate that all offsets are within MMDX chunk bounds.
    ///
    /// # Arguments
    ///
    /// * `mmdx_size` - Total size of MMDX chunk data in bytes
    ///
    /// # Returns
    ///
    /// `true` if all offsets are valid, `false` if any offset is out of bounds
    pub fn validate_offsets(&self, mmdx_size: usize) -> bool {
        self.offsets
            .iter()
            .all(|&offset| (offset as usize) < mmdx_size)
    }

    /// Get filename index for a given offset by counting null terminators.
    ///
    /// # Arguments
    ///
    /// * `mmdx` - Reference to MMDX chunk containing filenames
    /// * `offset` - Byte offset into MMDX data
    ///
    /// # Returns
    ///
    /// Index into `mmdx.filenames` array, or None if offset is invalid
    pub fn get_filename_index(&self, mmdx: &MmdxChunk, offset: u32) -> Option<usize> {
        // Count null terminators before this offset to find filename index
        let mut current_offset = 0;
        for (idx, filename) in mmdx.filenames.iter().enumerate() {
            if current_offset == offset {
                return Some(idx);
            }
            current_offset += filename.len() as u32 + 1; // +1 for null terminator
        }
        None
    }
}

/// MWID chunk - WMO offset indices (Vanilla+)
///
/// Contains offset table into MWMO string data. Each u32 is a byte offset
/// pointing to a null-terminated filename in the MWMO chunk.
///
/// Used by MODF (WMO placement) chunk to reference World Map Objects.
///
/// Reference: <https://wowdev.wiki/ADT/v18#MWID_chunk>
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ✅ Introduced
/// - **TBC (2.4.3)**: ✅ Present
/// - **WotLK (3.3.5a)**: ✅ Present
/// - **Cataclysm (4.3.4)**: ✅ Present (moved to _obj0.adt in split files)
/// - **MoP (5.4.8)**: ✅ Present (in _obj0.adt split files)
#[derive(Debug, Clone, Default, BinRead, BinWrite)]
#[brw(little)]
pub struct MwidChunk {
    /// Byte offsets into MWMO chunk (one per WMO)
    #[br(parse_with = binrw::helpers::until_eof)]
    pub offsets: Vec<u32>,
}

impl MwidChunk {
    /// Validate that all offsets are within MWMO chunk bounds.
    ///
    /// # Arguments
    ///
    /// * `mwmo_size` - Total size of MWMO chunk data in bytes
    ///
    /// # Returns
    ///
    /// `true` if all offsets are valid, `false` if any offset is out of bounds
    pub fn validate_offsets(&self, mwmo_size: usize) -> bool {
        self.offsets
            .iter()
            .all(|&offset| (offset as usize) < mwmo_size)
    }

    /// Get filename index for a given offset by counting null terminators.
    ///
    /// # Arguments
    ///
    /// * `mwmo` - Reference to MWMO chunk containing filenames
    /// * `offset` - Byte offset into MWMO data
    ///
    /// # Returns
    ///
    /// Index into `mwmo.filenames` array, or None if offset is invalid
    pub fn get_filename_index(&self, mwmo: &MwmoChunk, offset: u32) -> Option<usize> {
        let mut current_offset = 0;
        for (idx, filename) in mwmo.filenames.iter().enumerate() {
            if current_offset == offset {
                return Some(idx);
            }
            current_offset += filename.len() as u32 + 1; // +1 for null terminator
        }
        None
    }
}

/// Parse null-terminated strings until end of stream.
///
/// Reads consecutive null-terminated strings from the reader until EOF.
/// Uses lossy UTF-8 conversion to handle potential non-ASCII characters.
fn parse_null_terminated_strings<R: Read>(reader: &mut R) -> binrw::BinResult<Vec<String>> {
    let mut strings = Vec::new();

    loop {
        let mut string_bytes = Vec::new();

        // Read bytes until null terminator or EOF
        loop {
            let mut byte = [0u8; 1];
            match reader.read_exact(&mut byte) {
                Ok(()) => {
                    if byte[0] == 0 {
                        // Null terminator found
                        break;
                    }
                    string_bytes.push(byte[0]);
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // EOF reached - if we have bytes, create final string
                    if !string_bytes.is_empty() {
                        let string = String::from_utf8_lossy(&string_bytes).into_owned();
                        strings.push(string);
                    }
                    return Ok(strings);
                }
                Err(e) => {
                    return Err(binrw::Error::Io(e));
                }
            }
        }

        // Create string from collected bytes (lossy UTF-8)
        if !string_bytes.is_empty() {
            let string = String::from_utf8_lossy(&string_bytes).into_owned();
            strings.push(string);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_mtex_chunk_parse() {
        // Two filenames: "test.blp\0" and "world.blp\0"
        let data = b"test.blp\0world.blp\0";
        let mut cursor = Cursor::new(data.to_vec());

        let mtex = MtexChunk::read_le(&mut cursor).unwrap();
        assert_eq!(mtex.filenames.len(), 2);
        assert_eq!(mtex.filenames[0], "test.blp");
        assert_eq!(mtex.filenames[1], "world.blp");
    }

    #[test]
    fn test_mtex_chunk_single_file() {
        let data = b"terrain.blp\0";
        let mut cursor = Cursor::new(data.to_vec());

        let mtex = MtexChunk::read_le(&mut cursor).unwrap();
        assert_eq!(mtex.filenames.len(), 1);
        assert_eq!(mtex.filenames[0], "terrain.blp");
    }

    #[test]
    fn test_mtex_chunk_empty() {
        let data = b"";
        let mut cursor = Cursor::new(data.to_vec());

        let mtex = MtexChunk::read_le(&mut cursor).unwrap();
        assert_eq!(mtex.filenames.len(), 0);
    }

    #[test]
    fn test_mtex_chunk_utf8_lossy() {
        // Test lossy UTF-8 conversion with invalid byte
        let data = b"test\xFF.blp\0"; // 0xFF is invalid UTF-8
        let mut cursor = Cursor::new(data.to_vec());

        let mtex = MtexChunk::read_le(&mut cursor).unwrap();
        assert_eq!(mtex.filenames.len(), 1);
        // Should handle invalid UTF-8 gracefully with replacement character
        assert!(mtex.filenames[0].contains("test"));
        assert!(mtex.filenames[0].contains(".blp"));
    }

    #[test]
    fn test_mtex_chunk_backslash_path() {
        // Real WoW texture path with backslashes
        let data = b"Tileset\\Generic\\CityTile.blp\0";
        let mut cursor = Cursor::new(data.to_vec());

        let mtex = MtexChunk::read_le(&mut cursor).unwrap();
        assert_eq!(mtex.filenames.len(), 1);
        assert_eq!(mtex.filenames[0], "Tileset\\Generic\\CityTile.blp");
    }

    #[test]
    fn test_mtex_chunk_multiple_paths() {
        let data = b"Tileset\\Ground.blp\0Tileset\\Detail.blp\0Tileset\\Alpha.blp\0";
        let mut cursor = Cursor::new(data.to_vec());

        let mtex = MtexChunk::read_le(&mut cursor).unwrap();
        assert_eq!(mtex.filenames.len(), 3);
        assert_eq!(mtex.filenames[0], "Tileset\\Ground.blp");
        assert_eq!(mtex.filenames[1], "Tileset\\Detail.blp");
        assert_eq!(mtex.filenames[2], "Tileset\\Alpha.blp");
    }

    #[test]
    fn test_mtex_chunk_round_trip() {
        let original = MtexChunk {
            filenames: vec![
                "texture1.blp".to_string(),
                "texture2.blp".to_string(),
                "texture3.blp".to_string(),
            ],
        };

        let mut buffer = Cursor::new(Vec::new());
        original.write_le(&mut buffer).unwrap();

        let data = buffer.into_inner();
        let mut cursor = Cursor::new(data);
        let parsed = MtexChunk::read_le(&mut cursor).unwrap();

        assert_eq!(original.filenames.len(), parsed.filenames.len());
        assert_eq!(original.filenames, parsed.filenames);
    }

    #[test]
    fn test_mtex_chunk_round_trip_with_paths() {
        let original = MtexChunk {
            filenames: vec![
                "Tileset\\Ground\\Grass.blp".to_string(),
                "Tileset\\Detail\\Rock.blp".to_string(),
            ],
        };

        let mut buffer = Cursor::new(Vec::new());
        original.write_le(&mut buffer).unwrap();

        let data = buffer.into_inner();
        let mut cursor = Cursor::new(data);
        let parsed = MtexChunk::read_le(&mut cursor).unwrap();

        assert_eq!(original.filenames, parsed.filenames);
    }

    #[test]
    fn test_mtex_chunk_default() {
        let mtex = MtexChunk::default();
        assert_eq!(mtex.filenames.len(), 0);
    }

    #[test]
    fn test_mtex_chunk_no_trailing_null() {
        // Edge case: file ends without final null terminator
        let data = b"test.blp\0world.blp";
        let mut cursor = Cursor::new(data.to_vec());

        let mtex = MtexChunk::read_le(&mut cursor).unwrap();
        assert_eq!(mtex.filenames.len(), 2);
        assert_eq!(mtex.filenames[0], "test.blp");
        assert_eq!(mtex.filenames[1], "world.blp");
    }

    #[test]
    fn test_mwmo_chunk() {
        let data = b"building.wmo\0castle.wmo\0";
        let mut cursor = Cursor::new(data.to_vec());

        let mwmo = MwmoChunk::read_le(&mut cursor).unwrap();
        assert_eq!(mwmo.filenames.len(), 2);
        assert_eq!(mwmo.filenames[0], "building.wmo");
        assert_eq!(mwmo.filenames[1], "castle.wmo");
    }

    #[test]
    fn test_mwmo_chunk_single_file() {
        let data = b"dungeon.wmo\0";
        let mut cursor = Cursor::new(data.to_vec());

        let mwmo = MwmoChunk::read_le(&mut cursor).unwrap();
        assert_eq!(mwmo.filenames.len(), 1);
        assert_eq!(mwmo.filenames[0], "dungeon.wmo");
    }

    #[test]
    fn test_mwmo_chunk_empty() {
        let data = b"";
        let mut cursor = Cursor::new(data.to_vec());

        let mwmo = MwmoChunk::read_le(&mut cursor).unwrap();
        assert_eq!(mwmo.filenames.len(), 0);
    }

    #[test]
    fn test_mwmo_chunk_with_path() {
        let data = b"World\\wmo\\Dungeon\\Dungeon.wmo\0";
        let mut cursor = Cursor::new(data.to_vec());

        let mwmo = MwmoChunk::read_le(&mut cursor).unwrap();
        assert_eq!(mwmo.filenames.len(), 1);
        assert_eq!(mwmo.filenames[0], "World\\wmo\\Dungeon\\Dungeon.wmo");
    }

    #[test]
    fn test_mwmo_chunk_round_trip() {
        let original = MwmoChunk {
            filenames: vec!["building.wmo".to_string(), "castle.wmo".to_string()],
        };

        let mut buffer = Cursor::new(Vec::new());
        original.write_le(&mut buffer).unwrap();

        let data = buffer.into_inner();
        let mut cursor = Cursor::new(data);
        let parsed = MwmoChunk::read_le(&mut cursor).unwrap();

        assert_eq!(original.filenames, parsed.filenames);
    }

    #[test]
    fn test_mmdx_chunk() {
        let data = b"model1.m2\0model2.m2\0";
        let mut cursor = Cursor::new(data.to_vec());

        let mmdx = MmdxChunk::read_le(&mut cursor).unwrap();
        assert_eq!(mmdx.filenames.len(), 2);
        assert_eq!(mmdx.filenames[0], "model1.m2");
        assert_eq!(mmdx.filenames[1], "model2.m2");
    }

    #[test]
    fn test_mmdx_chunk_with_path() {
        let data = b"World\\Doodad\\Tree.m2\0";
        let mut cursor = Cursor::new(data.to_vec());

        let mmdx = MmdxChunk::read_le(&mut cursor).unwrap();
        assert_eq!(mmdx.filenames.len(), 1);
        assert_eq!(mmdx.filenames[0], "World\\Doodad\\Tree.m2");
    }

    #[test]
    fn test_mmdx_chunk_round_trip() {
        let original = MmdxChunk {
            filenames: vec!["model1.m2".to_string(), "model2.m2".to_string()],
        };

        let mut buffer = Cursor::new(Vec::new());
        original.write_le(&mut buffer).unwrap();

        let data = buffer.into_inner();
        let mut cursor = Cursor::new(data);
        let parsed = MmdxChunk::read_le(&mut cursor).unwrap();

        assert_eq!(original.filenames, parsed.filenames);
    }

    #[test]
    fn test_mmid_chunk() {
        // 3 offsets: 0, 10, 20
        let data = vec![
            0x00, 0x00, 0x00, 0x00, // offset[0] = 0
            0x0A, 0x00, 0x00, 0x00, // offset[1] = 10
            0x14, 0x00, 0x00, 0x00, // offset[2] = 20
        ];
        let mut cursor = Cursor::new(data);
        let mmid = MmidChunk::read_le(&mut cursor).unwrap();

        assert_eq!(mmid.offsets.len(), 3);
        assert_eq!(mmid.offsets[0], 0);
        assert_eq!(mmid.offsets[1], 10);
        assert_eq!(mmid.offsets[2], 20);
    }

    #[test]
    fn test_mmid_validate_offsets() {
        let mmid = MmidChunk {
            offsets: vec![0, 10, 20],
        };

        // All offsets valid with MMDX size of 30
        assert!(mmid.validate_offsets(30));

        // Offset 20 invalid with MMDX size of 20
        assert!(!mmid.validate_offsets(20));
    }

    #[test]
    fn test_mmid_get_filename_index() {
        let mmdx = MmdxChunk {
            filenames: vec![
                "model1.m2".to_string(), // offset 0
                "model2.m2".to_string(), // offset 10 (9 + 1)
                "model3.m2".to_string(), // offset 20 (10 + 10)
            ],
        };

        let mmid = MmidChunk {
            offsets: vec![0, 10, 20],
        };

        assert_eq!(mmid.get_filename_index(&mmdx, 0), Some(0));
        assert_eq!(mmid.get_filename_index(&mmdx, 10), Some(1));
        assert_eq!(mmid.get_filename_index(&mmdx, 20), Some(2));
        assert_eq!(mmid.get_filename_index(&mmdx, 999), None);
    }
    #[test]
    fn test_mwid_chunk() {
        // 2 offsets: 0, 15
        let data = vec![
            0x00, 0x00, 0x00, 0x00, // offset[0] = 0
            0x0F, 0x00, 0x00, 0x00, // offset[1] = 15
        ];
        let mut cursor = Cursor::new(data);
        let mwid = MwidChunk::read_le(&mut cursor).unwrap();

        assert_eq!(mwid.offsets.len(), 2);
        assert_eq!(mwid.offsets[0], 0);
        assert_eq!(mwid.offsets[1], 15);
    }

    #[test]
    fn test_mwid_validate_offsets() {
        let mwid = MwidChunk {
            offsets: vec![0, 15, 30],
        };

        // All offsets valid with MWMO size of 40
        assert!(mwid.validate_offsets(40));

        // Offset 30 invalid with MWMO size of 30
        assert!(!mwid.validate_offsets(30));
    }

    #[test]
    fn test_mwid_get_filename_index() {
        let mwmo = MwmoChunk {
            filenames: vec![
                "building.wmo".to_string(), // offset 0
                "castle.wmo".to_string(),   // offset 13 (12 + 1)
            ],
        };

        let mwid = MwidChunk {
            offsets: vec![0, 13],
        };

        assert_eq!(mwid.get_filename_index(&mwmo, 0), Some(0));
        assert_eq!(mwid.get_filename_index(&mwmo, 13), Some(1));
        assert_eq!(mwid.get_filename_index(&mwmo, 999), None);
    }
}
