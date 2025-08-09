use wow_data::prelude::*;
use wow_data_derive::{WowHeaderRV, WowHeaderW};

use crate::chunks::animation::M2AnimationBlock;
use crate::version::M2Version;

#[derive(Debug, Clone, WowHeaderRV, WowHeaderW)]
#[wow_data(version = M2Version)]
pub struct M2TransparencyAnimation {
    #[wow_data(versioned)]
    pub alpha: M2AnimationBlock<u16>,
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::io::Cursor;
//
//     #[test]
//     fn test_transparency_animation_parse_write() {
//         let mut data = Vec::new();
//
//         // Alpha animation track
//         data.extend_from_slice(&1u16.to_le_bytes()); // Interpolation type (Linear)
//         data.extend_from_slice(&(-1i16).to_le_bytes()); // Global sequence
//         data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps count
//         data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps offset
//         data.extend_from_slice(&0u32.to_le_bytes()); // Values count
//         data.extend_from_slice(&0u32.to_le_bytes()); // Values offset
//
//         let mut cursor = Cursor::new(data);
//         let trans_anim = M2TransparencyAnimation::parse(&mut cursor, 264).unwrap();
//
//         // Test write
//         let mut output = Vec::new();
//         trans_anim.write(&mut output).unwrap();
//
//         // Check output size (should be the same as input)
//         assert_eq!(output.len(), cursor.get_ref().len());
//     }
// }
