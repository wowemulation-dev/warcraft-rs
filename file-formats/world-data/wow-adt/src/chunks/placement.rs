//! Object placement chunk structures for ADT format.
//!
//! Placement chunks define positions, rotations, and scales for models (M2) and
//! world map objects (WMO) within ADT terrain tiles.
//!
//! - **MDDF**: M2 model (doodad) placement referencing MMID/MMDX chunks
//! - **MODF**: WMO placement referencing MWID/MWMO chunks
//!
//! These chunks split into separate `_obj0.adt` files in Cataclysm+.

use binrw::{BinRead, BinWrite};

/// M2 model placement (doodad) - 36 bytes per entry.
///
/// Defines position, rotation, and scale for an M2 model (tree, rock, building prop).
/// nameId references MMID chunk which indexes into MMDX filenames.
///
/// # Binary Layout
///
/// ```text
/// Offset | Size | Field     | Description
/// -------|------|-----------|------------------------------------------
/// 0x00   |  4   | nameId    | MMID entry index
/// 0x04   |  4   | uniqueId  | Unique identifier across loaded ADTs
/// 0x08   | 12   | position  | World coordinates (X, Y, Z)
/// 0x14   | 12   | rotation  | Rotation in degrees (X, Y, Z)
/// 0x20   |  2   | scale     | 1024 = 1.0 scale factor
/// 0x22   |  2   | flags     | MDDFFlags
/// ```
///
/// # Coordinate System
///
/// Position uses WoW's coordinate system:
/// - X-axis: West ← East
/// - Y-axis: Vertical (up)
/// - Z-axis: North ← South
///
/// To convert to terrain coordinates: `terrain_coord = 17066.0 - position`
///
/// # Scale Factor
///
/// Scale is stored as `u16` where 1024 = 1.0. To convert:
/// ```rust
/// # let scale_value: u16 = 2048; // Example: 2x scale
/// let actual_scale = scale_value as f32 / 1024.0;
/// assert_eq!(actual_scale, 2.0);
/// ```
///
/// Reference: <https://wowdev.wiki/ADT/v18#MDDF_chunk>
#[derive(Debug, Clone, Copy, BinRead, BinWrite)]
#[brw(little)]
pub struct DoodadPlacement {
    /// Reference to MMID chunk entry (offset index into MMDX)
    pub name_id: u32,

    /// Unique identifier across all loaded ADT files
    pub unique_id: u32,

    /// World position [X, Y, Z] in WoW coordinates
    pub position: [f32; 3],

    /// Rotation angles in degrees [X, Y, Z]
    pub rotation: [f32; 3],

    /// Scale factor (1024 = 1.0)
    pub scale: u16,

    /// MDDFFlags bitfield
    pub flags: u16,
}

impl DoodadPlacement {
    /// Convert scale to floating point (1024 = 1.0).
    ///
    /// # Returns
    ///
    /// Scale factor as f32 (e.g., 1024 → 1.0, 2048 → 2.0)
    #[must_use]
    pub fn get_scale(&self) -> f32 {
        f32::from(self.scale) / 1024.0
    }

    /// Check if doodad accepts projected textures (Legion+).
    #[must_use]
    pub fn accepts_proj_textures(&self) -> bool {
        self.flags & 0x1000 != 0
    }

    /// Check if nameId is a file data ID instead of MMID offset (Legion+).
    #[must_use]
    pub fn uses_file_data_id(&self) -> bool {
        self.flags & 0x40 != 0
    }
}

/// MDDF chunk - M2 model placement array (Vanilla+)
///
/// Contains placement data for all doodads (M2 models) in the ADT tile.
/// Each entry is 36 bytes. Split into `_obj0.adt` in Cataclysm+.
///
/// Reference: <https://wowdev.wiki/ADT/v18#MDDF_chunk>
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
pub struct MddfChunk {
    /// Array of doodad placements
    #[br(parse_with = binrw::helpers::until_eof)]
    pub placements: Vec<DoodadPlacement>,
}

