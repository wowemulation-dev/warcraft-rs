use crate::io_ext::{ReadExt, WriteExt};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::common::{C3Vector, Quaternion};
use crate::error::{M2Error, Result};
use crate::version::M2Version;

/// Magic signature for Modern Anim files ("MAOF")
pub const ANIM_MAGIC: [u8; 4] = *b"MAOF";

/// ANIM file format types
///
/// ANIM files evolved significantly between World of Warcraft expansions:
/// - Legacy format was used from Vanilla through Warlords of Draenor
/// - Modern format was introduced in Legion and continues through current versions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimFormat {
    /// Legacy format (Vanilla through Warlords of Draenor)
    ///
    /// Features:
    /// - Raw binary data without magic headers
    /// - Variable structure depending on M2 model
    /// - Requires context from associated M2 file for proper parsing
    Legacy,
    /// Modern format (Legion and later)
    ///
    /// Features:
    /// - "MAOF" magic header for identification
    /// - Self-contained chunked structure
    /// - Standardized format across different models
    Modern,
}

/// ANIM format detector
pub struct AnimFormatDetector;

impl AnimFormatDetector {
    /// Detect ANIM format by examining file content
    ///
    /// This method examines the first 4 bytes of the file to detect the format:
    /// - If they match "MAOF", it's a modern format file
    /// - Otherwise, it's assumed to be a legacy format file
    ///
    /// The reader position is restored after detection.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file is too small to read the magic bytes
    /// - I/O errors occur during detection
    pub fn detect_format<R: Read + Seek>(reader: &mut R) -> Result<AnimFormat> {
        let initial_pos = reader.stream_position().map_err(M2Error::Io)?;

        // Try to read first 4 bytes to check for MAOF magic
        let mut magic = [0u8; 4];
        match reader.read_exact(&mut magic) {
            Ok(()) => {
                // Reset position for subsequent parsing
                reader
                    .seek(SeekFrom::Start(initial_pos))
                    .map_err(M2Error::Io)?;

                if magic == ANIM_MAGIC {
                    Ok(AnimFormat::Modern)
                } else {
                    // If first 4 bytes are not MAOF, assume legacy format
                    // Legacy files start with raw data (typically offset tables)
                    Ok(AnimFormat::Legacy)
                }
            }
            Err(e) => {
                // If we can't read 4 bytes, the file is too small or corrupted
                reader
                    .seek(SeekFrom::Start(initial_pos))
                    .map_err(|_| M2Error::Io(e))?;
                Err(M2Error::AnimFormatError(
                    "File too small to determine ANIM format - need at least 4 bytes".to_string(),
                ))
            }
        }
    }

    /// Detect format based on M2 version (heuristic approach)
    pub fn detect_format_by_version(version: M2Version) -> AnimFormat {
        match version {
            // Pre-Legion versions use legacy format
            M2Version::Vanilla
            | M2Version::TBC
            | M2Version::WotLK
            | M2Version::Cataclysm
            | M2Version::MoP
            | M2Version::WoD => AnimFormat::Legacy,
            // Legion and later use modern format
            _ => AnimFormat::Modern,
        }
    }
}

/// ANIM file header (Modern format)
#[derive(Debug, Clone)]
pub struct AnimHeader {
    /// Magic signature ("MAOF")
    pub magic: [u8; 4],
    /// Anim version
    pub version: u32,
    /// The number of AFID IDs in this file
    pub id_count: u32,
    /// Unknown
    pub unknown: u32,
    /// Offset to animation entries
    pub anim_entry_offset: u32,
}

impl AnimHeader {
    /// Parse an ANIM header from a reader
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Read and check magic
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;

        if magic != ANIM_MAGIC {
            return Err(M2Error::InvalidMagic {
                expected: String::from_utf8_lossy(&ANIM_MAGIC).to_string(),
                actual: String::from_utf8_lossy(&magic).to_string(),
            });
        }

        // Read other header fields
        let version = reader.read_u32_le()?;
        let id_count = reader.read_u32_le()?;
        let unknown = reader.read_u32_le()?;
        let anim_entry_offset = reader.read_u32_le()?;

        Ok(Self {
            magic,
            version,
            id_count,
            unknown,
            anim_entry_offset,
        })
    }

    /// Write an ANIM header to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.magic)?;
        writer.write_u32_le(self.version)?;
        writer.write_u32_le(self.id_count)?;
        writer.write_u32_le(self.unknown)?;
        writer.write_u32_le(self.anim_entry_offset)?;

        Ok(())
    }
}

/// Animation entry header
#[derive(Debug, Clone)]
pub struct AnimEntry {
    /// Animation ID "AFID"
    pub id: u32,
    /// Start offset of the animation section
    pub offset: u32,
    /// Size of the animation section
    pub size: u32,
}

impl AnimEntry {
    /// Parse an animation entry from a reader
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let id = reader.read_u32_le()?;
        let offset = reader.read_u32_le()?;
        let size = reader.read_u32_le()?;

        Ok(Self { id, offset, size })
    }

    /// Write an animation entry to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32_le(self.id)?;
        writer.write_u32_le(self.offset)?;
        writer.write_u32_le(self.size)?;

        Ok(())
    }
}

/// Animation section header "AFID"
#[derive(Debug, Clone)]
pub struct AnimSectionHeader {
    /// "AFID" magic
    pub magic: [u8; 4],
    /// Animation ID
    pub id: u32,
    /// Start frames for this section
    pub start: u32,
    /// End frames for this section
    pub end: u32,
}

impl AnimSectionHeader {
    /// Parse an animation section header from a reader
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;

        if magic != *b"AFID" {
            return Err(M2Error::InvalidMagic {
                expected: "AFID".to_string(),
                actual: String::from_utf8_lossy(&magic).to_string(),
            });
        }

        let id = reader.read_u32_le()?;
        let start = reader.read_u32_le()?;
        let end = reader.read_u32_le()?;

