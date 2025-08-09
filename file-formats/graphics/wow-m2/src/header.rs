use crate::M2Error;
use crate::chunks::animation::{M2Animation, M2SequenceFallback};
use crate::chunks::bone::M2BoneHeader;
use crate::chunks::material::M2Material;
use crate::chunks::{
    M2Attachment, M2Camera, M2ColorAnimation, M2Event, M2Light, M2ParticleEmitter, M2RibbonEmitter,
    M2TextureHeader, M2TextureTransform, M2TransparencyAnimation, M2Vertex,
};
use bitflags::bitflags;
use std::io::{Read, Seek, SeekFrom, Write};
use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{BoundingBox, C3Vector, WowArray, WowArrayV, WowCharArray};
use wow_data_derive::{VWowHeaderR, WowHeaderR, WowHeaderW};

use crate::version::M2Version;

/// Magic signature for M2 files ("MD20")
pub const M2_MAGIC: [u8; 4] = *b"MD20";

bitflags! {
    /// Model flags as defined in the M2 format
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct M2ModelFlags: u32 {
        /// Tilt on X axis
        const TILT_X = 0x0001;
        /// Tilt on Y axis
        const TILT_Y = 0x0002;
        /// Add a back-reference to the model
        const ADD_BACK_REFERENCE = 0x0004;
        /// Use texture combiners
        const USE_TEXTURE_COMBINERS = 0x0008;
        /// Is it a camera?
        const IS_CAMERA = 0x0010;
        /// Unused flag
        const UNUSED = 0x0020;
        /// No particle trails
        const NO_PARTICLE_TRAILS = 0x0040;
        /// Unknown
        const UNKNOWN_0x80 = 0x0080;
        /// Load phys data
        const LOAD_PHYS_DATA = 0x0100;
        /// Unknown
        const UNKNOWN_0x200 = 0x0200;
        /// Has bones
        const HAS_BONES = 0x0400;
        /// Unused 0x800
        const UNUSED_0x800 = 0x0800;
        /// Unknown
        const UNKNOWN_0x1000 = 0x1000;
        /// Use texture IDs
        const USE_TEXTURE_IDS = 0x2000;
        /// Camera can be modified
        const CAMERA_MODIFIABLE = 0x4000;
        /// New particle system
        const NEW_PARTICLE_SYSTEM = 0x8000;
        /// Unknown
        const UNKNOWN_0x10000 = 0x10000;
        /// Unknown
        const UNKNOWN_0x20000 = 0x20000;
        /// Unknown
        const UNKNOWN_0x40000 = 0x40000;
        /// Unknown
        const UNKNOWN_0x80000 = 0x80000;
        /// Unknown
        const UNKNOWN_0x100000 = 0x100000;
        /// Unknown
        const UNKNOWN_0x200000 = 0x200000;
        /// Unknown
        const UNKNOWN_0x400000 = 0x400000;
        /// Unknown
        const UNKNOWN_0x800000 = 0x800000;
        /// Unknown
        const UNKNOWN_0x1000000 = 0x1000000;
        /// Unknown
        const UNKNOWN_0x2000000 = 0x2000000;
        /// Unknown
        const UNKNOWN_0x4000000 = 0x4000000;
        /// Unknown
        const UNKNOWN_0x8000000 = 0x8000000;
        /// Unknown
        const UNKNOWN_0x10000000 = 0x10000000;
        /// Unknown
        const UNKNOWN_0x20000000 = 0x20000000;
        /// Unknown
        const UNKNOWN_0x40000000 = 0x40000000;
        /// Unknown
        const UNKNOWN_0x80000000 = 0x80000000;
    }
}

impl WowHeaderR for M2ModelFlags {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        Ok(Self::from_bits_retain(reader.wow_read()?))
    }
}
impl WowHeaderW for M2ModelFlags {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        writer.wow_write(&self.bits())?;
        Ok(())
    }
    fn wow_size(&self) -> usize {
        4
    }
}

#[derive(Debug, Clone, VWowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub enum M2PlayableAnimationLookup {
    #[wow_data(read_if = version <= M2Version::TBC)]
    Some(WowArray<M2SequenceFallback>),
    None,
}

pub type M2SkinProfile = u32;

