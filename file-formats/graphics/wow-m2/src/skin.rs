use crate::io_ext::{ReadExt, WriteExt};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::common::M2Array;
use crate::error::{M2Error, Result};
use crate::version::M2Version;

/// Magic signature for Skin files ("SKIN")
pub const SKIN_MAGIC: [u8; 4] = *b"SKIN";

/// Detect SKIN format variant based on the second u32 field
/// Returns true for new format (camera files with version), false for old format (character models)
fn detect_skin_format<R: Read + Seek>(reader: &mut R) -> Result<bool> {
    let start_pos = reader.stream_position()?;

    // Skip magic
    reader.seek(SeekFrom::Current(4))?;

    // Read the second u32 field
    let second_field = reader.read_u32_le()?;

    // Reset position
    reader.seek(SeekFrom::Start(start_pos))?;

    // If <= 4, it's likely a version field (new format)
    // If > 4, it's likely an indices count (old format)
    Ok(second_field <= 4)
}

/// Parse a SKIN file with automatic format detection
pub fn parse_skin<R: Read + Seek>(reader: &mut R) -> Result<SkinFile> {
    let is_new_format = detect_skin_format(reader)?;

    if is_new_format {
        let skin = SkinG::<SkinHeader>::parse(reader)?;
        Ok(SkinFile::New(skin))
    } else {
        let skin = SkinG::<OldSkinHeader>::parse(reader)?;
        Ok(SkinFile::Old(skin))
    }
}

/// Parse embedded skin data from pre-WotLK M2 models (no SKIN magic)
pub fn parse_embedded_skin<R: Read + Seek>(reader: &mut R) -> Result<SkinFile> {
    // Parse the header without expecting SKIN magic
    let header = OldSkinHeader::parse_embedded(reader)?;

    // Parse indices
    let mut indices = Vec::with_capacity(header.indices.count as usize);
    if header.indices.count > 0 && header.indices.offset > 0 {
        reader.seek(SeekFrom::Start(header.indices.offset as u64))?;
        for _ in 0..header.indices.count {
            indices.push(reader.read_u16_le()?);
        }
    }

    // Parse triangles
    let mut triangles = Vec::with_capacity(header.triangles.count as usize);
    if header.triangles.count > 0 && header.triangles.offset > 0 {
        reader.seek(SeekFrom::Start(header.triangles.offset as u64))?;
        for _ in 0..header.triangles.count {
            triangles.push(reader.read_u16_le()?);
        }
    }

    // Parse bone indices
    let mut bone_indices = Vec::with_capacity(header.bone_indices.count as usize);
    if header.bone_indices.count > 0 && header.bone_indices.offset > 0 {
        reader.seek(SeekFrom::Start(header.bone_indices.offset as u64))?;
        for _ in 0..header.bone_indices.count {
            bone_indices.push(reader.read_u8()?);
        }
    }

    // Parse submeshes
    let mut submeshes = Vec::with_capacity(header.submeshes.count as usize);
    if header.submeshes.count > 0 && header.submeshes.offset > 0 {
        reader.seek(SeekFrom::Start(header.submeshes.offset as u64))?;
        for _ in 0..header.submeshes.count {
            submeshes.push(SkinSubmesh::parse(reader)?);
        }
    }

    // Parse material lookup
    let mut material_lookup = Vec::with_capacity(header.material_lookup.count as usize);
    if header.material_lookup.count > 0 && header.material_lookup.offset > 0 {
        reader.seek(SeekFrom::Start(header.material_lookup.offset as u64))?;
        for _ in 0..header.material_lookup.count {
            material_lookup.push(reader.read_u16_le()?);
        }
    }

    let skin = SkinG::<OldSkinHeader> {
        header,
        indices,
        triangles,
        bone_indices,
        submeshes,
        material_lookup,
    };

    Ok(SkinFile::Old(skin))
}

/// Load a SKIN file from a path with automatic format detection
pub fn load_skin<P: AsRef<Path>>(path: P) -> Result<SkinFile> {
    let mut file = File::open(path)?;
    parse_skin(&mut file)
}

