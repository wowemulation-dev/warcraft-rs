//! MCNK chunk header structure.
//!
//! The MCNK header is 136 bytes and contains metadata for a single terrain chunk
//! (16×16 yards) including tile position, texture layer count, and offsets to subchunks.
//!
//! **Critical:** Subchunk offsets are relative to the beginning of the MCNK chunk
//! (including the 8-byte chunk header), NOT relative to the MCNK chunk data.

use binrw::{BinRead, BinWrite};

/// MCNK chunk flags (32-bit bitfield).
///
/// Controls optional features and data formats within the MCNK chunk.
/// Many flags are version-specific (WotLK+, Cataclysm+, etc.).
///
/// Reference: <https://wowdev.wiki/ADT/v18#MCNK_header>
#[derive(Debug, Clone, Copy, Default, BinRead, BinWrite)]
#[brw(little)]
pub struct McnkFlags {
    /// Raw flags value
    pub value: u32,
}

impl McnkFlags {
    /// Shadow map (MCSH) present
    pub fn has_mcsh(&self) -> bool {
        self.value & 0x01 != 0
    }

    /// Terrain is impassable
    pub fn impassable(&self) -> bool {
        self.value & 0x02 != 0
    }

    /// Contains river liquid
    pub fn has_river(&self) -> bool {
        self.value & 0x04 != 0
    }

    /// Contains ocean liquid
    pub fn has_ocean(&self) -> bool {
        self.value & 0x08 != 0
    }

    /// Contains magma liquid
    pub fn has_magma(&self) -> bool {
        self.value & 0x10 != 0
    }

    /// Contains slime liquid
    pub fn has_slime(&self) -> bool {
        self.value & 0x20 != 0
    }

    /// Vertex colors (MCCV) present (WotLK+)
    pub fn has_mccv(&self) -> bool {
        self.value & 0x40 != 0
    }

    /// Do not fix alpha map (use full 64×64 instead of 63×63)
    /// Bit 15 (0x8000) - When set, alpha maps are full 64×64; when clear, they're 63×63
    pub fn do_not_fix_alpha_map(&self) -> bool {
        self.value & 0x8000 != 0
    }

    /// High-resolution holes (64-bit hole map, ~MoP 5.3+)
    pub fn high_res_holes(&self) -> bool {
        self.value & 0x200 != 0
    }
}

/// MCNK chunk header - 136 bytes of terrain metadata.
///
/// Contains tile coordinates, texture layer info, and offsets to subchunks.
/// Each MCNK represents a 16×16 yard terrain tile within the 533.33333 yard ADT.
///
/// # Binary Layout (Version-Dependent)
///
/// **Vanilla/TBC/WotLK/Cataclysm (Pre-MoP 5.3):**
/// ```text
/// Offset | Size | Field              | Description
/// -------|------|--------------------|---------------------------------
/// 0x00   |  4   | flags              | McnkFlags bitfield
/// 0x04   |  4   | index_x            | Tile X coordinate (0-15)
/// 0x08   |  4   | index_y            | Tile Y coordinate (0-15)
/// 0x0C   |  4   | n_layers           | Texture layer count (max 4)
/// 0x10   |  4   | n_doodad_refs      | M2 model reference count
/// 0x14   |  4   | ofs_height         | MCVT offset (vertex heights)
/// 0x18   |  4   | ofs_normal         | MCNR offset (vertex normals)
/// 0x1C   |  4   | ofs_layer          | MCLY offset (texture layers)
/// 0x20   |  4   | ofs_refs           | MCRF offset (object refs)
/// 0x24   |  4   | ofs_alpha          | MCAL offset (alpha maps)
/// 0x28   |  4   | size_alpha         | MCAL size in bytes
/// 0x2C   |  4   | ofs_shadow         | MCSH offset (shadow map)
/// 0x30   |  4   | size_shadow        | MCSH size in bytes
/// 0x34   |  4   | area_id            | Area ID from AreaTable.dbc
/// 0x38   |  4   | n_map_obj_refs     | WMO reference count
/// 0x3C   |  2   | holes_low_res      | Low-res hole map (16-bit)
/// 0x3E   |  2   | unknown_but_used   | Unknown (set to 1)
/// 0x40   |  8   | predTex[8][8]      | Texture minimap (2-bit/cell)
/// 0x48   |  8   | noEffectDoodad     | Effect flag (1-bit/cell)
/// 0x50   |  4   | ofs_snd_emitters   | MCSE offset
/// 0x54   |  4   | n_snd_emitters     | MCSE entry count
/// 0x58   |  4   | ofs_liquid         | MCLQ offset (legacy)
/// 0x5C   |  4   | size_liquid        | MCLQ size in bytes
/// 0x60   | 12   | position           | World position [X, Y, Z]
/// 0x6C   |  4   | ofs_mccv           | MCCV offset (WotLK+)
/// 0x70   |  4   | ofs_mclv           | MCLV offset (Cata+)
/// 0x74   |  4   | unused             | Padding
/// ```
///
/// **MoP 5.3+ (with high_res_holes flag):**
/// Same as above, but offsets 0x14-0x1B contain `holes_high_res: u64` instead,
/// shifting subsequent fields by 8 bytes.
///
/// **Total:** 0x88 (136) bytes
///
/// # Offset Interpretation
///
/// All `ofs_*` fields are relative to the START of the MCNK chunk (including the
/// 8-byte MCNK chunk header). To read subchunks:
/// 1. Seek to MCNK chunk start + offset
/// 2. Read subchunk header (magic + size)
/// 3. Parse subchunk data
///
/// Reference: <https://wowdev.wiki/ADT/v18#MCNK_header>
#[derive(Debug, Clone, BinRead, BinWrite)]
#[brw(little)]
pub struct McnkHeader {
    /// Chunk flags
    pub flags: McnkFlags,

