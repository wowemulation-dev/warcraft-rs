// validator.rs - Comprehensive ADT validation

use crate::Adt;
use crate::error::{AdtError, Result};
use crate::split_adt::SplitAdtType;
use crate::version::AdtVersion;
use std::collections::HashSet;
use std::path::Path;

/// Validation levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ValidationLevel {
    /// Basic structure validation
    Basic,
    /// Standard validation with consistency checks
    #[default]
    Standard,
    /// Strict validation with all checks enabled
    Strict,
}

/// Validate an ADT file
pub fn validate_adt(adt: &Adt, level: ValidationLevel) -> Result<ValidationReport> {
    validate_adt_with_context(adt, level, None::<&Path>)
}

/// Validate an ADT file with file context
pub fn validate_adt_with_context<P: AsRef<Path>>(
    adt: &Adt,
    level: ValidationLevel,
    file_path: Option<P>,
) -> Result<ValidationReport> {
    let mut report = ValidationReport::new();

    // Determine file type if path is provided
    let file_type = file_path
        .as_ref()
        .map(|p| SplitAdtType::from_filename(&p.as_ref().to_string_lossy()));

    // Basic structure validation (always performed)
    basic_validation(adt, &mut report, file_type)?;

    // Return early if only basic validation is requested
    if level == ValidationLevel::Basic {
        return Ok(report);
    }

    // Cross-reference validation
    xref_validation(adt, &mut report)?;

    // Strict validation checks
    if level == ValidationLevel::Strict {
        strict_validation(adt, &mut report)?;
    }

    Ok(report)
}

/// Basic structure validation
fn basic_validation(
    adt: &Adt,
    report: &mut ValidationReport,
    file_type: Option<SplitAdtType>,
) -> Result<()> {
    // For split ADT files, different validation rules apply
    match file_type {
        Some(SplitAdtType::Obj0) | Some(SplitAdtType::Obj1) => {
            // Object files don't require MHDR or MCNK chunks
            // They contain object placement data (MMDX, MMID, MWMO, MWID, MDDF, MODF)
            return Ok(());
        }
        Some(SplitAdtType::Tex0) | Some(SplitAdtType::Tex1) => {
            // Texture files don't require MHDR or MCNK chunks
            // They contain texture data (MTEX and texture-related MCNK subchunks)
            return Ok(());
        }
        Some(SplitAdtType::Lod) => {
            // LOD files have different requirements
            return Ok(());
        }
        _ => {
            // Root ADT file - apply normal validation
        }
    }

    // Check for required chunks in root ADT files
    if adt.mhdr.is_none() {
        report.add_error("Missing MHDR chunk".to_string());
        return Err(AdtError::MissingChunk("MHDR".to_string()));
    }

    // Check for MCNK chunks - should be 256 (16x16) for a complete map tile
    if adt.mcnk_chunks.is_empty() {
        report.add_error("No MCNK chunks found".to_string());
        return Err(AdtError::ValidationError(
            "No MCNK chunks found".to_string(),
        ));
    }

    if adt.mcnk_chunks.len() != 256 {
        report.add_warning(format!(
            "Expected 256 MCNK chunks for a complete map tile, found {}",
            adt.mcnk_chunks.len()
        ));
    }

    // Version-specific validation
    match adt.version() {
        AdtVersion::TBC | AdtVersion::WotLK | AdtVersion::Cataclysm => {
            // Validate that MHDR has appropriate offsets for the version
            if let Some(ref mhdr) = adt.mhdr {
                if adt.version() >= AdtVersion::TBC && mhdr.mfbo_offset.is_none() {
                    report.add_warning("TBC+ ADT should have MFBO offset in MHDR".to_string());
                }

                if adt.version() >= AdtVersion::WotLK && mhdr.mh2o_offset.is_none() {
                    report.add_warning("WotLK+ ADT should have MH2O offset in MHDR".to_string());
                }

                if adt.version() >= AdtVersion::Cataclysm && mhdr.mtfx_offset.is_none() {
                    report
                        .add_warning("Cataclysm+ ADT should have MTFX offset in MHDR".to_string());
                }
            }
        }
        _ => {}
    }

    // Validate MHDR offsets vs actual data
    if let Some(ref mhdr) = adt.mhdr {
        if mhdr.mcin_offset > 0 && adt.mcin.is_none() {
            report.add_warning("MHDR has MCIN offset but no MCIN chunk found".to_string());
        }

        if mhdr.mtex_offset > 0 && adt.mtex.is_none() {
            report.add_warning("MHDR has MTEX offset but no MTEX chunk found".to_string());
        }

        if mhdr.mmdx_offset > 0 && adt.mmdx.is_none() {
            report.add_warning("MHDR has MMDX offset but no MMDX chunk found".to_string());
        }
    }

    Ok(())
}

