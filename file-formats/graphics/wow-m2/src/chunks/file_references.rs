//! File reference chunks for M2 chunked format (MD21+)
//!
//! These chunks contain FileDataID references to external files that are
//! loaded separately from the main M2 model.

use std::io::{Read, Seek};

use crate::chunks::infrastructure::ChunkReader;
use crate::error::Result;
use crate::io_ext::ReadExt;

/// SFID chunk - Skin File References
/// Contains an array of FileDataIDs for skin files (.skin files)
#[derive(Debug, Clone)]
pub struct SkinFileIds {
    /// Array of FileDataIDs for skin files
    pub ids: Vec<u32>,
}

impl SkinFileIds {
    /// Read SFID chunk data from a chunk reader
    pub fn read<R: Read + Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        let count = reader.chunk_size() / 4; // Each ID is 4 bytes
        let mut ids = Vec::with_capacity(count as usize);

        for _ in 0..count {
            ids.push(reader.read_u32_le()?);
        }

        Ok(SkinFileIds { ids })
    }

    /// Get the number of skin file IDs
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Check if there are no skin file IDs
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// Get a skin file ID by index
    pub fn get(&self, index: usize) -> Option<u32> {
        self.ids.get(index).copied()
    }

    /// Iterate over all skin file IDs
    pub fn iter(&self) -> std::slice::Iter<u32> {
        self.ids.iter()
    }
}

/// AFID chunk - Animation File References
/// Contains an array of FileDataIDs for external animation files (.anim files)
#[derive(Debug, Clone)]
pub struct AnimationFileIds {
    /// Array of FileDataIDs for animation files
    pub ids: Vec<u32>,
}

impl AnimationFileIds {
    /// Read AFID chunk data from a chunk reader
    pub fn read<R: Read + Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        let count = reader.chunk_size() / 4; // Each ID is 4 bytes
        let mut ids = Vec::with_capacity(count as usize);

        for _ in 0..count {
            ids.push(reader.read_u32_le()?);
        }

        Ok(AnimationFileIds { ids })
    }

    /// Get the number of animation file IDs
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Check if there are no animation file IDs
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// Get an animation file ID by index
    pub fn get(&self, index: usize) -> Option<u32> {
        self.ids.get(index).copied()
    }

    /// Iterate over all animation file IDs
    pub fn iter(&self) -> std::slice::Iter<u32> {
        self.ids.iter()
    }
}

/// TXID chunk - Texture File References
/// Contains an array of FileDataIDs for texture files (.blp files)
#[derive(Debug, Clone)]
pub struct TextureFileIds {
    /// Array of FileDataIDs for texture files
    pub ids: Vec<u32>,
}

/// PFID chunk - Physics File Reference
/// Contains a single FileDataID for the physics file (.phys file)
#[derive(Debug, Clone)]
pub struct PhysicsFileId {
    /// FileDataID for the physics file
    pub id: u32,
}

/// SKID chunk - Skeleton File Reference
/// Contains a single FileDataID for the skeleton file (.skel file)
#[derive(Debug, Clone)]
pub struct SkeletonFileId {
    /// FileDataID for the skeleton file
    pub id: u32,
}

/// BFID chunk - Bone File References
/// Contains an array of FileDataIDs for bone files (.bone files)
#[derive(Debug, Clone)]
pub struct BoneFileIds {
    /// Array of FileDataIDs for bone files
    pub ids: Vec<u32>,
}

/// LDV1 chunk - Level of Detail Configuration
/// Contains level of detail settings and skin file assignments
#[derive(Debug, Clone)]
pub struct LodData {
    /// LOD levels configuration
    pub levels: Vec<LodLevel>,
}

/// Single LOD level configuration
#[derive(Debug, Clone)]
pub struct LodLevel {
    /// Distance threshold for this LOD level
    pub distance: f32,
    /// Skin file index to use for this LOD level
    pub skin_file_index: u16,
    /// Number of vertices at this LOD level
    pub vertex_count: u32,
    /// Number of triangles at this LOD level
    pub triangle_count: u32,
}

/// Basic physics data structure for .phys files
#[derive(Debug, Clone)]
pub struct PhysicsData {
    /// Collision mesh data (if present)
    pub collision_mesh: Option<CollisionMesh>,
    /// Physics material properties
    pub material_properties: Vec<PhysicsMaterial>,
}