    /// Tile X index (0-15 within ADT)
    pub index_x: u32,

    /// Tile Y index (0-15 within ADT)
    pub index_y: u32,

    /// Number of texture layers (max 4 pre-Midnight)
    pub n_layers: u32,

    /// Number of M2 doodad references
    pub n_doodad_refs: u32,

    /// Multi-purpose 8-byte field at offset 0x14-0x1B
    ///
    /// Interpretation depends on `flags.high_res_holes()`:
    /// - **If false (Vanilla/TBC/WotLK/Cata)**: First 4 bytes = ofs_height, last 4 bytes = ofs_normal
    /// - **If true (MoP 5.3+)**: All 8 bytes = holes_high_res bitmap
    ///
    /// Use `ofs_height()` and `ofs_normal()` helper methods for version-aware access.
    /// Use `holes_high_res()` helper method to get the holes bitmap when applicable.
    pub multipurpose_field: [u8; 8],

    /// Offset to MCLY chunk (texture layers)
    ///
    /// Relative to MCNK chunk start. Each layer is 16 bytes.
    pub ofs_layer: u32,

    /// Offset to MCRF chunk (object references)
    ///
    /// Relative to MCNK chunk start. Pre-Cataclysm format.
    pub ofs_refs: u32,

    /// Offset to MCAL chunk (alpha maps)
    ///
    /// Relative to MCNK chunk start. Multiple compression formats possible.
    pub ofs_alpha: u32,

    /// Size of MCAL chunk data in bytes
    pub size_alpha: u32,

    /// Offset to MCSH chunk (shadow map)
    ///
    /// Relative to MCNK chunk start. 512 bytes (64×64 1-bit map).
    /// Only present if `flags.has_mcsh()` is true.
    pub ofs_shadow: u32,

    /// Size of MCSH chunk data in bytes
    pub size_shadow: u32,

    /// Area ID from AreaTable.dbc
    pub area_id: u32,

    /// Number of WMO map object references
    pub n_map_obj_refs: u32,

    /// Low-resolution hole map (16-bit, 4×4 grid)
    ///
    /// Each bit represents one sub-region. Bit set = hole.
    pub holes_low_res: u16,

    /// Unknown field (typically set to 1)
    pub unknown_but_used: u16,

    /// Predominant texture per cell (2 bits per cell, 8×8 grid)
    ///
    /// Packed into 8 bytes (64 bits total). Used for minimap rendering.
    /// Access: `(pred_tex[row] >> (col * 2)) & 0x3`
    pub pred_tex: [u8; 8],

    /// No-effect doodad flag (1 bit per cell, 8×8 grid)
    ///
    /// Packed into 8 bytes (64 bits total). Controls doodad ground effects.
    /// Access: `(no_effect_doodad[row] >> col) & 0x1`
    pub no_effect_doodad: [u8; 8],