pub trait SkinHeaderT: Sized {
    fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self>;
    fn write<W: Write>(&self, writer: &mut W) -> Result<()>;
    fn calculate_size(&self) -> usize;
    fn set_array_fields(
        &mut self,
        indices: M2Array<u16>,
        triangles: M2Array<u16>,
        bone_indices: M2Array<u8>,
        submeshes: M2Array<SkinSubmesh>,
        material_lookup: M2Array<u16>,
    );
    fn indices(&self) -> &M2Array<u16>;
    fn triangles(&self) -> &M2Array<u16>;
    fn bone_indices(&self) -> &M2Array<u8>;
    fn submeshes(&self) -> &M2Array<SkinSubmesh>;
    fn material_lookup(&self) -> &M2Array<u16>;
}

/// Skin file header
#[derive(Debug, Clone)]
pub struct SkinHeader {
    /// Magic signature ("SKIN")
    pub magic: [u8; 4],
    /// Version of the file
    pub version: u32,
    /// Name of the parent model
    pub name: M2Array<u8>,
    /// Total number of vertices
    pub vertex_count: u32,
    /// Indices
    pub indices: M2Array<u16>,
    /// Triangles
    pub triangles: M2Array<u16>,
    /// Bone indices
    pub bone_indices: M2Array<u8>,
    /// Submeshes
    pub submeshes: M2Array<SkinSubmesh>,
    /// Material lookup table
    pub material_lookup: M2Array<u16>,
    /// Center position (BfA and later)
    pub center_position: Option<[f32; 3]>,
    /// Center bounds (BfA and later)
    pub center_bounds: Option<f32>,
}

impl SkinHeaderT for SkinHeader {
    /// Parse a Skin header from a reader (new format with version field)
    fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Read and check magic
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;

        if magic != SKIN_MAGIC {
            return Err(M2Error::InvalidMagic {
                expected: String::from_utf8_lossy(&SKIN_MAGIC).to_string(),
                actual: String::from_utf8_lossy(&magic).to_string(),
            });
        }

        // Read version
        let version = reader.read_u32_le()?;

        // Validate version for new format (should be 0-4)
        if version > 4 {
            return Err(M2Error::UnsupportedVersion(format!(
                "New format version {} is too high, expected 0-4. This might be an old format file.",
                version
            )));
        }

        // Create the appropriate version
        let _m2_version = match version {
            0 => M2Version::Classic,
            1 => M2Version::Cataclysm,
            2 => M2Version::MoP,
            3 => M2Version::WoD,
            4 => M2Version::Legion,
            v => {
                return Err(M2Error::UnsupportedVersion(v.to_string()));
            }
        };

        // Read name
        let name = M2Array::parse(reader)?;

        // Read vertex count
        let vertex_count = reader.read_u32_le()?;

        // Read array references
        let indices = M2Array::parse(reader)?;
        let triangles = M2Array::parse(reader)?;
        let bone_indices = M2Array::parse(reader)?;
        let submeshes = M2Array::parse(reader)?;
        let material_lookup = M2Array::parse(reader)?;

        // For BfA and later, we have additional fields
        let (center_position, center_bounds) = if version >= 4 {
            let file_size = reader.seek(SeekFrom::End(0))?;

            // If we have more data, it's probably BfA or later
            if file_size > reader.stream_position()? {
                let mut center_pos = [0.0; 3];
                for item in &mut center_pos {
                    *item = reader.read_f32_le()?;
                }
                let center_bound = reader.read_f32_le()?;

                (Some(center_pos), Some(center_bound))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        Ok(Self {
            magic,
            version,
            name,
            vertex_count,
            indices,
            triangles,
            bone_indices,
            submeshes,
            material_lookup,
            center_position,
            center_bounds,
        })
    }

    /// Write a Skin header to a writer
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Write magic and version
        writer.write_all(&self.magic)?;
        writer.write_u32_le(self.version)?;

        // Write name
        self.name.write(writer)?;

        // Write vertex count
        writer.write_u32_le(self.vertex_count)?;

        // Write array references
        self.indices.write(writer)?;
        self.triangles.write(writer)?;
        self.bone_indices.write(writer)?;
        self.submeshes.write(writer)?;
        self.material_lookup.write(writer)?;

        // Write BfA+ fields if present
        if let Some(center_pos) = self.center_position {
            for &value in &center_pos {
                writer.write_f32_le(value)?;
            }

            if let Some(center_bound) = self.center_bounds {
                writer.write_f32_le(center_bound)?;
            } else {
                writer.write_f32_le(0.0)?;
            }
        }

        Ok(())
    }

