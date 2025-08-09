use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{C3Vector, Color};
use wow_data_derive::{WowHeaderRV, WowHeaderW};

use crate::M2Error;
use crate::chunks::animation::M2AnimationTrackHeader;
use crate::error::Result;
use crate::version::M2Version;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct M2LightFlags: u16 {
        /// Light is directional (otherwise it's a point light)
        const DIRECTIONAL = 0x01;
        /// Unknown flag from Blood Elf "BE_hairSynthesizer.m2"
        const UNKNOWN_BE_HAIR = 0x02;
    }
}

impl WowHeaderR for M2LightFlags {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        Ok(Self::from_bits_retain(reader.wow_read()?))
    }
}
impl WowHeaderW for M2LightFlags {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        writer.wow_write(&self.bits())?;
        Ok(())
    }
    fn wow_size(&self) -> usize {
        2
    }
}

/// Light type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M2LightType {
    /// Directional light (like the sun)
    Directional = 0,
    /// Point light (emits light in all directions)
    Point = 1,
    /// Spot light (emits light in a cone)
    Spot = 2,
    /// Ambient light (global illumination)
    Ambient = 3,
}

impl TryFrom<u16> for M2LightType {
    type Error = M2Error;

    fn try_from(value: u16) -> Result<Self> {
        match value {
            0 => Ok(Self::Directional),
            1 => Ok(Self::Point),
            2 => Ok(Self::Spot),
            3 => Ok(Self::Ambient),
            _ => Err(M2Error::UnsupportedNumericVersion(value as u32)),
        }
    }
}

impl From<M2LightType> for u16 {
    fn from(value: M2LightType) -> Self {
        match value {
            M2LightType::Directional => 0,
            M2LightType::Point => 1,
            M2LightType::Spot => 2,
            M2LightType::Ambient => 3,
        }
    }
}

impl WowHeaderR for M2LightType {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        let value: u16 = reader.wow_read()?;
        Ok(value.try_into()?)
    }
}
impl WowHeaderW for M2LightType {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        let value: u16 = (*self).into();
        writer.wow_write(&value)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        2
    }
}

/// Represents a light in an M2 model
#[derive(Debug, Clone, WowHeaderRV, WowHeaderW)]
#[wow_data(version = M2Version)]
pub struct M2Light {
    pub light_type: M2LightType,
    /// Bone to attach the light to
    pub bone_index: u16,
    pub position: C3Vector,
    #[wow_data(versioned)]
    pub ambient_color_animation: M2AnimationTrackHeader<Color>,
    #[wow_data(versioned)]
    pub ambient_intensity: M2AnimationTrackHeader<f32>,
    #[wow_data(versioned)]
    pub diffuse_color_animation: M2AnimationTrackHeader<Color>,
    #[wow_data(versioned)]
    pub diffuse_intensity: M2AnimationTrackHeader<f32>,
    /// Attenuation start animation (where light begins to fade)
    #[wow_data(versioned)]
    pub attenuation_start_animation: M2AnimationTrackHeader<f32>,
    /// Attenuation end animation (where light fully fades)
    #[wow_data(versioned)]
    pub attenuation_end_animation: M2AnimationTrackHeader<f32>,
    #[wow_data(versioned)]
    pub visibility_animation: M2AnimationTrackHeader<u8>,
    // /// Light ID
    // pub id: u32,
    // /// Light flags
    // pub flags: M2LightFlags,
}

