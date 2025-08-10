use bitflags::bitflags;
use custom_debug::Debug;
use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{C3Vector, WowArray};
use wow_data_derive::{VWowHeaderR, WowHeaderR, WowHeaderW};
use wow_utils::debug;

use std::fmt;
use std::fs::File;
use std::io::{Cursor, SeekFrom};
use std::path::Path;

use crate::error::{M2Error, Result};
use crate::version::M2Version;

/// Magic signature for Skin files ("SKIN")
pub const SKIN_MAGIC: [u8; 4] = *b"SKIN";

pub trait SkinHeaderT: Sized {
    type ExtraElement: fmt::Debug + WowHeaderR + WowHeaderW;

    fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self>;
    fn write<W: Write>(&self, writer: &mut W) -> Result<()>;
    fn calculate_size(&self) -> usize;
    fn set_array_fields(
        &mut self,
        indices: WowArray<u16>,
        triangles: WowArray<u16>,
        bone_indices: WowArray<u8>,
        submeshes: WowArray<SkinSubmesh>,
        extra_array: WowArray<Self::ExtraElement>,
    );
    fn indices(&self) -> &WowArray<u16>;
    fn triangles(&self) -> &WowArray<u16>;
    fn bone_indices(&self) -> &WowArray<u8>;
    fn submeshes(&self) -> &WowArray<SkinSubmesh>;
    fn extra_array(&self) -> &WowArray<Self::ExtraElement>;
    fn parse_extra_array<R: Read + Seek>(&self, reader: &mut R) -> Result<Vec<Self::ExtraElement>>;

    fn write_extra_element(data_section: &mut Vec<u8>, element: &Self::ExtraElement)
    -> Result<u32>;
}

#[derive(Debug, Clone, VWowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub enum SkinCenterPosition {
    None,

    #[wow_data(read_if = version > M2Version::BfA)]
    Some {
        position: C3Vector,
        radius: f32,
    },
}

/// Skin file header
#[derive(Debug, Clone, VWowHeaderR, WowHeaderW)]
#[wow_data(version = M2Version)]
pub struct SkinHeader {
    /// Magic signature ("SKIN")
    pub magic: [u8; 4],
    pub version: M2Version,
    pub name: WowArray<u8>,
    pub vertex_count: u32,
    pub indices: WowArray<u16>,
    pub triangles: WowArray<u16>,
    pub bone_indices: WowArray<u8>,
    pub submeshes: WowArray<SkinSubmesh>,
    pub material_lookup: WowArray<u16>,
    #[wow_data(versioned)]
    pub center_position: SkinCenterPosition,
}

impl WowHeaderR for SkinHeader {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        let start_position = reader.stream_position()?;

        let magic: [u8; 4] = reader.wow_read()?;
        let magic_str = String::from_utf8_lossy(&magic);
        let skin_magic_str = String::from_utf8_lossy(&SKIN_MAGIC);

