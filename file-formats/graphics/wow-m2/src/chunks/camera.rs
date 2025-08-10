use crate::chunks::animation::M2AnimationTrackHeader;
use crate::version::M2Version;
use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::C3Vector;
use wow_data_derive::{VWowHeaderR, WowHeaderW};

use super::animation::M2SplineKey;

bitflags::bitflags! {
    /// Camera flags as defined in the M2 format
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct M2CameraFlags: u16 {
        /// Camera uses custom UVs for positioning
        const CUSTOM_UV = 0x01;
        /// Auto-generated camera based on model
        const AUTO_GENERATED = 0x02;
        /// Camera is at global scene coordinates
        const GLOBAL_POSITION = 0x04;
    }
}

impl WowHeaderR for M2CameraFlags {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        Ok(Self::from_bits_retain(reader.wow_read()?))
    }
}
impl WowHeaderW for M2CameraFlags {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        writer.wow_write(&self.bits())?;
        Ok(())
    }
    fn wow_size(&self) -> usize {
        2
    }
}

#[derive(Debug, Clone, VWowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub enum M2CameraFov {
    None,

    #[wow_data(read_if = version < M2Version::Cataclysm)]
    Some(f32),
}

#[derive(Debug, Clone, WowHeaderW)]
#[wow_data(version = M2Version)]
pub enum M2CameraFovAnimation {
    None,

    #[wow_data(read_if = version >= M2Version::Cataclysm)]
    Some(M2AnimationTrackHeader<M2SplineKey<f32>>),
}

impl VWowHeaderR<M2Version> for M2CameraFovAnimation {
    fn wow_read<R: Read + Seek>(reader: &mut R, version: M2Version) -> WDResult<Self> {
        Ok(if version >= M2Version::Cataclysm {
            Self::Some(reader.wow_read_versioned(version)?)
        } else {
            Self::None
        })
    }
}

/// Represents a camera in an M2 model
#[derive(Debug, Clone, VWowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub struct M2Camera {
    pub camera_type: u32,
    /// Field of view (in radians)
    #[wow_data(versioned)]
    pub fov: M2CameraFov,

    pub far_clip: f32,
    pub near_clip: f32,

    #[wow_data(versioned)]
    pub position_animation: M2AnimationTrackHeader<M2SplineKey<C3Vector>>,
    pub position_base: C3Vector,

    #[wow_data(versioned)]
    pub target_position_animation: M2AnimationTrackHeader<M2SplineKey<C3Vector>>,
    pub target_position_base: C3Vector,

    #[wow_data(versioned)]
    pub roll_animation: M2AnimationTrackHeader<M2SplineKey<f32>>,

    #[wow_data(versioned)]
    pub fov_animation: M2CameraFovAnimation,
    // pub id: u32,
    // pub flags: M2CameraFlags,
}

impl M2Camera {
    // /// Create a new camera with default values
    // pub fn new(id: u32) -> Self {
    //     Self {
    //         camera_type: 0,
    //         fov: 0.8726646, // 50 degrees in radians
    //         far_clip: 100.0,
    //         near_clip: 0.1,
    //         position_animation: M2AnimationTrackHeader::new(M2AnimationTrackHeader::new()),
    //         target_position_animation: M2AnimationTrackHeader::new(M2AnimationTrackHeader::new()),
    //         roll_animation: M2AnimationTrackHeader::new(M2AnimationTrackHeader::new()),
    //         id,
    //         flags: M2CameraFlags::empty(),
    //     }
    // }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::io::Cursor;
//
//     #[test]
//     fn test_camera_parse_write() {
//         let camera = M2Camera::new(1);
//
//         // Test write
//         let mut data = Vec::new();
//         camera
//             .write(&mut data, M2Version::Classic.to_header_version())
//             .unwrap();
//
//         // Test parse
//         let mut cursor = Cursor::new(data);
//         let parsed = M2Camera::parse(&mut cursor, M2Version::Classic.to_header_version()).unwrap();
//
//         assert_eq!(parsed.camera_type, 0);
//         assert_eq!(parsed.id, 1);
//         assert_eq!(parsed.flags, M2CameraFlags::empty());
//     }
//
//     #[test]
//     fn test_camera_flags() {
//         let flags = M2CameraFlags::CUSTOM_UV | M2CameraFlags::AUTO_GENERATED;
//         assert!(flags.contains(M2CameraFlags::CUSTOM_UV));
//         assert!(flags.contains(M2CameraFlags::AUTO_GENERATED));
//         assert!(!flags.contains(M2CameraFlags::GLOBAL_POSITION));
//     }
// }
