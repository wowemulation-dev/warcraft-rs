use binrw::{BinRead, BinWrite};

/// Texture layer flags (32-bit bitfield).
///
/// Controls animation, alpha blending, and reflection effects.
///
/// # Bit Layout (validated against noggit-red)
///
/// ```text
/// Bits  | Mask   | Description
/// ------|--------|------------------------------------------
/// 0-2   | 0x007  | Animation rotation (0-7)
/// 3-5   | 0x038  | Animation speed (0-7)
/// 6     | 0x040  | Animation enabled
/// 7     | 0x080  | Overbright (layer rendered brighter)
/// 8     | 0x100  | Use alpha map for blending
/// 9     | 0x200  | Alpha map compressed (4096 RLE)
/// 10    | 0x400  | Use cube map reflection (environment)
/// 11-31 | ---    | Unused/reserved
/// ```
///
/// Reference: noggit-red `MapChunk.cpp:442-451`, wowdev.wiki ADT/v18#MCLY
#[derive(Debug, Clone, Copy, Default, BinRead, BinWrite, PartialEq, Eq)]
#[brw(little)]
pub struct MclyFlags {
    /// Raw flags value
    pub value: u32,
}

impl MclyFlags {
    /// Get animation rotation speed (0-7).
    ///
    /// Determines texture rotation speed. 0 = no rotation.
    pub fn animation_rotation(&self) -> u8 {
        (self.value & 0x007) as u8
    }

    /// Get animation speed (0-7).
    ///
    /// Determines UV scrolling speed. 0 = no animation.
    pub fn animation_speed(&self) -> u8 {
        ((self.value & 0x038) >> 3) as u8
    }

    /// Animation enabled (texture scrolling).
    ///
    /// When set, texture animates using animation_speed value.
    pub fn animation_enabled(&self) -> bool {
        self.value & 0x040 != 0
    }

    /// Overbright blending enabled.
    ///
    /// Layer is rendered brighter than normal (additive blending).
    pub fn overbright(&self) -> bool {
        self.value & 0x080 != 0
    }

    /// Use alpha map for blending (all layers except first).
    ///
    /// When set, layer uses alpha map from MCAL chunk for blending.
    /// ✅ FIXED: Correct bit position (bit 8, not bit 1)
    pub fn use_alpha_map(&self) -> bool {
        self.value & 0x100 != 0
    }

    /// Alpha map is compressed (RLE format).
    ///
    /// When set, alpha map uses 4096-byte RLE compression instead of raw 2048 bytes.
    /// ✅ FIXED: Correct bit position (bit 9, not bit 2)
    pub fn alpha_map_compressed(&self) -> bool {
        self.value & 0x200 != 0
    }

    /// Use cube map reflection (skybox reflection).
    ///
    /// When set, layer reflects environment/skybox (water, metal surfaces).
    /// ✅ FIXED: Correct bit position (bit 10, not bit 3)
    pub fn use_cube_map_reflection(&self) -> bool {
        self.value & 0x400 != 0
    }
}

/// Single texture layer entry (16 bytes).
///
/// Defines one texture layer for terrain blending. The first layer is always
/// the base texture with no alpha blending. Subsequent layers use alpha maps
/// from the MCAL chunk to blend with previous layers.
///
/// # Binary Layout
///
/// ```text
/// Offset | Size | Field           | Description
/// -------|------|-----------------|----------------------------------
/// 0x00   |  4   | texture_id      | Index into MTEX chunk
/// 0x04   |  4   | flags           | MclyFlags bitfield
/// 0x08   |  4   | offset_in_mcal  | Byte offset into MCAL chunk
/// 0x0C   |  4   | effect_id       | Ground effect/doodad ID
/// ```
///
/// Reference: <https://wowdev.wiki/ADT/v18#MCLY_sub-chunk>
#[derive(Debug, Clone, Copy, Default, BinRead, BinWrite, PartialEq, Eq)]
#[brw(little)]
pub struct MclyLayer {
    /// Texture ID (index into MTEX chunk)
    pub texture_id: u32,

    /// Layer flags
    pub flags: MclyFlags,

    /// Offset into MCAL chunk for alpha map data
    ///
    /// Only valid if `flags.use_alpha_map()` is true.
    /// First layer typically has offset 0 (no alpha map).
    pub offset_in_mcal: u32,

    /// Effect ID for ground effects/doodads
    pub effect_id: u32,
}

