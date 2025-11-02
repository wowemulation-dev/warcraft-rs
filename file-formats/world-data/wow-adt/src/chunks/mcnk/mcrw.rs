use binrw::{BinRead, BinWrite};

/// MCRW chunk - WMO references for split files (Cataclysm+)
///
/// Contains indices into the MODF chunk, specifying which WMOs (World Map Objects)
/// appear within this terrain chunk. Replaces the WMO portion of MCRF in
/// split file architecture.
///
/// ## Structure
///
/// ```text
/// struct {
///     uint32_t wmo_refs[];  // Variable length
/// } MCRW;
/// ```
///
/// Each value is an index into the file's MODF chunk array.
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
/// dimensions defined in the WMO model files.
///
/// ## References
///
/// - **wowdev.wiki**: <https://wowdev.wiki/ADT/v18#MCRW_sub-chunk>
/// - **WoWFormatLib**: Not found in ADT.Struct.cs (implementation gap)
/// - **noggit-red**: Not implemented (Cataclysm+ not supported)
/// - **wow.export**: Not found in ADTLoader.js (implementation gap)
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ❌ Not present
/// - **TBC (2.4.3)**: ❌ Not present
/// - **WotLK (3.3.5a)**: ❌ Not present
/// - **Cataclysm (4.3.4)**: ✅ Introduced (replaces MCRF for WMOs)
/// - **MoP (5.4.8)**: ✅ Present
///
/// ## Deviations
///
/// None - straightforward u32 array matching wowdev.wiki specification.
///
/// Reference: <https://wowdev.wiki/ADT/v18#MCRW_sub-chunk>
#[derive(Debug, Clone, Default, BinRead, BinWrite)]
#[brw(little)]
pub struct McrwChunk {
    /// WMO reference indices (MODF entry indices)
    ///
    /// Each value is an index into the MODF placement array.
    /// Variable length - parse until end of chunk.
    #[br(parse_with = binrw::helpers::until_eof)]
    pub wmo_refs: Vec<u32>,
}

impl McrwChunk {
    /// Get number of WMO references.
    #[must_use]
    pub fn count(&self) -> usize {
        self.wmo_refs.len()
    }

    /// Check if a specific MODF index is referenced.
    ///
    /// # Arguments
    ///
    /// * `modf_index` - Index into MODF placement array
    ///
    /// # Returns
    ///
    /// `true` if this chunk references the WMO at `modf_index`
    #[must_use]
    pub fn contains_wmo(&self, modf_index: u32) -> bool {
        self.wmo_refs.contains(&modf_index)
    }

    /// Validate that all references are within MODF bounds.
    ///
    /// # Arguments
    ///
    /// * `modf_count` - Total number of entries in MODF chunk
    ///
    /// # Returns
    ///
    /// `true` if all indices are valid
    #[must_use]
    pub fn validate_refs(&self, modf_count: usize) -> bool {
        self.wmo_refs.iter().all(|&idx| (idx as usize) < modf_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::{BinReaderExt, BinWriterExt};
    use std::io::Cursor;

    #[test]
    fn test_mcrw_parse() {
        // 5 WMO references
        let data = vec![
            0x00, 0x00, 0x00, 0x00, // ref[0] = 0
            0x02, 0x00, 0x00, 0x00, // ref[1] = 2
            0x04, 0x00, 0x00, 0x00, // ref[2] = 4
            0x08, 0x00, 0x00, 0x00, // ref[3] = 8
            0x10, 0x00, 0x00, 0x00, // ref[4] = 16
        ];

        let mut cursor = Cursor::new(data);
        let mcrw: McrwChunk = cursor.read_le().unwrap();

        assert_eq!(mcrw.count(), 5);
        assert_eq!(mcrw.wmo_refs[0], 0);
        assert_eq!(mcrw.wmo_refs[1], 2);
        assert_eq!(mcrw.wmo_refs[2], 4);
        assert_eq!(mcrw.wmo_refs[3], 8);
        assert_eq!(mcrw.wmo_refs[4], 16);
    }

    #[test]
    fn test_mcrw_empty() {
        let data = vec![];
        let mut cursor = Cursor::new(data);
        let mcrw: McrwChunk = cursor.read_le().unwrap();

        assert_eq!(mcrw.count(), 0);
        assert!(mcrw.wmo_refs.is_empty());
    }

    #[test]
    fn test_mcrw_round_trip() {
        let original = McrwChunk {
            wmo_refs: vec![0, 1, 3, 7, 15, 31],
        };

        let mut buffer = Cursor::new(Vec::new());
        buffer.write_le(&original).unwrap();
        assert_eq!(buffer.position(), 24); // 6 refs × 4 bytes

        let data = buffer.into_inner();
        let mut cursor = Cursor::new(data);
        let parsed: McrwChunk = cursor.read_le().unwrap();

        assert_eq!(original.wmo_refs, parsed.wmo_refs);
    }

    #[test]
    fn test_mcrw_contains_wmo() {
        let mcrw = McrwChunk {
            wmo_refs: vec![5, 10, 15, 20],
        };

        assert!(mcrw.contains_wmo(5));
        assert!(mcrw.contains_wmo(15));
        assert!(!mcrw.contains_wmo(7));
        assert!(!mcrw.contains_wmo(25));
    }

    #[test]
    fn test_mcrw_validate_refs() {
        let mcrw = McrwChunk {
            wmo_refs: vec![0, 3, 7, 12],
        };

        // All refs valid with MODF count of 15
        assert!(mcrw.validate_refs(15));

        // Ref 12 invalid with MODF count of 12
        assert!(!mcrw.validate_refs(12));

        // All refs invalid with MODF count of 0
        assert!(!mcrw.validate_refs(0));
    }

    #[test]
    fn test_mcrw_default() {
        let mcrw = McrwChunk::default();
        assert_eq!(mcrw.count(), 0);
        assert!(mcrw.wmo_refs.is_empty());
    }

    #[test]
    fn test_mcrw_single_ref() {
        let data = vec![0x64, 0x00, 0x00, 0x00]; // ref = 100
        let mut cursor = Cursor::new(data);
        let mcrw: McrwChunk = cursor.read_le().unwrap();

        assert_eq!(mcrw.count(), 1);
        assert_eq!(mcrw.wmo_refs[0], 100);
    }

    #[test]
    fn test_mcrw_large_indices() {
        let mcrw = McrwChunk {
            wmo_refs: vec![500, 2500, 5000],
        };

        let mut buffer = Cursor::new(Vec::new());
        buffer.write_le(&mcrw).unwrap();

        let data = buffer.into_inner();
        let mut cursor = Cursor::new(data);
        let parsed: McrwChunk = cursor.read_le().unwrap();

        assert_eq!(parsed.wmo_refs[0], 500);
        assert_eq!(parsed.wmo_refs[1], 2500);
        assert_eq!(parsed.wmo_refs[2], 5000);
    }
}