        if magic != SKIN_MAGIC {
            return Err(M2Error::InvalidMagic {
                expected: skin_magic_str.into(),
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

impl SkinHeaderT for SkinHeader {
    type ExtraElement = u16;

    fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok(reader.wow_read()?)
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.wow_write(self)?;
        Ok(())
    }

    fn calculate_size(&self) -> usize {
        self.wow_size()
    }

    fn set_array_fields(
        &mut self,
        indices: WowArray<u16>,
        triangles: WowArray<u16>,
        bone_indices: WowArray<u8>,
        submeshes: WowArray<SkinSubmesh>,
        extra_array: WowArray<Self::ExtraElement>,
    ) {
        self.indices = indices;
        self.triangles = triangles;
        self.bone_indices = bone_indices;
        self.submeshes = submeshes;
        self.material_lookup = extra_array;
    }

    fn indices(&self) -> &WowArray<u16> {
        &self.indices
    }

    fn triangles(&self) -> &WowArray<u16> {
        &self.triangles
    }

    fn bone_indices(&self) -> &WowArray<u8> {
        &self.bone_indices
    }

    fn submeshes(&self) -> &WowArray<SkinSubmesh> {
        &self.submeshes
    }

    fn extra_array(&self) -> &WowArray<Self::ExtraElement> {
        &self.material_lookup
    }

    fn parse_extra_array<R: Read + Seek>(&self, reader: &mut R) -> Result<Vec<Self::ExtraElement>> {
        reader.seek(SeekFrom::Start(self.material_lookup.offset as u64))?;
        let mut material_lookup = Vec::with_capacity(self.material_lookup.count as usize);
        for _ in 0..self.material_lookup.count {
            material_lookup.push(reader.read_u16_le()?);
        }
        Ok(material_lookup)
    }

    fn write_extra_element(
        data_section: &mut Vec<u8>,
        element: &Self::ExtraElement,
    ) -> Result<u32> {
        let res = element.to_le_bytes();
        data_section.extend_from_slice(&res);
        Ok(res.len() as u32)
    }
}

impl SkinHeader {
    /// Create a new Skin header for a specific version
    pub fn new(version: M2Version) -> Self {
        Self {
            magic: SKIN_MAGIC,
            version,
            name: WowArray::new(0, 0),
            vertex_count: 0,
            indices: WowArray::new(0, 0),
            triangles: WowArray::new(0, 0),
            bone_indices: WowArray::new(0, 0),
            submeshes: WowArray::new(0, 0),
            material_lookup: WowArray::new(0, 0),
            center_position: if version >= M2Version::BfA {
                SkinCenterPosition::Some {
                    position: C3Vector::default(),
                    radius: 0.0,
                }
            } else {
                SkinCenterPosition::None
            },
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, WowHeaderR, WowHeaderW)]
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

/// OldSkin file header
#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
pub struct OldSkinHeader {
    /// Magic signature ("SKIN")
    pub magic: [u8; 4],
    pub indices: WowArray<u16>,
    pub triangles: WowArray<u16>,
    pub bone_indices: WowArray<u8>,
    pub submeshes: WowArray<SkinSubmesh>,
    /// Texture units (batches)
    pub texture_units: WowArray<M2Batch>,
    pub bone_count_max: u32,
}

impl SkinHeaderT for OldSkinHeader {
    type ExtraElement = M2Batch;

    fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok(reader.wow_read()?)
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.wow_write(self)?;
        Ok(())
    }

    fn calculate_size(&self) -> usize {
        self.wow_size()
    }

    fn set_array_fields(
        &mut self,
        indices: WowArray<u16>,
        triangles: WowArray<u16>,
        bone_indices: WowArray<u8>,
        submeshes: WowArray<SkinSubmesh>,
        extra_array: WowArray<Self::ExtraElement>,
    ) {
        self.indices = indices;
        self.triangles = triangles;
        self.bone_indices = bone_indices;
        self.submeshes = submeshes;
        self.texture_units = extra_array;
    }

    fn indices(&self) -> &WowArray<u16> {
        &self.indices
    }

    fn triangles(&self) -> &WowArray<u16> {
        &self.triangles
    }

    fn bone_indices(&self) -> &WowArray<u8> {
        &self.bone_indices
    }

    fn submeshes(&self) -> &WowArray<SkinSubmesh> {
        &self.submeshes
    }

    fn extra_array(&self) -> &WowArray<Self::ExtraElement> {
        &self.texture_units
    }

    fn parse_extra_array<R: Read + Seek>(&self, reader: &mut R) -> Result<Vec<Self::ExtraElement>> {
        reader.seek(SeekFrom::Start(self.texture_units.offset as u64))?;
        let mut texture_units = Vec::with_capacity(self.texture_units.count as usize);
        for _ in 0..self.texture_units.count {
            texture_units.push(reader.wow_read()?);
        }
        Ok(texture_units)
    }

    fn write_extra_element(
        data_section: &mut Vec<u8>,
        element: &Self::ExtraElement,
    ) -> Result<u32> {
        let mut res = Vec::new();
        let mut writer = Cursor::new(&mut res);
        writer.wow_write(element)?;
        data_section.extend_from_slice(&res);
        Ok(res.len() as u32)
    }
}

impl OldSkinHeader {
    /// Create a new Skin header for a specific version
    pub fn new() -> Self {
        Self {
            magic: SKIN_MAGIC,
            indices: WowArray::new(0, 0),
            triangles: WowArray::new(0, 0),
            bone_indices: WowArray::new(0, 0),
            submeshes: WowArray::new(0, 0),
            texture_units: WowArray::new(0, 0),
            bone_count_max: 0,
        }
    }
}

/// Submesh structure
#[derive(Debug, Clone, WowHeaderR, WowHeaderW)]
pub struct SkinSubmesh {
    /// Submesh ID
    pub id: u16,
    /// Level of detail
    pub level: u16,
    /// Start vertex index
    pub vertex_start: u16,
    /// Vertex count
    pub vertex_count: u16,
    /// Start triangle index
    pub triangle_start: u16,
    /// Triangle count
    pub triangle_count: u16,
    /// Bone count
    pub bone_count: u16,
    /// Start bone index
    pub bone_start: u16,
    /// Bone influence count (max bones per vertex)
    pub bone_influence: u16,
    /// Center of mass
    pub center: C3Vector,
    /// Sort center
    pub sort_center: C3Vector,
    /// Bounding sphere radius
    pub bounding_radius: f32,
}

/// Main Skin structure
#[derive(Debug, Clone)]
pub struct SkinG<H>
where
    H: SkinHeaderT,
{
    /// Skin header
    pub header: H,
    /// Indices
    #[debug(with = debug::trimmed_collection_fmt)]
    pub indices: Vec<u16>,
    /// Triangles (each is 3 indices)
    #[debug(with = debug::trimmed_collection_fmt)]
    pub triangles: Vec<u16>,
    /// Bone indices
    #[debug(with = debug::trimmed_collection_fmt)]
    pub bone_indices: Vec<u8>,
    /// Submeshes
    #[debug(with = debug::trimmed_collection_fmt)]
    pub submeshes: Vec<SkinSubmesh>,
    #[debug(with = debug::trimmed_collection_fmt)]
    pub extra_array: Vec<H::ExtraElement>,
}

impl<H> SkinG<H>
where
    H: SkinHeaderT + Clone,
{
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let header = H::parse(reader)?;

        let indices = header.indices().wow_read_to_vec(reader)?;
        let triangles = header.triangles().wow_read_to_vec(reader)?;
        let bone_indices = header.bone_indices().wow_read_to_vec(reader)?;
        let submeshes = header.submeshes().wow_read_to_vec(reader)?;
        let extra_array = header.parse_extra_array(reader)?;

        Ok(Self {
            header,
            indices,
            triangles,
            bone_indices,
            submeshes,
            extra_array,
        })
    }

    /// Load a Skin from a file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        Self::parse(&mut file)
    }

    /// Save a Skin to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut file = File::create(path)?;
        self.write(&mut file)
    }

