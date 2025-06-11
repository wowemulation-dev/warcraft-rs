use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Write};

use crate::error::Result;
use crate::version::M2Version;

bitflags::bitflags! {
    /// Render flags as defined in the M2 format
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct M2RenderFlags: u16 {
        /// Unlit
        const UNLIT = 0x01;
        /// Unfogged
        const UNFOGGED = 0x02;
        /// No backface culling
        const NO_BACKFACE_CULLING = 0x04;
        /// No z-buffer
        const NO_ZBUFFER = 0x08;
        /// Affeceted by projection
        const AFFECTED_BY_PROJECTION = 0x10;
        /// Depth test
        const DEPTH_TEST = 0x20;
        /// Depth write
        const DEPTH_WRITE = 0x40;
        /// Unused in code
        const UNUSED = 0x80;
        /// Shadow batch related 1
        const SHADOW_BATCH_1 = 0x100;
        /// Shadow batch related 2
        const SHADOW_BATCH_2 = 0x200;
        /// Unknown
        const UNKNOWN_400 = 0x400;
        /// Unknown
        const UNKNOWN_800 = 0x800;
    }
}

bitflags::bitflags! {
    /// Blend modes as defined in the M2 format
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct M2BlendMode: u16 {
        /// Blend mode: opaque
        const OPAQUE = 0;
        /// Blend mode: alpha key
        const ALPHA_KEY = 1;
        /// Blend mode: alpha
        const ALPHA = 2;
        /// Blend mode: no alpha add
        const NO_ALPHA_ADD = 3;
        /// Blend mode: add
        const ADD = 4;
        /// Blend mode: mod
        const MOD = 5;
        /// Blend mode: mod2x
        const MOD2X = 6;
        /// Blend mode for MoP and later: blend add
        const BLEND_ADD = 7;
    }
}

/// Material texture transformations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M2TexTransformType {
    /// No texture transform
    None = 0,
    /// Scroll texture
    Scroll = 1,
    /// Rotate texture
    Rotate = 2,
    /// Scale texture
    Scale = 3,
    /// Stretch texture based on time
    Stretch = 4,
    /// Transform texture based on camera
    Camera = 5,
}

impl M2TexTransformType {
    /// Parse from integer value
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Scroll),
            2 => Some(Self::Rotate),
            3 => Some(Self::Scale),
            4 => Some(Self::Stretch),
            5 => Some(Self::Camera),
            _ => None,
        }
    }
}

/// Represents a material layer (render flags) in an M2 model
/// This corresponds to the ModelRenderFlagsM2 structure in WMVx
#[derive(Debug, Clone)]
pub struct M2Material {
    /// Render flags
    pub flags: M2RenderFlags,
    /// Blend mode
    pub blend_mode: M2BlendMode,
}

impl M2Material {
    /// Parse a material from a reader based on the M2 version
    pub fn parse<R: Read>(reader: &mut R, _version: u32) -> Result<Self> {
        let flags = M2RenderFlags::from_bits_retain(reader.read_u16_le()?);
        let blend_mode_raw = reader.read_u16_le()?;
        let blend_mode = M2BlendMode::from_bits_retain(blend_mode_raw);

        Ok(Self { flags, blend_mode })
    }

    /// Write a material to a writer based on the M2 version
    pub fn write<W: Write>(&self, writer: &mut W, _version: u32) -> Result<()> {
        writer.write_u16_le(self.flags.bits())?;
        writer.write_u16_le(self.blend_mode.bits())?;
        Ok(())
    }

    /// Convert this material to a different version
    pub fn convert(&self, _target_version: M2Version) -> Self {
        // Materials have the same structure across all versions
        self.clone()
    }

    /// Create a new material with default values
    pub fn new(blend_mode: M2BlendMode) -> Self {
        Self {
            flags: M2RenderFlags::DEPTH_TEST | M2RenderFlags::DEPTH_WRITE,
            blend_mode,
        }
    }

    /// Calculate the size of this material in bytes for a specific version
    pub fn size_in_bytes(_version: M2Version) -> usize {
        4 // flags (2) + blend_mode (2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_material_parse() {
        let mut data = Vec::new();

        // Flags (DEPTH_TEST | DEPTH_WRITE)
        data.extend_from_slice(
            &(M2RenderFlags::DEPTH_TEST | M2RenderFlags::DEPTH_WRITE)
                .bits()
                .to_le_bytes(),
        );

        // Blend mode (ALPHA)
        data.extend_from_slice(&M2BlendMode::ALPHA.bits().to_le_bytes());

        let mut cursor = Cursor::new(data);
        let material =
            M2Material::parse(&mut cursor, M2Version::Classic.to_header_version()).unwrap();

        assert_eq!(
            material.flags,
            M2RenderFlags::DEPTH_TEST | M2RenderFlags::DEPTH_WRITE
        );
        assert_eq!(material.blend_mode, M2BlendMode::ALPHA);
    }

    #[test]
    fn test_material_write() {
        let material = M2Material {
            flags: M2RenderFlags::DEPTH_TEST | M2RenderFlags::DEPTH_WRITE,
            blend_mode: M2BlendMode::ALPHA,
        };

        let mut data = Vec::new();
        material
            .write(&mut data, M2Version::Classic.to_header_version())
            .unwrap();

        // Should be 4 bytes total
        assert_eq!(data.len(), 4);

        // Check the written data
        assert_eq!(
            data[0..2],
            (M2RenderFlags::DEPTH_TEST | M2RenderFlags::DEPTH_WRITE)
                .bits()
                .to_le_bytes()
        );
        assert_eq!(data[2..4], M2BlendMode::ALPHA.bits().to_le_bytes());
    }

    #[test]
    fn test_material_size() {
        assert_eq!(M2Material::size_in_bytes(M2Version::Classic), 4);
        assert_eq!(M2Material::size_in_bytes(M2Version::Cataclysm), 4);
        assert_eq!(M2Material::size_in_bytes(M2Version::WoD), 4);
    }
}
