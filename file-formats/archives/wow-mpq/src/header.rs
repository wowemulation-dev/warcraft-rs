//! MPQ header structures and parsing

use crate::security::{SecurityLimits, validate_header_security};
use crate::{Error, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Seek, SeekFrom};

use crate::debug::{format_size, hex_string};

/// MPQ archive header signature ('MPQ\x1A')
pub const MPQ_HEADER_SIGNATURE: u32 = 0x1A51504D;

/// MPQ user data header signature ('MPQ\x1B')
pub const MPQ_USERDATA_SIGNATURE: u32 = 0x1B51504D;

/// Header alignment requirement (512 bytes)
pub const HEADER_ALIGNMENT: u64 = 0x200;

/// MPQ format version
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormatVersion {
    /// Version 1 - Original format (32-byte header)
    V1 = 0,
    /// Version 2 - Burning Crusade (44-byte header)
    V2 = 1,
    /// Version 3 - Cataclysm Beta (68-byte header)
    V3 = 2,
    /// Version 4 - Cataclysm+ (208-byte header)
    V4 = 3,
}

impl FormatVersion {
    /// Get the header size for this version
    pub fn header_size(&self) -> u32 {
        match self {
            FormatVersion::V1 => 0x20, // 32 bytes
            FormatVersion::V2 => 0x2C, // 44 bytes
            FormatVersion::V3 => 0x44, // 68 bytes
            FormatVersion::V4 => 0xD0, // 208 bytes
        }
    }

    /// Create from raw version number
    pub fn from_raw(raw: u16) -> Option<Self> {
        match raw {
            0 => Some(FormatVersion::V1),
            1 => Some(FormatVersion::V2),
            2 => Some(FormatVersion::V3),
            3 => Some(FormatVersion::V4),
            _ => None,
        }
    }
}

/// MPQ user data header (optional, appears before main header)
#[derive(Debug, Clone)]
pub struct UserDataHeader {
    /// Maximum size of the user data
    pub user_data_size: u32,
    /// Offset of the MPQ header, relative to the beginning of this header
    pub header_offset: u32,
    /// Size of user data header (commonly used in SC2 maps)
    pub user_data_header_size: u32,
}

/// Main MPQ header structure
#[derive(Debug, Clone)]
pub struct MpqHeader {
    /// Size of the archive header
    pub header_size: u32,
    /// Size of MPQ archive (deprecated in v2+)
    pub archive_size: u32,
    /// Format version
    pub format_version: FormatVersion,
    /// Block size (power of two exponent)
    pub block_size: u16,
    /// Offset to the hash table
    pub hash_table_pos: u32,
    /// Offset to the block table
    pub block_table_pos: u32,
    /// Number of entries in the hash table
    pub hash_table_size: u32,
    /// Number of entries in the block table
    pub block_table_size: u32,

    // Version 2+ fields
    /// Extended block table position
    pub hi_block_table_pos: Option<u64>,
    /// High 16 bits of hash table offset
    pub hash_table_pos_hi: Option<u16>,
    /// High 16 bits of block table offset
    pub block_table_pos_hi: Option<u16>,

    // Version 3+ fields
    /// 64-bit archive size
    pub archive_size_64: Option<u64>,
    /// Position of BET table
    pub bet_table_pos: Option<u64>,
    /// Position of HET table
    pub het_table_pos: Option<u64>,

    // Version 4 fields
    /// Compressed sizes and MD5 hashes
    pub v4_data: Option<MpqHeaderV4Data>,
}

/// Version 4 specific header data
#[derive(Debug, Clone)]
pub struct MpqHeaderV4Data {
    /// Compressed size of hash table
    pub hash_table_size_64: u64,
    /// Compressed size of block table
    pub block_table_size_64: u64,
    /// Compressed size of hi-block table
    pub hi_block_table_size_64: u64,
    /// Compressed size of HET table
    pub het_table_size_64: u64,
    /// Compressed size of BET table
    pub bet_table_size_64: u64,
    /// Size of raw data chunk for MD5
    pub raw_chunk_size: u32,
    /// MD5 of block table
    pub md5_block_table: [u8; 16],
    /// MD5 of hash table
    pub md5_hash_table: [u8; 16],
    /// MD5 of hi-block table
    pub md5_hi_block_table: [u8; 16],
    /// MD5 of BET table
    pub md5_bet_table: [u8; 16],
    /// MD5 of HET table
    pub md5_het_table: [u8; 16],
    /// MD5 of MPQ header
    pub md5_mpq_header: [u8; 16],
}

