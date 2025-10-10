use std::io::{Read, Seek, Write};

use crate::chunks::animation::M2AnimationBlock;
use crate::error::Result;
use crate::version::M2Version;

/// Transparency animation structure
#[derive(Debug, Default, Clone)]
pub struct M2TransparencyAnimation {
    /// Alpha animation
    pub alpha: M2AnimationBlock<f32>,
}

impl M2TransparencyAnimation {
    /// Parse a transparency animation from a reader
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let alpha = M2AnimationBlock::parse(reader)?;

        Ok(Self { alpha })
    }

    /// Write a transparency animation to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.alpha.write(writer)?;

        Ok(())
    }

    /// Convert this transparency animation to a different version (no version differences yet)
    pub fn convert(&self, _target_version: M2Version) -> Self {
        self.clone()
    }

    /// Create a new transparency animation with default values
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_transparency_animation_parse_write() {
        let mut data = Vec::new();

        // Alpha animation track
        data.extend_from_slice(&1u16.to_le_bytes()); // Interpolation type (Linear)
        data.extend_from_slice(&(-1i16).to_le_bytes()); // Global sequence
        data.extend_from_slice(&0u32.to_le_bytes()); // Interpolation ranges count
        data.extend_from_slice(&0u32.to_le_bytes()); // Interpolation ranges offset
        data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps count
        data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps offset
        data.extend_from_slice(&0u32.to_le_bytes()); // Values count
        data.extend_from_slice(&0u32.to_le_bytes()); // Values offset

        let mut cursor = Cursor::new(data);
        let trans_anim = M2TransparencyAnimation::parse(&mut cursor).unwrap();

        // Test write
        let mut output = Vec::new();
        trans_anim.write(&mut output).unwrap();

        // Check output size (should be the same as input)
        assert_eq!(output.len(), cursor.get_ref().len());
    }
}
