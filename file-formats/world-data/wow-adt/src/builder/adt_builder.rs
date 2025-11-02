//! Core AdtBuilder implementation with fluent API.

use crate::ChunkId;
use crate::builder::built_adt::BuiltAdt;
use crate::builder::validation::{
    validate_blend_mesh_data, validate_doodad_placement_references, validate_model_filename,
    validate_texture_filename, validate_version_chunk_compatibility, validate_wmo_filename,
    validate_wmo_placement_references,
};
use crate::chunks::mh2o::Mh2oChunk;
use crate::chunks::{
    DoodadPlacement, MampChunk, MbbbChunk, MbmhChunk, MbmiChunk, MbnvChunk, McnkChunk, MfboChunk,
    MtxfChunk, MtxpChunk, WmoPlacement,
};
use crate::error::{AdtError, Result};
use crate::version::AdtVersion;

/// Fluent builder for constructing valid ADT terrain files.
///
/// The builder follows a progressive validation strategy:
/// - Method calls validate basic parameters
/// - `build()` validates structural integrity and references
///
/// # Examples
///
/// ```no_run
/// use wow_adt::builder::AdtBuilder;
/// use wow_adt::AdtVersion;
/// use wow_adt::DoodadPlacement;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let adt = AdtBuilder::new()
///     .with_version(AdtVersion::WotLK)
///     .add_texture("terrain/grass_01.blp")
///     .add_texture("terrain/dirt_01.blp")
///     .add_model("doodad/tree_01.m2")
///     .add_doodad_placement(DoodadPlacement {
///         name_id: 0,
///         unique_id: 1,
///         position: [1000.0, 1000.0, 100.0],
///         rotation: [0.0, 0.0, 0.0],
///         scale: 1024,
///         flags: 0,
///     })
///     .build()?;
///
/// adt.write_to_file("world/maps/custom/custom_32_32.adt")?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct AdtBuilder {
    /// Target ADT version
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

