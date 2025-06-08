//! Core types for the WDL file format

use std::collections::HashMap;
use std::io::{self, Read, Seek, Write};

use crate::error::{Result, WdlError};
use crate::version::WdlVersion;

/// Magic number for the MVER chunk (Version)
pub const MVER_MAGIC: [u8; 4] = [b'R', b'E', b'V', b'M'];
/// Magic number for the MWMO chunk (WMO Filenames)
pub const MWMO_MAGIC: [u8; 4] = [b'O', b'M', b'W', b'M'];
/// Magic number for the MWID chunk (WMO Filename Offsets)
pub const MWID_MAGIC: [u8; 4] = [b'D', b'I', b'W', b'M'];
/// Magic number for the MODF chunk (WMO Placement Data)
pub const MODF_MAGIC: [u8; 4] = [b'F', b'D', b'O', b'M'];
/// Magic number for the MAOF chunk (Map Area Offset Table)
pub const MAOF_MAGIC: [u8; 4] = [b'F', b'O', b'A', b'M'];
/// Magic number for the MARE chunk (Map Area Low-Resolution Heights)
pub const MARE_MAGIC: [u8; 4] = [b'E', b'R', b'A', b'M'];
/// Magic number for the MAHO chunk (Map Area Holes)
pub const MAHO_MAGIC: [u8; 4] = [b'O', b'H', b'A', b'M'];
/// Magic number for the MLDD chunk (M2 doodad placement, Legion+)
pub const MLDD_MAGIC: [u8; 4] = [b'D', b'D', b'L', b'M'];
/// Magic number for the MLDX chunk (M2 doodad visibility, Legion+)
pub const MLDX_MAGIC: [u8; 4] = [b'X', b'D', b'L', b'M'];
/// Magic number for the MLMD chunk (WMO placement, Legion+)
pub const MLMD_MAGIC: [u8; 4] = [b'D', b'M', b'L', b'M'];
/// Magic number for the MLMX chunk (WMO visibility, Legion+)
pub const MLMX_MAGIC: [u8; 4] = [b'X', b'M', b'L', b'M'];

/// Vector 3D type used in WoW files
#[derive(Debug, Clone, PartialEq)]
pub struct Vec3d {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
    /// Z coordinate
    pub z: f32,
}

impl Vec3d {
    /// Creates a new 3D vector
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Creates a vector at origin (0, 0, 0)
    pub fn origin() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Reads a Vec3d from a reader
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        let x = f32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let y = f32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let z = f32::from_le_bytes(buf);
        Ok(Self::new(x, y, z))
    }

    /// Writes a Vec3d to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.x.to_le_bytes())?;
        writer.write_all(&self.y.to_le_bytes())?;
        writer.write_all(&self.z.to_le_bytes())?;
        Ok(())
    }
}

/// Bounding box used in WoW files
#[derive(Debug, Clone, PartialEq)]
pub struct BoundingBox {
    /// Minimum corner of the bounding box
    pub min: Vec3d,
    /// Maximum corner of the bounding box
    pub max: Vec3d,
}

impl BoundingBox {
    /// Creates a new bounding box
    pub fn new(min: Vec3d, max: Vec3d) -> Self {
        Self { min, max }
    }

    /// Creates a default bounding box at origin with no size
    pub fn zero() -> Self {
        Self::new(Vec3d::origin(), Vec3d::origin())
    }

    /// Reads a BoundingBox from a reader
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let min = Vec3d::read(reader)?;
        let max = Vec3d::read(reader)?;
        Ok(Self::new(min, max))
    }

    /// Writes a BoundingBox to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.min.write(writer)?;
        self.max.write(writer)?;
        Ok(())
    }
}

/// A chunk in a WDL file
#[derive(Debug, Clone)]
pub struct Chunk {
    /// The four-character identifier for this chunk
    pub magic: [u8; 4],
    /// The size of the data in this chunk
    pub size: u32,
    /// The data contained in this chunk
    pub data: Vec<u8>,
}