/// Cross-reference validation
fn xref_validation(adt: &Adt, report: &mut ValidationReport) -> Result<()> {
    // Validate MCNK indices
    for (i, chunk) in adt.mcnk_chunks.iter().enumerate() {
        let expected_x = (i % 16) as u32;
        let expected_y = (i / 16) as u32;

        if chunk.ix != expected_x || chunk.iy != expected_y {
            report.add_warning(format!(
                "MCNK chunk at index {} has incorrect indices [{}, {}], expected [{}, {}]",
                i, chunk.ix, chunk.iy, expected_x, expected_y
            ));
        }
    }

    // Validate MCIN indices point to valid MCNK chunks
    if let Some(ref mcin) = adt.mcin {
        for (i, entry) in mcin.entries.iter().enumerate() {
            if entry.offset == 0 && entry.size == 0 {
                // Empty entry
                continue;
            }

            // In a file, the offset would be relative to the file start
            // But we don't know the actual offset here, so we can only check
            // if the size seems reasonable
            if entry.size < 8 || entry.size > 1024 * 1024 {
                report.add_warning(format!(
                    "MCIN entry {} has suspicious size: {}",
                    i, entry.size
                ));
            }
        }
    }

    // Validate MMID references valid indices in MMDX
    if let Some(ref _mmdx) = adt.mmdx {
        if let Some(ref mmid) = adt.mmid {
            for (i, &offset) in mmid.offsets.iter().enumerate() {
                let found = false;

                // Ideally, we would check if each offset points to a valid position
                // in the MMDX chunk, but we don't have the raw data here

                if !found {
                    report.add_info(format!(
                        "MMID entry {i} references offset {offset} in MMDX"
                    ));
                }
            }
        }
    }

    // Validate MWID references valid indices in MWMO
    if let Some(ref _mwmo) = adt.mwmo {
        if let Some(ref mwid) = adt.mwid {
            for (i, &offset) in mwid.offsets.iter().enumerate() {
                let found = false;

                // Similar to MMID, we would need the raw data to properly validate

                if !found {
                    report.add_info(format!(
                        "MWID entry {i} references offset {offset} in MWMO"
                    ));
                }
            }
        }
    }

    // Validate MDDF references valid MMID indices
    if let Some(ref mddf) = adt.mddf {
        if let Some(ref mmid) = adt.mmid {
            for (i, doodad) in mddf.doodads.iter().enumerate() {
                if doodad.name_id as usize >= mmid.offsets.len() {
                    report.add_error(format!(
                        "MDDF entry {} references invalid MMID index: {}",
                        i, doodad.name_id
                    ));
                }
            }
        } else if !mddf.doodads.is_empty() {
            report.add_error("MDDF references doodads but no MMID chunk found".to_string());
        }
    }

    // Validate MODF references valid MWID indices
    if let Some(ref modf) = adt.modf {
        if let Some(ref mwid) = adt.mwid {
            for (i, model) in modf.models.iter().enumerate() {
                if model.name_id as usize >= mwid.offsets.len() {
                    report.add_error(format!(
                        "MODF entry {} references invalid MWID index: {}",
                        i, model.name_id
                    ));
                }
            }
        } else if !modf.models.is_empty() {
            report.add_error("MODF references WMOs but no MWID chunk found".to_string());
        }
    }

    // Validate texture references
    if let Some(ref mtex) = adt.mtex {
        let texture_count = mtex.filenames.len();

        for (i, chunk) in adt.mcnk_chunks.iter().enumerate() {
            for (j, layer) in chunk.texture_layers.iter().enumerate() {
                if layer.texture_id as usize >= texture_count {
                    report.add_error(format!(
                        "MCNK chunk {}, layer {} references invalid texture ID: {}",
                        i, j, layer.texture_id
                    ));
                }
            }
        }
    }

    // Validate MCNK doodad references
    for (i, chunk) in adt.mcnk_chunks.iter().enumerate() {
        if !chunk.doodad_refs.is_empty() {
            if let Some(ref mmid) = adt.mmid {
                for (j, &doodad_ref) in chunk.doodad_refs.iter().enumerate() {
                    if doodad_ref as usize >= mmid.offsets.len() {
                        report.add_error(format!(
                            "MCNK chunk {i}, doodad ref {j} references invalid MMID index: {doodad_ref}"
                        ));
                    }
                }
            } else {
                report.add_error(format!(
                    "MCNK chunk {i} references doodads but no MMID chunk found"
                ));
            }
        }

        if !chunk.map_obj_refs.is_empty() {
            if let Some(ref mwid) = adt.mwid {
                for (j, &map_obj_ref) in chunk.map_obj_refs.iter().enumerate() {
                    if map_obj_ref as usize >= mwid.offsets.len() {
                        report.add_error(format!(
                            "MCNK chunk {i}, map object ref {j} references invalid MWID index: {map_obj_ref}"
                        ));
                    }
                }
            } else {
                report.add_error(format!(
                    "MCNK chunk {i} references map objects but no MWID chunk found"
                ));
            }
        }
    }

    Ok(())
}

