use wow_data::prelude::*;
use wow_data::types::C3Vector;
use wow_data_derive::{WowDataR, WowEnumFrom, WowHeaderR, WowHeaderW};

use crate::chunks::animation::M2AnimationTrackHeader;
use crate::version::MD20Version;

use super::animation::M2AnimationTrackData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, WowEnumFrom, WowHeaderR, WowHeaderW)]
#[wow_data(from_type=u32)]
pub enum M2AttachmentId {
    /// MountMain / ItemVisual0
    #[wow_data(lit = 0)]
    Shield = 0,
    /// ItemVisual1
    #[wow_data(lit = 1)]
    HandRight = 1,
    /// ItemVisual2
    #[wow_data(lit = 2)]
    HandLeft = 2,
    /// ItemVisual3
    #[wow_data(lit = 3)]
    ElbowRight = 3,
    /// ItemVisual4
    #[wow_data(lit = 4)]
    ElbowLeft = 4,
    #[wow_data(lit = 5)]
    ShoulderRight = 5,
    #[wow_data(lit = 6)]
    ShoulderLeft = 6,
    #[wow_data(lit = 7)]
    KneeRight = 7,
    #[wow_data(lit = 8)]
    KneeLeft = 8,
    #[wow_data(lit = 9)]
    HipRight = 9,
    #[wow_data(lit = 10)]
    HipLeft = 10,
    #[wow_data(lit = 11)]
    Helm = 11,
    #[wow_data(lit = 12)]
    Back = 12,
    #[wow_data(lit = 13)]
    ShoulderFlapRight = 13,
    #[wow_data(lit = 14)]
    ShoulderFlapLeft = 14,
    #[wow_data(lit = 15)]
    ChestBloodFront = 15,
    #[wow_data(lit = 16)]
    ChestBloodBack = 16,
    #[wow_data(lit = 17)]
    Breath = 17,
    #[wow_data(lit = 18)]
    PlayerName = 18,
    #[wow_data(lit = 19)]
    Base = 19,
    #[wow_data(lit = 20)]
    Head = 20,
    #[wow_data(lit = 21)]
    SpellLeftHand = 21,
    #[wow_data(lit = 22)]
    SpellRightHand = 22,
    #[wow_data(lit = 23)]
    Special1 = 23,
    #[wow_data(lit = 24)]
    Special2 = 24,
    #[wow_data(lit = 25)]
    Special3 = 25,
    #[wow_data(lit = 26)]
    SheathMainHand = 26,
    #[wow_data(lit = 27)]
    SheathOffHand = 27,
    #[wow_data(lit = 28)]
    SheathShield = 28,
    #[wow_data(lit = 29)]
    PlayerNameMounted = 29,
    #[wow_data(lit = 30)]
    LargeWeaponLeft = 30,
    #[wow_data(lit = 31)]
    LargeWeaponRight = 31,
    #[wow_data(lit = 32)]
    HipWeaponLeft = 32,
    #[wow_data(lit = 33)]
    HipWeaponRight = 33,
    #[wow_data(lit = 34)]
    Chest = 34,
    #[wow_data(lit = 35)]
    HandArrow = 35,
    #[wow_data(lit = 36)]
    Bullet = 36,
    #[wow_data(lit = 37)]
    SpellHandOmni = 37,
    #[wow_data(lit = 38)]
    SpellHandDirected = 38,
    #[wow_data(lit = 39)]
    VehicleSeat1 = 39,
    #[wow_data(lit = 40)]
    VehicleSeat2 = 40,
    #[wow_data(lit = 41)]
    VehicleSeat3 = 41,
    #[wow_data(lit = 42)]
    VehicleSeat4 = 42,
    #[wow_data(lit = 43)]
    VehicleSeat5 = 43,
    #[wow_data(lit = 44)]
    VehicleSeat6 = 44,
    #[wow_data(lit = 45)]
    VehicleSeat7 = 45,
    #[wow_data(lit = 46)]
    VehicleSeat8 = 46,
    #[wow_data(lit = 47)]
    LeftFoot = 47,
    #[wow_data(lit = 48)]
    RightFoot = 48,
    #[wow_data(lit = 49)]
    ShieldNoGlove = 49,
    #[wow_data(lit = 50)]
    SpineLow = 50,
    #[wow_data(lit = 51)]
    AlteredShoulderR = 51,
    #[wow_data(lit = 52)]
    AlteredShoulderL = 52,
    #[wow_data(lit = 53)]
    BeltBuckle = 53,
    #[wow_data(lit = 54)]
    SheathCrossbow = 54,
    #[wow_data(lit = 55)]
    HeadTop = 55,
    #[wow_data(lit = 56)]
    VirtualSpellDirected = 56,
    #[wow_data(lit = 57)]
    Backpack = 57,
    #[wow_data(lit = 58)]
    Unknown = 60,
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub struct M2AttachmentHeader {
    pub id: M2AttachmentId,
    pub bone_index: u16,
    pub parent_bone_flying: u16,
    pub position: C3Vector,
    #[wow_data(versioned)]
    pub animate_attached: M2AnimationTrackHeader<u8>,
}

#[derive(Debug, Clone, WowDataR)]
#[wow_data(version = MD20Version, header = M2AttachmentHeader)]
pub struct M2AttachmentData {
    #[wow_data(versioned)]
    pub animate_attached: M2AnimationTrackData<u8>,
}

#[derive(Debug, Clone)]
pub struct M2Attachment {
    pub header: M2AttachmentHeader,
    pub data: M2AttachmentData,
}
