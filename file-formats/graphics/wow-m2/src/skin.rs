use bitflags::bitflags;
use custom_debug::Debug;
use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{C3Vector, MagicStr, VWowStructR, WowArray, WowStructW};
use wow_data_derive::{WowHeaderR, WowHeaderW};
use wow_utils::debug;

use std::io::Cursor;

use crate::M2Error;
use crate::error::Result;

pub const SKIN_MAGIC: MagicStr = *b"SKIN";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SkinVersion {
    /// Used up to WotLK
    V1,
    /// Used in WotLK
    V2,
    /// Used in Cataclysm and later
    V3,
}

impl DataVersion for SkinVersion {}

impl TryFrom<u32> for SkinVersion {
    type Error = M2Error;

    fn try_from(value: u32) -> Result<Self> {
        Ok(match value {
            1 => Self::V1,
            2 => Self::V2,
            3 => Self::V3,
            _ => {
                return Err(M2Error::ParseError(format!(
                    "Invalid skin version: {}",
                    value
                )));
            }
        })
    }
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = SkinVersion)]
pub enum SkinMagic {
    None,

    #[wow_data(read_if = version >= SkinVersion::V2)]
    Some([u8; 4]),
}

impl Default for SkinMagic {
    fn default() -> Self {
        Self::Some(SKIN_MAGIC)
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    /// Usually 0x10(BATCH_SUPPORT) for static textures, and 0 for animated textures
    pub struct M2BatchFlags: u8 {
        const MATERIAL_INVERT = 0x01;
        const TRANSFORM = 0x02;
        const PROJECTED = 0x04;
        const BATCH_SUPPORT = 0x10;
        const PROJECTED_2 = 0x20;
        const TRANSPARENCY_SOMETHING = 0x40;
        const UNKNOWN_0x80 = 0x80;
    }
}

impl WowHeaderR for M2BatchFlags {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        Ok(Self::from_bits_retain(reader.wow_read()?))
    }
}
impl WowHeaderW for M2BatchFlags {
    fn wow_write<W: Write>(&self, writer: &mut W) -> WDResult<()> {
        writer.wow_write(&self.bits())?;
        Ok(())
    }
    fn wow_size(&self) -> usize {
        1
    }
}

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
pub struct M2Batch {
    pub flags: M2BatchFlags,
    pub priority_plane: i8,
    pub shader_id: u16,
    pub skin_section_index: u16,
    pub geoset_index: u16,
    pub color_index: u16,
    pub material_index: u16,
    pub material_layer: u16,
    pub texture_count: u16,
    pub texture_combo_index: u16,
    pub texture_coord_combo_index: u16,
    pub texture_weight_combo_index: u16,
    pub texture_transform_combo_index: u16,
}

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
pub struct M2ShadowBatch {
    pub flags: u8,
    pub flags2: u8,
    pub _unknown1: u16,
    pub submesh_id: u16,
    pub texture_id: u16,
    pub color_id: u16,
    pub transparency_id: u16,
}

#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
#[wow_data(version = SkinVersion)]
pub enum SkinShadowBatchesHeader {
    None,

    #[wow_data(read_if = version >= SkinVersion::V3)]
    Some(WowArray<M2ShadowBatch>),
}

impl Default for SkinShadowBatchesHeader {
    fn default() -> Self {
        Self::Some(Default::default())
    }
}

impl SkinShadowBatchesHeader {
    fn add_offset(&mut self, offset: usize) {
        if let Self::Some(arr) = self {
            arr.add_offset(offset);
        }
    }
}

#[derive(Debug, Clone)]
pub enum SkinShadowBatches {
    None,

    Some(Vec<M2ShadowBatch>),
}

impl Default for SkinShadowBatches {
    fn default() -> Self {
        Self::Some(Default::default())
    }
}

impl VWowDataR<SkinVersion, SkinShadowBatchesHeader> for SkinShadowBatches {
    fn new_from_header<R: Read + Seek>(
        reader: &mut R,
        header: &SkinShadowBatchesHeader,
    ) -> WDResult<Self> {
        Ok(match header {
            SkinShadowBatchesHeader::Some(array) => Self::Some(array.wow_read_to_vec(reader)?),
            SkinShadowBatchesHeader::None => Self::None,
        })
    }
}

impl SkinShadowBatches {
    fn wow_write<W: Write + Seek>(&self, writer: &mut W) -> Result<SkinShadowBatchesHeader> {
        Ok(match self {
            Self::None => SkinShadowBatchesHeader::None,
            Self::Some(vec) => SkinShadowBatchesHeader::Some(vec.wow_write(writer)?),
        })
    }
}

