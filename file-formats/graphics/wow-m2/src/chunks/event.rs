use wow_data::prelude::*;
use wow_data::types::C3Vector;
use wow_data_derive::{WowDataR, WowHeaderR, WowHeaderW};

use crate::version::M2Version;

use super::animation::{M2AnimationBaseTrackData, M2AnimationBaseTrackHeader};

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
pub struct M2EventHeader {
    pub identifier: [u8; 4],
    pub data: u32,
    pub bone_index: u32,
    pub position: C3Vector,
    #[wow_data(versioned)]
    pub enabled: M2AnimationBaseTrackHeader,
    // pub event_type: M2EventType,
    // pub event_track: M2Array<u32>,
}

#[derive(Debug, Clone, WowDataR)]
#[wow_data(version = M2Version, header = M2EventHeader)]
pub struct M2EventData {
    #[wow_data(versioned)]
    pub enabled: M2AnimationBaseTrackData,
}

#[derive(Debug, Clone)]
pub struct M2Event {
    pub header: M2EventHeader,
    pub data: M2EventData,
}
