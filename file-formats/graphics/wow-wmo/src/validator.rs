use crate::error::Result;
use crate::version::WmoVersion;
use crate::wmo_group_types::WmoGroup;
use crate::wmo_types::{WmoFlags, WmoRoot};

// Use WmoGroupFlags from wmo_group_types since that's where WmoGroupHeader uses it
use crate::wmo_group_types::WmoGroupFlags;

/// Validator for WMO files
pub struct WmoValidator;

impl Default for WmoValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl WmoValidator {
    /// Create a new WMO validator
    pub fn new() -> Self {
        Self
    }

    /// Validate a WMO root file
    pub fn validate_root(&self, wmo: &WmoRoot) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();

        // Check version supported
        if wmo.version < WmoVersion::min_supported() || wmo.version > WmoVersion::max_supported() {
            report.add_error(ValidationError::UnsupportedVersion(wmo.version.to_raw()));
        }

        // Validate header counts match actual counts
        if wmo.materials.len() != wmo.header.n_materials as usize {
            report.add_error(ValidationError::CountMismatch {
                field: "materials".to_string(),
                expected: wmo.header.n_materials,
                actual: wmo.materials.len() as u32,
            });
        }

        if wmo.groups.len() != wmo.header.n_groups as usize {
            report.add_error(ValidationError::CountMismatch {
                field: "groups".to_string(),
                expected: wmo.header.n_groups,
                actual: wmo.groups.len() as u32,
            });
        }

        if wmo.portals.len() != wmo.header.n_portals as usize {
            report.add_error(ValidationError::CountMismatch {
                field: "portals".to_string(),
                expected: wmo.header.n_portals,
                actual: wmo.portals.len() as u32,
            });
        }

        if wmo.lights.len() != wmo.header.n_lights as usize {
            report.add_error(ValidationError::CountMismatch {
                field: "lights".to_string(),
                expected: wmo.header.n_lights,
                actual: wmo.lights.len() as u32,
            });
        }

        if wmo.doodad_defs.len() != wmo.header.n_doodad_defs as usize {
            report.add_error(ValidationError::CountMismatch {
                field: "doodad_defs".to_string(),
                expected: wmo.header.n_doodad_defs,
                actual: wmo.doodad_defs.len() as u32,
            });
        }

        if wmo.doodad_sets.len() != wmo.header.n_doodad_sets as usize {
            report.add_error(ValidationError::CountMismatch {
                field: "doodad_sets".to_string(),
                expected: wmo.header.n_doodad_sets,
                actual: wmo.doodad_sets.len() as u32,
            });
        }

        // Validate material references
        for (i, material) in wmo.materials.iter().enumerate() {
            // Check texture indices are valid
            // Special values above 0xFF000000 are used as markers (e.g., no texture)
            const SPECIAL_TEXTURE_THRESHOLD: u32 = 0xFF000000;

            if material.texture1 != 0
                && material.texture1 < SPECIAL_TEXTURE_THRESHOLD
                && material.texture1 as usize >= wmo.textures.len()
            {
                report.add_error(ValidationError::InvalidReference {
                    field: format!("material[{i}].texture1"),
                    value: material.texture1,
                    max: wmo.textures.len() as u32 - 1,
                });
            }

            if material.texture2 != 0
                && material.texture2 < SPECIAL_TEXTURE_THRESHOLD
                && material.texture2 as usize >= wmo.textures.len()
            {
                report.add_error(ValidationError::InvalidReference {
                    field: format!("material[{i}].texture2"),
                    value: material.texture2,
                    max: wmo.textures.len() as u32 - 1,
                });
            }

            // Check shader ID is valid
            // Special values above 0xFF000000 are used as markers
            if material.shader > 20 && material.shader < SPECIAL_TEXTURE_THRESHOLD {
                report.add_warning(ValidationWarning::UnusualValue {
                    field: format!("material[{i}].shader"),
                    value: material.shader,
                    explanation: "Shader ID is unusually high".to_string(),
                });
            }
        }

