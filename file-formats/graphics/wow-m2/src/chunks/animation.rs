use custom_debug::Debug;
use std::{cmp, fmt};
use wow_utils::debug;

use crate::M2Error;
use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Seek, SeekFrom, Write};

use crate::common::{BoundingBox, ItemParser, M2Array, read_array};
use crate::error::Result;
use crate::version::M2Version;

/// Animation interpolation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M2InterpolationType {
    /// No interpolation
    None = 0,
    /// Linear interpolation
    Linear = 1,
    /// Bezier curve interpolation
    Bezier = 2,
    /// Hermite curve interpolation
    Hermite = 3,
}

impl M2InterpolationType {
    /// Parse from integer value
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Linear),
            2 => Some(Self::Bezier),
            3 => Some(Self::Hermite),
            _ => None,
        }
    }
}

bitflags::bitflags! {
    /// Animation flags as defined in the M2 format
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct M2Range {
    /// Minimum value
    pub minimum: f32,
    /// Maximum value
    pub maximum: f32,
}

impl M2Range {
    /// Parse a range from a reader
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let minimum = reader.read_f32_le()?;
        let maximum = reader.read_f32_le()?;

        Ok(Self { minimum, maximum })
    }

    /// Write a range to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_f32_le(self.minimum)?;
        writer.write_f32_le(self.maximum)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum TrackArray<T> {
    Single(M2Array<T>),
    Multiple(M2Array<M2Array<T>>),
}

impl<T> TrackArray<T> {
    pub fn parse_single<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(Self::Single(M2Array::parse(reader)?))
    }

    pub fn parse_multiple<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(Self::Multiple(M2Array::parse(reader)?))
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        match self {
            Self::Single(single) => single.write(writer),
            Self::Multiple(_multiple) => todo!("not implemented"),
        }
    }
}

/// An animation track header
#[derive(Debug, Clone)]
pub struct M2AnimationTrackHeader<T> {
    /// Interpolation type
    pub interpolation_type: M2InterpolationType,
    /// Global sequence ID or -1
    pub global_sequence: i16,
    pub interpolation_ranges: Option<M2Array<(u32, u32)>>,
    /// Timestamps
    pub timestamps: TrackArray<u32>,
    /// Values
    pub values: TrackArray<T>,
}

impl<T> M2AnimationTrackHeader<T> {
    /// Parse an animation track from a reader
    pub fn parse<R: Read>(reader: &mut R, version: u32) -> Result<Self> {
        let version = M2Version::from_header_version(version)
            .ok_or_else(|| M2Error::UnsupportedNumericVersion(version))?;

        let interpolation_type_raw = reader.read_u16_le()?;
        let interpolation_type = M2InterpolationType::from_u16(interpolation_type_raw)
            .unwrap_or(M2InterpolationType::None);

        let global_sequence = reader.read_i16_le()?;

        let interpolation_ranges = if version <= M2Version::TBC {
            Some(M2Array::parse(reader)?)
        } else {
            None
        };

        let timestamps = if version <= M2Version::TBC {
            TrackArray::parse_single(reader)?
        } else {
            TrackArray::parse_multiple(reader)?
        };

        let values = if version <= M2Version::TBC {
            TrackArray::parse_single(reader)?
        } else {
            TrackArray::parse_multiple(reader)?
        };

        Ok(Self {
            interpolation_type,
            interpolation_ranges,
            global_sequence,
            timestamps,
            values,
        })
    }

    /// Write an animation track to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u16_le(self.interpolation_type as u16)?;
        writer.write_i16_le(self.global_sequence)?;
        self.timestamps.write(writer)?;
        self.values.write(writer)?;