    /// Unknown 8-byte field (empirically discovered)
    ///
    /// These 8 bytes appear between no_effect_doodad and ofs_snd_emitters
    /// in actual game files. Purpose unknown. May be related to sound or liquid.
    /// Without this field, ofs_liquid and size_liquid are misaligned by 8 bytes.
    pub unknown_8bytes: [u8; 8],

    /// Offset to MCSE chunk (sound emitters)
    ///
    /// Relative to MCNK chunk start. Each entry is 28 bytes (modern).
    ///
    /// **Version:** MCSE existence in Vanilla is uncertain. Some Vanilla files
    /// may have garbage values here. Use `has_sound_emitters()` to check validity.
    #[br(map = |x: u32| if x >= 0x100000 { 0 } else { x })]
    pub ofs_snd_emitters: u32,

    /// Number of sound emitters
    #[br(map = |x: u32| if x >= 0x10000 { 0 } else { x })]
    pub n_snd_emitters: u32,

    /// Offset to MCLQ chunk (legacy liquid, pre-WotLK)
    ///
    /// Relative to MCNK chunk start. Replaced by MH2O chunk in WotLK+.
    #[br(map = |x: u32| if x >= 0x100000 { 0 } else { x })]
    pub ofs_liquid: u32,

    /// Size of MCLQ chunk data in bytes
    #[br(map = |x: u32| if x >= 0x100000 { 0 } else { x })]
    pub size_liquid: u32,

    /// World position of chunk stored as [Z, X, Y] in file
    ///
    /// **IMPORTANT:** The file format stores this as [Z, X, Y] (verified against noggit-red).
    /// Use the `world_position()` helper method to get [X, Y, Z] coordinates.
    /// Vertex heights are relative to this position.
    ///
    /// Raw field access:
    /// - `position[0]` = Z (vertical height)
    /// - `position[1]` = X (north/south)
    /// - `position[2]` = Y (west/east)
    pub position: [f32; 3],

    /// Offset to MCCV chunk (vertex colors, WotLK 3.x+)
    ///
    /// Relative to MCNK chunk start. 145 BGRA color entries.
    /// Only present if `flags.has_mccv()` is true.
    ///
    /// **Version:** This field only exists in WotLK 3.x+ files. In Vanilla/TBC files,
    /// the header ends at offset 0x6C (108 bytes) after the `position` field.
    /// Reading this field from Vanilla files will return garbage data.
    #[br(map = |x: u32| if x >= 0x1000000 { 0 } else { x })]
    pub ofs_mccv: u32,

    /// Offset to MCLV chunk (baked lighting, Cataclysm 4.x+)
    ///
    /// Relative to MCNK chunk start. 145 ARGB light values.
    ///
    /// **Version:** This field only exists in Cataclysm 4.x+ files. In Vanilla/TBC/WotLK files,
    /// reading this field will return garbage data.
    #[br(map = |x: u32| if x >= 0x1000000 { 0 } else { x })]
    pub ofs_mclv: u32,

    /// Unused padding (always 0, Cataclysm 4.x+)
    ///
    /// **Version:** This field only exists in Cataclysm 4.x+ files.
    #[br(map = |x: u32| if x >= 0x1000000 { 0 } else { x })]
    pub unused: u32,

    /// Padding to reach 128-byte alignment
    ///
    /// The MCNK header is specified as 136 bytes (0x88) in wowdev.wiki.
    /// These 8 bytes of padding ensure the header matches the expected size.
    #[br(pad_before = 0)]
    pub _padding: [u8; 8],
}

impl McnkHeader {
    /// Create multipurpose_field from ofs_height and ofs_normal (Vanilla/TBC/WotLK/Cata).
    pub fn multipurpose_from_offsets(ofs_height: u32, ofs_normal: u32) -> [u8; 8] {
        let mut field = [0u8; 8];
        field[0..4].copy_from_slice(&ofs_height.to_le_bytes());
        field[4..8].copy_from_slice(&ofs_normal.to_le_bytes());
        field
    }

    /// Create multipurpose_field from holes_high_res bitmap (MoP 5.3+).
    pub fn multipurpose_from_holes(holes: u64) -> [u8; 8] {
        holes.to_le_bytes()
    }

