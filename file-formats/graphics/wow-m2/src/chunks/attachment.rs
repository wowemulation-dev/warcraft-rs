use wow_data::prelude::*;
use wow_data::types::C3Vector;
use wow_data_derive::{VWowHeaderR, WowHeaderW};

use crate::chunks::animation::M2AnimationTrackHeader;
use crate::version::M2Version;

/// Attachment types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M2AttachmentType {
    /// Also attach another model
    Shoulder = 0,
    /// Also attach a particle emitter
    ShoulderLeft = 1,
    /// Also attach a light
    ShoulderRight = 2,
    /// For shield attachments
    Shield = 3,
    /// Unknown
    Unknown4 = 4,
    /// Unknown
    LeftPalm = 5,
    /// Unknown
    RightPalm = 6,
    /// Unknown
    Unknown7 = 7,
    /// Unknown
    Unknown8 = 8,
    /// Unknown
    Unknown9 = 9,
    /// Unknown
    Head = 10,
    /// Unknown
    SpellLeftHand = 11,
    /// Unknown
    SpellRightHand = 12,
    /// Main hand weapon
    WeaponMain = 13,
    /// Off-hand weapon
    WeaponOff = 14,
}

impl M2AttachmentType {
    /// Parse from integer value
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(Self::Shoulder),
            1 => Some(Self::ShoulderLeft),
            2 => Some(Self::ShoulderRight),
            3 => Some(Self::Shield),
            4 => Some(Self::Unknown4),
            5 => Some(Self::LeftPalm),
            6 => Some(Self::RightPalm),
            7 => Some(Self::Unknown7),
            8 => Some(Self::Unknown8),
            9 => Some(Self::Unknown9),
            10 => Some(Self::Head),
            11 => Some(Self::SpellLeftHand),
            12 => Some(Self::SpellRightHand),
            13 => Some(Self::WeaponMain),
            14 => Some(Self::WeaponOff),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, VWowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub struct M2Attachment {
    pub id: u32,
    pub bone_index: u16,
    pub parent_bone_flying: u16,
    // pub attachment_type: M2AttachmentType,
    // pub animation_data: u16,
    pub position: C3Vector,
    #[wow_data(versioned)]
    pub animate_attached: M2AnimationTrackHeader<u8>,
}

// impl M2Attachment {
//     /// Parse an attachment from a reader based on the M2 version
//     pub fn parse<R: Read>(reader: &mut R, version: u32) -> Result<Self> {
//         let id = reader.read_u32_le()?;
//         let bone_index = reader.read_u16_le()?;
//         let parent_bone_flying = reader.read_u16_le()?;
//
//         let attachment_type_raw = reader.read_u16_le()?;
//         let attachment_type =
//             M2AttachmentType::from_u16(attachment_type_raw).unwrap_or(M2AttachmentType::Shoulder);
//
//         let animation_data = reader.read_u16_le()?;
//         let position = C3Vector::parse(reader)?;
//
//         let scale_animation = M2AnimationTrackHeader::parse(reader, version)?;
//
//         Ok(Self {
//             id,
//             bone_index,
//             parent_bone_flying,
//             attachment_type,
//             animation_data,
//             position,
//             animate_attached: scale_animation,
//         })
//     }
//
//     /// Write an attachment to a writer based on the M2 version
//     pub fn write<W: Write>(&self, writer: &mut W, _version: u32) -> Result<()> {
//         writer.write_u32_le(self.id)?;
//         writer.write_u16_le(self.bone_index)?;
//         writer.write_u16_le(self.parent_bone_flying)?;
//
//         writer.write_u16_le(self.attachment_type as u16)?;
//         writer.write_u16_le(self.animation_data)?;
//
//         self.position.write(writer)?;
//
//         self.animate_attached.write(writer)?;
//
//         Ok(())
//     }
//
//     /// Convert this attachment to a different version (no version differences for attachments)
//     pub fn convert(&self, _target_version: M2Version) -> Self {
//         self.clone()
//     }
//
//     /// Create a new attachment with default values
//     pub fn new(id: u32, bone_index: u16, attachment_type: M2AttachmentType) -> Self {
//         Self {
//             id,
//             bone_index,
//             parent_bone_flying: 0,
//             attachment_type,
//             animation_data: 0,
//             position: C3Vector {
//                 x: 0.0,
//                 y: 0.0,
//                 z: 0.0,
//             },
//             animate_attached: M2AnimationTrackHeader::new(M2AnimationTrackHeader::new()),
//         }
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::io::Cursor;
//
//     #[test]
//     fn test_attachment_parse_write() {
//         let attachment = M2Attachment::new(1, 2, M2AttachmentType::WeaponMain);
//
//         // Test write
//         let mut data = Vec::new();
//         attachment
//             .write(&mut data, M2Version::Classic.to_header_version())
//             .unwrap();
//
//         // Test parse
//         let mut cursor = Cursor::new(data);
//         let parsed =
//             M2Attachment::parse(&mut cursor, M2Version::Classic.to_header_version()).unwrap();
//
//         assert_eq!(parsed.id, 1);
//         assert_eq!(parsed.bone_index, 2);
//         assert_eq!(parsed.attachment_type, M2AttachmentType::WeaponMain);
//     }
//
//     #[test]
//     fn test_attachment_types() {
//         assert_eq!(
//             M2AttachmentType::from_u16(0),
//             Some(M2AttachmentType::Shoulder)
//         );
//         assert_eq!(
//             M2AttachmentType::from_u16(13),
//             Some(M2AttachmentType::WeaponMain)
//         );
//         assert_eq!(
//             M2AttachmentType::from_u16(14),
//             Some(M2AttachmentType::WeaponOff)
//         );
//         assert_eq!(M2AttachmentType::from_u16(20), None);
//     }
// }
