use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Write};

use crate::common::{C2Vector, C3Vector};
use crate::error::Result;
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

/// Represents a vertex in an M2 model
#[derive(Debug, Clone)]
pub struct M2Vertex {
    /// Position of the vertex
    pub position: C3Vector,
    /// Bone weights (0-255)
    pub bone_weights: [u8; 4],
    /// Bone indices
    pub bone_indices: [u8; 4],
    /// Normal vector
    pub normal: C3Vector,
    /// Primary texture coordinates
    pub tex_coords: C2Vector,
    /// Secondary texture coordinates
    pub tex_coords2: C2Vector,
}

impl M2Vertex {
    /// Parse a vertex from a reader based on the M2 version
    pub fn parse<R: Read>(reader: &mut R, _version: u32) -> Result<Self> {
        // Position
        let position = C3Vector::parse(reader)?;

        // Bone weights
        let bone_weights = [
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
        ];

        // Bone indices
        let bone_indices = [
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
        ];

        // Normal
        let normal = C3Vector::parse(reader)?;

        // Texture coordinates
        let tex_coords = C2Vector::parse(reader)?;

        // Secondary texture coordinates
        let tex_coords2 = C2Vector::parse(reader)?;

        Ok(Self {
            position,
            bone_weights,
            bone_indices,
            normal,
            tex_coords,
            tex_coords2,
        })
    }

    /// Write a vertex to a writer based on the M2 version
    pub fn write<W: Write>(&self, writer: &mut W, _version: u32) -> Result<()> {
        // Position
        self.position.write(writer)?;

        // Bone weights
        for &weight in &self.bone_weights {
            writer.write_u8(weight)?;
        }

        // Bone indices
        for &index in &self.bone_indices {
            writer.write_u8(index)?;
        }

        // Normal
        self.normal.write(writer)?;

        // Texture coordinates
        self.tex_coords.write(writer)?;
        self.tex_coords2.write(writer)?;

        Ok(())
    }

    /// Convert this vertex to a different version
    pub fn convert(&self, _target_version: M2Version) -> Self {
        self.clone()
    }

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

    /// Calculate the size of this vertex in bytes for a specific version
    pub fn size_in_bytes(version: M2Version) -> usize {
        let mut size = 0;

        // Position (3 floats)
        size += 3 * 4;

        // Bone weights (4 bytes)
        size += 4;

        // Bone indices (4 bytes)
        size += 4;

        // Normal (3 floats)
        size += 3 * 4;

        // Texture coordinates (2 floats)
        size += 2 * 4;

        // Secondary texture coordinates (2 floats, Cataclysm and later)
        if version >= M2Version::Cataclysm {
            size += 2 * 4;
        }

        size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_vertex_parse() {
        let mut data = Vec::new();

        // Position
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&2.0f32.to_le_bytes());
        data.extend_from_slice(&3.0f32.to_le_bytes());

        // Bone weights
        data.push(255);
        data.push(128);
        data.push(64);
        data.push(0);

        // Bone indices
        data.push(0);
        data.push(1);
        data.push(2);
        data.push(3);

        // Normal
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&std::f32::consts::FRAC_1_SQRT_2.to_le_bytes());

        // Texture coordinates
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());

        let mut cursor = Cursor::new(data);
        let vertex = M2Vertex::parse(&mut cursor, M2Version::Classic.to_header_version()).unwrap();

        assert_eq!(vertex.position.x, 1.0);
        assert_eq!(vertex.position.y, 2.0);
        assert_eq!(vertex.position.z, 3.0);

        assert_eq!(vertex.bone_weights, [255, 128, 64, 0]);
        assert_eq!(vertex.bone_indices, [0, 1, 2, 3]);

        assert_eq!(vertex.normal.x, 0.5);
        assert_eq!(vertex.normal.y, 0.5);
        assert!((vertex.normal.z - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.0001);

        assert_eq!(vertex.tex_coords.x, 0.0);
        assert_eq!(vertex.tex_coords.y, 1.0);

        assert_eq!(vertex.tex_coords2.x, 0.0);
        assert_eq!(vertex.tex_coords2.y, 1.0);
    }

    #[test]
    fn test_vertex_write() {
        let vertex = M2Vertex {
            position: C3Vector {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            bone_weights: [255, 128, 64, 0],
            bone_indices: [0, 1, 2, 3],
            normal: C3Vector {
                x: 0.5,
                y: 0.5,
                z: std::f32::consts::FRAC_1_SQRT_2,
            },
            tex_coords: C2Vector { x: 0.0, y: 1.0 },
            tex_coords2: C2Vector { x: 0.5, y: 0.5 },
        };

        // Test writing
        let mut cata_data = Vec::new();
        vertex
            .write(&mut cata_data, M2Version::Cataclysm.to_header_version())
            .unwrap();

        // position (12) + bone_weights (4) + bone_indices (4) + normal (12) + tex_coords (8) + tex_coords2 (8) = 48 bytes
        assert_eq!(cata_data.len(), 48);
    }
}
