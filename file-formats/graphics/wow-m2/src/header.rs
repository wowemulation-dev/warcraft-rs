use crate::io_ext::{ReadExt, WriteExt};
use bitflags::bitflags;
use std::io::{Read, Seek, Write};

use crate::common::M2Array;
use crate::error::{M2Error, Result};
use crate::version::M2Version;

/// Magic signature for M2 files ("MD20")
pub const M2_MAGIC: [u8; 4] = *b"MD20";

bitflags! {
    /// Model flags as defined in the M2 format
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct M2ModelFlags: u32 {
        /// Tilt on X axis
        const TILT_X = 0x0001;
        /// Tilt on Y axis
        const TILT_Y = 0x0002;
        /// Add a back-reference to the model
        const ADD_BACK_REFERENCE = 0x0004;
        /// Use texture combiners
        const USE_TEXTURE_COMBINERS = 0x0008;
        /// Is it a camera?
        const IS_CAMERA = 0x0010;
        /// Unused flag
        const UNUSED = 0x0020;
        /// No particle trails
        const NO_PARTICLE_TRAILS = 0x0040;
        /// Unknown
        const UNKNOWN_0x80 = 0x0080;
        /// Load phys data
        const LOAD_PHYS_DATA = 0x0100;
        /// Unknown
        const UNKNOWN_0x200 = 0x0200;
        /// Has bones
        const HAS_BONES = 0x0400;
        /// Unused 0x800
        const UNUSED_0x800 = 0x0800;
        /// Unknown
        const UNKNOWN_0x1000 = 0x1000;
        /// Use texture IDs
        const USE_TEXTURE_IDS = 0x2000;
        /// Camera can be modified
        const CAMERA_MODIFIABLE = 0x4000;
        /// New particle system
        const NEW_PARTICLE_SYSTEM = 0x8000;
        /// Unknown
        const UNKNOWN_0x10000 = 0x10000;
        /// Unknown
        const UNKNOWN_0x20000 = 0x20000;
        /// Unknown
        const UNKNOWN_0x40000 = 0x40000;
        /// Unknown
        const UNKNOWN_0x80000 = 0x80000;
        /// Unknown
        const UNKNOWN_0x100000 = 0x100000;
        /// Unknown
        const UNKNOWN_0x200000 = 0x200000;
        /// Unknown
        const UNKNOWN_0x400000 = 0x400000;
        /// Unknown
        const UNKNOWN_0x800000 = 0x800000;
        /// Unknown
        const UNKNOWN_0x1000000 = 0x1000000;
        /// Unknown
        const UNKNOWN_0x2000000 = 0x2000000;
        /// Unknown
        const UNKNOWN_0x4000000 = 0x4000000;
        /// Unknown
        const UNKNOWN_0x8000000 = 0x8000000;
        /// Unknown
        const UNKNOWN_0x10000000 = 0x10000000;
        /// Unknown
        const UNKNOWN_0x20000000 = 0x20000000;
        /// Unknown
        const UNKNOWN_0x40000000 = 0x40000000;
        /// Unknown
        const UNKNOWN_0x80000000 = 0x80000000;
    }
}

/// M2 model header structure
/// Based on: <https://wowdev.wiki/M2#Header>
#[derive(Debug, Clone)]
pub struct M2Header {
    /// Magic signature ("MD20")
    pub magic: [u8; 4],
    /// Version of the M2 file
    pub version: u32,
    /// Name of the model
    pub name: M2Array<u8>,
    /// Flags
    pub flags: M2ModelFlags,

    // Sequence-related fields
    /// Global sequences
    pub global_sequences: M2Array<u32>,
    /// Animations
    pub animations: M2Array<u32>,
    /// Animation lookups (C in Classic)
    pub animation_lookup: M2Array<u16>,
    /// Playable animation lookup - only present in versions <= 263
    pub playable_animation_lookup: Option<M2Array<u16>>,

    // Bone-related fields
    /// Bones
    pub bones: M2Array<u32>,
    /// Key bone lookup
    pub key_bone_lookup: M2Array<u16>,

