use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Seek, Write};

use crate::common::{M2Array, M2Parse, M2Vec};
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
    pub struct M2AnimationFlags: u16 {
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

/// An animation track header
#[derive(Debug, Clone)]
pub struct M2AnimationTrack<T: M2Parse> {
    /// Interpolation type
    pub interpolation_type: M2InterpolationType,
    /// Global sequence ID or -1
    pub global_sequence: i16,
    // Interpolation ranges
    pub interpolation_ranges: M2Array<(u32, u32)>,
    /// Timestamps
    pub timestamps: M2Array<u32>,
    /// Values
    pub values: M2Vec<T>,
}

impl<T: M2Parse> Default for M2AnimationTrack<T> {
    fn default() -> Self {
        Self {
            interpolation_type: M2InterpolationType::None,
            global_sequence: -1,
            interpolation_ranges: M2Array::new(0, 0),
            timestamps: M2Array::new(0, 0),
            values: M2Vec::new(),
        }
    }
}

impl<T: M2Parse> M2AnimationTrack<T> {
    /// Parse an animation track from a reader
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let interpolation_type_raw = reader.read_u16_le()?;
        let interpolation_type = M2InterpolationType::from_u16(interpolation_type_raw)
            .unwrap_or(M2InterpolationType::None);

        let global_sequence = reader.read_i16_le()?;
        let interpolation_ranges = M2Array::parse(reader)?;
        let timestamps = M2Array::parse(reader)?;
        let values = M2Vec::<T>::parse(reader)?;

        Ok(Self {
            interpolation_type,
            global_sequence,
            interpolation_ranges,
            timestamps,
            values,
        })
    }

    /// Write an animation track to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u16_le(self.interpolation_type as u16)?;
        writer.write_i16_le(self.global_sequence)?;
        self.interpolation_ranges.write(writer)?;
        self.timestamps.write(writer)?;
        self.values.write(writer)?;

        Ok(())
    }
}

/// Animation block for a specific animation type
#[derive(Debug, Default, Clone)]
pub struct M2AnimationBlock<T: M2Parse> {
    /// Animation track
    pub track: M2AnimationTrack<T>,
    /// Data type (phantom data)
    _phantom: std::marker::PhantomData<T>,
}