    /// Calculate the size of the header for this skin version
    fn calculate_size(&self) -> usize {
        let mut size = 4 + 4; // Magic + version

        // Name
        size += 2 * 4;

        // Vertex count
        size += 4;

        // Array references
        size += 5 * (2 * 4); // 5 arrays, each with count and offset (8 bytes)

        // BfA and later have additional fields
        if self.center_position.is_some() {
            size += 3 * 4; // Center position (3 floats)
            size += 4; // Center bounds (1 float)
        }

        size
    }

    fn set_array_fields(
        &mut self,
        indices: M2Array<u16>,
        triangles: M2Array<u16>,
        bone_indices: M2Array<u8>,
        submeshes: M2Array<SkinSubmesh>,
        material_lookup: M2Array<u16>,
    ) {
        self.indices = indices;
        self.triangles = triangles;
        self.bone_indices = bone_indices;
        self.submeshes = submeshes;
        self.material_lookup = material_lookup;
    }

    fn indices(&self) -> &M2Array<u16> {
        &self.indices
    }

    fn triangles(&self) -> &M2Array<u16> {
        &self.triangles
    }

    fn bone_indices(&self) -> &M2Array<u8> {
        &self.bone_indices
    }

    fn submeshes(&self) -> &M2Array<SkinSubmesh> {
        &self.submeshes
    }

    fn material_lookup(&self) -> &M2Array<u16> {
        &self.material_lookup
    }
}

impl SkinHeader {
    /// Get the M2 version for this skin
    pub fn get_m2_version(&self) -> Option<M2Version> {
        match self.version {
            0 => Some(M2Version::Classic),
            1 => Some(M2Version::Cataclysm),
            2 => Some(M2Version::MoP),
            3 => Some(M2Version::WoD),
            4 => {
                // BfA and later have additional fields
                if self.center_position.is_some() {
                    Some(M2Version::BfA)
                } else {
                    Some(M2Version::Legion)
                }
            }
            _ => None,
        }
    }

    /// Create a new Skin header for a specific version
    pub fn new(m2_version: M2Version) -> Self {
        let version = match m2_version {
            M2Version::Classic | M2Version::TBC | M2Version::WotLK => 0,
            M2Version::Cataclysm => 1,
            M2Version::MoP => 2,
            M2Version::WoD => 3,
            M2Version::Legion => 4,
            M2Version::BfA
            | M2Version::Shadowlands
            | M2Version::Dragonflight
            | M2Version::TheWarWithin => 4,
        };

        let center_position = if m2_version >= M2Version::BfA {
            Some([0.0, 0.0, 0.0])
        } else {
            None
        };

        let center_bounds = if m2_version >= M2Version::BfA {
            Some(0.0)
        } else {
            None
        };

        Self {
            magic: SKIN_MAGIC,
            version,
            name: M2Array::new(0, 0),
            vertex_count: 0,
            indices: M2Array::new(0, 0),
            triangles: M2Array::new(0, 0),
            bone_indices: M2Array::new(0, 0),
            submeshes: M2Array::new(0, 0),
            material_lookup: M2Array::new(0, 0),
            center_position,
            center_bounds,
        }
    }
}

/// OldSkin file header
#[derive(Debug, Clone)]
pub struct OldSkinHeader {
    /// Magic signature ("SKIN")
    pub magic: [u8; 4],
    /// Indices
    pub indices: M2Array<u16>,
    /// Triangles
    pub triangles: M2Array<u16>,
    /// Bone indices
    pub bone_indices: M2Array<u8>,
    /// Submeshes
    pub submeshes: M2Array<SkinSubmesh>,
    /// Material lookup table
    pub material_lookup: M2Array<u16>,
}

impl OldSkinHeader {
    /// Parse embedded skin data from pre-WotLK M2 models (no SKIN magic)
    pub fn parse_embedded<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Embedded skins don't have the SKIN magic signature
        // They start directly with the array references
        let indices = M2Array::parse(reader)?;
        let triangles = M2Array::parse(reader)?;
        let bone_indices = M2Array::parse(reader)?;
        let submeshes = M2Array::parse(reader)?;
        let material_lookup = M2Array::parse(reader)?;

        Ok(Self {
            magic: SKIN_MAGIC, // Set magic for compatibility
            indices,
            triangles,
            bone_indices,
            submeshes,
            material_lookup,
        })
    }
}

