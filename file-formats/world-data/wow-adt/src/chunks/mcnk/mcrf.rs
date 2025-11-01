use binrw::{BinRead, BinWrite};

/// MCRF chunk - Object references (pre-Cataclysm).
///
/// Contains indices into MDDF (doodad placements) and MODF (WMO placements)
/// chunks. Only objects referenced here should be rendered for this terrain tile.
///
/// Format:
/// - First n_doodad_refs entries: MDDF indices
/// - Next n_map_obj_refs entries: MODF indices
///
/// In Cataclysm+, this chunk is split into:
/// - MCRD: Doodad references only
/// - MCRW: WMO references only
///
/// ## Version Support
///
/// - **Vanilla (1.12.1)**: ✅ Introduced
/// - **TBC (2.4.3)**: ✅ Present
/// - **WotLK (3.3.5a)**: ✅ Present
/// - **Cataclysm (4.3.4)**: ❌ Removed (split file architecture)
/// - **MoP (5.4.8)**: ❌ Not present
///
/// Reference: <https://wowdev.wiki/ADT/v18#MCRF_sub-chunk>
#[derive(Debug, Clone, Default, BinRead, BinWrite)]
#[brw(little)]
pub struct McrfChunk {
    /// Object reference indices (doodads first, then WMOs)
    ///
    /// Parse as u32 array until EOF. Split using McnkHeader counts:
    /// - `references[0..n_doodad_refs]` = MDDF indices
    /// - `references[n_doodad_refs..]` = MODF indices
    #[br(parse_with = binrw::helpers::until_eof)]
    pub references: Vec<u32>,
}

impl McrfChunk {
    /// Get doodad (M2) references.
    ///
    /// # Arguments
    ///
    /// * `n_doodad_refs` - Count from McnkHeader.n_doodad_refs
    ///
    /// # Returns
    ///
    /// Slice of MDDF indices
    pub fn doodad_refs(&self, n_doodad_refs: usize) -> &[u32] {
        let end = n_doodad_refs.min(self.references.len());
        &self.references[0..end]
    }

    /// Get WMO references.
    ///
    /// # Arguments
    ///
    /// * `n_doodad_refs` - Count from McnkHeader.n_doodad_refs
    ///
    /// # Returns
    ///
    /// Slice of MODF indices
    pub fn wmo_refs(&self, n_doodad_refs: usize) -> &[u32] {
        if n_doodad_refs >= self.references.len() {
            &[]
        } else {
            &self.references[n_doodad_refs..]
        }
    }

    /// Validate reference counts match header.
    ///
    /// # Arguments
    ///
    /// * `n_doodad_refs` - Expected doodad count from McnkHeader
    /// * `n_map_obj_refs` - Expected WMO count from McnkHeader
    ///
    /// # Returns
    ///
    /// `true` if total references match expected count
    pub fn validate_counts(&self, n_doodad_refs: usize, n_map_obj_refs: usize) -> bool {
        self.references.len() == n_doodad_refs + n_map_obj_refs
    }