/// Collision mesh for physics calculations
#[derive(Debug, Clone)]
pub struct CollisionMesh {
    /// Collision vertices
    pub vertices: Vec<[f32; 3]>,
    /// Collision triangles (vertex indices)
    pub triangles: Vec<[u16; 3]>,
}

/// Physics material properties
#[derive(Debug, Clone)]
pub struct PhysicsMaterial {
    /// Material identifier
    pub material_id: u32,
    /// Friction coefficient
    pub friction: f32,
    /// Restitution (bounciness)
    pub restitution: f32,
    /// Density
    pub density: f32,
}

/// Basic skeleton data structure for .skel files
#[derive(Debug, Clone)]
pub struct SkeletonData {
    /// Bone hierarchy nodes
    pub bone_hierarchy: Vec<BoneNode>,
    /// Root bone indices
    pub root_bones: Vec<u16>,
}

/// Bone node in skeleton hierarchy
#[derive(Debug, Clone)]
pub struct BoneNode {
    /// Bone index in the model
    pub bone_index: u16,
    /// Parent bone index (-1 if root)
    pub parent_index: i16,
    /// Children bone indices
    pub children: Vec<u16>,
    /// Local transform relative to parent
    pub local_transform: [f32; 16], // 4x4 matrix
}

/// Basic bone data structure for .bone files
#[derive(Debug, Clone)]
pub struct BoneData {
    /// Bone index this data applies to
    pub bone_index: u16,
    /// Translation animation track
    pub translation_track: AnimationTrack<[f32; 3]>,
    /// Rotation animation track (quaternion)
    pub rotation_track: AnimationTrack<[f32; 4]>,
    /// Scale animation track
    pub scale_track: AnimationTrack<[f32; 3]>,
}

/// Animation track for bone data
#[derive(Debug, Clone)]
pub struct AnimationTrack<T> {
    /// Keyframe timestamps
    pub timestamps: Vec<u32>,
    /// Keyframe values
    pub values: Vec<T>,
    /// Interpolation type
    pub interpolation: InterpolationType,
}

/// Animation interpolation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpolationType {
    /// No interpolation (step)
    None,
    /// Linear interpolation
    Linear,
    /// Bezier curve interpolation
    Bezier,
}

impl TextureFileIds {
    /// Read TXID chunk data from a chunk reader
    pub fn read<R: Read + Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        let count = reader.chunk_size() / 4; // Each ID is 4 bytes
        let mut ids = Vec::with_capacity(count as usize);

        for _ in 0..count {
            ids.push(reader.read_u32_le()?);
        }

        Ok(TextureFileIds { ids })
    }

    /// Get the number of texture file IDs
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Check if there are no texture file IDs
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// Get a texture file ID by index
    pub fn get(&self, index: usize) -> Option<u32> {
        self.ids.get(index).copied()
    }

    /// Iterate over all texture file IDs
    pub fn iter(&self) -> std::slice::Iter<u32> {
        self.ids.iter()
    }
}

impl PhysicsFileId {
    /// Read PFID chunk data from a chunk reader
    pub fn read<R: Read + Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        if reader.chunk_size() != 4 {
            return Err(crate::error::M2Error::ParseError(format!(
                "PFID chunk should contain exactly 4 bytes, got {}",
                reader.chunk_size()
            )));
        }

        let id = reader.read_u32_le()?;
        Ok(PhysicsFileId { id })
    }
}

impl SkeletonFileId {
    /// Read SKID chunk data from a chunk reader
    pub fn read<R: Read + Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        if reader.chunk_size() != 4 {
            return Err(crate::error::M2Error::ParseError(format!(
                "SKID chunk should contain exactly 4 bytes, got {}",
                reader.chunk_size()
            )));
        }

        let id = reader.read_u32_le()?;
        Ok(SkeletonFileId { id })
    }
}

impl BoneFileIds {
    /// Read BFID chunk data from a chunk reader
    pub fn read<R: Read + Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        let count = reader.chunk_size() / 4; // Each ID is 4 bytes
        let mut ids = Vec::with_capacity(count as usize);

        for _ in 0..count {
            ids.push(reader.read_u32_le()?);
        }

