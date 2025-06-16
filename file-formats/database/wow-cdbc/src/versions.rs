//! Support for different DBC file versions

use crate::{DbcHeader, Error, Result};
use std::io::{Read, Seek, SeekFrom};

/// DBC file format version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbcVersion {
    /// Original WDBC format (World of Warcraft Classic)
    WDBC,
    /// World of Warcraft: The Burning Crusade
    WDB2,
    /// World of Warcraft: Wrath of the Lich King
    WDB3,
    /// World of Warcraft: Cataclysm
    WDB4,
    /// World of Warcraft: Mists of Pandaria
    WDB5,
}

impl DbcVersion {
    /// Detect the DBC version from a reader
    pub fn detect<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        reader.seek(SeekFrom::Start(0))?;

        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;

        match &magic {
            b"WDBC" => Ok(DbcVersion::WDBC),
            b"WDB2" => Ok(DbcVersion::WDB2),
            b"WDB3" => Ok(DbcVersion::WDB3),
            b"WDB4" => Ok(DbcVersion::WDB4),
            b"WDB5" => Ok(DbcVersion::WDB5),
            _ => Err(Error::InvalidHeader(format!(
                "Unknown DBC version: {:?}",
                std::str::from_utf8(&magic).unwrap_or("Invalid UTF-8")
            ))),
        }
    }

    /// Get the magic signature for this DBC version
    pub fn magic(&self) -> [u8; 4] {
        match self {
            DbcVersion::WDBC => *b"WDBC",
            DbcVersion::WDB2 => *b"WDB2",
            DbcVersion::WDB3 => *b"WDB3",
            DbcVersion::WDB4 => *b"WDB4",
            DbcVersion::WDB5 => *b"WDB5",
        }
    }
}

/// WDB2 header (The Burning Crusade)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wdb2Header {
    /// The magic signature, should be "WDB2"
    pub magic: [u8; 4],
    /// Number of records in the file
    pub record_count: u32,
    /// Number of fields in each record
    pub field_count: u32,
    /// Size of each record in bytes
    pub record_size: u32,
    /// Size of the string block in bytes
    pub string_block_size: u32,
    /// Table hash
    pub table_hash: u32,
    /// Build number
    pub build: u32,
}

impl Wdb2Header {
    /// The size of a WDB2 header in bytes
    pub const SIZE: usize = 28;

    /// Parse a WDB2 header from a reader
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Ensure we're at the beginning of the file
        reader.seek(SeekFrom::Start(0))?;

        // Read the magic signature
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;

        // Validate the magic signature
        if magic != *b"WDB2" {
            return Err(Error::InvalidHeader(format!(
                "Invalid magic signature: {:?}, expected: {:?}",
                magic, b"WDB2"
            )));
        }

        // Read the rest of the header
        let mut buf = [0u8; 4];

        reader.read_exact(&mut buf)?;
        let record_count = u32::from_le_bytes(buf);

        reader.read_exact(&mut buf)?;
        let field_count = u32::from_le_bytes(buf);

        reader.read_exact(&mut buf)?;
        let record_size = u32::from_le_bytes(buf);

        reader.read_exact(&mut buf)?;
        let string_block_size = u32::from_le_bytes(buf);

        reader.read_exact(&mut buf)?;
        let table_hash = u32::from_le_bytes(buf);

        reader.read_exact(&mut buf)?;
        let build = u32::from_le_bytes(buf);

        Ok(Self {
            magic,
            record_count,
            field_count,
            record_size,
            string_block_size,
            table_hash,
            build,
        })
    }

    /// Convert to a standard DBC header
    pub fn to_dbc_header(&self) -> DbcHeader {
        DbcHeader {
            magic: *b"WDBC", // Convert to WDBC format
            record_count: self.record_count,
            field_count: self.field_count,
            record_size: self.record_size,
            string_block_size: self.string_block_size,
        }
    }

    /// Calculates the offset to the string block
    pub fn string_block_offset(&self) -> u64 {
        Self::SIZE as u64 + (self.record_count as u64 * self.record_size as u64)
    }

    /// Calculates the total size of the WDB2 file
    pub fn total_size(&self) -> u64 {
        self.string_block_offset() + self.string_block_size as u64
    }
}

/// WDB5 header (Mists of Pandaria)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wdb5Header {
    /// The magic signature, should be "WDB5"
    pub magic: [u8; 4],
    /// Number of records in the file
    pub record_count: u32,
    /// Number of fields in each record
    pub field_count: u32,
    /// Size of each record in bytes
    pub record_size: u32,
    /// Size of the string block in bytes
    pub string_block_size: u32,
    /// Table hash
    pub table_hash: u32,
    /// Layout hash
    pub layout_hash: u32,
    /// Min ID
    pub min_id: u32,
    /// Max ID
    pub max_id: u32,
    /// Locale
    pub locale: u32,
    /// Flags
    pub flags: u16,
    /// ID index
    pub id_index: u16,
}

impl Wdb5Header {
    /// The size of a WDB5 header in bytes
    pub const SIZE: usize = 48;

    /// Parse a WDB5 header from a reader
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Ensure we're at the beginning of the file
        reader.seek(SeekFrom::Start(0))?;

        // Read the magic signature
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;

        // Validate the magic signature
        if magic != *b"WDB5" {
            return Err(Error::InvalidHeader(format!(
                "Invalid magic signature: {:?}, expected: {:?}",
                magic, b"WDB5"
            )));
        }

        // Read the rest of the header
        let mut buf4 = [0u8; 4];
        let mut buf2 = [0u8; 2];

        reader.read_exact(&mut buf4)?;
        let record_count = u32::from_le_bytes(buf4);

        reader.read_exact(&mut buf4)?;
        let field_count = u32::from_le_bytes(buf4);

        reader.read_exact(&mut buf4)?;
        let record_size = u32::from_le_bytes(buf4);

        reader.read_exact(&mut buf4)?;
        let string_block_size = u32::from_le_bytes(buf4);

        reader.read_exact(&mut buf4)?;
        let table_hash = u32::from_le_bytes(buf4);

        reader.read_exact(&mut buf4)?;
        let layout_hash = u32::from_le_bytes(buf4);

        reader.read_exact(&mut buf4)?;
        let min_id = u32::from_le_bytes(buf4);

        reader.read_exact(&mut buf4)?;
        let max_id = u32::from_le_bytes(buf4);

        reader.read_exact(&mut buf4)?;
        let locale = u32::from_le_bytes(buf4);

        reader.read_exact(&mut buf2)?;
        let flags = u16::from_le_bytes(buf2);

        reader.read_exact(&mut buf2)?;
        let id_index = u16::from_le_bytes(buf2);

        Ok(Self {
            magic,
            record_count,
            field_count,
            record_size,
            string_block_size,
            table_hash,
            layout_hash,
            min_id,
            max_id,
            locale,
            flags,
            id_index,
        })
    }

    /// Convert to a standard DBC header
    pub fn to_dbc_header(&self) -> DbcHeader {
        DbcHeader {
            magic: *b"WDBC", // Convert to WDBC format
            record_count: self.record_count,
            field_count: self.field_count,
            record_size: self.record_size,
            string_block_size: self.string_block_size,
        }
    }

    /// Calculates the offset to the string block
    pub fn string_block_offset(&self) -> u64 {
        Self::SIZE as u64 + (self.record_count as u64 * self.record_size as u64)
    }

    /// Calculates the total size of the WDB5 file
    pub fn total_size(&self) -> u64 {
        self.string_block_offset() + self.string_block_size as u64
    }
}