    /// Write a Skin to a writer
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        // We need to recalculate all offsets and build the file in memory
        let mut data_section = Vec::new();
        let mut header = self.header.clone();

        // Start with header size (will be written last)
        let header_size = header.calculate_size();
        let mut current_offset = header_size as u32;

        // Write indices
        let indices = if !self.indices.is_empty() {
            let indices = WowArray::new(self.indices.len() as u32, current_offset);

            for &index in &self.indices {
                data_section.extend_from_slice(&index.to_le_bytes());
            }

            current_offset += (self.indices.len() * std::mem::size_of::<u16>()) as u32;
            indices
        } else {
            WowArray::new(0, 0)
        };

        // Write triangles
        let triangles = if !self.triangles.is_empty() {
            let triangles = WowArray::new(self.triangles.len() as u32, current_offset);

            for &triangle in &self.triangles {
                data_section.extend_from_slice(&triangle.to_le_bytes());
            }

            current_offset += (self.triangles.len() * std::mem::size_of::<u16>()) as u32;

            triangles
        } else {
            WowArray::new(0, 0)
        };

        // Write bone indices
        let bone_indices = if !self.bone_indices.is_empty() {
            let bone_indices = WowArray::new(self.bone_indices.len() as u32, current_offset);

            for &bone_index in &self.bone_indices {
                data_section.push(bone_index);
            }

            current_offset += self.bone_indices.len() as u32;

            bone_indices
        } else {
            WowArray::new(0, 0)
        };

        // Write submeshes
        let submeshes = if !self.submeshes.is_empty() {
            let submeshes = WowArray::new(self.submeshes.len() as u32, current_offset);

            for submesh in &self.submeshes {
                let mut submesh_data = Vec::new();
                submesh.wow_write(&mut submesh_data)?;
                data_section.extend_from_slice(&submesh_data);
            }

            current_offset += (self.submeshes.len() * 40) as u32; // Each submesh is 40 bytes
            submeshes
        } else {
            WowArray::new(0, 0)
        };

        let extra_array = if !self.extra_array.is_empty() {
            let extra_array = WowArray::new(self.extra_array.len() as u32, current_offset);

            for element in &self.extra_array {
                current_offset += H::write_extra_element(&mut data_section, &element)?;
            }

            extra_array
        } else {
            WowArray::new(0, 0)
        };

        header.set_array_fields(indices, triangles, bone_indices, submeshes, extra_array);

        // Finally, write the header followed by the data section
        header.write(writer)?;
        writer.write_all(&data_section)?;

        Ok(())
    }
}

