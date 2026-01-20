//! MAID chunk - Map Area ID (FileDataIDs)

use crate::chunks::WDT_MAP_SIZE;
use crate::error::{Error, Result};
use std::io::{Read, Write};

/// Section types in MAID chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaidSection {
    RootAdt,
    Obj0Adt,
    Obj1Adt,
    Tex0Adt,
    LodAdt,
    MapTexture,
    MapTextureN,
    MinimapTexture,
}

impl MaidSection {
    /// Get all sections in order
    pub fn all() -> &'static [MaidSection] {
        &[
            MaidSection::RootAdt,
            MaidSection::Obj0Adt,
            MaidSection::Obj1Adt,
            MaidSection::Tex0Adt,
            MaidSection::LodAdt,
            MaidSection::MapTexture,
            MaidSection::MapTextureN,
            MaidSection::MinimapTexture,
        ]
    }

    /// Get the section index
    pub fn index(&self) -> usize {
        match self {
            MaidSection::RootAdt => 0,
            MaidSection::Obj0Adt => 1,
            MaidSection::Obj1Adt => 2,
            MaidSection::Tex0Adt => 3,
            MaidSection::LodAdt => 4,
            MaidSection::MapTexture => 5,
            MaidSection::MapTextureN => 6,
            MaidSection::MinimapTexture => 7,
        }
    }

    /// Get section name for debugging
    pub fn name(&self) -> &'static str {
        match self {
            MaidSection::RootAdt => "root ADT",
            MaidSection::Obj0Adt => "_obj0.adt",
            MaidSection::Obj1Adt => "_obj1.adt",
            MaidSection::Tex0Adt => "_tex0.adt",
            MaidSection::LodAdt => "_lod.adt",
            MaidSection::MapTexture => "map texture",
            MaidSection::MapTextureN => "map normal texture",
            MaidSection::MinimapTexture => "minimap texture",
        }
    }
}

/// MAID chunk - Contains FileDataIDs for all map files
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaidChunk {
    /// FileDataIDs for each section
    /// Each section contains 64x64 entries stored in \[Y\]\[X\] order
    sections: Vec<Vec<Vec<u32>>>,
}

impl MaidChunk {
    pub fn new() -> Self {
        let mut sections = Vec::new();

        // Initialize all sections with zeros
        for _ in MaidSection::all() {
            let mut section = Vec::with_capacity(WDT_MAP_SIZE);
            for _ in 0..WDT_MAP_SIZE {
                section.push(vec![0u32; WDT_MAP_SIZE]);
            }
            sections.push(section);
        }

        Self { sections }
    }

    /// Get FileDataID for a specific section and tile
    pub fn get(&self, section: MaidSection, x: usize, y: usize) -> Option<u32> {
        self.sections
            .get(section.index())
            .and_then(|s| s.get(y))
            .and_then(|row| row.get(x))
            .copied()
    }

    /// Set FileDataID for a specific section and tile
    pub fn set(
        &mut self,
        section: MaidSection,
        x: usize,
        y: usize,
        file_data_id: u32,
    ) -> Result<()> {
        let section_data =
            self.sections
                .get_mut(section.index())
                .ok_or_else(|| Error::InvalidChunkData {
                    chunk: "MAID".to_string(),
                    message: format!("Invalid section index: {}", section.index()),
                })?;

        let row = section_data
            .get_mut(y)
            .ok_or_else(|| Error::InvalidChunkData {
                chunk: "MAID".to_string(),
                message: format!("Invalid Y coordinate: {y}"),
            })?;

        let cell = row.get_mut(x).ok_or_else(|| Error::InvalidChunkData {
            chunk: "MAID".to_string(),
            message: format!("Invalid X coordinate: {x}"),
        })?;

        *cell = file_data_id;
        Ok(())
    }

    /// Get all FileDataIDs for root ADT files
    pub fn get_root_adt_ids(&self) -> &Vec<Vec<u32>> {
        &self.sections[MaidSection::RootAdt.index()]
    }

    /// Check if a tile has any files
    pub fn has_tile(&self, x: usize, y: usize) -> bool {
        // A tile exists if it has a root ADT FileDataID
        self.get(MaidSection::RootAdt, x, y)
            .map(|id| id != 0)
            .unwrap_or(false)
    }

    /// Count tiles with ADT data
    pub fn count_existing_tiles(&self) -> usize {
        self.sections[MaidSection::RootAdt.index()]
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&id| id != 0)
            .count()
    }

    /// Get the number of sections in this MAID chunk
    pub fn section_count(&self) -> usize {
        self.sections.len()
    }

    /// Create from a specific number of sections (for version compatibility)
    pub fn with_section_count(section_count: usize) -> Self {
        let mut sections = Vec::new();

        for _ in 0..section_count {
            let mut section = Vec::with_capacity(WDT_MAP_SIZE);
            for _ in 0..WDT_MAP_SIZE {
                section.push(vec![0u32; WDT_MAP_SIZE]);
            }
            sections.push(section);
        }

        Self { sections }
    }
}