impl Chunk {
    /// Creates a new chunk with the specified magic, size, and data
    pub fn new(magic: [u8; 4], data: Vec<u8>) -> Self {
        let size = data.len() as u32;
        Self { magic, size, data }
    }

    /// Reads a chunk from a reader
    pub fn read<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
        let mut magic = [0u8; 4];
        if reader.read_exact(&mut magic).is_err() {
            // We've reached EOF
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "unexpected end of file",
            ));
        }

        let mut size_buf = [0u8; 4];
        reader.read_exact(&mut size_buf)?;
        let size = u32::from_le_bytes(size_buf);
        let mut data = vec![0u8; size as usize];
        reader.read_exact(&mut data)?;

        Ok(Self { magic, size, data })
    }

    /// Writes a chunk to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.magic)?;
        writer.write_all(&self.size.to_le_bytes())?;
        writer.write_all(&self.data)?;
        Ok(())
    }

    /// Returns a string representation of the magic value
    pub fn magic_str(&self) -> String {
        String::from_utf8_lossy(&self.magic).to_string()
    }
}

/// Model placement information (MODF chunk data)
#[derive(Debug, Clone)]
pub struct ModelPlacement {
    /// Unique ID for this instance
    pub id: u32,
    /// Referenced WMO ID
    pub wmo_id: u32,
    /// Position in world
    pub position: Vec3d,
    /// Rotation vector (radians)
    pub rotation: Vec3d,
    /// Bounds information
    pub bounds: BoundingBox,
    /// Flags
    pub flags: u16,
    /// Doodad set
    pub doodad_set: u16,
    /// Name set
    pub name_set: u16,
    /// Padding/reserved value
    pub padding: u16,
}

impl ModelPlacement {
    /// Reads a ModelPlacement from a reader
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut buf4 = [0u8; 4];
        let mut buf2 = [0u8; 2];

        reader.read_exact(&mut buf4)?;
        let wmo_id = u32::from_le_bytes(buf4);
        reader.read_exact(&mut buf4)?;
        let id = u32::from_le_bytes(buf4);
        let position = Vec3d::read(reader)?;
        let rotation = Vec3d::read(reader)?;
        let bounds = BoundingBox::read(reader)?;
        reader.read_exact(&mut buf2)?;
        let flags = u16::from_le_bytes(buf2);
        reader.read_exact(&mut buf2)?;
        let doodad_set = u16::from_le_bytes(buf2);
        reader.read_exact(&mut buf2)?;
        let name_set = u16::from_le_bytes(buf2);

        reader.read_exact(&mut buf2)?;
        let padding = u16::from_le_bytes(buf2);

        Ok(Self {
            id,
            wmo_id,
            position,
            rotation,
            bounds,
            flags,
            doodad_set,
            name_set,
            padding,
        })
    }

    /// Writes a ModelPlacement to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.wmo_id.to_le_bytes())?;
        writer.write_all(&self.id.to_le_bytes())?;
        self.position.write(writer)?;
        self.rotation.write(writer)?;
        self.bounds.write(writer)?;
        writer.write_all(&self.flags.to_le_bytes())?;
        writer.write_all(&self.doodad_set.to_le_bytes())?;
        writer.write_all(&self.name_set.to_le_bytes())?;

        writer.write_all(&self.padding.to_le_bytes())?;

        Ok(())
    }
}

/// M2 Model placement information (MLDD chunk data in Legion+)
#[derive(Debug, Clone)]
pub struct M2Placement {
    /// Unique ID for this instance
    pub id: u32,
    /// Referenced M2 file data ID
    pub m2_id: u32,
    /// Position in world
    pub position: Vec3d,
    /// Rotation vector (radians)
    pub rotation: Vec3d,
    /// Scale factor
    pub scale: f32,
    /// Flags
    pub flags: u32,
}

impl M2Placement {
    /// Reads an M2Placement from a reader
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut buf = [0u8; 4];

        reader.read_exact(&mut buf)?;
        let id = u32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let m2_id = u32::from_le_bytes(buf);
        let position = Vec3d::read(reader)?;
        let rotation = Vec3d::read(reader)?;
        reader.read_exact(&mut buf)?;
        let scale = f32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let flags = u32::from_le_bytes(buf);