/// Strict validation with detailed checks
fn strict_validation(adt: &Adt, report: &mut ValidationReport) -> Result<()> {
    // Check for unique IDs in doodad placements
    if let Some(ref mddf) = adt.mddf {
        let mut doodad_ids = HashSet::new();

        for (i, doodad) in mddf.doodads.iter().enumerate() {
            if !doodad_ids.insert(doodad.unique_id) {
                report.add_warning(format!(
                    "MDDF entry {} has duplicate unique ID: {}",
                    i, doodad.unique_id
                ));
            }
        }
    }

    // Check for unique IDs in model placements
    if let Some(ref modf) = adt.modf {
        let mut model_ids = HashSet::new();

        for (i, model) in modf.models.iter().enumerate() {
            if !model_ids.insert(model.unique_id) {
                report.add_warning(format!(
                    "MODF entry {} has duplicate unique ID: {}",
                    i, model.unique_id
                ));
            }
        }
    }

    // Check for holes consistency
    for (i, chunk) in adt.mcnk_chunks.iter().enumerate() {
        if chunk.holes != 0 {
            // Holes are stored as a bit field where each bit represents
            // a triangle in the terrain mesh.
            // We could do more detailed validation here.
            report.add_info(format!("MCNK chunk {} has holes: {:#06x}", i, chunk.holes));
        }
    }

    // Version-specific strict validation
    match adt.version() {
        AdtVersion::WotLK | AdtVersion::Cataclysm => {
            // Validate MH2O consistency
            if let Some(ref mh2o) = adt.mh2o {
                if mh2o.chunks.len() != 256 {
                    report.add_error(format!(
                        "MH2O has {} chunks, expected 256",
                        mh2o.chunks.len()
                    ));
                }

                // More detailed MH2O validation could be done here
            }
        }
        _ => {}
    }

    Ok(())
}

/// Report of validation results
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Errors (fatal issues)
    pub errors: Vec<String>,
    /// Warnings (potential issues)
    pub warnings: Vec<String>,
    /// Informational messages
    pub info: Vec<String>,
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationReport {
    /// Create a new validation report
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
        }
    }

    /// Add an error message
    pub fn add_error(&mut self, message: String) {
        self.errors.push(message);
    }

    /// Add a warning message
    pub fn add_warning(&mut self, message: String) {
        self.warnings.push(message);
    }

    /// Add an info message
    pub fn add_info(&mut self, message: String) {
        self.info.push(message);
    }

    /// Check if the validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Check if the validation passed without warnings
    pub fn is_clean(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }

    /// Format the report as a string
    pub fn format(&self) -> String {
        let mut result = String::new();

        if self.is_valid() {
            result.push_str("Validation passed");
            if !self.warnings.is_empty() {
                result.push_str(" with warnings");
            }
            result.push_str(".\n\n");
        } else {
            result.push_str(&format!(
                "Validation failed with {} errors.\n\n",
                self.errors.len()
            ));
        }

        if !self.errors.is_empty() {
            result.push_str("Errors:\n");
            for (i, error) in self.errors.iter().enumerate() {
                result.push_str(&format!("  {}. {}\n", i + 1, error));
            }
            result.push('\n');
        }

        if !self.warnings.is_empty() {
            result.push_str("Warnings:\n");
            for (i, warning) in self.warnings.iter().enumerate() {
                result.push_str(&format!("  {}. {}\n", i + 1, warning));
            }
            result.push('\n');
        }

        if !self.info.is_empty() {
            result.push_str("Info:\n");
            for (i, info) in self.info.iter().enumerate() {
                result.push_str(&format!("  {}. {}\n", i + 1, info));
            }
        }

        result
    }
}