        // Validate doodad sets
        for (i, set) in wmo.doodad_sets.iter().enumerate() {
            let end_index = set.start_doodad + set.n_doodads;
            if end_index > wmo.doodad_defs.len() as u32 {
                report.add_error(ValidationError::InvalidReference {
                    field: format!("doodad_set[{i}]"),
                    value: end_index,
                    max: wmo.doodad_defs.len() as u32,
                });
            }
        }

        // Validate portal references
        for (i, portal_ref) in wmo.portal_references.iter().enumerate() {
            if portal_ref.portal_index as usize >= wmo.portals.len() {
                report.add_error(ValidationError::InvalidReference {
                    field: format!("portal_reference[{i}].portal_index"),
                    value: portal_ref.portal_index as u32,
                    max: wmo.portals.len() as u32 - 1,
                });
            }

            if portal_ref.group_index as usize >= wmo.groups.len() {
                report.add_error(ValidationError::InvalidReference {
                    field: format!("portal_reference[{i}].group_index"),
                    value: portal_ref.group_index as u32,
                    max: wmo.groups.len() as u32 - 1,
                });
            }

            if portal_ref.side > 1 {
                report.add_error(ValidationError::InvalidValue {
                    field: format!("portal_reference[{i}].side"),
                    value: portal_ref.side as u32,
                    explanation: "Portal side must be 0 or 1".to_string(),
                });
            }
        }

        // Check skybox flag consistency
        if wmo.header.flags.contains(WmoFlags::HAS_SKYBOX) && wmo.skybox.is_none() {
            report.add_warning(ValidationWarning::FlagInconsistency {
                flag: "HAS_SKYBOX".to_string(),
                field: "skybox".to_string(),
                explanation: "HAS_SKYBOX flag is set but no skybox model is defined".to_string(),
            });
        }

        if !wmo.header.flags.contains(WmoFlags::HAS_SKYBOX) && wmo.skybox.is_some() {
            report.add_warning(ValidationWarning::FlagInconsistency {
                flag: "HAS_SKYBOX".to_string(),
                field: "skybox".to_string(),
                explanation: "Skybox model is defined but HAS_SKYBOX flag is not set".to_string(),
            });
        }

        // Check for portals with no vertices
        for (i, portal) in wmo.portals.iter().enumerate() {
            if portal.vertices.is_empty() {
                report.add_warning(ValidationWarning::UnusualStructure {
                    field: format!("portal[{i}]"),
                    explanation: "Portal has no vertices".to_string(),
                });
            }
        }

        // Check for empty or missing visible block lists
        if wmo.visible_block_lists.is_empty() && !wmo.portals.is_empty() {
            report.add_warning(ValidationWarning::MissingData {
                field: "visible_block_lists".to_string(),
                explanation: "No visible block lists defined but portals exist".to_string(),
            });
        }

        // Check for non-normalized normals in portals
        for (i, portal) in wmo.portals.iter().enumerate() {
            let normal = &portal.normal;
            let length_squared = normal.x * normal.x + normal.y * normal.y + normal.z * normal.z;

            // Check if length is significantly different from 1.0
            if (length_squared - 1.0).abs() > 0.01 {
                report.add_warning(ValidationWarning::UnusualValue {
                    field: format!("portal[{i}].normal"),
                    value: length_squared as u32,
                    explanation: "Portal normal is not normalized".to_string(),
                });
            }
        }

        // Check bounding box validity
        if wmo.bounding_box.min.x > wmo.bounding_box.max.x
            || wmo.bounding_box.min.y > wmo.bounding_box.max.y
            || wmo.bounding_box.min.z > wmo.bounding_box.max.z
        {
            report.add_error(ValidationError::InvalidBoundingBox {
                min: format!(
                    "({}, {}, {})",
                    wmo.bounding_box.min.x, wmo.bounding_box.min.y, wmo.bounding_box.min.z
                ),
                max: format!(
                    "({}, {}, {})",
                    wmo.bounding_box.max.x, wmo.bounding_box.max.y, wmo.bounding_box.max.z
                ),
            });
        }