        Ok(BoneFileIds { ids })
    }

    /// Get the number of bone file IDs
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Check if there are no bone file IDs
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// Get a bone file ID by index
    pub fn get(&self, index: usize) -> Option<u32> {
        self.ids.get(index).copied()
    }

    /// Iterate over all bone file IDs
    pub fn iter(&self) -> std::slice::Iter<u32> {
        self.ids.iter()
    }
}

impl LodData {
    /// Read LDV1 chunk data from a chunk reader
    pub fn read<R: Read + Seek>(reader: &mut ChunkReader<R>) -> Result<Self> {
        // LDV1 format: each LOD level is 14 bytes
        // 4 bytes: distance (float)
        // 2 bytes: skin_file_index (u16)
        // 4 bytes: vertex_count (u32)
        // 4 bytes: triangle_count (u32)
        const LOD_LEVEL_SIZE: u32 = 14;

        if reader.chunk_size() % LOD_LEVEL_SIZE != 0 {
            return Err(crate::error::M2Error::ParseError(format!(
                "LDV1 chunk size {} is not a multiple of LOD level size {}",
                reader.chunk_size(),
                LOD_LEVEL_SIZE
            )));
        }

        let count = reader.chunk_size() / LOD_LEVEL_SIZE;
        let mut levels = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let distance = reader.read_f32_le()?;
            let skin_file_index = reader.read_u16_le()?;
            let vertex_count = reader.read_u32_le()?;
            let triangle_count = reader.read_u32_le()?;

            levels.push(LodLevel {
                distance,
                skin_file_index,
                vertex_count,
                triangle_count,
            });
        }

        Ok(LodData { levels })
    }

    /// Select the appropriate LOD level for a given distance
    pub fn select_lod(&self, distance: f32) -> Option<&LodLevel> {
        // Find the first LOD level where the distance is less than or equal to the threshold
        // LOD levels should be sorted by distance (closest first)
        self.levels
            .iter()
            .find(|lod| distance <= lod.distance)
            .or_else(|| {
                // If no level matches, use the last (most distant) level
                self.levels.last()
            })
    }

    /// Get the number of LOD levels
    pub fn len(&self) -> usize {
        self.levels.len()
    }

    /// Check if there are no LOD levels
    pub fn is_empty(&self) -> bool {
        self.levels.is_empty()
    }
}

