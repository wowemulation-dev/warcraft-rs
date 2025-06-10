//! WDT chunk implementations

use crate::error::{Error, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

pub mod maid;
pub mod mphd;

// Re-export chunk types
pub use maid::MaidChunk;
pub use mphd::{MphdChunk, MphdFlags};

/// WDT file version (always 18)
pub const WDT_VERSION: u32 = 18;

/// Map dimensions (64x64 grid)
pub const WDT_MAP_SIZE: usize = 64;

/// Total number of tiles
pub const WDT_TILE_COUNT: usize = WDT_MAP_SIZE * WDT_MAP_SIZE;

/// Chunk header size (magic + size)
pub const CHUNK_HEADER_SIZE: usize = 8;

/// Common trait for all chunks
pub trait Chunk: Sized {
    /// Get the chunk magic bytes
    fn magic() -> &'static [u8; 4];

    /// Get the expected chunk size (None for variable-sized chunks)
    fn expected_size() -> Option<usize> {
        None
    }

    /// Read chunk data from a reader (after magic and size have been read)
    fn read(reader: &mut impl Read, size: usize) -> Result<Self>;

    /// Write chunk data to a writer
    fn write(&self, writer: &mut impl Write) -> Result<()>;

    /// Get the size of the chunk data (excluding header)
    fn size(&self) -> usize;

    /// Write the complete chunk including header
    fn write_chunk(&self, writer: &mut impl Write) -> Result<()> {
        writer.write_all(Self::magic())?;
        writer.write_u32::<LittleEndian>(self.size() as u32)?;
        self.write(writer)?;
        Ok(())
    }
}

/// MVER chunk - Version information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MverChunk {
    pub version: u32,
}

impl MverChunk {
    pub fn new() -> Self {
        Self {
            version: WDT_VERSION,
        }
    }
}

impl Default for MverChunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chunk for MverChunk {
    fn magic() -> &'static [u8; 4] {
        b"REVM" // 'MVER' reversed
    }

    fn expected_size() -> Option<usize> {
        Some(4)
    }

    fn read(reader: &mut impl Read, size: usize) -> Result<Self> {
        if size != 4 {
            return Err(Error::InvalidChunkSize {
                chunk: "MVER".to_string(),
                expected: 4,
                found: size,
            });
        }

        let version = reader.read_u32::<LittleEndian>()?;
        if version != WDT_VERSION {
            return Err(Error::InvalidVersion(version));
        }

        Ok(Self { version })
    }

    fn write(&self, writer: &mut impl Write) -> Result<()> {
        writer.write_u32::<LittleEndian>(self.version)?;
        Ok(())
    }

    fn size(&self) -> usize {
        4
    }
}

/// MAIN chunk entry - Tile information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MainEntry {
    pub flags: u32,
    pub area_id: u32,
}

impl MainEntry {
    pub fn new() -> Self {
        Self {
            flags: 0,
            area_id: 0,
        }
    }

    /// Check if this tile has ADT data
    pub fn has_adt(&self) -> bool {
        (self.flags & 0x0001) != 0
    }

    /// Set whether this tile has ADT data
    pub fn set_has_adt(&mut self, has_adt: bool) {
        if has_adt {
            self.flags |= 0x0001;
        } else {
            self.flags &= !0x0001;
        }
    }
}

impl Default for MainEntry {
    fn default() -> Self {
        Self::new()
    }
}

/// MAIN chunk - Map tile information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MainChunk {
    pub entries: Vec<Vec<MainEntry>>,
}

impl MainChunk {
    pub fn new() -> Self {
        let mut entries = Vec::with_capacity(WDT_MAP_SIZE);
        for _ in 0..WDT_MAP_SIZE {
            let mut row = Vec::with_capacity(WDT_MAP_SIZE);
            for _ in 0..WDT_MAP_SIZE {
                row.push(MainEntry::new());
            }
            entries.push(row);
        }
        Self { entries }
    }

    /// Get entry at coordinates
    pub fn get(&self, x: usize, y: usize) -> Option<&MainEntry> {
        self.entries.get(y).and_then(|row| row.get(x))
    }

