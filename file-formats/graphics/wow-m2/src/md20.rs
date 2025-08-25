use wow_data::error::Result as WDResult;
use wow_data::{prelude::*, v_wow_collection, wow_collection};

use custom_debug::Debug;
use wow_data::types::{C3Vector, WowStructR, WowStructW};
use wow_utils::debug;

use crate::chunks::animation::M2Animation;
use crate::chunks::bone::M2Bone;
use crate::chunks::color_animation::M2ColorAnimation;
use crate::chunks::material::M2Material;
use crate::chunks::texture::M2Texture;
use crate::chunks::texture_transform::M2TextureTransform;
use crate::chunks::{
    M2Attachment, M2Camera, M2Event, M2Light, M2ParticleEmitter, M2RibbonEmitter,
    M2TransparencyAnimation, M2Vertex,
};
use crate::header::{
    M2PlayableAnimationLookup, M2SkinProfiles, M2TextureCombinerCombos, M2TextureFlipbooks,
    MD20Header,
};

/// Main M2 model structure
#[derive(Debug, Clone, Default)]
pub struct MD20Model {
    pub header: MD20Header,
    pub name: String,
    pub global_sequences: Vec<u32>,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub animations: Vec<M2Animation>,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub animation_lookup: Vec<i16>,
    pub playable_animation_lookup: M2PlayableAnimationLookup,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub bones: Vec<M2Bone>,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub key_bone_lookup: Vec<i16>,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub vertices: Vec<M2Vertex>,

    pub skin_profiles: M2SkinProfiles,

    #[debug(with = debug::trimmed_collection_fmt)]
    pub color_animations: Vec<M2ColorAnimation>,

    #[debug(with = debug::trimmed_collection_fmt)]
    pub textures: Vec<M2Texture>,

    #[debug(with = debug::trimmed_collection_fmt)]
    pub texture_weights: Vec<M2TransparencyAnimation>,

    pub texture_flipbooks: M2TextureFlipbooks,

    #[debug(with = debug::trimmed_collection_fmt)]
    pub texture_transforms: Vec<M2TextureTransform>,

    #[debug(with = debug::trimmed_collection_fmt)]
    pub replaceable_texture_lookup: Vec<i16>,

    #[debug(with = debug::trimmed_collection_fmt)]
    pub materials: Vec<M2Material>,

    #[debug(with = debug::trimmed_collection_fmt)]
    pub bone_lookup_table: Vec<i16>,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub texture_lookup_table: Vec<i16>,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub texture_mapping_lookup_table: Vec<i16>,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub transparency_lookup_table: Vec<i16>,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub texture_animation_lookup: Vec<i16>,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub bounding_triangles: Vec<u16>,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub bounding_vertices: Vec<C3Vector>,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub bounding_normals: Vec<C3Vector>,

    #[debug(with = debug::trimmed_collection_fmt)]
    pub attachments: Vec<M2Attachment>,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub attachment_lookup_table: Vec<i16>,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub events: Vec<M2Event>,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub lights: Vec<M2Light>,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub cameras: Vec<M2Camera>,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub camera_lookup_table: Vec<i16>,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub ribbon_emitters: Vec<M2RibbonEmitter>,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub particle_emitters: Vec<M2ParticleEmitter>,
    pub texture_combiner_combos: M2TextureCombinerCombos,
}

impl WowStructR for MD20Model {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        let header: MD20Header = reader.wow_read()?;

        let color_animations = v_wow_collection!(
            reader,
            header.version,
            header.color_animations,
            |reader, item_header| {
                M2ColorAnimation {
                    data: reader.v_new_from_header(&item_header)?,
                    header: item_header,
                }
            }
        );

        let textures = wow_collection!(reader, header.textures, |reader, item_header| {
            M2Texture {
                data: reader.new_from_header(&item_header)?,
                header: item_header,
            }
        });

        let texture_weights = v_wow_collection!(
            reader,
            header.version,
            header.texture_weights,
            |reader, item_header| {
                M2TransparencyAnimation {
                    data: reader.v_new_from_header(&item_header)?,
                    header: item_header,
                }
            }
        );

