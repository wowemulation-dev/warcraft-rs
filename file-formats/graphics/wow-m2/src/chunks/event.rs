use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::C3Vector;
use wow_data_derive::{WowDataR, WowHeaderR, WowHeaderW};

use crate::version::M2Version;

use super::animation::{M2AnimationBaseTrackData, M2AnimationBaseTrackHeader};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M2EventIdentifier {
    /// PlaySoundKit (customAttack[x]) | soundEffect ID is defined by CreatureSoundDataRec::m_customAttack[x]
    AH0,
    /// PlaySoundKit (customAttack[x]) | soundEffect ID is defined by CreatureSoundDataRec::m_customAttack[x]
    AH1,
    /// PlaySoundKit (customAttack[x]) | soundEffect ID is defined by CreatureSoundDataRec::m_customAttack[x]
    AH2,
    /// PlaySoundKit (customAttack[x]) | soundEffect ID is defined by CreatureSoundDataRec::m_customAttack[x]
    AH3,
    /// BowMissleDestination | RangedWeapon | Bow Middle
    BMD,
    /// Vehicles | CGUnit_C::ComputeMissileTrajectory | Position used as MissileFirePos.
    AIM,
    /// anim_swap_event / DisplayTransition | Unit | CUnitDisplayTransition_C::UpdateState(1) or CUnitDisplayTransition_C::HandleAnimSwapEvent
    ALT,
    /// FootstepAnimEventHit (left) | Unit | Backwards
    BL0,
    /// FootstepAnimEventHit (left) | Unit | Backwards
    BL1,
    /// FootstepAnimEventHit (left) | Unit | Backwards
    BL2,
    /// FootstepAnimEventHit (left) | Unit | Backwards
    BL3,
    /// FootstepAnimEventHit (right) | Unit | Backwards
    BR0,
    /// FootstepAnimEventHit (right) | Unit | Backwards
    BR1,
    /// FootstepAnimEventHit (right) | Unit | Backwards
    BR2,
    /// FootstepAnimEventHit (right) | Unit | Backwards
    BR3,
    /// PlaySoundKit (birth) | soundEffect ID is defined by CreatureSoundDatarec::m_birthSoundID
    BRT,
    /// Breath | Unit | All situations, where nothing happens or breathing. | Adds Special Unit Effect based on unit state (under water, in-snow, …)
    BTH,
    /// PlayRangedItemPull (Bow Pull) | Unit | LoadRifle, LoadBow
    BWP,
    /// BowRelease | Unit | AttackRifle, AttackBow, AttackThrown
    BWR,
    /// Unit | Attack*, *Unarmed, ShieldBash, Special* | attack hold? CGUnit_C::HandleCombatAnimEvent
    CAH,
    /// Unit | mostly not fired, AttackThrown | Fishing Pole CEffect::DrawFishingString needs this on the model for getting the string attachments.
    CCH,
    /// Unit | CGCamera::UpdateMountHeightOrOffset | CGCamera::UpdateMountHeightOrOffset: Only z is used. Non-animated. Not used if $CMA
    CFM,
    /// Unit | not fired | probably does not exist?!
    CHD,
    /// Unit | CGCamera::UpdateMountHeightOrOffset: Position for camera
    CMA,
    /// PlayCombatActionAnimKit | parry, anims, depending on some state, also some callback which might do more
    CPP,
    /// soundEntryId | PlayEmoteSound | Unit | Emote*
    CSD,
    /// release_missiles_on_next_update if has_pending_missiles (left) | Unit | AttackRifle, SpellCast*, ChannelCast* | "x is {L or R} (""Left/right hand"") (?)"
    CSL,
    /// release_missiles_on_next_update if has_pending_missiles (right) | Unit | AttackBow, AttackRifle, AttackThrown, SpellCast*, ChannelCast* | "x is {L or R} (""Left/right hand"") (?)"
    CSR,
    /// PlayWeaponSwooshSound | sound played depends on CGUnit_C::GetWeaponSwingType
    CSS,
    /// release_missiles_on_next_update if has_pending_missiles | Unit | Attack*, *Unarmed, ShieldBash, Special*, SpellCast, Parry*, EmoteEat, EmoteRoar, Kick, ... | $CSL/R/T are also used in CGUnit_C::ComputeDefaultMissileFirePos.
    CST,
    ///  ? | Data: SoundEntriesAdvanced.dbc, Sound — Not present in 6.0.1.18179
    CVS,
    /// DestroyEmitter | MapObj
    DSE,
    /// soundEntryId | DoodadSoundLoop (low priority) | GO
    DSL,
    /// soundEntryId | DoodadSoundOneShot | GO
    DSO,
    /// DeathThud + LootEffect | Unit | Death, Drown, Knockdown | """I'm dead now!"", UnitCombat_C, this plays death sounds and more." Note that this is NOT triggering CreatureSoundDataRec::m_soundDeathID, but that is just always triggered as soon as the death animation plays.
    DTH,
    /// object package state enter 3, exit 2, 4, 5
    EAC,
    /// object package state enter 5, exit 3, 4, 2
    EDC,
    /// object package state enter 4, exit 3, 2, 5
    EMV,
    /// PlayEmoteStateSound | Unit | soundEffect ID is implicit by currently played emote
    ESD,
    /// object package state enter 2, exit 3, 4, 5
    EWT,
    /// PlayFidgetSound | CreatureSoundDataRec::m_soundFidget (only has 5 entries, so don’t use 6-9)
    FD0,
    /// PlayFidgetSound | CreatureSoundDataRec::m_soundFidget (only has 5 entries, so don’t use 6-9)
    FD1,
    /// PlayFidgetSound | CreatureSoundDataRec::m_soundFidget (only has 5 entries, so don’t use 6-9)
    FD2,
    /// PlayFidgetSound | CreatureSoundDataRec::m_soundFidget (only has 5 entries, so don’t use 6-9)
    FD3,
    /// PlayFidgetSound | CreatureSoundDataRec::m_soundFidget (only has 5 entries, so don’t use 6-9)
    FD4,
    /// PlayFidgetSound | CreatureSoundDataRec::m_soundFidget (only has 5 entries, so don’t use 6-9)
    FD5,
    /// PlayFidgetSound | CreatureSoundDataRec::m_soundFidget (only has 5 entries, so don’t use 6-9)
    FD6,
    /// PlayFidgetSound | CreatureSoundDataRec::m_soundFidget (only has 5 entries, so don’t use 6-9)
    FD7,
    /// PlayFidgetSound | CreatureSoundDataRec::m_soundFidget (only has 5 entries, so don’t use 6-9)
    FD8,
    /// PlayFidgetSound | CreatureSoundDataRec::m_soundFidget (only has 5 entries, so don’t use 6-9)
    FD9,
    /// PlayUnitSound (stand) | soundEffect ID is defined by CreatureSoundDataRec::m_soundStandID
    FDX,
    /// FootstepAnimEventHit (left) | Foot Left Forward
    FL0,
    /// FootstepAnimEventHit (left) | Foot Left Forward
    FL1,
    /// FootstepAnimEventHit (left) | Foot Left Forward
    FL2,
    /// FootstepAnimEventHit (left) | Foot Left Forward
    FL3,
    /// FootstepAnimEventHit (right) | Foot Right Forward
    FR0,
    /// FootstepAnimEventHit (right) | Foot Right Forward
    FR1,
    /// FootstepAnimEventHit (right) | Foot Right Forward
    FR2,
    /// FootstepAnimEventHit (right) | Foot Right Forward
    FR3,
    /// HandleFootfallAnimEvent | Unit | Walk, Run (multiple times), ... | Plays some sound. Footstep? Also seen at several emotes etc. where feet are moved. CGUnit_C::HandleFootfallAnimEvent
    FSD,
    /// GameObject_C_PlayAnimatedSound | soundEffect ID is defined by GameObjectDisplayInfoRec::m_Sound[x + 6] ({Custom0, Custom1, Custom2, Custom3})
    GC0,
    /// GameObject_C_PlayAnimatedSound | soundEffect ID is defined by GameObjectDisplayInfoRec::m_Sound[x + 6] ({Custom0, Custom1, Custom2, Custom3})
    GC1,
    /// GameObject_C_PlayAnimatedSound | soundEffect ID is defined by GameObjectDisplayInfoRec::m_Sound[x + 6] ({Custom0, Custom1, Custom2, Custom3})
    GC2,
    /// GameObject_C_PlayAnimatedSound | soundEffect ID is defined by GameObjectDisplayInfoRec::m_Sound[x + 6] ({Custom0, Custom1, Custom2, Custom3})
    GC3,
    /// GameObject_C_PlayAnimatedSound | soundEffect ID is defined by GameObjectDisplayInfoRec::m_Sound[x] ({Stand, Open, Loop, Close, Destroy, Opened})
    GO0,
    /// GameObject_C_PlayAnimatedSound | soundEffect ID is defined by GameObjectDisplayInfoRec::m_Sound[x] ({Stand, Open, Loop, Close, Destroy, Opened})
    GO1,
    /// GameObject_C_PlayAnimatedSound | soundEffect ID is defined by GameObjectDisplayInfoRec::m_Sound[x] ({Stand, Open, Loop, Close, Destroy, Opened})
    GO2,
    /// GameObject_C_PlayAnimatedSound | soundEffect ID is defined by GameObjectDisplayInfoRec::m_Sound[x] ({Stand, Open, Loop, Close, Destroy, Opened})
    GO3,
    /// GameObject_C_PlayAnimatedSound | soundEffect ID is defined by GameObjectDisplayInfoRec::m_Sound[x] ({Stand, Open, Loop, Close, Destroy, Opened})
    GO4,
    /// GameObject_C_PlayAnimatedSound | soundEffect ID is defined by GameObjectDisplayInfoRec::m_Sound[x] ({Stand, Open, Loop, Close, Destroy, Opened})
    GO5,
    /// PlayWoundAnimKit | Unit | Attack*, *Unarmed, ShieldBash, Special* | soundEntryId depends on SpellVisualKit
    HIT,
    ///  ? | MapLoad.cpp -- not found in 6.0.1.18179
    KVS,
    /// FootstepAnimEventHit (left) | Running
    RL0,
    /// FootstepAnimEventHit (left) | Running
    RL1,
    /// FootstepAnimEventHit (left) | Running
    RL2,
    /// FootstepAnimEventHit (left) | Running
    RL3,
    /// FootstepAnimEventHit (right) | Running
    RR0,
    /// FootstepAnimEventHit (right) | Running
    RR1,
    /// FootstepAnimEventHit (right) | Running
    RR2,
    /// FootstepAnimEventHit (right) | Running
    RR3,
    /// PlaySoundKit (spellCastDirectedSound) | soundEffect ID is defined by CreatureSoundDataRec::m_spellCastDirectedSoundID
    SCD,
    /// spellEffectCameraShakesID | AddShake | GO
    SHK,
    /// ExchangeSheathedWeapon (left) | Sheath, HipSheath
    SHL,
    /// ExchangeSheathedWeapon (right) | Sheath, HipSheath
    SHR,
    /// FootstepAnimEventHit (left) | Stop, (JumpEnd), (Shuffle*) | Stop
    SL0,
    /// FootstepAnimEventHit (left) | Stop, (JumpEnd), (Shuffle*) | Stop
    SL1,
    /// FootstepAnimEventHit (left) | Stop, (JumpEnd), (Shuffle*) | Stop
    SL2,
    /// FootstepAnimEventHit (left) | Stop, (JumpEnd), (Shuffle*) | Stop
    SL3,
    /// PlaySoundKit (submerged) | soundEffect ID is defined by CreatureSoundDatarec::m_submergedSoundID
    SMD,
    /// PlaySoundKit (submerge) | soundEffect ID is defined by CreatureSoundDatarec::m_submergeSoundID
    SMG,
    /// soundEntryId | PlaySoundKit (custom) | GO
    SND,
    /// FootstepAnimEventHit (right) | Stop, (JumpEnd), (Shuffle*) | Stop
    SR0,
    /// FootstepAnimEventHit (right) | Stop, (JumpEnd), (Shuffle*) | Stop
    SR1,
    /// FootstepAnimEventHit (right) | Stop, (JumpEnd), (Shuffle*) | Stop
    SR2,
    /// FootstepAnimEventHit (right) | Stop, (JumpEnd), (Shuffle*) | Stop
    SR3,
    /// Mounts | MountTransitionObject::UpdateCharacterData | Not seen in 6.0.1.18179 -- x is {E and B} , sequence time is taken of both, pivot of $STB. (Also, attachment info for attachment 0)
    STx,
    /// HandleSpellEventSound | Unit | EmoteWork*, UseStanding* | soundEffect ID is implicit by SpellRec
    TRD,
    /// HandleBoneAnimGrabEvent
    VG0,
    /// HandleBoneAnimGrabEvent
    VG1,
    /// HandleBoneAnimGrabEvent
    VG2,
    /// HandleBoneAnimGrabEvent
    VG3,
    /// HandleBoneAnimGrabEvent
    VG4,
    /// HandleBoneAnimGrabEvent
    VG5,
    /// HandleBoneAnimGrabEvent
    VG6,
    /// HandleBoneAnimGrabEvent
    VG7,
    /// HandleBoneAnimGrabEvent
    VG8,
    /// HandleBoneAnimThrowEvent
    VT0,
    /// HandleBoneAnimThrowEvent
    VT1,
    /// HandleBoneAnimThrowEvent
    VT2,
    /// HandleBoneAnimThrowEvent
    VT3,
    /// HandleBoneAnimThrowEvent
    VT4,
    /// HandleBoneAnimThrowEvent
    VT5,
    /// HandleBoneAnimThrowEvent
    VT6,
    /// HandleBoneAnimThrowEvent
    VT7,
    /// HandleBoneAnimThrowEvent
    VT8,
    /// PlayUnitSound (wingGlide) | soundEffect ID is defined by CreatureSoundDataRec::m_soundWingGlideID
    WGG,
    /// FootstepAnimEventHit (left)
    WL0,
    /// FootstepAnimEventHit (left)
    WL1,
    /// FootstepAnimEventHit (left)
    WL2,
    /// FootstepAnimEventHit (left)
    WL3,
    /// Model Weapon Left Bot
    WLB,
    /// Model Weapon Left Top
    WLT,
    /// PlayUnitSound (wingFlap) | soundEffect ID is defined by CreatureSoundDataRec::m_soundWingFlapID
    WNG,
    /// FootstepAnimEventHit (right)
    WR0,
    /// FootstepAnimEventHit (right)
    WR1,
    /// FootstepAnimEventHit (right)
    WR2,
    /// FootstepAnimEventHit (right)
    WR3,
    /// Model Weapon Right Bot
    WRB,
    /// Model Weapon Right Top
    WRT,
    /// Weapons | Bow Bottom, Weapon Trail Bottom position, also used for Bow String
    WTB,
    /// Weapons | Bow Top, Weapon Trail Top position
    WTT,
    ///  ? | Calls some function in the Object VMT. -- Not seen in 6.0.1.18179
    WWG,
    ///  ? | exploding ballista, that one has a really fucked up block. Oo
    DEST,
    /// Unit | not fired | Data: ?, seen on multiple models. Basilisk for example. (6801)
    POIN,
    ///  ? | Data: 601+, Used on wheels at vehicles.
    WHEE,
    ///  ? | Purpose unknown. Seen in well_vortex01.m2
    BOTT,
    ///  ? | Purpose unknown. Seen in well_vortex01.m2
    TOP0,

    Unknown([u8; 4]),
}