impl<T: M2Parse> M2AnimationBlock<T> {
    /// Create a new animation block from a track
    pub fn new(track: M2AnimationTrack<T>) -> Self {
        Self {
            track,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Parse an animation block from a reader
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let track = M2AnimationTrack::parse(reader)?;

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

/// Animation data for a model
#[derive(Debug, Clone)]
pub struct M2Animation {
    /// Animation ID
    pub animation_id: u16,
    /// Sub-animation ID (variation index)
    pub sub_animation_id: u16,
    /// Start timestamp (Classic) or Duration (BC+) in milliseconds
    pub start_timestamp: u32,
    /// End timestamp (Classic only)
    pub end_timestamp: Option<u32>,
    /// Movement speed
    pub movement_speed: f32,
    /// Flags
    pub flags: u32,
    /// Frequency/Probability (renamed in later versions)
    pub frequency: i16,
    /// Padding/Realignment
    pub padding: u16,
    /// Replay range (Classic only)
    pub replay: Option<M2Range>,
    /// Minimum extent (BC+ only)
    pub minimum_extent: Option<[f32; 3]>,
    /// Maximum extent (BC+ only)
    pub maximum_extent: Option<[f32; 3]>,
    /// Extent radius (BC+ only)
    pub extent_radius: Option<f32>,
    /// Next animation ID (BC+ only)
    pub next_animation: Option<i16>,
    /// Aliasing (BC+ only)
    pub aliasing: Option<u16>,
}

impl M2Animation {
    /// Parse an animation from a reader based on the M2 version
    pub fn parse<R: Read>(reader: &mut R, version: u32) -> Result<Self> {
        let animation_id = reader.read_u16_le()?;
        let sub_animation_id = reader.read_u16_le()?;

        // Version differences: Classic (256) vs BC+ (260+)
        if version <= 256 {
            // Vanilla format
            let start_timestamp = reader.read_u32_le()?;
            let end_timestamp = reader.read_u32_le()?;
            let movement_speed = reader.read_f32_le()?;
            let flags = reader.read_u32_le()?;
            let frequency = reader.read_i16_le()?;
            let padding = reader.read_u16_le()?;
            let replay = M2Range::parse(reader)?;

            Ok(Self {
                animation_id,
                sub_animation_id,
                start_timestamp,
                end_timestamp: Some(end_timestamp),
                movement_speed,
                flags,
                frequency,
                padding,
                replay: Some(replay),
                minimum_extent: None,
                maximum_extent: None,
                extent_radius: None,
                next_animation: None,
                aliasing: None,
            })
        } else {
            // BC+ format
            let duration = reader.read_u32_le()?;
            let movement_speed = reader.read_f32_le()?;
            let flags = reader.read_u32_le()?;
            let frequency = reader.read_i16_le()?;
            let padding = reader.read_u16_le()?;

            let mut minimum_extent = [0.0; 3];
            let mut maximum_extent = [0.0; 3];

            for item in &mut minimum_extent {
                *item = reader.read_f32_le()?;
            }

            for item in &mut maximum_extent {
                *item = reader.read_f32_le()?;
            }

            let extent_radius = reader.read_f32_le()?;
            let next_animation = reader.read_i16_le()?;
            let aliasing = reader.read_u16_le()?;

            Ok(Self {
                animation_id,
                sub_animation_id,
                start_timestamp: duration, // In BC+, this field is duration
                end_timestamp: None,
                movement_speed,
                flags,
                frequency,
                padding,
                replay: None,
                minimum_extent: Some(minimum_extent),
                maximum_extent: Some(maximum_extent),
                extent_radius: Some(extent_radius),
                next_animation: Some(next_animation),
                aliasing: Some(aliasing),
            })
        }
    }

    /// Write an animation to a writer
    pub fn write<W: Write>(&self, writer: &mut W, version: u32) -> Result<()> {
        writer.write_u16_le(self.animation_id)?;
        writer.write_u16_le(self.sub_animation_id)?;

        if version <= 256 {
            // Vanilla format
            writer.write_u32_le(self.start_timestamp)?;
            writer.write_u32_le(self.end_timestamp.unwrap_or(self.start_timestamp + 1000))?;
            writer.write_f32_le(self.movement_speed)?;
            writer.write_u32_le(self.flags)?;
            writer.write_i16_le(self.frequency)?;
            writer.write_u16_le(self.padding)?;

            if let Some(replay) = &self.replay {
                replay.write(writer)?;
            } else {
                // Default replay range
                M2Range {
                    minimum: 0.0,
                    maximum: 1.0,
                }
                .write(writer)?;
            }
        } else {
            // BC+ format
            writer.write_u32_le(self.start_timestamp)?; // This is duration in BC+
            writer.write_f32_le(self.movement_speed)?;
            writer.write_u32_le(self.flags)?;
            writer.write_i16_le(self.frequency)?;
            writer.write_u16_le(self.padding)?;

            let minimum_extent = self.minimum_extent.unwrap_or([0.0, 0.0, 0.0]);
            let maximum_extent = self.maximum_extent.unwrap_or([0.0, 0.0, 0.0]);

            for &value in &minimum_extent {
                writer.write_f32_le(value)?;
            }

            for &value in &maximum_extent {
                writer.write_f32_le(value)?;
            }

            writer.write_f32_le(self.extent_radius.unwrap_or(0.0))?;
            writer.write_i16_le(self.next_animation.unwrap_or(-1))?;
            writer.write_u16_le(self.aliasing.unwrap_or(0))?;
        }

        Ok(())
    }

    /// Convert this animation to a different version (no version differences for animations yet)
    pub fn convert(&self, _target_version: M2Version) -> Self {
        self.clone()
    }
}
