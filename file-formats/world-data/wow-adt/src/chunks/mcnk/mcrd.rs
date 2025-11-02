use binrw::{BinRead, BinWrite};

/// MCRD chunk - Doodad references for split files (Cataclysm+)
///
/// Contains indices into the MDDF chunk, specifying which doodads (M2 models)
/// appear within this terrain chunk. Replaces the doodad portion of MCRF in
/// split file architecture.
///
/// ## Structure
///
/// ```text
/// struct {
///     uint32_t doodad_refs[];  // Variable length
/// } MCRD;
/// ```
///
/// Each value is an index into the file's MDDF chunk array.
///
/// ## Split File Architecture
///
/// - **Pre-Cataclysm**: MCRF contained both doodad and WMO references
/// - **Cataclysm+**: Split into MCRD (doodads) and MCRW (WMOs) in _obj files
///
/// This separation allows the client to load object references independently
/// from terrain geometry, improving streaming performance.
///
/// ## Size Category Sorting
///
/// When WDT flag 0x0008 (`adt_has_doodadrefs_sorted_by_size_cat`) is set,
/// entries must be sorted by size category for culling optimization.
///
/// Size categories (default limits):
/// - Tiny: < 1.0 units
/// - Small: 1.0 - 4.0 units
/// - Medium: 4.0 - 25.0 units
/// - Large: 25.0 - 100.0 units
/// - Huge: >= 100.0 units
///
/// Note: Size category ≠ actual size. Categories are based on bounding box
/// dimensions defined in the M2 model files.
///
/// ## References
///
/// - **wowdev.wiki**: <https://wowdev.wiki/ADT/v18#MCRD_sub-chunk>
/// - **WoWFormatLib**: Not found in ADT.Struct.cs (implementation gap)
/// - **noggit-red**: Not implemented (Cataclysm+ not supported)
/// - **wow.export**: Not found in ADTLoader.js (implementation gap)
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ❌ Not present
/// - **TBC (2.4.3)**: ❌ Not present
/// - **WotLK (3.3.5a)**: ❌ Not present
/// - **Cataclysm (4.3.4)**: ✅ Introduced (replaces MCRF for doodads)
/// - **MoP (5.4.8)**: ✅ Present
///
/// ## Deviations
///
/// None - straightforward u32 array matching wowdev.wiki specification.
///
/// Reference: <https://wowdev.wiki/ADT/v18#MCRD_sub-chunk>
#[derive(Debug, Clone, Default, BinRead, BinWrite)]
#[brw(little)]
pub struct McrdChunk {
    /// Doodad reference indices (MDDF entry indices)
    ///
    /// Each value is an index into the MDDF placement array.
    /// Variable length - parse until end of chunk.
    #[br(parse_with = binrw::helpers::until_eof)]
    pub doodad_refs: Vec<u32>,
}

impl McrdChunk {
    /// Get number of doodad references.
    #[must_use]
    pub fn count(&self) -> usize {
        self.doodad_refs.len()
    }

    /// Check if a specific MDDF index is referenced.
    ///
    /// # Arguments
    ///
    /// * `mddf_index` - Index into MDDF placement array
    ///
    /// # Returns
    ///
    /// `true` if this chunk references the doodad at `mddf_index`
    #[must_use]
    pub fn contains_doodad(&self, mddf_index: u32) -> bool {
        self.doodad_refs.contains(&mddf_index)
    }