impl From<[u8; 4]> for M2EventIdentifier {
    fn from(value: [u8; 4]) -> Self {
        match value {
            [0x24, 0x41, 0x48, 0x30] => Self::AH0,
            [0x24, 0x41, 0x48, 0x31] => Self::AH1,
            [0x24, 0x41, 0x48, 0x32] => Self::AH2,
            [0x24, 0x41, 0x48, 0x33] => Self::AH3,
            [0x24, 0x42, 0x4d, 0x44] => Self::BMD,
            [0x24, 0x41, 0x49, 0x4d] => Self::AIM,
            [0x24, 0x41, 0x4c, 0x54] => Self::ALT,
            [0x24, 0x42, 0x4c, 0x30] => Self::BL0,
            [0x24, 0x42, 0x4c, 0x31] => Self::BL1,
            [0x24, 0x42, 0x4c, 0x32] => Self::BL2,
            [0x24, 0x42, 0x4c, 0x33] => Self::BL3,
            [0x24, 0x42, 0x52, 0x30] => Self::BR0,
            [0x24, 0x42, 0x52, 0x31] => Self::BR1,
            [0x24, 0x42, 0x52, 0x32] => Self::BR2,
            [0x24, 0x42, 0x52, 0x33] => Self::BR3,
            [0x24, 0x42, 0x52, 0x54] => Self::BRT,
            [0x24, 0x42, 0x54, 0x48] => Self::BTH,
            [0x24, 0x42, 0x57, 0x50] => Self::BWP,
            [0x24, 0x42, 0x57, 0x52] => Self::BWR,
            [0x24, 0x43, 0x41, 0x48] => Self::CAH,
            [0x24, 0x43, 0x43, 0x48] => Self::CCH,
            [0x24, 0x43, 0x46, 0x4d] => Self::CFM,
            [0x24, 0x43, 0x48, 0x44] => Self::CHD,
            [0x24, 0x43, 0x4d, 0x41] => Self::CMA,
            [0x24, 0x43, 0x50, 0x50] => Self::CPP,
            [0x24, 0x43, 0x53, 0x44] => Self::CSD,
            [0x24, 0x43, 0x53, 0x4c] => Self::CSL,
            [0x24, 0x43, 0x53, 0x52] => Self::CSR,
            [0x24, 0x43, 0x53, 0x53] => Self::CSS,
            [0x24, 0x43, 0x53, 0x54] => Self::CST,
            [0x24, 0x43, 0x56, 0x53] => Self::CVS,
            [0x24, 0x44, 0x53, 0x45] => Self::DSE,
            [0x24, 0x44, 0x53, 0x4c] => Self::DSL,
            [0x24, 0x44, 0x53, 0x4f] => Self::DSO,
            [0x24, 0x44, 0x54, 0x48] => Self::DTH,
            [0x24, 0x45, 0x41, 0x43] => Self::EAC,
            [0x24, 0x45, 0x44, 0x43] => Self::EDC,
            [0x24, 0x45, 0x4d, 0x56] => Self::EMV,
            [0x24, 0x45, 0x53, 0x44] => Self::ESD,
            [0x24, 0x45, 0x57, 0x54] => Self::EWT,
            [0x24, 0x46, 0x44, 0x30] => Self::FD0,
            [0x24, 0x46, 0x44, 0x31] => Self::FD1,
            [0x24, 0x46, 0x44, 0x32] => Self::FD2,
            [0x24, 0x46, 0x44, 0x33] => Self::FD3,
            [0x24, 0x46, 0x44, 0x34] => Self::FD4,
            [0x24, 0x46, 0x44, 0x35] => Self::FD5,
            [0x24, 0x46, 0x44, 0x36] => Self::FD6,
            [0x24, 0x46, 0x44, 0x37] => Self::FD7,
            [0x24, 0x46, 0x44, 0x38] => Self::FD8,
            [0x24, 0x46, 0x44, 0x39] => Self::FD9,
            [0x24, 0x46, 0x44, 0x58] => Self::FDX,
            [0x24, 0x46, 0x4c, 0x30] => Self::FL0,
            [0x24, 0x46, 0x4c, 0x31] => Self::FL1,
            [0x24, 0x46, 0x4c, 0x32] => Self::FL2,
            [0x24, 0x46, 0x4c, 0x33] => Self::FL3,
            [0x24, 0x46, 0x52, 0x30] => Self::FR0,
            [0x24, 0x46, 0x52, 0x31] => Self::FR1,
            [0x24, 0x46, 0x52, 0x32] => Self::FR2,
            [0x24, 0x46, 0x52, 0x33] => Self::FR3,
            [0x24, 0x46, 0x53, 0x44] => Self::FSD,
            [0x24, 0x47, 0x43, 0x30] => Self::GC0,
            [0x24, 0x47, 0x43, 0x31] => Self::GC1,
            [0x24, 0x47, 0x43, 0x32] => Self::GC2,
            [0x24, 0x47, 0x43, 0x33] => Self::GC3,
            [0x24, 0x47, 0x4f, 0x30] => Self::GO0,
            [0x24, 0x47, 0x4f, 0x31] => Self::GO1,
            [0x24, 0x47, 0x4f, 0x32] => Self::GO2,
            [0x24, 0x47, 0x4f, 0x33] => Self::GO3,
            [0x24, 0x47, 0x4f, 0x34] => Self::GO4,
            [0x24, 0x47, 0x4f, 0x35] => Self::GO5,
            [0x24, 0x48, 0x49, 0x54] => Self::HIT,
            [0x24, 0x4b, 0x56, 0x53] => Self::KVS,
            [0x24, 0x52, 0x4c, 0x30] => Self::RL0,
            [0x24, 0x52, 0x4c, 0x31] => Self::RL1,
            [0x24, 0x52, 0x4c, 0x32] => Self::RL2,
            [0x24, 0x52, 0x4c, 0x33] => Self::RL3,
            [0x24, 0x52, 0x52, 0x30] => Self::RR0,
            [0x24, 0x52, 0x52, 0x31] => Self::RR1,
            [0x24, 0x52, 0x52, 0x32] => Self::RR2,
            [0x24, 0x52, 0x52, 0x33] => Self::RR3,
            [0x24, 0x53, 0x43, 0x44] => Self::SCD,
            [0x24, 0x53, 0x48, 0x4b] => Self::SHK,
            [0x24, 0x53, 0x48, 0x4c] => Self::SHL,
            [0x24, 0x53, 0x48, 0x52] => Self::SHR,
            [0x24, 0x53, 0x4c, 0x30] => Self::SL0,
            [0x24, 0x53, 0x4c, 0x31] => Self::SL1,
            [0x24, 0x53, 0x4c, 0x32] => Self::SL2,
            [0x24, 0x53, 0x4c, 0x33] => Self::SL3,
            [0x24, 0x53, 0x4d, 0x44] => Self::SMD,
            [0x24, 0x53, 0x4d, 0x47] => Self::SMG,
            [0x24, 0x53, 0x4e, 0x44] => Self::SND,
            [0x24, 0x53, 0x52, 0x30] => Self::SR0,
            [0x24, 0x53, 0x52, 0x31] => Self::SR1,
            [0x24, 0x53, 0x52, 0x32] => Self::SR2,
            [0x24, 0x53, 0x52, 0x33] => Self::SR3,
            [0x24, 0x53, 0x54, 0x78] => Self::STx,
            [0x24, 0x54, 0x52, 0x44] => Self::TRD,
            [0x24, 0x56, 0x47, 0x30] => Self::VG0,
            [0x24, 0x56, 0x47, 0x31] => Self::VG1,
            [0x24, 0x56, 0x47, 0x32] => Self::VG2,
            [0x24, 0x56, 0x47, 0x33] => Self::VG3,
            [0x24, 0x56, 0x47, 0x34] => Self::VG4,
            [0x24, 0x56, 0x47, 0x35] => Self::VG5,
            [0x24, 0x56, 0x47, 0x36] => Self::VG6,
            [0x24, 0x56, 0x47, 0x37] => Self::VG7,
            [0x24, 0x56, 0x47, 0x38] => Self::VG8,
            [0x24, 0x56, 0x54, 0x30] => Self::VT0,
            [0x24, 0x56, 0x54, 0x31] => Self::VT1,
            [0x24, 0x56, 0x54, 0x32] => Self::VT2,
            [0x24, 0x56, 0x54, 0x33] => Self::VT3,
            [0x24, 0x56, 0x54, 0x34] => Self::VT4,
            [0x24, 0x56, 0x54, 0x35] => Self::VT5,
            [0x24, 0x56, 0x54, 0x36] => Self::VT6,
            [0x24, 0x56, 0x54, 0x37] => Self::VT7,
            [0x24, 0x56, 0x54, 0x38] => Self::VT8,
            [0x24, 0x57, 0x47, 0x47] => Self::WGG,
            [0x24, 0x57, 0x4c, 0x30] => Self::WL0,
            [0x24, 0x57, 0x4c, 0x31] => Self::WL1,
            [0x24, 0x57, 0x4c, 0x32] => Self::WL2,
            [0x24, 0x57, 0x4c, 0x33] => Self::WL3,
            [0x24, 0x57, 0x4c, 0x42] => Self::WLB,
            [0x24, 0x57, 0x4c, 0x54] => Self::WLT,
            [0x24, 0x57, 0x4e, 0x47] => Self::WNG,
            [0x24, 0x57, 0x52, 0x30] => Self::WR0,
            [0x24, 0x57, 0x52, 0x31] => Self::WR1,
            [0x24, 0x57, 0x52, 0x32] => Self::WR2,
            [0x24, 0x57, 0x52, 0x33] => Self::WR3,
            [0x24, 0x57, 0x52, 0x42] => Self::WRB,
            [0x24, 0x57, 0x52, 0x54] => Self::WRT,
            [0x24, 0x57, 0x54, 0x42] => Self::WTB,
            [0x24, 0x57, 0x54, 0x54] => Self::WTT,
            [0x24, 0x57, 0x57, 0x47] => Self::WWG,
            [0x44, 0x45, 0x53, 0x54] => Self::DEST,
            [0x50, 0x4f, 0x49, 0x4e] => Self::POIN,
            [0x57, 0x48, 0x45, 0x45] => Self::WHEE,
            [0x42, 0x4f, 0x54, 0x54] => Self::BOTT,
            [0x54, 0x4f, 0x50, 0x30] => Self::TOP0,
            _ => Self::Unknown(value),
        }
    }
}

