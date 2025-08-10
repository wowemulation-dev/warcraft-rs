use crate::chunks::animation::M2AnimationTrackHeader;
use crate::version::M2Version;
use wow_data::prelude::*;
use wow_data::types::{C3Vector, Color, WowArray};
use wow_data_derive::{WowHeaderR, WowHeaderW};

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub enum M2RibbonEmitterRest {
    None,

    #[wow_data(read_if = version >= M2Version::WotLK)]
    Some {
        priority_plane: u16,
        ribbon_color_index: u8,
        texture_transform_lookup: u8,
    },
}

/// Represents a ribbon emitter in an M2 model
#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub struct M2RibbonEmitter {
    pub id: u32,
    pub bone_index: u32,
    pub position: C3Vector,
    pub texture_indices: WowArray<u16>,
    pub material_indices: WowArray<u16>,
    #[wow_data(versioned)]
    pub color_animation: M2AnimationTrackHeader<Color>,
    #[wow_data(versioned)]
    pub alpha_animation: M2AnimationTrackHeader<u16>,
    #[wow_data(versioned)]
    pub height_above_animation: M2AnimationTrackHeader<f32>,
    #[wow_data(versioned)]
    pub height_below_animation: M2AnimationTrackHeader<f32>,
    pub edges_per_second: f32,
    pub edge_lifetime: f32,
    pub gravity: f32,
    pub texture_rows: u16,
    pub texture_cols: u16,
    #[wow_data(versioned)]
    pub texture_slot_animation: M2AnimationTrackHeader<u16>,
    #[wow_data(versioned)]
    pub visibility_animation: M2AnimationTrackHeader<u8>,
    #[wow_data(versioned)]
    pub rest: M2RibbonEmitterRest,
}

