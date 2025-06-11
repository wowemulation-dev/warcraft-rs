use crate::io_ext::{ReadExt, WriteExt};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::common::{C3Vector, Quaternion};
use crate::error::{M2Error, Result};
use crate::version::M2Version;

/// Magic signature for Anim files ("MAOF")
pub const ANIM_MAGIC: [u8; 4] = *b"MAOF";

/// ANIM file header
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

/// Represents an ANIM file containing animation data
#[derive(Debug, Clone)]
pub struct AnimFile {
    /// ANIM file header
    pub header: AnimHeader,
    /// Animation entries
    pub entries: Vec<AnimEntry>,
    /// Animation sections
    pub sections: Vec<AnimSection>,
}

impl AnimFile {
    /// Parse an ANIM file from a reader
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
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

        Ok(Self {
            header,
            entries,
            sections,
        })
    }

    /// Load an ANIM file from a file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        Self::parse(&mut file)
    }

    /// Save an ANIM file to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut file = File::create(path)?;
        self.write(&mut file)
    }

    /// Write an ANIM file to a writer
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        // Calculate offsets
        let header_size = 20; // Magic + version + id count + unknown + entry offset
        let entry_size = 12; // ID + offset + size

        let entry_offset = header_size;
        let _section_offset = entry_offset + self.entries.len() as u32 * entry_size;

        // Write header
        let mut header = self.header.clone();
        header.anim_entry_offset = entry_offset;
        header.write(writer)?;

        // Write entry placeholders
        let mut entries = Vec::with_capacity(self.entries.len());

        for i in 0..self.entries.len() {
            let entry = AnimEntry {
                id: self.entries[i].id,
                offset: 0, // Placeholder
                size: 0,   // Placeholder
            };

            entry.write(writer)?;
            entries.push(entry);
        }

        // Write sections and update entries
        for (i, section) in self.sections.iter().enumerate() {
            let section_start = writer.stream_position()? as u32;
            section.write(writer)?;
            let section_end = writer.stream_position()? as u32;

            entries[i].offset = section_start;
            entries[i].size = section_end - section_start;
        }

        // Update entries
        writer.seek(SeekFrom::Start(entry_offset as u64))?;

        for entry in &entries {
            entry.write(writer)?;
        }

        Ok(())
    }

    /// Convert this ANIM file to a different version (no version differences for ANIM files yet)
    pub fn convert(&self, _target_version: M2Version) -> Self {
        self.clone()
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
}