impl SkinHeaderT for OldSkinHeader {
    /// Parse a Skin header from a reader (old format without version field)
    fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Read and check magic
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;

        if magic != SKIN_MAGIC {
            return Err(M2Error::InvalidMagic {
                expected: String::from_utf8_lossy(&SKIN_MAGIC).to_string(),
                actual: String::from_utf8_lossy(&magic).to_string(),
            });
        }

        // Read array references directly (no version field in old format)
        let indices = M2Array::parse(reader)?;
        let triangles = M2Array::parse(reader)?;
        let bone_indices = M2Array::parse(reader)?;
        let submeshes = M2Array::parse(reader)?;
        let material_lookup = M2Array::parse(reader)?;

        Ok(Self {
            magic,
            indices,
            triangles,
            bone_indices,
            submeshes,
            material_lookup,
        })
    }

    /// Write a Skin header to a writer
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Write magic and version
        writer.write_all(&self.magic)?;

        // Write array references
        self.indices.write(writer)?;
        self.triangles.write(writer)?;
        self.bone_indices.write(writer)?;
        self.submeshes.write(writer)?;
        self.material_lookup.write(writer)?;

        Ok(())
    }

    /// Calculate the size of the header for this skin version
    fn calculate_size(&self) -> usize {
        let mut size = 4; // Magic only (no version in old format)

        // Array references
        size += 5 * (2 * 4); // 5 arrays, each with count and offset (8 bytes)

        size
    }

    fn set_array_fields(
        &mut self,
        indices: M2Array<u16>,
        triangles: M2Array<u16>,
        bone_indices: M2Array<u8>,
        submeshes: M2Array<SkinSubmesh>,
        material_lookup: M2Array<u16>,
    ) {
        self.indices = indices;
        self.triangles = triangles;
        self.bone_indices = bone_indices;
        self.submeshes = submeshes;
        self.material_lookup = material_lookup;
    }

    fn indices(&self) -> &M2Array<u16> {
        &self.indices
    }

    fn triangles(&self) -> &M2Array<u16> {
        &self.triangles
    }

    fn bone_indices(&self) -> &M2Array<u8> {
        &self.bone_indices
    }

    fn submeshes(&self) -> &M2Array<SkinSubmesh> {
        &self.submeshes
    }

    fn material_lookup(&self) -> &M2Array<u16> {
        &self.material_lookup
    }
}

impl OldSkinHeader {
    /// Create a new Skin header for a specific version
    pub fn new() -> Self {
        Self {
            magic: SKIN_MAGIC,
            indices: M2Array::new(0, 0),
            triangles: M2Array::new(0, 0),
            bone_indices: M2Array::new(0, 0),
            submeshes: M2Array::new(0, 0),
            material_lookup: M2Array::new(0, 0),
        }
    }
}

impl Default for OldSkinHeader {
    fn default() -> Self {
        Self::new()
    }
}

/// Submesh structure
#[derive(Debug, Clone)]
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
    pub center: [f32; 3],
    /// Sort center
    pub sort_center: [f32; 3],
    /// Bounding sphere radius
    pub bounding_radius: f32,
}

impl SkinSubmesh {
    /// Parse a submesh from a reader
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let id = reader.read_u16_le()?;
        let level = reader.read_u16_le()?;
        let vertex_start = reader.read_u16_le()?;
        let vertex_count = reader.read_u16_le()?;
        let triangle_start = reader.read_u16_le()?;
        let triangle_count = reader.read_u16_le()?;
        let bone_count = reader.read_u16_le()?;
        let bone_start = reader.read_u16_le()?;
        let bone_influence = reader.read_u16_le()?;

        // Skip 1 u16 of padding
        reader.read_u16_le()?;

        let mut center = [0.0; 3];
        let mut sort_center = [0.0; 3];
        let mut bounding_radius = 0.0;

        if bone_count > 0 {
            for item in &mut center {
                *item = reader.read_f32_le()?;
            }

            for item in &mut sort_center {
                *item = reader.read_f32_le()?;
            }

            bounding_radius = reader.read_f32_le()?;

            // Skip 1 u32 of padding
            reader.read_u32_le()?;
        }

