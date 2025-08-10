use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{C3Vector, Quaternion};
use wow_data_derive::{VWowHeaderR, WowHeaderW};

use crate::M2Error;
use crate::chunks::animation::{M2AnimationBlock, M2AnimationTrackHeader};
use crate::error::Result;
use crate::version::M2Version;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M2TextureTransformType {
    None = 0,
    Scroll = 1,
    Rotate = 2,
    Scale = 3,
    Matrix = 4,
}

impl TryFrom<u16> for M2TextureTransformType {
    type Error = M2Error;

    fn try_from(value: u16) -> Result<Self> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Scroll),
            2 => Ok(Self::Rotate),
            3 => Ok(Self::Scale),
            4 => Ok(Self::Matrix),
            _ => Err(M2Error::UnsupportedNumericVersion(value as u32)),
        }
    }
}

impl From<M2TextureTransformType> for u16 {
    fn from(value: M2TextureTransformType) -> Self {
        match value {
            M2TextureTransformType::None => 0,
            M2TextureTransformType::Scroll => 1,
            M2TextureTransformType::Rotate => 2,
            M2TextureTransformType::Scale => 3,
            M2TextureTransformType::Matrix => 4,
        }
    }
}

impl WowHeaderR for M2TextureTransformType {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        let value: u16 = reader.wow_read()?;
        Ok(value.try_into()?)
    }
}
impl WowHeaderW for M2TextureTransformType {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        let value: u16 = (*self).into();
        writer.wow_write(&value)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        2
    }
}

#[derive(Debug, Clone, Copy, VWowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub enum M2TextureTransformIdType {
    #[wow_data(read_if = version >= M2Version::Legion)]
    Some {
        id: u32,
        transform_type: M2TextureTransformType,
    },
    None,
}

#[derive(Debug, Clone, VWowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub struct M2TextureTransform {
    #[wow_data(versioned)]
    pub id_type: M2TextureTransformIdType,

    #[wow_data(versioned)]
    pub translation: M2AnimationBlock<C3Vector>,

    #[wow_data(versioned)]
    pub rotation: M2AnimationBlock<Quaternion>,

    #[wow_data(versioned)]
    pub scaling: M2AnimationBlock<C3Vector>,
}

impl M2TextureTransform {
    // /// Parse a texture transform from a reader
    // pub fn parse<R: Read>(reader: &mut R, version: u32) -> Result<Self> {
    //     let id = reader.read_u32_le()?;
    //
    //     let transform_type_raw = reader.read_u16_le()?;
    //     let transform_type = M2TextureTransformType::from_u16(transform_type_raw)
    //         .unwrap_or(M2TextureTransformType::None);
    //
    //     // Skip 2 bytes of padding
    //     reader.read_u16_le()?;
    //
    //     let translation = M2AnimationBlock::parse(reader, version)?;
    //     let rotation = M2AnimationBlock::parse(reader, version)?;
    //     let scaling = M2AnimationBlock::parse(reader, version)?;
    //
    //     Ok(Self {
    //         id,
    //         transform_type,
    //         translation,
    //         rotation,
    //         scaling,
    //     })
    // }

    // /// Write a texture transform to a writer
    // pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
    //     writer.write_u32_le(self.id)?;
    //     writer.write_u16_le(self.transform_type as u16)?;
    //
    //     // Write 2 bytes of padding
    //     writer.write_u16_le(0)?;
    //
    //     self.translation.write(writer)?;
    //     self.rotation.write(writer)?;
    //     self.scaling.write(writer)?;
    //
    //     Ok(())
    // }

    /// Create a new texture transform with default values
    pub fn new(id: u32, transform_type: M2TextureTransformType) -> Self {
        Self {
            id_type: M2TextureTransformIdType::Some { id, transform_type },
            translation: M2AnimationBlock::new(M2AnimationTrackHeader::new()),
            rotation: M2AnimationBlock::new(M2AnimationTrackHeader::new()),
            scaling: M2AnimationBlock::new(M2AnimationTrackHeader::new()),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::io::Cursor;
//
//     #[test]
//     fn test_c4quaternion_parse_write() {
//         let quat = C4Quaternion {
//             x: 0.0,
//             y: 0.0,
//             z: 0.0,
//             w: 1.0,
//         };
//
//         let mut data = Vec::new();
//         quat.write(&mut data).unwrap();
//
//         let mut cursor = Cursor::new(data);
//         let parsed_quat = C4Quaternion::parse(&mut cursor).unwrap();
//
//         assert_eq!(parsed_quat.x, 0.0);
//         assert_eq!(parsed_quat.y, 0.0);
//         assert_eq!(parsed_quat.z, 0.0);
//         assert_eq!(parsed_quat.w, 1.0);
//     }
//
//     #[test]
//     fn test_texture_transform_type() {
//         assert_eq!(
//             M2TextureTransformType::from_u16(0),
//             Some(M2TextureTransformType::None)
//         );
//         assert_eq!(
//             M2TextureTransformType::from_u16(1),
//             Some(M2TextureTransformType::Scroll)
//         );
//         assert_eq!(
//             M2TextureTransformType::from_u16(2),
//             Some(M2TextureTransformType::Rotate)
//         );
//         assert_eq!(
//             M2TextureTransformType::from_u16(3),
//             Some(M2TextureTransformType::Scale)
//         );
//         assert_eq!(
//             M2TextureTransformType::from_u16(4),
//             Some(M2TextureTransformType::Matrix)
//         );
//         assert_eq!(M2TextureTransformType::from_u16(5), None);
//     }
// }