impl M2Light {
    // /// Parse a light from a reader based on the M2 version
    // pub fn parse<R: Read>(reader: &mut R, version: u32) -> Result<Self> {
    //     let light_type_raw = reader.read_u8()?;
    //     let light_type = M2LightType::from_u8(light_type_raw).unwrap_or(M2LightType::Point);
    //
    //     let bone_index = reader.read_u16_le()?;
    //     reader.read_u8()?; // Skip padding
    //
    //     let position = C3Vector::parse(reader)?;
    //
    //     let ambient_color_animation = M2AnimationTrackHeader::parse(reader, version)?;
    //     let diffuse_color_animation = M2AnimationTrackHeader::parse(reader, version)?;
    //     let attenuation_start_animation = M2AnimationTrackHeader::parse(reader, version)?;
    //     let attenuation_end_animation = M2AnimationTrackHeader::parse(reader, version)?;
    //     let visibility_animation = M2AnimationTrackHeader::parse(reader, version)?;
    //
    //     let id = reader.read_u32_le()?;
    //
    //     // 2 bytes for flags, 2 bytes of padding
    //     let flags = M2LightFlags::from_bits_retain(reader.read_u16_le()?);
    //     reader.read_u16_le()?; // Skip padding
    //
    //     Ok(Self {
    //         light_type,
    //         bone_index,
    //         position,
    //         ambient_color_animation,
    //         diffuse_color_animation,
    //         attenuation_start_animation,
    //         attenuation_end_animation,
    //         visibility_animation,
    //         id,
    //         flags,
    //     })
    // }
    //
    // /// Write a light to a writer based on the M2 version
    // pub fn write<W: Write>(&self, writer: &mut W, _version: u32) -> Result<()> {
    //     writer.write_u8(self.light_type as u8)?;
    //     writer.write_u16_le(self.bone_index)?;
    //     writer.write_u8(0)?; // Write padding
    //
    //     self.position.write(writer)?;
    //
    //     self.ambient_color_animation.write(writer)?;
    //     self.diffuse_color_animation.write(writer)?;
    //     self.attenuation_start_animation.write(writer)?;
    //     self.attenuation_end_animation.write(writer)?;
    //     self.visibility_animation.write(writer)?;
    //
    //     writer.write_u32_le(self.id)?;
    //
    //     // 2 bytes for flags, 2 bytes of padding
    //     writer.write_u16_le(self.flags.bits())?;
    //     writer.write_u16_le(0)?; // Write padding
    //
    //     Ok(())
    // }

    // /// Create a new light with default values
    // pub fn new(light_type: M2LightType, bone_index: u16, id: u32) -> Self {
    //     Self {
    //         light_type,
    //         bone_index,
    //         position: C3Vector {
    //             x: 0.0,
    //             y: 0.0,
    //             z: 0.0,
    //         },
    //         ambient_color_animation: M2AnimationTrackHeader::new(M2AnimationTrackHeader::new()),
    //         diffuse_color_animation: M2AnimationTrackHeader::new(M2AnimationTrackHeader::new()),
    //         attenuation_start_animation: M2AnimationTrackHeader::new(M2AnimationTrackHeader::new()),
    //         attenuation_end_animation: M2AnimationTrackHeader::new(M2AnimationTrackHeader::new()),
    //         visibility_animation: M2AnimationTrackHeader::new(M2AnimationTrackHeader::new()),
    //         id,
    //         flags: match light_type {
    //             M2LightType::Directional => M2LightFlags::DIRECTIONAL,
    //             _ => M2LightFlags::empty(),
    //         },
    //     }
    // }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::io::Cursor;
//
//     #[test]
//     fn test_light_parse_write() {
//         let light = M2Light::new(M2LightType::Point, 1, 0);
//
//         // Test write
//         let mut data = Vec::new();
//         light
//             .write(&mut data, M2Version::Classic.to_header_version())
//             .unwrap();
//
//         // Test parse
//         let mut cursor = Cursor::new(data);
//         let parsed = M2Light::parse(&mut cursor, M2Version::Classic.to_header_version()).unwrap();
//
//         assert_eq!(parsed.light_type, M2LightType::Point);
//         assert_eq!(parsed.bone_index, 1);
//         assert_eq!(parsed.id, 0);
//         assert_eq!(parsed.flags, M2LightFlags::empty());
//     }
//
//     #[test]
//     fn test_light_types() {
//         assert_eq!(M2LightType::from_u8(0), Some(M2LightType::Directional));
//         assert_eq!(M2LightType::from_u8(1), Some(M2LightType::Point));
//         assert_eq!(M2LightType::from_u8(2), Some(M2LightType::Spot));
//         assert_eq!(M2LightType::from_u8(3), Some(M2LightType::Ambient));
//         assert_eq!(M2LightType::from_u8(4), None);
//     }
//
//     #[test]
//     fn test_light_flags() {
//         let flags = M2LightFlags::DIRECTIONAL;
//         assert!(flags.contains(M2LightFlags::DIRECTIONAL));
//         assert!(!flags.contains(M2LightFlags::UNKNOWN_BE_HAIR));
//     }
// }