impl MpqHeader {
    /// Read an MPQ header from the given reader with security validation
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Self::read_with_limits(reader, &SecurityLimits::default())
    }

    /// Read an MPQ header with custom security limits
    pub fn read_with_limits<R: Read + Seek>(
        reader: &mut R,
        limits: &SecurityLimits,
    ) -> Result<Self> {
        // Read the signature
        let signature = reader.read_u32::<LittleEndian>()?;
        if signature != MPQ_HEADER_SIGNATURE {
            return Err(Error::invalid_format("Invalid MPQ header signature"));
        }

        // Remember where the header starts (after signature)
        let header_start = reader.stream_position()? - 4;

        // Read basic header fields
        let header_size = reader.read_u32::<LittleEndian>()?;
        let archive_size = reader.read_u32::<LittleEndian>()?;
        let format_version_raw = reader.read_u16::<LittleEndian>()?;
        let block_size = reader.read_u16::<LittleEndian>()?;
        let hash_table_pos = reader.read_u32::<LittleEndian>()?;
        let block_table_pos = reader.read_u32::<LittleEndian>()?;
        let hash_table_size = reader.read_u32::<LittleEndian>()?;
        let block_table_size = reader.read_u32::<LittleEndian>()?;

        let format_version = FormatVersion::from_raw(format_version_raw)
            .ok_or(Error::UnsupportedVersion(format_version_raw))?;

        // Security validation - validate header before proceeding
        validate_header_security(
            signature,
            header_size,
            archive_size,
            format_version_raw,
            block_size,
            hash_table_pos,
            block_table_pos,
            hash_table_size,
            block_table_size,
            limits,
        )?;

        // Validate header size
        if header_size < format_version.header_size() {
            return Err(Error::invalid_format(format!(
                "Header size {header_size} too small for version {format_version:?}"
            )));
        }

        let mut header = MpqHeader {
            header_size,
            archive_size,
            format_version,
            block_size,
            hash_table_pos,
            block_table_pos,
            hash_table_size,
            block_table_size,
            hi_block_table_pos: None,
            hash_table_pos_hi: None,
            block_table_pos_hi: None,
            archive_size_64: None,
            bet_table_pos: None,
            het_table_pos: None,
            v4_data: None,
        };

        // Read version-specific fields
        if format_version >= FormatVersion::V2 {
            // Version 2+ fields
            header.hi_block_table_pos = Some(reader.read_u64::<LittleEndian>()?);
            header.hash_table_pos_hi = Some(reader.read_u16::<LittleEndian>()?);
            header.block_table_pos_hi = Some(reader.read_u16::<LittleEndian>()?);
        }

        if format_version >= FormatVersion::V3 {
            // Version 3+ fields
            header.archive_size_64 = Some(reader.read_u64::<LittleEndian>()?);
            header.bet_table_pos = Some(reader.read_u64::<LittleEndian>()?);
            header.het_table_pos = Some(reader.read_u64::<LittleEndian>()?);
        }

        // Check if we have V4 data based on header size
        // Some V3 archives have 208-byte headers with V4 data
        if format_version >= FormatVersion::V3 && header_size >= 208 {
            // First check if we have enough data to read V4 fields
            let current_pos = reader.stream_position()?;
            let remaining_header = (header_start + header_size as u64).saturating_sub(current_pos);
            let v4_data_size = 8 * 5 + 4 + 16 * 6; // 5 u64s + 1 u32 + 6 MD5s

            if remaining_header < v4_data_size as u64 {
                log::warn!(
                    "Header size is 208 but not enough space for V4 data. Remaining: {remaining_header}, needed: {v4_data_size}"
                );
            } else {
                // Version 4 fields
                let mut v4_data = MpqHeaderV4Data {
                    hash_table_size_64: reader.read_u64::<LittleEndian>()?,
                    block_table_size_64: reader.read_u64::<LittleEndian>()?,
                    hi_block_table_size_64: reader.read_u64::<LittleEndian>()?,
                    het_table_size_64: reader.read_u64::<LittleEndian>()?,
                    bet_table_size_64: reader.read_u64::<LittleEndian>()?,
                    raw_chunk_size: reader.read_u32::<LittleEndian>()?,
                    md5_block_table: [0; 16],
                    md5_hash_table: [0; 16],
                    md5_hi_block_table: [0; 16],
                    md5_bet_table: [0; 16],
                    md5_het_table: [0; 16],
                    md5_mpq_header: [0; 16],
                };

                reader.read_exact(&mut v4_data.md5_block_table)?;
                reader.read_exact(&mut v4_data.md5_hash_table)?;
                reader.read_exact(&mut v4_data.md5_hi_block_table)?;
                reader.read_exact(&mut v4_data.md5_bet_table)?;
                reader.read_exact(&mut v4_data.md5_het_table)?;
                reader.read_exact(&mut v4_data.md5_mpq_header)?;

                header.v4_data = Some(v4_data);
            }
        }

        Ok(header)
    }

    /// Get the actual archive size (using 64-bit value if available)
    pub fn get_archive_size(&self) -> u64 {
        self.archive_size_64.unwrap_or(self.archive_size as u64)
    }

    /// Get the full hash table position
    pub fn get_hash_table_pos(&self) -> u64 {
        if let Some(hi) = self.hash_table_pos_hi {
            ((hi as u64) << 32) | (self.hash_table_pos as u64)
        } else {
            self.hash_table_pos as u64
        }
    }

    /// Get the full block table position
    pub fn get_block_table_pos(&self) -> u64 {
        if let Some(hi) = self.block_table_pos_hi {
            ((hi as u64) << 32) | (self.block_table_pos as u64)
        } else {
            self.block_table_pos as u64
        }
    }

    /// Calculate the sector size from block size
    pub fn sector_size(&self) -> usize {
        512 << self.block_size
    }
}