    /// Get the offset to MCVT (height) chunk, version-aware.
    ///
    /// Interprets the first 4 bytes of `multipurpose_field` as ofs_height (little-endian u32).
    pub fn ofs_height(&self) -> u32 {
        u32::from_le_bytes([
            self.multipurpose_field[0],
            self.multipurpose_field[1],
            self.multipurpose_field[2],
            self.multipurpose_field[3],
        ])
    }

    /// Get the offset to MCNR (normal) chunk, version-aware.
    ///
    /// Interprets the last 4 bytes of `multipurpose_field` as ofs_normal (little-endian u32).
    pub fn ofs_normal(&self) -> u32 {
        u32::from_le_bytes([
            self.multipurpose_field[4],
            self.multipurpose_field[5],
            self.multipurpose_field[6],
            self.multipurpose_field[7],
        ])
    }

    /// Get the high-resolution holes bitmap (MoP 5.3+ only).
    ///
    /// Only valid if `flags.high_res_holes()` is true.
    /// Interprets all 8 bytes of `multipurpose_field` as a u64 bitmap.
    pub fn holes_high_res(&self) -> Option<u64> {
        if self.flags.high_res_holes() {
            Some(u64::from_le_bytes(self.multipurpose_field))
        } else {
            None
        }
    }

    /// Check if MCVT (height) subchunk offset is present and valid in header.
    ///
    /// Returns false for:
    /// - MoP 5.3+ when `high_res_holes` flag is set (multipurpose_field contains holes)
    /// - Zero offset (no data)
    /// - Garbage offsets (> 256KB, typical max MCNK size)
    ///
    /// In cases where this returns false, MCVT must be found by scanning chunk data.
    pub fn has_height(&self) -> bool {
        // When high_res_holes is set, multipurpose_field contains holes, not offsets
        if self.flags.high_res_holes() {
            return false;
        }
        let ofs = self.ofs_height();
        // Offset 0 means no data; > 256KB is likely garbage (typical MCNK is 1-64KB)
        ofs != 0 && ofs < 0x40000
    }

    /// Check if MCNR (normal) subchunk offset is present and valid in header.
    ///
    /// Returns false for:
    /// - MoP 5.3+ when `high_res_holes` flag is set (multipurpose_field contains holes)
    /// - Zero offset (no data)
    /// - Garbage offsets (> 256KB, typical max MCNK size)
    ///
    /// In cases where this returns false, MCNR must be found by scanning chunk data.
    pub fn has_normal(&self) -> bool {
        // When high_res_holes is set, multipurpose_field contains holes, not offsets
        if self.flags.high_res_holes() {
            return false;
        }
        let ofs = self.ofs_normal();
        // Offset 0 means no data; > 256KB is likely garbage (typical MCNK is 1-64KB)
        ofs != 0 && ofs < 0x40000
    }

    /// Check if MCLY (layer) subchunk is present.
    ///
    /// Note: In some early Vanilla files, `n_layers` may be 0 even when layers exist.
    /// We also check `ofs_layer != 0` to handle these cases.
    pub fn has_layer(&self) -> bool {
        self.ofs_layer != 0
    }

    /// Check if MCAL (alpha) subchunk is present.
    pub fn has_alpha(&self) -> bool {
        self.ofs_alpha != 0 && self.size_alpha > 0
    }

    /// Check if MCSH (shadow) subchunk is present.
    ///
    /// Note: In early Vanilla files, the `has_mcsh` flag may not be set even when
    /// MCSH chunks exist. We check `ofs_shadow` and `size_shadow` instead for
    /// compatibility with all file versions.
    pub fn has_shadow(&self) -> bool {
        self.ofs_shadow != 0 && self.size_shadow > 0
    }

    /// Get the world position in correct [X, Y, Z] order.
    ///
    /// The file stores position as [Z, X, Y], but this method returns [X, Y, Z]
    /// for correct world-space coordinates.
    ///
    /// # Returns
    ///
    /// - `[0]` = X (north/south)
    /// - `[1]` = Y (west/east)
    /// - `[2]` = Z (vertical height)
    pub fn world_position(&self) -> [f32; 3] {
        [
            self.position[1], // X from file position[1]
            self.position[2], // Y from file position[2]
            self.position[0], // Z from file position[0]
        ]
    }

    /// Check if MCRF (refs) subchunk is present.
    pub fn has_refs(&self) -> bool {
        self.ofs_refs != 0 && (self.n_doodad_refs > 0 || self.n_map_obj_refs > 0)
    }