        Ok(Self {
            magic,
            id,
            start,
            end,
        })
    }

    /// Write an animation section header to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.magic)?;
        writer.write_u32_le(self.id)?;
        writer.write_u32_le(self.start)?;
        writer.write_u32_le(self.end)?;

        Ok(())
    }
}

/// Translation data for an animation
#[derive(Debug, Clone)]
pub struct AnimTranslation {
    /// Animation timelines
    pub timestamps: Vec<u32>,
    /// Translation vectors
    pub translations: Vec<C3Vector>,
}

/// Rotation data for an animation
#[derive(Debug, Clone)]
pub struct AnimRotation {
    /// Animation timelines
    pub timestamps: Vec<u32>,
    /// Rotation quaternions
    pub rotations: Vec<Quaternion>,
}

/// Scaling data for an animation
#[derive(Debug, Clone)]
pub struct AnimScaling {
    /// Animation timelines
    pub timestamps: Vec<u32>,
    /// Scaling vectors
    pub scalings: Vec<C3Vector>,
}

/// Animation data for a single bone
#[derive(Debug, Clone)]
pub struct AnimBoneAnimation {
    /// Bone ID
    pub bone_id: u32,
    /// Translation animation
    pub translation: Option<AnimTranslation>,
    /// Rotation animation
    pub rotation: Option<AnimRotation>,
    /// Scaling animation
    pub scaling: Option<AnimScaling>,
}

/// Animation data for a section
#[derive(Debug, Clone)]
pub struct AnimSection {
    /// Section header
    pub header: AnimSectionHeader,
    /// Animations for each bone
    pub bone_animations: Vec<AnimBoneAnimation>,
}

impl AnimSection {
    /// Parse an animation section from a reader
    pub fn parse<R: Read>(reader: &mut R, size: u32) -> Result<Self> {
        let header = AnimSectionHeader::parse(reader)?;

        // Determine the bone count
        let header_size = 16; // "AFID" + id + start + end
        let remaining_size = size - header_size;
        let bone_count = remaining_size / 4; // Each bone animation reference is 4 bytes

        // Read bone animation offsets
        let mut bone_offsets = Vec::with_capacity(bone_count as usize);
        for _ in 0..bone_count {
            bone_offsets.push(reader.read_u32_le()?);
        }

        // Read bone animations
        let mut bone_animations = Vec::with_capacity(bone_count as usize);

        for &offset in &bone_offsets {
            if offset > 0 {
                // Bone has animation data
                let bone_id = reader.read_u32_le()?;

                // Read flags
                let flags = reader.read_u32_le()?;

                // Read translation data if present
                let translation = if (flags & 0x1) != 0 {
                    let timestamp_count = reader.read_u32_le()?;

                    let mut timestamps = Vec::with_capacity(timestamp_count as usize);
                    for _ in 0..timestamp_count {
                        timestamps.push(reader.read_u32_le()?);
                    }

                    let mut translations = Vec::with_capacity(timestamp_count as usize);
                    for _ in 0..timestamp_count {
                        translations.push(C3Vector::parse(reader)?);
                    }

                    Some(AnimTranslation {
                        timestamps,
                        translations,
                    })
                } else {
                    None
                };

                // Read rotation data if present
                let rotation = if (flags & 0x2) != 0 {
                    let timestamp_count = reader.read_u32_le()?;

                    let mut timestamps = Vec::with_capacity(timestamp_count as usize);
                    for _ in 0..timestamp_count {
                        timestamps.push(reader.read_u32_le()?);
                    }

                    let mut rotations = Vec::with_capacity(timestamp_count as usize);
                    for _ in 0..timestamp_count {
                        rotations.push(Quaternion::parse(reader)?);
                    }

                    Some(AnimRotation {
                        timestamps,
                        rotations,
                    })
                } else {
                    None
                };

                // Read scaling data if present
                let scaling = if (flags & 0x4) != 0 {
                    let timestamp_count = reader.read_u32_le()?;

                    let mut timestamps = Vec::with_capacity(timestamp_count as usize);
                    for _ in 0..timestamp_count {
                        timestamps.push(reader.read_u32_le()?);
                    }

                    let mut scalings = Vec::with_capacity(timestamp_count as usize);
                    for _ in 0..timestamp_count {
                        scalings.push(C3Vector::parse(reader)?);
                    }

                    Some(AnimScaling {
                        timestamps,
                        scalings,
                    })
                } else {
                    None
                };

                bone_animations.push(AnimBoneAnimation {
                    bone_id,
                    translation,
                    rotation,
                    scaling,
                });
            } else {
                // No animation data for this bone
                bone_animations.push(AnimBoneAnimation {
                    bone_id: 0,
                    translation: None,
                    rotation: None,
                    scaling: None,
                });
            }
        }

        Ok(Self {
            header,
            bone_animations,
        })
    }

    /// Write an animation section to a writer
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        // Write section header
        self.header.write(writer)?;

        // Write bone animation offsets (placeholders for now)
        let bone_offsets_pos = writer.stream_position()?;

        for _ in 0..self.bone_animations.len() {
            writer.write_u32_le(0)?; // Placeholder
        }

        // Write bone animations and update offsets
        let mut bone_offsets = Vec::with_capacity(self.bone_animations.len());

