//! BuiltAdt structure representing validated ADT ready for serialization.

use std::fs::File;
use std::io::{BufWriter, Cursor, Write};
use std::path::Path;

use crate::api::RootAdt;
use crate::chunks::mh2o::Mh2oChunk;
use crate::chunks::{
    DoodadPlacement, MampChunk, MbbbChunk, MbmhChunk, MbmiChunk, MbnvChunk, McnkChunk, MfboChunk,
    MtxfChunk, MtxpChunk, WmoPlacement,
};
use crate::error::Result;
use crate::version::AdtVersion;

use super::serializer;

/// Validated ADT structure ready for serialization.
///
/// This structure is returned by `AdtBuilder::build()` after all validation
/// passes. It contains all data necessary to serialize a valid ADT file.
///
/// # Guarantees
///
/// - All required chunks present (textures, MCNK)
/// - MCNK count ≤ 256
/// - All placement references valid
/// - Version-chunk compatibility verified
///
/// # Examples
///
/// ```no_run
/// use wow_adt::builder::AdtBuilder;
/// use wow_adt::AdtVersion;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let adt = AdtBuilder::new()
///     .with_version(AdtVersion::WotLK)
///     .add_texture("terrain/grass.blp")
///     .build()?;
///
/// // Serialize to file
/// adt.write_to_file("world/maps/custom/custom_32_32.adt")?;
///
/// // Or get bytes
/// let bytes = adt.to_bytes()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct BuiltAdt {
    /// Detected ADT version
    version: AdtVersion,

    /// Texture filenames (MTEX chunk)
    textures: Vec<String>,

    /// M2 model filenames (MMDX chunk)
    models: Vec<String>,

    /// WMO filenames (MWMO chunk)
    wmos: Vec<String>,

    /// M2 model placements (MDDF chunk)
    doodad_placements: Vec<DoodadPlacement>,

    /// WMO placements (MODF chunk)
    wmo_placements: Vec<WmoPlacement>,

    /// Terrain chunks (MCNK chunks)
    mcnk_chunks: Vec<McnkChunk>,

    /// Flight boundaries (MFBO chunk, TBC+)
    flight_bounds: Option<MfboChunk>,

    /// Water data (MH2O chunk, WotLK+)
    water_data: Option<Mh2oChunk>,

    /// Texture flags (MTXF chunk, WotLK+)
    texture_flags: Option<MtxfChunk>,

    /// Texture amplifier (MAMP chunk, Cataclysm+)
    texture_amplifier: Option<MampChunk>,

    /// Texture parameters (MTXP chunk, MoP+)
    texture_params: Option<MtxpChunk>,

    /// Blend mesh headers (MBMH chunk, MoP+)
    blend_mesh_headers: Option<MbmhChunk>,

    /// Blend mesh bounding boxes (MBBB chunk, MoP+)
    blend_mesh_bounds: Option<MbbbChunk>,

    /// Blend mesh vertices (MBNV chunk, MoP+)
    blend_mesh_vertices: Option<MbnvChunk>,

    /// Blend mesh indices (MBMI chunk, MoP+)
    blend_mesh_indices: Option<MbmiChunk>,
}