impl Default for MaidChunk {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Chunk for MaidChunk {
    fn magic() -> &'static [u8; 4] {
        b"DIAM" // 'MAID' reversed
    }

    fn read(reader: &mut impl Read, size: usize) -> Result<Self> {
        const ENTRIES_PER_SECTION: usize = WDT_MAP_SIZE * WDT_MAP_SIZE;
        const BYTES_PER_SECTION: usize = ENTRIES_PER_SECTION * 4;

        if !size.is_multiple_of(BYTES_PER_SECTION) {
            return Err(Error::InvalidChunkData {
                chunk: "MAID".to_string(),
                message: format!(
                    "Size {size} is not a multiple of section size {BYTES_PER_SECTION}"
                ),
            });
        }

        let section_count = size / BYTES_PER_SECTION;
        let mut sections = Vec::with_capacity(section_count);

        for _section_idx in 0..section_count {
            let mut section = Vec::with_capacity(WDT_MAP_SIZE);

            for _y in 0..WDT_MAP_SIZE {
                let mut row = Vec::with_capacity(WDT_MAP_SIZE);

                for _x in 0..WDT_MAP_SIZE {
                    let mut buf = [0u8; 4];
                    reader.read_exact(&mut buf)?;
                    let file_data_id = u32::from_le_bytes(buf);
                    row.push(file_data_id);
                }

                section.push(row);
            }

            sections.push(section);
        }

        Ok(Self { sections })
    }

    fn write(&self, writer: &mut impl Write) -> Result<()> {
        for section in &self.sections {
            for row in section {
                for &file_data_id in row {
                    writer.write_all(&file_data_id.to_le_bytes())?;
                }
            }
        }
        Ok(())
    }

    fn size(&self) -> usize {
        const ENTRIES_PER_SECTION: usize = WDT_MAP_SIZE * WDT_MAP_SIZE;
        const BYTES_PER_SECTION: usize = ENTRIES_PER_SECTION * 4;

        self.sections.len() * BYTES_PER_SECTION
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunks::Chunk;

    #[test]
    fn test_maid_new() {
        let maid = MaidChunk::new();
        assert_eq!(maid.section_count(), 8);
        assert_eq!(maid.count_existing_tiles(), 0);
    }

    #[test]
    fn test_maid_set_get() {
        let mut maid = MaidChunk::new();

        // Set some FileDataIDs
        maid.set(MaidSection::RootAdt, 10, 20, 12345).unwrap();
        maid.set(MaidSection::Obj0Adt, 10, 20, 23456).unwrap();

        // Verify
        assert_eq!(maid.get(MaidSection::RootAdt, 10, 20), Some(12345));
        assert_eq!(maid.get(MaidSection::Obj0Adt, 10, 20), Some(23456));
        assert_eq!(maid.get(MaidSection::RootAdt, 0, 0), Some(0));

        assert!(maid.has_tile(10, 20));
        assert!(!maid.has_tile(0, 0));
        assert_eq!(maid.count_existing_tiles(), 1);
    }

    #[test]
    fn test_maid_read_write() {
        let mut maid = MaidChunk::new();

        // Set some test data
        maid.set(MaidSection::RootAdt, 0, 0, 100).unwrap();
        maid.set(MaidSection::RootAdt, 63, 63, 200).unwrap();
        maid.set(MaidSection::Tex0Adt, 32, 32, 300).unwrap();

        // Write to buffer
        let mut buffer = Vec::new();
        maid.write(&mut buffer).unwrap();

        // Read back
        let mut reader = std::io::Cursor::new(buffer);
        let read_maid = MaidChunk::read(&mut reader, maid.size()).unwrap();

        // Verify
        assert_eq!(read_maid.get(MaidSection::RootAdt, 0, 0), Some(100));
        assert_eq!(read_maid.get(MaidSection::RootAdt, 63, 63), Some(200));
        assert_eq!(read_maid.get(MaidSection::Tex0Adt, 32, 32), Some(300));
        assert_eq!(read_maid.section_count(), maid.section_count());
    }

    #[test]
    fn test_maid_variable_sections() {
        // Test with fewer sections (e.g., early BfA versions)
        let maid = MaidChunk::with_section_count(5);
        assert_eq!(maid.section_count(), 5);

        // Test with more sections (e.g., later versions)
        let maid = MaidChunk::with_section_count(10);
        assert_eq!(maid.section_count(), 10);
    }
}
