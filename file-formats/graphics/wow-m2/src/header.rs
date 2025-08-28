use std::io::SeekFrom;

use bitflags::bitflags;
use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{BoundingBox, C3Vector, MagicStr, WowArray, WowArrayV, WowCharArray};
use wow_data_derive::{WowHeaderR, WowHeaderW};

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
use crate::version::MD20Version;

pub const MD20_MAGIC: MagicStr = *b"MD20";

bitflags! {
    #[derive(Debug, Clone, Default, Copy, PartialEq, Eq, WowHeaderR, WowHeaderW)]
    #[wow_data(bitflags=u32)]
    pub struct M2ModelFlags: u32 {
        const TILT_X = 0x0001;
        const TILT_Y = 0x0002;
        const ADD_BACK_REFERENCE = 0x0004;
        const USE_TEXTURE_COMBINERS = 0x0008;
        const IS_CAMERA = 0x0010;
        const UNUSED = 0x0020;
        const NO_PARTICLE_TRAILS = 0x0040;
        const UNKNOWN_0x80 = 0x0080;
        const LOAD_PHYS_DATA = 0x0100;
        const UNKNOWN_0x200 = 0x0200;
        const HAS_BONES = 0x0400;
        const UNUSED_0x800 = 0x0800;
        const UNKNOWN_0x1000 = 0x1000;
        const USE_TEXTURE_IDS = 0x2000;
        const CAMERA_MODIFIABLE = 0x4000;
        const NEW_PARTICLE_SYSTEM = 0x8000;
        const UNKNOWN_0x10000 = 0x10000;
        const UNKNOWN_0x20000 = 0x20000;
        const UNKNOWN_0x40000 = 0x40000;
        const UNKNOWN_0x80000 = 0x80000;
        const UNKNOWN_0x100000 = 0x100000;
        const UNKNOWN_0x200000 = 0x200000;
        const UNKNOWN_0x400000 = 0x400000;
        const UNKNOWN_0x800000 = 0x800000;
        const UNKNOWN_0x1000000 = 0x1000000;
        const UNKNOWN_0x2000000 = 0x2000000;
        const UNKNOWN_0x4000000 = 0x4000000;
        const UNKNOWN_0x8000000 = 0x8000000;
        const UNKNOWN_0x10000000 = 0x10000000;
        const UNKNOWN_0x20000000 = 0x20000000;
        const UNKNOWN_0x40000000 = 0x40000000;
        const UNKNOWN_0x80000000 = 0x80000000;
    }
}

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub enum M2PlayableAnimationLookupHeader {
    #[wow_data(read_if = version <= MD20Version::TBCV4)]
    Some(WowArray<M2SequenceFallback>),

    #[default]
    None,
}

#[derive(Debug, Clone, Default)]
pub enum M2PlayableAnimationLookup {
    Some(Vec<M2SequenceFallback>),

    #[default]
    None,
}

impl VWowDataR<MD20Version, M2PlayableAnimationLookupHeader> for M2PlayableAnimationLookup {
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
#[wow_data(version = MD20Version)]
pub enum M2SkinProfilesHeader {
    UpToTBC(WowArray<M2SkinProfile>),

    #[wow_data(read_if = version >= MD20Version::WotLK)]
    Later(u32),
}

impl Default for M2SkinProfilesHeader {
    fn default() -> Self {
        Self::Later(4)
    }
}

#[derive(Debug, Clone, Default)]
pub enum M2SkinProfiles {
    Some(Vec<M2SkinProfile>),

    #[default]
    None,
}

impl VWowDataR<MD20Version, M2SkinProfilesHeader> for M2SkinProfiles {
    fn new_from_header<R: Read + Seek>(
        reader: &mut R,
        header: &M2SkinProfilesHeader,
    ) -> WDResult<Self> {
        Ok(match header {
            M2SkinProfilesHeader::UpToTBC(array) => Self::Some(array.wow_read_to_vec(reader)?),
            M2SkinProfilesHeader::Later(_) => Self::None,
        })
    }
}

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
pub struct M2TextureFlipbook {
    // 4 uints according to wowdev wiki
    a: u32,
    b: u32,
    c: u32,
    d: u32,
}

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub enum M2TextureFlipbooksHeader {
    Some(WowArray<M2TextureFlipbook>),

    #[default]
    #[wow_data(read_if = version >= MD20Version::WotLK)]
    None,
}

#[derive(Debug, Clone, Default)]
pub enum M2TextureFlipbooks {
    Some(Vec<M2TextureFlipbook>),

    #[default]
    None,
}

impl VWowDataR<MD20Version, M2TextureFlipbooksHeader> for M2TextureFlipbooks {
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
#[wow_data(version = MD20Version)]
pub enum M2BlenMapOverrides {
    None,

    #[wow_data(read_if = version >= MD20Version::TBCV1)]
    Some(WowArray<u16>),
}

#[derive(Debug, Clone, Default, WowHeaderW)]
pub enum M2TextureCombinerCombosHeader {
    #[default]
    None,

    Some(WowArray<u16>),
}

impl WowHeaderR for M2TextureCombinerCombosHeader {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        Ok(Self::Some(reader.wow_read()?))
    }
}

#[derive(Debug, Clone, Default)]
pub enum M2TextureCombinerCombos {
    Some(Vec<u16>),