impl AdtBuilder {
    /// Create new ADT builder with default version (Vanilla).
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_adt::builder::AdtBuilder;
    ///
    /// let builder = AdtBuilder::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: AdtVersion::VanillaEarly,
            textures: Vec::new(),
            models: Vec::new(),
            wmos: Vec::new(),
            doodad_placements: Vec::new(),
            wmo_placements: Vec::new(),
            mcnk_chunks: Vec::new(),
            flight_bounds: None,
            water_data: None,
            texture_flags: None,
            texture_amplifier: None,
            texture_params: None,
            blend_mesh_headers: None,
            blend_mesh_bounds: None,
            blend_mesh_vertices: None,
            blend_mesh_indices: None,
        }
    }

    /// Set target ADT version.
    ///
    /// Version determines which chunks are valid for this ADT:
    /// - Vanilla: Basic chunks only
    /// - TBC: Adds MFBO (flight bounds)
    /// - WotLK: Adds MH2O (advanced water)
    /// - Cataclysm: Adds MAMP (texture amplifier)
    /// - MoP: Adds MTXP (texture parameters)
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_adt::builder::AdtBuilder;
    /// use wow_adt::AdtVersion;
    ///
    /// let builder = AdtBuilder::new()
    ///     .with_version(AdtVersion::WotLK);
    /// ```
    #[must_use]
    pub fn with_version(mut self, version: AdtVersion) -> Self {
        self.version = version;
        self
    }

    /// Add texture filename to MTEX chunk.
    ///
    /// # Validation
    ///
    /// - Filename not empty
    /// - Uses forward slashes (not backslashes)
    /// - Has .blp extension (case-insensitive)
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_adt::builder::AdtBuilder;
    ///
    /// let builder = AdtBuilder::new()
    ///     .add_texture("terrain/grass_01.blp")
    ///     .add_texture("terrain/dirt_01.blp");
    /// ```
    #[must_use]
    pub fn add_texture<S: Into<String>>(mut self, filename: S) -> Self {
        let filename = filename.into();
        if let Err(e) = validate_texture_filename(&filename) {
            panic!("Invalid texture filename: {}", e);
        }
        self.textures.push(filename);
        self
    }

    /// Add multiple textures at once.
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_adt::builder::AdtBuilder;
    ///
    /// let textures = vec![
    ///     "terrain/grass.blp",
    ///     "terrain/dirt.blp",
    ///     "terrain/rock.blp",
    /// ];
    ///
    /// let builder = AdtBuilder::new()
    ///     .add_textures(textures);
    /// ```
    #[must_use]
    pub fn add_textures<I, S>(mut self, filenames: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for filename in filenames {
            let filename = filename.into();
            if let Err(e) = validate_texture_filename(&filename) {
                panic!("Invalid texture filename: {}", e);
            }
            self.textures.push(filename);
        }
        self
    }

    /// Add M2 model filename to MMDX chunk.
    ///
    /// # Validation
    ///
    /// - Filename not empty
    /// - Uses forward slashes (not backslashes)
    /// - Has .m2 extension (case-insensitive)
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_adt::builder::AdtBuilder;
    ///
    /// let builder = AdtBuilder::new()
    ///     .add_model("doodad/tree_01.m2")
    ///     .add_model("doodad/rock_01.m2");
    /// ```
    #[must_use]
    pub fn add_model<S: Into<String>>(mut self, filename: S) -> Self {
        let filename = filename.into();
        if let Err(e) = validate_model_filename(&filename) {
            panic!("Invalid model filename: {}", e);
        }
        self.models.push(filename);
        self
    }

    /// Add WMO filename to MWMO chunk.
    ///
    /// # Validation
    ///
    /// - Filename not empty
    /// - Uses forward slashes (not backslashes)
    /// - Has .wmo extension (case-insensitive)
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_adt::builder::AdtBuilder;
    ///
    /// let builder = AdtBuilder::new()
    ///     .add_wmo("building/house_01.wmo")
    ///     .add_wmo("building/tower_01.wmo");
    /// ```
    #[must_use]
    pub fn add_wmo<S: Into<String>>(mut self, filename: S) -> Self {
        let filename = filename.into();
        if let Err(e) = validate_wmo_filename(&filename) {
            panic!("Invalid WMO filename: {}", e);
        }
        self.wmos.push(filename);
        self
    }

    /// Add M2 model placement (MDDF entry).
    ///
    /// # Validation
    ///
    /// Reference validation (name_id < model count) is deferred to `build()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_adt::builder::AdtBuilder;
    /// use wow_adt::DoodadPlacement;
    ///
    /// let builder = AdtBuilder::new()
    ///     .add_model("doodad/tree.m2")
    ///     .add_doodad_placement(DoodadPlacement {
    ///         name_id: 0,
    ///         unique_id: 1,
    ///         position: [1000.0, 1000.0, 100.0],
    ///         rotation: [0.0, 0.0, 0.0],
    ///         scale: 1024,
    ///         flags: 0,
    ///     });
    /// ```
    #[must_use]
    pub fn add_doodad_placement(mut self, placement: DoodadPlacement) -> Self {
        if placement.scale == 0 {
            panic!("Doodad placement scale must be greater than 0");
        }
        self.doodad_placements.push(placement);
        self
    }

    /// Add WMO placement (MODF entry).
    ///
    /// # Validation
    ///
    /// Reference validation (name_id < WMO count) is deferred to `build()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_adt::builder::AdtBuilder;
    /// use wow_adt::WmoPlacement;
    ///
    /// let builder = AdtBuilder::new()
    ///     .add_wmo("building/house.wmo")
    ///     .add_wmo_placement(WmoPlacement {
    ///         name_id: 0,
    ///         unique_id: 100,
    ///         position: [2000.0, 2000.0, 50.0],
    ///         rotation: [0.0, 0.0, 0.0],
    ///         extents_min: [-100.0, -100.0, 0.0],
    ///         extents_max: [100.0, 100.0, 200.0],
    ///         flags: 0,
    ///         doodad_set: 0,
    ///         name_set: 0,
    ///         scale: 1024,
    ///     });
    /// ```
    #[must_use]
    pub fn add_wmo_placement(mut self, placement: WmoPlacement) -> Self {
        self.wmo_placements.push(placement);
        self
    }

    /// Add terrain chunk (MCNK).
    ///
    /// # Validation
    ///
    /// - MCNK count ≤ 256 (deferred to `build()`)
    /// - Texture references valid (deferred to `build()`)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wow_adt::builder::AdtBuilder;
    /// use wow_adt::McnkChunk;
    ///
    /// # let chunk = todo!();
    /// let builder = AdtBuilder::new()
    ///     .add_texture("terrain/grass.blp")
    ///     .add_mcnk_chunk(chunk);
    /// ```
    #[must_use]
    pub fn add_mcnk_chunk(mut self, chunk: McnkChunk) -> Self {
        self.mcnk_chunks.push(chunk);
        self
    }

    /// Add flight bounds (MFBO chunk, TBC+).
    ///
    /// # Validation
    ///
    /// Version compatibility is validated in `build()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_adt::builder::AdtBuilder;
    /// use wow_adt::{AdtVersion, MfboChunk};
    ///
    /// let bounds = MfboChunk {
    ///     max_plane: [500, 500, 500, 500, 500, 500, 500, 500, 500],
    ///     min_plane: [0, 0, 0, 0, 0, 0, 0, 0, 0],
    /// };
    ///
    /// let builder = AdtBuilder::new()
    ///     .with_version(AdtVersion::TBC)
    ///     .add_flight_bounds(bounds);
    /// ```
    #[must_use]
    pub fn add_flight_bounds(mut self, bounds: MfboChunk) -> Self {
        self.flight_bounds = Some(bounds);
        self
    }

    /// Add water data (MH2O chunk, WotLK+).
    ///
    /// # Validation
    ///
    /// Version compatibility is validated in `build()`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wow_adt::builder::AdtBuilder;
    /// use wow_adt::{AdtVersion, Mh2oChunk};
    ///
    /// # let water = todo!();
    /// let builder = AdtBuilder::new()
    ///     .with_version(AdtVersion::WotLK)
    ///     .add_water_data(water);
    /// ```
    #[must_use]
    pub fn add_water_data(mut self, water: Mh2oChunk) -> Self {
        self.water_data = Some(water);
        self
    }

    /// Add texture flags (MTXF chunk, WotLK+).
    ///
    /// Texture flags control rendering properties like specularity,
    /// environment mapping, and animation for each texture.
    ///
    /// # Validation
    ///
    /// - Version compatibility is validated in `build()`
    /// - Flag count should match texture count (validated in noggit-red)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wow_adt::builder::AdtBuilder;
    /// use wow_adt::{AdtVersion, chunks::MtxfChunk};
    ///
    /// let flags = MtxfChunk {
    ///     flags: vec![0x01, 0x02, 0x00],
    /// };
    ///
    /// let builder = AdtBuilder::new()
    ///     .with_version(AdtVersion::WotLK)
    ///     .add_texture("terrain/grass.blp")
    ///     .add_texture("terrain/dirt.blp")
    ///     .add_texture("terrain/rock.blp")
    ///     .add_texture_flags(flags);
    /// ```
    #[must_use]
    pub fn add_texture_flags(mut self, flags: MtxfChunk) -> Self {
        self.texture_flags = Some(flags);
        self
    }

    /// Add texture amplifier (MAMP chunk, Cataclysm+).
    ///
    /// # Validation
    ///
    /// Version compatibility is validated in `build()`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wow_adt::builder::AdtBuilder;
    /// use wow_adt::{AdtVersion, MampChunk};
    ///
    /// # let amp = todo!();
    /// let builder = AdtBuilder::new()
    ///     .with_version(AdtVersion::Cataclysm)
    ///     .add_texture_amplifier(amp);
    /// ```
    #[must_use]
    pub fn add_texture_amplifier(mut self, amp: MampChunk) -> Self {
        self.texture_amplifier = Some(amp);
        self
    }

    /// Add texture parameters (MTXP chunk, MoP+).
    ///
    /// # Validation
    ///
    /// Version compatibility is validated in `build()`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wow_adt::builder::AdtBuilder;
    /// use wow_adt::{AdtVersion, MtxpChunk};
    ///
    /// # let params = todo!();
    /// let builder = AdtBuilder::new()
    ///     .with_version(AdtVersion::MoP)
    ///     .add_texture_params(params);
    /// ```
    #[must_use]
    pub fn add_texture_params(mut self, params: MtxpChunk) -> Self {
        self.texture_params = Some(params);
        self
    }

    /// Add blend mesh headers (MoP+).
    ///
    /// Blend mesh headers define metadata for each blend mesh segment including
    /// map object IDs, texture IDs, and index/vertex ranges. Must be provided together
    /// with bounds, vertices, and indices for complete blend mesh support.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wow_adt::builder::AdtBuilder;
    /// use wow_adt::chunks::blend_mesh::MbmhChunk;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let headers = MbmhChunk::default();
    /// let builder = AdtBuilder::new()
    ///     .add_blend_mesh_headers(headers);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn add_blend_mesh_headers(mut self, headers: MbmhChunk) -> Self {
        self.blend_mesh_headers = Some(headers);
        self
    }

    /// Add blend mesh bounding boxes (MoP+).
    ///
    /// Bounding boxes define visibility culling regions for blend mesh segments.
    /// Must be provided together with headers, vertices, and indices.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wow_adt::builder::AdtBuilder;
    /// use wow_adt::chunks::blend_mesh::MbbbChunk;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let bounds = MbbbChunk::default();
    /// let builder = AdtBuilder::new()
    ///     .add_blend_mesh_bounds(bounds);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn add_blend_mesh_bounds(mut self, bounds: MbbbChunk) -> Self {
        self.blend_mesh_bounds = Some(bounds);
        self
    }

    /// Add blend mesh vertices (MoP+).
    ///
    /// Vertices contain position, normal, UV, and color data for blend mesh geometry.
    /// Must be provided together with headers, bounds, and indices.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wow_adt::builder::AdtBuilder;
    /// use wow_adt::chunks::blend_mesh::MbnvChunk;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let vertices = MbnvChunk::default();
    /// let builder = AdtBuilder::new()
    ///     .add_blend_mesh_vertices(vertices);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn add_blend_mesh_vertices(mut self, vertices: MbnvChunk) -> Self {
        self.blend_mesh_vertices = Some(vertices);
        self
    }

    /// Add blend mesh indices (MoP+).
    ///
    /// Indices define triangle connectivity for blend mesh geometry, referencing
    /// vertex data in MBNV. Must be provided together with headers, bounds, and vertices.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wow_adt::builder::AdtBuilder;
    /// use wow_adt::chunks::blend_mesh::MbmiChunk;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let indices = MbmiChunk::default();
    /// let builder = AdtBuilder::new()
    ///     .add_blend_mesh_indices(indices);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn add_blend_mesh_indices(mut self, indices: MbmiChunk) -> Self {
        self.blend_mesh_indices = Some(indices);
        self
    }

    /// Create builder from parsed ADT for modify workflows.
    ///
    /// This method enables load-modify-save patterns:
    /// 1. Parse existing ADT with [`parse_adt()`](crate::api::parse_adt)
    /// 2. Modify fields using mutable access methods
    /// 3. Convert back to builder with `from_parsed()`
    /// 4. Build and write with [`build()`](AdtBuilder::build) → [`write_to_file()`](crate::builder::BuiltAdt::write_to_file)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wow_adt::api::{parse_adt, ParsedAdt};
    /// use wow_adt::builder::AdtBuilder;
    /// use std::fs::File;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Load existing ADT
    /// let mut file = File::open("terrain.adt")?;
    /// let adt = parse_adt(&mut file)?;
    ///
    /// if let ParsedAdt::Root(mut root) = adt {
    ///     // Modify terrain heights
    ///     if let Some(heights) = &mut root.mcnk_chunks_mut()[0].heights {
    ///         for height in &mut heights.heights {
    ///             *height += 10.0;
    ///         }
    ///     }
    ///
    ///     // Convert back to builder and write
    ///     let built = AdtBuilder::from_parsed(*root).build()?;
    ///     built.write_to_file("terrain_modified.adt")?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_parsed(root: crate::api::RootAdt) -> Self {
        Self {
            version: root.version,
            textures: root.textures,
            models: root.models,
            wmos: root.wmos,
            doodad_placements: root.doodad_placements,
            wmo_placements: root.wmo_placements,
            mcnk_chunks: root.mcnk_chunks,
            flight_bounds: root.flight_bounds,
            water_data: root.water_data,
            texture_flags: root.texture_flags,
            texture_amplifier: root.texture_amplifier,
            texture_params: root.texture_params,
            blend_mesh_headers: root.blend_mesh_headers,
            blend_mesh_bounds: root.blend_mesh_bounds,
            blend_mesh_vertices: root.blend_mesh_vertices,
            blend_mesh_indices: root.blend_mesh_indices,
        }
    }

    /// Build validated ADT structure.
    ///
    /// # Validation
    ///
    /// 1. Required chunks present (at least 1 texture, at least 1 MCNK)
    /// 2. MCNK count ≤ 256
    /// 3. All placement references valid (name_id < model/WMO count)
    /// 4. Version-chunk compatibility
    ///
    /// # Errors
    ///
    /// Returns `AdtError` if validation fails:
    /// - `MissingRequiredChunk`: Missing textures or MCNK chunks
    /// - `InvalidModelReference`: Placement references non-existent model
    /// - `ChunkParseError`: Version-chunk incompatibility or invalid counts
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
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self) -> Result<BuiltAdt> {
        // Validate required chunks
        if self.textures.is_empty() {
            return Err(AdtError::MissingRequiredChunk(ChunkId::MTEX));
        }

        // NOTE: MCNK chunks are not required in builder - serializer auto-generates
        // minimal MCNK chunks if none provided. This allows simple ADT creation
        // without requiring full terrain data.

        // Validate MCNK count if any were added
        if self.mcnk_chunks.len() > 256 {
            return Err(AdtError::ChunkParseError {
                chunk: ChunkId::MCNK,
                offset: 0,
                details: format!(
                    "MCNK count {} exceeds maximum 256 terrain chunks",
                    self.mcnk_chunks.len()
                ),
            });
        }

        // Validate placement references
        validate_doodad_placement_references(&self.doodad_placements, self.models.len())?;
        validate_wmo_placement_references(&self.wmo_placements, self.wmos.len())?;

        // Validate version-chunk compatibility
        if let Some(_bounds) = &self.flight_bounds {
            validate_version_chunk_compatibility(self.version, ChunkId::MFBO)?;
        }
        if let Some(_water) = &self.water_data {
            validate_version_chunk_compatibility(self.version, ChunkId::MH2O)?;
        }
        if let Some(_amp) = &self.texture_amplifier {
            validate_version_chunk_compatibility(self.version, ChunkId::MAMP)?;
        }
        if let Some(_params) = &self.texture_params {
            validate_version_chunk_compatibility(self.version, ChunkId::MTXP)?;
        }

        // Validate blend mesh data completeness and consistency
        validate_blend_mesh_data(
            &self.blend_mesh_headers,
            &self.blend_mesh_bounds,
            &self.blend_mesh_vertices,
            &self.blend_mesh_indices,
        )?;

        // Validate blend mesh version compatibility (any one chunk present means all present)
        if self.blend_mesh_headers.is_some() {
            validate_version_chunk_compatibility(self.version, ChunkId::MBMH)?;
        }

        // Build validated ADT
        Ok(BuiltAdt::new(
            self.version,
            self.textures,
            self.models,
            self.wmos,
            self.doodad_placements,
            self.wmo_placements,
            self.mcnk_chunks,
            self.flight_bounds,
            self.water_data,
            self.texture_flags,
            self.texture_amplifier,
            self.texture_params,
            self.blend_mesh_headers,
            self.blend_mesh_bounds,
            self.blend_mesh_vertices,
            self.blend_mesh_indices,
        ))
    }
}

