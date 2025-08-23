use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::C3Vector;
use wow_data_derive::{WowDataR, WowHeaderR, WowHeaderW};

use crate::chunks::animation::M2AnimationTrackHeader;
use crate::version::M2Version;
use crate::{M2Error, Result};

use super::animation::M2AnimationTrackData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M2AttachmentId {
    /// MountMain / ItemVisual0
    Shield = 0,
    /// ItemVisual1
    HandRight = 1,
    /// ItemVisual2
    HandLeft = 2,
    /// ItemVisual3
    ElbowRight = 3,
    /// ItemVisual4
    ElbowLeft = 4,
    ShoulderRight = 5,
    ShoulderLeft = 6,
    KneeRight = 7,
    KneeLeft = 8,
    HipRight = 9,
    HipLeft = 10,
    Helm = 11,
    Back = 12,
    ShoulderFlapRight = 13,
    ShoulderFlapLeft = 14,
    ChestBloodFront = 15,
    ChestBloodBack = 16,
    Breath = 17,
    PlayerName = 18,
    Base = 19,
    Head = 20,
    SpellLeftHand = 21,
    SpellRightHand = 22,
    Special1 = 23,
    Special2 = 24,
    Special3 = 25,
    SheathMainHand = 26,
    SheathOffHand = 27,
    SheathShield = 28,
    PlayerNameMounted = 29,
    LargeWeaponLeft = 30,
    LargeWeaponRight = 31,
    HipWeaponLeft = 32,
    HipWeaponRight = 33,
    Chest = 34,
    HandArrow = 35,
    Bullet = 36,
    SpellHandOmni = 37,
    SpellHandDirected = 38,
    VehicleSeat1 = 39,
    VehicleSeat2 = 40,
    VehicleSeat3 = 41,
    VehicleSeat4 = 42,
    VehicleSeat5 = 43,
    VehicleSeat6 = 44,
    VehicleSeat7 = 45,
    VehicleSeat8 = 46,
    LeftFoot = 47,
    RightFoot = 48,
    ShieldNoGlove = 49,
    SpineLow = 50,
    AlteredShoulderR = 51,
    AlteredShoulderL = 52,
    BeltBuckle = 53,
    SheathCrossbow = 54,
    HeadTop = 55,
    VirtualSpellDirected = 56,
    Backpack = 57,
    Unknown = 60,
}

impl TryFrom<u32> for M2AttachmentId {
    type Error = M2Error;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            0 => Ok(Self::Shield),
            1 => Ok(Self::HandRight),
            2 => Ok(Self::HandLeft),
            3 => Ok(Self::ElbowRight),
            4 => Ok(Self::ElbowLeft),
            5 => Ok(Self::ShoulderRight),
            6 => Ok(Self::ShoulderLeft),
            7 => Ok(Self::KneeRight),
            8 => Ok(Self::KneeLeft),
            9 => Ok(Self::HipRight),
            10 => Ok(Self::HipLeft),
            11 => Ok(Self::Helm),
            12 => Ok(Self::Back),
            13 => Ok(Self::ShoulderFlapRight),
            14 => Ok(Self::ShoulderFlapLeft),
            15 => Ok(Self::ChestBloodFront),
            16 => Ok(Self::ChestBloodBack),
            17 => Ok(Self::Breath),
            18 => Ok(Self::PlayerName),
            19 => Ok(Self::Base),
            20 => Ok(Self::Head),
            21 => Ok(Self::SpellLeftHand),
            22 => Ok(Self::SpellRightHand),
            23 => Ok(Self::Special1),
            24 => Ok(Self::Special2),
            25 => Ok(Self::Special3),
            26 => Ok(Self::SheathMainHand),
            27 => Ok(Self::SheathOffHand),
            28 => Ok(Self::SheathShield),
            29 => Ok(Self::PlayerNameMounted),
            30 => Ok(Self::LargeWeaponLeft),
            31 => Ok(Self::LargeWeaponRight),
            32 => Ok(Self::HipWeaponLeft),
            33 => Ok(Self::HipWeaponRight),
            34 => Ok(Self::Chest),
            35 => Ok(Self::HandArrow),
            36 => Ok(Self::Bullet),
            37 => Ok(Self::SpellHandOmni),
            38 => Ok(Self::SpellHandDirected),
            39 => Ok(Self::VehicleSeat1),
            40 => Ok(Self::VehicleSeat2),
            41 => Ok(Self::VehicleSeat3),
            42 => Ok(Self::VehicleSeat4),
            43 => Ok(Self::VehicleSeat5),
            44 => Ok(Self::VehicleSeat6),
            45 => Ok(Self::VehicleSeat7),
            46 => Ok(Self::VehicleSeat8),
            47 => Ok(Self::LeftFoot),
            48 => Ok(Self::RightFoot),
            49 => Ok(Self::ShieldNoGlove),
            50 => Ok(Self::SpineLow),
            51 => Ok(Self::AlteredShoulderR),
            52 => Ok(Self::AlteredShoulderL),
            53 => Ok(Self::BeltBuckle),
            54 => Ok(Self::SheathCrossbow),
            55 => Ok(Self::HeadTop),
            56 => Ok(Self::VirtualSpellDirected),
            57 => Ok(Self::Backpack),
            60 => Ok(Self::Unknown),
            _ => Err(M2Error::ParseError(format!(
                "Invalid attachment id value: {}",
                value
            ))),
        }
    }
}

