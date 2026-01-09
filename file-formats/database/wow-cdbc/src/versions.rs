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

/// WDB2 header (Cataclysm 4.0+)
///
/// The WDB2 format was introduced in Cataclysm and has two variants:
/// - Basic header (build <= 12880): 28 bytes
/// - Extended header (build > 12880): 48 bytes + optional index arrays
///
/// Reference: https://wowdev.wiki/DB2
/// Reference: TrinityCore DB2StorageLoader.cpp
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Timestamp (only present in extended header)
    pub timestamp: u32,
    // Extended header fields (build > 12880)
    /// Minimum ID in the file
    pub min_index: i32,
    /// Maximum ID in the file
    pub max_index: i32,
    /// Locale flags
    pub locale: i32,
    /// Copy table size (unused in Cataclysm, always 0)
    pub copy_table_size: u32,
    /// Whether this header uses the extended format
    pub has_extended_header: bool,
    /// Size of the index array section (to be skipped)
    pub index_array_size: u64,
}

impl Wdb2Header {
    /// The size of a basic WDB2 header in bytes (build <= 12880)
    pub const BASIC_SIZE: usize = 28;

    /// The size of an extended WDB2 header in bytes (build > 12880)
    pub const EXTENDED_SIZE: usize = 48;

    /// Build number threshold for extended header format
    pub const EXTENDED_BUILD_THRESHOLD: u32 = 12880;

    /// Parse a WDB2 header from a reader
    ///
    /// This handles both the basic (build <= 12880) and extended (build > 12880) formats.
    /// For builds > 12880, also skips the index arrays if max_index != 0.
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

        // Read the basic header fields
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

        // Read timestamp (present in all WDB2 files)
        reader.read_exact(&mut buf)?;
        let timestamp = u32::from_le_bytes(buf);

        // Check if this is an extended header (build > 12880)
        let has_extended_header = build > Self::EXTENDED_BUILD_THRESHOLD;

        let (min_index, max_index, locale, copy_table_size, index_array_size) =
            if has_extended_header {
                // Read extended header fields
                reader.read_exact(&mut buf)?;
                let min_index = i32::from_le_bytes(buf);

                reader.read_exact(&mut buf)?;
                let max_index = i32::from_le_bytes(buf);

                reader.read_exact(&mut buf)?;
                let locale = i32::from_le_bytes(buf);

                reader.read_exact(&mut buf)?;
                let copy_table_size = u32::from_le_bytes(buf);

                // Calculate index array size to skip
                let index_array_size = if max_index > 0 {
                    let diff = (max_index - min_index + 1) as u64;
                    // Index array: diff * 4 bytes (u32 per entry)
                    // String length array: diff * 2 bytes (u16 per entry)
                    diff * 4 + diff * 2
                } else {
                    0
                };

                // Skip the index arrays
                if index_array_size > 0 {
                    reader.seek(SeekFrom::Current(index_array_size as i64))?;
                }

                (
                    min_index,
                    max_index,
                    locale,
                    copy_table_size,
                    index_array_size,
                )
            } else {
                (0, 0, 0, 0, 0)
            };

        Ok(Self {
            magic,
            record_count,
            field_count,
            record_size,
            string_block_size,
            table_hash,
            build,
            timestamp,
            min_index,
            max_index,
            locale,
            copy_table_size,
            has_extended_header,
            index_array_size,
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

    /// Calculates the size of the header including any index arrays
    pub fn header_size(&self) -> u64 {
        if self.has_extended_header {
            Self::EXTENDED_SIZE as u64 + self.index_array_size
        } else {
            Self::BASIC_SIZE as u64
        }
    }

    /// Calculates the offset to the record data
    pub fn record_data_offset(&self) -> u64 {
        self.header_size()
    }

    /// Calculates the offset to the string block
    pub fn string_block_offset(&self) -> u64 {
        self.header_size() + (self.record_count as u64 * self.record_size as u64)
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