impl Default for AdtBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_builder_defaults_to_vanilla() {
        let builder = AdtBuilder::new();
        assert_eq!(builder.version, AdtVersion::VanillaEarly);
    }

    #[test]
    fn test_with_version() {
        let builder = AdtBuilder::new().with_version(AdtVersion::WotLK);
        assert_eq!(builder.version, AdtVersion::WotLK);
    }

    #[test]
    fn test_add_texture_valid() {
        let builder = AdtBuilder::new().add_texture("terrain/grass.blp");
        assert_eq!(builder.textures.len(), 1);
        assert_eq!(builder.textures[0], "terrain/grass.blp");
    }

    #[test]
    #[should_panic(expected = "Invalid texture filename")]
    fn test_add_texture_empty() {
        let _builder = AdtBuilder::new().add_texture("");
    }

    #[test]
    #[should_panic(expected = "Invalid texture filename")]
    fn test_add_texture_backslash() {
        let _builder = AdtBuilder::new().add_texture("terrain\\grass.blp");
    }

    #[test]
    fn test_add_textures() {
        let textures = vec!["terrain/grass.blp", "terrain/dirt.blp"];
        let builder = AdtBuilder::new().add_textures(textures);
        assert_eq!(builder.textures.len(), 2);
    }

    #[test]
    fn test_add_model_valid() {
        let builder = AdtBuilder::new().add_model("doodad/tree.m2");
        assert_eq!(builder.models.len(), 1);
        assert_eq!(builder.models[0], "doodad/tree.m2");
    }

    #[test]
    #[should_panic(expected = "Invalid model filename")]
    fn test_add_model_wrong_extension() {
        let _builder = AdtBuilder::new().add_model("doodad/tree.mdx");
    }

    #[test]
    fn test_add_wmo_valid() {
        let builder = AdtBuilder::new().add_wmo("building/house.wmo");
        assert_eq!(builder.wmos.len(), 1);
        assert_eq!(builder.wmos[0], "building/house.wmo");
    }

    #[test]
    fn test_add_doodad_placement() {
        let placement = DoodadPlacement {
            name_id: 0,
            unique_id: 1,
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: 1024,
            flags: 0,
        };

        let builder = AdtBuilder::new().add_doodad_placement(placement);
        assert_eq!(builder.doodad_placements.len(), 1);
    }

    #[test]
    #[should_panic(expected = "scale must be greater than 0")]
    fn test_add_doodad_placement_zero_scale() {
        let placement = DoodadPlacement {
            name_id: 0,
            unique_id: 1,
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: 0,
            flags: 0,
        };

        let _builder = AdtBuilder::new().add_doodad_placement(placement);
    }

    #[test]
    fn test_add_wmo_placement() {
        let placement = WmoPlacement {
            name_id: 0,
            unique_id: 1,
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            extents_min: [-100.0, -100.0, 0.0],
            extents_max: [100.0, 100.0, 200.0],
            flags: 0,
            doodad_set: 0,
            name_set: 0,
            scale: 1024,
        };

        let builder = AdtBuilder::new().add_wmo_placement(placement);
        assert_eq!(builder.wmo_placements.len(), 1);
    }

    #[test]
    fn test_build_missing_texture() {
        let builder = AdtBuilder::new();
        let result = builder.build();
        assert!(matches!(
            result,
            Err(AdtError::MissingRequiredChunk(ChunkId::MTEX))
        ));
    }

    #[test]
    fn test_default() {
        let builder = AdtBuilder::default();
        assert_eq!(builder.version, AdtVersion::VanillaEarly);
    }
}