impl BuiltAdt {
    /// Create new BuiltAdt from validated components.
    ///
    /// This is an internal constructor used by `AdtBuilder::build()`.
    /// External code should use `AdtBuilder` instead.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        version: AdtVersion,
        textures: Vec<String>,
        models: Vec<String>,
        wmos: Vec<String>,
        doodad_placements: Vec<DoodadPlacement>,
        wmo_placements: Vec<WmoPlacement>,
        mcnk_chunks: Vec<McnkChunk>,
        flight_bounds: Option<MfboChunk>,
        water_data: Option<Mh2oChunk>,
        texture_flags: Option<MtxfChunk>,
        texture_amplifier: Option<MampChunk>,
        texture_params: Option<MtxpChunk>,
        blend_mesh_headers: Option<MbmhChunk>,
        blend_mesh_bounds: Option<MbbbChunk>,
        blend_mesh_vertices: Option<MbnvChunk>,
        blend_mesh_indices: Option<MbmiChunk>,
    ) -> Self {
        Self {
            version,
            textures,
            models,
            wmos,
            doodad_placements,
            wmo_placements,
            mcnk_chunks,
            flight_bounds,
            water_data,
            texture_flags,
            texture_amplifier,
            texture_params,
            blend_mesh_headers,
            blend_mesh_bounds,
            blend_mesh_vertices,
            blend_mesh_indices,
        }
    }

    /// Create a BuiltAdt from a parsed RootAdt with optional version conversion.
    ///
    /// This method allows converting an existing ADT to a different version or
    /// preparing it for re-serialization. The conversion handles version-specific
    /// chunks appropriately:
    ///
    /// - **Upgrading**: Adds empty version-specific chunks as needed
    /// - **Downgrading**: Removes chunks not supported in target version
    ///
    /// # Arguments
    ///
    /// * `root` - Parsed RootAdt to convert
    /// * `target_version` - Target ADT version (or None to keep original)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wow_adt::{parse_adt, ParsedAdt, AdtVersion};
    /// use wow_adt::builder::BuiltAdt;
    /// use std::fs::File;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut file = File::open("input.adt")?;
    /// let parsed = parse_adt(&mut file)?;
    ///
    /// if let ParsedAdt::Root(root) = parsed {
    ///     let converted = BuiltAdt::from_root_adt(*root, Some(AdtVersion::WotLK));
    ///     converted.write_to_file("output.adt")?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn from_root_adt(root: RootAdt, target_version: Option<AdtVersion>) -> Self {
        let version = target_version.unwrap_or(root.version);

        // Handle version-specific chunks based on target version
        let flight_bounds = if version >= AdtVersion::TBC {
            // Use existing flight bounds or create defaults for TBC+
            root.flight_bounds.or(Some(MfboChunk {
                max_plane: [0; 9],
                min_plane: [0; 9],
            }))
        } else {
            None // Remove for pre-TBC
        };

        let water_data = if version >= AdtVersion::WotLK {
            root.water_data
        } else {
            None // Remove for pre-WotLK
        };

        let texture_flags = if version >= AdtVersion::WotLK {
            root.texture_flags
        } else {
            None // Remove for pre-WotLK
        };

        let texture_amplifier = if version >= AdtVersion::Cataclysm {
            root.texture_amplifier
        } else {
            None // Remove for pre-Cataclysm
        };

        let texture_params = if version >= AdtVersion::MoP {
            root.texture_params
        } else {
            None // Remove for pre-MoP
        };

        // Blend mesh data is MoP+
        let blend_mesh_headers = if version >= AdtVersion::MoP {
            root.blend_mesh_headers
        } else {
            None
        };

        let blend_mesh_bounds = if version >= AdtVersion::MoP {
            root.blend_mesh_bounds
        } else {
            None
        };

        let blend_mesh_vertices = if version >= AdtVersion::MoP {
            root.blend_mesh_vertices
        } else {
            None
        };

        let blend_mesh_indices = if version >= AdtVersion::MoP {
            root.blend_mesh_indices
        } else {
            None
        };

        Self {
            version,
            textures: root.textures,
            models: root.models,
            wmos: root.wmos,
            doodad_placements: root.doodad_placements,
            wmo_placements: root.wmo_placements,
            mcnk_chunks: root.mcnk_chunks,
            flight_bounds,
            water_data,
            texture_flags,
            texture_amplifier,
            texture_params,
            blend_mesh_headers,
            blend_mesh_bounds,
            blend_mesh_vertices,
            blend_mesh_indices,
        }
    }

    /// Get ADT version.
    #[must_use]
    pub fn version(&self) -> AdtVersion {
        self.version
    }

    /// Get textures.
    #[must_use]
    pub fn textures(&self) -> &[String] {
        &self.textures
    }

    /// Get models.
    #[must_use]
    pub fn models(&self) -> &[String] {
        &self.models
    }

    /// Get WMOs.
    #[must_use]
    pub fn wmos(&self) -> &[String] {
        &self.wmos
    }

    /// Get doodad placements.
    #[must_use]
    pub fn doodad_placements(&self) -> &[DoodadPlacement] {
        &self.doodad_placements
    }

    /// Get WMO placements.
    #[must_use]
    pub fn wmo_placements(&self) -> &[WmoPlacement] {
        &self.wmo_placements
    }

    /// Get MCNK chunks.
    #[must_use]
    pub fn mcnk_chunks(&self) -> &[McnkChunk] {
        &self.mcnk_chunks
    }

    /// Get flight bounds (if present).
    #[must_use]
    pub fn flight_bounds(&self) -> Option<&MfboChunk> {
        self.flight_bounds.as_ref()
    }

    /// Get water data (if present).
    #[must_use]
    pub fn water_data(&self) -> Option<&Mh2oChunk> {
        self.water_data.as_ref()
    }

    /// Get texture flags (if present).
    #[must_use]
    pub fn texture_flags(&self) -> Option<&MtxfChunk> {
        self.texture_flags.as_ref()
    }

    /// Get texture amplifier (if present).
    #[must_use]
    pub fn texture_amplifier(&self) -> Option<&MampChunk> {
        self.texture_amplifier.as_ref()
    }

    /// Get texture parameters (if present).
    #[must_use]
    pub fn texture_params(&self) -> Option<&MtxpChunk> {
        self.texture_params.as_ref()
    }

    /// Get blend mesh headers (if present).
    #[must_use]
    pub fn blend_mesh_headers(&self) -> Option<&MbmhChunk> {
        self.blend_mesh_headers.as_ref()
    }

    /// Get blend mesh bounding boxes (if present).
    #[must_use]
    pub fn blend_mesh_bounds(&self) -> Option<&MbbbChunk> {
        self.blend_mesh_bounds.as_ref()
    }

    /// Get blend mesh vertices (if present).
    #[must_use]
    pub fn blend_mesh_vertices(&self) -> Option<&MbnvChunk> {
        self.blend_mesh_vertices.as_ref()
    }

    /// Get blend mesh indices (if present).
    #[must_use]
    pub fn blend_mesh_indices(&self) -> Option<&MbmiChunk> {
        self.blend_mesh_indices.as_ref()
    }

    /// Serialize ADT to file.
    ///
    /// This method will:
    /// 1. Write all chunks in proper order
    /// 2. Calculate MHDR and MCIN offsets
    /// 3. Update offset tables
    ///
    /// # Chunk Order
    ///
    /// ```text
    /// MVER
    /// MHDR
    /// MCIN
    /// MTEX
    /// MMDX
    /// MMID
    /// MWMO
    /// MWID
    /// MDDF
    /// MODF
    /// MFBO (if present)
    /// MH2O (if present)
    /// MAMP (if present)
    /// MTXP (if present)
    /// MCNK[0..N]
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `AdtError::Io` if file I/O fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wow_adt::builder::AdtBuilder;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let adt = AdtBuilder::new()
    ///     .add_texture("terrain/grass.blp")
    ///     .build()?;
    ///
    /// adt.write_to_file("world/maps/custom/custom_32_32.adt")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        serializer::serialize_to_writer(self, &mut writer)?;
        writer.flush()?;
        Ok(())
    }

    /// Serialize ADT to byte vector.
    ///
    /// This method performs the same serialization as `write_to_file()` but
    /// returns the result as an in-memory byte vector.
    ///
    /// # Errors
    ///
    /// Returns `AdtError` if serialization fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wow_adt::builder::AdtBuilder;
    /// use std::io::Cursor;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let adt = AdtBuilder::new()
    ///     .add_texture("terrain/grass.blp")
    ///     .build()?;
    ///
    /// let bytes = adt.to_bytes()?;
    ///
    /// // Parse back to verify round-trip
    /// let mut cursor = Cursor::new(bytes);
    /// let parsed = wow_adt::parse_adt(&mut cursor)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = Cursor::new(Vec::new());
        serializer::serialize_to_writer(self, &mut buffer)?;
        Ok(buffer.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_minimal_built_adt() -> BuiltAdt {
        BuiltAdt::new(
            AdtVersion::VanillaEarly,
            vec!["terrain/grass.blp".to_string()],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
    }

    #[test]
    fn test_new_built_adt() {
        let adt = create_minimal_built_adt();
        assert_eq!(adt.version(), AdtVersion::VanillaEarly);
        assert_eq!(adt.textures().len(), 1);
        assert_eq!(adt.textures()[0], "terrain/grass.blp");
    }

    #[test]
    fn test_getters() {
        let adt = create_minimal_built_adt();
        assert_eq!(adt.version(), AdtVersion::VanillaEarly);
        assert_eq!(adt.textures().len(), 1);
        assert_eq!(adt.models().len(), 0);
        assert_eq!(adt.wmos().len(), 0);
        assert_eq!(adt.doodad_placements().len(), 0);
        assert_eq!(adt.wmo_placements().len(), 0);
        assert_eq!(adt.mcnk_chunks().len(), 0);
        assert!(adt.flight_bounds().is_none());
        assert!(adt.water_data().is_none());
        assert!(adt.texture_amplifier().is_none());
        assert!(adt.texture_params().is_none());
    }

    #[test]
    fn test_to_bytes_now_works() {
        let adt = create_minimal_built_adt();
        let result = adt.to_bytes();
        assert!(result.is_ok(), "Serialization should succeed");

        let bytes = result.unwrap();
        assert!(!bytes.is_empty(), "Serialized bytes should not be empty");

        // Verify MVER magic
        assert_eq!(&bytes[0..4], b"REVM");
    }

    #[test]
    fn test_write_to_file_integration() {
        let adt = create_minimal_built_adt();

        // Use temporary file
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("test_adt_write.adt");

        let result = adt.write_to_file(&temp_path);
        assert!(result.is_ok(), "Write to file should succeed");

        // Verify file exists and has content
        let metadata = std::fs::metadata(&temp_path);
        assert!(metadata.is_ok(), "File should exist");
        assert!(metadata.unwrap().len() > 0, "File should not be empty");

        // Cleanup
        let _ = std::fs::remove_file(&temp_path);
    }
}
