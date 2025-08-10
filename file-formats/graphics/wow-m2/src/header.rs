use crate::M2Error;
use crate::chunks::M2Vertex;
use crate::chunks::animation::{M2Animation, M2SequenceFallback};
use crate::chunks::attachment::M2AttachmentHeader;
use crate::chunks::bone::M2BoneHeader;
use crate::chunks::camera::M2CameraHeader;
use crate::chunks::color_animation::M2ColorAnimationHeader;
use crate::chunks::event::M2EventHeader;
use crate::chunks::light::M2LightHeader;
use crate::chunks::material::M2Material;
use crate::chunks::particle_emitter::M2ParticleEmitterHeader;
use crate::chunks::ribbon_emitter::M2RibbonEmitterHeader;
use crate::chunks::texture::M2TextureHeader;
use crate::chunks::texture_transform::M2TextureTransformHeader;
use crate::chunks::transparency_animation::M2TransparencyAnimationHeader;
use bitflags::bitflags;
use std::io::{Read, Seek, SeekFrom, Write};
use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{BoundingBox, C3Vector, WowArray, WowArrayV, WowCharArray};
use wow_data_derive::{WowHeaderR, WowHeaderW};

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

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub enum M2PlayableAnimationLookupHeader {
    #[wow_data(read_if = version <= M2Version::TBC)]
    Some(WowArray<M2SequenceFallback>),
    None,
}

#[derive(Debug, Clone)]
pub enum M2PlayableAnimationLookup {
    Some(Vec<M2SequenceFallback>),
    None,
}

impl VWowDataR<M2Version, M2PlayableAnimationLookupHeader> for M2PlayableAnimationLookup {
    fn new_from_header<R: Read + Seek>(
        reader: &mut R,
        header: &M2PlayableAnimationLookupHeader,
    ) -> WDResult<Self> {
        Ok(match header {
            M2PlayableAnimationLookupHeader::Some(array) => {
                Self::Some(array.wow_read_to_vec(reader)?)
            }
            M2PlayableAnimationLookupHeader::None => Self::None,
        })
    }
}

pub type M2SkinProfile = u32;

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
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

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub enum M2TextureFlipbooksHeader {
    Some(WowArray<M2TextureFlipbook>),

    #[wow_data(read_if = version > M2Version::TBC)]
    None,
}

#[derive(Debug, Clone)]
pub enum M2TextureFlipbooks {
    Some(Vec<M2TextureFlipbook>),
    None,
}

impl VWowDataR<M2Version, M2TextureFlipbooksHeader> for M2TextureFlipbooks {
    fn new_from_header<R: Read + Seek>(
        reader: &mut R,
        header: &M2TextureFlipbooksHeader,
    ) -> WDResult<Self> {
        Ok(match header {
            M2TextureFlipbooksHeader::Some(array) => Self::Some(array.wow_read_to_vec(reader)?),
            M2TextureFlipbooksHeader::None => Self::None,
        })
    }
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub enum M2BlenMapOverrides {
    None,

    #[wow_data(read_if = version > M2Version::Classic)]
    Some(WowArray<u16>),
}

#[derive(Debug, Clone, WowHeaderW)]
pub enum M2TextureCombinerCombosHeader {
    None,
    Some(WowArray<u16>),
}

impl WowHeaderR for M2TextureCombinerCombosHeader {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        Ok(Self::Some(reader.wow_read()?))
    }
}

#[derive(Debug, Clone)]
pub enum M2TextureCombinerCombos {
    Some(Vec<u16>),
    None,
}

impl WowDataR<M2TextureCombinerCombosHeader> for M2TextureCombinerCombos {
    fn new_from_header<R: Read + Seek>(
        reader: &mut R,
        header: &M2TextureCombinerCombosHeader,
    ) -> WDResult<Self> {
        Ok(match header {
            M2TextureCombinerCombosHeader::Some(array) => {
                Self::Some(reader.new_from_header(array)?)
            }
            M2TextureCombinerCombosHeader::None => Self::None,
        })
    }
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub enum M2TextureTransforms {
    None,

    #[wow_data(read_if = version > M2Version::Legion)]
    Some(WowArray<u32>),
}

/// M2 model header structure
/// Based on: <https://wowdev.wiki/M2#Header>
#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
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
    pub playable_animation_lookup: M2PlayableAnimationLookupHeader,

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
    pub color_animations: WowArrayV<M2Version, M2ColorAnimationHeader>,

    // Texture-related fields
    pub textures: WowArray<M2TextureHeader>,

    #[wow_data(versioned)]
    pub texture_weights: WowArrayV<M2Version, M2TransparencyAnimationHeader>,

    #[wow_data(versioned)]
    pub texture_flipbooks: M2TextureFlipbooksHeader,

    #[wow_data(versioned)]
    pub texture_transforms: WowArrayV<M2Version, M2TextureTransformHeader>,

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
    pub attachments: WowArrayV<M2Version, M2AttachmentHeader>,
    pub attachment_lookup_table: WowArray<u16>,
    #[wow_data(versioned)]
    pub events: WowArrayV<M2Version, M2EventHeader>,
    #[wow_data(versioned)]
    pub lights: WowArrayV<M2Version, M2LightHeader>,
    #[wow_data(versioned)]
    pub cameras: WowArrayV<M2Version, M2CameraHeader>,
    pub camera_lookup_table: WowArray<u16>,

    // Particle systems
    #[wow_data(versioned)]
    pub ribbon_emitters: WowArrayV<M2Version, M2RibbonEmitterHeader>,
    #[wow_data(versioned)]
    pub particle_emitters: WowArrayV<M2Version, M2ParticleEmitterHeader>,

    pub texture_combiner_combos: M2TextureCombinerCombosHeader,
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
                M2PlayableAnimationLookupHeader::Some(WowArray::default())
            } else {
                M2PlayableAnimationLookupHeader::None
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
                M2TextureFlipbooksHeader::Some(WowArray::default())
            } else {
                M2TextureFlipbooksHeader::None
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
                M2TextureCombinerCombosHeader::Some(WowArray::default())
            } else {
                M2TextureCombinerCombosHeader::None
            },
        }
    }
}