impl From<M2EventIdentifier> for [u8; 4] {
    fn from(value: M2EventIdentifier) -> Self {
        match value {
            M2EventIdentifier::AH0 => [0x24, 0x41, 0x48, 0x30],
            M2EventIdentifier::AH1 => [0x24, 0x41, 0x48, 0x31],
            M2EventIdentifier::AH2 => [0x24, 0x41, 0x48, 0x32],
            M2EventIdentifier::AH3 => [0x24, 0x41, 0x48, 0x33],
            M2EventIdentifier::BMD => [0x24, 0x42, 0x4d, 0x44],
            M2EventIdentifier::AIM => [0x24, 0x41, 0x49, 0x4d],
            M2EventIdentifier::ALT => [0x24, 0x41, 0x4c, 0x54],
            M2EventIdentifier::BL0 => [0x24, 0x42, 0x4c, 0x30],
            M2EventIdentifier::BL1 => [0x24, 0x42, 0x4c, 0x31],
            M2EventIdentifier::BL2 => [0x24, 0x42, 0x4c, 0x32],
            M2EventIdentifier::BL3 => [0x24, 0x42, 0x4c, 0x33],
            M2EventIdentifier::BR0 => [0x24, 0x42, 0x52, 0x30],
            M2EventIdentifier::BR1 => [0x24, 0x42, 0x52, 0x31],
            M2EventIdentifier::BR2 => [0x24, 0x42, 0x52, 0x32],
            M2EventIdentifier::BR3 => [0x24, 0x42, 0x52, 0x33],
            M2EventIdentifier::BRT => [0x24, 0x42, 0x52, 0x54],
            M2EventIdentifier::BTH => [0x24, 0x42, 0x54, 0x48],
            M2EventIdentifier::BWP => [0x24, 0x42, 0x57, 0x50],
            M2EventIdentifier::BWR => [0x24, 0x42, 0x57, 0x52],
            M2EventIdentifier::CAH => [0x24, 0x43, 0x41, 0x48],
            M2EventIdentifier::CCH => [0x24, 0x43, 0x43, 0x48],
            M2EventIdentifier::CFM => [0x24, 0x43, 0x46, 0x4d],
            M2EventIdentifier::CHD => [0x24, 0x43, 0x48, 0x44],
            M2EventIdentifier::CMA => [0x24, 0x43, 0x4d, 0x41],
            M2EventIdentifier::CPP => [0x24, 0x43, 0x50, 0x50],
            M2EventIdentifier::CSD => [0x24, 0x43, 0x53, 0x44],
            M2EventIdentifier::CSL => [0x24, 0x43, 0x53, 0x4c],
            M2EventIdentifier::CSR => [0x24, 0x43, 0x53, 0x52],
            M2EventIdentifier::CSS => [0x24, 0x43, 0x53, 0x53],
            M2EventIdentifier::CST => [0x24, 0x43, 0x53, 0x54],
            M2EventIdentifier::CVS => [0x24, 0x43, 0x56, 0x53],
            M2EventIdentifier::DSE => [0x24, 0x44, 0x53, 0x45],
            M2EventIdentifier::DSL => [0x24, 0x44, 0x53, 0x4c],
            M2EventIdentifier::DSO => [0x24, 0x44, 0x53, 0x4f],
            M2EventIdentifier::DTH => [0x24, 0x44, 0x54, 0x48],
            M2EventIdentifier::EAC => [0x24, 0x45, 0x41, 0x43],
            M2EventIdentifier::EDC => [0x24, 0x45, 0x44, 0x43],
            M2EventIdentifier::EMV => [0x24, 0x45, 0x4d, 0x56],
            M2EventIdentifier::ESD => [0x24, 0x45, 0x53, 0x44],
            M2EventIdentifier::EWT => [0x24, 0x45, 0x57, 0x54],
            M2EventIdentifier::FD0 => [0x24, 0x46, 0x44, 0x30],
            M2EventIdentifier::FD1 => [0x24, 0x46, 0x44, 0x31],
            M2EventIdentifier::FD2 => [0x24, 0x46, 0x44, 0x32],
            M2EventIdentifier::FD3 => [0x24, 0x46, 0x44, 0x33],
            M2EventIdentifier::FD4 => [0x24, 0x46, 0x44, 0x34],
            M2EventIdentifier::FD5 => [0x24, 0x46, 0x44, 0x35],
            M2EventIdentifier::FD6 => [0x24, 0x46, 0x44, 0x36],
            M2EventIdentifier::FD7 => [0x24, 0x46, 0x44, 0x37],
            M2EventIdentifier::FD8 => [0x24, 0x46, 0x44, 0x38],
            M2EventIdentifier::FD9 => [0x24, 0x46, 0x44, 0x39],
            M2EventIdentifier::FDX => [0x24, 0x46, 0x44, 0x58],
            M2EventIdentifier::FL0 => [0x24, 0x46, 0x4c, 0x30],
            M2EventIdentifier::FL1 => [0x24, 0x46, 0x4c, 0x31],
            M2EventIdentifier::FL2 => [0x24, 0x46, 0x4c, 0x32],
            M2EventIdentifier::FL3 => [0x24, 0x46, 0x4c, 0x33],
            M2EventIdentifier::FR0 => [0x24, 0x46, 0x52, 0x30],
            M2EventIdentifier::FR1 => [0x24, 0x46, 0x52, 0x31],
            M2EventIdentifier::FR2 => [0x24, 0x46, 0x52, 0x32],
            M2EventIdentifier::FR3 => [0x24, 0x46, 0x52, 0x33],
            M2EventIdentifier::FSD => [0x24, 0x46, 0x53, 0x44],
            M2EventIdentifier::GC0 => [0x24, 0x47, 0x43, 0x30],
            M2EventIdentifier::GC1 => [0x24, 0x47, 0x43, 0x31],
            M2EventIdentifier::GC2 => [0x24, 0x47, 0x43, 0x32],
            M2EventIdentifier::GC3 => [0x24, 0x47, 0x43, 0x33],
            M2EventIdentifier::GO0 => [0x24, 0x47, 0x4f, 0x30],
            M2EventIdentifier::GO1 => [0x24, 0x47, 0x4f, 0x31],
            M2EventIdentifier::GO2 => [0x24, 0x47, 0x4f, 0x32],
            M2EventIdentifier::GO3 => [0x24, 0x47, 0x4f, 0x33],
            M2EventIdentifier::GO4 => [0x24, 0x47, 0x4f, 0x34],
            M2EventIdentifier::GO5 => [0x24, 0x47, 0x4f, 0x35],
            M2EventIdentifier::HIT => [0x24, 0x48, 0x49, 0x54],
            M2EventIdentifier::KVS => [0x24, 0x4b, 0x56, 0x53],
            M2EventIdentifier::RL0 => [0x24, 0x52, 0x4c, 0x30],
            M2EventIdentifier::RL1 => [0x24, 0x52, 0x4c, 0x31],
            M2EventIdentifier::RL2 => [0x24, 0x52, 0x4c, 0x32],
            M2EventIdentifier::RL3 => [0x24, 0x52, 0x4c, 0x33],
            M2EventIdentifier::RR0 => [0x24, 0x52, 0x52, 0x30],
            M2EventIdentifier::RR1 => [0x24, 0x52, 0x52, 0x31],
            M2EventIdentifier::RR2 => [0x24, 0x52, 0x52, 0x32],
            M2EventIdentifier::RR3 => [0x24, 0x52, 0x52, 0x33],
            M2EventIdentifier::SCD => [0x24, 0x53, 0x43, 0x44],
            M2EventIdentifier::SHK => [0x24, 0x53, 0x48, 0x4b],
            M2EventIdentifier::SHL => [0x24, 0x53, 0x48, 0x4c],
            M2EventIdentifier::SHR => [0x24, 0x53, 0x48, 0x52],
            M2EventIdentifier::SL0 => [0x24, 0x53, 0x4c, 0x30],
            M2EventIdentifier::SL1 => [0x24, 0x53, 0x4c, 0x31],
            M2EventIdentifier::SL2 => [0x24, 0x53, 0x4c, 0x32],
            M2EventIdentifier::SL3 => [0x24, 0x53, 0x4c, 0x33],
            M2EventIdentifier::SMD => [0x24, 0x53, 0x4d, 0x44],
            M2EventIdentifier::SMG => [0x24, 0x53, 0x4d, 0x47],
            M2EventIdentifier::SND => [0x24, 0x53, 0x4e, 0x44],
            M2EventIdentifier::SR0 => [0x24, 0x53, 0x52, 0x30],
            M2EventIdentifier::SR1 => [0x24, 0x53, 0x52, 0x31],
            M2EventIdentifier::SR2 => [0x24, 0x53, 0x52, 0x32],
            M2EventIdentifier::SR3 => [0x24, 0x53, 0x52, 0x33],
            M2EventIdentifier::STx => [0x24, 0x53, 0x54, 0x78],
            M2EventIdentifier::TRD => [0x24, 0x54, 0x52, 0x44],
            M2EventIdentifier::VG0 => [0x24, 0x56, 0x47, 0x30],
            M2EventIdentifier::VG1 => [0x24, 0x56, 0x47, 0x31],
            M2EventIdentifier::VG2 => [0x24, 0x56, 0x47, 0x32],
            M2EventIdentifier::VG3 => [0x24, 0x56, 0x47, 0x33],
            M2EventIdentifier::VG4 => [0x24, 0x56, 0x47, 0x34],
            M2EventIdentifier::VG5 => [0x24, 0x56, 0x47, 0x35],
            M2EventIdentifier::VG6 => [0x24, 0x56, 0x47, 0x36],
            M2EventIdentifier::VG7 => [0x24, 0x56, 0x47, 0x37],
            M2EventIdentifier::VG8 => [0x24, 0x56, 0x47, 0x38],
            M2EventIdentifier::VT0 => [0x24, 0x56, 0x54, 0x30],
            M2EventIdentifier::VT1 => [0x24, 0x56, 0x54, 0x31],
            M2EventIdentifier::VT2 => [0x24, 0x56, 0x54, 0x32],
            M2EventIdentifier::VT3 => [0x24, 0x56, 0x54, 0x33],
            M2EventIdentifier::VT4 => [0x24, 0x56, 0x54, 0x34],
            M2EventIdentifier::VT5 => [0x24, 0x56, 0x54, 0x35],
            M2EventIdentifier::VT6 => [0x24, 0x56, 0x54, 0x36],
            M2EventIdentifier::VT7 => [0x24, 0x56, 0x54, 0x37],
            M2EventIdentifier::VT8 => [0x24, 0x56, 0x54, 0x38],
            M2EventIdentifier::WGG => [0x24, 0x57, 0x47, 0x47],
            M2EventIdentifier::WL0 => [0x24, 0x57, 0x4c, 0x30],
            M2EventIdentifier::WL1 => [0x24, 0x57, 0x4c, 0x31],
            M2EventIdentifier::WL2 => [0x24, 0x57, 0x4c, 0x32],
            M2EventIdentifier::WL3 => [0x24, 0x57, 0x4c, 0x33],
            M2EventIdentifier::WLB => [0x24, 0x57, 0x4c, 0x42],
            M2EventIdentifier::WLT => [0x24, 0x57, 0x4c, 0x54],
            M2EventIdentifier::WNG => [0x24, 0x57, 0x4e, 0x47],
            M2EventIdentifier::WR0 => [0x24, 0x57, 0x52, 0x30],
            M2EventIdentifier::WR1 => [0x24, 0x57, 0x52, 0x31],
            M2EventIdentifier::WR2 => [0x24, 0x57, 0x52, 0x32],
            M2EventIdentifier::WR3 => [0x24, 0x57, 0x52, 0x33],
            M2EventIdentifier::WRB => [0x24, 0x57, 0x52, 0x42],
            M2EventIdentifier::WRT => [0x24, 0x57, 0x52, 0x54],
            M2EventIdentifier::WTB => [0x24, 0x57, 0x54, 0x42],
            M2EventIdentifier::WTT => [0x24, 0x57, 0x54, 0x54],
            M2EventIdentifier::WWG => [0x24, 0x57, 0x57, 0x47],
            M2EventIdentifier::DEST => [0x44, 0x45, 0x53, 0x54],
            M2EventIdentifier::POIN => [0x50, 0x4f, 0x49, 0x4e],
            M2EventIdentifier::WHEE => [0x57, 0x48, 0x45, 0x45],
            M2EventIdentifier::BOTT => [0x42, 0x4f, 0x54, 0x54],
            M2EventIdentifier::TOP0 => [0x54, 0x4f, 0x50, 0x30],
            M2EventIdentifier::Unknown(val) => val,
        }
    }
}

impl WowHeaderR for M2EventIdentifier {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        let val: [u8; 4] = reader.wow_read()?;
        Ok(val.into())
    }
}

impl WowHeaderW for M2EventIdentifier {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        let val: [u8; 4] = (*self).into();
        writer.wow_write(&val)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        4
    }
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub struct M2EventHeader {
    pub identifier: M2EventIdentifier,
    pub data: u32,
    pub bone_index: u32,
    pub position: C3Vector,
    #[wow_data(versioned)]
    pub enabled: M2AnimationBaseTrackHeader,
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
