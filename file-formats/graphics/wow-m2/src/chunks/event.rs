use wow_data::prelude::*;
use wow_data::types::C3Vector;
use wow_data_derive::{WowHeaderR, WowHeaderW};

use crate::version::M2Version;

use super::animation::M2AnimationBaseTrackHeader;

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

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub struct M2Event {
    pub identifier: [u8; 4],
    pub data: u32,
    pub bone_index: u32,
    pub position: C3Vector,
    #[wow_data(versioned)]
    pub enabled: M2AnimationBaseTrackHeader,
    // pub event_type: M2EventType,
    // pub event_track: M2Array<u32>,
}

impl M2Event {
    // /// Parse an event from a reader based on the M2 version
    // pub fn parse<R: Read>(reader: &mut R, _version: u32) -> Result<Self> {
    //     let id = reader.read_u32_le()?;
    //     let data = reader.read_u32_le()?;
    //     let bone_index = reader.read_u32_le()?;
    //
    //     let mut position = [0.0; 3];
    //     for item in &mut position {
    //         *item = reader.read_f32_le()?;
    //     }
    //
    //     let enabled = reader.read_u16_le()? != 0;
    //     reader.read_u16_le()?; // Skip padding
    //
    //     let event_type_raw = reader.read_u32_le()?;
    //     let event_type = M2EventType::from_u32(event_type_raw).unwrap_or(M2EventType::Sound);
    //
    //     let event_track = M2Array::parse(reader)?;
    //
    //     Ok(Self {
    //         id,
    //         data,
    //         bone_index,
    //         position,
    //         enabled,
    //         event_type,
    //         event_track,
    //     })
    // }

    // pub fn new(id: u32, bone_index: u32, event_type: M2EventType) -> Self {
    //     Self {
    //         id,
    //         data: 0,
    //         bone_index,
    //         position: [0.0, 0.0, 0.0],
    //         enabled: true,
    //         event_type,
    //         event_track: M2Array::new(0, 0),
    //     }
    // }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::io::Cursor;
//
//     #[test]
//     fn test_event_parse_write() {
//         let event = M2Event::new(1, 2, M2EventType::Sound);
//
//         // Test write
//         let mut data = Vec::new();
//         event
//             .write(&mut data, M2Version::Classic.to_header_version())
//             .unwrap();
//
//         // Test parse
//         let mut cursor = Cursor::new(data);
//         let parsed = M2Event::parse(&mut cursor, M2Version::Classic.to_header_version()).unwrap();
//
//         assert_eq!(parsed.id, 1);
//         assert_eq!(parsed.bone_index, 2);
//         assert_eq!(parsed.event_type, M2EventType::Sound);
//         assert!(parsed.enabled);
//     }
//
//     #[test]
//     fn test_event_types() {
//         assert_eq!(M2EventType::from_u32(0), Some(M2EventType::Sound));
//         assert_eq!(M2EventType::from_u32(5), Some(M2EventType::StandOrLand));
//         assert_eq!(
//             M2EventType::from_u32(9),
//             Some(M2EventType::PlaySoundKitFromTable)
//         );
//         assert_eq!(M2EventType::from_u32(20), None);
//     }
// }