    /// Get mutable entry at coordinates
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut MainEntry> {
        self.entries.get_mut(y).and_then(|row| row.get_mut(x))
    }

    /// Count tiles with ADT data
    pub fn count_existing_tiles(&self) -> usize {
        self.entries
            .iter()
            .flat_map(|row| row.iter())
            .filter(|entry| entry.has_adt())
            .count()
    }
}

impl Default for MainChunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chunk for MainChunk {
    fn magic() -> &'static [u8; 4] {
        b"NIAM" // 'MAIN' reversed
    }

    fn expected_size() -> Option<usize> {
        Some(WDT_TILE_COUNT * 8)
    }

    fn read(reader: &mut impl Read, size: usize) -> Result<Self> {
        let expected = WDT_TILE_COUNT * 8;
        if size != expected {
            return Err(Error::InvalidChunkSize {
                chunk: "MAIN".to_string(),
                expected,
                found: size,
            });
        }

        let mut entries = Vec::with_capacity(WDT_MAP_SIZE);
        for _y in 0..WDT_MAP_SIZE {
            let mut row = Vec::with_capacity(WDT_MAP_SIZE);
            for _x in 0..WDT_MAP_SIZE {
                let flags = reader.read_u32::<LittleEndian>()?;
                let area_id = reader.read_u32::<LittleEndian>()?;
                row.push(MainEntry { flags, area_id });
            }
            entries.push(row);
        }

        Ok(Self { entries })
    }

    fn write(&self, writer: &mut impl Write) -> Result<()> {
        for row in &self.entries {
            for entry in row {
                writer.write_u32::<LittleEndian>(entry.flags)?;
                writer.write_u32::<LittleEndian>(entry.area_id)?;
            }
        }
        Ok(())
    }

    fn size(&self) -> usize {
        WDT_TILE_COUNT * 8
    }
}

/// MWMO chunk - World Map Object filenames
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MwmoChunk {
    pub filenames: Vec<String>,
}

impl MwmoChunk {
    pub fn new() -> Self {
        Self {
            filenames: Vec::new(),
        }
    }

    /// Add a WMO filename
    pub fn add_filename(&mut self, filename: String) {
        self.filenames.push(filename);
    }

    /// Check if chunk is empty
    pub fn is_empty(&self) -> bool {
        self.filenames.is_empty()
    }
}

impl Default for MwmoChunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chunk for MwmoChunk {
    fn magic() -> &'static [u8; 4] {
        b"OMWM" // 'MWMO' reversed
    }

    fn read(reader: &mut impl Read, size: usize) -> Result<Self> {
        if size == 0 {
            return Ok(Self::new());
        }

        // Read all data
        let mut data = vec![0u8; size];
        reader.read_exact(&mut data)?;

        // Split by null terminators
        let mut filenames = Vec::new();
        let mut current = Vec::new();

        for &byte in &data {
            if byte == 0 {
                if !current.is_empty() {
                    let filename =
                        String::from_utf8(current.clone()).map_err(|e| Error::StringError {
                            context: "MWMO filename".to_string(),
                            message: e.to_string(),
                        })?;
                    filenames.push(filename);
                    current.clear();
                }
            } else {
                current.push(byte);
            }
        }

        // Handle case where data doesn't end with null
        if !current.is_empty() {
            let filename = String::from_utf8(current).map_err(|e| Error::StringError {
                context: "MWMO filename".to_string(),
                message: e.to_string(),
            })?;
            filenames.push(filename);
        }

        Ok(Self { filenames })
    }

    fn write(&self, writer: &mut impl Write) -> Result<()> {
        for filename in &self.filenames {
            writer.write_all(filename.as_bytes())?;
            writer.write_u8(0)?; // Null terminator
        }
        Ok(())
    }

    fn size(&self) -> usize {
        self.filenames
            .iter()
            .map(|f| f.len() + 1) // +1 for null terminator
            .sum()
    }
}

