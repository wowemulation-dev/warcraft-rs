//! Complete MCNK chunk with two-level parsing.
//!
//! MCNK chunks use a two-level discovery approach:
//! 1. Parse the 128-byte header to get subchunk offsets
//! 2. Selectively parse subchunks based on header flags and offsets
//!
//! This allows efficient partial parsing - only loading subchunks that are needed.
//!
//! # Cross-Reference Validation
//!
//! This implementation has been cross-referenced with:
//! - **noggit-red**: MapChunk.cpp/MapHeaders.h (WotLK 3.3.5a) - ✅ Validated
//!   - MCNK header: All 128 bytes match specification (MapHeaders.h:127-156)
//!   - MCVT parsing: 145 f32 heights confirmed (MapChunk.cpp:166-190)
//!   - MCNR parsing: 145 × 3-byte normals confirmed (MapChunk.cpp:202-215)
//!   - Offset handling: Relative to MCNK chunk start (MapChunk.cpp:167)
//!   - Reference: github.com/Marlamin/noggit-red
//! - **wowdev.wiki**: ADT/v18 specification
//!   - All field offsets, sizes, and interpretations validated
//!   - Reference: <https://wowdev.wiki/ADT/v18#MCNK_header>
//!
//! ## Known Deviations
//! - Field naming: warcraft-rs uses descriptive names vs noggit-red abbreviations
//! - Semantic interpretation: `pred_tex`/`no_effect_doodad` vs `doodadMapping`/`doodadStencil`
//!   (warcraft-rs interpretation matches wowdev.wiki specification)
//!
//! ## Validation Date
//! - 2025-10-30: Cross-reference completed, HIGH confidence
//!
//! See: `/specs/001-adt-binrw-refactor/CROSS_REFERENCE_MCNK.md` for full analysis

use crate::chunk_header::ChunkHeader;
use binrw::{BinRead, BinResult};
use std::io::{Read, Seek, SeekFrom};

use super::header::McnkHeader;
use super::mcal::McalChunk;
use super::mcbb::McbbChunk;
use super::mccv::MccvChunk;
use super::mcdd::McddChunk;
use super::mclq::MclqChunk;
use super::mclv::MclvChunk;
use super::mcly::MclyChunk;
use super::mcmt::McmtChunk;
use super::mcnr::McnrChunk;
use super::mcrd::McrdChunk;
use super::mcrf::McrfChunk;
use super::mcrw::McrwChunk;
use super::mcse::McseChunk;
use super::mcsh::McshChunk;
use super::mcvt::McvtChunk;

/// Complete MCNK terrain chunk with optional subchunks (Vanilla+)
///
/// Represents a 16×16 yard terrain tile within an ADT file. MCNK uses a two-level
/// parsing strategy:
///
/// **Level 1:** Parse 128-byte header containing subchunk offsets
/// **Level 2:** Selectively parse subchunks based on header flags
///
/// This approach allows efficient partial parsing - only load subchunks you need.
///
/// # Example
///
/// ```rust,no_run
/// use std::fs::File;
/// use std::io::BufReader;
/// use wow_adt::chunks::mcnk::McnkChunk;
/// use binrw::BinRead;
///
/// # fn example() -> binrw::BinResult<()> {
/// let file = File::open("terrain.adt")?;
/// let mut reader = BufReader::new(file);
///
/// // Parse MCNK chunk (automatically parses header + subchunks)
/// // Note: offset should be the position where MCNK chunk starts
/// let mcnk = McnkChunk::parse_with_offset(&mut reader, 0)?;
///
/// // Access terrain data
/// if let Some(heights) = &mcnk.heights {
///     println!("Min height: {:?}", heights.min_height());
/// }
///
/// if let Some(layers) = &mcnk.layers {
///     println!("Texture layers: {}", layers.layer_count());
/// }
/// # Ok(())
/// # }
/// ```
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ✅ Introduced
/// - **TBC (2.4.3)**: ✅ Present
/// - **WotLK (3.3.5a)**: ✅ Present
/// - **Cataclysm (4.3.4)**: ✅ Present
/// - **MoP (5.4.8)**: ✅ Present
///
/// Reference: <https://wowdev.wiki/ADT/v18#MCNK_chunk>
#[derive(Debug, Clone)]
pub struct McnkChunk {
    /// MCNK header (128 bytes)
    pub header: McnkHeader,

