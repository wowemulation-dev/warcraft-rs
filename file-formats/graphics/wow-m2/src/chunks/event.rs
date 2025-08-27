use wow_data::prelude::*;
use wow_data::types::C3Vector;
use wow_data::types::MagicStr;
use wow_data_derive::{WowDataR, WowEnumFrom, WowHeaderR, WowHeaderW};

use crate::version::MD20Version;

use super::animation::{M2AnimationBaseTrackData, M2AnimationBaseTrackHeader};

#[derive(Debug, Clone, Copy, PartialEq, Eq, WowEnumFrom, WowHeaderR, WowHeaderW)]
#[wow_data(from_type=MagicStr)]
pub enum M2EventIdentifier {
    /// PlaySoundKit (customAttack[x]) | soundEffect ID is defined by CreatureSoundDataRec::m_customAttack[x]
    #[wow_data(expr=[0x24,0x41,0x48,0x30])]
    AH0,
    /// PlaySoundKit (customAttack[x]) | soundEffect ID is defined by CreatureSoundDataRec::m_customAttack[x]
    #[wow_data(expr=[0x24,0x41,0x48,0x31])]
    AH1,
    /// PlaySoundKit (customAttack[x]) | soundEffect ID is defined by CreatureSoundDataRec::m_customAttack[x]
    #[wow_data(expr=[0x24,0x41,0x48,0x32])]
    AH2,
    /// PlaySoundKit (customAttack[x]) | soundEffect ID is defined by CreatureSoundDataRec::m_customAttack[x]
    #[wow_data(expr=[0x24,0x41,0x48,0x33])]
    AH3,
    /// BowMissleDestination | RangedWeapon | Bow Middle
    #[wow_data(expr=[0x24,0x42,0x4d,0x44])]
    BMD,
    /// Vehicles | CGUnit_C::ComputeMissileTrajectory | Position used as MissileFirePos.
    #[wow_data(expr=[0x24,0x41,0x49,0x4d])]
    AIM,
    /// anim_swap_event / DisplayTransition | Unit | CUnitDisplayTransition_C::UpdateState(1) or CUnitDisplayTransition_C::HandleAnimSwapEvent
    #[wow_data(expr=[0x24,0x41,0x4c,0x54])]
    ALT,
    /// FootstepAnimEventHit (left) | Unit | Backwards
    #[wow_data(expr=[0x24,0x42,0x4c,0x30])]
    BL0,
    /// FootstepAnimEventHit (left) | Unit | Backwards
    #[wow_data(expr=[0x24,0x42,0x4c,0x31])]
    BL1,
    /// FootstepAnimEventHit (left) | Unit | Backwards
    #[wow_data(expr=[0x24,0x42,0x4c,0x32])]
    BL2,
    /// FootstepAnimEventHit (left) | Unit | Backwards
    #[wow_data(expr=[0x24,0x42,0x4c,0x33])]
    BL3,
    /// FootstepAnimEventHit (right) | Unit | Backwards
    #[wow_data(expr=[0x24,0x42,0x52,0x30])]
    BR0,
    /// FootstepAnimEventHit (right) | Unit | Backwards
    #[wow_data(expr=[0x24,0x42,0x52,0x31])]
    BR1,
    /// FootstepAnimEventHit (right) | Unit | Backwards
    #[wow_data(expr=[0x24,0x42,0x52,0x32])]
    BR2,
    /// FootstepAnimEventHit (right) | Unit | Backwards
    #[wow_data(expr=[0x24,0x42,0x52,0x33])]
    BR3,
    /// PlaySoundKit (birth) | soundEffect ID is defined by CreatureSoundDatarec::m_birthSoundID
    #[wow_data(expr=[0x24,0x42,0x52,0x54])]
    BRT,
    /// Breath | Unit | All situations, where nothing happens or breathing. | Adds Special Unit Effect based on unit state (under water, in-snow, …)
    #[wow_data(expr=[0x24,0x42,0x54,0x48])]
    BTH,
    /// PlayRangedItemPull (Bow Pull) | Unit | LoadRifle, LoadBow
    #[wow_data(expr=[0x24,0x42,0x57,0x50])]
    BWP,
    /// BowRelease | Unit | AttackRifle, AttackBow, AttackThrown
    #[wow_data(expr=[0x24,0x42,0x57,0x52])]
    BWR,
    /// Unit | Attack*, *Unarmed, ShieldBash, Special* | attack hold? CGUnit_C::HandleCombatAnimEvent
    #[wow_data(expr=[0x24,0x43,0x41,0x48])]
    CAH,
    /// Unit | mostly not fired, AttackThrown | Fishing Pole CEffect::DrawFishingString needs this on the model for getting the string attachments.
    #[wow_data(expr=[0x24,0x43,0x43,0x48])]
    CCH,
    /// Unit | CGCamera::UpdateMountHeightOrOffset | CGCamera::UpdateMountHeightOrOffset: Only z is used. Non-animated. Not used if $CMA
    #[wow_data(expr=[0x24,0x43,0x46,0x4d])]
    CFM,
    /// Unit | not fired | probably does not exist?!
    #[wow_data(expr=[0x24,0x43,0x48,0x44])]
    CHD,
    /// Unit | CGCamera::UpdateMountHeightOrOffset: Position for camera
    #[wow_data(expr=[0x24,0x43,0x4d,0x41])]
    CMA,
    /// PlayCombatActionAnimKit | parry, anims, depending on some state, also some callback which might do more
    #[wow_data(expr=[0x24,0x43,0x50,0x50])]
    CPP,
    /// soundEntryId | PlayEmoteSound | Unit | Emote*
    #[wow_data(expr=[0x24,0x43,0x53,0x44])]
    CSD,
    /// release_missiles_on_next_update if has_pending_missiles (left) | Unit | AttackRifle, SpellCast*, ChannelCast* | "x is {L or R} (""Left/right hand"") (?)"
    #[wow_data(expr=[0x24,0x43,0x53,0x4c])]
    CSL,
    /// release_missiles_on_next_update if has_pending_missiles (right) | Unit | AttackBow, AttackRifle, AttackThrown, SpellCast*, ChannelCast* | "x is {L or R} (""Left/right hand"") (?)"
    #[wow_data(expr=[0x24,0x43,0x53,0x52])]
    CSR,
    /// PlayWeaponSwooshSound | sound played depends on CGUnit_C::GetWeaponSwingType
    #[wow_data(expr=[0x24,0x43,0x53,0x53])]
    CSS,
    /// release_missiles_on_next_update if has_pending_missiles | Unit | Attack*, *Unarmed, ShieldBash, Special*, SpellCast, Parry*, EmoteEat, EmoteRoar, Kick, ... | $CSL/R/T are also used in CGUnit_C::ComputeDefaultMissileFirePos.
    #[wow_data(expr=[0x24,0x43,0x53,0x54])]
    CST,
    ///  ? | Data: SoundEntriesAdvanced.dbc, Sound — Not present in 6.0.1.18179
    #[wow_data(expr=[0x24,0x43,0x56,0x53])]
    CVS,
    /// DestroyEmitter | MapObj
    #[wow_data(expr=[0x24,0x44,0x53,0x45])]
    DSE,
    /// soundEntryId | DoodadSoundLoop (low priority) | GO
    #[wow_data(expr=[0x24,0x44,0x53,0x4c])]
    DSL,
    /// soundEntryId | DoodadSoundOneShot | GO
    #[wow_data(expr=[0x24,0x44,0x53,0x4f])]
    DSO,
    /// DeathThud + LootEffect | Unit | Death, Drown, Knockdown | """I'm dead now!"", UnitCombat_C, this plays death sounds and more." Note that this is NOT triggering CreatureSoundDataRec::m_soundDeathID, but that is just always triggered as soon as the death animation plays.
    #[wow_data(expr=[0x24,0x44,0x54,0x48])]
    DTH,
    /// object package state enter 3, exit 2, 4, 5
    #[wow_data(expr=[0x24,0x45,0x41,0x43])]
    EAC,
    /// object package state enter 5, exit 3, 4, 2
    #[wow_data(expr=[0x24,0x45,0x44,0x43])]
    EDC,
    /// object package state enter 4, exit 3, 2, 5
    #[wow_data(expr=[0x24,0x45,0x4d,0x56])]
    EMV,
    /// PlayEmoteStateSound | Unit | soundEffect ID is implicit by currently played emote
    #[wow_data(expr=[0x24,0x45,0x53,0x44])]
    ESD,
    /// object package state enter 2, exit 3, 4, 5
    #[wow_data(expr=[0x24,0x45,0x57,0x54])]
    EWT,
    /// PlayFidgetSound | CreatureSoundDataRec::m_soundFidget (only has 5 entries, so don’t use 6-9)
    #[wow_data(expr=[0x24,0x46,0x44,0x30])]
    FD0,
    /// PlayFidgetSound | CreatureSoundDataRec::m_soundFidget (only has 5 entries, so don’t use 6-9)
    #[wow_data(expr=[0x24,0x46,0x44,0x31])]
    FD1,
    /// PlayFidgetSound | CreatureSoundDataRec::m_soundFidget (only has 5 entries, so don’t use 6-9)
    #[wow_data(expr=[0x24,0x46,0x44,0x32])]
    FD2,
    /// PlayFidgetSound | CreatureSoundDataRec::m_soundFidget (only has 5 entries, so don’t use 6-9)
    #[wow_data(expr=[0x24,0x46,0x44,0x33])]
    FD3,
    /// PlayFidgetSound | CreatureSoundDataRec::m_soundFidget (only has 5 entries, so don’t use 6-9)
    #[wow_data(expr=[0x24,0x46,0x44,0x34])]
    FD4,
    /// PlayFidgetSound | CreatureSoundDataRec::m_soundFidget (only has 5 entries, so don’t use 6-9)
    #[wow_data(expr=[0x24,0x46,0x44,0x35])]
    FD5,
    /// PlayFidgetSound | CreatureSoundDataRec::m_soundFidget (only has 5 entries, so don’t use 6-9)
    #[wow_data(expr=[0x24,0x46,0x44,0x36])]
    FD6,
    /// PlayFidgetSound | CreatureSoundDataRec::m_soundFidget (only has 5 entries, so don’t use 6-9)
    #[wow_data(expr=[0x24,0x46,0x44,0x37])]
    FD7,
    /// PlayFidgetSound | CreatureSoundDataRec::m_soundFidget (only has 5 entries, so don’t use 6-9)
    #[wow_data(expr=[0x24,0x46,0x44,0x38])]
    FD8,
    /// PlayFidgetSound | CreatureSoundDataRec::m_soundFidget (only has 5 entries, so don’t use 6-9)
    #[wow_data(expr=[0x24,0x46,0x44,0x39])]
    FD9,
    /// PlayUnitSound (stand) | soundEffect ID is defined by CreatureSoundDataRec::m_soundStandID
    #[wow_data(expr=[0x24,0x46,0x44,0x58])]
    FDX,
    /// FootstepAnimEventHit (left) | Foot Left Forward
    #[wow_data(expr=[0x24,0x46,0x4c,0x30])]
    FL0,
    /// FootstepAnimEventHit (left) | Foot Left Forward
    #[wow_data(expr=[0x24,0x46,0x4c,0x31])]
    FL1,
    /// FootstepAnimEventHit (left) | Foot Left Forward
    #[wow_data(expr=[0x24,0x46,0x4c,0x32])]
    FL2,
    /// FootstepAnimEventHit (left) | Foot Left Forward
    #[wow_data(expr=[0x24,0x46,0x4c,0x33])]
    FL3,
    /// FootstepAnimEventHit (right) | Foot Right Forward
    #[wow_data(expr=[0x24,0x46,0x52,0x30])]
    FR0,
    /// FootstepAnimEventHit (right) | Foot Right Forward
    #[wow_data(expr=[0x24,0x46,0x52,0x31])]
    FR1,
    /// FootstepAnimEventHit (right) | Foot Right Forward
    #[wow_data(expr=[0x24,0x46,0x52,0x32])]
    FR2,
    /// FootstepAnimEventHit (right) | Foot Right Forward
    #[wow_data(expr=[0x24,0x46,0x52,0x33])]
    FR3,
    /// HandleFootfallAnimEvent | Unit | Walk, Run (multiple times), ... | Plays some sound. Footstep? Also seen at several emotes etc. where feet are moved. CGUnit_C::HandleFootfallAnimEvent
    #[wow_data(expr=[0x24,0x46,0x53,0x44])]
    FSD,
    /// GameObject_C_PlayAnimatedSound | soundEffect ID is defined by GameObjectDisplayInfoRec::m_Sound[x + 6] ({Custom0, Custom1, Custom2, Custom3})
    #[wow_data(expr=[0x24,0x47,0x43,0x30])]
    GC0,
    /// GameObject_C_PlayAnimatedSound | soundEffect ID is defined by GameObjectDisplayInfoRec::m_Sound[x + 6] ({Custom0, Custom1, Custom2, Custom3})
    #[wow_data(expr=[0x24,0x47,0x43,0x31])]
    GC1,
    /// GameObject_C_PlayAnimatedSound | soundEffect ID is defined by GameObjectDisplayInfoRec::m_Sound[x + 6] ({Custom0, Custom1, Custom2, Custom3})
    #[wow_data(expr=[0x24,0x47,0x43,0x32])]
    GC2,
    /// GameObject_C_PlayAnimatedSound | soundEffect ID is defined by GameObjectDisplayInfoRec::m_Sound[x + 6] ({Custom0, Custom1, Custom2, Custom3})
    #[wow_data(expr=[0x24,0x47,0x43,0x33])]
    GC3,
    /// GameObject_C_PlayAnimatedSound | soundEffect ID is defined by GameObjectDisplayInfoRec::m_Sound[x] ({Stand, Open, Loop, Close, Destroy, Opened})
    #[wow_data(expr=[0x24,0x47,0x4f,0x30])]
    GO0,
    /// GameObject_C_PlayAnimatedSound | soundEffect ID is defined by GameObjectDisplayInfoRec::m_Sound[x] ({Stand, Open, Loop, Close, Destroy, Opened})
    #[wow_data(expr=[0x24,0x47,0x4f,0x31])]
    GO1,
    /// GameObject_C_PlayAnimatedSound | soundEffect ID is defined by GameObjectDisplayInfoRec::m_Sound[x] ({Stand, Open, Loop, Close, Destroy, Opened})
    #[wow_data(expr=[0x24,0x47,0x4f,0x32])]
    GO2,
    /// GameObject_C_PlayAnimatedSound | soundEffect ID is defined by GameObjectDisplayInfoRec::m_Sound[x] ({Stand, Open, Loop, Close, Destroy, Opened})
    #[wow_data(expr=[0x24,0x47,0x4f,0x33])]
    GO3,
    /// GameObject_C_PlayAnimatedSound | soundEffect ID is defined by GameObjectDisplayInfoRec::m_Sound[x] ({Stand, Open, Loop, Close, Destroy, Opened})
    #[wow_data(expr=[0x24,0x47,0x4f,0x34])]
    GO4,
    /// GameObject_C_PlayAnimatedSound | soundEffect ID is defined by GameObjectDisplayInfoRec::m_Sound[x] ({Stand, Open, Loop, Close, Destroy, Opened})
    #[wow_data(expr=[0x24,0x47,0x4f,0x35])]
    GO5,
    /// PlayWoundAnimKit | Unit | Attack*, *Unarmed, ShieldBash, Special* | soundEntryId depends on SpellVisualKit
    #[wow_data(expr=[0x24,0x48,0x49,0x54])]
    HIT,
    ///  ? | MapLoad.cpp -- not found in 6.0.1.18179
    #[wow_data(expr=[0x24,0x4b,0x56,0x53])]
    KVS,
    /// FootstepAnimEventHit (left) | Running
    #[wow_data(expr=[0x24,0x52,0x4c,0x30])]
    RL0,
    /// FootstepAnimEventHit (left) | Running
    #[wow_data(expr=[0x24,0x52,0x4c,0x31])]
    RL1,
    /// FootstepAnimEventHit (left) | Running
    #[wow_data(expr=[0x24,0x52,0x4c,0x32])]
    RL2,
    /// FootstepAnimEventHit (left) | Running
    #[wow_data(expr=[0x24,0x52,0x4c,0x33])]
    RL3,
    /// FootstepAnimEventHit (right) | Running
    #[wow_data(expr=[0x24,0x52,0x52,0x30])]
    RR0,
    /// FootstepAnimEventHit (right) | Running
    #[wow_data(expr=[0x24,0x52,0x52,0x31])]
    RR1,
    /// FootstepAnimEventHit (right) | Running
    #[wow_data(expr=[0x24,0x52,0x52,0x32])]
    RR2,
    /// FootstepAnimEventHit (right) | Running
    #[wow_data(expr=[0x24,0x52,0x52,0x33])]
    RR3,
    /// PlaySoundKit (spellCastDirectedSound) | soundEffect ID is defined by CreatureSoundDataRec::m_spellCastDirectedSoundID
    #[wow_data(expr=[0x24,0x53,0x43,0x44])]
    SCD,
    /// spellEffectCameraShakesID | AddShake | GO
    #[wow_data(expr=[0x24,0x53,0x48,0x4b])]
    SHK,
    /// ExchangeSheathedWeapon (left) | Sheath, HipSheath
    #[wow_data(expr=[0x24,0x53,0x48,0x4c])]
    SHL,
    /// ExchangeSheathedWeapon (right) | Sheath, HipSheath
    #[wow_data(expr=[0x24,0x53,0x48,0x52])]
    SHR,
    /// FootstepAnimEventHit (left) | Stop, (JumpEnd), (Shuffle*) | Stop
    #[wow_data(expr=[0x24,0x53,0x4c,0x30])]
    SL0,
    /// FootstepAnimEventHit (left) | Stop, (JumpEnd), (Shuffle*) | Stop
    #[wow_data(expr=[0x24,0x53,0x4c,0x31])]
    SL1,
    /// FootstepAnimEventHit (left) | Stop, (JumpEnd), (Shuffle*) | Stop
    #[wow_data(expr=[0x24,0x53,0x4c,0x32])]
    SL2,
    /// FootstepAnimEventHit (left) | Stop, (JumpEnd), (Shuffle*) | Stop
    #[wow_data(expr=[0x24,0x53,0x4c,0x33])]
    SL3,
    /// PlaySoundKit (submerged) | soundEffect ID is defined by CreatureSoundDatarec::m_submergedSoundID
    #[wow_data(expr=[0x24,0x53,0x4d,0x44])]
    SMD,
    /// PlaySoundKit (submerge) | soundEffect ID is defined by CreatureSoundDatarec::m_submergeSoundID
    #[wow_data(expr=[0x24,0x53,0x4d,0x47])]
    SMG,
    /// soundEntryId | PlaySoundKit (custom) | GO
    #[wow_data(expr=[0x24,0x53,0x4e,0x44])]
    SND,
    /// FootstepAnimEventHit (right) | Stop, (JumpEnd), (Shuffle*) | Stop
    #[wow_data(expr=[0x24,0x53,0x52,0x30])]
    SR0,
    /// FootstepAnimEventHit (right) | Stop, (JumpEnd), (Shuffle*) | Stop
    #[wow_data(expr=[0x24,0x53,0x52,0x31])]
    SR1,
    /// FootstepAnimEventHit (right) | Stop, (JumpEnd), (Shuffle*) | Stop
    #[wow_data(expr=[0x24,0x53,0x52,0x32])]
    SR2,
    /// FootstepAnimEventHit (right) | Stop, (JumpEnd), (Shuffle*) | Stop
    #[wow_data(expr=[0x24,0x53,0x52,0x33])]
    SR3,
    /// Mounts | MountTransitionObject::UpdateCharacterData | Not seen in 6.0.1.18179 -- x is {E and B} , sequence time is taken of both, pivot of $STB. (Also, attachment info for attachment 0)
    #[wow_data(expr=[0x24,0x53,0x54,0x78])]
    STx,
    /// HandleSpellEventSound | Unit | EmoteWork*, UseStanding* | soundEffect ID is implicit by SpellRec
    #[wow_data(expr=[0x24,0x54,0x52,0x44])]
    TRD,
    /// HandleBoneAnimGrabEvent
    #[wow_data(expr=[0x24,0x56,0x47,0x30])]
    VG0,
    /// HandleBoneAnimGrabEvent
    #[wow_data(expr=[0x24,0x56,0x47,0x31])]
    VG1,
    /// HandleBoneAnimGrabEvent
    #[wow_data(expr=[0x24,0x56,0x47,0x32])]
    VG2,
    /// HandleBoneAnimGrabEvent
    #[wow_data(expr=[0x24,0x56,0x47,0x33])]
    VG3,
    /// HandleBoneAnimGrabEvent
    #[wow_data(expr=[0x24,0x56,0x47,0x34])]
    VG4,
    /// HandleBoneAnimGrabEvent
    #[wow_data(expr=[0x24,0x56,0x47,0x35])]
    VG5,
    /// HandleBoneAnimGrabEvent
    #[wow_data(expr=[0x24,0x56,0x47,0x36])]
    VG6,
    /// HandleBoneAnimGrabEvent
    #[wow_data(expr=[0x24,0x56,0x47,0x37])]
    VG7,
    /// HandleBoneAnimGrabEvent
    #[wow_data(expr=[0x24,0x56,0x47,0x38])]
    VG8,
    /// HandleBoneAnimThrowEvent
    #[wow_data(expr=[0x24,0x56,0x54,0x30])]
    VT0,
    /// HandleBoneAnimThrowEvent
    #[wow_data(expr=[0x24,0x56,0x54,0x31])]
    VT1,
    /// HandleBoneAnimThrowEvent
    #[wow_data(expr=[0x24,0x56,0x54,0x32])]
    VT2,
    /// HandleBoneAnimThrowEvent
    #[wow_data(expr=[0x24,0x56,0x54,0x33])]
    VT3,
    /// HandleBoneAnimThrowEvent
    #[wow_data(expr=[0x24,0x56,0x54,0x34])]
    VT4,
    /// HandleBoneAnimThrowEvent
    #[wow_data(expr=[0x24,0x56,0x54,0x35])]
    VT5,
    /// HandleBoneAnimThrowEvent
    #[wow_data(expr=[0x24,0x56,0x54,0x36])]
    VT6,
    /// HandleBoneAnimThrowEvent
    #[wow_data(expr=[0x24,0x56,0x54,0x37])]
    VT7,
    /// HandleBoneAnimThrowEvent
    #[wow_data(expr=[0x24,0x56,0x54,0x38])]
    VT8,
    /// PlayUnitSound (wingGlide) | soundEffect ID is defined by CreatureSoundDataRec::m_soundWingGlideID
    #[wow_data(expr=[0x24,0x57,0x47,0x47])]
    WGG,
    /// FootstepAnimEventHit (left)
    #[wow_data(expr=[0x24,0x57,0x4c,0x30])]
    WL0,
    /// FootstepAnimEventHit (left)
    #[wow_data(expr=[0x24,0x57,0x4c,0x31])]
    WL1,
    /// FootstepAnimEventHit (left)
    #[wow_data(expr=[0x24,0x57,0x4c,0x32])]
    WL2,
    /// FootstepAnimEventHit (left)
    #[wow_data(expr=[0x24,0x57,0x4c,0x33])]
    WL3,
    /// Model Weapon Left Bot
    #[wow_data(expr=[0x24,0x57,0x4c,0x42])]
    WLB,
    /// Model Weapon Left Top
    #[wow_data(expr=[0x24,0x57,0x4c,0x54])]
    WLT,
    /// PlayUnitSound (wingFlap) | soundEffect ID is defined by CreatureSoundDataRec::m_soundWingFlapID
    #[wow_data(expr=[0x24,0x57,0x4e,0x47])]
    WNG,
    /// FootstepAnimEventHit (right)
    #[wow_data(expr=[0x24,0x57,0x52,0x30])]
    WR0,
    /// FootstepAnimEventHit (right)
    #[wow_data(expr=[0x24,0x57,0x52,0x31])]
    WR1,
    /// FootstepAnimEventHit (right)
    #[wow_data(expr=[0x24,0x57,0x52,0x32])]
    WR2,
    /// FootstepAnimEventHit (right)
    #[wow_data(expr=[0x24,0x57,0x52,0x33])]
    WR3,
    /// Model Weapon Right Bot
    #[wow_data(expr=[0x24,0x57,0x52,0x42])]
    WRB,
    /// Model Weapon Right Top
    #[wow_data(expr=[0x24,0x57,0x52,0x54])]
    WRT,
    /// Weapons | Bow Bottom, Weapon Trail Bottom position, also used for Bow String
    #[wow_data(expr=[0x24,0x57,0x54,0x42])]
    WTB,
    /// Weapons | Bow Top, Weapon Trail Top position
    #[wow_data(expr=[0x24,0x57,0x54,0x54])]
    WTT,
    ///  ? | Calls some function in the Object VMT. -- Not seen in 6.0.1.18179
    #[wow_data(expr=[0x24,0x57,0x57,0x47])]
    WWG,
    ///  ? | exploding ballista, that one has a really fucked up block. Oo
    #[wow_data(expr=[0x44,0x45,0x53,0x54])]
    DEST,
    /// Unit | not fired | Data: ?, seen on multiple models. Basilisk for example. (6801)
    #[wow_data(expr=[0x50,0x4f,0x49,0x4e])]
    POIN,
    ///  ? | Data: 601+, Used on wheels at vehicles.
    #[wow_data(expr=[0x57,0x48,0x45,0x45])]
    WHEE,
    ///  ? | Purpose unknown. Seen in well_vortex01.m2
    #[wow_data(expr=[0x42,0x4f,0x54,0x54])]
    BOTT,
    ///  ? | Purpose unknown. Seen in well_vortex01.m2
    #[wow_data(expr=[0x54,0x4f,0x50,0x30])]
    TOP0,

    #[wow_data(
        default,
        from_arm = "_ => Self::Unknown(value)",
        to_arm = "M2EventIdentifier::Unknown(val) => val"
    )]
    Unknown(MagicStr),
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub struct M2EventHeader {
    pub identifier: M2EventIdentifier,
    pub data: u32,
    pub bone_index: u32,
    pub position: C3Vector,
    #[wow_data(versioned)]
    pub enabled: M2AnimationBaseTrackHeader,
}

#[derive(Debug, Clone, WowDataR)]
#[wow_data(version = MD20Version, header = M2EventHeader)]
pub struct M2EventData {
    #[wow_data(versioned)]
    pub enabled: M2AnimationBaseTrackData,
}

#[derive(Debug, Clone)]
pub struct M2Event {
    pub header: M2EventHeader,
    pub data: M2EventData,
}