impl From<M2AttachmentId> for u32 {
    fn from(value: M2AttachmentId) -> Self {
        match value {
            M2AttachmentId::Shield => 0,
            M2AttachmentId::HandRight => 1,
            M2AttachmentId::HandLeft => 2,
            M2AttachmentId::ElbowRight => 3,
            M2AttachmentId::ElbowLeft => 4,
            M2AttachmentId::ShoulderRight => 5,
            M2AttachmentId::ShoulderLeft => 6,
            M2AttachmentId::KneeRight => 7,
            M2AttachmentId::KneeLeft => 8,
            M2AttachmentId::HipRight => 9,
            M2AttachmentId::HipLeft => 10,
            M2AttachmentId::Helm => 11,
            M2AttachmentId::Back => 12,
            M2AttachmentId::ShoulderFlapRight => 13,
            M2AttachmentId::ShoulderFlapLeft => 14,
            M2AttachmentId::ChestBloodFront => 15,
            M2AttachmentId::ChestBloodBack => 16,
            M2AttachmentId::Breath => 17,
            M2AttachmentId::PlayerName => 18,
            M2AttachmentId::Base => 19,
            M2AttachmentId::Head => 20,
            M2AttachmentId::SpellLeftHand => 21,
            M2AttachmentId::SpellRightHand => 22,
            M2AttachmentId::Special1 => 23,
            M2AttachmentId::Special2 => 24,
            M2AttachmentId::Special3 => 25,
            M2AttachmentId::SheathMainHand => 26,
            M2AttachmentId::SheathOffHand => 27,
            M2AttachmentId::SheathShield => 28,
            M2AttachmentId::PlayerNameMounted => 29,
            M2AttachmentId::LargeWeaponLeft => 30,
            M2AttachmentId::LargeWeaponRight => 31,
            M2AttachmentId::HipWeaponLeft => 32,
            M2AttachmentId::HipWeaponRight => 33,
            M2AttachmentId::Chest => 34,
            M2AttachmentId::HandArrow => 35,
            M2AttachmentId::Bullet => 36,
            M2AttachmentId::SpellHandOmni => 37,
            M2AttachmentId::SpellHandDirected => 38,
            M2AttachmentId::VehicleSeat1 => 39,
            M2AttachmentId::VehicleSeat2 => 40,
            M2AttachmentId::VehicleSeat3 => 41,
            M2AttachmentId::VehicleSeat4 => 42,
            M2AttachmentId::VehicleSeat5 => 43,
            M2AttachmentId::VehicleSeat6 => 44,
            M2AttachmentId::VehicleSeat7 => 45,
            M2AttachmentId::VehicleSeat8 => 46,
            M2AttachmentId::LeftFoot => 47,
            M2AttachmentId::RightFoot => 48,
            M2AttachmentId::ShieldNoGlove => 49,
            M2AttachmentId::SpineLow => 50,
            M2AttachmentId::AlteredShoulderR => 51,
            M2AttachmentId::AlteredShoulderL => 52,
            M2AttachmentId::BeltBuckle => 53,
            M2AttachmentId::SheathCrossbow => 54,
            M2AttachmentId::HeadTop => 55,
            M2AttachmentId::VirtualSpellDirected => 56,
            M2AttachmentId::Backpack => 57,
            M2AttachmentId::Unknown => 60,
        }
    }
}

impl WowHeaderR for M2AttachmentId {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        Ok(u32::wow_read(reader)?.try_into()?)
    }
}

impl WowHeaderW for M2AttachmentId {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        u32::wow_write(&(*self).into(), writer)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        u32::wow_size(&(*self).into())
    }
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub struct M2AttachmentHeader {
    pub id: M2AttachmentId,
    pub bone_index: u16,
    pub parent_bone_flying: u16,
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