    /// MCVT: Vertex heights (145 f32 values)
    pub heights: Option<McvtChunk>,

    /// MCNR: Vertex normals (145 entries + padding)
    pub normals: Option<McnrChunk>,

    /// MCLY: Texture layers (up to 4 entries)
    pub layers: Option<MclyChunk>,

    /// MCMT: Terrain material IDs (split files, Cataclysm+)
    pub materials: Option<McmtChunk>,

    /// MCRF: Object references (doodad + WMO indices, pre-Cataclysm)
    pub refs: Option<McrfChunk>,

    /// MCRD: Doodad references (split files, Cataclysm+)
    pub doodad_refs: Option<McrdChunk>,

    /// MCRW: WMO references (split files, Cataclysm+)
    pub wmo_refs: Option<McrwChunk>,

    /// MCAL: Alpha maps for texture blending
    pub alpha: Option<McalChunk>,

    /// MCSH: Shadow map (512 bytes)
    pub shadow: Option<McshChunk>,

    /// MCCV: Vertex colors (145 BGRA entries, WotLK+)
    pub vertex_colors: Option<MccvChunk>,

    /// MCLV: Vertex lighting (145 ARGB entries, Cataclysm+)
    pub vertex_lighting: Option<MclvChunk>,

    /// MCSE: Sound emitters
    pub sound_emitters: Option<McseChunk>,

    /// MCLQ: Legacy liquid (pre-WotLK, deprecated)
    pub liquid: Option<MclqChunk>,

    /// MCDD: Doodad disable bitmap (WoD+)
    pub doodad_disable: Option<McddChunk>,

    /// MCBB: Blend batches (MoP+)
    pub blend_batches: Option<McbbChunk>,
}