/// Find the MPQ header in a file
pub fn find_header<R: Read + Seek>(
    reader: &mut R,
) -> Result<(u64, Option<UserDataHeader>, MpqHeader)> {
    find_header_with_limits(reader, &SecurityLimits::default())
}

/// Find MPQ header with custom security limits
pub fn find_header_with_limits<R: Read + Seek>(
    reader: &mut R,
    limits: &SecurityLimits,
) -> Result<(u64, Option<UserDataHeader>, MpqHeader)> {
    let mut offset = 0u64;
    let file_size = reader.seek(SeekFrom::End(0))?;
    reader.seek(SeekFrom::Start(0))?;

    loop {
        if offset >= file_size {
            return Err(Error::invalid_format("No MPQ header found"));
        }

        reader.seek(SeekFrom::Start(offset))?;

        // Try to read a signature
        let signature = match reader.read_u32::<LittleEndian>() {
            Ok(sig) => sig,
            Err(_) => {
                offset += HEADER_ALIGNMENT;
                continue;
            }
        };

        match signature {
            MPQ_HEADER_SIGNATURE => {
                // Found standard MPQ header
                reader.seek(SeekFrom::Start(offset))?;
                let header = MpqHeader::read_with_limits(reader, limits)?;
                return Ok((offset, None, header));
            }
            MPQ_USERDATA_SIGNATURE => {
                // Found user data header
                let user_data_size = reader.read_u32::<LittleEndian>()?;
                let header_offset = reader.read_u32::<LittleEndian>()?;
                let user_data_header_size = reader.read_u32::<LittleEndian>()?;

                let user_data = UserDataHeader {
                    user_data_size,
                    header_offset,
                    user_data_header_size,
                };

                // Calculate actual header position
                let mpq_offset = offset + header_offset as u64;
                if mpq_offset < file_size {
                    reader.seek(SeekFrom::Start(mpq_offset))?;

                    // Verify there's an MPQ header at the calculated position
                    let mpq_sig = reader.read_u32::<LittleEndian>()?;
                    if mpq_sig == MPQ_HEADER_SIGNATURE {
                        reader.seek(SeekFrom::Start(mpq_offset))?;
                        let header = MpqHeader::read_with_limits(reader, limits)?;
                        return Ok((mpq_offset, Some(user_data), header));
                    }
                }
            }
            _ => {}
        }

        // Move to next potential header position
        offset += HEADER_ALIGNMENT;
    }
}