impl MddfChunk {
    /// Get number of placed doodads.
    #[must_use]
    pub fn count(&self) -> usize {
        self.placements.len()
    }

    /// Validate that all nameId references are within bounds.
    ///
    /// # Arguments
    ///
    /// * `mmid_count` - Number of entries in MMID chunk
    ///
    /// # Returns
    ///
    /// `true` if all nameId values are valid indices
    #[must_use]
    pub fn validate_name_ids(&self, mmid_count: usize) -> bool {
        self.placements.iter().all(|p| {
            // Skip validation for file data IDs (Legion+)
            if p.uses_file_data_id() {
                true
            } else {
                (p.name_id as usize) < mmid_count
            }
        })
    }
}

/// WMO placement - 64 bytes per entry.
///
/// Defines position, rotation, scale, and bounding box for a World Map Object.
/// nameId references MWID chunk which indexes into MWMO filenames.
///
/// # Binary Layout
///
/// ```text
/// Offset | Size | Field        | Description
/// -------|------|--------------|------------------------------------------
/// 0x00   |  4   | nameId       | MWID entry index
/// 0x04   |  4   | uniqueId     | Unique identifier
/// 0x08   | 12   | position     | World coordinates (X, Y, Z)
/// 0x14   | 12   | rotation     | Rotation in degrees (X, Y, Z)
/// 0x20   | 12   | extents_min  | Bounding box minimum (X, Y, Z)
/// 0x2C   | 12   | extents_max  | Bounding box maximum (X, Y, Z)
/// 0x38   |  2   | flags        | MODFFlags
/// 0x3A   |  2   | doodadSet    | WMO doodad set index
/// 0x3C   |  2   | nameSet      | WMO name set index
/// 0x3E   |  2   | scale        | 1024 = 1.0 (Legion+)
/// ```
///
/// Reference: <https://wowdev.wiki/ADT/v18#MODF_chunk>
#[derive(Debug, Clone, Copy, BinRead, BinWrite)]
#[brw(little)]
pub struct WmoPlacement {
    /// Reference to MWID chunk entry (offset index into MWMO)
    pub name_id: u32,

    /// Unique identifier across all loaded ADT files
    pub unique_id: u32,

    /// World position [X, Y, Z] in WoW coordinates
    pub position: [f32; 3],

    /// Rotation angles in degrees [X, Y, Z]
    pub rotation: [f32; 3],

    /// Bounding box minimum corner [X, Y, Z]
    pub extents_min: [f32; 3],

    /// Bounding box maximum corner [X, Y, Z]
    pub extents_max: [f32; 3],

    /// MODFFlags bitfield
    pub flags: u16,

    /// WMO doodad set index (which set of props to render)
    pub doodad_set: u16,

    /// WMO name set index
    pub name_set: u16,

    /// Scale factor (1024 = 1.0, Legion+)
    pub scale: u16,
}

impl WmoPlacement {
    /// Convert scale to floating point (1024 = 1.0).
    ///
    /// # Returns
    ///
    /// Scale factor as f32 (e.g., 1024 → 1.0, 2048 → 2.0)
    #[must_use]
    pub fn get_scale(&self) -> f32 {
        f32::from(self.scale) / 1024.0
    }

    /// Check if WMO is destructible (server-controllable).
    #[must_use]
    pub fn is_destroyable(&self) -> bool {
        self.flags & 0x1 != 0
    }

    /// Check if WMO uses LOD variant (_LOD1.WMO).
    #[must_use]
    pub fn uses_lod(&self) -> bool {
        self.flags & 0x2 != 0
    }

    /// Check if nameId is a file data ID instead of MWID offset (Legion+).
    #[must_use]
    pub fn uses_file_data_id(&self) -> bool {
        self.flags & 0x8 != 0
    }