        Ok(report)
    }

    /// Validate a WMO group file
    pub fn validate_group(&self, group: &WmoGroup) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();

        // Check for empty vertices
        if group.vertices.is_empty() {
            report.add_error(ValidationError::EmptyData {
                field: "vertices".to_string(),
                explanation: "Group has no vertices".to_string(),
            });
        }

        // Check for empty indices
        if group.indices.is_empty() {
            report.add_error(ValidationError::EmptyData {
                field: "indices".to_string(),
                explanation: "Group has no indices".to_string(),
            });
        }

        // Check for empty batches
        if group.batches.is_empty() {
            report.add_warning(ValidationWarning::EmptyData {
                field: "batches".to_string(),
                explanation: "Group has no batches".to_string(),
            });
        }

        // Check index references
        for (i, batch) in group.batches.iter().enumerate() {
            let end_index = batch.start_index + batch.count as u32;
            if end_index > group.indices.len() as u32 {
                report.add_error(ValidationError::InvalidReference {
                    field: format!("batch[{i}].indices"),
                    value: end_index,
                    max: group.indices.len() as u32,
                });
            }

            if batch.end_vertex as usize > group.vertices.len() {
                report.add_error(ValidationError::InvalidReference {
                    field: format!("batch[{i}].end_vertex"),
                    value: batch.end_vertex as u32,
                    max: group.vertices.len() as u32,
                });
            }
        }

        // Check vertex indices are in range
        for (i, &index) in group.indices.iter().enumerate() {
            if index as usize >= group.vertices.len() {
                report.add_error(ValidationError::InvalidReference {
                    field: format!("indices[{i}]"),
                    value: index as u32,
                    max: group.vertices.len() as u32 - 1,
                });
            }
        }

        // Check normals count matches vertices if present
        if !group.normals.is_empty() && group.normals.len() != group.vertices.len() {
            report.add_error(ValidationError::CountMismatch {
                field: "normals".to_string(),
                expected: group.vertices.len() as u32,
                actual: group.normals.len() as u32,
            });
        }

        // Check texture coordinates count matches vertices if present
        if !group.tex_coords.is_empty() && group.tex_coords.len() != group.vertices.len() {
            report.add_error(ValidationError::CountMismatch {
                field: "tex_coords".to_string(),
                expected: group.vertices.len() as u32,
                actual: group.tex_coords.len() as u32,
            });
        }

        // Check vertex colors count matches vertices if present
        if let Some(colors) = &group.vertex_colors
            && colors.len() != group.vertices.len()
        {
            report.add_error(ValidationError::CountMismatch {
                field: "vertex_colors".to_string(),
                expected: group.vertices.len() as u32,
                actual: colors.len() as u32,
            });
        }

        // Check flags consistency for normals
        if group.header.flags.contains(WmoGroupFlags::HAS_NORMALS) && group.normals.is_empty() {
            report.add_warning(ValidationWarning::FlagInconsistency {
                flag: "HAS_NORMALS".to_string(),
                field: "normals".to_string(),
                explanation: "HAS_NORMALS flag is set but no normals are present".to_string(),
            });
        }

        if !group.header.flags.contains(WmoGroupFlags::HAS_NORMALS) && !group.normals.is_empty() {
            report.add_warning(ValidationWarning::FlagInconsistency {
                flag: "HAS_NORMALS".to_string(),
                field: "normals".to_string(),
                explanation: "Normals are present but HAS_NORMALS flag is not set".to_string(),
            });
        }

        // Check flags consistency for vertex colors
        if group
            .header
            .flags
            .contains(WmoGroupFlags::HAS_VERTEX_COLORS)
            && group.vertex_colors.is_none()
        {
            report.add_warning(ValidationWarning::FlagInconsistency {
                flag: "HAS_VERTEX_COLORS".to_string(),
                field: "vertex_colors".to_string(),
                explanation: "HAS_VERTEX_COLORS flag is set but no vertex colors are present"
                    .to_string(),
            });
        }

        if !group
            .header
            .flags
            .contains(WmoGroupFlags::HAS_VERTEX_COLORS)
            && group.vertex_colors.is_some()
        {
            report.add_warning(ValidationWarning::FlagInconsistency {
                flag: "HAS_VERTEX_COLORS".to_string(),
                field: "vertex_colors".to_string(),
                explanation: "Vertex colors are present but HAS_VERTEX_COLORS flag is not set"
                    .to_string(),
            });
        }

        // Check flags consistency for doodads
        if group.header.flags.contains(WmoGroupFlags::HAS_DOODADS) && group.doodad_refs.is_none() {
            report.add_warning(ValidationWarning::FlagInconsistency {
                flag: "HAS_DOODADS".to_string(),
                field: "doodad_refs".to_string(),
                explanation: "HAS_DOODADS flag is set but no doodad references are present"
                    .to_string(),
            });
        }

        if !group.header.flags.contains(WmoGroupFlags::HAS_DOODADS) && group.doodad_refs.is_some() {
            report.add_warning(ValidationWarning::FlagInconsistency {
                flag: "HAS_DOODADS".to_string(),
                field: "doodad_refs".to_string(),
                explanation: "Doodad references are present but HAS_DOODADS flag is not set"
                    .to_string(),
            });
        }

        // Check flags consistency for water
        if group.header.flags.contains(WmoGroupFlags::HAS_WATER) && group.liquid.is_none() {
            report.add_warning(ValidationWarning::FlagInconsistency {
                flag: "HAS_WATER".to_string(),
                field: "liquid".to_string(),
                explanation: "HAS_WATER flag is set but no liquid data is present".to_string(),
            });
        }

        if !group.header.flags.contains(WmoGroupFlags::HAS_WATER) && group.liquid.is_some() {
            report.add_warning(ValidationWarning::FlagInconsistency {
                flag: "HAS_WATER".to_string(),
                field: "liquid".to_string(),
                explanation: "Liquid data is present but HAS_WATER flag is not set".to_string(),
            });
        }

        // Check bounding box validity
        if group.header.bounding_box.min.x > group.header.bounding_box.max.x
            || group.header.bounding_box.min.y > group.header.bounding_box.max.y
            || group.header.bounding_box.min.z > group.header.bounding_box.max.z
        {
            report.add_error(ValidationError::InvalidBoundingBox {
                min: format!(
                    "({}, {}, {})",
                    group.header.bounding_box.min.x,
                    group.header.bounding_box.min.y,
                    group.header.bounding_box.min.z
                ),
                max: format!(
                    "({}, {}, {})",
                    group.header.bounding_box.max.x,
                    group.header.bounding_box.max.y,
                    group.header.bounding_box.max.z
                ),
            });
        }

        // Check if vertices fit in bounding box
        for (i, vertex) in group.vertices.iter().enumerate() {
            if vertex.x < group.header.bounding_box.min.x
                || vertex.x > group.header.bounding_box.max.x
                || vertex.y < group.header.bounding_box.min.y
                || vertex.y > group.header.bounding_box.max.y
                || vertex.z < group.header.bounding_box.min.z
                || vertex.z > group.header.bounding_box.max.z
            {
                report.add_warning(ValidationWarning::OutOfBounds {
                    field: format!("vertex[{i}]"),
                    value: format!("({}, {}, {})", vertex.x, vertex.y, vertex.z),
                    bounds: format!(
                        "({}, {}, {}) - ({}, {}, {})",
                        group.header.bounding_box.min.x,
                        group.header.bounding_box.min.y,
                        group.header.bounding_box.min.z,
                        group.header.bounding_box.max.x,
                        group.header.bounding_box.max.y,
                        group.header.bounding_box.max.z
                    ),
                });
            }
        }

        Ok(report)
    }
}