    // Geometry data
    /// Vertices
    pub vertices: M2Array<u32>,
    /// Views (LOD levels) - M2Array for BC and earlier, count for later versions
    pub views: M2Array<u32>,
    /// Number of skin profiles for WotLK+ (when views becomes a count)
    pub num_skin_profiles: Option<u32>,

    // Color data
    /// Color animations
    pub color_animations: M2Array<u32>,

    // Texture-related fields
    /// Textures
    pub textures: M2Array<u32>,
    /// Transparency lookups
    pub transparency_lookup: M2Array<u16>,
    /// Transparency animations
    pub transparency_animations: M2Array<u32>,
    /// Texture flipbooks - only present in BC and earlier
    pub texture_flipbooks: Option<M2Array<u32>>,
    /// Texture animations
    pub texture_animations: M2Array<u32>,

    // Material data
    /// Render flags
    pub render_flags: M2Array<u32>,
    /// Bone lookup table
    pub bone_lookup_table: M2Array<u16>,
    /// Texture lookup table
    pub texture_lookup_table: M2Array<u16>,
    /// Texture mapping lookup table
    pub texture_mapping_lookup_table: M2Array<u16>,
    /// Texture units
    pub texture_units: M2Array<u16>,
    /// Transparency lookup table
    pub transparency_lookup_table: M2Array<u16>,
    /// Texture animation lookup table
    pub texture_animation_lookup: M2Array<u16>,

    // Bounding box data
    /// Bounding box min corner
    pub bounding_box_min: [f32; 3],
    /// Bounding box max corner
    pub bounding_box_max: [f32; 3],
    /// Bounding sphere radius
    pub bounding_sphere_radius: f32,
    /// Collision bounding box min corner
    pub collision_box_min: [f32; 3],
    /// Collision bounding box max corner
    pub collision_box_max: [f32; 3],
    /// Collision bounding sphere radius
    pub collision_sphere_radius: f32,

    // Additional geometry data
    /// Bounding triangles
    pub bounding_triangles: M2Array<u32>,
    /// Bounding vertices
    pub bounding_vertices: M2Array<u32>,
    /// Bounding normals
    pub bounding_normals: M2Array<u32>,

    // Attachments and events
    /// Attachments
    pub attachments: M2Array<u32>,
    /// Attachment lookup table
    pub attachment_lookup_table: M2Array<u16>,
    /// Events
    pub events: M2Array<u32>,
    /// Lights
    pub lights: M2Array<u32>,
    /// Cameras
    pub cameras: M2Array<u32>,
    /// Camera lookup table
    pub camera_lookup_table: M2Array<u16>,

    // Ribbon emitters
    /// Ribbon emitters
    pub ribbon_emitters: M2Array<u32>,

    // Particle systems
    /// Particle emitters
    pub particle_emitters: M2Array<u32>,

    // Additional fields for newer versions
    /// Blend map overrides (BC+ with specific flag)
    pub blend_map_overrides: Option<M2Array<u32>>,
    /// Texture combiner combos (added in Cataclysm)
    pub texture_combiner_combos: Option<M2Array<u32>>,

    // Fields added in Legion
    /// Texture transforms
    pub texture_transforms: Option<M2Array<u32>>,
}

impl M2Header {
    /// Parse the M2 header from a reader
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Read and check magic
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;

        if magic != M2_MAGIC {
            return Err(M2Error::InvalidMagic {
                expected: String::from_utf8_lossy(&M2_MAGIC).to_string(),
                actual: String::from_utf8_lossy(&magic).to_string(),
            });
        }

        // Read version
        let version = reader.read_u32_le()?;

        // Check if version is supported
        if M2Version::from_header_version(version).is_none() {
            return Err(M2Error::UnsupportedVersion(version.to_string()));
        }

        // Read the common fields present in all versions
        let name = M2Array::parse(reader)?;
        let flags = M2ModelFlags::from_bits_retain(reader.read_u32_le()?);

        let global_sequences = M2Array::parse(reader)?;
        let animations = M2Array::parse(reader)?;
        let animation_lookup = M2Array::parse(reader)?;