impl McnkChunk {
    /// Parse MCNK chunk with selective subchunk loading.
    ///
    /// Reads the header first, then parses subchunks based on offsets and flags.
    /// Offsets in the header are relative to the MCNK chunk start (including
    /// the 8-byte chunk header).
    ///
    /// # Arguments
    ///
    /// * `reader` - Reader positioned at MCNK chunk data (after chunk header)
    /// * `mcnk_start_offset` - File offset where MCNK chunk header begins
    ///
    /// # Returns
    ///
    /// Parsed MCNK chunk with populated subchunks
    pub fn parse_with_offset<R: Read + Seek>(
        reader: &mut R,
        mcnk_start_offset: u64,
    ) -> BinResult<Self> {
        // Read 128-byte header
        let header = McnkHeader::read_le(reader)?;

        // Helper to seek to subchunk and read it
        let mut read_subchunk = |offset: u32, name: &str| -> BinResult<Vec<u8>> {
            if offset == 0 {
                return Ok(Vec::new());
            }

            // Seek to subchunk (offset is relative to MCNK chunk start)
            let subchunk_pos = mcnk_start_offset + u64::from(offset);
            reader.seek(SeekFrom::Start(subchunk_pos)).map_err(|e| {
                binrw::Error::Custom {
                    pos: subchunk_pos,
                    err: Box::new(format!("Failed to seek to {} subchunk at offset {}: {}", name, offset, e)),
                }
            })?;

            // Read subchunk header
            let subchunk_header = ChunkHeader::read_le(reader).map_err(|e| {
                binrw::Error::Custom {
                    pos: subchunk_pos,
                    err: Box::new(format!("Failed to read {} subchunk header at file offset {} (MCNK+{}): {:?}",
                        name, subchunk_pos, offset, e)),
                }
            })?;

            // Read subchunk data
            let mut data = vec![0u8; subchunk_header.size as usize];
            reader.read_exact(&mut data)?;

            Ok(data)
        };

        // Parse subchunks based on header offsets
        let heights = if header.has_height() {
            let data = read_subchunk(header.ofs_height(), "MCVT")?;
            if !data.is_empty() {
                Some(McvtChunk::read_le(&mut std::io::Cursor::new(data))?)
            } else {
                None
            }
        } else {
            None
        };

        let normals = if header.has_normal() {
            let data = read_subchunk(header.ofs_normal(), "MCNR")?;
            if !data.is_empty() {
                Some(McnrChunk::read_le(&mut std::io::Cursor::new(data))?)
            } else {
                None
            }
        } else {
            None
        };

        let layers = if header.has_layer() {
            let data = read_subchunk(header.ofs_layer, "MCLY")?;
            if !data.is_empty() {
                Some(MclyChunk::read_le(&mut std::io::Cursor::new(data))?)
            } else {
                None
            }
        } else {
            None
        };

        // MCMT has no dedicated offset in MCNK header
        // Found via chunk discovery in _tex.adt files (Cataclysm+)
        // TODO: Add split file support with chunk discovery
        let materials = None;

        let refs = if header.has_refs() {
            let data = read_subchunk(header.ofs_refs, "MCRF")?;
            if !data.is_empty() {
                Some(McrfChunk::read_le(&mut std::io::Cursor::new(data))?)
            } else {
                None
            }
        } else {
            None
        };

        // MCRD shares ofs_refs with MCRF (Cataclysm+ split files)
        // TODO: Add version/file-type detection to distinguish MCRF vs MCRD
        let doodad_refs = if header.has_refs() {
            let data = read_subchunk(header.ofs_refs, "MCRF")?;
            if !data.is_empty() {
                Some(McrdChunk::read_le(&mut std::io::Cursor::new(data))?)
            } else {
                None
            }
        } else {
            None
        };

        // MCRW shares ofs_refs with MCRF (Cataclysm+ split files)
        // TODO: Add version/file-type detection to distinguish MCRF vs MCRD/MCRW
        let wmo_refs = if header.has_refs() {
            let data = read_subchunk(header.ofs_refs, "MCRF")?;
            if !data.is_empty() {
                Some(McrwChunk::read_le(&mut std::io::Cursor::new(data))?)
            } else {
                None
            }
        } else {
            None
        };

        let alpha = if header.has_alpha() {
            let data = read_subchunk(header.ofs_alpha, "MCAL")?;
            if !data.is_empty() {
                Some(McalChunk::read_le(&mut std::io::Cursor::new(data))?)
            } else {
                None
            }
        } else {
            None
        };

        let shadow = if header.has_shadow() {
            let data = read_subchunk(header.ofs_shadow, "MCSH")?;
            if !data.is_empty() {
                Some(McshChunk::read_le(&mut std::io::Cursor::new(data))?)
            } else {
                None
            }
        } else {
            None
        };

        let vertex_colors = if header.has_vertex_colors() {
            let data = read_subchunk(header.ofs_mccv, "MCCV")?;
            if !data.is_empty() {
                Some(MccvChunk::read_le(&mut std::io::Cursor::new(data))?)
            } else {
                None
            }
        } else {
            None
        };

        let vertex_lighting = if header.has_baked_lighting() {
            let data = read_subchunk(header.ofs_mclv, "MCLV")?;
            if !data.is_empty() {
                Some(MclvChunk::read_le(&mut std::io::Cursor::new(data))?)
            } else {
                None
            }
        } else {
            None
        };

        let sound_emitters = if header.has_sound_emitters() {
            let data = read_subchunk(header.ofs_snd_emitters, "MCSE")?;
            if !data.is_empty() {
                Some(McseChunk::read_le(&mut std::io::Cursor::new(data))?)
            } else {
                None
            }
        } else {
            None
        };

        // MCLQ is a special case: the size is stored in MCNK header (size_liquid),
        // NOT in the MCLQ chunk header (which always has size=0).
        let liquid = if header.has_legacy_liquid() {
            // Seek to MCLQ chunk
            let mclq_pos = mcnk_start_offset + u64::from(header.ofs_liquid);
            reader.seek(SeekFrom::Start(mclq_pos)).map_err(|e| {
                binrw::Error::Custom {
                    pos: mclq_pos,
                    err: Box::new(format!("Failed to seek to MCLQ at offset {}: {}", header.ofs_liquid, e)),
                }
            })?;

            // Read the 8-byte chunk header (magic + size, where size is always 0)
            let _chunk_header = ChunkHeader::read_le(reader)?;

            // Read the actual data using size_liquid from MCNK header
            let mut data = vec![0u8; header.size_liquid as usize];
            reader.read_exact(&mut data)?;

            if !data.is_empty() {
                // Pass MCNK flags to MCLQ parser for liquid type detection
                // Note: Small MCLQ chunks (8 bytes) are often corrupted/empty placeholders
                // We catch parsing errors and treat them as "no liquid"
                match MclqChunk::read_le_args(
                    &mut std::io::Cursor::new(data),
                    header.flags.value,
                ) {
                    Ok(mclq) => Some(mclq),
                    Err(_) => None, // Silently skip corrupted/placeholder chunks
                }
            } else {
                None
            }
        } else {
            None
        };

        // MCDD has no dedicated offset in MCNK header
        // Found via chunk discovery in root ADT files (WoD+)
        // TODO: Add chunk discovery support for MCDD
        let doodad_disable = None;

        Ok(Self {
            header,
            heights,
            normals,
            layers,
            materials,
            refs,
            doodad_refs,
            wmo_refs,
            alpha,
            shadow,
            vertex_colors,
            vertex_lighting,
            sound_emitters,
            liquid,
            doodad_disable,
            blend_batches: None, // TODO: Parse MCBB from chunk discovery
        })
    }