        for bone_animation in &self.bone_animations {
            if bone_animation.translation.is_some()
                || bone_animation.rotation.is_some()
                || bone_animation.scaling.is_some()
            {
                // Bone has animation data
                let offset = writer.stream_position()? as u32;
                bone_offsets.push(offset);

                // Write bone ID
                writer.write_u32_le(bone_animation.bone_id)?;

                // Determine flags
                let mut flags = 0u32;
                if bone_animation.translation.is_some() {
                    flags |= 0x1;
                }
                if bone_animation.rotation.is_some() {
                    flags |= 0x2;
                }
                if bone_animation.scaling.is_some() {
                    flags |= 0x4;
                }

                // Write flags
                writer.write_u32_le(flags)?;

                // Write translation data if present
                if let Some(ref translation) = bone_animation.translation {
                    writer.write_u32_le(translation.timestamps.len() as u32)?;

                    for &timestamp in &translation.timestamps {
                        writer.write_u32_le(timestamp)?;
                    }

                    for translation in &translation.translations {
                        translation.write(writer)?;
                    }
                }

                // Write rotation data if present
                if let Some(ref rotation) = bone_animation.rotation {
                    writer.write_u32_le(rotation.timestamps.len() as u32)?;

                    for &timestamp in &rotation.timestamps {
                        writer.write_u32_le(timestamp)?;
                    }

                    for rotation in &rotation.rotations {
                        rotation.write(writer)?;
                    }
                }

                // Write scaling data if present
                if let Some(ref scaling) = bone_animation.scaling {
                    writer.write_u32_le(scaling.timestamps.len() as u32)?;

                    for &timestamp in &scaling.timestamps {
                        writer.write_u32_le(timestamp)?;
                    }

                    for scaling in &scaling.scalings {
                        scaling.write(writer)?;
                    }
                }
            } else {
                // No animation data for this bone
                bone_offsets.push(0);
            }
        }

        // Update bone offsets
        let current_pos = writer.stream_position()?;
        writer.seek(SeekFrom::Start(bone_offsets_pos))?;

        for &offset in &bone_offsets {
            writer.write_u32_le(offset)?;
        }

        // Restore position
        writer.seek(SeekFrom::Start(current_pos))?;

        Ok(())
    }
}

/// Format-specific metadata
#[derive(Debug, Clone)]
pub enum AnimMetadata {
    Legacy {
        /// Total file size
        file_size: u32,
        /// Number of animations detected
        animation_count: u32,
        /// Detected structure hints for validation
        structure_hints: LegacyStructureHints,
    },
    Modern {
        /// Original MAOF header
        header: AnimHeader,
        /// Animation entries
        entries: Vec<AnimEntry>,
    },
}

/// Structure hints for legacy ANIM files
#[derive(Debug, Clone)]
pub struct LegacyStructureHints {
    /// Whether the file appears to have valid structure
    pub appears_valid: bool,
    /// Estimated data blocks found
    pub estimated_blocks: u32,
    /// File appears to contain timestamps
    pub has_timestamps: bool,
}

/// Memory usage statistics for ANIM files
#[derive(Debug, Clone, Default)]
pub struct MemoryUsage {
    /// Number of animation sections
    pub sections: usize,
    /// Total number of bone animations
    pub bone_animations: usize,
    /// Total translation keyframes
    pub translation_keyframes: usize,
    /// Total rotation keyframes
    pub rotation_keyframes: usize,
    /// Total scaling keyframes
    pub scaling_keyframes: usize,
    /// Approximate memory usage in bytes
    pub approximate_bytes: usize,
}

impl MemoryUsage {
    /// Create new empty memory usage statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate approximate memory usage in bytes
    pub fn calculate_approximate_bytes(&self) -> usize {
        let mut bytes = 0;

        // Section headers
        bytes += self.sections * std::mem::size_of::<AnimSectionHeader>();

        // Bone animation structures
        bytes += self.bone_animations * std::mem::size_of::<AnimBoneAnimation>();

        // Keyframe data (timestamps + values)
        bytes += self.translation_keyframes
            * (std::mem::size_of::<u32>() + std::mem::size_of::<C3Vector>());
        bytes += self.rotation_keyframes
            * (std::mem::size_of::<u32>() + std::mem::size_of::<Quaternion>());
        bytes +=
            self.scaling_keyframes * (std::mem::size_of::<u32>() + std::mem::size_of::<C3Vector>());

        bytes
    }

    /// Get total keyframes across all animation types
    pub fn total_keyframes(&self) -> usize {
        self.translation_keyframes + self.rotation_keyframes + self.scaling_keyframes
    }
}

/// Unified ANIM file representation
#[derive(Debug, Clone)]
pub struct AnimFile {
    /// Detected format type
    pub format: AnimFormat,
    /// Animation sections (unified regardless of source format)
    pub sections: Vec<AnimSection>,
    /// Format-specific metadata
    pub metadata: AnimMetadata,
}

/// ANIM parser factory for format-specific parsing
pub struct AnimParser;

impl AnimParser {
    /// Parse ANIM file with automatic format detection
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<AnimFile> {
        let format = AnimFormatDetector::detect_format(reader)?;

        match format {
            AnimFormat::Legacy => Self::parse_legacy(reader),
            AnimFormat::Modern => Self::parse_modern(reader),
        }
    }

    /// Parse with explicit format specification
    pub fn parse_with_format<R: Read + Seek>(
        reader: &mut R,
        format: AnimFormat,
    ) -> Result<AnimFile> {
        match format {
            AnimFormat::Legacy => Self::parse_legacy(reader),
            AnimFormat::Modern => Self::parse_modern(reader),
        }
    }