/// MCLY chunk - Texture layer definitions (Vanilla+).
///
/// Contains texture layer metadata for terrain rendering. Maximum 4 layers
/// in pre-Midnight clients (8 in Midnight+).
///
/// Layer blending order:
/// 1. Layer 0: Base texture (no blending)
/// 2. Layer 1: Blended over layer 0 using alpha map
/// 3. Layer 2: Blended over layers 0-1 using alpha map
/// 4. Layer 3: Blended over layers 0-2 using alpha map
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ✅ Introduced
/// - **TBC (2.4.3)**: ✅ Present
/// - **WotLK (3.3.5a)**: ✅ Present
/// - **Cataclysm (4.3.4)**: ✅ Present
/// - **MoP (5.4.8)**: ✅ Present
///
/// Reference: <https://wowdev.wiki/ADT/v18#MCLY_sub-chunk>
#[derive(Debug, Clone, Default, BinRead, BinWrite, PartialEq, Eq)]
#[brw(little)]
pub struct MclyChunk {
    /// Texture layers (up to 4 pre-Midnight, 8 in Midnight+)
    #[br(parse_with = binrw::helpers::until_eof)]
    pub layers: Vec<MclyLayer>,
}

impl MclyChunk {
    /// Maximum layer count for pre-Midnight clients.
    pub const MAX_LAYERS_CLASSIC: usize = 4;

    /// Maximum layer count for Midnight+ clients.
    pub const MAX_LAYERS_MIDNIGHT: usize = 8;

    /// Get number of texture layers.
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Check if layer count is valid for classic clients.
    pub fn is_valid_classic(&self) -> bool {
        self.layers.len() <= Self::MAX_LAYERS_CLASSIC
    }

    /// Get base layer (layer 0).
    ///
    /// The base layer never uses alpha blending.
    pub fn base_layer(&self) -> Option<&MclyLayer> {
        self.layers.first()
    }

    /// Get blend layers (layers 1+).
    ///
    /// These layers blend over previous layers using alpha maps.
    pub fn blend_layers(&self) -> &[MclyLayer] {
        if self.layers.len() > 1 {
            &self.layers[1..]
        } else {
            &[]
        }
    }

