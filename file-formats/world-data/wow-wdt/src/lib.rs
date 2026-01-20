//! World of Warcraft WDT (World Data Table) parser library
//!
//! This library provides functionality to parse, validate, create, and convert
//! WDT files used in World of Warcraft for map data organization.
//!
//! # Features
//!
//! - Parse WDT files from any WoW version (Classic through modern)
//! - Validate WDT structure with version-aware rules
//! - Create new WDT files programmatically
//! - Convert WDT files between different WoW versions
//! - Support for all chunk types (MVER, MPHD, MAIN, MAID, MWMO, MODF)
//! - Coordinate system conversion utilities
//!
//! # Version Support
//!
//! Tested against 100+ real WDT files from:
//! - 1.12.1 (Classic)
//! - 2.4.3 (The Burning Crusade)
//! - 3.3.5a (Wrath of the Lich King)
//! - 4.3.4 (Cataclysm) - Breaking change: terrain maps lose MWMO chunks
//! - 5.4.8 (Mists of Pandaria)
//! - 8.x+ (Battle for Azeroth) - FileDataID support
//!
//! # Example
//!
//! ```no_run
//! use std::fs::File;
//! use std::io::BufReader;
//! use wow_wdt::{WdtReader, version::WowVersion};
//!
//! let file = File::open("path/to/map.wdt").unwrap();
//! let mut reader = WdtReader::new(BufReader::new(file), WowVersion::WotLK);
//! let wdt = reader.read().unwrap();
//!
//! println!("Map has {} tiles", wdt.count_existing_tiles());
//! ```

pub mod chunks;
pub mod conversion;
pub mod error;
pub mod version;

use crate::chunks::{Chunk, MaidChunk, MainChunk, ModfChunk, MphdChunk, MverChunk, MwmoChunk};
use crate::error::{Error, Result};
use crate::version::{VersionConfig, WowVersion};
use std::io::{Read, Seek, SeekFrom, Write};

/// A complete WDT file representation
#[derive(Debug, Clone, PartialEq)]
pub struct WdtFile {
    /// Version chunk (always required)
    pub mver: MverChunk,

    /// Map header chunk (always required)
    pub mphd: MphdChunk,

    /// Map area information (always required)
    pub main: MainChunk,

    /// FileDataIDs for map files (BfA+ only)
    pub maid: Option<MaidChunk>,

    /// Global WMO filename (WMO-only maps, or pre-4.x terrain maps)
    pub mwmo: Option<MwmoChunk>,

    /// Global WMO placement (WMO-only maps)
    pub modf: Option<ModfChunk>,

    /// Version configuration for validation
    pub version_config: VersionConfig,
}

impl WdtFile {
    /// Create a new empty WDT file
    pub fn new(version: WowVersion) -> Self {
        Self {
            mver: MverChunk::new(),
            mphd: MphdChunk::new(),
            main: MainChunk::new(),
            maid: None,
            mwmo: None,
            modf: None,
            version_config: VersionConfig::new(version),
        }
    }

    /// Check if this is a WMO-only map
    pub fn is_wmo_only(&self) -> bool {
        self.mphd.is_wmo_only()
    }

    /// Count tiles with ADT data
    pub fn count_existing_tiles(&self) -> usize {
        if let Some(ref maid) = self.maid {
            maid.count_existing_tiles()
        } else {
            self.main.count_existing_tiles()
        }
    }

    /// Get tile information at coordinates
    pub fn get_tile(&self, x: usize, y: usize) -> Option<TileInfo> {
        let main_entry = self.main.get(x, y)?;

        let has_adt = if let Some(ref maid) = self.maid {
            maid.has_tile(x, y)
        } else {
            main_entry.has_adt()
        };

        Some(TileInfo {
            x,
            y,
            has_adt,
            area_id: main_entry.area_id,
            flags: main_entry.flags,
        })
    }

    /// Get the detected WoW version
    pub fn version(&self) -> WowVersion {
        self.version_config.version
    }

    /// Validate the WDT file structure
    pub fn validate(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        // Version validation
        if self.mver.version != chunks::WDT_VERSION {
            warnings.push(format!(
                "Invalid WDT version: expected {}, found {}",
                chunks::WDT_VERSION,
                self.mver.version
            ));
        }

        // Flag validation
        warnings.extend(
            self.version_config
                .validate_mphd_flags(self.mphd.flags.bits()),
        );

        // Structure validation
        if self.is_wmo_only() {
            if self.mwmo.is_none() {
                warnings.push("WMO-only map missing MWMO chunk".to_string());
            }
            if self.modf.is_none() {
                warnings.push("WMO-only map missing MODF chunk".to_string());
            }
        } else {
            // Terrain map validations
            if self.modf.is_some() {
                warnings.push("Terrain map should not have MODF chunk".to_string());
            }

            // Check MWMO presence based on version
            let should_have_mwmo = self.version_config.should_have_chunk("MWMO", false);
            let has_mwmo = self.mwmo.is_some();

            if should_have_mwmo && !has_mwmo {
                warnings
                    .push("Terrain map missing expected MWMO chunk for this version".to_string());
            } else if !should_have_mwmo && has_mwmo {
                warnings.push("Terrain map has unexpected MWMO chunk for this version".to_string());
            }
        }

        // MAID validation
        if self.mphd.has_maid() && self.maid.is_none() {
            warnings
                .push("MPHD indicates MAID chunk should be present but it's missing".to_string());
        } else if !self.mphd.has_maid() && self.maid.is_some() {
            warnings.push("MAID chunk present but not indicated in MPHD flags".to_string());
        }

        warnings
    }
}