/// Report of validation results
#[derive(Debug)]
pub struct ValidationReport {
    /// Validation errors (severe issues)
    pub errors: Vec<ValidationError>,

    /// Validation warnings (potential issues)
    pub warnings: Vec<ValidationWarning>,
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationReport {
    /// Create a new empty validation report
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add an error to the report
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Add a warning to the report
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// Check if the report has any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if the report has any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Count the number of errors
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Count the number of warnings
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Print the report to the console
    pub fn print(&self) {
        if self.has_errors() {
            println!("Validation Errors:");
            for error in &self.errors {
                println!("  - {error}");
            }
        }

        if self.has_warnings() {
            println!("Validation Warnings:");
            for warning in &self.warnings {
                println!("  - {warning}");
            }
        }

        if !self.has_errors() && !self.has_warnings() {
            println!("No validation issues found.");
        }
    }
}

/// Validation error types
#[derive(Debug)]
pub enum ValidationError {
    /// Unsupported WMO version
    UnsupportedVersion(u32),

    /// Count mismatch between header and actual data
    CountMismatch {
        field: String,
        expected: u32,
        actual: u32,
    },

    /// Invalid reference
    InvalidReference { field: String, value: u32, max: u32 },

    /// Invalid value
    InvalidValue {
        field: String,
        value: u32,
        explanation: String,
    },