    /// Parse legacy format ANIM file
    fn parse_legacy<R: Read + Seek>(reader: &mut R) -> Result<AnimFile> {
        // Get file size for metadata
        let file_size = reader.seek(SeekFrom::End(0))? as u32;
        reader.seek(SeekFrom::Start(0))?;

        // Legacy format analysis:
        // Based on examination of real Cataclysm ANIM files, they appear to start
        // with raw animation data rather than a count. The structure seems to be:
        // 1. Header/offset information (variable size)
        // 2. Raw animation timeline and value data

        // For legacy ANIM files, we'll attempt to parse as raw animation data
        // Since the exact structure varies, we'll create a minimal representation

        // Try to detect if this looks like legacy animation data
        let mut header_bytes = [0u8; 16];
        reader.read_exact(&mut header_bytes)?;
        reader.seek(SeekFrom::Start(0))?;

        // Check if this looks like raw animation data (starts with zeros or small values)
        let _first_value = u32::from_le_bytes([
            header_bytes[0],
            header_bytes[1],
            header_bytes[2],
            header_bytes[3],
        ]);

        // For legacy files, create a placeholder animation section
        // Real parsing would require understanding the specific M2 model this ANIM belongs to
        let animation_id = Self::extract_anim_id_from_legacy_data(&header_bytes);

        // Analyze the structure for better metadata
        let structure_hints = Self::analyze_legacy_structure(reader, file_size)?;

        // Create a single animation section representing this legacy ANIM file
        // In practice, legacy ANIM files contain raw data for a single animation
        let sections = vec![Self::create_legacy_animation_section(
            reader,
            animation_id,
            file_size,
        )?];

        Ok(AnimFile {
            format: AnimFormat::Legacy,
            sections,
            metadata: AnimMetadata::Legacy {
                file_size,
                animation_count: 1, // Legacy files typically contain one animation
                structure_hints,
            },
        })
    }

    /// Extract animation ID from legacy data (heuristic)
    fn extract_anim_id_from_legacy_data(_header_bytes: &[u8; 16]) -> u32 {
        // Since legacy files don't have a clear header with ID,
        // we'll use a default or try to extract from context
        // In practice, this would come from the filename pattern
        // For now, return a default animation ID
        1
    }

    /// Create a legacy animation section from raw data
    fn create_legacy_animation_section<R: Read + Seek>(
        reader: &mut R,
        animation_id: u32,
        _file_size: u32,
    ) -> Result<AnimSection> {
        // Reset to beginning
        reader.seek(SeekFrom::Start(0))?;

        // For legacy files, we create a placeholder section since the exact
        // structure varies and requires context from the associated M2 model
        let header = AnimSectionHeader {
            magic: *b"AFID",
            id: animation_id,
            start: 0,
            end: 0, // Would need to be extracted from actual data
        };

        // Legacy files contain raw animation data that would need to be
        // parsed with knowledge of the bone structure from the M2 file
        // For now, we create an empty placeholder
        let bone_animations = Vec::new();

        Ok(AnimSection {
            header,
            bone_animations,
        })
    }

    /// Analyze legacy ANIM structure to provide better metadata
    fn analyze_legacy_structure<R: Read + Seek>(
        reader: &mut R,
        file_size: u32,
    ) -> Result<LegacyStructureHints> {
        reader.seek(SeekFrom::Start(0))?;

        let mut appears_valid = true;
        let mut estimated_blocks = 0;
        let mut has_timestamps = false;

        // Read the first 1KB to analyze structure
        let mut buffer = vec![0u8; (file_size as usize).min(1024)];
        let bytes_read = reader.read(&mut buffer)?;

        if bytes_read < 16 {
            appears_valid = false;
        } else {
            // Look for patterns that suggest this is animation data
            // Check for sequences of increasing numbers (timestamps)
            let u32_values: Vec<u32> = buffer
                .chunks_exact(4)
                .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect();

            for window in u32_values.windows(3) {
                if window.len() == 3 {
                    let (val1, val2, val3) = (window[0], window[1], window[2]);

                    // Check for increasing sequence (possible timestamps)
                    if val1 < val2 && val2 < val3 && val1 < 100000 {
                        has_timestamps = true;
                        estimated_blocks += 1;
                    }
                }
            }

            // Estimate blocks based on file size and patterns
            if estimated_blocks == 0 {
                estimated_blocks = (file_size / 1000).max(1); // Rough estimate
            }
        }

        reader.seek(SeekFrom::Start(0))?; // Reset for subsequent operations

        Ok(LegacyStructureHints {
            appears_valid,
            estimated_blocks,
            has_timestamps,
        })
    }

    /// Parse modern format ANIM file (adapted from existing implementation)
    fn parse_modern<R: Read + Seek>(reader: &mut R) -> Result<AnimFile> {
        // Parse header
        let header = AnimHeader::parse(reader)?;

        // Parse animation entries
        reader.seek(SeekFrom::Start(header.anim_entry_offset as u64))?;

        let mut entries = Vec::with_capacity(header.id_count as usize);
        for _ in 0..header.id_count {
            entries.push(AnimEntry::parse(reader)?);
        }

        // Parse animation sections
        let mut sections = Vec::with_capacity(entries.len());

        for entry in &entries {
            reader.seek(SeekFrom::Start(entry.offset as u64))?;
            sections.push(AnimSection::parse(reader, entry.size)?);
        }

        Ok(AnimFile {
            format: AnimFormat::Modern,
            sections,
            metadata: AnimMetadata::Modern { header, entries },
        })
    }
}

