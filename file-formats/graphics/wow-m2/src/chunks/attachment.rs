use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Seek, Write};

use crate::chunks::animation::{M2AnimationBlock, M2AnimationTrack};
use crate::common::C3Vector;
use crate::error::Result;
use crate::version::M2Version;

/// Attachment types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M2AttachmentType {
    /// Attach another model
    Shoulder = 0,
    /// Attach a particle emitter
    ShoulderLeft = 1,
    /// Attach a light
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

/// Represents an attachment in an M2 model
///
/// Attachment structure (48 bytes for all versions):
/// - id: u32 (4 bytes) - attachment ID
/// - bone: i32 (4 bytes) - bone index to attach to
/// - position: C3Vector (12 bytes) - position relative to bone
/// - scale_animation: M2AnimationBlock (28 bytes) - scale animation track
#[derive(Debug, Clone)]
pub struct M2Attachment {
    /// Attachment ID (determines attachment point type)
    pub id: u32,
    /// Bone to attach to
    pub bone_index: i32,
    /// Position relative to bone
    pub position: C3Vector,
    /// Scale animation data (visibility/scale of attached object)
    pub scale_animation: M2AnimationBlock<f32>,
}

impl M2Attachment {
    /// Parse an attachment from a reader based on the M2 version
    ///
    /// Attachment structure is 48 bytes for all versions.
    pub fn parse<R: Read + Seek>(reader: &mut R, _version: u32) -> Result<Self> {
        let id = reader.read_u32_le()?;
        let bone_index = reader.read_i32_le()?;
        let position = C3Vector::parse(reader)?;
        let scale_animation = M2AnimationBlock::parse(reader)?;

        Ok(Self {
            id,
            bone_index,
            position,
            scale_animation,
        })
    }

    /// Write an attachment to a writer based on the M2 version
    pub fn write<W: Write>(&self, writer: &mut W, _version: u32) -> Result<()> {
        writer.write_u32_le(self.id)?;
        writer.write_i32_le(self.bone_index)?;
        self.position.write(writer)?;
        self.scale_animation.write(writer)?;

        Ok(())
    }

    /// Convert this attachment to a different version (no version differences for attachments)
    pub fn convert(&self, _target_version: M2Version) -> Self {
        self.clone()
    }

    /// Create a new attachment with default values
    pub fn new(id: u32, bone_index: i32) -> Self {
        Self {
            id,
            bone_index,
            position: C3Vector::default(),
            scale_animation: M2AnimationBlock::new(M2AnimationTrack::default()),
        }
    }

    /// Returns the size of an attachment in bytes (always 48)
    pub const fn size() -> usize {
        48
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_attachment_parse_write() {
        let attachment = M2Attachment::new(1, 2);

        // Test write
        let mut data = Vec::new();
        attachment
            .write(&mut data, M2Version::Vanilla.to_header_version())
            .unwrap();

        // Attachment should be 48 bytes
        assert_eq!(data.len(), 48);

        // Test parse
        let mut cursor = Cursor::new(data);
        let parsed =
            M2Attachment::parse(&mut cursor, M2Version::Vanilla.to_header_version()).unwrap();

        assert_eq!(parsed.id, 1);
        assert_eq!(parsed.bone_index, 2);
    }

    #[test]
    fn test_attachment_size() {
        assert_eq!(M2Attachment::size(), 48);
    }

    #[test]
    fn test_attachment_types() {
        assert_eq!(
            M2AttachmentType::from_u16(0),
            Some(M2AttachmentType::Shoulder)
        );
        assert_eq!(
            M2AttachmentType::from_u16(13),
            Some(M2AttachmentType::WeaponMain)
        );
        assert_eq!(
            M2AttachmentType::from_u16(14),
            Some(M2AttachmentType::WeaponOff)
        );
        assert_eq!(M2AttachmentType::from_u16(20), None);
    }
}
