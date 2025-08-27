use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data_derive::{WowEnumFrom, WowHeaderR, WowHeaderW};

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
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, WowEnumFrom, WowHeaderR, WowHeaderW)]
#[wow_data(ty=u16)]
pub enum M2TexTransformType {
    /// No texture transform
    #[default]
    #[wow_data(lit = 0)]
    None = 0,
    /// Scroll texture
    #[wow_data(lit = 1)]
    Scroll = 1,
    /// Rotate texture
    #[wow_data(lit = 2)]
    Rotate = 2,
    /// Scale texture
    #[wow_data(lit = 3)]
    Scale = 3,
    /// Stretch texture based on time
    #[wow_data(lit = 4)]
    Stretch = 4,
    /// Transform texture based on camera
    #[wow_data(lit = 5)]
    Camera = 5,
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