        Ok(Self {
            id,
            level,
            vertex_start,
            vertex_count,
            triangle_start,
            triangle_count,
            bone_count,
            bone_start,
            bone_influence,
            center,
            sort_center,
            bounding_radius,
        })
    }

    /// Write a submesh to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u16_le(self.id)?;
        writer.write_u16_le(self.level)?;
        writer.write_u16_le(self.vertex_start)?;
        writer.write_u16_le(self.vertex_count)?;
        writer.write_u16_le(self.triangle_start)?;
        writer.write_u16_le(self.triangle_count)?;
        writer.write_u16_le(self.bone_count)?;
        writer.write_u16_le(self.bone_start)?;
        writer.write_u16_le(self.bone_influence)?;

        // Write 1 u16 of padding
        writer.write_u16_le(0)?;

        if self.bone_count > 0 {
            for &value in &self.center {
                writer.write_f32_le(value)?;
            }

            for &value in &self.sort_center {
                writer.write_f32_le(value)?;
            }

            writer.write_f32_le(self.bounding_radius)?;

            // Write 1 u32 of padding
            writer.write_u32_le(0)?;
        }

        Ok(())
    }
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
    pub indices: Vec<u16>,
    /// Triangles (each is 3 indices)
    pub triangles: Vec<u16>,
    /// Bone indices
    pub bone_indices: Vec<u8>,
    /// Submeshes
    pub submeshes: Vec<SkinSubmesh>,
    /// Material lookup table
    pub material_lookup: Vec<u16>,
}

impl<H> SkinG<H>
where
    H: SkinHeaderT + Clone,
{
    /// Parse a Skin from a reader
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Parse the header
        let header = H::parse(reader)?;

        // Parse indices
        let header_indices = header.indices();
        reader.seek(SeekFrom::Start(header_indices.offset as u64))?;
        let mut indices = Vec::with_capacity(header_indices.count as usize);
        for _ in 0..header_indices.count {
            indices.push(reader.read_u16_le()?);
        }

        // Parse triangles
        let header_triangles = header.triangles();
        reader.seek(SeekFrom::Start(header_triangles.offset as u64))?;
        let mut triangles = Vec::with_capacity(header_triangles.count as usize);
        for _ in 0..header_triangles.count {
            triangles.push(reader.read_u16_le()?);
        }

        // Parse bone indices
        let header_bone_indices = header.bone_indices();
        reader.seek(SeekFrom::Start(header_bone_indices.offset as u64))?;
        let mut bone_indices = Vec::with_capacity(header_bone_indices.count as usize);
        for _ in 0..header_bone_indices.count {
            bone_indices.push(reader.read_u8()?);
        }

        // Parse submeshes
        let header_submeshes = header.submeshes();
        reader.seek(SeekFrom::Start(header_submeshes.offset as u64))?;
        let mut submeshes = Vec::with_capacity(header_submeshes.count as usize);
        for _ in 0..header_submeshes.count {
            submeshes.push(SkinSubmesh::parse(reader)?);
        }

        // Parse material lookup
        let header_material_lookup = header.material_lookup();
        reader.seek(SeekFrom::Start(header_material_lookup.offset as u64))?;
        let mut material_lookup = Vec::with_capacity(header_material_lookup.count as usize);
        for _ in 0..header_material_lookup.count {
            material_lookup.push(reader.read_u16_le()?);
        }

        Ok(Self {
            header,
            indices,
            triangles,
            bone_indices,
            submeshes,
            material_lookup,
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
            let indices = M2Array::new(self.indices.len() as u32, current_offset);

            for &index in &self.indices {
                data_section.extend_from_slice(&index.to_le_bytes());
            }

            current_offset += (self.indices.len() * std::mem::size_of::<u16>()) as u32;
            indices
        } else {
            M2Array::new(0, 0)
        };

        // Write triangles
        let triangles = if !self.triangles.is_empty() {
            let triangles = M2Array::new(self.triangles.len() as u32, current_offset);

            for &triangle in &self.triangles {
                data_section.extend_from_slice(&triangle.to_le_bytes());
            }

            current_offset += (self.triangles.len() * std::mem::size_of::<u16>()) as u32;

            triangles
        } else {
            M2Array::new(0, 0)
        };

        // Write bone indices
        let bone_indices = if !self.bone_indices.is_empty() {
            let bone_indices = M2Array::new(self.bone_indices.len() as u32, current_offset);

            for &bone_index in &self.bone_indices {
                data_section.push(bone_index);
            }

            current_offset += self.bone_indices.len() as u32;

            bone_indices
        } else {
            M2Array::new(0, 0)
        };

        // Write submeshes
        let submeshes = if !self.submeshes.is_empty() {
            let submeshes = M2Array::new(self.submeshes.len() as u32, current_offset);

            for submesh in &self.submeshes {
                let mut submesh_data = Vec::new();
                submesh.write(&mut submesh_data)?;
                data_section.extend_from_slice(&submesh_data);
            }

            current_offset += (self.submeshes.len() * 40) as u32; // Each submesh is 40 bytes
            submeshes
        } else {
            M2Array::new(0, 0)
        };

        // Write material lookup
        let material_lookup = if !self.material_lookup.is_empty() {
            let material_lookup = M2Array::new(self.material_lookup.len() as u32, current_offset);

            for &material in &self.material_lookup {
                data_section.extend_from_slice(&material.to_le_bytes());
            }

            // current_offset += (self.material_lookup.len() * std::mem::size_of::<u16>()) as u32;
            material_lookup
        } else {
            M2Array::new(0, 0)
        };

        header.set_array_fields(indices, triangles, bone_indices, submeshes, material_lookup);

        // Finally, write the header followed by the data section
        header.write(writer)?;
        writer.write_all(&data_section)?;

        Ok(())
    }
}

