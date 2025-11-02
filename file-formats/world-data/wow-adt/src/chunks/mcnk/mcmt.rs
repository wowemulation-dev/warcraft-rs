use binrw::{BinRead, BinWrite};

/// MCMT chunk - Terrain material IDs (Cataclysm+)
///
/// Contains terrain material IDs for each texture layer, referencing the
/// TerrainMaterialRec database table. One material ID per MCLY layer.
///
/// ## Structure
///
/// ```text
/// struct {
///     uint8_t material_id[4];  // Per MCLY layer
/// } MCMT;
/// ```
///
/// ## Split File Architecture
///
/// MCMT chunks appear in _tex.adt files (Cataclysm+), not in root ADT files.
/// They provide material properties for terrain rendering like wetness, footstep
/// sounds, and visual effects.
///
/// **Location**: Split files (tex variant only)
/// **Size**: Fixed 4 bytes
/// **Layer Limit**: Maximum 4 layers (pre-Midnight expansion)
///
/// ## Material Properties
///
/// Each material_id references TerrainMaterialRec::m_ID which defines:
/// - Surface shader properties
/// - Footstep sound effects
/// - Ground effect particles
/// - Wetness behavior
/// - Environmental interaction
///
/// ## Layer Association
///
/// Material IDs correspond one-to-one with MCLY entries:
/// - material_id[0] → MCLY layer 0
/// - material_id[1] → MCLY layer 1
/// - material_id[2] → MCLY layer 2
/// - material_id[3] → MCLY layer 3
///
/// Unused slots are typically 0 or 255.
///
/// ## References
///
/// - **wowdev.wiki**: <https://wowdev.wiki/ADT/v18#MCMT_sub-chunk>
/// - **WoWFormatLib**: Not found in ADT.Struct.cs (implementation gap)
/// - **noggit-red**: Not implemented (Cataclysm+ not supported)
/// - **wow.export**: Not found in ADTLoader.js (implementation gap)
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ❌ Not present
/// - **TBC (2.4.3)**: ❌ Not present
/// - **WotLK (3.3.5a)**: ❌ Not present
/// - **Cataclysm (4.3.4)**: ✅ Introduced in split files
/// - **MoP (5.4.8)**: ✅ Present
///
/// ## Deviations
///
/// None - straightforward 4-byte array matching wowdev.wiki specification.
///
/// Reference: <https://wowdev.wiki/ADT/v18#MCMT_sub-chunk>
#[derive(Debug, Clone, Copy, Default, BinRead, BinWrite)]
#[brw(little)]
pub struct McmtChunk {
    /// Terrain material IDs (per MCLY layer)
    ///
    /// Each value references TerrainMaterialRec::m_ID.
    /// Maximum 4 layers supported.
    /// Unused slots are typically 0 or 255.
    pub material_ids: [u8; 4],
}

impl McmtChunk {
    /// Fixed size in bytes.
    pub const SIZE_BYTES: usize = 4;

    /// Get material ID for a specific layer.
    ///
    /// # Arguments
    ///
    /// * `layer` - Layer index (0-3)
    ///
    /// # Returns
    ///
    /// Material ID for the layer, or None if layer index is invalid
    #[must_use]
    pub fn get_material(&self, layer: usize) -> Option<u8> {
        self.material_ids.get(layer).copied()
    }

    /// Check if a layer has a valid material ID.
    ///
    /// # Arguments
    ///
    /// * `layer` - Layer index (0-3)
    ///
    /// # Returns
    ///
    /// `true` if layer has a non-zero, non-255 material ID
    #[must_use]
    pub fn has_material(&self, layer: usize) -> bool {
        self.get_material(layer)
            .is_some_and(|id| id != 0 && id != 255)
    }

    /// Get number of layers with valid material IDs.
    #[must_use]
    pub fn material_count(&self) -> usize {
        self.material_ids
            .iter()
            .filter(|&&id| id != 0 && id != 255)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::{BinReaderExt, BinWriterExt};
    use std::io::Cursor;

    #[test]
    fn test_mcmt_size() {
        assert_eq!(McmtChunk::SIZE_BYTES, 4);
    }

    #[test]
    fn test_mcmt_parse() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let mut cursor = Cursor::new(data);
        let mcmt: McmtChunk = cursor.read_le().unwrap();

        assert_eq!(mcmt.material_ids[0], 1);
        assert_eq!(mcmt.material_ids[1], 2);
        assert_eq!(mcmt.material_ids[2], 3);
        assert_eq!(mcmt.material_ids[3], 4);
    }

    #[test]
    fn test_mcmt_round_trip() {
        let original = McmtChunk {
            material_ids: [10, 20, 30, 40],
        };

        let mut buffer = Cursor::new(Vec::new());
        buffer.write_le(&original).unwrap();
        assert_eq!(buffer.position(), 4);

        let data = buffer.into_inner();
        let mut cursor = Cursor::new(data);
        let parsed: McmtChunk = cursor.read_le().unwrap();

        assert_eq!(original.material_ids, parsed.material_ids);
    }

    #[test]
    fn test_mcmt_get_material() {
        let mcmt = McmtChunk {
            material_ids: [5, 10, 15, 20],
        };

        assert_eq!(mcmt.get_material(0), Some(5));
        assert_eq!(mcmt.get_material(1), Some(10));
        assert_eq!(mcmt.get_material(2), Some(15));
        assert_eq!(mcmt.get_material(3), Some(20));
        assert_eq!(mcmt.get_material(4), None);
    }

    #[test]
    fn test_mcmt_has_material() {
        let mcmt = McmtChunk {
            material_ids: [5, 0, 255, 20],
        };

        assert!(mcmt.has_material(0)); // 5 is valid
        assert!(!mcmt.has_material(1)); // 0 is invalid
        assert!(!mcmt.has_material(2)); // 255 is invalid
        assert!(mcmt.has_material(3)); // 20 is valid
        assert!(!mcmt.has_material(4)); // out of bounds
    }

    #[test]
    fn test_mcmt_material_count() {
        let mcmt1 = McmtChunk {
            material_ids: [1, 2, 3, 4],
        };
        assert_eq!(mcmt1.material_count(), 4);

        let mcmt2 = McmtChunk {
            material_ids: [1, 0, 3, 255],
        };
        assert_eq!(mcmt2.material_count(), 2);

        let mcmt3 = McmtChunk {
            material_ids: [0, 0, 0, 0],
        };
        assert_eq!(mcmt3.material_count(), 0);
    }

    #[test]
    fn test_mcmt_default() {
        let mcmt = McmtChunk::default();
        assert_eq!(mcmt.material_ids, [0, 0, 0, 0]);
        assert_eq!(mcmt.material_count(), 0);
    }

    #[test]
    fn test_mcmt_all_layers_used() {
        let mcmt = McmtChunk {
            material_ids: [10, 20, 30, 40],
        };

        assert_eq!(mcmt.material_count(), 4);
        for i in 0..4 {
            assert!(mcmt.has_material(i));
        }
    }

    #[test]
    fn test_mcmt_partial_layers() {
        let mcmt = McmtChunk {
            material_ids: [15, 25, 0, 0],
        };

        assert_eq!(mcmt.material_count(), 2);
        assert!(mcmt.has_material(0));
        assert!(mcmt.has_material(1));
        assert!(!mcmt.has_material(2));
        assert!(!mcmt.has_material(3));
    }
}
