use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Write};

use crate::chunks::animation::M2AnimationBlock;
use crate::chunks::color_animation::M2Color;
use crate::common::{C3Vector, M2Array};
use crate::error::Result;
use crate::version::M2Version;

/// Represents a ribbon emitter in an M2 model
#[derive(Debug, Clone)]
pub struct M2RibbonEmitter {
    /// Bone ID that this emitter is attached to
    pub bone_index: u32,
    /// Position of the emitter relative to the bone
    pub position: C3Vector,
    /// Texture indices used by this ribbon
    pub texture_indices: M2Array<u16>,
    /// Material indices used by this ribbon
    pub material_indices: M2Array<u16>,
    /// Color animation data
    pub color_animation: M2AnimationBlock<M2Color>,
    /// Alpha animation data
    pub alpha_animation: M2AnimationBlock<f32>,
    /// Height above animation data
    pub height_above_animation: M2AnimationBlock<f32>,
    /// Height below animation data
    pub height_below_animation: M2AnimationBlock<f32>,
    /// Edges per second
    pub edges_per_second: f32,
    /// Edge lifetime in seconds
    pub edge_lifetime: f32,
    /// Gravity applied to the ribbon
    pub gravity: f32,
    /// Number of texture tiles
    pub texture_rows: u16,
    /// Number of texture columns
    pub texture_cols: u16,
    /// Texture slice (added in MoP)
    pub texture_slice: Option<u16>,
    /// Variation of the ribbon (added in MoP)
    pub variation: Option<u16>,
    /// Ribbon ID
    pub id: u32,
    /// Ribbon flags
    pub flags: u32,
}