impl MpqHeader {
    /// Debug dump the MPQ header
    pub fn debug_dump(&self) -> String {
        let mut output = String::new();
        output.push_str("MPQ Header Debug Dump\n");
        output.push_str("====================\n");
        output.push_str(&format!("Header Size: {}\n", self.header_size));
        output.push_str(&format!(
            "Archive Size (v1): {}\n",
            format_size(self.archive_size as u64)
        ));
        output.push_str(&format!(
            "Format Version: {:?} (v{})\n",
            self.format_version,
            self.format_version as u16 + 1
        ));
        output.push_str(&format!(
            "Block Size: {} (sector size: {})\n",
            self.block_size,
            format_size((512 << self.block_size) as u64)
        ));
        output.push_str(&format!(
            "Hash Table: offset=0x{:08X}, size={} entries\n",
            self.hash_table_pos, self.hash_table_size
        ));
        output.push_str(&format!(
            "Block Table: offset=0x{:08X}, size={} entries\n",
            self.block_table_pos, self.block_table_size
        ));

        if let Some(hi_pos) = self.hi_block_table_pos {
            output.push_str("\nVersion 2+ fields:\n");
            output.push_str(&format!("  Hi-Block Table: offset=0x{hi_pos:016X}\n"));
            output.push_str(&format!(
                "  Hash Table High: 0x{:04X}\n",
                self.hash_table_pos_hi.unwrap_or(0)
            ));
            output.push_str(&format!(
                "  Block Table High: 0x{:04X}\n",
                self.block_table_pos_hi.unwrap_or(0)
            ));
        }

        if let Some(size64) = self.archive_size_64 {
            output.push_str("\nVersion 3+ fields:\n");
            output.push_str(&format!(
                "  Archive Size (64-bit): {}\n",
                format_size(size64)
            ));
            output.push_str(&format!(
                "  BET Table: offset=0x{:016X}\n",
                self.bet_table_pos.unwrap_or(0)
            ));
            output.push_str(&format!(
                "  HET Table: offset=0x{:016X}\n",
                self.het_table_pos.unwrap_or(0)
            ));
        }

        if let Some(ref v4) = self.v4_data {
            output.push_str("\nVersion 4 fields:\n");
            output.push_str(&format!(
                "  Hash Table Size: {}\n",
                format_size(v4.hash_table_size_64)
            ));
            output.push_str(&format!(
                "  Block Table Size: {}\n",
                format_size(v4.block_table_size_64)
            ));
            output.push_str(&format!(
                "  Hi-Block Table Size: {}\n",
                format_size(v4.hi_block_table_size_64)
            ));
            output.push_str(&format!(
                "  HET Table Size: {}\n",
                format_size(v4.het_table_size_64)
            ));
            output.push_str(&format!(
                "  BET Table Size: {}\n",
                format_size(v4.bet_table_size_64)
            ));
            output.push_str(&format!(
                "  Raw Chunk Size: {}\n",
                format_size(v4.raw_chunk_size as u64)
            ));
            output.push_str("  MD5 Hashes:\n");
            output.push_str(&format!(
                "    Block Table: {}\n",
                hex_string(&v4.md5_block_table, 16)
            ));
            output.push_str(&format!(
                "    Hash Table: {}\n",
                hex_string(&v4.md5_hash_table, 16)
            ));
            output.push_str(&format!(
                "    Hi-Block Table: {}\n",
                hex_string(&v4.md5_hi_block_table, 16)
            ));
            output.push_str(&format!(
                "    BET Table: {}\n",
                hex_string(&v4.md5_bet_table, 16)
            ));
            output.push_str(&format!(
                "    HET Table: {}\n",
                hex_string(&v4.md5_het_table, 16)
            ));
            output.push_str(&format!(
                "    MPQ Header: {}\n",
                hex_string(&v4.md5_mpq_header, 16)
            ));
        }

        output
    }
}

impl UserDataHeader {
    /// Debug dump the user data header
    pub fn debug_dump(&self) -> String {
        let mut output = String::new();
        output.push_str("MPQ User Data Header\n");
        output.push_str("===================\n");
        output.push_str(&format!(
            "User Data Size: {}\n",
            format_size(self.user_data_size as u64)
        ));
        output.push_str(&format!("Header Offset: 0x{:08X}\n", self.header_offset));
        output.push_str(&format!(
            "User Data Header Size: {}\n",
            self.user_data_header_size
        ));
        output
    }
}