/// Information about a specific tile
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileInfo {
    pub x: usize,
    pub y: usize,
    pub has_adt: bool,
    pub area_id: u32,
    pub flags: u32,
}

/// WDT file reader
pub struct WdtReader<R: Read + Seek> {
    reader: R,
    version: WowVersion,
}

impl<R: Read + Seek> WdtReader<R> {
    /// Create a new WDT reader
    pub fn new(reader: R, version: WowVersion) -> Self {
        Self { reader, version }
    }

    /// Read a complete WDT file
    pub fn read(&mut self) -> Result<WdtFile> {
        let mut wdt = WdtFile::new(self.version);

        // Track which chunks we've seen
        let mut has_mver = false;
        let mut has_mphd = false;
        let mut has_main = false;

        // Read chunks until EOF
        loop {
            match self.read_chunk_header() {
                Ok((magic, size)) => {
                    match &magic {
                        b"REVM" => {
                            wdt.mver = MverChunk::read(&mut self.reader, size)?;
                            has_mver = true;
                        }
                        b"DHPM" => {
                            wdt.mphd = MphdChunk::read(&mut self.reader, size)?;
                            has_mphd = true;
                        }
                        b"NIAM" => {
                            wdt.main = MainChunk::read(&mut self.reader, size)?;
                            has_main = true;
                        }
                        b"DIAM" => {
                            wdt.maid = Some(MaidChunk::read(&mut self.reader, size)?);
                        }
                        b"OMWM" => {
                            wdt.mwmo = Some(MwmoChunk::read(&mut self.reader, size)?);
                        }
                        b"FDOM" => {
                            wdt.modf = Some(ModfChunk::read(&mut self.reader, size)?);
                        }
                        _ => {
                            // Skip unknown chunks
                            self.reader.seek(SeekFrom::Current(size as i64))?;
                        }
                    }
                }
                Err(e) => {
                    // Check if we hit EOF by matching on the Io variant
                    if let Error::Io(ref io_err) = e
                        && io_err.kind() == std::io::ErrorKind::UnexpectedEof
                    {
                        break;
                    }
                    return Err(e);
                }
            }
        }

        // Verify required chunks
        if !has_mver {
            return Err(Error::MissingChunk("MVER".to_string()));
        }
        if !has_mphd {
            return Err(Error::MissingChunk("MPHD".to_string()));
        }
        if !has_main {
            return Err(Error::MissingChunk("MAIN".to_string()));
        }

        // Detect actual version based on chunk presence and content
        let detected_version = self.detect_version(&wdt);

        // Update the WDT's version configuration with detected version
        wdt.version_config = VersionConfig::new(detected_version);

        Ok(wdt)
    }

    /// Detect WoW version based on chunk presence and content
    fn detect_version(&self, wdt: &WdtFile) -> WowVersion {
        // Version detection based on chunk presence and content:
        // 1. MAID chunk (BfA+ 8.x+)
        // 2. MWMO presence rules (Cataclysm+ breaking change)
        // 3. Flag usage patterns

        // Check for MAID chunk (Battle for Azeroth+)
        if wdt.maid.is_some() {
            return WowVersion::BfA;
        }

        // Check MWMO presence rules
        let has_mwmo = wdt.mwmo.is_some();
        let is_wmo_only = wdt.is_wmo_only();

        // In Cataclysm+, terrain maps don't have MWMO chunks
        // Pre-Cataclysm, all maps have MWMO chunks (even if empty)
        if !is_wmo_only && !has_mwmo {
            // Terrain map without MWMO = Cataclysm+
            // Without other indicators, assume Cataclysm
            return WowVersion::Cataclysm;
        } else if !is_wmo_only && has_mwmo {
            // Terrain map with MWMO = pre-Cataclysm
            // Check flags to differentiate further
            let flags = wdt.mphd.flags.bits();

            // Advanced flags suggest later pre-Cata versions
            if (flags & 0x0002) != 0 || (flags & 0x0004) != 0 || (flags & 0x0008) != 0 {
                // These flags were more commonly used from WotLK
                return WowVersion::WotLK;
            } else if (flags & 0x0001) != 0 || flags > 1 {
                // Basic flags suggest TBC+
                return WowVersion::TBC;
            } else {
                // Minimal flags suggest Vanilla
                return WowVersion::Classic;
            }
        }

        // WMO-only maps should have both MWMO and MODF
        if is_wmo_only && wdt.modf.is_some() {
            // Both chunks present, check flag sophistication
            let flags = wdt.mphd.flags.bits();
            if flags > 0x000F {
                return WowVersion::WotLK;
            } else if flags > 1 {
                return WowVersion::TBC;
            } else {
                return WowVersion::Classic;
            }
        }

        // Default fallback based on initial version hint
        self.version
    }