impl PhysicsData {
    /// Parse physics data from raw .phys file contents
    /// .phys files are chunked format with PHYS header chunk followed by data chunks
    pub fn parse(data: &[u8]) -> Result<Self> {
        use std::io::Cursor;

        if data.len() < 8 {
            return Err(crate::error::M2Error::ParseError(
                "PHYS file too small for header".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);

        // Read main PHYS chunk header
        let mut magic = [0u8; 4];
        cursor.read_exact(&mut magic).map_err(|e| {
            crate::error::M2Error::ParseError(format!("Failed to read PHYS magic: {}", e))
        })?;

        if &magic != b"PHYS" {
            return Err(crate::error::M2Error::ParseError(format!(
                "Invalid PHYS magic: {:?}, expected PHYS",
                magic
            )));
        }

        let _chunk_size = cursor.read_u32_le().map_err(|e| {
            crate::error::M2Error::ParseError(format!("Failed to read PHYS chunk size: {}", e))
        })?;

        // Basic physics data - for now, parse minimal structure
        // TODO: Implement full chunked parsing for BODY, SHAP, JOIN chunks

        // Try to read basic collision data if present
        let mut collision_mesh = None;
        let mut material_properties = Vec::new();

        // If there's data after the header, try to parse basic collision info
        if cursor.position() < data.len() as u64 {
            // For now, create a basic collision mesh placeholder
            // Real implementation would parse BODY, SHAP chunks
            collision_mesh = Some(CollisionMesh {
                vertices: Vec::new(),
                triangles: Vec::new(),
            });

            // Add default physics material
            material_properties.push(PhysicsMaterial {
                material_id: 0,
                friction: 0.5,
                restitution: 0.0,
                density: 1.0,
            });
        }

        Ok(PhysicsData {
            collision_mesh,
            material_properties,
        })
    }
}

impl SkeletonData {
    /// Parse skeleton data from raw .skel file contents
    /// .skel files contain SKB1 chunk with bone hierarchy information
    pub fn parse(data: &[u8]) -> Result<Self> {
        use std::io::Cursor;

        if data.len() < 8 {
            return Err(crate::error::M2Error::ParseError(
                "SKEL file too small for header".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);
        let mut bone_hierarchy = Vec::new();
        let mut root_bones = Vec::new();

        // Parse chunks until end of file
        while (cursor.position() as usize) < data.len() {
            if data.len() - (cursor.position() as usize) < 8 {
                break; // Not enough data for chunk header
            }

            let mut magic = [0u8; 4];
            cursor.read_exact(&mut magic).map_err(|e| {
                crate::error::M2Error::ParseError(format!("Failed to read chunk magic: {}", e))
            })?;

            let chunk_size = cursor.read_u32_le().map_err(|e| {
                crate::error::M2Error::ParseError(format!("Failed to read chunk size: {}", e))
            })?;

            match &magic {
                b"SKB1" => {
                    // SKB1 contains skeleton bone data
                    // For now, create basic bone hierarchy
                    // TODO: Implement full SKB1 parsing with M2Array<M2CompBone>

                    if chunk_size >= 16 {
                        // Minimum size for one bone entry
                        // Parse basic bone data - simplified for now
                        let bone_count = std::cmp::min(chunk_size / 16, 64) as usize; // Limit bones

                        for i in 0..bone_count {
                            let bone_node = BoneNode {
                                bone_index: i as u16,
                                parent_index: if i == 0 { -1 } else { (i - 1) as i16 },
                                children: Vec::new(),
                                local_transform: [
                                    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
                                    0.0, 0.0, 0.0, 1.0,
                                ], // Identity matrix
                            };
                            bone_hierarchy.push(bone_node);
                        }

                        if !bone_hierarchy.is_empty() {
                            root_bones.push(0); // First bone is root
                        }
                    }
                }
                _ => {
                    // Skip unknown chunks
                    cursor.set_position(cursor.position() + chunk_size as u64);
                }
            }
        }

        Ok(SkeletonData {
            bone_hierarchy,
            root_bones,
        })
    }
}

impl BoneData {
    /// Parse bone data from raw .bone file contents
    /// .bone files contain bone animation tracks with keyframes
    pub fn parse(data: &[u8]) -> Result<Self> {
        use std::io::Cursor;

        if data.len() < 12 {
            // Minimum: header(4) + bone_count(4) + bone_id(2) + matrix(64)
            return Err(crate::error::M2Error::ParseError(
                "BONE file too small".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);

        // Read .bone file header
        let version = cursor.read_u32_le().map_err(|e| {
            crate::error::M2Error::ParseError(format!("Failed to read .bone version: {}", e))
        })?;

        // Version should be 1 according to wowdev.wiki
        if version != 1 {
            return Err(crate::error::M2Error::ParseError(format!(
                "Unsupported .bone version: {}, expected 1",
                version
            )));
        }

        // Read bone ID array count
        let bone_count = cursor.read_u32_le().map_err(|e| {
            crate::error::M2Error::ParseError(format!("Failed to read bone count: {}", e))
        })?;

        if bone_count == 0 || bone_count > 1024 {
            // Reasonable limit
            return Err(crate::error::M2Error::ParseError(format!(
                "Invalid bone count: {}",
                bone_count
            )));
        }

        // Read first bone ID (we'll process just the first bone for now)
        let bone_index = cursor.read_u16_le().map_err(|e| {
            crate::error::M2Error::ParseError(format!("Failed to read bone ID: {}", e))
        })?;

        // Skip remaining bone IDs for now
        cursor.set_position(cursor.position() + (bone_count as u64 - 1) * 2);

        // Read bone offset matrices (skip for now as they're complex)
        // Each matrix is 16 floats (64 bytes)
        cursor.set_position(cursor.position() + bone_count as u64 * 64);

        // Create basic animation tracks
        // TODO: Parse actual keyframe data from the file
        let translation_track = AnimationTrack {
            timestamps: vec![0, 1000],                      // Basic 1-second animation
            values: vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]], // No translation
            interpolation: InterpolationType::Linear,
        };

        let rotation_track = AnimationTrack {
            timestamps: vec![0, 1000],
            values: vec![[0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.0, 1.0]], // Identity quaternion
            interpolation: InterpolationType::Linear,
        };

        let scale_track = AnimationTrack {
            timestamps: vec![0, 1000],
            values: vec![[1.0, 1.0, 1.0], [1.0, 1.0, 1.0]], // Unity scale
            interpolation: InterpolationType::Linear,
        };

        Ok(BoneData {
            bone_index,
            translation_track,
            rotation_track,
            scale_track,
        })
    }
}

impl<T> AnimationTrack<T> {
    /// Create a new empty animation track
    pub fn new(interpolation: InterpolationType) -> Self {
        AnimationTrack {
            timestamps: Vec::new(),
            values: Vec::new(),
            interpolation,
        }
    }

    /// Get the number of keyframes
    pub fn len(&self) -> usize {
        self.timestamps.len()
    }

    /// Check if the track has no keyframes
    pub fn is_empty(&self) -> bool {
        self.timestamps.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunks::infrastructure::ChunkHeader;
    use std::io::Cursor;

    #[test]
    fn test_sfid_parsing() {
        // Create test data: 3 FileDataIDs (12 bytes total)
        let data = vec![
            0x01, 0x00, 0x00, 0x00, // FileDataID 1
            0x02, 0x00, 0x00, 0x00, // FileDataID 2
            0x03, 0x00, 0x00, 0x00, // FileDataID 3
        ];

        let header = ChunkHeader {
            magic: *b"SFID",
            size: 12,
        };
        let cursor = Cursor::new(data);
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        let sfid = SkinFileIds::read(&mut chunk_reader).unwrap();

        assert_eq!(sfid.len(), 3);
        assert_eq!(sfid.get(0), Some(1));
        assert_eq!(sfid.get(1), Some(2));
        assert_eq!(sfid.get(2), Some(3));
        assert_eq!(sfid.get(3), None);

        let ids: Vec<u32> = sfid.iter().copied().collect();
        assert_eq!(ids, vec![1, 2, 3]);
    }

    #[test]
    fn test_afid_parsing() {
        let data = vec![
            0x10, 0x00, 0x00, 0x00, // FileDataID 16
            0x20, 0x00, 0x00, 0x00, // FileDataID 32
        ];

        let header = ChunkHeader {
            magic: *b"AFID",
            size: 8,
        };
        let cursor = Cursor::new(data);
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        let afid = AnimationFileIds::read(&mut chunk_reader).unwrap();

        assert_eq!(afid.len(), 2);
        assert_eq!(afid.get(0), Some(16));
        assert_eq!(afid.get(1), Some(32));
        assert!(!afid.is_empty());
    }

    #[test]
    fn test_txid_parsing() {
        let data = vec![
            0xFF, 0x00, 0x01, 0x00, // FileDataID 65791
        ];

        let header = ChunkHeader {
            magic: *b"TXID",
            size: 4,
        };
        let cursor = Cursor::new(data);
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        let txid = TextureFileIds::read(&mut chunk_reader).unwrap();

        assert_eq!(txid.len(), 1);
        assert_eq!(txid.get(0), Some(65791));
        assert!(!txid.is_empty());
    }

    #[test]
    fn test_pfid_parsing() {
        let data = vec![
            0x40, 0x00, 0x05, 0x00, // FileDataID 327744
        ];

        let header = ChunkHeader {
            magic: *b"PFID",
            size: 4,
        };
        let cursor = Cursor::new(data);
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        let pfid = PhysicsFileId::read(&mut chunk_reader).unwrap();

        assert_eq!(pfid.id, 327744);
    }

    #[test]
    fn test_skid_parsing() {
        let data = vec![
            0x80, 0x00, 0x02, 0x00, // FileDataID 131200
        ];

        let header = ChunkHeader {
            magic: *b"SKID",
            size: 4,
        };
        let cursor = Cursor::new(data);
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        let skid = SkeletonFileId::read(&mut chunk_reader).unwrap();

        assert_eq!(skid.id, 131200);
    }

    #[test]
    fn test_bfid_parsing() {
        let data = vec![
            0x01, 0x00, 0x10, 0x00, // FileDataID 1048577
            0x02, 0x00, 0x10, 0x00, // FileDataID 1048578
            0x03, 0x00, 0x10, 0x00, // FileDataID 1048579
        ];

        let header = ChunkHeader {
            magic: *b"BFID",
            size: 12,
        };
        let cursor = Cursor::new(data);
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        let bfid = BoneFileIds::read(&mut chunk_reader).unwrap();

        assert_eq!(bfid.len(), 3);
        assert_eq!(bfid.get(0), Some(1048577));
        assert_eq!(bfid.get(1), Some(1048578));
        assert_eq!(bfid.get(2), Some(1048579));
        assert!(!bfid.is_empty());
    }

    #[test]
    fn test_ldv1_parsing() {
        let data = vec![
            // First LOD level (14 bytes)
            0x00, 0x00, 0xA0, 0x42, // distance = 80.0f
            0x00, 0x00, // skin_file_index = 0
            0x00, 0x10, 0x00, 0x00, // vertex_count = 4096
            0x00, 0x20, 0x00, 0x00, // triangle_count = 8192
            // Second LOD level (14 bytes)
            0x00, 0x00, 0xC8, 0x42, // distance = 100.0f
            0x01, 0x00, // skin_file_index = 1
            0x00, 0x08, 0x00, 0x00, // vertex_count = 2048
            0x00, 0x10, 0x00, 0x00, // triangle_count = 4096
        ];

        let header = ChunkHeader {
            magic: *b"LDV1",
            size: 28,
        };
        let cursor = Cursor::new(data);
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        let ldv1 = LodData::read(&mut chunk_reader).unwrap();

        assert_eq!(ldv1.len(), 2);
        assert!(!ldv1.is_empty());

        let level0 = &ldv1.levels[0];
        assert_eq!(level0.distance, 80.0);
        assert_eq!(level0.skin_file_index, 0);
        assert_eq!(level0.vertex_count, 4096);
        assert_eq!(level0.triangle_count, 8192);

        let level1 = &ldv1.levels[1];
        assert_eq!(level1.distance, 100.0);
        assert_eq!(level1.skin_file_index, 1);
        assert_eq!(level1.vertex_count, 2048);
        assert_eq!(level1.triangle_count, 4096);

        // Test LOD selection
        assert_eq!(ldv1.select_lod(50.0).unwrap().skin_file_index, 0); // Close distance -> high detail
        assert_eq!(ldv1.select_lod(90.0).unwrap().skin_file_index, 1); // Far distance -> low detail
        assert_eq!(ldv1.select_lod(200.0).unwrap().skin_file_index, 1); // Very far -> use last level
    }

    #[test]
    fn test_pfid_invalid_size() {
        let data = vec![0x01, 0x02]; // Only 2 bytes instead of 4

        let header = ChunkHeader {
            magic: *b"PFID",
            size: 2,
        };
        let cursor = Cursor::new(data);
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        let result = PhysicsFileId::read(&mut chunk_reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_ldv1_invalid_size() {
        let data = vec![0x01, 0x02, 0x03]; // Not a multiple of 14 bytes

        let header = ChunkHeader {
            magic: *b"LDV1",
            size: 3,
        };
        let cursor = Cursor::new(data);
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        let result = LodData::read(&mut chunk_reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_chunks() {
        // Test empty chunks
        let header = ChunkHeader {
            magic: *b"SFID",
            size: 0,
        };
        let cursor = Cursor::new(Vec::new());
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        let sfid = SkinFileIds::read(&mut chunk_reader).unwrap();
        assert!(sfid.is_empty());
        assert_eq!(sfid.len(), 0);
    }

    #[test]
    fn test_physics_data_parsing_valid() {
        // Create valid PHYS chunk data
        let data = vec![
            b'P', b'H', b'Y', b'S', // PHYS magic
            0x04, 0x00, 0x00, 0x00, // Chunk size: 4 bytes
            0x01, 0x00, 0x00, 0x00, // Some physics data
        ];

        let result = PhysicsData::parse(&data).unwrap();
        assert!(result.collision_mesh.is_some());
        assert_eq!(result.material_properties.len(), 1);
        assert_eq!(result.material_properties[0].material_id, 0);
        assert_eq!(result.material_properties[0].friction, 0.5);
    }

    #[test]
    fn test_physics_data_parsing_invalid_magic() {
        let data = vec![
            b'B', b'A', b'D', b'!', // Invalid magic
            0x04, 0x00, 0x00, 0x00, // Chunk size
        ];

        let result = PhysicsData::parse(&data);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid PHYS magic")
        );
    }

    #[test]
    fn test_physics_data_parsing_too_small() {
        let data = vec![b'P', b'H', b'Y']; // Too small for header

        let result = PhysicsData::parse(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too small"));
    }

    #[test]
    fn test_skeleton_data_parsing_valid() {
        // Create valid SKB1 chunk data
        let data = vec![
            b'S', b'K', b'B', b'1', // SKB1 magic
            0x10, 0x00, 0x00, 0x00, // Chunk size: 16 bytes
            0x01, 0x00, 0x00, 0x00, // Bone data (simplified)
            0x02, 0x00, 0x00, 0x00, // More bone data
            0x03, 0x00, 0x00, 0x00, // More bone data
            0x04, 0x00, 0x00, 0x00, // More bone data
        ];

        let result = SkeletonData::parse(&data).unwrap();
        assert!(!result.bone_hierarchy.is_empty());
        assert!(!result.root_bones.is_empty());
        assert_eq!(result.root_bones[0], 0); // First bone is root
    }

    #[test]
    fn test_skeleton_data_parsing_empty() {
        let data = vec![
            b'U', b'N', b'K', b'N', // Unknown chunk
            0x04, 0x00, 0x00, 0x00, // Chunk size
            0x00, 0x00, 0x00, 0x00, // Data
        ];

        let result = SkeletonData::parse(&data).unwrap();
        assert!(result.bone_hierarchy.is_empty());
        assert!(result.root_bones.is_empty());
    }

    #[test]
    fn test_skeleton_data_parsing_too_small() {
        let data = vec![b'S', b'K', b'B']; // Too small

        let result = SkeletonData::parse(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too small"));
    }

    #[test]
    fn test_bone_data_parsing_valid() {
        // Create valid .bone file data
        let data = vec![
            0x01, 0x00, 0x00, 0x00, // Version: 1
            0x01, 0x00, 0x00, 0x00, // Bone count: 1
            0x05, 0x00, // Bone ID: 5
            // 64 bytes for bone offset matrix (identity matrix as floats)
            0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F,
        ];

        let result = BoneData::parse(&data).unwrap();
        assert_eq!(result.bone_index, 5);
        assert_eq!(result.translation_track.timestamps.len(), 2);
        assert_eq!(result.rotation_track.timestamps.len(), 2);
        assert_eq!(result.scale_track.timestamps.len(), 2);
        assert_eq!(
            result.translation_track.interpolation,
            InterpolationType::Linear
        );
    }

    #[test]
    fn test_bone_data_parsing_invalid_version() {
        // Create data large enough to pass size check but with invalid version
        let mut data = vec![
            0x02, 0x00, 0x00, 0x00, // Version: 2 (unsupported)
            0x01, 0x00, 0x00, 0x00, // Bone count: 1
            0x05, 0x00, // Bone ID: 5
        ];
        // Add 64 bytes for bone offset matrix
        data.extend(vec![0u8; 64]);

        let result = BoneData::parse(&data);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Unsupported .bone version"));
    }

    #[test]
    fn test_bone_data_parsing_invalid_bone_count() {
        // Create data large enough to pass size check but with invalid bone count
        let mut data = vec![
            0x01, 0x00, 0x00, 0x00, // Version: 1
            0x00, 0x00, 0x00, 0x00, // Bone count: 0 (invalid)
        ];
        // Add extra bytes to meet minimum size requirement
        data.extend(vec![0u8; 64]);

        let result = BoneData::parse(&data);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid bone count"));
    }

    #[test]
    fn test_bone_data_parsing_too_small() {
        let data = vec![0x01, 0x00, 0x00]; // Too small

        let result = BoneData::parse(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too small"));
    }

    #[test]
    fn test_animation_track_properties() {
        let track = AnimationTrack::<[f32; 3]>::new(InterpolationType::Bezier);
        assert!(track.is_empty());
        assert_eq!(track.len(), 0);
        assert_eq!(track.interpolation, InterpolationType::Bezier);

        let track_with_data = AnimationTrack {
            timestamps: vec![0, 500, 1000],
            values: vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [2.0, 2.0, 2.0]],
            interpolation: InterpolationType::Linear,
        };
        assert!(!track_with_data.is_empty());
        assert_eq!(track_with_data.len(), 3);
    }

    #[test]
    fn test_interpolation_type_equality() {
        assert_eq!(InterpolationType::None, InterpolationType::None);
        assert_eq!(InterpolationType::Linear, InterpolationType::Linear);
        assert_eq!(InterpolationType::Bezier, InterpolationType::Bezier);
        assert_ne!(InterpolationType::None, InterpolationType::Linear);
    }
}
