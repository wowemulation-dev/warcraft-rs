use std::io::{Read, Seek, Write};
use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{C2Vector, C3Vector};
use wow_data_derive::{WowHeaderR, WowHeaderW};

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

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub enum M2TexCoords2 {
    #[default]
    None,

    #[wow_data(read_if = version >= M2Version::Cataclysm)]
    Some(C2Vector),
}

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
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