    /// Read a chunk header (magic + size)
    fn read_chunk_header(&mut self) -> Result<([u8; 4], usize)> {
        let mut magic = [0u8; 4];
        self.reader.read_exact(&mut magic)?;

        let mut buf = [0u8; 4];
        self.reader.read_exact(&mut buf)?;
        let size = u32::from_le_bytes(buf) as usize;

        Ok((magic, size))
    }
}

/// WDT file writer
pub struct WdtWriter<W: Write> {
    writer: W,
}

impl<W: Write> WdtWriter<W> {
    /// Create a new WDT writer
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Write a complete WDT file
    pub fn write(&mut self, wdt: &WdtFile) -> Result<()> {
        // Write required chunks in order
        wdt.mver.write_chunk(&mut self.writer)?;
        wdt.mphd.write_chunk(&mut self.writer)?;
        wdt.main.write_chunk(&mut self.writer)?;

        // Write optional chunks
        if let Some(ref maid) = wdt.maid {
            maid.write_chunk(&mut self.writer)?;
        }

        // Write MWMO only if appropriate for the version and map type
        if let Some(ref mwmo) = wdt.mwmo {
            let should_write = wdt
                .version_config
                .should_have_chunk("MWMO", wdt.is_wmo_only());
            if should_write {
                mwmo.write_chunk(&mut self.writer)?;
            }
        }

        if let Some(ref modf) = wdt.modf {
            modf.write_chunk(&mut self.writer)?;
        }

        Ok(())
    }
}

/// Convert ADT tile coordinates to world coordinates
pub fn tile_to_world(tile_x: u32, tile_y: u32) -> (f32, f32) {
    const MAP_SIZE: f32 = 533.333_3;
    const MAP_OFFSET: f32 = 32.0 * MAP_SIZE;

    let world_x = MAP_OFFSET - (tile_y as f32 * MAP_SIZE);
    let world_y = MAP_OFFSET - (tile_x as f32 * MAP_SIZE);

    (world_x, world_y)
}

/// Convert world coordinates to ADT tile coordinates
pub fn world_to_tile(world_x: f32, world_y: f32) -> (u32, u32) {
    const MAP_SIZE: f32 = 533.333_3;
    const MAP_OFFSET: f32 = 32.0 * MAP_SIZE;

    let tile_x = ((MAP_OFFSET - world_y) / MAP_SIZE) as u32;
    let tile_y = ((MAP_OFFSET - world_x) / MAP_SIZE) as u32;

    (tile_x.min(63), tile_y.min(63))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_empty_wdt() {
        let wdt = WdtFile::new(WowVersion::Classic);
        assert_eq!(wdt.count_existing_tiles(), 0);
        assert!(!wdt.is_wmo_only());
    }

    #[test]
    fn test_wdt_read_write() {
        let mut wdt = WdtFile::new(WowVersion::BfA);

        // Set up some test data
        wdt.mphd.flags |= chunks::MphdFlags::ADT_HAS_HEIGHT_TEXTURING;
        wdt.main.get_mut(10, 20).unwrap().set_has_adt(true);
        wdt.main.get_mut(10, 20).unwrap().area_id = 1234;

        // Write to buffer
        let mut buffer = Vec::new();
        let mut writer = WdtWriter::new(&mut buffer);
        writer.write(&wdt).unwrap();

        // Read back
        let mut reader = WdtReader::new(Cursor::new(buffer), WowVersion::BfA);
        let read_wdt = reader.read().unwrap();

        // Verify
        assert_eq!(read_wdt.mphd.flags, wdt.mphd.flags);
        assert_eq!(read_wdt.main.get(10, 20).unwrap().area_id, 1234);
        assert!(read_wdt.main.get(10, 20).unwrap().has_adt());
    }

    #[test]
    fn test_coordinate_conversion() {
        // Test center of map
        let (wx, wy) = tile_to_world(32, 32);
        assert!((wx - 0.0).abs() < 0.1);
        assert!((wy - 0.0).abs() < 0.1);

        // Test reverse conversion
        let (tx, ty) = world_to_tile(wx, wy);
        assert_eq!(tx, 32);
        assert_eq!(ty, 32);
    }
}