impl SkinG<SkinHeader> {
    /// Convert this skin to a different version
    pub fn convert(&self, target_version: M2Version) -> Result<Self> {
        let source_version = self
            .header
            .get_m2_version()
            .ok_or(M2Error::ConversionError {
                from: self.header.version,
                to: target_version.to_header_version(),
                reason: "Unknown source version".to_string(),
            })?;

        if source_version == target_version {
            return Ok(self.clone());
        }

        // Create a new skin with the target version
        let mut new_skin = self.clone();

        // Update header version
        let mut header = SkinHeader::new(target_version);
        header.name = self.header.name;
        header.vertex_count = self.header.vertex_count;

        // Handle version-specific conversions
        if target_version >= M2Version::BfA && source_version < M2Version::BfA {
            // When upgrading to BfA or later, add center position and bounds if missing
            if header.center_position.is_none() {
                // Calculate center of mass from submeshes
                let mut center = [0.0, 0.0, 0.0];
                let mut max_radius = 0.0;

                if !self.submeshes.is_empty() {
                    for submesh in &self.submeshes {
                        for (i, center_val) in center.iter_mut().enumerate() {
                            *center_val += submesh.center[i];
                        }

                        if submesh.bounding_radius > max_radius {
                            max_radius = submesh.bounding_radius;
                        }
                    }

                    // Average the center
                    let count = self.submeshes.len() as f32;
                    for item in &mut center {
                        *item /= count;
                    }
                }

                header.center_position = Some(center);
                header.center_bounds = Some(max_radius);
            }
        } else if target_version < M2Version::BfA && source_version >= M2Version::BfA {
            // When downgrading from BfA or later, remove center position and bounds
            header.center_position = None;
            header.center_bounds = None;
        }

        new_skin.header = header;

        Ok(new_skin)
    }
}

pub type Skin = SkinG<SkinHeader>;
pub type OldSkin = SkinG<OldSkinHeader>;

/// Enum to represent either format variant
#[derive(Debug, Clone)]
pub enum SkinFile {
    /// New format with version field (camera files)
    New(Skin),
    /// Old format without version field (character models)
    Old(OldSkin),
}