#[derive(Debug, Clone, VWowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub enum M2SkinProfiles {
    UpToTBC(WowArray<M2SkinProfile>),

    #[wow_data(read_if = version > M2Version::TBC)]
    Later(u32),
}

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
pub struct M2TextureFlipbook {
    // 4 uints according to wowdev wiki
    a: u32,
    b: u32,
    c: u32,
    d: u32,
}

#[derive(Debug, Clone, VWowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub enum M2TextureFlipbooks {
    Some(WowArray<M2TextureFlipbook>),

    #[wow_data(read_if = version > M2Version::TBC)]
    None,
}

#[derive(Debug, Clone, VWowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub enum M2BlenMapOverrides {
    None,

    #[wow_data(read_if = version > M2Version::Classic)]
    Some(WowArray<u16>),
}

#[derive(Debug, Clone, WowHeaderW)]
pub enum M2TextureCombinerCombos {
    None,
    Some(WowArray<u16>),
}

impl WowHeaderR for M2TextureCombinerCombos {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        Ok(Self::Some(reader.wow_read()?))
    }
}

#[derive(Debug, Clone, VWowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub enum M2TextureTransforms {
    None,

    #[wow_data(read_if = version > M2Version::Legion)]
    Some(WowArray<u32>),
}

/// M2 model header structure
/// Based on: <https://wowdev.wiki/M2#Header>
#[derive(Debug, Clone, VWowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub struct M2Header {
    /// Magic signature ("MD20")
    pub magic: [u8; 4],
    pub version: M2Version,
    pub name: WowCharArray,
    pub flags: M2ModelFlags,

    // Sequence-related fields
    pub global_sequences: WowArray<u32>,

    #[wow_data(versioned)]
    pub animations: WowArrayV<M2Version, M2Animation>,

    /// Animation lookups (C in Classic)
    pub animation_lookup: WowArray<u16>,

    /// Playable animation lookup - only present in versions <= 263
    #[wow_data(versioned)]
    pub playable_animation_lookup: M2PlayableAnimationLookup,

    // Bone-related fields
    #[wow_data(versioned)]
    pub bones: WowArrayV<M2Version, M2BoneHeader>,

    pub key_bone_lookup: WowArray<u16>,

    // Geometry data
    #[wow_data(versioned)]
    pub vertices: WowArrayV<M2Version, M2Vertex>,

    #[wow_data(versioned)]
    pub skin_profiles: M2SkinProfiles,

    // Color data
    #[wow_data(versioned)]
    pub color_animations: WowArrayV<M2Version, M2ColorAnimation>,

    // Texture-related fields
    pub textures: WowArray<M2TextureHeader>,

    #[wow_data(versioned)]
    pub texture_weights: WowArrayV<M2Version, M2TransparencyAnimation>,

    #[wow_data(versioned)]
    pub texture_flipbooks: M2TextureFlipbooks,

    #[wow_data(versioned)]
    pub texture_transforms: WowArrayV<M2Version, M2TextureTransform>,

    pub replaceable_texture_lookup: WowArray<u16>,

    pub materials: WowArray<M2Material>,
    pub bone_lookup_table: WowArray<u16>,
    pub texture_lookup_table: WowArray<u16>,
    pub texture_mapping_lookup_table: WowArray<u16>,
    pub transparency_lookup_table: WowArray<u16>,
    pub texture_animation_lookup: WowArray<u16>,

    pub bounding_box: BoundingBox,
    pub bounding_sphere_radius: f32,
    pub collision_box: BoundingBox,
    pub collision_sphere_radius: f32,

    // Additional geometry data
    pub bounding_triangles: WowArray<u16>,
    pub bounding_vertices: WowArray<C3Vector>,
    pub bounding_normals: WowArray<C3Vector>,

    // Attachments and events
    #[wow_data(versioned)]
    pub attachments: WowArrayV<M2Version, M2Attachment>,
    pub attachment_lookup_table: WowArray<u16>,
    #[wow_data(versioned)]
    pub events: WowArrayV<M2Version, M2Event>,
    #[wow_data(versioned)]
    pub lights: WowArrayV<M2Version, M2Light>,
    #[wow_data(versioned)]
    pub cameras: WowArrayV<M2Version, M2Camera>,
    pub camera_lookup_table: WowArray<u16>,

    // Particle systems
    #[wow_data(versioned)]
    pub ribbon_emitters: WowArrayV<M2Version, M2RibbonEmitter>,
    #[wow_data(versioned)]
    pub particle_emitters: WowArrayV<M2Version, M2ParticleEmitter>,

    pub texture_combiner_combos: M2TextureCombinerCombos,
}