    #[default]
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
#[wow_data(version = MD20Version)]
pub enum M2TextureTransforms {
    None,

    #[wow_data(read_if = version > MD20Version::BfAPlus)]
    Some(WowArray<u32>),
}

/// Based on: <https://wowdev.wiki/M2#Header>
#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
#[wow_data(version = MD20Version)]
pub struct MD20Header {
    pub version: MD20Version,
    pub name: WowCharArray,
    pub flags: M2ModelFlags,

    // Sequence-related fields
    pub global_sequences: WowArray<u32>,

    #[wow_data(versioned)]
    pub animations: WowArrayV<MD20Version, M2Animation>,

    /// Animation lookups (C in Vanilla)
    pub animation_lookup: WowArray<i16>,

    /// Playable animation lookup - only present in versions <= 263
    #[wow_data(versioned)]
    pub playable_animation_lookup: M2PlayableAnimationLookupHeader,

    // Bone-related fields
    #[wow_data(versioned)]
    pub bones: WowArrayV<MD20Version, M2BoneHeader>,

    pub key_bone_lookup: WowArray<i16>,

    // Geometry data
    pub vertices: WowArray<M2Vertex>,

    #[wow_data(versioned)]
    pub skin_profiles: M2SkinProfilesHeader,

    // Color data
    #[wow_data(versioned)]
    pub color_animations: WowArrayV<MD20Version, M2ColorAnimationHeader>,

    // Texture-related fields
    pub textures: WowArray<M2TextureHeader>,

    #[wow_data(versioned)]
    pub texture_weights: WowArrayV<MD20Version, M2TransparencyAnimationHeader>,

    #[wow_data(versioned)]
    pub texture_flipbooks: M2TextureFlipbooksHeader,

    #[wow_data(versioned)]
    pub texture_transforms: WowArrayV<MD20Version, M2TextureTransformHeader>,

    pub replaceable_texture_lookup: WowArray<i16>,

    pub materials: WowArray<M2Material>,
    pub bone_lookup_table: WowArray<i16>,
    pub texture_lookup_table: WowArray<i16>,
    pub texture_mapping_lookup_table: WowArray<i16>,
    pub transparency_lookup_table: WowArray<i16>,
    pub texture_animation_lookup: WowArray<i16>,

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
    pub attachments: WowArrayV<MD20Version, M2AttachmentHeader>,
    pub attachment_lookup_table: WowArray<i16>,
    #[wow_data(versioned)]
    pub events: WowArrayV<MD20Version, M2EventHeader>,
    #[wow_data(versioned)]
    pub lights: WowArrayV<MD20Version, M2LightHeader>,
    #[wow_data(versioned)]
    pub cameras: WowArrayV<MD20Version, M2CameraHeader>,
    pub camera_lookup_table: WowArray<i16>,

    // Particle systems
    #[wow_data(versioned)]
    pub ribbon_emitters: WowArrayV<MD20Version, M2RibbonEmitterHeader>,
    #[wow_data(versioned)]
    pub particle_emitters: WowArrayV<MD20Version, M2ParticleEmitterHeader>,

    #[wow_data(override_read = if flags.contains(M2ModelFlags::USE_TEXTURE_COMBINERS) {
        M2TextureCombinerCombosHeader::Some(reader.wow_read()?)
    } else {
        M2TextureCombinerCombosHeader::None
    } )]
    pub texture_combiner_combos: M2TextureCombinerCombosHeader,
}

impl WowHeaderR for MD20Header {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        let start_position = reader.stream_position()?;

        let version_val: u32 = reader.wow_read()?;
        let version = MD20Version::try_from(version_val)?;

        // rewind reader because the function below reads the version again
        reader.seek(SeekFrom::Start(start_position))?;
        reader.wow_read_versioned(version)
    }
}

impl MD20Header {
    /// Create a new M2 header for a specific version
    pub fn new(version: MD20Version) -> Self {
        Self {
            version,
            name: WowArray::default(),
            flags: M2ModelFlags::empty(),
            global_sequences: WowArray::default(),
            animations: WowArrayV::default(),
            animation_lookup: WowArray::default(),
            playable_animation_lookup: if version <= MD20Version::TBCV4 {
                M2PlayableAnimationLookupHeader::Some(WowArray::default())
            } else {
                M2PlayableAnimationLookupHeader::None
            },
            bones: WowArrayV::default(),
            key_bone_lookup: WowArray::default(),
            vertices: WowArray::default(),
            skin_profiles: if version >= MD20Version::WotLK {
                M2SkinProfilesHeader::Later(0)
            } else {
                M2SkinProfilesHeader::UpToTBC(WowArray::default())
            },
            color_animations: WowArrayV::default(),
            textures: WowArray::default(),
            texture_weights: WowArrayV::default(),
            texture_transforms: WowArrayV::default(),
            texture_flipbooks: if version <= MD20Version::TBCV4 {
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
            texture_combiner_combos: if version >= MD20Version::TBCV1 {
                M2TextureCombinerCombosHeader::Some(WowArray::default())
            } else {
                M2TextureCombinerCombosHeader::None
            },
        }
    }
}