    /// Get total reference count.
    pub fn total_refs(&self) -> usize {
        self.references.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::{BinRead, BinWrite};
    use std::io::Cursor;

    #[test]
    fn parse_mixed_doodad_and_wmo_references() {
        let data: Vec<u8> = vec![
            0x01, 0x00, 0x00, 0x00, // Doodad ref 1
            0x03, 0x00, 0x00, 0x00, // Doodad ref 3
            0x05, 0x00, 0x00, 0x00, // Doodad ref 5
            0x0A, 0x00, 0x00, 0x00, // WMO ref 10
            0x14, 0x00, 0x00, 0x00, // WMO ref 20
        ];

        let mut cursor = Cursor::new(data);
        let mcrf = McrfChunk::read(&mut cursor).expect("Failed to parse MCRF");

        assert_eq!(mcrf.references.len(), 5);
        assert_eq!(mcrf.references, vec![1, 3, 5, 10, 20]);
    }

    #[test]
    fn split_references_using_n_doodad_refs() {
        let mcrf = McrfChunk {
            references: vec![1, 3, 5, 10, 20],
        };

        let doodad_refs = mcrf.doodad_refs(3);
        assert_eq!(doodad_refs, &[1, 3, 5]);

        let wmo_refs = mcrf.wmo_refs(3);
        assert_eq!(wmo_refs, &[10, 20]);
    }

    #[test]
    fn doodad_only_chunk() {
        let mcrf = McrfChunk {
            references: vec![0, 1, 2, 3],
        };

        let doodad_refs = mcrf.doodad_refs(4);
        assert_eq!(doodad_refs, &[0, 1, 2, 3]);

        let wmo_refs = mcrf.wmo_refs(4);
        assert_eq!(wmo_refs, &[]);
    }

    #[test]
    fn wmo_only_chunk() {
        let mcrf = McrfChunk {
            references: vec![10, 20, 30],
        };

        let doodad_refs = mcrf.doodad_refs(0);
        assert_eq!(doodad_refs, &[]);

        let wmo_refs = mcrf.wmo_refs(0);
        assert_eq!(wmo_refs, &[10, 20, 30]);
    }

    #[test]
    fn empty_chunk() {
        let data: Vec<u8> = vec![];
        let mut cursor = Cursor::new(data);
        let mcrf = McrfChunk::read(&mut cursor).expect("Failed to parse empty MCRF");

        assert_eq!(mcrf.references.len(), 0);
        assert_eq!(mcrf.total_refs(), 0);

        let doodad_refs = mcrf.doodad_refs(0);
        assert_eq!(doodad_refs, &[]);

        let wmo_refs = mcrf.wmo_refs(0);
        assert_eq!(wmo_refs, &[]);
    }

    #[test]
    fn validate_counts() {
        let mcrf = McrfChunk {
            references: vec![1, 2, 3, 10, 20],
        };

        assert!(mcrf.validate_counts(3, 2));
        assert!(!mcrf.validate_counts(2, 2));
        assert!(!mcrf.validate_counts(3, 3));
        assert!(mcrf.validate_counts(5, 0));
        assert!(mcrf.validate_counts(0, 5));
    }

    #[test]
    fn round_trip_serialization() {
        let original = McrfChunk {
            references: vec![1, 3, 5, 7, 9, 11],
        };

        let mut buffer = Cursor::new(Vec::new());
        original.write(&mut buffer).expect("Failed to write MCRF");

        buffer.set_position(0);
        let parsed = McrfChunk::read(&mut buffer).expect("Failed to read MCRF");

        assert_eq!(original.references, parsed.references);
    }

    #[test]
    fn boundary_split_at_start() {
        let mcrf = McrfChunk {
            references: vec![10, 20, 30],
        };

        let doodad_refs = mcrf.doodad_refs(0);
        assert_eq!(doodad_refs, &[]);

        let wmo_refs = mcrf.wmo_refs(0);
        assert_eq!(wmo_refs, &[10, 20, 30]);
    }

    #[test]
    fn boundary_split_at_end() {
        let mcrf = McrfChunk {
            references: vec![1, 2, 3],
        };

        let doodad_refs = mcrf.doodad_refs(3);
        assert_eq!(doodad_refs, &[1, 2, 3]);

        let wmo_refs = mcrf.wmo_refs(3);
        assert_eq!(wmo_refs, &[]);
    }

    #[test]
    fn boundary_n_doodad_refs_exceeds_total() {
        let mcrf = McrfChunk {
            references: vec![1, 2, 3],
        };

        let doodad_refs = mcrf.doodad_refs(10);
        assert_eq!(doodad_refs, &[1, 2, 3]);

        let wmo_refs = mcrf.wmo_refs(10);
        assert_eq!(wmo_refs, &[]);
    }

    #[test]
    fn total_refs_count() {
        let mcrf = McrfChunk {
            references: vec![1, 2, 3, 4, 5],
        };
        assert_eq!(mcrf.total_refs(), 5);

        let empty = McrfChunk::default();
        assert_eq!(empty.total_refs(), 0);
    }

    #[test]
    fn single_reference() {
        let mcrf = McrfChunk {
            references: vec![42],
        };

        assert_eq!(mcrf.doodad_refs(1), &[42]);
        assert_eq!(mcrf.wmo_refs(1), &[]);

        assert_eq!(mcrf.doodad_refs(0), &[]);
        assert_eq!(mcrf.wmo_refs(0), &[42]);
    }

    #[test]
    fn large_reference_values() {
        let data: Vec<u8> = vec![
            0xFF, 0xFF, 0x00, 0x00, // 65535 in little-endian
            0x00, 0x00, 0x00, 0x10, // 0x1000_0000 in little-endian
            0xFF, 0xFF, 0xFF, 0xFF, // u32::MAX
        ];

        let mut cursor = Cursor::new(data);
        let mcrf = McrfChunk::read(&mut cursor).expect("Failed to parse MCRF");

        assert_eq!(mcrf.references.len(), 3);
        assert_eq!(mcrf.references[0], 65535);
        assert_eq!(mcrf.references[1], 0x1000_0000);
        assert_eq!(mcrf.references[2], u32::MAX);
    }
}