        Ok(Self {
            id,
            m2_id,
            position,
            rotation,
            scale,
            flags,
        })
    }

    /// Writes an M2Placement to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.id.to_le_bytes())?;
        writer.write_all(&self.m2_id.to_le_bytes())?;
        self.position.write(writer)?;
        self.rotation.write(writer)?;
        writer.write_all(&self.scale.to_le_bytes())?;
        writer.write_all(&self.flags.to_le_bytes())?;

        Ok(())
    }
}

/// WMO Model visibility info (MLDX chunk data in Legion+)
#[derive(Debug, Clone)]
pub struct M2VisibilityInfo {
    /// Bounding box for visibility check
    pub bounds: BoundingBox,
    /// Visibility radius
    pub radius: f32,
}

impl M2VisibilityInfo {
    /// Reads an M2VisibilityInfo from a reader
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let bounds = BoundingBox::read(reader)?;
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        let radius = f32::from_le_bytes(buf);

        Ok(Self { bounds, radius })
    }

    /// Writes an M2VisibilityInfo to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.bounds.write(writer)?;
        writer.write_all(&self.radius.to_le_bytes())?;

        Ok(())
    }
}

/// Height map data for a single map tile (MARE chunk)
///
/// WDL height data provides low-resolution terrain heights for each ADT tile.
/// The data consists of 545 signed 16-bit integers total:
/// - 17x17 outer grid (289 values) - vertices at chunk corners
/// - 16x16 inner grid (256 values) - vertices at chunk centers
///
/// This matches the vertex layout of full ADT heightmaps but at lower resolution.
#[derive(Debug, Clone)]
pub struct HeightMapTile {
    /// Outer heightmap values (17x17 grid)
    /// These represent the height values at the corners of each chunk
    pub outer_values: Vec<i16>,
    /// Inner heightmap values (16x16 grid)
    /// These represent the height values at the centers of each chunk
    pub inner_values: Vec<i16>,
}

impl Default for HeightMapTile {
    fn default() -> Self {
        Self::new()
    }
}

impl HeightMapTile {
    /// Number of outer height map values (17x17)
    pub const OUTER_COUNT: usize = 17 * 17;
    /// Number of inner height map values (16x16)
    pub const INNER_COUNT: usize = 16 * 16;
    /// Total count of height map values (545)
    pub const TOTAL_COUNT: usize = Self::OUTER_COUNT + Self::INNER_COUNT;

    /// Creates a new HeightMapTile with default values (all zeroes)
    pub fn new() -> Self {
        Self {
            outer_values: vec![0; Self::OUTER_COUNT],
            inner_values: vec![0; Self::INNER_COUNT],
        }
    }

    /// Reads a HeightMapTile from a reader
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut outer_values = Vec::with_capacity(Self::OUTER_COUNT);
        let mut buf = [0u8; 2];
        for _ in 0..Self::OUTER_COUNT {
            reader.read_exact(&mut buf)?;
            outer_values.push(i16::from_le_bytes(buf));
        }

        let mut inner_values = Vec::with_capacity(Self::INNER_COUNT);
        for _ in 0..Self::INNER_COUNT {
            reader.read_exact(&mut buf)?;
            inner_values.push(i16::from_le_bytes(buf));
        }

        Ok(Self {
            outer_values,
            inner_values,
        })
    }

    /// Writes a HeightMapTile to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        for value in &self.outer_values {
            writer.write_all(&value.to_le_bytes())?;
        }

        for value in &self.inner_values {
            writer.write_all(&value.to_le_bytes())?;
        }

        Ok(())
    }
}

/// Holes data for a map tile (MAHO chunk)
///
/// Represents terrain holes in a 16x16 chunk grid for an ADT tile.
/// Each bit in the bitmask represents whether a chunk has a hole:
/// - 0 = hole present
/// - 1 = no hole (solid terrain)
#[derive(Debug, Clone)]
pub struct HolesData {
    /// Bitmasks for holes (16 uint16 values, one per row)
    /// Each uint16 represents 16 chunks in a row (bits 0-15 = chunks 0-15)
    pub hole_masks: [u16; 16],
}

