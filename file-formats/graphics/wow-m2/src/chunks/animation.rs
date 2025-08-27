use custom_debug::Debug;
#[cfg(feature = "trimmed-debug-output")]
use std::cmp;
use std::fmt;
use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{BoundingBox, C3Vector, VWowDataR, WowArray};
use wow_data_derive::{WowDataR, WowEnumFrom, WowHeaderR, WowHeaderW};
#[cfg(feature = "trimmed-debug-output")]
use wow_utils::debug;

use crate::version::MD20Version;

#[derive(Debug, Clone, Copy, PartialEq, Eq, WowEnumFrom, WowHeaderR, WowHeaderW)]
#[wow_data(from_type=u16)]
pub enum M2InterpolationType {
    #[wow_data(lit = 0)]
    None = 0,
    #[wow_data(lit = 1)]
    Linear = 1,
    #[wow_data(lit = 2)]
    Bezier = 2,
    #[wow_data(lit = 3)]
    Hermite = 3,
}

bitflags::bitflags! {
    /// Animation flags as defined in the M2 format
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, WowHeaderR, WowHeaderW)]
    #[wow_data(bitflags=u32)]
    pub struct M2AnimationFlags: u32 {
        /// Animation has translation keyframes
        const HAS_TRANSLATION = 0x1;
        /// Animation has rotation keyframes
        const HAS_ROTATION = 0x2;
        /// Animation has scaling keyframes
        const HAS_SCALING = 0x4;
        /// Animation is in world space (instead of local model space)
        const WORLD_SPACE = 0x8;
        /// Animation has billboarded rotation keyframes
        const BILLBOARD_ROTATION = 0x10;
        const PRIMARY_BONE_SEQUENCE = 0x20;
        const IS_ALIAS = 0x40;
        const BLENDED_ANIMATION = 0x80;
        /// sequence stored in model?
        const UNKNOWN_0x100 = 0x100;
        const BLEND_TIME_IN_OUT = 0x200;
    }
}

/// Animation value ranges
#[derive(Debug, Clone, Default, PartialEq, WowHeaderR, WowHeaderW)]
pub struct M2Range {
    pub minimum: f32,
    pub maximum: f32,
}

