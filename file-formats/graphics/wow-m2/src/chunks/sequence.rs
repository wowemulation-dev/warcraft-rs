use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Write};

use crate::error::Result;
use crate::version::M2Version;

/// Sequence data for an animation
#[derive(Debug, Clone)]
pub struct M2Sequence {
    /// Animation ID
    pub animation_id: u16,
    /// SubAnimation ID
    pub sub_animation_id: u16,
    /// Start frame
    pub start: u32,
    /// End frame
    pub end: u32,
    /// Movement speed
    pub movement_speed: f32,
    /// Flags
    pub flags: u32,
    /// Frequency
    pub frequency: i16,
    /// Padding
    pub padding: u16,
    /// Replay information
    pub replay: M2AnimationRepeat,
    /// Blend time
    pub blend_time: u16,
    /// Bounds
    pub bounds: M2Bounds,
    /// Variation next
    pub variation_next: i16,
    /// Alias next
    pub alias_next: u16,
}

/// Animation repeat parameters
#[derive(Debug, Clone)]
pub struct M2AnimationRepeat {
    /// Minimum repeat count
    pub minimum: u32,
    /// Maximum repeat count
    pub maximum: u32,
}

impl M2AnimationRepeat {
    /// Parse animation repeat parameters from a reader
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let minimum = reader.read_u32_le()?;
        let maximum = reader.read_u32_le()?;

        Ok(Self { minimum, maximum })
    }

    /// Write animation repeat parameters to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32_le(self.minimum)?;
        writer.write_u32_le(self.maximum)?;

        Ok(())
    }
}

/// Animation bounds
#[derive(Debug, Clone)]
pub struct M2Bounds {
    /// Minimum extent
    pub minimum: [f32; 3],
    /// Maximum extent
    pub maximum: [f32; 3],
    /// Bounds radius
    pub radius: f32,
}

impl M2Bounds {
    /// Parse animation bounds from a reader
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let mut minimum = [0.0; 3];
        let mut maximum = [0.0; 3];

        for item in &mut minimum {
            *item = reader.read_f32_le()?;
        }

        for item in &mut maximum {
            *item = reader.read_f32_le()?;
        }

        let radius = reader.read_f32_le()?;

        Ok(Self {
            minimum,
            maximum,
            radius,
        })
    }

    /// Write animation bounds to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        for &value in &self.minimum {
            writer.write_f32_le(value)?;
        }

        for &value in &self.maximum {
            writer.write_f32_le(value)?;
        }

        writer.write_f32_le(self.radius)?;

        Ok(())
    }
}

impl M2Sequence {
    /// Parse a sequence from a reader based on the M2 version
    pub fn parse<R: Read>(reader: &mut R, _version: u32) -> Result<Self> {
        let animation_id = reader.read_u16_le()?;
        let sub_animation_id = reader.read_u16_le()?;
        let start = reader.read_u32_le()?;
        let end = reader.read_u32_le()?;
        let movement_speed = reader.read_f32_le()?;
        let flags = reader.read_u32_le()?;
        let frequency = reader.read_i16_le()?;
        let padding = reader.read_u16_le()?;
        let replay = M2AnimationRepeat::parse(reader)?;
        let blend_time = reader.read_u16_le()?;
        // Skip 2 bytes of padding
        reader.read_u16_le()?;
        let bounds = M2Bounds::parse(reader)?;
        let variation_next = reader.read_i16_le()?;
        let alias_next = reader.read_u16_le()?;

        Ok(Self {
            animation_id,
            sub_animation_id,
            start,
            end,
            movement_speed,
            flags,
            frequency,
            padding,
            replay,
            blend_time,
            bounds,
            variation_next,
            alias_next,
        })
    }

    /// Write a sequence to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u16_le(self.animation_id)?;
        writer.write_u16_le(self.sub_animation_id)?;
        writer.write_u32_le(self.start)?;
        writer.write_u32_le(self.end)?;
        writer.write_f32_le(self.movement_speed)?;
        writer.write_u32_le(self.flags)?;
        writer.write_i16_le(self.frequency)?;
        writer.write_u16_le(self.padding)?;
        self.replay.write(writer)?;
        writer.write_u16_le(self.blend_time)?;
        // Write 2 bytes of padding
        writer.write_u16_le(0)?;
        self.bounds.write(writer)?;
        writer.write_i16_le(self.variation_next)?;
        writer.write_u16_le(self.alias_next)?;

        Ok(())
    }

    /// Convert this sequence to a different version (no version differences for sequences yet)
    pub fn convert(&self, _target_version: M2Version) -> Self {
        self.clone()
    }

    /// Create a new sequence with default values
    pub fn new(animation_id: u16, sub_animation_id: u16, start: u32, end: u32) -> Self {
        Self {
            animation_id,
            sub_animation_id,
            start,
            end,
            movement_speed: 0.0,
            flags: 0,
            frequency: 0,
            padding: 0,
            replay: M2AnimationRepeat {
                minimum: 0,
                maximum: 0,
            },
            blend_time: 0,
            bounds: M2Bounds {
                minimum: [0.0, 0.0, 0.0],
                maximum: [0.0, 0.0, 0.0],
                radius: 0.0,
            },
            variation_next: -1,
            alias_next: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_sequence_parse_write() {
        let sequence = M2Sequence::new(1, 0, 0, 30);

        // Write to buffer
        let mut buffer = Vec::new();
        sequence.write(&mut buffer).unwrap();

        // Read back
        let mut cursor = Cursor::new(buffer);
        let parsed =
            M2Sequence::parse(&mut cursor, M2Version::Classic.to_header_version()).unwrap();

        // Verify
        assert_eq!(parsed.animation_id, 1);
        assert_eq!(parsed.sub_animation_id, 0);
        assert_eq!(parsed.start, 0);
        assert_eq!(parsed.end, 30);
    }
}
