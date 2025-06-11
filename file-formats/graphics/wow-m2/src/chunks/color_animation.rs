use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Write};

use crate::chunks::animation::M2AnimationBlock;
use crate::error::Result;
use crate::version::M2Version;

/// RGBA color structure
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct M2Color {
    /// Red component (0-1)
    pub r: f32,
    /// Green component (0-1)
    pub g: f32,
    /// Blue component (0-1)
    pub b: f32,
    /// Alpha component (0-1)
    pub a: f32,
}

impl M2Color {
    /// Parse a color from a reader
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let r = reader.read_f32_le()?;
        let g = reader.read_f32_le()?;
        let b = reader.read_f32_le()?;
        let a = reader.read_f32_le()?;

        Ok(Self { r, g, b, a })
    }

    /// Write a color to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_f32_le(self.r)?;
        writer.write_f32_le(self.g)?;
        writer.write_f32_le(self.b)?;
        writer.write_f32_le(self.a)?;

        Ok(())
    }

    /// Create a new color
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a new white color with full alpha
    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }

    /// Create a new black color with full alpha
    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }

    /// Create a new transparent color
    pub fn transparent() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

/// Color animation structure
#[derive(Debug, Clone)]
pub struct M2ColorAnimation {
    /// Animation for color RGB
    pub color: M2AnimationBlock<M2Color>,
    /// Animation for alpha
    pub alpha: M2AnimationBlock<f32>,
}

impl M2ColorAnimation {
    /// Parse a color animation from a reader
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let color = M2AnimationBlock::parse(reader)?;
        let alpha = M2AnimationBlock::parse(reader)?;

        Ok(Self { color, alpha })
    }

    /// Write a color animation to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.color.write(writer)?;
        self.alpha.write(writer)?;

        Ok(())
    }

    /// Convert this color animation to a different version (no version differences yet)
    pub fn convert(&self, _target_version: M2Version) -> Self {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_color_parse_write() {
        let color = M2Color::new(0.5, 0.6, 0.7, 1.0);

        let mut data = Vec::new();
        color.write(&mut data).unwrap();

        let mut cursor = Cursor::new(data);
        let parsed_color = M2Color::parse(&mut cursor).unwrap();

        assert_eq!(parsed_color.r, 0.5);
        assert_eq!(parsed_color.g, 0.6);
        assert_eq!(parsed_color.b, 0.7);
        assert_eq!(parsed_color.a, 1.0);
    }
}