        // Vanilla/TBC versions have playable animation lookup
        let playable_animation_lookup = if version <= 263 {
            Some(M2Array::parse(reader)?)
        } else {
            None
        };

        let bones = M2Array::parse(reader)?;
        let key_bone_lookup = M2Array::parse(reader)?;

        let vertices = M2Array::parse(reader)?;

        // Views field changes between versions
        let (views, num_skin_profiles) = if version <= 263 {
            // BC and earlier: views is M2Array
            (M2Array::parse(reader)?, None)
        } else {
            // WotLK+: views becomes a count (num_skin_profiles)
            let count = reader.read_u32_le()?;
            (M2Array::new(0, 0), Some(count))
        };

        let color_animations = M2Array::parse(reader)?; // colors

        let textures = M2Array::parse(reader)?; // textures
        let transparency_lookup = M2Array::parse(reader)?; // texture_weights

        // Texture flipbooks only exist in BC and earlier
        let texture_flipbooks = if version <= 263 {
            Some(M2Array::parse(reader)?)
        } else {
            None
        };

        let transparency_animations = M2Array::parse(reader)?; // texture_transforms

        let texture_animations = M2Array::parse(reader)?; // texture_indices_by_id

        let render_flags = M2Array::parse(reader)?; // materials
        let bone_lookup_table = M2Array::parse(reader)?; // bone_combos
        let texture_lookup_table = M2Array::parse(reader)?; // texture_combos
        let texture_mapping_lookup_table = M2Array::parse(reader)?; // texture_coord_combos
        let texture_units = M2Array::parse(reader)?; // texture weight combos
        let transparency_lookup_table = M2Array::parse(reader)?; // texture transform combos
        // let mut texture_animation_lookup = M2Array::parse(reader)?;
        //
        // // Workaround: Some M2 files have corrupted texture_animation_lookup fields
        // // This may be due to field alignment differences across versions or file corruption
        // // If the count is extremely large, treat as empty to prevent crashes
        // if texture_animation_lookup.count > 1_000_000 {
        let texture_animation_lookup = M2Array::new(0, 0);
        // }

        // Read bounding box
        let mut bounding_box_min = [0.0; 3];
        let mut bounding_box_max = [0.0; 3];

        for item in &mut bounding_box_min {
            *item = reader.read_f32_le()?;
        }

        for item in &mut bounding_box_max {
            *item = reader.read_f32_le()?;
        }

        let bounding_sphere_radius = reader.read_f32_le()?;

        // Read collision box
        let mut collision_box_min = [0.0; 3];
        let mut collision_box_max = [0.0; 3];

        for item in &mut collision_box_min {
            *item = reader.read_f32_le()?;
        }

        for item in &mut collision_box_max {
            *item = reader.read_f32_le()?;
        }

        let collision_sphere_radius = reader.read_f32_le()?;

        let bounding_triangles = M2Array::parse(reader)?;
        let bounding_vertices = M2Array::parse(reader)?;
        let bounding_normals = M2Array::parse(reader)?;

        let attachments = M2Array::parse(reader)?;
        let attachment_lookup_table = M2Array::parse(reader)?;
        let events = M2Array::parse(reader)?;
        let lights = M2Array::parse(reader)?;
        let cameras = M2Array::parse(reader)?;
        let camera_lookup_table = M2Array::parse(reader)?;

        let ribbon_emitters = M2Array::parse(reader)?;
        let particle_emitters = M2Array::parse(reader)?;

        // Version-specific fields
        let m2_version = M2Version::from_header_version(version).unwrap();

        // Blend map overrides (BC+ with specific flag)
        let blend_map_overrides = if version >= 260 && (flags.bits() & 0x8000000 != 0) {
            // USE_BLEND_MAP_OVERRIDES flag - using hex value as we don't have the flag defined
            Some(M2Array::parse(reader)?)
        } else {
            None
        };

        let texture_combiner_combos = if m2_version >= M2Version::Cataclysm {
            Some(M2Array::parse(reader)?)
        } else {
            None
        };

        let texture_transforms = if m2_version >= M2Version::Legion {
            Some(M2Array::parse(reader)?)
        } else {
            None
        };

