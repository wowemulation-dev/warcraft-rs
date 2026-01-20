use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Write};

use crate::common::M2Array;
use crate::error::Result;
use crate::version::M2Version;

/// Event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M2EventType {
    /// Play sound
    Sound = 0,
    /// Stop sound
    SoundStop = 1,
    /// Create a spell animation at a bone
    SpellCastOMG = 2,
    /// Hide a model
    Hide = 3,
    /// Unknown
    Unknown4 = 4,
    /// Footstep sound
    StandOrLand = 5,
    /// Used for displaying speech bubbles
    SpeechEmote = 6,
    /// Unknown
    FootstepFront = 7,
    /// Unknown
    FootstepBack = 8,
    /// Play a sound from a sound table
    PlaySoundKitFromTable = 9,
}

impl M2EventType {
    /// Parse from integer value
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Sound),
            1 => Some(Self::SoundStop),
            2 => Some(Self::SpellCastOMG),
            3 => Some(Self::Hide),
            4 => Some(Self::Unknown4),
            5 => Some(Self::StandOrLand),
            6 => Some(Self::SpeechEmote),
            7 => Some(Self::FootstepFront),
            8 => Some(Self::FootstepBack),
            9 => Some(Self::PlaySoundKitFromTable),
            _ => None,
        }
    }
}

/// Represents an event in an M2 model
///
/// Event structure (44 bytes):
/// - identifier\[4\]: Event name string (e.g., "$CAH", "$CST", "$HIT")
/// - data: u32: Sound/spell database ID
/// - bone: i16: Bone index to attach event to
/// - unknown: u16: Unknown field (possibly padding or submesh ID)
/// - position: C3Vector (12 bytes): Position relative to bone
/// - interp_type: u16: Interpolation type for animation
/// - global_sequence: i16: Global sequence ID or -1
/// - ranges: M2Array (8 bytes): Animation ranges
/// - times: M2Array (8 bytes): Timestamp arrays
#[derive(Debug, Clone)]
pub struct M2Event {
    /// Event identifier (4-char string like "$CAH", "$CST")
    pub identifier: [u8; 4],
    /// Event data (sound/spell database ID)
    pub data: u32,
    /// Bone to attach the event to
    pub bone_index: i16,
    /// Unknown field (possibly submesh ID or padding)
    pub unknown: u16,
    /// Position relative to bone
    pub position: [f32; 3],
    /// Interpolation type
    pub interp_type: u16,
    /// Global sequence ID or -1
    pub global_sequence: i16,
    /// Animation ranges (for per-animation timing)
    pub ranges: M2Array<u32>,
    /// Event timestamps
    pub times: M2Array<u32>,
}

impl M2Event {
    /// Parse an event from a reader based on the M2 version
    ///
    /// Event structure is 44 bytes for all versions.
    pub fn parse<R: Read>(reader: &mut R, _version: u32) -> Result<Self> {
        let mut identifier = [0u8; 4];
        reader.read_exact(&mut identifier)?;

        let data = reader.read_u32_le()?;
        let bone_index = reader.read_i16_le()?;
        let unknown = reader.read_u16_le()?;

        let mut position = [0.0; 3];
        for item in &mut position {
            *item = reader.read_f32_le()?;
        }

        let interp_type = reader.read_u16_le()?;
        let global_sequence = reader.read_i16_le()?;

        let ranges = M2Array::parse(reader)?;
        let times = M2Array::parse(reader)?;

        Ok(Self {
            identifier,
            data,
            bone_index,
            unknown,
            position,
            interp_type,
            global_sequence,
            ranges,
            times,
        })
    }

    /// Write an event to a writer based on the M2 version
    pub fn write<W: Write>(&self, writer: &mut W, _version: u32) -> Result<()> {
        writer.write_all(&self.identifier)?;
        writer.write_u32_le(self.data)?;
        writer.write_i16_le(self.bone_index)?;
        writer.write_u16_le(self.unknown)?;

        for &pos in &self.position {
            writer.write_f32_le(pos)?;
        }

        writer.write_u16_le(self.interp_type)?;
        writer.write_i16_le(self.global_sequence)?;

        self.ranges.write(writer)?;
        self.times.write(writer)?;

        Ok(())
    }

    /// Convert this event to a different version (no version differences for events)
    pub fn convert(&self, _target_version: M2Version) -> Self {
        self.clone()
    }

    /// Create a new event with default values
    pub fn new(identifier: [u8; 4], bone_index: i16) -> Self {
        Self {
            identifier,
            data: 0,
            bone_index,
            unknown: 0,
            position: [0.0, 0.0, 0.0],
            interp_type: 0,
            global_sequence: -1,
            ranges: M2Array::new(0, 0),
            times: M2Array::new(0, 0),
        }
    }

    /// Get the event identifier as a string
    pub fn identifier_str(&self) -> String {
        String::from_utf8_lossy(&self.identifier).to_string()
    }

    /// Returns the size of an event in bytes (always 44)
    pub const fn size() -> usize {
        44
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_event_parse_write() {
        let event = M2Event::new(*b"$CST", 25);

        // Test write
        let mut data = Vec::new();
        event
            .write(&mut data, M2Version::Vanilla.to_header_version())
            .unwrap();

        // Event should be 44 bytes
        assert_eq!(data.len(), 44);

        // Test parse
        let mut cursor = Cursor::new(data);
        let parsed = M2Event::parse(&mut cursor, M2Version::Vanilla.to_header_version()).unwrap();

        assert_eq!(parsed.identifier, *b"$CST");
        assert_eq!(parsed.bone_index, 25);
        assert_eq!(parsed.global_sequence, -1);
        assert_eq!(parsed.identifier_str(), "$CST");
    }

    #[test]
    fn test_event_size() {
        assert_eq!(M2Event::size(), 44);
    }

    #[test]
    fn test_event_types() {
        assert_eq!(M2EventType::from_u32(0), Some(M2EventType::Sound));
        assert_eq!(M2EventType::from_u32(5), Some(M2EventType::StandOrLand));
        assert_eq!(
            M2EventType::from_u32(9),
            Some(M2EventType::PlaySoundKitFromTable)
        );
        assert_eq!(M2EventType::from_u32(20), None);
    }
}