    /// Validate that blend layers have proper flags.
    ///
    /// Blend layers (1+) should have `use_alpha_map` flag set.
    pub fn validate_blend_flags(&self) -> bool {
        self.blend_layers()
            .iter()
            .all(|layer| layer.flags.use_alpha_map())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::{BinReaderExt, BinWriterExt, io::Cursor};

    #[test]
    fn test_mcly_flags_animation_rotation() {
        let flags = MclyFlags { value: 0x005 }; // Rotation = 5
        assert_eq!(flags.animation_rotation(), 5);
        assert_eq!(flags.animation_speed(), 0);
    }

    #[test]
    fn test_mcly_flags_animation_speed() {
        let flags = MclyFlags { value: 0x018 }; // Speed = 3 (bits 3-5)
        assert_eq!(flags.animation_rotation(), 0);
        assert_eq!(flags.animation_speed(), 3);
    }

    #[test]
    fn test_mcly_flags_animation_enabled() {
        let flags = MclyFlags { value: 0x040 }; // Bit 6
        assert!(flags.animation_enabled());
        assert!(!flags.overbright());
        assert!(!flags.use_alpha_map());
        assert!(!flags.alpha_map_compressed());
        assert!(!flags.use_cube_map_reflection());
    }

    #[test]
    fn test_mcly_flags_overbright() {
        let flags = MclyFlags { value: 0x080 }; // Bit 7
        assert!(!flags.animation_enabled());
        assert!(flags.overbright());
        assert!(!flags.use_alpha_map());
    }

    #[test]
    fn test_mcly_flags_use_alpha_map() {
        let flags = MclyFlags { value: 0x100 }; // Bit 8 (FIXED from 0x02)
        assert!(!flags.animation_enabled());
        assert!(!flags.overbright());
        assert!(flags.use_alpha_map());
        assert!(!flags.alpha_map_compressed());
        assert!(!flags.use_cube_map_reflection());
    }

    #[test]
    fn test_mcly_flags_alpha_map_compressed() {
        let flags = MclyFlags { value: 0x200 }; // Bit 9 (FIXED from 0x04)
        assert!(!flags.animation_enabled());
        assert!(!flags.use_alpha_map());
        assert!(flags.alpha_map_compressed());
        assert!(!flags.use_cube_map_reflection());
    }

    #[test]
    fn test_mcly_flags_use_cube_map_reflection() {
        let flags = MclyFlags { value: 0x400 }; // Bit 10 (FIXED from 0x08)
        assert!(!flags.animation_enabled());
        assert!(!flags.use_alpha_map());
        assert!(!flags.alpha_map_compressed());
        assert!(flags.use_cube_map_reflection());
    }

    #[test]
    fn test_mcly_flags_multiple_bits() {
        let flags = MclyFlags { value: 0x7C0 }; // Bits 6-10
        assert!(flags.animation_enabled());
        assert!(flags.overbright());
        assert!(flags.use_alpha_map());
        assert!(flags.alpha_map_compressed());
        assert!(flags.use_cube_map_reflection());
    }

    #[test]
    fn test_mcly_flags_animation_combined() {
        let flags = MclyFlags { value: 0x07F }; // All animation bits (0-6)
        assert_eq!(flags.animation_rotation(), 7);
        assert_eq!(flags.animation_speed(), 7);
        assert!(flags.animation_enabled());
        assert!(!flags.overbright());
    }

    #[test]
    fn test_mcly_layer_parsing() {
        let data: [u8; 16] = [
            0x05, 0x00, 0x00, 0x00, // texture_id: 5
            0x00, 0x01, 0x00, 0x00, // flags: 0x100 (use_alpha_map, bit 8)
            0x00, 0x08, 0x00, 0x00, // offset_in_mcal: 2048
            0x0A, 0x00, 0x00, 0x00, // effect_id: 10
        ];

        let mut cursor = Cursor::new(&data[..]);
        let layer: MclyLayer = cursor.read_le().expect("Failed to parse MclyLayer");

        assert_eq!(layer.texture_id, 5);
        assert_eq!(layer.flags.value, 0x100);
        assert!(layer.flags.use_alpha_map());
        assert_eq!(layer.offset_in_mcal, 2048);
        assert_eq!(layer.effect_id, 10);
    }

    #[test]
    fn test_mcly_layer_default() {
        let layer = MclyLayer::default();
        assert_eq!(layer.texture_id, 0);
        assert_eq!(layer.flags.value, 0);
        assert_eq!(layer.offset_in_mcal, 0);
        assert_eq!(layer.effect_id, 0);
    }

    #[test]
    fn test_mcly_chunk_single_layer() {
        let data: [u8; 16] = [
            0x01, 0x00, 0x00, 0x00, // texture_id: 1
            0x00, 0x00, 0x00, 0x00, // flags: none (base layer)
            0x00, 0x00, 0x00, 0x00, // offset_in_mcal: 0
            0x00, 0x00, 0x00, 0x00, // effect_id: 0
        ];

        let mut cursor = Cursor::new(&data[..]);
        let chunk: MclyChunk = cursor.read_le().expect("Failed to parse MclyChunk");

        assert_eq!(chunk.layer_count(), 1);
        assert!(chunk.is_valid_classic());
        assert_eq!(chunk.base_layer().unwrap().texture_id, 1);
        assert_eq!(chunk.blend_layers().len(), 0);
    }

    #[test]
    fn test_mcly_chunk_multiple_layers() {
        let mut data = Vec::new();

        // Layer 0: Base texture
        data.extend_from_slice(&[
            0x01, 0x00, 0x00, 0x00, // texture_id: 1
            0x00, 0x00, 0x00, 0x00, // flags: none
            0x00, 0x00, 0x00, 0x00, // offset_in_mcal: 0
            0x00, 0x00, 0x00, 0x00, // effect_id: 0
        ]);

        // Layer 1: Blend layer
        data.extend_from_slice(&[
            0x02, 0x00, 0x00, 0x00, // texture_id: 2
            0x00, 0x01, 0x00, 0x00, // flags: 0x100 (use_alpha_map, bit 8)
            0x00, 0x08, 0x00, 0x00, // offset_in_mcal: 2048
            0x00, 0x00, 0x00, 0x00, // effect_id: 0
        ]);

        // Layer 2: Blend layer
        data.extend_from_slice(&[
            0x03, 0x00, 0x00, 0x00, // texture_id: 3
            0x00, 0x01, 0x00, 0x00, // flags: 0x100 (use_alpha_map, bit 8)
            0x00, 0x10, 0x00, 0x00, // offset_in_mcal: 4096
            0x00, 0x00, 0x00, 0x00, // effect_id: 0
        ]);

        let mut cursor = Cursor::new(&data[..]);
        let chunk: MclyChunk = cursor.read_le().expect("Failed to parse MclyChunk");

        assert_eq!(chunk.layer_count(), 3);
        assert!(chunk.is_valid_classic());
        assert_eq!(chunk.base_layer().unwrap().texture_id, 1);
        assert_eq!(chunk.blend_layers().len(), 2);
        assert_eq!(chunk.blend_layers()[0].texture_id, 2);
        assert_eq!(chunk.blend_layers()[1].texture_id, 3);
    }

    #[test]
    fn test_mcly_chunk_max_layers_classic() {
        let mut data = Vec::new();

        // Create 4 layers (max for classic)
        for i in 0..4 {
            let flags_bytes = if i == 0 {
                [0x00, 0x00, 0x00, 0x00] // Base layer has no flags
            } else {
                [0x00, 0x01, 0x00, 0x00] // 0x100 = use_alpha_map (bit 8)
            };
            data.extend_from_slice(&[
                i as u8, 0x00, 0x00, 0x00, // texture_id
            ]);
            data.extend_from_slice(&flags_bytes); // flags
            data.extend_from_slice(&[
                0x00, 0x00, 0x00, 0x00, // offset_in_mcal
                0x00, 0x00, 0x00, 0x00, // effect_id
            ]);
        }

        let mut cursor = Cursor::new(&data[..]);
        let chunk: MclyChunk = cursor.read_le().expect("Failed to parse MclyChunk");

        assert_eq!(chunk.layer_count(), 4);
        assert!(chunk.is_valid_classic());
        assert_eq!(chunk.blend_layers().len(), 3);
    }

    #[test]
    fn test_mcly_chunk_exceeds_classic_limit() {
        let mut data = Vec::new();

        // Create 5 layers (exceeds classic limit)
        for i in 0..5 {
            let flags_bytes = if i == 0 {
                [0x00, 0x00, 0x00, 0x00]
            } else {
                [0x00, 0x01, 0x00, 0x00] // 0x100 = use_alpha_map
            };
            data.extend_from_slice(&[
                i as u8, 0x00, 0x00, 0x00, // texture_id
            ]);
            data.extend_from_slice(&flags_bytes); // flags
            data.extend_from_slice(&[
                0x00, 0x00, 0x00, 0x00, // offset_in_mcal
                0x00, 0x00, 0x00, 0x00, // effect_id
            ]);
        }

        let mut cursor = Cursor::new(&data[..]);
        let chunk: MclyChunk = cursor.read_le().expect("Failed to parse MclyChunk");

        assert_eq!(chunk.layer_count(), 5);
        assert!(!chunk.is_valid_classic());
    }

    #[test]
    fn test_mcly_validate_blend_flags_valid() {
        let mut data = Vec::new();

        // Layer 0: Base (no alpha map flag)
        data.extend_from_slice(&[
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ]);

        // Layer 1: Blend (with alpha map flag = 0x100)
        data.extend_from_slice(&[
            0x02, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ]);

        let mut cursor = Cursor::new(&data[..]);
        let chunk: MclyChunk = cursor.read_le().expect("Failed to parse MclyChunk");

        assert!(chunk.validate_blend_flags());
    }

    #[test]
    fn test_mcly_validate_blend_flags_invalid() {
        let mut data = Vec::new();

        // Layer 0: Base
        data.extend_from_slice(&[
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ]);

        // Layer 1: Blend but missing alpha map flag (invalid)
        data.extend_from_slice(&[
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ]);

        let mut cursor = Cursor::new(&data[..]);
        let chunk: MclyChunk = cursor.read_le().expect("Failed to parse MclyChunk");

        assert!(!chunk.validate_blend_flags());
    }

    #[test]
    fn test_mcly_layer_round_trip() {
        let original = MclyLayer {
            texture_id: 42,
            flags: MclyFlags { value: 0x0F },
            offset_in_mcal: 1024,
            effect_id: 7,
        };

        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        cursor
            .write_le(&original)
            .expect("Failed to write MclyLayer");

        let mut cursor = Cursor::new(&buffer[..]);
        let parsed: MclyLayer = cursor.read_le().expect("Failed to read MclyLayer");

        assert_eq!(parsed, original);
    }

    #[test]
    fn test_mcly_chunk_round_trip() {
        let mut original = MclyChunk { layers: Vec::new() };

        // Add base layer
        original.layers.push(MclyLayer {
            texture_id: 1,
            flags: MclyFlags { value: 0x00 },
            offset_in_mcal: 0,
            effect_id: 0,
        });

        // Add blend layers
        for i in 2..=4 {
            original.layers.push(MclyLayer {
                texture_id: i,
                flags: MclyFlags { value: 0x100 }, // use_alpha_map (bit 8)
                offset_in_mcal: (i - 1) * 2048,
                effect_id: 0,
            });
        }

        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        cursor
            .write_le(&original)
            .expect("Failed to write MclyChunk");

        let mut cursor = Cursor::new(&buffer[..]);
        let parsed: MclyChunk = cursor.read_le().expect("Failed to read MclyChunk");

        assert_eq!(parsed.layer_count(), original.layer_count());
        assert_eq!(parsed, original);
    }

    #[test]
    fn test_mcly_chunk_empty() {
        let chunk = MclyChunk::default();
        assert_eq!(chunk.layer_count(), 0);
        assert!(chunk.is_valid_classic());
        assert!(chunk.base_layer().is_none());
        assert_eq!(chunk.blend_layers().len(), 0);
        assert!(chunk.validate_blend_flags());
    }

    #[test]
    fn test_mcly_flags_default() {
        let flags = MclyFlags::default();
        assert_eq!(flags.value, 0);
        assert!(!flags.animation_enabled());
        assert!(!flags.use_alpha_map());
        assert!(!flags.alpha_map_compressed());
        assert!(!flags.use_cube_map_reflection());
    }
}