impl WowHeaderR for M2Header {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        let start_position = reader.stream_position()?;

        let magic: [u8; 4] = reader.wow_read()?;
        let magic_str = String::from_utf8_lossy(&magic);
        let m2_magic_str = String::from_utf8_lossy(&M2_MAGIC);

        if magic != M2_MAGIC {
            return Err(M2Error::InvalidMagic {
                expected: m2_magic_str.into(),
                actual: magic_str.into(),
            }
            .into());
        }
        let version = M2Version::from_header_version(reader.wow_read()?)?;

        // rewind reader because the function below reads magic and version again
        reader.seek(SeekFrom::Start(start_position))?;
        Ok(reader.wow_read_versioned(version)?)
    }
}

impl M2Header {
    /// Create a new M2 header for a specific version
    pub fn new(version: M2Version) -> Self {
        Self {
            magic: M2_MAGIC.into(),
            version,
            name: WowArray::default(),
            flags: M2ModelFlags::empty(),
            global_sequences: WowArray::default(),
            animations: WowArrayV::default(),
            animation_lookup: WowArray::default(),
            playable_animation_lookup: if version <= M2Version::TBC {
                M2PlayableAnimationLookup::Some(WowArray::default())
            } else {
                M2PlayableAnimationLookup::None
            },
            bones: WowArrayV::default(),
            key_bone_lookup: WowArray::default(),
            vertices: WowArrayV::default(),
            skin_profiles: if version > M2Version::TBC {
                M2SkinProfiles::Later(0)
            } else {
                M2SkinProfiles::UpToTBC(WowArray::default())
            },
            color_animations: WowArrayV::default(),
            textures: WowArray::default(),
            texture_weights: WowArrayV::default(),
            texture_transforms: WowArrayV::default(),
            texture_flipbooks: if version <= M2Version::TBC {
                M2TextureFlipbooks::Some(WowArray::default())
            } else {
                M2TextureFlipbooks::None
            },
            materials: WowArray::default(),
            bone_lookup_table: WowArray::default(),
            replaceable_texture_lookup: WowArray::default(),
            texture_lookup_table: WowArray::default(),
            texture_mapping_lookup_table: WowArray::default(),
            transparency_lookup_table: WowArray::default(),
            texture_animation_lookup: WowArray::default(),
            bounding_box: BoundingBox::default(),
            bounding_sphere_radius: 0.0,
            collision_box: BoundingBox::default(),
            collision_sphere_radius: 0.0,
            bounding_triangles: WowArray::default(),
            bounding_vertices: WowArray::default(),
            bounding_normals: WowArray::default(),
            attachments: WowArrayV::default(),
            attachment_lookup_table: WowArray::default(),
            events: WowArrayV::default(),
            lights: WowArrayV::default(),
            cameras: WowArrayV::default(),
            camera_lookup_table: WowArray::default(),
            ribbon_emitters: WowArrayV::default(),
            particle_emitters: WowArrayV::default(),
            texture_combiner_combos: if version >= M2Version::TBC {
                M2TextureCombinerCombos::Some(WowArray::default())
            } else {
                M2TextureCombinerCombos::None
            },
        }
    }