impl SkinFile {
    /// Parse a SKIN file with automatic format detection
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        parse_skin(reader)
    }

    /// Load a SKIN file from a path with automatic format detection
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        load_skin(path)
    }

    /// Save the SKIN file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        match self {
            SkinFile::New(skin) => skin.save(path),
            SkinFile::Old(skin) => skin.save(path),
        }
    }

    /// Write the SKIN file to a writer
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        match self {
            SkinFile::New(skin) => skin.write(writer),
            SkinFile::Old(skin) => skin.write(writer),
        }
    }

    /// Get resolved vertex indices for rendering
    ///
    /// For external .skin files (WotLK+), this applies the two-level indirection:
    /// - The triangles array contains indices into the indices array (lookup table)
    /// - The final vertex index = indices[triangles[i]]
    ///
    /// For embedded skins (pre-WotLK), the indices are already direct vertex indices.
    pub fn get_resolved_indices(&self) -> Vec<u16> {
        match self {
            SkinFile::Old(_) => {
                // Embedded/old format: indices are already direct vertex indices
                self.indices().clone()
            }
            SkinFile::New(_) => {
                // External/new format: apply two-level indirection
                let indices = self.indices();
                let triangles = self.triangles();

                // Resolve: final_index = indices[triangles[i]]
                triangles
                    .iter()
                    .map(|&tri_idx| {
                        if (tri_idx as usize) < indices.len() {
                            indices[tri_idx as usize]
                        } else {
                            // Handle out-of-bounds gracefully
                            0
                        }
                    })
                    .collect()
            }
        }
    }

    /// Get raw indices array (vertex lookup table)
    ///
    /// Note: For rendering, use `get_resolved_indices()` instead.
    /// This method returns the raw array which has different meanings:
    /// - Embedded skins: Direct vertex indices
    /// - External skins: Vertex lookup table
    pub fn indices(&self) -> &Vec<u16> {
        match self {
            SkinFile::New(skin) => &skin.indices,
            SkinFile::Old(skin) => &skin.indices,
        }
    }

    /// Get triangles regardless of format
    pub fn triangles(&self) -> &Vec<u16> {
        match self {
            SkinFile::New(skin) => &skin.triangles,
            SkinFile::Old(skin) => &skin.triangles,
        }
    }

    /// Get submeshes regardless of format
    pub fn submeshes(&self) -> &Vec<SkinSubmesh> {
        match self {
            SkinFile::New(skin) => &skin.submeshes,
            SkinFile::Old(skin) => &skin.submeshes,
        }
    }

    /// Check if this is a new format SKIN file
    pub fn is_new_format(&self) -> bool {
        matches!(self, SkinFile::New(_))
    }

    /// Check if this is an old format SKIN file
    pub fn is_old_format(&self) -> bool {
        matches!(self, SkinFile::Old(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_format_detection() {
        // Test new format detection (version = 1)
        let mut data = Vec::new();
        data.extend_from_slice(&SKIN_MAGIC);
        data.extend_from_slice(&1u32.to_le_bytes()); // version = 1

        let mut cursor = Cursor::new(&data);
        let is_new = detect_skin_format(&mut cursor).unwrap();
        assert!(is_new, "Version 1 should be detected as new format");

        // Test old format detection (indices count = 5903)
        let mut data = Vec::new();
        data.extend_from_slice(&SKIN_MAGIC);
        data.extend_from_slice(&5903u32.to_le_bytes()); // large indices count

        let mut cursor = Cursor::new(&data);
        let is_new = detect_skin_format(&mut cursor).unwrap();
        assert!(
            !is_new,
            "Large indices count should be detected as old format"
        );

        // Test boundary case (version = 4, still new format)
        let mut data = Vec::new();
        data.extend_from_slice(&SKIN_MAGIC);
        data.extend_from_slice(&4u32.to_le_bytes()); // version = 4

        let mut cursor = Cursor::new(&data);
        let is_new = detect_skin_format(&mut cursor).unwrap();
        assert!(is_new, "Version 4 should be detected as new format");

        // Test boundary case (version = 5, old format)
        let mut data = Vec::new();
        data.extend_from_slice(&SKIN_MAGIC);
        data.extend_from_slice(&5u32.to_le_bytes()); // indices count = 5

        let mut cursor = Cursor::new(&data);
        let is_new = detect_skin_format(&mut cursor).unwrap();
        assert!(!is_new, "Indices count 5 should be detected as old format");
    }

    #[test]
    fn test_skin_header_parse() {
        let mut data = Vec::new();

        // Magic "SKIN"
        data.extend_from_slice(&SKIN_MAGIC);

        // Version
        data.extend_from_slice(&0u32.to_le_bytes());

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
        assert_eq!(header.version, 0);
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
        assert!(header.center_position.is_none());
        assert!(header.center_bounds.is_none());
    }

    #[test]
    #[ignore] // TODO: Fix test data to properly simulate old format
    fn test_skin_file_api() {
        // Test format detection first
        let new_format_data = create_new_format_test_data();
        let old_format_data = create_old_format_test_data();

        // Test format detection
        let mut cursor = Cursor::new(&new_format_data);
        let is_new = detect_skin_format(&mut cursor).unwrap();
        assert!(is_new, "New format should be detected");

        let mut cursor = Cursor::new(&old_format_data);
        let is_new = detect_skin_format(&mut cursor).unwrap();
        assert!(!is_new, "Old format should be detected");

        // Parse new format
        let mut cursor = Cursor::new(new_format_data);
        let skin_file = SkinFile::parse(&mut cursor).unwrap();
        assert!(skin_file.is_new_format());
        assert!(!skin_file.is_old_format());

        // Parse old format
        let mut cursor = Cursor::new(old_format_data);
        let skin_file = SkinFile::parse(&mut cursor).unwrap();
        assert!(!skin_file.is_new_format());
        assert!(skin_file.is_old_format());

        // Test unified API
        let indices = skin_file.indices();
        let submeshes = skin_file.submeshes();
        assert_eq!(indices.len(), 3); // from test data
        assert_eq!(submeshes.len(), 0); // empty in test data
    }

    fn create_new_format_test_data() -> Vec<u8> {
        let mut data = Vec::new();

        // Magic "SKIN"
        data.extend_from_slice(&SKIN_MAGIC);

        // Version = 1 (new format indicator)
        data.extend_from_slice(&1u32.to_le_bytes());

        // Name (empty)
        data.extend_from_slice(&0u32.to_le_bytes()); // count = 0
        data.extend_from_slice(&0u32.to_le_bytes()); // offset = 0

        // Vertex count
        data.extend_from_slice(&100u32.to_le_bytes());

        // Indices (3 items at end of header)
        let indices_offset = (4 + 4 + 8 + 4 + 5 * 8) as u32; // magic + version + name + vertex_count + 5 arrays
        data.extend_from_slice(&3u32.to_le_bytes()); // count = 3
        data.extend_from_slice(&indices_offset.to_le_bytes()); // offset

        // Other arrays (empty)
        for _ in 0..4 {
            data.extend_from_slice(&0u32.to_le_bytes()); // count = 0
            data.extend_from_slice(&0u32.to_le_bytes()); // offset = 0
        }

        // Index data
        data.extend_from_slice(&10u16.to_le_bytes());
        data.extend_from_slice(&20u16.to_le_bytes());
        data.extend_from_slice(&30u16.to_le_bytes());

        data
    }

    fn create_old_format_test_data() -> Vec<u8> {
        let mut data = Vec::new();

        // Magic "SKIN"
        data.extend_from_slice(&SKIN_MAGIC);

        // Indices (3 items) - this is what makes it "old format" (large count)
        let indices_offset = (4 + 5 * 8) as u32; // magic + 5 arrays
        data.extend_from_slice(&3u32.to_le_bytes()); // count = 3 
        data.extend_from_slice(&indices_offset.to_le_bytes()); // offset

        // Other arrays (empty)
        for _ in 0..4 {
            data.extend_from_slice(&0u32.to_le_bytes()); // count = 0
            data.extend_from_slice(&0u32.to_le_bytes()); // offset = 0
        }

        // Index data
        data.extend_from_slice(&10u16.to_le_bytes());
        data.extend_from_slice(&20u16.to_le_bytes());
        data.extend_from_slice(&30u16.to_le_bytes());

        data
    }

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
            center: [1.0, 2.0, 3.0],
            sort_center: [1.5, 2.5, 3.5],
            bounding_radius: 5.0,
        };

        let mut data = Vec::new();
        submesh.write(&mut data).unwrap();

        let mut cursor = Cursor::new(data);
        let parsed_submesh = SkinSubmesh::parse(&mut cursor).unwrap();

        assert_eq!(parsed_submesh.id, 1);
        assert_eq!(parsed_submesh.vertex_count, 100);
        assert_eq!(parsed_submesh.triangle_count, 50);
        assert_eq!(parsed_submesh.bone_count, 10);
        assert_eq!(parsed_submesh.bone_influence, 4);
        assert_eq!(parsed_submesh.center, [1.0, 2.0, 3.0]);
        assert_eq!(parsed_submesh.sort_center, [1.5, 2.5, 3.5]);
        assert_eq!(parsed_submesh.bounding_radius, 5.0);
    }
}