/// OldSkin file header
#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
#[wow_data(version = SkinVersion)]
pub struct SkinHeader {
    /// Magic signature ("SKIN")
    #[wow_data(versioned)]
    pub magic: SkinMagic,
    pub indices: WowArray<u16>,
    pub triangles: WowArray<u16>,
    pub bone_indices: WowArray<u8>,
    pub submeshes: WowArray<SkinSubmesh>,
    /// Also known as batches
    pub texture_units: WowArray<M2Batch>,
    pub bone_count_max: u32,
    #[wow_data(versioned)]
    pub shadow_batches: SkinShadowBatchesHeader,
}

/// Submesh structure
#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
pub struct SkinSubmesh {
    pub id: u16,
    /// Level of detail
    pub level: u16,
    pub vertex_start: u16,
    pub vertex_count: u16,
    pub triangle_start: u16,
    pub triangle_count: u16,
    pub bone_count: u16,
    pub bone_start: u16,
    /// Bone influence count (max bones per vertex)
    pub bone_influence: u16,
    pub _padding: u16,
    /// Center of mass
    pub center: C3Vector,
    pub sort_center: C3Vector,
    /// Bounding sphere radius
    pub bounding_radius: f32,
}

#[derive(Debug, Clone, Default)]
pub struct Skin {
    pub header: SkinHeader,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub indices: Vec<u16>,
    /// Triangles (each is 3 indices)
    #[debug(with = debug::trimmed_collection_fmt)]
    pub triangles: Vec<u16>,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub bone_indices: Vec<u8>,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub submeshes: Vec<SkinSubmesh>,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub texture_units: Vec<M2Batch>,

    pub shadow_batches: SkinShadowBatches,
}

impl VWowStructR<SkinVersion> for Skin {
    fn wow_read<R: Read + Seek>(reader: &mut R, version: SkinVersion) -> WDResult<Self> {
        let header: SkinHeader = reader.wow_read_versioned(version)?;

        Ok(Self {
            indices: header.indices.wow_read_to_vec(reader)?,
            triangles: header.triangles.wow_read_to_vec(reader)?,
            bone_indices: header.bone_indices.wow_read_to_vec(reader)?,
            submeshes: header.submeshes.wow_read_to_vec(reader)?,
            texture_units: header.texture_units.wow_read_to_vec(reader)?,
            shadow_batches: reader.v_new_from_header(&header.shadow_batches)?,
            header,
        })
    }
}

impl WowStructW for Skin {
    fn wow_write<W: Write + Seek>(&self, writer: &mut W) -> WDResult<()> {
        let mut header = self.header.clone();

        let mut data_section = Vec::new();
        let mut data_section_writer = Cursor::new(&mut data_section);

        header.indices = self.indices.wow_write(&mut data_section_writer)?;
        header.triangles = self.triangles.wow_write(&mut data_section_writer)?;
        header.bone_indices = self.bone_indices.wow_write(&mut data_section_writer)?;
        header.submeshes = self.submeshes.wow_write(&mut data_section_writer)?;
        header.texture_units = self.texture_units.wow_write(&mut data_section_writer)?;
        header.shadow_batches = self.shadow_batches.wow_write(&mut data_section_writer)?;

        let header_size = header.wow_size();
        header.indices.add_offset(header_size);
        header.triangles.add_offset(header_size);
        header.bone_indices.add_offset(header_size);
        header.submeshes.add_offset(header_size);
        header.texture_units.add_offset(header_size);
        header.shadow_batches.add_offset(header_size);

        writer.wow_write(&header)?;
        writer.write_all(&data_section)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_submesh_parse_write() {
        let submesh = SkinSubmesh {
            id: 1,
            level: 0,
            vertex_start: 0,
            vertex_count: 100,
            triangle_start: 0,
            triangle_count: 50,
            bone_count: 10,
            bone_start: 0,
            bone_influence: 4,
            _padding: 0,
            center: C3Vector::new(1.0, 2.0, 3.0),
            sort_center: C3Vector::new(1.5, 2.5, 3.5),
            bounding_radius: 5.0,
        };

        let mut data = Vec::new();
        submesh.wow_write(&mut data).unwrap();

        let mut cursor = Cursor::new(data);
        let parsed_submesh = SkinSubmesh::wow_read(&mut cursor).unwrap();

        assert_eq!(parsed_submesh.id, 1);
        assert_eq!(parsed_submesh.vertex_count, 100);
        assert_eq!(parsed_submesh.triangle_count, 50);
        assert_eq!(parsed_submesh.bone_count, 10);
        assert_eq!(parsed_submesh.bone_influence, 4);
        assert_eq!(parsed_submesh.center, C3Vector::new(1.0, 2.0, 3.0));
        assert_eq!(parsed_submesh.sort_center, C3Vector::new(1.5, 2.5, 3.5));
        assert_eq!(parsed_submesh.bounding_radius, 5.0);
    }
}
