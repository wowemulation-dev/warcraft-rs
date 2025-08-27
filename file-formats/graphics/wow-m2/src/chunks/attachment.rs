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
    #[wow_data(expr = 0)]
    Shield = 0,
    /// ItemVisual1
    #[wow_data(expr = 1)]
    HandRight = 1,
    /// ItemVisual2
    #[wow_data(expr = 2)]
    HandLeft = 2,
    /// ItemVisual3
    #[wow_data(expr = 3)]
    ElbowRight = 3,
    /// ItemVisual4
    #[wow_data(expr = 4)]
    ElbowLeft = 4,
    #[wow_data(expr = 5)]
    ShoulderRight = 5,
    #[wow_data(expr = 6)]
    ShoulderLeft = 6,
    #[wow_data(expr = 7)]
    KneeRight = 7,
    #[wow_data(expr = 8)]
    KneeLeft = 8,
    #[wow_data(expr = 9)]
    HipRight = 9,
    #[wow_data(expr = 10)]
    HipLeft = 10,
    #[wow_data(expr = 11)]
    Helm = 11,
    #[wow_data(expr = 12)]
    Back = 12,
    #[wow_data(expr = 13)]
    ShoulderFlapRight = 13,
    #[wow_data(expr = 14)]
    ShoulderFlapLeft = 14,
    #[wow_data(expr = 15)]
    ChestBloodFront = 15,
    #[wow_data(expr = 16)]
    ChestBloodBack = 16,
    #[wow_data(expr = 17)]
    Breath = 17,
    #[wow_data(expr = 18)]
    PlayerName = 18,
    #[wow_data(expr = 19)]
    Base = 19,
    #[wow_data(expr = 20)]
    Head = 20,
    #[wow_data(expr = 21)]
    SpellLeftHand = 21,
    #[wow_data(expr = 22)]
    SpellRightHand = 22,
    #[wow_data(expr = 23)]
    Special1 = 23,
    #[wow_data(expr = 24)]
    Special2 = 24,
    #[wow_data(expr = 25)]
    Special3 = 25,
    #[wow_data(expr = 26)]
    SheathMainHand = 26,
    #[wow_data(expr = 27)]
    SheathOffHand = 27,
    #[wow_data(expr = 28)]
    SheathShield = 28,
    #[wow_data(expr = 29)]
    PlayerNameMounted = 29,
    #[wow_data(expr = 30)]
    LargeWeaponLeft = 30,
    #[wow_data(expr = 31)]
    LargeWeaponRight = 31,
    #[wow_data(expr = 32)]
    HipWeaponLeft = 32,
    #[wow_data(expr = 33)]
    HipWeaponRight = 33,
    #[wow_data(expr = 34)]
    Chest = 34,
    #[wow_data(expr = 35)]
    HandArrow = 35,
    #[wow_data(expr = 36)]
    Bullet = 36,
    #[wow_data(expr = 37)]
    SpellHandOmni = 37,
    #[wow_data(expr = 38)]
    SpellHandDirected = 38,
    #[wow_data(expr = 39)]
    VehicleSeat1 = 39,
    #[wow_data(expr = 40)]
    VehicleSeat2 = 40,
    #[wow_data(expr = 41)]
    VehicleSeat3 = 41,
    #[wow_data(expr = 42)]
    VehicleSeat4 = 42,
    #[wow_data(expr = 43)]
    VehicleSeat5 = 43,
    #[wow_data(expr = 44)]
    VehicleSeat6 = 44,
    #[wow_data(expr = 45)]
    VehicleSeat7 = 45,
    #[wow_data(expr = 46)]
    VehicleSeat8 = 46,
    #[wow_data(expr = 47)]
    LeftFoot = 47,
    #[wow_data(expr = 48)]
    RightFoot = 48,
    #[wow_data(expr = 49)]
    ShieldNoGlove = 49,
    #[wow_data(expr = 50)]
    SpineLow = 50,
    #[wow_data(expr = 51)]
    AlteredShoulderR = 51,
    #[wow_data(expr = 52)]
    AlteredShoulderL = 52,
    #[wow_data(expr = 53)]
    BeltBuckle = 53,
    #[wow_data(expr = 54)]
    SheathCrossbow = 54,
    #[wow_data(expr = 55)]
    HeadTop = 55,
    #[wow_data(expr = 56)]
    VirtualSpellDirected = 56,
    #[wow_data(expr = 57)]
    Backpack = 57,
    #[wow_data(expr = 58)]
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