impl M2RibbonEmitter {
    /// Parse a ribbon emitter from a reader based on the M2 version
    pub fn parse<R: Read>(reader: &mut R, version: u32) -> Result<Self> {
        let bone_index = reader.read_u32_le()?;
        let position = C3Vector::parse(reader)?;
        let texture_indices = M2Array::parse(reader)?;
        let material_indices = M2Array::parse(reader)?;

        let color_animation = M2AnimationBlock::parse(reader, version)?;
        let alpha_animation = M2AnimationBlock::parse(reader, version)?;
        let height_above_animation = M2AnimationBlock::parse(reader, version)?;
        let height_below_animation = M2AnimationBlock::parse(reader, version)?;

        let edges_per_second = reader.read_f32_le()?;
        let edge_lifetime = reader.read_f32_le()?;
        let gravity = reader.read_f32_le()?;

        let texture_rows = reader.read_u16_le()?;
        let texture_cols = reader.read_u16_le()?;

        // Version-specific fields
        let (texture_slice, variation) =
            if let Some(m2_version) = M2Version::try_from_header_version(version) {
                if m2_version >= M2Version::MoP {
                    let slice = reader.read_u16_le()?;
                    let var = reader.read_u16_le()?;
                    (Some(slice), Some(var))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

        let id = reader.read_u32_le()?;
        let flags = reader.read_u32_le()?;

        Ok(Self {
            bone_index,
            position,
            texture_indices,
            material_indices,
            color_animation,
            alpha_animation,
            height_above_animation,
            height_below_animation,
            edges_per_second,
            edge_lifetime,
            gravity,
            texture_rows,
            texture_cols,
            texture_slice,
            variation,
            id,
            flags,
        })
    }

    /// Write a ribbon emitter to a writer based on the M2 version
    pub fn write<W: Write>(&self, writer: &mut W, version: u32) -> Result<()> {
        writer.write_u32_le(self.bone_index)?;
        self.position.write(writer)?;
        self.texture_indices.write(writer)?;
        self.material_indices.write(writer)?;

        self.color_animation.write(writer)?;
        self.alpha_animation.write(writer)?;
        self.height_above_animation.write(writer)?;
        self.height_below_animation.write(writer)?;

        writer.write_f32_le(self.edges_per_second)?;
        writer.write_f32_le(self.edge_lifetime)?;
        writer.write_f32_le(self.gravity)?;

        writer.write_u16_le(self.texture_rows)?;
        writer.write_u16_le(self.texture_cols)?;

        // Version-specific fields
        if let Some(m2_version) = M2Version::try_from_header_version(version) {
            if m2_version >= M2Version::MoP {
                writer.write_u16_le(self.texture_slice.unwrap_or(0))?;
                writer.write_u16_le(self.variation.unwrap_or(0))?;
            }
        }

        writer.write_u32_le(self.id)?;
        writer.write_u32_le(self.flags)?;

        Ok(())
    }

    /// Convert this ribbon emitter to a different version
    pub fn convert(&self, target_version: M2Version) -> Self {
        let mut new_emitter = self.clone();

        // Handle version-specific conversions
        if target_version >= M2Version::MoP && self.texture_slice.is_none() {
            // When upgrading to MoP or later, add texture slice and variation if missing
            new_emitter.texture_slice = Some(0);
            new_emitter.variation = Some(0);
        } else if target_version < M2Version::MoP {
            // When downgrading to pre-MoP, remove texture slice and variation
            new_emitter.texture_slice = None;
            new_emitter.variation = None;
        }

        new_emitter
    }

    /// Calculate the size of this ribbon emitter in bytes for a specific version
    pub fn size_in_bytes(version: M2Version) -> usize {
        let mut size = 0;

        // Bone index
        size += 4;

        // Position
        size += 3 * 4;

        // Texture indices
        size += 2 * 4;

        // Material indices
        size += 2 * 4;

        // Animation blocks (4 blocks, each with 2 tracks)
        size += 4 * (2 * (2 + 2 + 2 * 4)); // interpolation + global_sequence + timestamps + values

        // Edges per second, edge lifetime, gravity
        size += 3 * 4;

        // Texture rows, texture cols
        size += 2 * 2;

        // Version-specific fields
        if version >= M2Version::MoP {
            // Texture slice, variation
            size += 2 * 2;
        }

        // ID, flags
        size += 2 * 4;

        size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunks::animation::M2AnimationTrackHeader;
    use std::io::Cursor;

    #[test]
    fn test_ribbon_emitter_parse_write_classic() {
        let ribbon = M2RibbonEmitter {
            bone_index: 1,
            position: C3Vector {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            texture_indices: M2Array::new(1, 0x100),
            material_indices: M2Array::new(1, 0x200),
            color_animation: M2AnimationBlock::new(M2AnimationTrackHeader::new()),
            alpha_animation: M2AnimationBlock::new(M2AnimationTrackHeader::new()),
            height_above_animation: M2AnimationBlock::new(M2AnimationTrackHeader::new()),
            height_below_animation: M2AnimationBlock::new(M2AnimationTrackHeader::new()),
            edges_per_second: 30.0,
            edge_lifetime: 1.0,
            gravity: 9.8,
            texture_rows: 1,
            texture_cols: 1,
            texture_slice: None,
            variation: None,
            id: 0,
            flags: 0,
        };

        // Test write
        let mut data = Vec::new();
        ribbon
            .write(&mut data, M2Version::Classic.to_header_version())
            .unwrap();

        // Test parse
        let mut cursor = Cursor::new(data);
        let parsed =
            M2RibbonEmitter::parse(&mut cursor, M2Version::Classic.to_header_version()).unwrap();

        assert_eq!(parsed.bone_index, 1);
        assert_eq!(parsed.position.x, 1.0);
        assert_eq!(parsed.position.y, 2.0);
        assert_eq!(parsed.position.z, 3.0);
        assert_eq!(parsed.texture_indices.count, 1);
        assert_eq!(parsed.texture_indices.offset, 0x100);
        assert_eq!(parsed.material_indices.count, 1);
        assert_eq!(parsed.material_indices.offset, 0x200);
        assert_eq!(parsed.edges_per_second, 30.0);
        assert_eq!(parsed.edge_lifetime, 1.0);
        assert_eq!(parsed.gravity, 9.8);
        assert_eq!(parsed.texture_rows, 1);
        assert_eq!(parsed.texture_cols, 1);
        assert_eq!(parsed.texture_slice, None);
        assert_eq!(parsed.variation, None);
        assert_eq!(parsed.id, 0);
        assert_eq!(parsed.flags, 0);
    }

    #[test]
    fn test_ribbon_emitter_parse_write_mop() {
        let ribbon = M2RibbonEmitter {
            bone_index: 1,
            position: C3Vector {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            texture_indices: M2Array::new(1, 0x100),
            material_indices: M2Array::new(1, 0x200),
            color_animation: M2AnimationBlock::new(M2AnimationTrackHeader::new()),
            alpha_animation: M2AnimationBlock::new(M2AnimationTrackHeader::new()),
            height_above_animation: M2AnimationBlock::new(M2AnimationTrackHeader::new()),
            height_below_animation: M2AnimationBlock::new(M2AnimationTrackHeader::new()),
            edges_per_second: 30.0,
            edge_lifetime: 1.0,
            gravity: 9.8,
            texture_rows: 1,
            texture_cols: 1,
            texture_slice: Some(0),
            variation: Some(0),
            id: 0,
            flags: 0,
        };

        // Test write
        let mut data = Vec::new();
        ribbon
            .write(&mut data, M2Version::MoP.to_header_version())
            .unwrap();

        // Test parse
        let mut cursor = Cursor::new(data);
        let parsed =
            M2RibbonEmitter::parse(&mut cursor, M2Version::MoP.to_header_version()).unwrap();

        assert_eq!(parsed.bone_index, 1);
        assert_eq!(parsed.position.x, 1.0);
        assert_eq!(parsed.position.y, 2.0);
        assert_eq!(parsed.position.z, 3.0);
        assert_eq!(parsed.texture_indices.count, 1);
        assert_eq!(parsed.texture_indices.offset, 0x100);
        assert_eq!(parsed.material_indices.count, 1);
        assert_eq!(parsed.material_indices.offset, 0x200);
        assert_eq!(parsed.edges_per_second, 30.0);
        assert_eq!(parsed.edge_lifetime, 1.0);
        assert_eq!(parsed.gravity, 9.8);
        assert_eq!(parsed.texture_rows, 1);
        assert_eq!(parsed.texture_cols, 1);
        assert_eq!(parsed.texture_slice, Some(0));
        assert_eq!(parsed.variation, Some(0));
        assert_eq!(parsed.id, 0);
        assert_eq!(parsed.flags, 0);
    }

    #[test]
    fn test_ribbon_emitter_convert() {
        // Create a Classic ribbon emitter
        let classic_ribbon = M2RibbonEmitter {
            bone_index: 1,
            position: C3Vector {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            texture_indices: M2Array::new(1, 0x100),
            material_indices: M2Array::new(1, 0x200),
            color_animation: M2AnimationBlock::new(M2AnimationTrackHeader::new()),
            alpha_animation: M2AnimationBlock::new(M2AnimationTrackHeader::new()),
            height_above_animation: M2AnimationBlock::new(M2AnimationTrackHeader::new()),
            height_below_animation: M2AnimationBlock::new(M2AnimationTrackHeader::new()),
            edges_per_second: 30.0,
            edge_lifetime: 1.0,
            gravity: 9.8,
            texture_rows: 1,
            texture_cols: 1,
            texture_slice: None,
            variation: None,
            id: 0,
            flags: 0,
        };

        // Convert to MoP
        let mop_ribbon = classic_ribbon.convert(M2Version::MoP);

        // Should have texture slice and variation
        assert!(mop_ribbon.texture_slice.is_some());
        assert!(mop_ribbon.variation.is_some());

        // Convert back to Classic
        let classic_ribbon2 = mop_ribbon.convert(M2Version::Classic);

        // Should not have texture slice and variation
        assert!(classic_ribbon2.texture_slice.is_none());
        assert!(classic_ribbon2.variation.is_none());
    }
}