    /// Validate that all references are within MDDF bounds.
    ///
    /// # Arguments
    ///
    /// * `mddf_count` - Total number of entries in MDDF chunk
    ///
    /// # Returns
    ///
    /// `true` if all indices are valid
    #[must_use]
    pub fn validate_refs(&self, mddf_count: usize) -> bool {
        self.doodad_refs
            .iter()
            .all(|&idx| (idx as usize) < mddf_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::{BinReaderExt, BinWriterExt};
    use std::io::Cursor;

    #[test]
    fn test_mcrd_parse() {
        // 5 doodad references
        let data = vec![
            0x00, 0x00, 0x00, 0x00, // ref[0] = 0
            0x05, 0x00, 0x00, 0x00, // ref[1] = 5
            0x0A, 0x00, 0x00, 0x00, // ref[2] = 10
            0x14, 0x00, 0x00, 0x00, // ref[3] = 20
            0x32, 0x00, 0x00, 0x00, // ref[4] = 50
        ];

        let mut cursor = Cursor::new(data);
        let mcrd: McrdChunk = cursor.read_le().unwrap();

        assert_eq!(mcrd.count(), 5);
        assert_eq!(mcrd.doodad_refs[0], 0);
        assert_eq!(mcrd.doodad_refs[1], 5);
        assert_eq!(mcrd.doodad_refs[2], 10);
        assert_eq!(mcrd.doodad_refs[3], 20);
        assert_eq!(mcrd.doodad_refs[4], 50);
    }

    #[test]
    fn test_mcrd_empty() {
        let data = vec![];
        let mut cursor = Cursor::new(data);
        let mcrd: McrdChunk = cursor.read_le().unwrap();

        assert_eq!(mcrd.count(), 0);
        assert!(mcrd.doodad_refs.is_empty());
    }

    #[test]
    fn test_mcrd_round_trip() {
        let original = McrdChunk {
            doodad_refs: vec![1, 2, 3, 5, 8, 13, 21],
        };

        let mut buffer = Cursor::new(Vec::new());
        buffer.write_le(&original).unwrap();
        assert_eq!(buffer.position(), 28); // 7 refs × 4 bytes

        let data = buffer.into_inner();
        let mut cursor = Cursor::new(data);
        let parsed: McrdChunk = cursor.read_le().unwrap();

        assert_eq!(original.doodad_refs, parsed.doodad_refs);
    }

    #[test]
    fn test_mcrd_contains_doodad() {
        let mcrd = McrdChunk {
            doodad_refs: vec![10, 20, 30, 40],
        };

        assert!(mcrd.contains_doodad(10));
        assert!(mcrd.contains_doodad(30));
        assert!(!mcrd.contains_doodad(15));
        assert!(!mcrd.contains_doodad(50));
    }

    #[test]
    fn test_mcrd_validate_refs() {
        let mcrd = McrdChunk {
            doodad_refs: vec![0, 5, 10, 15],
        };

        // All refs valid with MDDF count of 20
        assert!(mcrd.validate_refs(20));

        // Ref 15 invalid with MDDF count of 15
        assert!(!mcrd.validate_refs(15));

        // All refs invalid with MDDF count of 0
        assert!(!mcrd.validate_refs(0));
    }

    #[test]
    fn test_mcrd_default() {
        let mcrd = McrdChunk::default();
        assert_eq!(mcrd.count(), 0);
        assert!(mcrd.doodad_refs.is_empty());
    }

    #[test]
    fn test_mcrd_single_ref() {
        let data = vec![0x2A, 0x00, 0x00, 0x00]; // ref = 42
        let mut cursor = Cursor::new(data);
        let mcrd: McrdChunk = cursor.read_le().unwrap();

        assert_eq!(mcrd.count(), 1);
        assert_eq!(mcrd.doodad_refs[0], 42);
    }

    #[test]
    fn test_mcrd_large_indices() {
        let mcrd = McrdChunk {
            doodad_refs: vec![1000, 5000, 10000],
        };

        let mut buffer = Cursor::new(Vec::new());
        buffer.write_le(&mcrd).unwrap();

        let data = buffer.into_inner();
        let mut cursor = Cursor::new(data);
        let parsed: McrdChunk = cursor.read_le().unwrap();

        assert_eq!(parsed.doodad_refs[0], 1000);
        assert_eq!(parsed.doodad_refs[1], 5000);
        assert_eq!(parsed.doodad_refs[2], 10000);
    }
}