    /// Invalid bounding box
    InvalidBoundingBox { min: String, max: String },

    /// Empty required data
    EmptyData { field: String, explanation: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedVersion(version) => write!(f, "Unsupported WMO version: {version}"),
            Self::CountMismatch {
                field,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Count mismatch for {field}: expected {expected}, found {actual}"
                )
            }
            Self::InvalidReference { field, value, max } => {
                write!(
                    f,
                    "Invalid reference in {field}: {value}, max allowed: {max}"
                )
            }
            Self::InvalidValue {
                field,
                value,
                explanation,
            } => {
                write!(f, "Invalid value in {field}: {value} ({explanation})")
            }
            Self::InvalidBoundingBox { min, max } => {
                write!(f, "Invalid bounding box: min {min} exceeds max {max}")
            }
            Self::EmptyData { field, explanation } => {
                write!(f, "Empty data for {field}: {explanation}")
            }
        }
    }
}

/// Validation warning types
#[derive(Debug)]
pub enum ValidationWarning {
    /// Flag inconsistency
    FlagInconsistency {
        flag: String,
        field: String,
        explanation: String,
    },

    /// Unusual value
    UnusualValue {
        field: String,
        value: u32,
        explanation: String,
    },

    /// Unusual structure
    UnusualStructure { field: String, explanation: String },

    /// Missing data
    MissingData { field: String, explanation: String },

    /// Empty data (non-critical)
    EmptyData { field: String, explanation: String },

    /// Out of bounds
    OutOfBounds {
        field: String,
        value: String,
        bounds: String,
    },
}

impl std::fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FlagInconsistency {
                flag,
                field,
                explanation,
            } => {
                write!(f, "Flag inconsistency with {flag}: {field} ({explanation})")
            }
            Self::UnusualValue {
                field,
                value,
                explanation,
            } => {
                write!(f, "Unusual value in {field}: {value} ({explanation})")
            }
            Self::UnusualStructure { field, explanation } => {
                write!(f, "Unusual structure in {field}: {explanation}")
            }
            Self::MissingData { field, explanation } => {
                write!(f, "Missing data for {field}: {explanation}")
            }
            Self::EmptyData { field, explanation } => {
                write!(f, "Empty data for {field}: {explanation}")
            }
            Self::OutOfBounds {
                field,
                value,
                bounds,
            } => {
                write!(f, "{field} is out of bounds: {value} not within {bounds}")
            }
        }
    }
}