#[derive(Debug, Clone, PartialEq, WowHeaderR, WowHeaderW)]
pub struct M2Box {
    pub rotation_speed_min: C3Vector,
    pub rotation_speed_max: C3Vector,
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub enum TrackArray<T: WowHeaderR + WowHeaderW> {
    Single(WowArray<T>),

    #[wow_data(read_if = version >= MD20Version::WotLK)]
    Multiple(WowArray<WowArray<T>>),
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub enum M2InterpolationRangeHeader {
    None,

    #[wow_data(read_if = version <= MD20Version::TBCV4)]
    Some(WowArray<(u32, u32)>),
}

#[derive(Debug, Clone)]
pub enum M2InterpolationRange {
    None,

    Some(Vec<(u32, u32)>),
}

impl VWowDataR<MD20Version, M2InterpolationRangeHeader> for M2InterpolationRange {
    fn new_from_header<R: Read + Seek>(
        reader: &mut R,
        header: &M2InterpolationRangeHeader,
    ) -> WDResult<Self> {
        Ok(match header {
            M2InterpolationRangeHeader::Some(array) => Self::Some(array.wow_read_to_vec(reader)?),
            M2InterpolationRangeHeader::None => Self::None,
        })
    }
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub struct M2AnimationBaseTrackHeader {
    pub interpolation_type: M2InterpolationType,
    pub global_sequence: i16,
    #[wow_data(versioned)]
    pub interpolation_ranges: M2InterpolationRangeHeader,
    #[wow_data(versioned)]
    pub timestamps: TrackArray<u32>,
}

#[derive(Debug, Clone, WowDataR)]
#[wow_data(version = MD20Version, header=M2AnimationBaseTrackHeader)]
pub struct M2AnimationBaseTrackData {
    #[wow_data(versioned)]
    pub interpolation_ranges: M2InterpolationRange,
    #[wow_data(versioned)]
    pub timestamps: TrackVec<u32>,
}

#[derive(Debug, Clone)]
pub struct M2AnimationBaseTrack {
    pub header: M2AnimationBaseTrackHeader,
    pub data: M2AnimationBaseTrackData,
}

/// An animation track header
#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub struct M2AnimationTrackHeader<T: WowHeaderR + WowHeaderW> {
    pub interpolation_type: M2InterpolationType,
    pub global_sequence: i16,
    #[wow_data(versioned)]
    pub interpolation_ranges: M2InterpolationRangeHeader,
    #[wow_data(versioned)]
    pub timestamps: TrackArray<u32>,
    #[wow_data(versioned)]
    pub values: TrackArray<T>,
}

impl<T: WowHeaderR + WowHeaderW> M2AnimationTrackHeader<T> {
    pub fn new() -> Self {
        Self {
            interpolation_type: crate::chunks::animation::M2InterpolationType::None,
            global_sequence: -1,
            interpolation_ranges: M2InterpolationRangeHeader::None,
            timestamps: TrackArray::Multiple(WowArray::new(0, 0)),
            values: TrackArray::Multiple(WowArray::new(0, 0)),
        }
    }
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
pub struct M2SplineKey<T: WowHeaderR + WowHeaderW> {
    pub value: T,
    pub in_tan: T,
    pub out_tan: T,
}

#[derive(Debug, Clone)]
pub enum TrackVec<T> {
    Single(Vec<T>),
    Multiple(Vec<Vec<T>>),
}

impl<T: WowHeaderR + WowHeaderW> VWowDataR<MD20Version, TrackArray<T>> for TrackVec<T> {
    fn new_from_header<R: Read + Seek>(reader: &mut R, header: &TrackArray<T>) -> WDResult<Self> {
        Ok(match header {
            TrackArray::Multiple(array) => Self::Multiple(array.wow_read_to_vec_r(reader)?),
            TrackArray::Single(array) => Self::Single(array.wow_read_to_vec(reader)?),
        })
    }
}

#[cfg(feature = "trimmed-debug-output")]
pub fn trimmed_trackvec_fmt<T: fmt::Debug>(n: &TrackVec<T>, f: &mut fmt::Formatter) -> fmt::Result {
    match n {
        TrackVec::Single(single) => debug::trimmed_collection_fmt(single, f),
        TrackVec::Multiple(multiple) => {
            let end = cmp::min(debug::FIRST_N_ELEMENTS, multiple.len());
            let first_three = &multiple[..end];
            let num_elements = cmp::max(0, multiple.len() - first_three.len());

            write!(f, "[")?;

            let items_len = first_three.len();
            for i in 0..items_len {
                if i > 0 {
                    write!(f, ", ")?;
                }
                let item = &first_three[i];
                debug::trimmed_collection_fmt(item, f)?;
            }

            if num_elements == 0 {
                write!(f, "]")
            } else {
                write!(f, "] + {} elements", num_elements)
            }
        }
    }
}
#[cfg(not(feature = "trimmed-debug-output"))]
pub fn trimmed_trackvec_fmt<T: fmt::Debug>(n: &T, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:#?}", n)
}

#[derive(Debug, Clone, WowDataR)]
#[wow_data(version = MD20Version, header = M2AnimationTrackHeader<T>)]
pub struct M2AnimationTrackData<T: fmt::Debug + WowHeaderR + WowHeaderW> {
    #[wow_data(versioned)]
    pub interpolation_ranges: M2InterpolationRange,

    #[debug(with = trimmed_trackvec_fmt)]
    #[wow_data(versioned)]
    pub timestamps: TrackVec<u32>,

    #[debug(with = trimmed_trackvec_fmt)]
    #[wow_data(versioned)]
    pub values: TrackVec<T>,
}

impl<T: fmt::Debug + WowHeaderR + WowHeaderW> M2AnimationTrackData<T> {
    pub fn new() -> Self {
        Self {
            interpolation_ranges: M2InterpolationRange::None,
            timestamps: TrackVec::Multiple(vec![vec![]]),
            values: TrackVec::Multiple(vec![vec![]]),
        }
    }
}

/// Animation block for a specific animation type
#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub struct M2AnimationBlock<T: WowHeaderR + WowHeaderW> {
    #[wow_data(versioned)]
    pub track: M2AnimationTrackHeader<T>,

    #[wow_data(override_read = std::marker::PhantomData)]
    _phantom: std::marker::PhantomData<T>,
}

impl<T: WowHeaderR + WowHeaderW> M2AnimationBlock<T> {
    pub fn new(track: M2AnimationTrackHeader<T>) -> Self {
        Self {
            track,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
pub struct M2FakeAnimationBlockHeader<T: WowHeaderR + WowHeaderW> {
    pub timestamps: WowArray<u16>,
    pub keys: WowArray<u16>,
    pub values: WowArray<T>,
}

#[derive(Debug, Clone, WowDataR)]
#[wow_data(header = M2FakeAnimationBlockHeader<T>)]
pub struct M2FakeAnimationBlockData<T: WowHeaderR + WowHeaderW> {
    pub timestamps: Vec<u16>,
    pub keys: Vec<u16>,
    pub values: Vec<T>,
}

#[derive(Debug, Clone)]
pub struct M2FakeAnimationBlock<T: WowHeaderR + WowHeaderW> {
    pub header: M2FakeAnimationBlockHeader<T>,
    pub data: M2FakeAnimationBlockData<T>,
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub enum M2AnimationTiming {
    StartEnd(u32, u32),

    #[wow_data(read_if = version >= MD20Version::WotLK)]
    Duration(u32),
}

impl Default for M2AnimationTiming {
    fn default() -> Self {
        Self::Duration(0)
    }
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub enum M2AnimationBlending {
    Time(u32),

    #[wow_data(read_if = version >= MD20Version::BfAPlus)]
    InOut(u16, u16),
}

impl Default for M2AnimationBlending {
    fn default() -> Self {
        Self::InOut(0, 0)
    }
}

/// Animation data for a model
#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub struct M2Animation {
    pub animation_id: u16,
    pub sub_animation_id: u16,
    #[wow_data(versioned)]
    pub timing: M2AnimationTiming,
    pub movement_speed: f32,
    pub flags: M2AnimationFlags,
    /// Frequency/Probability (renamed in later versions)
    pub frequency: i16,
    pub padding: u16,
    /// Replay range
    pub replay: M2Range,
    #[wow_data(versioned)]
    pub blending: M2AnimationBlending,
    pub bounding_box: BoundingBox,
    pub bounding_radius: f32,
    pub next_animation: i16,
    pub next_alias: u16,
}

#[derive(Debug, Clone, Default, PartialEq, WowHeaderR, WowHeaderW)]
pub struct M2SequenceFallback {
    pub fallback_animation_id: i16,
    pub flags: u16,
}