impl M2RibbonEmitter {
    // /// Convert this ribbon emitter to a different version
    // pub fn convert(&self, target_version: M2Version) -> Self {
    //     let mut new_emitter = self.clone();
    //
    //     // Handle version-specific conversions
    //     if target_version >= M2Version::MoP && self.texture_slot_animation.is_none() {
    //         // When upgrading to MoP or later, add texture slice and variation if missing
    //         new_emitter.texture_slot_animation = Some(0);
    //         new_emitter.variation = Some(0);
    //     } else if target_version < M2Version::MoP {
    //         // When downgrading to pre-MoP, remove texture slice and variation
    //         new_emitter.texture_slot_animation = None;
    //         new_emitter.variation = None;
    //     }
    //
    //     new_emitter
    // }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::chunks::animation::M2AnimationTrackHeader;
//     use std::io::Cursor;
//
//     #[test]
//     fn test_ribbon_emitter_parse_write_classic() {
//         let ribbon = M2RibbonEmitter {
//             bone_index: 1,
//             position: C3Vector {
//                 x: 1.0,
//                 y: 2.0,
//                 z: 3.0,
//             },
//             texture_indices: WowArray::new(1, 0x100),
//             material_indices: WowArray::new(1, 0x200),
//             color_animation: M2AnimationTrackHeader::new(),
//             alpha_animation: M2AnimationTrackHeader::new(),
//             height_above_animation: M2AnimationTrackHeader::new(),
//             height_below_animation: M2AnimationTrackHeader::new(),
//             edges_per_second: 30.0,
//             edge_lifetime: 1.0,
//             gravity: 9.8,
//             texture_rows: 1,
//             texture_cols: 1,
//             texture_slot_animation: None,
//             variation: None,
//             id: 0,
//             flags: 0,
//         };
//
//         // Test write
//         let mut data = Vec::new();
//         ribbon
//             .write(&mut data, M2Version::Classic.to_header_version())
//             .unwrap();
//
//         // Test parse
//         let mut cursor = Cursor::new(data);
//         let parsed =
//             M2RibbonEmitter::parse(&mut cursor, M2Version::Classic.to_header_version()).unwrap();
//
//         assert_eq!(parsed.bone_index, 1);
//         assert_eq!(parsed.position.x, 1.0);
//         assert_eq!(parsed.position.y, 2.0);
//         assert_eq!(parsed.position.z, 3.0);
//         assert_eq!(parsed.texture_indices.count, 1);
//         assert_eq!(parsed.texture_indices.offset, 0x100);
//         assert_eq!(parsed.material_indices.count, 1);
//         assert_eq!(parsed.material_indices.offset, 0x200);
//         assert_eq!(parsed.edges_per_second, 30.0);
//         assert_eq!(parsed.edge_lifetime, 1.0);
//         assert_eq!(parsed.gravity, 9.8);
//         assert_eq!(parsed.texture_rows, 1);
//         assert_eq!(parsed.texture_cols, 1);
//         assert_eq!(parsed.texture_slot_animation, None);
//         assert_eq!(parsed.variation, None);
//         assert_eq!(parsed.id, 0);
//         assert_eq!(parsed.flags, 0);
//     }
//
//     #[test]
//     fn test_ribbon_emitter_parse_write_mop() {
//         let ribbon = M2RibbonEmitter {
//             bone_index: 1,
//             position: C3Vector {
//                 x: 1.0,
//                 y: 2.0,
//                 z: 3.0,
//             },
//             texture_indices: WowArray::new(1, 0x100),
//             material_indices: WowArray::new(1, 0x200),
//             color_animation: M2AnimationTrackHeader::new(),
//             alpha_animation: M2AnimationTrackHeader::new(),
//             height_above_animation: M2AnimationTrackHeader::new(),
//             height_below_animation: M2AnimationTrackHeader::new(),
//             edges_per_second: 30.0,
//             edge_lifetime: 1.0,
//             gravity: 9.8,
//             texture_rows: 1,
//             texture_cols: 1,
//             texture_slot_animation: Some(0),
//             variation: Some(0),
//             id: 0,
//             flags: 0,
//         };
//
//         // Test write
//         let mut data = Vec::new();
//         ribbon
//             .write(&mut data, M2Version::MoP.to_header_version())
//             .unwrap();
//
//         // Test parse
//         let mut cursor = Cursor::new(data);
//         let parsed =
//             M2RibbonEmitter::parse(&mut cursor, M2Version::MoP.to_header_version()).unwrap();
//
//         assert_eq!(parsed.bone_index, 1);
//         assert_eq!(parsed.position.x, 1.0);
//         assert_eq!(parsed.position.y, 2.0);
//         assert_eq!(parsed.position.z, 3.0);
//         assert_eq!(parsed.texture_indices.count, 1);
//         assert_eq!(parsed.texture_indices.offset, 0x100);
//         assert_eq!(parsed.material_indices.count, 1);
//         assert_eq!(parsed.material_indices.offset, 0x200);
//         assert_eq!(parsed.edges_per_second, 30.0);
//         assert_eq!(parsed.edge_lifetime, 1.0);
//         assert_eq!(parsed.gravity, 9.8);
//         assert_eq!(parsed.texture_rows, 1);
//         assert_eq!(parsed.texture_cols, 1);
//         assert_eq!(parsed.texture_slot_animation, Some(0));
//         assert_eq!(parsed.variation, Some(0));
//         assert_eq!(parsed.id, 0);
//         assert_eq!(parsed.flags, 0);
//     }
//
//     #[test]
//     fn test_ribbon_emitter_convert() {
//         // Create a Classic ribbon emitter
//         let classic_ribbon = M2RibbonEmitter {
//             bone_index: 1,
//             position: C3Vector {
//                 x: 1.0,
//                 y: 2.0,
//                 z: 3.0,
//             },
//             texture_indices: WowArray::new(1, 0x100),
//             material_indices: WowArray::new(1, 0x200),
//             color_animation: M2AnimationTrackHeader::new(),
//             alpha_animation: M2AnimationTrackHeader::new(),
//             height_above_animation: M2AnimationTrackHeader::new(),
//             height_below_animation: M2AnimationTrackHeader::new(),
//             edges_per_second: 30.0,
//             edge_lifetime: 1.0,
//             gravity: 9.8,
//             texture_rows: 1,
//             texture_cols: 1,
//             texture_slot_animation: None,
//             variation: None,
//             id: 0,
//             flags: 0,
//         };
//
//         // Convert to MoP
//         let mop_ribbon = classic_ribbon.convert(M2Version::MoP);
//
//         // Should have texture slice and variation
//         assert!(mop_ribbon.texture_slot_animation.is_some());
//         assert!(mop_ribbon.variation.is_some());
//
//         // Convert back to Classic
//         let classic_ribbon2 = mop_ribbon.convert(M2Version::Classic);
//
//         // Should not have texture slice and variation
//         assert!(classic_ribbon2.texture_slot_animation.is_none());
//         assert!(classic_ribbon2.variation.is_none());
//     }
// }