    /// Calculate bounding box volume.
    ///
    /// # Returns
    ///
    /// Volume in cubic world units
    #[must_use]
    pub fn bounding_box_volume(&self) -> f32 {
        let width = self.extents_max[0] - self.extents_min[0];
        let height = self.extents_max[1] - self.extents_min[1];
        let depth = self.extents_max[2] - self.extents_min[2];
        width * height * depth
    }
}

/// MODF chunk - WMO placement array (Vanilla+)
///
/// Contains placement data for all World Map Objects in the ADT tile.
/// Each entry is 64 bytes. Split into `_obj0.adt` in Cataclysm+.
///
/// Reference: <https://wowdev.wiki/ADT/v18#MODF_chunk>
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
pub struct ModfChunk {
    /// Array of WMO placements
    #[br(parse_with = binrw::helpers::until_eof)]
    pub placements: Vec<WmoPlacement>,
}

impl ModfChunk {
    /// Get number of placed WMOs.
    #[must_use]
    pub fn count(&self) -> usize {
        self.placements.len()
    }

    /// Validate that all nameId references are within bounds.
    ///
    /// # Arguments
    ///
    /// * `mwid_count` - Number of entries in MWID chunk
    ///
    /// # Returns
    ///
    /// `true` if all nameId values are valid indices
    #[must_use]
    pub fn validate_name_ids(&self, mwid_count: usize) -> bool {
        self.placements.iter().all(|p| {
            // Skip validation for file data IDs (Legion+)
            if p.uses_file_data_id() {
                true
            } else {
                (p.name_id as usize) < mwid_count
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_doodad_placement_parse() {
        let data = vec![
            0x05, 0x00, 0x00, 0x00, // nameId: 5
            0xAB, 0xCD, 0xEF, 0x01, // uniqueId: 0x01EFCDAB
            0x00, 0x00, 0x80, 0x44, // position.x: 1024.0
            0x00, 0x00, 0x00, 0x42, // position.y: 32.0
            0x00, 0x00, 0x40, 0x44, // position.z: 768.0
            0x00, 0x00, 0x00, 0x00, // rotation.x: 0.0
            0x00, 0x00, 0xB4, 0x42, // rotation.y: 90.0
            0x00, 0x00, 0x00, 0x00, // rotation.z: 0.0
            0x00, 0x04, // scale: 1024 (1.0x)
            0x00, 0x10, // flags: 0x1000 (accepts proj textures)
        ];

        let mut cursor = Cursor::new(data);
        let placement = DoodadPlacement::read_le(&mut cursor).unwrap();

        assert_eq!(placement.name_id, 5);
        assert_eq!(placement.unique_id, 0x01EF_CDAB);
        assert_eq!(placement.position, [1024.0, 32.0, 768.0]);
        assert_eq!(placement.rotation, [0.0, 90.0, 0.0]);
        assert_eq!(placement.scale, 1024);
        assert_eq!(placement.flags, 0x1000);
        assert_eq!(placement.get_scale(), 1.0);
        assert!(placement.accepts_proj_textures());
        assert!(!placement.uses_file_data_id());
    }

    #[test]
    fn test_doodad_placement_scale_conversion() {
        let placement = DoodadPlacement {
            name_id: 0,
            unique_id: 0,
            position: [0.0; 3],
            rotation: [0.0; 3],
            scale: 2048, // 2.0x
            flags: 0,
        };

        assert_eq!(placement.get_scale(), 2.0);
    }

    #[test]
    fn test_doodad_placement_flags() {
        let mut placement = DoodadPlacement {
            name_id: 0,
            unique_id: 0,
            position: [0.0; 3],
            rotation: [0.0; 3],
            scale: 1024,
            flags: 0x40, // file data ID flag
        };

        assert!(placement.uses_file_data_id());
        assert!(!placement.accepts_proj_textures());

        placement.flags = 0x1040; // both flags
        assert!(placement.uses_file_data_id());
        assert!(placement.accepts_proj_textures());
    }

    #[test]
    fn test_mddf_chunk_parse() {
        // Two doodad placements
        let mut data = Vec::new();

        // First placement
        data.extend_from_slice(&[
            0x01, 0x00, 0x00, 0x00, // nameId: 1
            0x10, 0x00, 0x00, 0x00, // uniqueId: 16
            0x00, 0x00, 0x00, 0x00, // position.x: 0.0
            0x00, 0x00, 0x00, 0x00, // position.y: 0.0
            0x00, 0x00, 0x00, 0x00, // position.z: 0.0
            0x00, 0x00, 0x00, 0x00, // rotation.x: 0.0
            0x00, 0x00, 0x00, 0x00, // rotation.y: 0.0
            0x00, 0x00, 0x00, 0x00, // rotation.z: 0.0
            0x00, 0x04, // scale: 1024
            0x00, 0x00, // flags: 0
        ]);

        // Second placement
        data.extend_from_slice(&[
            0x02, 0x00, 0x00, 0x00, // nameId: 2
            0x20, 0x00, 0x00, 0x00, // uniqueId: 32
            0x00, 0x00, 0x80, 0x3F, // position.x: 1.0
            0x00, 0x00, 0x00, 0x40, // position.y: 2.0
            0x00, 0x00, 0x40, 0x40, // position.z: 3.0
            0x00, 0x00, 0x00, 0x00, // rotation.x: 0.0
            0x00, 0x00, 0x00, 0x00, // rotation.y: 0.0
            0x00, 0x00, 0x00, 0x00, // rotation.z: 0.0
            0x00, 0x08, // scale: 2048 (2.0x)
            0x00, 0x00, // flags: 0
        ]);

        let mut cursor = Cursor::new(data);
        let mddf = MddfChunk::read_le(&mut cursor).unwrap();

        assert_eq!(mddf.count(), 2);
        assert_eq!(mddf.placements[0].name_id, 1);
        assert_eq!(mddf.placements[0].unique_id, 16);
        assert_eq!(mddf.placements[1].name_id, 2);
        assert_eq!(mddf.placements[1].position, [1.0, 2.0, 3.0]);
        assert_eq!(mddf.placements[1].get_scale(), 2.0);
    }

    #[test]
    fn test_mddf_chunk_validate_name_ids() {
        let mddf = MddfChunk {
            placements: vec![
                DoodadPlacement {
                    name_id: 0,
                    unique_id: 1,
                    position: [0.0; 3],
                    rotation: [0.0; 3],
                    scale: 1024,
                    flags: 0,
                },
                DoodadPlacement {
                    name_id: 2,
                    unique_id: 2,
                    position: [0.0; 3],
                    rotation: [0.0; 3],
                    scale: 1024,
                    flags: 0,
                },
            ],
        };

        // Valid with 3 MMID entries (indices 0, 1, 2)
        assert!(mddf.validate_name_ids(3));

        // Invalid with 2 MMID entries (index 2 out of bounds)
        assert!(!mddf.validate_name_ids(2));
    }

    #[test]
    fn test_mddf_chunk_validate_file_data_ids() {
        // File data IDs should skip bounds checking
        let mddf = MddfChunk {
            placements: vec![DoodadPlacement {
                name_id: 9999999, // Large file data ID
                unique_id: 1,
                position: [0.0; 3],
                rotation: [0.0; 3],
                scale: 1024,
                flags: 0x40, // file data ID flag
            }],
        };

        // Should pass validation despite large nameId
        assert!(mddf.validate_name_ids(10));
    }

    #[test]
    fn test_wmo_placement_parse() {
        let data = vec![
            0x03, 0x00, 0x00, 0x00, // nameId: 3
            0x42, 0x00, 0x00, 0x00, // uniqueId: 66
            0x00, 0x00, 0x80, 0x44, // position.x: 1024.0
            0x00, 0x00, 0x00, 0x42, // position.y: 32.0
            0x00, 0x00, 0x40, 0x44, // position.z: 768.0
            0x00, 0x00, 0x00, 0x00, // rotation.x: 0.0
            0x00, 0x00, 0xB4, 0x42, // rotation.y: 90.0
            0x00, 0x00, 0x00, 0x00, // rotation.z: 0.0
            0x00, 0x00, 0x00, 0x00, // extents_min.x: 0.0
            0x00, 0x00, 0x00, 0x00, // extents_min.y: 0.0
            0x00, 0x00, 0x00, 0x00, // extents_min.z: 0.0
            0x00, 0x00, 0x80, 0x3F, // extents_max.x: 1.0
            0x00, 0x00, 0x00, 0x40, // extents_max.y: 2.0
            0x00, 0x00, 0x40, 0x40, // extents_max.z: 3.0
            0x03, 0x00, // flags: 0x0003 (destroyable + lod)
            0x05, 0x00, // doodadSet: 5
            0x00, 0x00, // nameSet: 0
            0x00, 0x04, // scale: 1024 (1.0x)
        ];

        let mut cursor = Cursor::new(data);
        let placement = WmoPlacement::read_le(&mut cursor).unwrap();

        assert_eq!(placement.name_id, 3);
        assert_eq!(placement.unique_id, 66);
        assert_eq!(placement.position, [1024.0, 32.0, 768.0]);
        assert_eq!(placement.rotation, [0.0, 90.0, 0.0]);
        assert_eq!(placement.extents_min, [0.0, 0.0, 0.0]);
        assert_eq!(placement.extents_max, [1.0, 2.0, 3.0]);
        assert_eq!(placement.flags, 0x0003);
        assert_eq!(placement.doodad_set, 5);
        assert_eq!(placement.name_set, 0);
        assert_eq!(placement.scale, 1024);
        assert_eq!(placement.get_scale(), 1.0);
        assert!(placement.is_destroyable());
        assert!(placement.uses_lod());
        assert!(!placement.uses_file_data_id());
    }

    #[test]
    fn test_wmo_placement_bounding_box_volume() {
        let placement = WmoPlacement {
            name_id: 0,
            unique_id: 0,
            position: [0.0; 3],
            rotation: [0.0; 3],
            extents_min: [0.0, 0.0, 0.0],
            extents_max: [10.0, 20.0, 30.0],
            flags: 0,
            doodad_set: 0,
            name_set: 0,
            scale: 1024,
        };

        assert_eq!(placement.bounding_box_volume(), 6000.0); // 10 * 20 * 30
    }

    #[test]
    fn test_wmo_placement_flags() {
        let mut placement = WmoPlacement {
            name_id: 0,
            unique_id: 0,
            position: [0.0; 3],
            rotation: [0.0; 3],
            extents_min: [0.0; 3],
            extents_max: [0.0; 3],
            flags: 0x8, // file data ID flag
            doodad_set: 0,
            name_set: 0,
            scale: 1024,
        };

        assert!(placement.uses_file_data_id());
        assert!(!placement.is_destroyable());
        assert!(!placement.uses_lod());

        placement.flags = 0x0B; // all flags
        assert!(placement.uses_file_data_id());
        assert!(placement.is_destroyable());
        assert!(placement.uses_lod());
    }

    #[test]
    fn test_modf_chunk_parse() {
        // Single WMO placement (64 bytes)
        let data = vec![
            0x01, 0x00, 0x00, 0x00, // nameId: 1
            0x10, 0x00, 0x00, 0x00, // uniqueId: 16
            0x00, 0x00, 0x00, 0x00, // position.x: 0.0
            0x00, 0x00, 0x00, 0x00, // position.y: 0.0
            0x00, 0x00, 0x00, 0x00, // position.z: 0.0
            0x00, 0x00, 0x00, 0x00, // rotation.x: 0.0
            0x00, 0x00, 0x00, 0x00, // rotation.y: 0.0
            0x00, 0x00, 0x00, 0x00, // rotation.z: 0.0
            0x00, 0x00, 0x00, 0x00, // extents_min.x: 0.0
            0x00, 0x00, 0x00, 0x00, // extents_min.y: 0.0
            0x00, 0x00, 0x00, 0x00, // extents_min.z: 0.0
            0x00, 0x00, 0x80, 0x3F, // extents_max.x: 1.0
            0x00, 0x00, 0x80, 0x3F, // extents_max.y: 1.0
            0x00, 0x00, 0x80, 0x3F, // extents_max.z: 1.0
            0x01, 0x00, // flags: 0x0001 (destroyable)
            0x00, 0x00, // doodadSet: 0
            0x00, 0x00, // nameSet: 0
            0x00, 0x04, // scale: 1024
        ];

        let mut cursor = Cursor::new(data);
        let modf = ModfChunk::read_le(&mut cursor).unwrap();

        assert_eq!(modf.count(), 1);
        assert_eq!(modf.placements[0].name_id, 1);
        assert_eq!(modf.placements[0].unique_id, 16);
        assert!(modf.placements[0].is_destroyable());
    }

    #[test]
    fn test_modf_chunk_validate_name_ids() {
        let modf = ModfChunk {
            placements: vec![
                WmoPlacement {
                    name_id: 0,
                    unique_id: 1,
                    position: [0.0; 3],
                    rotation: [0.0; 3],
                    extents_min: [0.0; 3],
                    extents_max: [0.0; 3],
                    flags: 0,
                    doodad_set: 0,
                    name_set: 0,
                    scale: 1024,
                },
                WmoPlacement {
                    name_id: 1,
                    unique_id: 2,
                    position: [0.0; 3],
                    rotation: [0.0; 3],
                    extents_min: [0.0; 3],
                    extents_max: [0.0; 3],
                    flags: 0,
                    doodad_set: 0,
                    name_set: 0,
                    scale: 1024,
                },
            ],
        };

        // Valid with 2 MWID entries (indices 0, 1)
        assert!(modf.validate_name_ids(2));

        // Invalid with 1 MWID entry (index 1 out of bounds)
        assert!(!modf.validate_name_ids(1));
    }

    #[test]
    fn test_modf_chunk_round_trip() {
        let original = ModfChunk {
            placements: vec![WmoPlacement {
                name_id: 5,
                unique_id: 100,
                position: [1.0, 2.0, 3.0],
                rotation: [0.0, 90.0, 0.0],
                extents_min: [0.0, 0.0, 0.0],
                extents_max: [10.0, 10.0, 10.0],
                flags: 0x03,
                doodad_set: 2,
                name_set: 1,
                scale: 2048,
            }],
        };

        let mut buffer = Cursor::new(Vec::new());
        original.write_le(&mut buffer).unwrap();

        let data = buffer.into_inner();
        let mut cursor = Cursor::new(data);
        let parsed = ModfChunk::read_le(&mut cursor).unwrap();

        assert_eq!(original.placements.len(), parsed.placements.len());
        assert_eq!(original.placements[0].name_id, parsed.placements[0].name_id);
        assert_eq!(
            original.placements[0].position,
            parsed.placements[0].position
        );
        assert_eq!(parsed.placements[0].get_scale(), 2.0);
    }

    #[test]
    fn test_mddf_chunk_empty() {
        let data = vec![];
        let mut cursor = Cursor::new(data);
        let mddf = MddfChunk::read_le(&mut cursor).unwrap();

        assert_eq!(mddf.count(), 0);
        assert!(mddf.validate_name_ids(0));
    }

    #[test]
    fn test_modf_chunk_empty() {
        let data = vec![];
        let mut cursor = Cursor::new(data);
        let modf = ModfChunk::read_le(&mut cursor).unwrap();

        assert_eq!(modf.count(), 0);
        assert!(modf.validate_name_ids(0));
    }
}
