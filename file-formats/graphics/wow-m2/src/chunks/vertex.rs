use wow_data::prelude::*;
use wow_data::types::{C2Vector, C3Vector};
use wow_data_derive::{WowHeaderR, WowHeaderW};

bitflags::bitflags! {
    /// Vertex flags as defined in the M2 format
    #[derive(Debug, Clone, Copy, PartialEq, Eq, WowHeaderR, WowHeaderW)]
    #[wow_data(bitflags=u32)]
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

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
pub struct M2Vertex {
    pub position: C3Vector,
    pub bone_weights: [u8; 4],
    pub bone_indices: [u8; 4],
    pub normal: C3Vector,
    pub tex_coords: C2Vector,
    pub tex_coords2: C2Vector,
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