        Ok(Self {
            magic,
            version,
            name,
            flags,
            global_sequences,
            animations,
            animation_lookup,
            playable_animation_lookup,
            bones,
            key_bone_lookup,
            vertices,
            views,
            num_skin_profiles,
            color_animations,
            textures,
            transparency_lookup,
            transparency_animations,
            texture_flipbooks,
            texture_animations,
            render_flags,
            bone_lookup_table,
            texture_lookup_table,
            texture_mapping_lookup_table,
            texture_units,
            transparency_lookup_table,
            texture_animation_lookup,
            bounding_box_min,
            bounding_box_max,
            bounding_sphere_radius,
            collision_box_min,
            collision_box_max,
            collision_sphere_radius,
            bounding_triangles,
            bounding_vertices,
            bounding_normals,
            attachments,
            attachment_lookup_table,
            events,
            lights,
            cameras,
            camera_lookup_table,
            ribbon_emitters,
            particle_emitters,
            blend_map_overrides,
            texture_combiner_combos,
            texture_transforms,
        })
    }

    /// Write the M2 header to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Write magic and version
        writer.write_all(&self.magic)?;
        writer.write_u32_le(self.version)?;

        // Write common fields
        self.name.write(writer)?;
        writer.write_u32_le(self.flags.bits())?;

        self.global_sequences.write(writer)?;
        self.animations.write(writer)?;
        self.animation_lookup.write(writer)?;

        // Vanilla/TBC versions have playable animation lookup
        if self.version <= 263 {
            if let Some(ref pal) = self.playable_animation_lookup {
                pal.write(writer)?;
            }
        }

        self.bones.write(writer)?;
        self.key_bone_lookup.write(writer)?;

        self.vertices.write(writer)?;

        // Views field changes between versions
        if self.version <= 263 {
            // BC and earlier: views is M2Array
            self.views.write(writer)?;
        } else {
            // WotLK+: write num_skin_profiles as u32
            let count = self.num_skin_profiles.unwrap_or(0);
            writer.write_u32_le(count)?;
        }

        self.color_animations.write(writer)?;

        self.textures.write(writer)?;
        self.transparency_lookup.write(writer)?;
        self.transparency_animations.write(writer)?;

        // Texture flipbooks only exist in BC and earlier
        if self.version <= 263 {
            if let Some(ref flipbooks) = self.texture_flipbooks {
                flipbooks.write(writer)?;
            }
        }

        self.texture_animations.write(writer)?;

        self.render_flags.write(writer)?;
        self.bone_lookup_table.write(writer)?;
        self.texture_lookup_table.write(writer)?;
        self.texture_mapping_lookup_table.write(writer)?;
        self.texture_units.write(writer)?;
        self.transparency_lookup_table.write(writer)?;
        self.texture_animation_lookup.write(writer)?;

        // Write bounding box
        for &value in &self.bounding_box_min {
            writer.write_f32_le(value)?;
        }

        for &value in &self.bounding_box_max {
            writer.write_f32_le(value)?;
        }

        writer.write_f32_le(self.bounding_sphere_radius)?;

        // Write collision box
        for &value in &self.collision_box_min {
            writer.write_f32_le(value)?;
        }

        for &value in &self.collision_box_max {
            writer.write_f32_le(value)?;
        }

        writer.write_f32_le(self.collision_sphere_radius)?;

        self.bounding_triangles.write(writer)?;
        self.bounding_vertices.write(writer)?;
        self.bounding_normals.write(writer)?;

        self.attachments.write(writer)?;
        self.attachment_lookup_table.write(writer)?;
        self.events.write(writer)?;
        self.lights.write(writer)?;
        self.cameras.write(writer)?;
        self.camera_lookup_table.write(writer)?;

        self.ribbon_emitters.write(writer)?;
        self.particle_emitters.write(writer)?;

        // Version-specific fields
        if let Some(ref overrides) = self.blend_map_overrides {
            overrides.write(writer)?;
        }

        if let Some(ref combos) = self.texture_combiner_combos {
            combos.write(writer)?;
        }

        if let Some(ref transforms) = self.texture_transforms {
            transforms.write(writer)?;
        }

        Ok(())
    }

    /// Get the version of the M2 file
    pub fn version(&self) -> Option<M2Version> {
        M2Version::from_header_version(self.version)
    }

    /// Create a new M2 header for a specific version
    pub fn new(version: M2Version) -> Self {
        let version_num = version.to_header_version();

        let texture_combiner_combos = if version >= M2Version::Cataclysm {
            Some(M2Array::new(0, 0))
        } else {
            None
        };

        let texture_transforms = if version >= M2Version::Legion {
            Some(M2Array::new(0, 0))
        } else {
            None
        };

        // Version-specific fields
        let playable_animation_lookup = if (260..=263).contains(&version_num) {
            Some(M2Array::new(0, 0))
        } else {
            None
        };

        let texture_flipbooks = if version_num <= 263 {
            Some(M2Array::new(0, 0))
        } else {
            None
        };

        let num_skin_profiles = if version_num > 263 { Some(0) } else { None };

        Self {
            magic: M2_MAGIC,
            version: version_num,
            name: M2Array::new(0, 0),
            flags: M2ModelFlags::empty(),
            global_sequences: M2Array::new(0, 0),
            animations: M2Array::new(0, 0),
            animation_lookup: M2Array::new(0, 0),
            playable_animation_lookup,
            bones: M2Array::new(0, 0),
            key_bone_lookup: M2Array::new(0, 0),
            vertices: M2Array::new(0, 0),
            views: M2Array::new(0, 0),
            num_skin_profiles,
            color_animations: M2Array::new(0, 0),
            textures: M2Array::new(0, 0),
            transparency_lookup: M2Array::new(0, 0),
            transparency_animations: M2Array::new(0, 0),
            texture_flipbooks,
            texture_animations: M2Array::new(0, 0),
            render_flags: M2Array::new(0, 0),
            bone_lookup_table: M2Array::new(0, 0),
            texture_lookup_table: M2Array::new(0, 0),
            texture_mapping_lookup_table: M2Array::new(0, 0),
            texture_units: M2Array::new(0, 0),
            transparency_lookup_table: M2Array::new(0, 0),
            texture_animation_lookup: M2Array::new(0, 0),
            bounding_box_min: [0.0, 0.0, 0.0],
            bounding_box_max: [0.0, 0.0, 0.0],
            bounding_sphere_radius: 0.0,
            collision_box_min: [0.0, 0.0, 0.0],
            collision_box_max: [0.0, 0.0, 0.0],
            collision_sphere_radius: 0.0,
            bounding_triangles: M2Array::new(0, 0),
            bounding_vertices: M2Array::new(0, 0),
            bounding_normals: M2Array::new(0, 0),
            attachments: M2Array::new(0, 0),
            attachment_lookup_table: M2Array::new(0, 0),
            events: M2Array::new(0, 0),
            lights: M2Array::new(0, 0),
            cameras: M2Array::new(0, 0),
            camera_lookup_table: M2Array::new(0, 0),
            ribbon_emitters: M2Array::new(0, 0),
            particle_emitters: M2Array::new(0, 0),
            blend_map_overrides: None,
            texture_combiner_combos,
            texture_transforms,
        }
    }

    /// Convert this header to a different version
    pub fn convert(&self, target_version: M2Version) -> Result<Self> {
        let source_version = self.version().ok_or(M2Error::ConversionError {
            from: self.version,
            to: target_version.to_header_version(),
            reason: "Unknown source version".to_string(),
        })?;

        if source_version == target_version {
            return Ok(self.clone());
        }

        let mut new_header = self.clone();
        new_header.version = target_version.to_header_version();

        // Handle version-specific fields
        if target_version >= M2Version::Cataclysm && source_version < M2Version::Cataclysm {
            // Add texture_combiner_combos when upgrading to Cataclysm or later
            new_header.texture_combiner_combos = Some(M2Array::new(0, 0));
        } else if target_version < M2Version::Cataclysm && source_version >= M2Version::Cataclysm {
            // Remove texture_combiner_combos when downgrading to pre-Cataclysm
            new_header.texture_combiner_combos = None;
        }

        if target_version >= M2Version::Legion && source_version < M2Version::Legion {
            // Add texture_transforms when upgrading to Legion or later
            new_header.texture_transforms = Some(M2Array::new(0, 0));
        } else if target_version < M2Version::Legion && source_version >= M2Version::Legion {
            // Remove texture_transforms when downgrading to pre-Legion
            new_header.texture_transforms = None;
        }

        Ok(new_header)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // Helper function to create a basic test header
    fn create_test_header(version: M2Version) -> Vec<u8> {
        let mut data = Vec::new();

        // Magic "MD20"
        data.extend_from_slice(&M2_MAGIC);

        // Version
        data.extend_from_slice(&version.to_header_version().to_le_bytes());

        // Name
        data.extend_from_slice(&0u32.to_le_bytes()); // count = 0
        data.extend_from_slice(&0u32.to_le_bytes()); // offset = 0

        // Flags
        data.extend_from_slice(&0u32.to_le_bytes());

        // Global sequences
        data.extend_from_slice(&0u32.to_le_bytes()); // count = 0
        data.extend_from_slice(&0u32.to_le_bytes()); // offset = 0

        // ... continue for all required fields
        // This is simplified for brevity - in a real test, we'd populate all fields

        // For brevity, let's just add enough bytes to cover the base header
        for _ in 0..100 {
            data.extend_from_slice(&0u32.to_le_bytes());
        }

        data
    }

    #[test]
    fn test_header_parse_classic() {
        let data = create_test_header(M2Version::Classic);
        let mut cursor = Cursor::new(data);

        let header = M2Header::parse(&mut cursor).unwrap();

        assert_eq!(header.magic, M2_MAGIC);
        assert_eq!(header.version, M2Version::Classic.to_header_version());
        assert_eq!(header.texture_combiner_combos, None);
        assert_eq!(header.texture_transforms, None);
    }

    #[test]
    fn test_header_parse_cataclysm() {
        let data = create_test_header(M2Version::Cataclysm);
        let mut cursor = Cursor::new(data);

        let header = M2Header::parse(&mut cursor).unwrap();

        assert_eq!(header.magic, M2_MAGIC);
        assert_eq!(header.version, M2Version::Cataclysm.to_header_version());
        assert!(header.texture_combiner_combos.is_some());
        assert_eq!(header.texture_transforms, None);
    }

    #[test]
    fn test_header_parse_legion() {
        let data = create_test_header(M2Version::Legion);
        let mut cursor = Cursor::new(data);

        let header = M2Header::parse(&mut cursor).unwrap();

        assert_eq!(header.magic, M2_MAGIC);
        assert_eq!(header.version, M2Version::Legion.to_header_version());
        assert!(header.texture_combiner_combos.is_some());
        assert!(header.texture_transforms.is_some());
    }

    #[test]
    fn test_header_conversion() {
        let classic_header = M2Header::new(M2Version::Classic);

        // Convert Classic to Cataclysm
        let cataclysm_header = classic_header.convert(M2Version::Cataclysm).unwrap();
        assert_eq!(
            cataclysm_header.version,
            M2Version::Cataclysm.to_header_version()
        );
        assert!(cataclysm_header.texture_combiner_combos.is_some());
        assert_eq!(cataclysm_header.texture_transforms, None);

        // Convert Cataclysm to Legion
        let legion_header = cataclysm_header.convert(M2Version::Legion).unwrap();
        assert_eq!(legion_header.version, M2Version::Legion.to_header_version());
        assert!(legion_header.texture_combiner_combos.is_some());
        assert!(legion_header.texture_transforms.is_some());

        // Convert Legion back to Classic
        let classic_header_2 = legion_header.convert(M2Version::Classic).unwrap();
        assert_eq!(
            classic_header_2.version,
            M2Version::Classic.to_header_version()
        );
        assert_eq!(classic_header_2.texture_combiner_combos, None);
        assert_eq!(classic_header_2.texture_transforms, None);
    }
}