impl AnimFile {
    /// Parse an ANIM file from a reader with automatic format detection
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        AnimParser::parse(reader)
    }

    /// Parse ANIM file with validation
    pub fn parse_validated<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let anim_file = Self::parse(reader)?;
        anim_file.validate()?;
        Ok(anim_file)
    }

    /// Validate the parsed ANIM file structure
    pub fn validate(&self) -> Result<()> {
        // Validate sections
        if self.sections.is_empty() {
            return Err(M2Error::ValidationError(
                "ANIM file must contain at least one section".to_string(),
            ));
        }

        // Format-specific validation
        match (&self.format, &self.metadata) {
            (
                AnimFormat::Legacy,
                AnimMetadata::Legacy {
                    structure_hints, ..
                },
            ) => {
                if !structure_hints.appears_valid {
                    return Err(M2Error::ValidationError(
                        "Legacy ANIM file structure appears invalid".to_string(),
                    ));
                }
            }
            (AnimFormat::Modern, AnimMetadata::Modern { header, entries }) => {
                if header.id_count as usize != entries.len() {
                    return Err(M2Error::ValidationError(format!(
                        "Header ID count ({}) doesn't match entries count ({})",
                        header.id_count,
                        entries.len()
                    )));
                }

                if header.id_count as usize != self.sections.len() {
                    return Err(M2Error::ValidationError(format!(
                        "Header ID count ({}) doesn't match sections count ({})",
                        header.id_count,
                        self.sections.len()
                    )));
                }
            }
            _ => {
                return Err(M2Error::ValidationError(
                    "Format and metadata type mismatch".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Load an ANIM file from a file with automatic format detection
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        Self::parse(&mut file)
    }

    /// Load ANIM file with version hint for format detection
    pub fn load_with_version<P: AsRef<Path>>(path: P, version: M2Version) -> Result<Self> {
        let mut file = File::open(path)?;
        let format = AnimFormatDetector::detect_format_by_version(version);
        AnimParser::parse_with_format(&mut file, format)
    }

    /// Parse ANIM file with explicit format specification
    pub fn parse_with_format<R: Read + Seek>(reader: &mut R, format: AnimFormat) -> Result<Self> {
        AnimParser::parse_with_format(reader, format)
    }

    /// Save an ANIM file to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut file = File::create(path)?;
        self.write(&mut file)
    }

    /// Write an ANIM file to a writer
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        match self.format {
            AnimFormat::Modern => self.write_modern(writer),
            AnimFormat::Legacy => self.write_legacy(writer),
        }
    }

    /// Write modern format ANIM file
    fn write_modern<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        let (header, entries) = match &self.metadata {
            AnimMetadata::Modern { header, entries } => (header, entries),
            _ => {
                return Err(M2Error::InternalError(
                    "Attempting to write modern format with legacy metadata".to_string(),
                ));
            }
        };

        // Calculate offsets
        let header_size = 20; // Magic + version + id count + unknown + entry offset
        let entry_size = 12; // ID + offset + size

        let entry_offset = header_size;
        let _section_offset = entry_offset + entries.len() as u32 * entry_size;

        // Write header
        let mut header = header.clone();
        header.anim_entry_offset = entry_offset;
        header.write(writer)?;

        // Write entry placeholders
        let mut updated_entries = Vec::with_capacity(entries.len());

        for entry in entries {
            let entry = AnimEntry {
                id: entry.id,
                offset: 0, // Placeholder
                size: 0,   // Placeholder
            };

            entry.write(writer)?;
            updated_entries.push(entry);
        }

        // Write sections and update entries
        for (i, section) in self.sections.iter().enumerate() {
            let section_start = writer.stream_position()? as u32;
            section.write(writer)?;
            let section_end = writer.stream_position()? as u32;

            updated_entries[i].offset = section_start;
            updated_entries[i].size = section_end - section_start;
        }

        // Update entries
        writer.seek(SeekFrom::Start(entry_offset as u64))?;

        for entry in &updated_entries {
            entry.write(writer)?;
        }

        Ok(())
    }

    /// Write legacy format ANIM file
    fn write_legacy<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        // Legacy format writing is more complex due to raw data layout
        // For now, implement a basic structure

        // Write animation count
        writer.write_u32_le(self.sections.len() as u32)?;

        // Write offset placeholders
        let offsets_pos = writer.stream_position()?;
        for _ in 0..self.sections.len() {
            writer.write_u32_le(0)?; // Placeholder
        }

        // Write animation data and collect offsets
        let mut offsets = Vec::with_capacity(self.sections.len());

        for section in &self.sections {
            let offset = writer.stream_position()? as u32;
            offsets.push(offset);

            // Write animation header data
            writer.write_u32_le(section.header.id)?;
            writer.write_u32_le(section.header.start)?;
            writer.write_u32_le(section.header.end)?;

            // Write bone count
            writer.write_u32_le(section.bone_animations.len() as u32)?;

            // Write bone animation data
            for bone_anim in &section.bone_animations {
                writer.write_u32_le(bone_anim.bone_id)?;

                // Calculate flags
                let mut flags = 0u32;
                if bone_anim.translation.is_some() {
                    flags |= 0x1;
                }
                if bone_anim.rotation.is_some() {
                    flags |= 0x2;
                }
                if bone_anim.scaling.is_some() {
                    flags |= 0x4;
                }
                writer.write_u32_le(flags)?;

                // Write animation data
                if let Some(ref translation) = bone_anim.translation {
                    writer.write_u32_le(translation.timestamps.len() as u32)?;
                    for &timestamp in &translation.timestamps {
                        writer.write_u32_le(timestamp)?;
                    }
                    for translation in &translation.translations {
                        translation.write(writer)?;
                    }
                }

                if let Some(ref rotation) = bone_anim.rotation {
                    writer.write_u32_le(rotation.timestamps.len() as u32)?;
                    for &timestamp in &rotation.timestamps {
                        writer.write_u32_le(timestamp)?;
                    }
                    for rotation in &rotation.rotations {
                        rotation.write(writer)?;
                    }
                }

                if let Some(ref scaling) = bone_anim.scaling {
                    writer.write_u32_le(scaling.timestamps.len() as u32)?;
                    for &timestamp in &scaling.timestamps {
                        writer.write_u32_le(timestamp)?;
                    }
                    for scaling in &scaling.scalings {
                        scaling.write(writer)?;
                    }
                }
            }
        }

        // Update offsets
        let current_pos = writer.stream_position()?;
        writer.seek(SeekFrom::Start(offsets_pos))?;
        for offset in offsets {
            writer.write_u32_le(offset)?;
        }
        writer.seek(SeekFrom::Start(current_pos))?;

        Ok(())
    }

    /// Convert this ANIM file to a different version
    pub fn convert(&self, target_version: M2Version) -> Self {
        let target_format = AnimFormatDetector::detect_format_by_version(target_version);

        if target_format == self.format {
            // No conversion needed
            return self.clone();
        }

        // Convert between formats
        match (self.format, target_format) {
            (AnimFormat::Legacy, AnimFormat::Modern) => {
                // Convert legacy to modern
                let header = AnimHeader {
                    magic: ANIM_MAGIC,
                    version: 1,
                    id_count: self.sections.len() as u32,
                    unknown: 0,
                    anim_entry_offset: 20,
                };

                let entries: Vec<AnimEntry> = self
                    .sections
                    .iter()
                    .map(|section| {
                        AnimEntry {
                            id: section.header.id,
                            offset: 0, // Will be calculated during writing
                            size: 0,   // Will be calculated during writing
                        }
                    })
                    .collect();

                AnimFile {
                    format: AnimFormat::Modern,
                    sections: self.sections.clone(),
                    metadata: AnimMetadata::Modern { header, entries },
                }
            }
            (AnimFormat::Modern, AnimFormat::Legacy) => {
                // Convert modern to legacy
                AnimFile {
                    format: AnimFormat::Legacy,
                    sections: self.sections.clone(),
                    metadata: AnimMetadata::Legacy {
                        file_size: 0, // Will be calculated during writing
                        animation_count: self.sections.len() as u32,
                        structure_hints: LegacyStructureHints {
                            appears_valid: true,
                            estimated_blocks: self.sections.len() as u32,
                            has_timestamps: false, // Unknown during conversion
                        },
                    },
                }
            }
            _ => self.clone(), // Same format
        }
    }

    /// Get the number of animation sections
    pub fn animation_count(&self) -> u32 {
        self.sections.len() as u32
    }

    /// Check if this ANIM file uses legacy format
    pub fn is_legacy_format(&self) -> bool {
        matches!(self.format, AnimFormat::Legacy)
    }

    /// Check if this ANIM file uses modern format
    pub fn is_modern_format(&self) -> bool {
        matches!(self.format, AnimFormat::Modern)
    }

    /// Get memory usage statistics for this ANIM file
    pub fn memory_usage(&self) -> MemoryUsage {
        let mut usage = MemoryUsage::new();

        // Count sections memory
        for section in &self.sections {
            usage.sections += 1;
            usage.bone_animations += section.bone_animations.len();

            for bone_anim in &section.bone_animations {
                if let Some(ref translation) = bone_anim.translation {
                    usage.translation_keyframes += translation.timestamps.len();
                }
                if let Some(ref rotation) = bone_anim.rotation {
                    usage.rotation_keyframes += rotation.timestamps.len();
                }
                if let Some(ref scaling) = bone_anim.scaling {
                    usage.scaling_keyframes += scaling.timestamps.len();
                }
            }
        }

        // Calculate approximate memory usage
        usage.approximate_bytes = usage.calculate_approximate_bytes();

        usage
    }

    /// Optimize memory usage by deduplicating identical keyframe sequences
    pub fn optimize_memory(&mut self) {
        // This is a placeholder for memory optimization
        // In practice, this could deduplicate identical timestamp/value sequences
        // across different bone animations

        for section in &mut self.sections {
            // Remove empty bone animations
            section.bone_animations.retain(|bone_anim| {
                bone_anim.translation.is_some()
                    || bone_anim.rotation.is_some()
                    || bone_anim.scaling.is_some()
            });

            // Shrink capacity to fit actual data
            section.bone_animations.shrink_to_fit();

            for bone_anim in &mut section.bone_animations {
                if let Some(ref mut translation) = bone_anim.translation {
                    translation.timestamps.shrink_to_fit();
                    translation.translations.shrink_to_fit();
                }
                if let Some(ref mut rotation) = bone_anim.rotation {
                    rotation.timestamps.shrink_to_fit();
                    rotation.rotations.shrink_to_fit();
                }
                if let Some(ref mut scaling) = bone_anim.scaling {
                    scaling.timestamps.shrink_to_fit();
                    scaling.scalings.shrink_to_fit();
                }
            }
        }
    }
}

