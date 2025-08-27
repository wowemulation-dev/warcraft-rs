use wow_data::prelude::*;
use wow_data::types::WowCharArray;
use wow_data::{error::Result as WDResult, types::WowString};
use wow_data_derive::{WowEnumFrom, WowHeaderR, WowHeaderW};

/// Texture type enum as defined in the M2 format
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, WowEnumFrom, WowHeaderR, WowHeaderW)]
#[wow_data(ty=u32)]
pub enum M2TextureType {
    /// Texture defined in filename
    #[default]
    #[wow_data(lit = 0)]
    Hardcoded = 0,
    /// Body + clothes
    #[wow_data(lit = 1)]
    Body = 1,
    /// Item, capes
    #[wow_data(lit = 2)]
    Item = 2,
    /// Weapon blade
    #[wow_data(lit = 3)]
    WeaponBlade = 3,
    /// Weapon handle
    #[wow_data(lit = 4)]
    WeaponHandle = 4,
    /// Environment
    #[wow_data(lit = 5)]
    Environment = 5,
    /// Hair, beard
    #[wow_data(lit = 6)]
    Hair = 6,
    #[wow_data(lit = 7)]
    FacialHair = 7,
    #[wow_data(lit = 8)]
    SkinExtra = 8,
    #[wow_data(lit = 9)]
    UISkin = 9,
    #[wow_data(lit = 10)]
    TaurenMane = 10,
    #[wow_data(lit = 11)]
    Monster1 = 11,
    #[wow_data(lit = 12)]
    Monster2 = 12,
    #[wow_data(lit = 13)]
    Monster3 = 13,
    #[wow_data(lit = 14)]
    ItemIcon = 14,
    #[wow_data(lit = 15)]
    GuildBgColor = 15,
    #[wow_data(lit = 16)]
    GuildEmblemColor = 16,
    #[wow_data(lit = 17)]
    GuildBorderColor = 17,
    #[wow_data(lit = 18)]
    GuildEmblem = 18,
    #[wow_data(lit = 19)]
    CharacterEyes = 19,
    #[wow_data(lit = 20)]
    CharacterAccessory = 20,
    #[wow_data(lit = 21)]
    CharacterSecondarySkin = 21,
    #[wow_data(lit = 22)]
    CharacterSecondaryHair = 22,
    #[wow_data(lit = 23)]
    CharacterSecondaryArmor = 23,
    #[wow_data(lit = 24)]
    Unknown1 = 24,
    #[wow_data(lit = 25)]
    Unknown2 = 25,
    #[wow_data(lit = 26)]
    Unknown3 = 26,
}

bitflags::bitflags! {
    /// Texture flags as defined in the M2 format
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, WowHeaderR, WowHeaderW)]
    #[wow_data(bitflags=u32)]
    pub struct M2TextureFlags: u32 {
        /// Texture is wrapped horizontally
        const WRAP_X = 0x01;
        /// Texture is wrapped vertically
        const WRAP_Y = 0x02;
        /// Texture will not be replaced by other textures
        /// (character customization texture replacement)
        const NOT_REPLACEABLE = 0x04;
    }
}

/// Represents a texture in an M2 model
#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
pub struct M2TextureHeader {
    /// Type of the texture
    pub texture_type: M2TextureType,
    /// Flags for this texture
    pub flags: M2TextureFlags,
    /// Filename of the texture
    pub filename: WowCharArray,
}

impl M2TextureHeader {
    /// Create a new texture with the given type and filename
    pub fn new(texture_type: M2TextureType, filename: WowCharArray) -> Self {
        Self {
            texture_type,
            flags: M2TextureFlags::empty(),
            filename,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct M2TextureData {
    pub filename: String,
}

impl WowDataR<M2TextureHeader> for M2TextureData {
    fn new_from_header<R: Read + Seek>(reader: &mut R, header: &M2TextureHeader) -> WDResult<Self> {
        Ok(Self {
            filename: String::from_wow_char_array(reader, header.filename.clone())?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct M2Texture {
    pub header: M2TextureHeader,
    pub data: M2TextureData,
}
