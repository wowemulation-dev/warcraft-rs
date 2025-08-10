use wow_data::prelude::*;
use wow_data::types::C3Vector;
use wow_data_derive::{WowDataR, WowHeaderR, WowHeaderW};

use crate::chunks::animation::M2AnimationTrackHeader;
use crate::version::M2Version;

use super::animation::M2AnimationTrackData;

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

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub struct M2AttachmentHeader {
    pub id: u32,
    pub bone_index: u16,
    pub parent_bone_flying: u16,
    // pub attachment_type: M2AttachmentType,
    // pub animation_data: u16,
    pub position: C3Vector,
    #[wow_data(versioned)]
    pub animate_attached: M2AnimationTrackHeader<u8>,
}

#[derive(Debug, Clone, WowDataR)]
#[wow_data(version = M2Version, header = M2AttachmentHeader)]
pub struct M2AttachmentData {
    #[wow_data(versioned)]
    pub animate_attached: M2AnimationTrackData<u8>,
}

#[derive(Debug, Clone)]
pub struct M2Attachment {
    pub header: M2AttachmentHeader,
    pub data: M2AttachmentData,
}