impl SkinG<SkinHeader> {
    /// Convert this skin to a different version
    pub fn convert(&self, target_version: M2Version) -> Result<Self> {
        let source_version = self.header.version;

        if source_version == target_version {
            return Ok(self.clone());
        }

        // Create a new skin with the target version
        let mut new_skin = self.clone();

        // Update header version
        let mut header = SkinHeader::new(target_version);
        header.name = self.header.name.clone();
        header.vertex_count = self.header.vertex_count;

        // // Handle version-specific conversions
        // if target_version >= M2Version::BfA && source_version < M2Version::BfA {
        //     // When upgrading to BfA or later, add center position and bounds if missing
        //     if header.center_position.is_none() {
        //         // Calculate center of mass from submeshes
        //         let mut center = [0.0, 0.0, 0.0];
        //         let mut max_radius = 0.0;
        //
        //         if !self.submeshes.is_empty() {
        //             for submesh in &self.submeshes {
        //                 for (i, center_val) in center.iter_mut().enumerate() {
        //                     *center_val += submesh.center[i];
        //                 }
        //
        //                 if submesh.bounding_radius > max_radius {
        //                     max_radius = submesh.bounding_radius;
        //                 }
        //             }
        //
        //             // Average the center
        //             let count = self.submeshes.len() as f32;
        //             for item in &mut center {
        //                 *item /= count;
        //             }
        //         }
        //
        //         header.center_position = Some(center);
        //         header.center_bounds = Some(max_radius);
        //     }
        // } else if target_version < M2Version::BfA && source_version >= M2Version::BfA {
        //     // When downgrading from BfA or later, remove center position and bounds
        //     header.center_position = None;
        //     header.center_bounds = None;
        // }

        new_skin.header = header;

        Ok(new_skin)
    }
}

pub type Skin = SkinG<SkinHeader>;
pub type OldSkin = SkinG<OldSkinHeader>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_skin_header_parse() {
        let mut data = Vec::new();

        // Magic "SKIN"
        data.extend_from_slice(&SKIN_MAGIC);

        // Version
        data.extend_from_slice(&264u32.to_le_bytes());

        // Name
        data.extend_from_slice(&0u32.to_le_bytes()); // count = 0
        data.extend_from_slice(&0u32.to_le_bytes()); // offset = 0

        // Vertex count
        data.extend_from_slice(&100u32.to_le_bytes());

        // Indices
        data.extend_from_slice(&200u32.to_le_bytes()); // count = 200
        data.extend_from_slice(&0x100u32.to_le_bytes()); // offset = 0x100

        // Triangles
        data.extend_from_slice(&300u32.to_le_bytes()); // count = 300
        data.extend_from_slice(&0x200u32.to_le_bytes()); // offset = 0x200

        // Bone indices
        data.extend_from_slice(&50u32.to_le_bytes()); // count = 50
        data.extend_from_slice(&0x300u32.to_le_bytes()); // offset = 0x300

        // Submeshes
        data.extend_from_slice(&2u32.to_le_bytes()); // count = 2
        data.extend_from_slice(&0x400u32.to_le_bytes()); // offset = 0x400

        // Material lookup
        data.extend_from_slice(&5u32.to_le_bytes()); // count = 5
        data.extend_from_slice(&0x500u32.to_le_bytes()); // offset = 0x500

        let mut cursor = Cursor::new(data);
        let header = SkinHeader::parse(&mut cursor).unwrap();

        assert_eq!(header.magic, SKIN_MAGIC);
        assert_eq!(header.version, M2Version::WotLK);
        assert_eq!(header.vertex_count, 100);
        assert_eq!(header.indices.count, 200);
        assert_eq!(header.indices.offset, 0x100);
        assert_eq!(header.triangles.count, 300);
        assert_eq!(header.triangles.offset, 0x200);
        assert_eq!(header.bone_indices.count, 50);
        assert_eq!(header.bone_indices.offset, 0x300);
        assert_eq!(header.submeshes.count, 2);
        assert_eq!(header.submeshes.offset, 0x400);
        assert_eq!(header.material_lookup.count, 5);
        assert_eq!(header.material_lookup.offset, 0x500);
        // assert!(header.center_position.is_none());
        // assert!(header.center_bounds.is_none());
    }

    // #[test]
    // fn test_submesh_parse_write() {
    //     let submesh = SkinSubmesh {
    //         id: 1,
    //         level: 0,
    //         vertex_start: 0,
    //         vertex_count: 100,
    //         triangle_start: 0,
    //         triangle_count: 50,
    //         bone_count: 10,
    //         bone_start: 0,
    //         bone_influence: 4,
    //         center: [1.0, 2.0, 3.0],
    //         sort_center: [1.5, 2.5, 3.5],
    //         bounding_radius: 5.0,
    //     };
    //
    //     let mut data = Vec::new();
    //     submesh.write(&mut data).unwrap();
    //
    //     let mut cursor = Cursor::new(data);
    //     let parsed_submesh = SkinSubmesh::parse(&mut cursor).unwrap();
    //
    //     assert_eq!(parsed_submesh.id, 1);
    //     assert_eq!(parsed_submesh.vertex_count, 100);
    //     assert_eq!(parsed_submesh.triangle_count, 50);
    //     assert_eq!(parsed_submesh.bone_count, 10);
    //     assert_eq!(parsed_submesh.bone_influence, 4);
    //     assert_eq!(parsed_submesh.center, [1.0, 2.0, 3.0]);
    //     assert_eq!(parsed_submesh.sort_center, [1.5, 2.5, 3.5]);
    //     assert_eq!(parsed_submesh.bounding_radius, 5.0);
    // }
}