    /// Check if MCCV (vertex colors) subchunk is present.
    pub fn has_vertex_colors(&self) -> bool {
        self.flags.has_mccv() && self.ofs_mccv != 0
    }

    /// Check if MCLV (baked lighting) subchunk is present (Cataclysm+).
    pub fn has_baked_lighting(&self) -> bool {
        self.ofs_mclv != 0
    }

    /// Check if MCSE (sound emitters) subchunk is present.
    pub fn has_sound_emitters(&self) -> bool {
        self.ofs_snd_emitters != 0 && self.n_snd_emitters > 0
    }

    /// Check if MCLQ (legacy liquid) subchunk is present.
    pub fn has_legacy_liquid(&self) -> bool {
        self.ofs_liquid != 0 && self.size_liquid > 0
    }

    /// Get hole status at grid position (4×4 low-res grid).
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-3)
    /// * `y` - Row index (0-3)
    ///
    /// # Returns
    ///
    /// `true` if there's a hole at this position
    pub fn is_hole_low_res(&self, x: usize, y: usize) -> bool {
        if x >= 4 || y >= 4 {
            return false;
        }
        let bit_index = y * 4 + x;
        (self.holes_low_res >> bit_index) & 1 != 0
    }

    /// Get hole status at grid position (8×8 high-res grid).
    ///
    /// Only valid if `flags.high_res_holes()` is true.
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-7)
    /// * `y` - Row index (0-7)
    ///
    /// # Returns
    ///
    /// `true` if there's a hole at this position
    pub fn is_hole_high_res(&self, x: usize, y: usize) -> bool {
        if !self.flags.high_res_holes() || x >= 8 || y >= 8 {
            return false;
        }

        // Only available in MoP 5.3+
        if let Some(holes) = self.holes_high_res() {
            let byte_index = y;
            let bit_index = x;
            let byte = ((holes >> (byte_index * 8)) & 0xFF) as u8;
            (byte >> bit_index) & 1 != 0
        } else {
            false
        }
    }

    /// Get predominant texture index at grid position (8×8 grid).
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-7)
    /// * `y` - Row index (0-7)
    ///
    /// # Returns
    ///
    /// Texture index (0-3) for minimap rendering
    pub fn get_pred_texture(&self, x: usize, y: usize) -> u8 {
        if x >= 8 || y >= 8 {
            return 0;
        }
        let byte = self.pred_tex[y];
        (byte >> (x * 2)) & 0x3
    }

    /// Check if doodad effects are disabled at grid position (8×8 grid).
    ///
    /// # Arguments
    ///
    /// * `x` - Column index (0-7)
    /// * `y` - Row index (0-7)
    ///
    /// # Returns
    ///
    /// `true` if doodad ground effects should be disabled
    pub fn is_no_effect_doodad(&self, x: usize, y: usize) -> bool {
        if x >= 8 || y >= 8 {
            return false;
        }
        let byte = self.no_effect_doodad[y];
        (byte >> x) & 1 != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_mcnk_flags() {
        // 0x8049 = has_mcsh (0x01) | has_ocean (0x08) | has_mccv (0x40) | do_not_fix_alpha_map (0x8000)
        let flags = McnkFlags { value: 0x8049 };

        assert!(flags.has_mcsh());
        assert!(!flags.impassable());
        assert!(!flags.has_river());
        assert!(flags.has_ocean());
        assert!(!flags.has_magma());
        assert!(!flags.has_slime());
        assert!(flags.has_mccv());
        assert!(flags.do_not_fix_alpha_map());
        assert!(!flags.high_res_holes());
    }

    #[test]
    fn test_mcnk_header_serialized_size() {
        use binrw::BinWriterExt;
        use std::io::Cursor;

        // Test Vanilla/TBC/WotLK header (no high_res_holes)
        let vanilla_header = McnkHeader {
            flags: McnkFlags { value: 0 },
            index_x: 0,
            index_y: 0,
            n_layers: 0,
            n_doodad_refs: 0,
            multipurpose_field: McnkHeader::multipurpose_from_offsets(0, 0),
            ofs_layer: 0,
            ofs_refs: 0,
            ofs_alpha: 0,
            size_alpha: 0,
            ofs_shadow: 0,
            size_shadow: 0,
            area_id: 0,
            n_map_obj_refs: 0,
            holes_low_res: 0,
            unknown_but_used: 1,
            pred_tex: [0; 8],
            no_effect_doodad: [0; 8],
            unknown_8bytes: [0; 8],
            ofs_snd_emitters: 0,
            n_snd_emitters: 0,
            ofs_liquid: 0,
            size_liquid: 0,
            position: [0.0; 3],
            ofs_mccv: 0,
            ofs_mclv: 0,
            unused: 0,
            _padding: [0; 8],
        };

        eprintln!(
            "Size in memory: {} bytes",
            std::mem::size_of::<McnkHeader>()
        );
        let mut buffer = Cursor::new(Vec::new());
        buffer.write_le(&vanilla_header).unwrap();
        eprintln!("Serialized {} bytes", buffer.get_ref().len());
        eprintln!("All bytes: {:02x?}", buffer.get_ref());
        assert_eq!(
            buffer.get_ref().len(),
            136,
            "McnkHeader serializes to 136 bytes (including _padding)"
        );

        // Test MoP 5.3+ header (with high_res_holes)
        let mop_header = McnkHeader {
            flags: McnkFlags { value: 0x200 }, // high_res_holes flag
            index_x: 0,
            index_y: 0,
            n_layers: 0,
            n_doodad_refs: 0,
            multipurpose_field: McnkHeader::multipurpose_from_holes(0),
            ofs_layer: 0,
            ofs_refs: 0,
            ofs_alpha: 0,
            size_alpha: 0,
            ofs_shadow: 0,
            size_shadow: 0,
            area_id: 0,
            n_map_obj_refs: 0,
            holes_low_res: 0,
            unknown_but_used: 1,
            pred_tex: [0; 8],
            no_effect_doodad: [0; 8],
            unknown_8bytes: [0; 8],
            ofs_snd_emitters: 0,
            n_snd_emitters: 0,
            ofs_liquid: 0,
            size_liquid: 0,
            position: [0.0; 3],
            ofs_mccv: 0,
            ofs_mclv: 0,
            unused: 0,
            _padding: [0; 8],
        };

        let mut buffer = Cursor::new(Vec::new());
        buffer.write_le(&mop_header).unwrap();
        assert_eq!(
            buffer.get_ref().len(),
            136,
            "McnkHeader serializes to 136 bytes (including _padding)"
        );
    }

    #[test]
    fn test_mcnk_header_parse() {
        let mut data = vec![0u8; 136];

        // flags: 0x41 (has_mcsh | has_mccv)
        data[0] = 0x41;
        data[1] = 0x00;
        data[2] = 0x00;
        data[3] = 0x00;

        // index_x: 5
        data[4] = 0x05;
        data[7] = 0x00;

        // index_y: 7
        data[8] = 0x07;

        // n_layers: 3
        data[12] = 0x03;

        // n_doodad_refs: 10
        data[16] = 0x0A;

        // ofs_height: 0x100 (offset 0x14 = 20, first 4 bytes of multipurpose_field)
        data[20] = 0x00;
        data[21] = 0x01;
        data[22] = 0x00;
        data[23] = 0x00;

        // area_id: 1234 (offset 0x34 = 52)
        data[52] = 0xD2;
        data[53] = 0x04;
        data[54] = 0x00;
        data[55] = 0x00;

        // holes_low_res: 0x000F (4 holes in first row, offset 0x3C = 60)
        data[60] = 0x0F;
        data[61] = 0x00;

        let mut cursor = Cursor::new(data);
        let header = McnkHeader::read_le(&mut cursor).unwrap();

        assert_eq!(header.flags.value, 0x41);
        assert!(header.flags.has_mcsh());
        assert!(header.flags.has_mccv());
        assert_eq!(header.index_x, 5);
        assert_eq!(header.index_y, 7);
        assert_eq!(header.n_layers, 3);
        assert_eq!(header.n_doodad_refs, 10);
        assert_eq!(header.ofs_height(), 0x100);
        assert_eq!(header.area_id, 1234);
        assert_eq!(header.holes_low_res, 0x000F);
    }

    #[test]
    fn test_mcnk_header_has_checks() {
        let header = McnkHeader {
            flags: McnkFlags { value: 0x41 }, // has_mcsh | has_mccv
            index_x: 0,
            index_y: 0,
            n_layers: 2,
            n_doodad_refs: 5,
            multipurpose_field: McnkHeader::multipurpose_from_offsets(0x80, 0x200),
            ofs_layer: 0x300,
            ofs_refs: 0x400,
            ofs_alpha: 0x500,
            size_alpha: 1024,
            ofs_shadow: 0x600,
            size_shadow: 512,
            area_id: 1,
            n_map_obj_refs: 3,
            holes_low_res: 0,
            unknown_but_used: 1,
            pred_tex: [0; 8],
            no_effect_doodad: [0; 8],
            unknown_8bytes: [0; 8],
            ofs_snd_emitters: 0x700,
            n_snd_emitters: 2,
            ofs_liquid: 0,
            size_liquid: 0,
            position: [0.0; 3],
            ofs_mccv: 0x800,
            ofs_mclv: 0,
            unused: 0,
            _padding: [0; 8],
        };

        assert!(header.has_height());
        assert!(header.has_normal());
        assert!(header.has_layer());
        assert!(header.has_alpha());
        assert!(header.has_shadow());
        assert!(header.has_refs());
        assert!(header.has_vertex_colors());
        assert!(!header.has_baked_lighting());
        assert!(header.has_sound_emitters());
        assert!(!header.has_legacy_liquid());
    }

    #[test]
    fn test_hole_low_res() {
        let header = McnkHeader {
            flags: McnkFlags::default(),
            index_x: 0,
            index_y: 0,
            n_layers: 0,
            n_doodad_refs: 0,
            multipurpose_field: McnkHeader::multipurpose_from_offsets(0, 0),
            ofs_layer: 0,
            ofs_refs: 0,
            ofs_alpha: 0,
            size_alpha: 0,
            ofs_shadow: 0,
            size_shadow: 0,
            area_id: 0,
            n_map_obj_refs: 0,
            holes_low_res: 0b1000_0100_0010_0001, // Diagonal holes
            unknown_but_used: 1,
            pred_tex: [0; 8],
            no_effect_doodad: [0; 8],
            unknown_8bytes: [0; 8],
            ofs_snd_emitters: 0,
            n_snd_emitters: 0,
            ofs_liquid: 0,
            size_liquid: 0,
            position: [0.0; 3],
            ofs_mccv: 0,
            ofs_mclv: 0,
            unused: 0,
            _padding: [0; 8],
        };

        assert!(header.is_hole_low_res(0, 0)); // bit 0
        assert!(!header.is_hole_low_res(1, 0));
        assert!(header.is_hole_low_res(1, 1)); // bit 5
        assert!(header.is_hole_low_res(2, 2)); // bit 10
        assert!(header.is_hole_low_res(3, 3)); // bit 15
    }

    #[test]
    fn test_hole_high_res() {
        let header = McnkHeader {
            flags: McnkFlags { value: 0x200 }, // high_res_holes
            index_x: 0,
            index_y: 0,
            n_layers: 0,
            n_doodad_refs: 0,
            multipurpose_field: McnkHeader::multipurpose_from_holes(0x0102030405060708),
            ofs_layer: 0,
            ofs_refs: 0,
            ofs_alpha: 0,
            size_alpha: 0,
            ofs_shadow: 0,
            size_shadow: 0,
            area_id: 0,
            n_map_obj_refs: 0,
            holes_low_res: 0,
            unknown_but_used: 1,
            pred_tex: [0; 8],
            no_effect_doodad: [0; 8],
            unknown_8bytes: [0; 8],
            ofs_snd_emitters: 0,
            n_snd_emitters: 0,
            ofs_liquid: 0,
            size_liquid: 0,
            position: [0.0; 3],
            ofs_mccv: 0,
            ofs_mclv: 0,
            unused: 0,
            _padding: [0; 8],
        };

        // Row 0: byte 0x08 = 0b00001000
        assert!(!header.is_hole_high_res(0, 0));
        assert!(!header.is_hole_high_res(1, 0));
        assert!(!header.is_hole_high_res(2, 0));
        assert!(header.is_hole_high_res(3, 0));

        // Row 1: byte 0x07 = 0b00000111
        assert!(header.is_hole_high_res(0, 1));
        assert!(header.is_hole_high_res(1, 1));
        assert!(header.is_hole_high_res(2, 1));
    }

    #[test]
    fn test_pred_texture() {
        let header = McnkHeader {
            flags: McnkFlags::default(),
            index_x: 0,
            index_y: 0,
            n_layers: 0,
            n_doodad_refs: 0,
            multipurpose_field: McnkHeader::multipurpose_from_offsets(0, 0),
            ofs_layer: 0,
            ofs_refs: 0,
            ofs_alpha: 0,
            size_alpha: 0,
            ofs_shadow: 0,
            size_shadow: 0,
            area_id: 0,
            n_map_obj_refs: 0,
            holes_low_res: 0,
            unknown_but_used: 1,
            pred_tex: [
                0b11_10_01_00, // Row 0: tex 0,1,2,3 in cols 0-3
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
            ],
            no_effect_doodad: [0; 8],
            unknown_8bytes: [0; 8],
            ofs_snd_emitters: 0,
            n_snd_emitters: 0,
            ofs_liquid: 0,
            size_liquid: 0,
            position: [0.0; 3],
            ofs_mccv: 0,
            ofs_mclv: 0,
            unused: 0,
            _padding: [0; 8],
        };

        assert_eq!(header.get_pred_texture(0, 0), 0);
        assert_eq!(header.get_pred_texture(1, 0), 1);
        assert_eq!(header.get_pred_texture(2, 0), 2);
        assert_eq!(header.get_pred_texture(3, 0), 3);
    }

    #[test]
    fn test_no_effect_doodad() {
        let header = McnkHeader {
            flags: McnkFlags::default(),
            index_x: 0,
            index_y: 0,
            n_layers: 0,
            n_doodad_refs: 0,
            multipurpose_field: McnkHeader::multipurpose_from_offsets(0, 0),
            ofs_layer: 0,
            ofs_refs: 0,
            ofs_alpha: 0,
            size_alpha: 0,
            ofs_shadow: 0,
            size_shadow: 0,
            area_id: 0,
            n_map_obj_refs: 0,
            holes_low_res: 0,
            unknown_but_used: 1,
            pred_tex: [0; 8],
            no_effect_doodad: [
                0b10101010, // Row 0: alternating pattern
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            unknown_8bytes: [0; 8],
            ofs_snd_emitters: 0,
            n_snd_emitters: 0,
            ofs_liquid: 0,
            size_liquid: 0,
            position: [0.0; 3],
            ofs_mccv: 0,
            ofs_mclv: 0,
            unused: 0,
            _padding: [0; 8],
        };

        assert!(!header.is_no_effect_doodad(0, 0));
        assert!(header.is_no_effect_doodad(1, 0));
        assert!(!header.is_no_effect_doodad(2, 0));
        assert!(header.is_no_effect_doodad(3, 0));
    }

    #[test]
    fn test_mcnk_header_round_trip() {
        let original = McnkHeader {
            flags: McnkFlags { value: 0x341 }, // Set high_res_holes flag (0x200) + original flags
            index_x: 12,
            index_y: 8,
            n_layers: 4,
            n_doodad_refs: 15,
            multipurpose_field: McnkHeader::multipurpose_from_holes(0x123456789ABCDEF0),
            ofs_layer: 0x500,
            ofs_refs: 0x600,
            ofs_alpha: 0x700,
            size_alpha: 2048,
            ofs_shadow: 0x900,
            size_shadow: 512,
            area_id: 42,
            n_map_obj_refs: 8,
            holes_low_res: 0xF0F0,
            unknown_but_used: 1,
            pred_tex: [1, 2, 3, 4, 5, 6, 7, 8],
            no_effect_doodad: [0xFF, 0, 0, 0, 0, 0, 0, 0],
            unknown_8bytes: [0; 8],
            ofs_snd_emitters: 0xA00,
            n_snd_emitters: 3,
            ofs_liquid: 0xB00,
            size_liquid: 1024,
            position: [1024.0, 32.0, 768.0],
            ofs_mccv: 0xC00,
            ofs_mclv: 0xD00,
            unused: 0,
            _padding: [0; 8],
        };

        let mut buffer = Cursor::new(Vec::new());
        original.write_le(&mut buffer).unwrap();

        let data = buffer.into_inner();
        assert_eq!(data.len(), 136);

        let mut cursor = Cursor::new(data);
        let parsed = McnkHeader::read_le(&mut cursor).unwrap();

        assert_eq!(original.flags.value, parsed.flags.value);
        assert_eq!(original.index_x, parsed.index_x);
        assert_eq!(original.index_y, parsed.index_y);
        assert_eq!(original.n_layers, parsed.n_layers);
        assert_eq!(original.position, parsed.position);
    }
}