impl Default for HolesData {
    fn default() -> Self {
        Self::new()
    }
}

impl HolesData {
    /// Number of hole mask values
    pub const MASK_COUNT: usize = 16;

    /// Creates a new HolesData with no holes (all 1s)
    pub fn new() -> Self {
        Self {
            hole_masks: [0xFFFF; Self::MASK_COUNT],
        }
    }

    /// Creates a new HolesData with all holes (all 0s)
    pub fn all_holes() -> Self {
        Self {
            hole_masks: [0; Self::MASK_COUNT],
        }
    }

    /// Reads HolesData from a reader
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut hole_masks = [0u16; Self::MASK_COUNT];
        let mut buf = [0u8; 2];
        for mask in &mut hole_masks {
            reader.read_exact(&mut buf)?;
            *mask = u16::from_le_bytes(buf);
        }

        Ok(Self { hole_masks })
    }

    /// Writes HolesData to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        for mask in &self.hole_masks {
            writer.write_all(&mask.to_le_bytes())?;
        }

        Ok(())
    }

    /// Checks if a specific chunk has a hole
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate (0-15)
    /// * `y` - Y coordinate (0-15)
    ///
    /// # Returns
    ///
    /// `true` if there is a hole, `false` if there is no hole
    pub fn has_hole(&self, x: usize, y: usize) -> bool {
        if x >= 16 || y >= 16 {
            return false;
        }

        let mask = self.hole_masks[y];
        (mask & (1 << x)) == 0
    }

    /// Sets whether a specific chunk has a hole
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate (0-15)
    /// * `y` - Y coordinate (0-15)
    /// * `has_hole` - Whether the chunk should have a hole
    pub fn set_hole(&mut self, x: usize, y: usize, has_hole: bool) {
        if x >= 16 || y >= 16 {
            return;
        }

        if has_hole {
            // Clear the bit to create a hole
            self.hole_masks[y] &= !(1 << x);
        } else {
            // Set the bit to remove a hole
            self.hole_masks[y] |= 1 << x;
        }
    }
}

/// Main WDL file representation
#[derive(Debug)]
pub struct WdlFile {
    /// Version information
    pub version: WdlVersion,
    /// Version number from MVER chunk
    pub version_number: u32,
    /// Map tile offsets (MAOF chunk)
    /// Contains 4096 (64x64) absolute file offsets to MapAreaLow entries
    /// Zero values indicate tiles without low-resolution data
    pub map_tile_offsets: [u32; 64 * 64],
    /// Heightmap tiles (MARE chunks)
    pub heightmap_tiles: HashMap<(u32, u32), HeightMapTile>,
    /// Holes data (MAHO chunks)
    pub holes_data: HashMap<(u32, u32), HolesData>,
    /// WMO filenames (MWMO chunk)
    pub wmo_filenames: Vec<String>,
    /// WMO filename offsets into MWMO data (MWID chunk)
    pub wmo_indices: Vec<u32>,
    /// WMO placements (MODF chunk)
    pub wmo_placements: Vec<ModelPlacement>,
    /// M2 placements (MLDD chunk, Legion+)
    pub m2_placements: Vec<M2Placement>,
    /// M2 visibility info (MLDX chunk, Legion+)
    pub m2_visibility: Vec<M2VisibilityInfo>,
    /// WMO placements (MLMD chunk, Legion+)
    pub wmo_legion_placements: Vec<M2Placement>,
    /// WMO visibility info (MLMX chunk, Legion+)
    pub wmo_legion_visibility: Vec<M2VisibilityInfo>,
    /// All chunks in the file, in order
    pub chunks: Vec<Chunk>,
}