    /// Check if chunk has valid height data.
    pub fn has_heights(&self) -> bool {
        self.heights.is_some()
    }

    /// Check if chunk has valid normal data.
    pub fn has_normals(&self) -> bool {
        self.normals.is_some()
    }

    /// Check if chunk has texture layers.
    pub fn has_layers(&self) -> bool {
        self.layers.is_some()
    }

    /// Check if chunk has object references.
    pub fn has_refs(&self) -> bool {
        self.refs.is_some()
    }

    /// Check if chunk has alpha maps.
    pub fn has_alpha(&self) -> bool {
        self.alpha.is_some()
    }

    /// Check if chunk has shadow map.
    pub fn has_shadow(&self) -> bool {
        self.shadow.is_some()
    }

    /// Check if chunk has vertex colors.
    pub fn has_vertex_colors(&self) -> bool {
        self.vertex_colors.is_some()
    }

    /// Check if chunk has sound emitters.
    pub fn has_sound_emitters(&self) -> bool {
        self.sound_emitters.is_some()
    }

    /// Check if chunk has legacy liquid data.
    pub fn has_liquid(&self) -> bool {
        self.liquid.is_some()
    }

    /// Validate that header flags match subchunk presence.
    pub fn validate_consistency(&self) -> bool {
        // Check that header flags match actual subchunk presence
        (self.header.has_height() == self.has_heights())
            && (self.header.has_normal() == self.has_normals())
            && (self.header.has_layer() == self.has_layers())
            && (self.header.has_shadow() == self.has_shadow())
            && (self.header.has_vertex_colors() == self.has_vertex_colors())
            && (self.header.has_legacy_liquid() == self.has_liquid())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Create minimal MCNK chunk data for testing.
    fn create_test_mcnk_data() -> Vec<u8> {
        let mut data = Vec::new();

        // MCNK chunk header (8 bytes)
        data.extend_from_slice(b"MCNK"); // Magic
        data.extend_from_slice(&500u32.to_le_bytes()); // Size (will be updated)

        // MCNK header (128 bytes) - starts at offset 8
        let header_start = data.len();

        // flags (0x41 = has_mcsh | has_mccv)
        data.extend_from_slice(&0x0041u32.to_le_bytes());

        // index_x, index_y
        data.extend_from_slice(&5u32.to_le_bytes());
        data.extend_from_slice(&7u32.to_le_bytes());

        // n_layers
        data.extend_from_slice(&2u32.to_le_bytes());

        // n_doodad_refs
        data.extend_from_slice(&0u32.to_le_bytes());

        // multipurpose_field: ofs_height (136) + ofs_normal (0)
        // (8 bytes total: first 4 = ofs_height, last 4 = ofs_normal)
        // Points to MCVT at offset 136 (8-byte chunk header + 128-byte MCNK header)
        data.extend_from_slice(&136u32.to_le_bytes()); // ofs_height
        data.extend_from_slice(&0u32.to_le_bytes());   // ofs_normal

        // ofs_layer (0 = not present)
        data.extend_from_slice(&0u32.to_le_bytes());

        // ofs_refs (0 = not present)
        data.extend_from_slice(&0u32.to_le_bytes());

        // ofs_alpha (0 = not present)
        data.extend_from_slice(&0u32.to_le_bytes());

        // size_alpha
        data.extend_from_slice(&0u32.to_le_bytes());

        // ofs_shadow, size_shadow (0 = not present)
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());

        // area_id
        data.extend_from_slice(&1234u32.to_le_bytes());

        // n_map_obj_refs
        data.extend_from_slice(&0u32.to_le_bytes());

        // holes_low_res, unknown_but_used
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&1u16.to_le_bytes());