        let texture_transforms = v_wow_collection!(
            reader,
            header.version,
            header.texture_transforms,
            |reader, item_header| {
                M2TextureTransform {
                    data: reader.v_new_from_header(&item_header)?,
                    header: item_header,
                }
            }
        );

        let attachments = v_wow_collection!(
            reader,
            header.version,
            header.attachments,
            |reader, item_header| {
                M2Attachment {
                    data: reader.v_new_from_header(&item_header)?,
                    header: item_header,
                }
            }
        );

        let events = v_wow_collection!(
            reader,
            header.version,
            header.events,
            |reader, item_header| {
                M2Event {
                    data: reader.v_new_from_header(&item_header)?,
                    header: item_header,
                }
            }
        );

        let lights = v_wow_collection!(
            reader,
            header.version,
            header.lights,
            |reader, item_header| {
                M2Light {
                    data: reader.v_new_from_header(&item_header)?,
                    header: item_header,
                }
            }
        );

        let cameras = v_wow_collection!(
            reader,
            header.version,
            header.cameras,
            |reader, item_header| {
                M2Camera {
                    data: reader.v_new_from_header(&item_header)?,
                    header: item_header,
                }
            }
        );

        let ribbon_emitters = v_wow_collection!(
            reader,
            header.version,
            header.ribbon_emitters,
            |reader, item_header| {
                M2RibbonEmitter {
                    data: reader.v_new_from_header(&item_header)?,
                    header: item_header,
                }
            }
        );

        let particle_emitters = v_wow_collection!(
            reader,
            header.version,
            header.particle_emitters,
            |reader, item_header| reader.v_new_from_header(&item_header)?
        );

        Ok(Self {
            name: reader.new_from_header(&header.name)?,
            global_sequences: reader.new_from_header(&header.global_sequences)?,
            animations: header.animations.wow_read_to_vec(reader, header.version)?,
            animation_lookup: reader.new_from_header(&header.animation_lookup)?,
            playable_animation_lookup: reader
                .v_new_from_header(&header.playable_animation_lookup)?,
            bones: M2Bone::read_bone_array(reader, header.bones.clone(), header.version)?,
            key_bone_lookup: reader.new_from_header(&header.key_bone_lookup)?,
            vertices: header.vertices.wow_read_to_vec(reader)?,
            skin_profiles: reader.v_new_from_header(&header.skin_profiles)?,
            color_animations,
            textures,
            texture_weights,
            texture_flipbooks: reader.v_new_from_header(&header.texture_flipbooks)?,
            texture_transforms,
            replaceable_texture_lookup: header
                .replaceable_texture_lookup
                .wow_read_to_vec(reader)?,
            materials: reader.new_from_header(&header.materials)?,
            bone_lookup_table: reader.new_from_header(&header.bone_lookup_table)?,
            texture_lookup_table: reader.new_from_header(&header.texture_lookup_table)?,
            texture_mapping_lookup_table: header
                .texture_mapping_lookup_table
                .wow_read_to_vec(reader)?,
            transparency_lookup_table: reader.new_from_header(&header.transparency_lookup_table)?,
            texture_animation_lookup: reader.new_from_header(&header.texture_animation_lookup)?,
            bounding_triangles: reader.new_from_header(&header.bounding_triangles)?,
            bounding_vertices: reader.new_from_header(&header.bounding_vertices)?,
            bounding_normals: reader.new_from_header(&header.bounding_normals)?,
            attachments,
            attachment_lookup_table: reader.new_from_header(&header.attachment_lookup_table)?,
            events,
            lights,
            cameras,
            ribbon_emitters,
            camera_lookup_table: reader.new_from_header(&header.camera_lookup_table)?,
            particle_emitters,
            texture_combiner_combos: reader.new_from_header(&header.texture_combiner_combos)?,
            header,
        })
    }
}

impl WowStructW for MD20Model {
    fn wow_write<W: Write + Seek>(&self, _writer: &mut W) -> WDResult<()> {
        todo!()
    }
}
