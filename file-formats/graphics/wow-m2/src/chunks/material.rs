use crate::M2Error;
use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data_derive::{WowHeaderR, WowHeaderW};

use crate::error::Result;

bitflags::bitflags! {
    /// Render flags as defined in the M2 format
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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

impl WowHeaderR for M2RenderFlags {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        Ok(Self::from_bits_retain(reader.wow_read()?))
    }
}
impl WowHeaderW for M2RenderFlags {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        writer.wow_write(&self.bits())?;
        Ok(())
    }
    fn wow_size(&self) -> usize {
        2
    }
}

bitflags::bitflags! {
    /// Blend modes as defined in the M2 format
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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

impl WowHeaderR for M2BlendMode {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        Ok(Self::from_bits_retain(reader.wow_read()?))
    }
}
impl WowHeaderW for M2BlendMode {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        writer.wow_write(&self.bits())?;
        Ok(())
    }
    fn wow_size(&self) -> usize {
        2
    }
}

/// Material texture transformations
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum M2TexTransformType {
    /// No texture transform
    #[default]
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

impl TryFrom<u16> for M2TexTransformType {
    type Error = M2Error;

    fn try_from(value: u16) -> Result<Self> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Scroll),
            2 => Ok(Self::Rotate),
            3 => Ok(Self::Scale),
            4 => Ok(Self::Stretch),
            5 => Ok(Self::Camera),
            _ => Err(M2Error::UnsupportedNumericVersion(value as u32)),
        }
    }
}

impl From<M2TexTransformType> for u16 {
    fn from(value: M2TexTransformType) -> Self {
        match value {
            M2TexTransformType::None => 0,
            M2TexTransformType::Scroll => 1,
            M2TexTransformType::Rotate => 2,
            M2TexTransformType::Scale => 3,
            M2TexTransformType::Stretch => 4,
            M2TexTransformType::Camera => 4,
        }
    }
}

impl WowHeaderR for M2TexTransformType {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        let value: u16 = reader.wow_read()?;
        Ok(value.try_into()?)
    }
}
impl WowHeaderW for M2TexTransformType {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        let value: u16 = (*self).into();
        writer.wow_write(&value)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        2
    }
}

/// Represents a material layer (render flags) in an M2 model
/// This corresponds to the ModelRenderFlagsM2 structure in WMVx
#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
pub struct M2Material {
    pub flags: M2RenderFlags,
    pub blend_mode: M2BlendMode,
}

impl M2Material {
    /// Create a new material with default values
    pub fn new(blend_mode: M2BlendMode) -> Self {
        Self {
            flags: M2RenderFlags::DEPTH_TEST | M2RenderFlags::DEPTH_WRITE,
            blend_mode,
        }
    }
}