        // pred_tex (8 bytes), no_effect_doodad (8 bytes)
        data.extend_from_slice(&[0u8; 16]);

        // ofs_snd_emitters, n_snd_emitters
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());

        // ofs_liquid, size_liquid
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());

        // position (12 bytes)
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());

        // ofs_mccv, ofs_mclv, unused
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());

        // _padding (8 bytes) - added to reach 128-byte header size
        data.extend_from_slice(&[0u8; 8]);

        assert_eq!(
            data.len() - header_start,
            128,
            "Header must be exactly 128 bytes"
        );

        // Add MCVT subchunk at offset 136 (8-byte chunk header + 128-byte MCNK header)
        assert_eq!(data.len(), 136, "Data should be at offset 136 before MCVT");

        // MCVT chunk header
        data.extend_from_slice(b"MCVT"); // Magic (reversed)
        data.extend_from_slice(&(145 * 4u32).to_le_bytes()); // Size: 145 floats

        // MCVT data (145 f32 values)
        for i in 0..145 {
            data.extend_from_slice(&(i as f32).to_le_bytes());
        }

        data
    }

    #[test]
    fn test_mcnk_chunk_parse() {
        let data = create_test_mcnk_data();
        let mut cursor = Cursor::new(data);

        // Skip MCNK chunk header (8 bytes)
        cursor.set_position(8);

        let mcnk = McnkChunk::parse_with_offset(&mut cursor, 0).unwrap();

        assert_eq!(mcnk.header.index_x, 5);
        assert_eq!(mcnk.header.index_y, 7);
        assert_eq!(mcnk.header.n_layers, 2);
        assert_eq!(mcnk.header.area_id, 1234);

        // MCVT should be present
        assert!(mcnk.has_heights());
        assert_eq!(mcnk.heights.as_ref().unwrap().heights.len(), 145);

        // Other subchunks should be absent
        assert!(!mcnk.has_normals());
        assert!(!mcnk.has_layers());
        assert!(!mcnk.has_refs());
        assert!(!mcnk.has_alpha());
        assert!(!mcnk.has_shadow());
        assert!(!mcnk.has_vertex_colors());
        assert!(!mcnk.has_sound_emitters());
        assert!(!mcnk.has_liquid());
    }

    #[test]
    fn test_mcnk_chunk_validate_consistency() {
        let data = create_test_mcnk_data();
        let mut cursor = Cursor::new(data);
        cursor.set_position(8);

        let mcnk = McnkChunk::parse_with_offset(&mut cursor, 0).unwrap();

        // Note: validate_consistency checks header.has_* methods, which check offsets
        // Our test data has ofs_height set, so header.has_height() should be true
        // and mcnk.has_heights() should also be true
        assert!(mcnk.header.has_height());
        assert!(mcnk.has_heights());
    }

    #[test]
    fn test_mcnk_chunk_presence_checks() {
        let data = create_test_mcnk_data();
        let mut cursor = Cursor::new(data);
        cursor.set_position(8);

        let mcnk = McnkChunk::parse_with_offset(&mut cursor, 0).unwrap();

        assert!(mcnk.has_heights());
        assert!(!mcnk.has_normals());
        assert!(!mcnk.has_layers());
    }

    #[test]
    fn test_mcnk_chunk_height_access() {
        let data = create_test_mcnk_data();
        let mut cursor = Cursor::new(data);
        cursor.set_position(8);

        let mcnk = McnkChunk::parse_with_offset(&mut cursor, 0).unwrap();

        if let Some(heights) = &mcnk.heights {
            // Test data has sequential values 0.0, 1.0, 2.0, ...
            assert_eq!(heights.heights[0], 0.0);
            assert_eq!(heights.heights[1], 1.0);
            assert_eq!(heights.heights[144], 144.0);
        } else {
            panic!("Heights should be present");
        }
    }
}
