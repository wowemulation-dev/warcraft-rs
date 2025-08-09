use std::io::{Read, Seek, Write};
use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{C2Vector, C3Vector};
use wow_data_derive::{WowHeaderRV, WowHeaderW};

use crate::version::M2Version;

bitflags::bitflags! {
    /// Vertex flags as defined in the M2 format
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct M2VertexFlags: u8 {
        /// Transform using bone 0
        const TRANSFORM_BONE_0 = 0x01;
        /// Transform using bone 1
        const TRANSFORM_BONE_1 = 0x02;
        /// Transform using bone 2
        const TRANSFORM_BONE_2 = 0x04;
        /// Transform using bone 3
        const TRANSFORM_BONE_3 = 0x08;
        /// Normal compressed
        const NORMAL_COMPRESSED = 0x10;
        /// Unknown 0x20
        const UNKNOWN_0x20 = 0x20;
        /// Unknown 0x40
        const UNKNOWN_0x40 = 0x40;
        /// Unknown 0x80
        const UNKNOWN_0x80 = 0x80;
    }
}

impl WowHeaderR for M2VertexFlags {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        Ok(Self::from_bits_retain(reader.wow_read()?))
    }
}
impl WowHeaderW for M2VertexFlags {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        writer.wow_write(&self.bits())?;
        Ok(())
    }
    fn wow_size(&self) -> usize {
        4
    }
}

#[derive(Debug, Clone, Default, WowHeaderRV, WowHeaderW)]
#[wow_data(version = M2Version)]
enum M2TexCoords2 {
    #[default]
    None,

    #[wow_data(read_if = version >= M2Version::Cataclysm)]
    Some(C2Vector),
}

#[derive(Debug, Clone, Default, WowHeaderRV, WowHeaderW)]
#[wow_data(version = M2Version)]
pub struct M2Vertex {
    pub position: C3Vector,
    /// Bone weights (0-255)
    pub bone_weights: [u8; 4],
    /// Bone indices
    pub bone_indices: [u8; 4],
    /// Normal vector
    pub normal: C3Vector,
    /// Primary texture coordinates
    pub tex_coords: C2Vector,

    /// Secondary texture coordinates (added in Cataclysm)
    #[wow_data(versioned)]
    pub tex_coords2: M2TexCoords2,
}

impl M2Vertex {
    // /// Convert this vertex to a different version
    // pub fn convert(&self, target_version: M2Version) -> Self {
    //     let mut new_vertex = self.clone();
    //
    //     // Handle version-specific conversions
    //     if target_version >= M2Version::Cataclysm && self.tex_coords2.is_none() {
    //         // When upgrading to Cataclysm or later, add secondary texture coordinates if missing
    //         new_vertex.tex_coords2 = Some(self.tex_coords);
    //     } else if target_version < M2Version::Cataclysm {
    //         // When downgrading to pre-Cataclysm, remove secondary texture coordinates
    //         new_vertex.tex_coords2 = None;
    //     }
    //
    //     new_vertex
    // }