        Ok(())
    }

    pub fn new() -> Self {
        Self {
            interpolation_type: crate::chunks::animation::M2InterpolationType::None,
            global_sequence: -1,
            interpolation_ranges: None,
            timestamps: TrackArray::Multiple(M2Array::new(0, 0)),
            values: TrackArray::Multiple(M2Array::new(0, 0)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TrackVec<T> {
    Single(Vec<T>),
    Multiple(Vec<Vec<T>>),
}

impl<T> TrackVec<T> {
    pub fn from_trackarray<R, F>(reader: &mut R, array: &TrackArray<T>, parse_fn: F) -> Result<Self>
    where
        R: Read + Seek,
        F: Fn(&mut R) -> Result<T>,
    {
        Ok(match array {
            TrackArray::Single(single) => {
                Self::Single(read_array(reader, &single.convert(), &parse_fn)?)
            }
            TrackArray::Multiple(multiple) => {
                reader
                    .seek(SeekFrom::Start(multiple.offset as u64))
                    .map_err(M2Error::Io)?;

                let mut result = Vec::with_capacity(multiple.count as usize);
                for _ in 0..multiple.count {
                    let single: M2Array<T> = M2Array::parse(reader)?;
                    let item_end_position = reader.stream_position()?;
                    result.push(read_array(reader, &single.convert(), &parse_fn)?);
                    reader.seek(SeekFrom::Start(item_end_position))?;
                }

                Self::Multiple(result)
            }
        })
    }

    pub fn write<W: Write>(&self, _writer: &mut W) -> Result<()> {
        todo!("implement this")
    }
}

#[cfg(not(feature = "debug-print-all"))]
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
#[cfg(feature = "debug-print-all")]
pub fn trimmed_trackvec_fmt<T: fmt::Debug>(n: &T, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:#?}", n)
}

#[derive(Debug, Clone)]
pub struct M2AnimationTrack<T: fmt::Debug> {
    pub header: M2AnimationTrackHeader<T>,
    pub interpolation_ranges: Option<Vec<(u32, u32)>>,
    /// Timestamps
    #[debug(with = trimmed_trackvec_fmt)]
    pub timestamps: TrackVec<u32>,
    /// Values
    #[debug(with = trimmed_trackvec_fmt)]
    pub values: TrackVec<T>,
}

impl<T: fmt::Debug + ItemParser<T>> M2AnimationTrack<T> {
    pub fn parse<R: Read + Seek>(reader: &mut R, version: u32) -> Result<Self> {
        let header = M2AnimationTrackHeader::parse(reader, version)?;

        let header_end_position = reader.stream_position()?;

        let interpolation_ranges = match &header.interpolation_ranges {
            Some(interpolation_ranges) => {
                Some(read_array(reader, &interpolation_ranges.convert(), |r| {
                    Ok((r.read_u32_le()?, r.read_u32_le()?))
                })?)
            }
            None => None,
        };

        let timestamps =
            TrackVec::from_trackarray(reader, &header.timestamps, |r| Ok(r.read_u32_le()?))?;

        let values = TrackVec::from_trackarray(reader, &header.values, T::parse)?;

        reader.seek(SeekFrom::Start(header_end_position))?;

        Ok(Self {
            header,
            interpolation_ranges,
            timestamps,
            values,
        })
    }

    pub fn write<W: Write>(&self, _writer: &mut W) -> Result<()> {
        todo!("implement this")
    }

    pub fn new() -> Self {
        Self {
            header: M2AnimationTrackHeader::new(),
            interpolation_ranges: None,
            timestamps: TrackVec::Multiple(vec![vec![]]),
            values: TrackVec::Multiple(vec![vec![]]),
        }
    }
}

/// Animation block for a specific animation type
#[derive(Debug, Clone)]
pub struct M2AnimationBlock<T> {
    /// Animation track
    pub track: M2AnimationTrackHeader<T>,
    /// Data type (phantom data)
    _phantom: std::marker::PhantomData<T>,
}

impl<T> M2AnimationBlock<T> {
    /// Create a new animation block from a track
    pub fn new(track: M2AnimationTrackHeader<T>) -> Self {
        Self {
            track,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Parse an animation block from a reader
    pub fn parse<R: Read>(reader: &mut R, version: u32) -> Result<Self> {
        let track = M2AnimationTrackHeader::parse(reader, version)?;

        Ok(Self {
            track,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Write an animation block to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.track.write(writer)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum M2AnimationBlending {
    Time(u32),
    InOut(u16, u16),
}

impl M2AnimationBlending {
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        match self {
            Self::Time(val) => {
                writer.write_u32_le(*val)?;
            }
            Self::InOut(val_in, val_out) => {
                writer.write_u16_le(*val_in)?;
                writer.write_u16_le(*val_out)?;
            }
        }

        Ok(())
    }
}

/// Animation data for a model
#[derive(Debug, Clone)]
pub struct M2Animation {
    /// Animation ID
    pub animation_id: u16,
    /// Sub-animation ID (variation index)
    pub sub_animation_id: u16,
    /// Start timestamp (up to BC) or Duration (LK+) in milliseconds
    pub start_timestamp: u32,
    /// End timestamp (Classic only)
    pub end_timestamp: Option<u32>,
    /// Movement speed
    pub movement_speed: f32,
    /// Flags
    pub flags: M2AnimationFlags,
    /// Frequency/Probability (renamed in later versions)
    pub frequency: i16,
    /// Padding/Realignment
    pub padding: u16,
    /// Replay range
    pub replay: M2Range,
    pub blending: M2AnimationBlending,
    pub bounding_box: BoundingBox,
    pub bounding_radius: f32,
    pub next_animation: i16,
    pub next_alias: u16,
}

impl M2Animation {
    /// Parse an animation from a reader based on the M2 version
    pub fn parse<R: Read>(reader: &mut R, version: u32) -> Result<Self> {
        let version = M2Version::from_header_version(version)
            .ok_or_else(|| M2Error::UnsupportedNumericVersion(version))?;

        let animation_id = reader.read_u16_le()?;
        let sub_animation_id = reader.read_u16_le()?;
        let start_timestamp = reader.read_u32_le()?;
        let end_timestamp = if version <= M2Version::TBC {
            Some(reader.read_u32_le()?)
        } else {
            None
        };

        let movement_speed = reader.read_f32_le()?;
        let flags = M2AnimationFlags::from_bits_retain(reader.read_u32_le()?);
        let frequency = reader.read_i16_le()?;
        let padding = reader.read_u16_le()?;
        let replay = M2Range::parse(reader)?;
        let blending = if version <= M2Version::MoP {
            M2AnimationBlending::Time(reader.read_u32_le()?)
        } else {
            M2AnimationBlending::InOut(reader.read_u16_le()?, reader.read_u16_le()?)
        };

        let bounding_box = BoundingBox::read(reader)?;
        let bounding_radius = reader.read_f32_le()?;
        let next_animation = reader.read_i16_le()?;
        let next_alias = reader.read_u16_le()?;

        Ok(Self {
            animation_id,
            sub_animation_id,
            start_timestamp,
            end_timestamp,
            movement_speed,
            flags,
            frequency,
            padding,
            replay,
            blending,
            bounding_box,
            bounding_radius,
            next_animation,
            next_alias,
        })
    }

    /// Write an animation to a writer
    pub fn write<W: Write>(&self, writer: &mut W, version: u32) -> Result<()> {
        let version = M2Version::from_header_version(version)
            .ok_or_else(|| M2Error::UnsupportedNumericVersion(version))?;

        writer.write_u16_le(self.animation_id)?;
        writer.write_u16_le(self.sub_animation_id)?;
        writer.write_u32_le(self.start_timestamp)?;

        if version <= M2Version::TBC {
            writer.write_u32_le(self.end_timestamp.unwrap_or(self.start_timestamp + 1000))?;
        }

        writer.write_f32_le(self.movement_speed)?;
        writer.write_u32_le(self.flags.bits())?;
        writer.write_i16_le(self.frequency)?;
        writer.write_u16_le(self.padding)?;
        self.replay.write(writer)?;
        self.blending.write(writer)?;
        self.bounding_box.write(writer)?;
        writer.write_f32_le(self.bounding_radius)?;
        writer.write_i16_le(self.next_animation)?;
        writer.write_u16_le(self.next_alias)?;

        Ok(())
    }

    /// Convert this animation to a different version (no version differences for animations yet)
    pub fn convert(&self, _target_version: M2Version) -> Self {
        self.clone()
    }
}