impl WdlFile {
    /// Creates a new empty WDL file
    pub fn new() -> Self {
        Self {
            version: WdlVersion::default(),
            version_number: WdlVersion::default().version_number(),
            map_tile_offsets: [0; 64 * 64],
            heightmap_tiles: HashMap::new(),
            holes_data: HashMap::new(),
            wmo_filenames: Vec::new(),
            wmo_indices: Vec::new(),
            wmo_placements: Vec::new(),
            m2_placements: Vec::new(),
            m2_visibility: Vec::new(),
            wmo_legion_placements: Vec::new(),
            wmo_legion_visibility: Vec::new(),
            chunks: Vec::new(),
        }
    }

    /// Creates a new WDL file with the specified version
    pub fn with_version(version: WdlVersion) -> Self {
        let mut file = Self::new();
        file.version = version;
        file.version_number = version.version_number();
        file
    }

    /// Validates the WDL file
    pub fn validate(&self) -> Result<()> {
        // Check version
        if self.version_number != self.version.version_number() {
            return Err(WdlError::ValidationError(format!(
                "Version number mismatch: file has {}, expected {}",
                self.version_number,
                self.version.version_number()
            )));
        }

        // MWID contains offsets into MWMO data, not indices into the filename array
        // So we don't validate them here

        // TODO: Add more validation as needed

        Ok(())
    }

    /// Converts the WDL file to another version
    pub fn convert_to(&self, target_version: WdlVersion) -> Result<Self> {
        // Create a new file with the target version
        let mut new_file = WdlFile::with_version(target_version);

        // Copy basic data
        new_file.map_tile_offsets = self.map_tile_offsets;
        new_file.heightmap_tiles = self.heightmap_tiles.clone();
        new_file.holes_data = self.holes_data.clone();

        // Handle version-specific chunks
        if target_version.has_wmo_chunks() {
            new_file.wmo_filenames = self.wmo_filenames.clone();
            new_file.wmo_indices = self.wmo_indices.clone();
            new_file.wmo_placements = self.wmo_placements.clone();
        }

        if target_version.has_ml_chunks() {
            new_file.m2_placements = self.m2_placements.clone();
            new_file.m2_visibility = self.m2_visibility.clone();
            new_file.wmo_legion_placements = self.wmo_legion_placements.clone();
            new_file.wmo_legion_visibility = self.wmo_legion_visibility.clone();

            // If we're converting from a pre-Legion format to Legion+,
            // we need to convert the WMO data to Legion format
            if !self.version.has_ml_chunks() && self.version.has_wmo_chunks() {
                // TODO: Implement conversion from WMO to Legion format
            }
        }

        Ok(new_file)
    }
}

impl Default for WdlFile {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3d() {
        let vec = Vec3d::new(1.0, 2.0, 3.0);
        assert_eq!(vec.x, 1.0);
        assert_eq!(vec.y, 2.0);
        assert_eq!(vec.z, 3.0);

        let origin = Vec3d::origin();
        assert_eq!(origin.x, 0.0);
        assert_eq!(origin.y, 0.0);
        assert_eq!(origin.z, 0.0);
    }

    #[test]
    fn test_bounding_box() {
        let min = Vec3d::new(1.0, 2.0, 3.0);
        let max = Vec3d::new(4.0, 5.0, 6.0);
        let bbox = BoundingBox::new(min.clone(), max.clone());

        assert_eq!(bbox.min, min);
        assert_eq!(bbox.max, max);

        let zero = BoundingBox::zero();
        assert_eq!(zero.min, Vec3d::origin());
        assert_eq!(zero.max, Vec3d::origin());
    }

    #[test]
    fn test_holes_data() {
        let mut holes = HolesData::new();

        // By default, no holes
        for x in 0..16 {
            for y in 0..16 {
                assert!(!holes.has_hole(x, y));
            }
        }

        // Set a hole
        holes.set_hole(5, 7, true);
        assert!(holes.has_hole(5, 7));

        // Remove the hole
        holes.set_hole(5, 7, false);
        assert!(!holes.has_hole(5, 7));

        // Test all holes
        let all_holes = HolesData::all_holes();
        for x in 0..16 {
            for y in 0..16 {
                assert!(all_holes.has_hole(x, y));
            }
        }
    }
}