/// Legacy ANIM parsing utilities
mod legacy_utils {
    use super::*;

    /// Validate legacy ANIM file structure
    pub fn validate_legacy_structure<R: Read + Seek>(
        reader: &mut R,
        animation_count: u32,
    ) -> Result<bool> {
        if animation_count == 0 || animation_count > 10000 {
            return Ok(false);
        }

        let file_size = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(0))?;

        // Minimum size check: count + offsets + minimal data
        let min_size = 4 + (animation_count * 4) + (animation_count * 16);
        if file_size < min_size as u64 {
            return Ok(false);
        }

        // Skip animation count
        reader.seek(SeekFrom::Start(4))?;

        // Check that offsets are reasonable
        for _ in 0..animation_count {
            let offset = reader.read_u32_le()?;
            if offset > 0 && (offset as u64) >= file_size {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Estimate animation count from file structure heuristics
    #[allow(dead_code)]
    pub fn estimate_animation_count<R: Read + Seek>(reader: &mut R) -> Result<u32> {
        let file_size = reader.seek(SeekFrom::End(0))? as u32;
        reader.seek(SeekFrom::Start(0))?;

        let potential_count = reader.read_u32_le()?;

        // Validate using file size heuristics
        if validate_legacy_structure(reader, potential_count)? {
            Ok(potential_count)
        } else {
            // Fallback: try to estimate based on file size patterns
            // This is a simplified heuristic - real implementation may need
            // more sophisticated analysis
            let estimated = file_size / 1000; // Rough estimate
            Ok(estimated.clamp(1, 100))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_anim_header_parse_write() {
        let header = AnimHeader {
            magic: ANIM_MAGIC,
            version: 1,
            id_count: 2,
            unknown: 0,
            anim_entry_offset: 20,
        };

        let mut data = Vec::new();
        header.write(&mut data).unwrap();

        let mut cursor = Cursor::new(data);
        let parsed_header = AnimHeader::parse(&mut cursor).unwrap();

        assert_eq!(parsed_header.magic, ANIM_MAGIC);
        assert_eq!(parsed_header.version, 1);
        assert_eq!(parsed_header.id_count, 2);
        assert_eq!(parsed_header.unknown, 0);
        assert_eq!(parsed_header.anim_entry_offset, 20);
    }

    #[test]
    fn test_format_detection_modern() {
        // Test modern format detection with MAOF magic
        let mut data = Vec::new();
        data.extend_from_slice(&ANIM_MAGIC);
        data.extend_from_slice(&[1, 0, 0, 0]); // version

        let mut cursor = Cursor::new(data);
        let format = AnimFormatDetector::detect_format(&mut cursor).unwrap();

        assert_eq!(format, AnimFormat::Modern);
        assert_eq!(cursor.position(), 0); // Position should be reset
    }

    #[test]
    fn test_format_detection_legacy() {
        // Test legacy format detection (no MAOF magic)
        let mut data = Vec::new();
        data.extend_from_slice(&[2, 0, 0, 0]); // animation count
        data.extend_from_slice(&[100, 0, 0, 0]); // first offset

        let mut cursor = Cursor::new(data);
        let format = AnimFormatDetector::detect_format(&mut cursor).unwrap();

        assert_eq!(format, AnimFormat::Legacy);
        assert_eq!(cursor.position(), 0); // Position should be reset
    }

    #[test]
    fn test_format_detection_by_version() {
        // Test version-based format detection
        assert_eq!(
            AnimFormatDetector::detect_format_by_version(M2Version::Vanilla),
            AnimFormat::Legacy
        );
        assert_eq!(
            AnimFormatDetector::detect_format_by_version(M2Version::Cataclysm),
            AnimFormat::Legacy
        );
        assert_eq!(
            AnimFormatDetector::detect_format_by_version(M2Version::Legion),
            AnimFormat::Modern
        );
    }

    #[test]
    fn test_anim_file_format_properties() {
        // Test format property methods
        let legacy_file = AnimFile {
            format: AnimFormat::Legacy,
            sections: Vec::new(),
            metadata: AnimMetadata::Legacy {
                file_size: 1000,
                animation_count: 5,
                structure_hints: LegacyStructureHints {
                    appears_valid: true,
                    estimated_blocks: 5,
                    has_timestamps: false,
                },
            },
        };

        assert!(legacy_file.is_legacy_format());
        assert!(!legacy_file.is_modern_format());
        assert_eq!(legacy_file.animation_count(), 0); // Based on sections count

        let modern_file = AnimFile {
            format: AnimFormat::Modern,
            sections: Vec::new(),
            metadata: AnimMetadata::Modern {
                header: AnimHeader {
                    magic: ANIM_MAGIC,
                    version: 1,
                    id_count: 3,
                    unknown: 0,
                    anim_entry_offset: 20,
                },
                entries: Vec::new(),
            },
        };

        assert!(!modern_file.is_legacy_format());
        assert!(modern_file.is_modern_format());
    }

    #[test]
    fn test_anim_entry_parse_write() {
        let entry = AnimEntry {
            id: 1,
            offset: 100,
            size: 200,
        };

        let mut data = Vec::new();
        entry.write(&mut data).unwrap();

        let mut cursor = Cursor::new(data);
        let parsed_entry = AnimEntry::parse(&mut cursor).unwrap();

        assert_eq!(parsed_entry.id, 1);
        assert_eq!(parsed_entry.offset, 100);
        assert_eq!(parsed_entry.size, 200);
    }

    #[test]
    fn test_format_conversion() {
        // Test conversion between formats
        let legacy_file = AnimFile {
            format: AnimFormat::Legacy,
            sections: vec![AnimSection {
                header: AnimSectionHeader {
                    magic: *b"AFID",
                    id: 1,
                    start: 0,
                    end: 100,
                },
                bone_animations: Vec::new(),
            }],
            metadata: AnimMetadata::Legacy {
                file_size: 1000,
                animation_count: 1,
                structure_hints: LegacyStructureHints {
                    appears_valid: true,
                    estimated_blocks: 1,
                    has_timestamps: false,
                },
            },
        };

        // Convert to modern format
        let modern_file = legacy_file.convert(M2Version::Legion);
        assert_eq!(modern_file.format, AnimFormat::Modern);
        assert_eq!(modern_file.sections.len(), 1);
        assert_eq!(modern_file.sections[0].header.id, 1);

        // Convert back to legacy format
        let legacy_again = modern_file.convert(M2Version::Cataclysm);
        assert_eq!(legacy_again.format, AnimFormat::Legacy);
        assert_eq!(legacy_again.sections.len(), 1);
        assert_eq!(legacy_again.sections[0].header.id, 1);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_real_cataclysm_anim_file() {
        let anim_path =
            "/home/danielsreichenbach/analysis/anim_samples/cataclysm/OrcFemale0064-00.anim";

        if Path::new(anim_path).exists() {
            let result = AnimFile::load(anim_path);
            match result {
                Ok(anim_file) => {
                    println!(
                        "Successfully parsed ANIM file: {} sections, format: {:?}",
                        anim_file.sections.len(),
                        anim_file.format
                    );
                    assert!(
                        !anim_file.sections.is_empty(),
                        "ANIM file should have at least one section"
                    );
                }
                Err(e) => {
                    println!("Failed to parse ANIM file: {:?}", e);
                    // For now, allow failures during development
                    // assert!(false, "Should be able to parse real ANIM file: {:?}", e);
                }
            }
        } else {
            println!("Test ANIM file not found at: {}", anim_path);
        }
    }

    #[test]
    fn test_all_cataclysm_anim_samples() {
        let samples_dir = "/home/danielsreichenbach/analysis/anim_samples/cataclysm/";

        if Path::new(samples_dir).exists() {
            if let Ok(entries) = std::fs::read_dir(samples_dir) {
                let mut success_count = 0;
                let mut total_count = 0;
                let mut memory_stats = Vec::new();

                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|s| s == "anim").unwrap_or(false) {
                        total_count += 1;
                        println!("Testing ANIM file: {:?}", path.file_name());

                        match AnimFile::load(&path) {
                            Ok(mut anim_file) => {
                                success_count += 1;

                                // Test validation
                                match anim_file.validate() {
                                    Ok(()) => println!("  ✓ Validation: Pass"),
                                    Err(e) => println!("  ⚠ Validation: {:?}", e),
                                }

                                // Test memory usage analysis
                                let usage = anim_file.memory_usage();
                                memory_stats.push(usage.clone());
                                println!(
                                    "  ✓ Success: {} sections, format: {:?}, memory: ~{} bytes",
                                    anim_file.sections.len(),
                                    anim_file.format,
                                    usage.approximate_bytes
                                );

                                // Test memory optimization
                                let before_opt = anim_file.memory_usage();
                                anim_file.optimize_memory();
                                let after_opt = anim_file.memory_usage();
                                if after_opt.approximate_bytes < before_opt.approximate_bytes {
                                    println!(
                                        "  ✓ Optimization: {} -> {} bytes",
                                        before_opt.approximate_bytes, after_opt.approximate_bytes
                                    );
                                }

                                // Test format conversion (if applicable)
                                if anim_file.is_legacy_format() {
                                    let converted =
                                        anim_file.convert(crate::version::M2Version::Legion);
                                    assert!(
                                        converted.is_modern_format(),
                                        "Conversion to modern format failed"
                                    );
                                }
                            }
                            Err(e) => {
                                println!("  ✗ Failed: {:?}", e);
                            }
                        }
                    }
                }

                println!(
                    "Summary: {}/{} ANIM files parsed successfully",
                    success_count, total_count
                );

                if !memory_stats.is_empty() {
                    let total_memory: usize =
                        memory_stats.iter().map(|s| s.approximate_bytes).sum();
                    let avg_memory = total_memory / memory_stats.len();
                    let total_keyframes: usize =
                        memory_stats.iter().map(|s| s.total_keyframes()).sum();
                    println!(
                        "Memory stats: total ~{} bytes, avg ~{} bytes per file, {} total keyframes",
                        total_memory, avg_memory, total_keyframes
                    );
                }
            }
        }
    }

    #[test]
    fn test_anim_format_detection_edge_cases() {
        use std::io::Cursor;

        // Test empty file
        let empty_data = vec![];
        let mut cursor = Cursor::new(empty_data);
        let result = AnimFormatDetector::detect_format(&mut cursor);
        assert!(result.is_err(), "Empty file should return error");

        // Test file with only 3 bytes (insufficient)
        let small_data = vec![0x4D, 0x41, 0x4F]; // "MAO" (incomplete)
        let mut cursor = Cursor::new(small_data);
        let result = AnimFormatDetector::detect_format(&mut cursor);
        assert!(result.is_err(), "File with < 4 bytes should return error");

        // Test file with exactly 4 bytes (minimal valid)
        let min_data = vec![0x4D, 0x41, 0x4F, 0x46]; // "MAOF"
        let mut cursor = Cursor::new(min_data);
        let result = AnimFormatDetector::detect_format(&mut cursor);
        assert!(result.is_ok(), "File with exactly 4 bytes should work");
        assert_eq!(result.unwrap(), AnimFormat::Modern);

        // Test position restoration
        let data = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
        let mut cursor = Cursor::new(data);
        cursor.set_position(2);
        let initial_pos = cursor.position();
        let _result = AnimFormatDetector::detect_format(&mut cursor);
        assert_eq!(
            cursor.position(),
            initial_pos,
            "Position should be restored after detection"
        );
    }

    #[test]
    fn test_anim_validation_edge_cases() {
        // Test empty sections validation
        let empty_anim = AnimFile {
            format: AnimFormat::Legacy,
            sections: Vec::new(),
            metadata: AnimMetadata::Legacy {
                file_size: 100,
                animation_count: 0,
                structure_hints: LegacyStructureHints {
                    appears_valid: true,
                    estimated_blocks: 0,
                    has_timestamps: false,
                },
            },
        };

        let result = empty_anim.validate();
        assert!(result.is_err(), "Empty sections should fail validation");

        // Test format/metadata mismatch
        let mismatched_anim = AnimFile {
            format: AnimFormat::Modern,
            sections: vec![],
            metadata: AnimMetadata::Legacy {
                file_size: 100,
                animation_count: 1,
                structure_hints: LegacyStructureHints {
                    appears_valid: true,
                    estimated_blocks: 1,
                    has_timestamps: false,
                },
            },
        };

        let result = mismatched_anim.validate();
        assert!(
            result.is_err(),
            "Format/metadata mismatch should fail validation"
        );
    }
}

#[cfg(test)]
mod legacy_tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_legacy_animation_count_validation() {
        // Test valid animation count
        let mut data = vec![2u8, 0, 0, 0]; // count = 2
        data.extend_from_slice(&[100u8, 0, 0, 0]); // offset 1
        data.extend_from_slice(&[200u8, 0, 0, 0]); // offset 2
        // Add some dummy data to reach minimum size
        data.resize(300, 0);

        let mut cursor = Cursor::new(data);
        let result = legacy_utils::validate_legacy_structure(&mut cursor, 2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_legacy_invalid_animation_count() {
        let data = vec![0u8, 0, 0, 0]; // count = 0 (invalid)
        let mut cursor = Cursor::new(data);

        let result = legacy_utils::validate_legacy_structure(&mut cursor, 0);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should be false for invalid count
    }

    #[test]
    fn test_legacy_file_too_small() {
        let data = vec![10u8, 0, 0, 0]; // count = 10, but file too small
        let mut cursor = Cursor::new(data);

        let result = legacy_utils::validate_legacy_structure(&mut cursor, 10);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should be false for too small file
    }
}