/// MODF entry - Map Object Definition
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModfEntry {
    pub id: u32,
    pub unique_id: u32,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub lower_bounds: [f32; 3],
    pub upper_bounds: [f32; 3],
    pub flags: u16,
    pub doodad_set: u16,
    pub name_set: u16,
    pub scale: u16,
}

impl ModfEntry {
    pub fn new() -> Self {
        Self {
            id: 0,
            unique_id: 0xFFFFFFFF, // -1, common in early versions
            position: [0.0; 3],
            rotation: [0.0; 3],
            lower_bounds: [0.0; 3],
            upper_bounds: [0.0; 3],
            flags: 0,
            doodad_set: 0,
            name_set: 0,
            scale: 0, // 0 in early versions, 1024 (1.0) in later
        }
    }
}

impl Default for ModfEntry {
    fn default() -> Self {
        Self::new()
    }
}

/// MODF chunk - Map Object Definitions
#[derive(Debug, Clone, PartialEq)]
pub struct ModfChunk {
    pub entries: Vec<ModfEntry>,
}

impl ModfChunk {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add an entry
    pub fn add_entry(&mut self, entry: ModfEntry) {
        self.entries.push(entry);
    }
}

impl Default for ModfChunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chunk for ModfChunk {
    fn magic() -> &'static [u8; 4] {
        b"FDOM" // 'MODF' reversed
    }

    fn read(reader: &mut impl Read, size: usize) -> Result<Self> {
        if size % 64 != 0 {
            return Err(Error::InvalidChunkData {
                chunk: "MODF".to_string(),
                message: format!("Size {} is not a multiple of 64", size),
            });
        }

        let count = size / 64;
        let mut entries = Vec::with_capacity(count);

        for _ in 0..count {
            let id = reader.read_u32::<LittleEndian>()?;
            let unique_id = reader.read_u32::<LittleEndian>()?;

            let mut position = [0.0f32; 3];
            for item in &mut position {
                *item = reader.read_f32::<LittleEndian>()?;
            }

            let mut rotation = [0.0f32; 3];
            for item in &mut rotation {
                *item = reader.read_f32::<LittleEndian>()?;
            }

            let mut lower_bounds = [0.0f32; 3];
            for item in &mut lower_bounds {
                *item = reader.read_f32::<LittleEndian>()?;
            }

            let mut upper_bounds = [0.0f32; 3];
            for item in &mut upper_bounds {
                *item = reader.read_f32::<LittleEndian>()?;
            }

            let flags = reader.read_u16::<LittleEndian>()?;
            let doodad_set = reader.read_u16::<LittleEndian>()?;
            let name_set = reader.read_u16::<LittleEndian>()?;
            let scale = reader.read_u16::<LittleEndian>()?;

            entries.push(ModfEntry {
                id,
                unique_id,
                position,
                rotation,
                lower_bounds,
                upper_bounds,
                flags,
                doodad_set,
                name_set,
                scale,
            });
        }

        Ok(Self { entries })
    }

    fn write(&self, writer: &mut impl Write) -> Result<()> {
        for entry in &self.entries {
            writer.write_u32::<LittleEndian>(entry.id)?;
            writer.write_u32::<LittleEndian>(entry.unique_id)?;

            for &v in &entry.position {
                writer.write_f32::<LittleEndian>(v)?;
            }

            for &v in &entry.rotation {
                writer.write_f32::<LittleEndian>(v)?;
            }

            for &v in &entry.lower_bounds {
                writer.write_f32::<LittleEndian>(v)?;
            }

            for &v in &entry.upper_bounds {
                writer.write_f32::<LittleEndian>(v)?;
            }

            writer.write_u16::<LittleEndian>(entry.flags)?;
            writer.write_u16::<LittleEndian>(entry.doodad_set)?;
            writer.write_u16::<LittleEndian>(entry.name_set)?;
            writer.write_u16::<LittleEndian>(entry.scale)?;
        }
        Ok(())
    }

    fn size(&self) -> usize {
        self.entries.len() * 64
    }
}