    ///// Convert this header to a different version
    // pub fn convert(&self, target_version: M2Version) -> Result<Self> {
    //     let source_version = self.version().ok_or(M2Error::ConversionError {
    //         from: self.version,
    //         to: target_version.to_header_version(),
    //         reason: "Unknown source version".to_string(),
    //     })?;
    //
    //     if source_version == target_version {
    //         return Ok(self.clone());
    //     }
    //
    //     let mut new_header = self.clone();
    //     new_header.version = target_version.to_header_version();
    //
    //     // Handle version-specific fields
    //     if target_version >= M2Version::Cataclysm && source_version < M2Version::Cataclysm {
    //         // Add texture_combiner_combos when upgrading to Cataclysm or later
    //         new_header.texture_combiner_combos = Some(WowArray::default());
    //     } else if target_version < M2Version::Cataclysm && source_version >= M2Version::Cataclysm {
    //         // Remove texture_combiner_combos when downgrading to pre-Cataclysm
    //         new_header.texture_combiner_combos = None;
    //     }
    //
    //     if target_version >= M2Version::Legion && source_version < M2Version::Legion {
    //         // Add texture_transforms when upgrading to Legion or later
    //         new_header.texture_transforms = Some(WowArray::default());
    //     } else if target_version < M2Version::Legion && source_version >= M2Version::Legion {
    //         // Remove texture_transforms when downgrading to pre-Legion
    //         new_header.texture_transforms = None;
    //     }
    //
    //     Ok(new_header)
    // }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::io::Cursor;
//
//     // Helper function to create a basic test header
//     fn create_test_header(version: M2Version) -> Vec<u8> {
//         let mut data = Vec::new();
//
//         // Magic "MD20"
//         data.extend_from_slice(&M2_MAGIC);
//
//         // Version
//         data.extend_from_slice(&version.to_header_version().to_le_bytes());
//
//         // Name
//         data.extend_from_slice(&0u32.to_le_bytes()); // count = 0
//         data.extend_from_slice(&0u32.to_le_bytes()); // offset = 0
//
//         // Flags
//         data.extend_from_slice(&0u32.to_le_bytes());
//
//         // Global sequences
//         data.extend_from_slice(&0u32.to_le_bytes()); // count = 0
//         data.extend_from_slice(&0u32.to_le_bytes()); // offset = 0
//
//         // ... continue for all required fields
//         // This is simplified for brevity - in a real test, we'd populate all fields
//
//         // For brevity, let's just add enough bytes to cover the base header
//         for _ in 0..100 {
//             data.extend_from_slice(&0u32.to_le_bytes());
//         }
//
//         data
//     }
//
//     #[test]
//     fn test_header_parse_classic() {
//         let data = create_test_header(M2Version::Classic);
//         let mut cursor = Cursor::new(data);
//
//         let header = M2Header::parse(&mut cursor).unwrap();
//
//         assert_eq!(header.magic, M2_MAGIC);
//         assert_eq!(header.version, M2Version::Classic.to_header_version());
//         assert_eq!(header.texture_combiner_combos, None);
//         assert_eq!(header.texture_transforms, None);
//     }
//
//     #[test]
//     fn test_header_parse_cataclysm() {
//         let data = create_test_header(M2Version::Cataclysm);
//         let mut cursor = Cursor::new(data);
//
//         let header = M2Header::parse(&mut cursor).unwrap();
//
//         assert_eq!(header.magic, M2_MAGIC);
//         assert_eq!(header.version, M2Version::Cataclysm.to_header_version());
//         assert!(header.texture_combiner_combos.is_some());
//         assert_eq!(header.texture_transforms, None);
//     }
//
//     #[test]
//     fn test_header_parse_legion() {
//         let data = create_test_header(M2Version::Legion);
//         let mut cursor = Cursor::new(data);
//
//         let header = M2Header::parse(&mut cursor).unwrap();
//
//         assert_eq!(header.magic, M2_MAGIC);
//         assert_eq!(header.version, M2Version::Legion.to_header_version());
//         assert!(header.texture_combiner_combos.is_some());
//         assert!(header.texture_transforms.is_some());
//     }
//
//     #[test]
//     fn test_header_conversion() {
//         let classic_header = M2Header::new(M2Version::Classic);
//
//         // Convert Classic to Cataclysm
//         let cataclysm_header = classic_header.convert(M2Version::Cataclysm).unwrap();
//         assert_eq!(
//             cataclysm_header.version,
//             M2Version::Cataclysm.to_header_version()
//         );
//         assert!(cataclysm_header.texture_combiner_combos.is_some());
//         assert_eq!(cataclysm_header.texture_transforms, None);
//
//         // Convert Cataclysm to Legion
//         let legion_header = cataclysm_header.convert(M2Version::Legion).unwrap();
//         assert_eq!(legion_header.version, M2Version::Legion.to_header_version());
//         assert!(legion_header.texture_combiner_combos.is_some());
//         assert!(legion_header.texture_transforms.is_some());
//
//         // Convert Legion back to Classic
//         let classic_header_2 = legion_header.convert(M2Version::Classic).unwrap();
//         assert_eq!(
//             classic_header_2.version,
//             M2Version::Classic.to_header_version()
//         );
//         assert_eq!(classic_header_2.texture_combiner_combos, None);
//         assert_eq!(classic_header_2.texture_transforms, None);
//     }
// }