    /// Get the effective bone count used by this vertex
    pub fn effective_bone_count(&self) -> u32 {
        let mut count = 0;

        for i in 0..4 {
            if self.bone_weights[i] > 0 {
                count += 1;
            }
        }

        count
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::io::Cursor;
//
//     #[test]
//     fn test_vertex_parse_classic() {
//         let mut data = Vec::new();
//
//         // Position
//         data.extend_from_slice(&1.0f32.to_le_bytes());
//         data.extend_from_slice(&2.0f32.to_le_bytes());
//         data.extend_from_slice(&3.0f32.to_le_bytes());
//
//         // Bone weights
//         data.push(255);
//         data.push(128);
//         data.push(64);
//         data.push(0);
//
//         // Bone indices
//         data.push(0);
//         data.push(1);
//         data.push(2);
//         data.push(3);
//
//         // Normal
//         data.extend_from_slice(&0.5f32.to_le_bytes());
//         data.extend_from_slice(&0.5f32.to_le_bytes());
//         data.extend_from_slice(&std::f32::consts::FRAC_1_SQRT_2.to_le_bytes());
//
//         // Texture coordinates
//         data.extend_from_slice(&0.0f32.to_le_bytes());
//         data.extend_from_slice(&1.0f32.to_le_bytes());
//
//         let mut cursor = Cursor::new(data);
//         let vertex = M2Vertex::parse(&mut cursor, M2Version::Classic.to_header_version()).unwrap();
//
//         assert_eq!(vertex.position.x, 1.0);
//         assert_eq!(vertex.position.y, 2.0);
//         assert_eq!(vertex.position.z, 3.0);
//
//         assert_eq!(vertex.bone_weights, [255, 128, 64, 0]);
//         assert_eq!(vertex.bone_indices, [0, 1, 2, 3]);
//
//         assert_eq!(vertex.normal.x, 0.5);
//         assert_eq!(vertex.normal.y, 0.5);
//         assert!((vertex.normal.z - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.0001);
//
//         assert_eq!(vertex.tex_coords.x, 0.0);
//         assert_eq!(vertex.tex_coords.y, 1.0);
//
//         assert!(vertex.tex_coords2.is_none());
//     }
//
//     #[test]
//     fn test_vertex_parse_cataclysm() {
//         let mut data = Vec::new();
//
//         // Position
//         data.extend_from_slice(&1.0f32.to_le_bytes());
//         data.extend_from_slice(&2.0f32.to_le_bytes());
//         data.extend_from_slice(&3.0f32.to_le_bytes());
//
//         // Bone weights
//         data.push(255);
//         data.push(128);
//         data.push(64);
//         data.push(0);
//
//         // Bone indices
//         data.push(0);
//         data.push(1);
//         data.push(2);
//         data.push(3);
//
//         // Normal
//         data.extend_from_slice(&0.5f32.to_le_bytes());
//         data.extend_from_slice(&0.5f32.to_le_bytes());
//         data.extend_from_slice(&std::f32::consts::FRAC_1_SQRT_2.to_le_bytes());
//
//         // Texture coordinates
//         data.extend_from_slice(&0.0f32.to_le_bytes());
//         data.extend_from_slice(&1.0f32.to_le_bytes());
//
//         // Secondary texture coordinates (added in Cataclysm)
//         data.extend_from_slice(&0.5f32.to_le_bytes());
//         data.extend_from_slice(&0.5f32.to_le_bytes());
//
//         let mut cursor = Cursor::new(data);
//         let vertex =
//             M2Vertex::parse(&mut cursor, M2Version::Cataclysm.to_header_version()).unwrap();
//
//         assert_eq!(vertex.position.x, 1.0);
//         assert_eq!(vertex.position.y, 2.0);
//         assert_eq!(vertex.position.z, 3.0);
//
//         assert_eq!(vertex.bone_weights, [255, 128, 64, 0]);
//         assert_eq!(vertex.bone_indices, [0, 1, 2, 3]);
//
//         assert_eq!(vertex.normal.x, 0.5);
//         assert_eq!(vertex.normal.y, 0.5);
//         assert!((vertex.normal.z - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.0001);
//
//         assert_eq!(vertex.tex_coords.x, 0.0);
//         assert_eq!(vertex.tex_coords.y, 1.0);
//
//         assert!(vertex.tex_coords2.is_some());
//         let tex_coords2 = vertex.tex_coords2.unwrap();
//         assert_eq!(tex_coords2.x, 0.5);
//         assert_eq!(tex_coords2.y, 0.5);
//     }
//
//     #[test]
//     fn test_vertex_write() {
//         let vertex = M2Vertex {
//             position: C3Vector {
//                 x: 1.0,
//                 y: 2.0,
//                 z: 3.0,
//             },
//             bone_weights: [255, 128, 64, 0],
//             bone_indices: [0, 1, 2, 3],
//             normal: C3Vector {
//                 x: 0.5,
//                 y: 0.5,
//                 z: std::f32::consts::FRAC_1_SQRT_2,
//             },
//             tex_coords: C2Vector { x: 0.0, y: 1.0 },
//             tex_coords2: Some(C2Vector { x: 0.5, y: 0.5 }),
//         };
//
//         // Test writing in Classic format
//         let mut classic_data = Vec::new();
//         vertex
//             .write(&mut classic_data, M2Version::Classic.to_header_version())
//             .unwrap();
//
//         // Should not include secondary texture coordinates
//         // position (12) + bone_weights (4) + bone_indices (4) + normal (12) + tex_coords (8) = 40 bytes
//         assert_eq!(classic_data.len(), 40);
//
//         // Test writing in Cataclysm format
//         let mut cata_data = Vec::new();
//         vertex
//             .write(&mut cata_data, M2Version::Cataclysm.to_header_version())
//             .unwrap();
//
//         // Should include secondary texture coordinates
//         // position (12) + bone_weights (4) + bone_indices (4) + normal (12) + tex_coords (8) + tex_coords2 (8) = 48 bytes
//         assert_eq!(cata_data.len(), 48);
//     }
//
//     #[test]
//     fn test_vertex_conversion() {
//         // Create a Classic vertex
//         let classic_vertex = M2Vertex {
//             position: C3Vector {
//                 x: 1.0,
//                 y: 2.0,
//                 z: 3.0,
//             },
//             bone_weights: [255, 128, 64, 0],
//             bone_indices: [0, 1, 2, 3],
//             normal: C3Vector {
//                 x: 0.5,
//                 y: 0.5,
//                 z: std::f32::consts::FRAC_1_SQRT_2,
//             },
//             tex_coords: C2Vector { x: 0.0, y: 1.0 },
//             tex_coords2: None,
//         };
//
//         // Convert to Cataclysm
//         let cata_vertex = classic_vertex.convert(M2Version::Cataclysm);
//
//         // Should have secondary texture coordinates
//         assert!(cata_vertex.tex_coords2.is_some());
//         let tex_coords2 = cata_vertex.tex_coords2.unwrap();
//         assert_eq!(tex_coords2.x, classic_vertex.tex_coords.x);
//         assert_eq!(tex_coords2.y, classic_vertex.tex_coords.y);
//
//         // Convert back to Classic
//         let classic_vertex2 = cata_vertex.convert(M2Version::Classic);
//
//         // Should not have secondary texture coordinates
//         assert!(classic_vertex2.tex_coords2.is_none());
//     }
// }
