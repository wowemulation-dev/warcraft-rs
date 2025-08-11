//! BET (Block Extended Table) implementation for MPQ v3+ archives

use super::common::decrypt_table_data;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::compression::decompress;
use crate::{Error, Result};
use std::io::{Read, Seek, SeekFrom};

/// BET (Block Entry Table) for v3+ archives
#[derive(Debug)]
pub struct BetTable {
    /// Table header data
    pub header: BetHeader,
    /// File flags array
    pub file_flags: Vec<u32>,
    /// File table (bit-packed)
    pub file_table: Vec<u8>,
    /// BET hash array
    pub bet_hashes: Vec<u64>,
}

/// Block Entry Table (BET) header structure for MPQ v3+
/// This follows the extended header in the file
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct BetHeader {
    /// Size of the entire table including header
    pub table_size: u32,
    /// Number of files in BET table
    pub file_count: u32,
    /// Unknown, typically 0x10
    pub unknown_08: u32,
    /// Size of one table entry in bits
    pub table_entry_size: u32,
    /// Bit positions for various fields
    pub bit_index_file_pos: u32,
    /// Bit index for file size field
    pub bit_index_file_size: u32,
    /// Bit index for compressed size field
    pub bit_index_cmp_size: u32,
    /// Bit index for flag index field
    pub bit_index_flag_index: u32,
    /// Bit index for unknown field
    pub bit_index_unknown: u32,
    /// Bit counts for various fields
    pub bit_count_file_pos: u32,
    /// Bit count for file size field
    pub bit_count_file_size: u32,
    /// Bit count for compressed size field
    pub bit_count_cmp_size: u32,
    /// Bit count for flag index field
    pub bit_count_flag_index: u32,
    /// Bit count for unknown field
    pub bit_count_unknown: u32,
    /// BET hash information
    pub total_bet_hash_size: u32,
    /// Extra bits in BET hash size
    pub bet_hash_size_extra: u32,
    /// Size of BET hash
    pub bet_hash_size: u32,
    /// Size of BET hash array
    pub bet_hash_array_size: u32,
    /// Number of flags
    pub flag_count: u32,
}

impl BetTable {
    const SIGNATURE: u32 = 0x1A544542; // "BET\x1A"

    /// Read and decompress/decrypt a BET table
    pub fn read<R: Read + Seek>(
        reader: &mut R,
        offset: u64,
        compressed_size: u64,
        key: u32,
    ) -> Result<Self> {
        reader.seek(SeekFrom::Start(offset))?;

        // Read the compressed/encrypted data
        let mut data = vec![0u8; compressed_size as usize];
        reader.read_exact(&mut data)?;

        // Check if we have at least the extended header (12 bytes)
        if data.len() < 12 {
            return Err(Error::invalid_format(
                "BET table too small for extended header",
            ));
        }

        // Parse the extended header (first 12 bytes - never encrypted)
        let ext_signature = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let ext_version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let ext_data_size = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);

        log::debug!(
            "BET extended header: sig=0x{ext_signature:08X}, ver={ext_version}, data_size={ext_data_size}"
        );

        // Verify extended header
        if ext_signature != Self::SIGNATURE {
            // Check if we got HET signature instead - this is a known issue in some archives
            if ext_signature == 0x1A544548 {
                return Err(Error::invalid_format(
                    "BET offset points to HET table - archive has swapped table offsets",
                ));
            }
            return Err(Error::invalid_format("Invalid BET extended signature"));
        }

        // The data after the extended header may be encrypted
        if key != 0 && data.len() > 12 {
            log::debug!("Decrypting BET data after extended header with key 0x{key:08X}");
            let data_portion = &mut data[12..];
            decrypt_table_data(data_portion, key);
        }

        // Check for compression by comparing sizes
        let total_size = data.len();
        let expected_uncompressed_size = ext_data_size as usize + 12; // data_size + header

        log::debug!(
            "BET table total_size={total_size}, expected_uncompressed_size={expected_uncompressed_size}"
        );

        let table_data = if expected_uncompressed_size > total_size {
            // Table is compressed - the data after extended header contains compressed data
            log::debug!("BET table is compressed");

            if data.len() <= 12 {
                return Err(Error::invalid_format(
                    "No compressed data after BET extended header",
                ));
            }

            // First byte after extended header is compression type
            let compression_type = data[12];
            log::debug!("BET compression type: 0x{compression_type:02X}");

            // Decompress the data (skip extended header and compression byte)
            let compressed_data = &data[13..];
            let mut decompressed =
                decompress(compressed_data, compression_type, ext_data_size as usize)?;

            // Reconstruct the full table with extended header
            let mut full_table = Vec::with_capacity(12 + decompressed.len());
            full_table.extend_from_slice(&data[..12]); // Extended header
            full_table.append(&mut decompressed); // Decompressed data
            full_table
        } else {
            // Table is not compressed
            log::debug!("BET table is NOT compressed");
            data
        };

        // Parse header - skip the extended header (first 12 bytes)
        let header = Self::parse_header(&table_data[12..])?;

        // Copy values to avoid packed struct issues
        let file_count = header.file_count;
        let table_entry_size = header.table_entry_size;
        let flag_count = header.flag_count;
        let bit_count_file_pos = header.bit_count_file_pos;
        let bit_index_file_pos = header.bit_index_file_pos;
        let bit_count_file_size = header.bit_count_file_size;
        let bit_index_file_size = header.bit_index_file_size;
        let bit_count_cmp_size = header.bit_count_cmp_size;
        let bit_index_cmp_size = header.bit_index_cmp_size;
        let bit_count_flag_index = header.bit_count_flag_index;
        let bit_index_flag_index = header.bit_index_flag_index;

        log::debug!(
            "BET header: file_count={file_count}, table_entry_size={table_entry_size} bits, flag_count={flag_count}"
        );
        log::debug!(
            "BET field bits: file_pos={bit_count_file_pos} at {bit_index_file_pos}, file_size={bit_count_file_size} at {bit_index_file_size}, cmp_size={bit_count_cmp_size} at {bit_index_cmp_size}, flag_index={bit_count_flag_index} at {bit_index_flag_index}"
        );

        // No need to validate signature/version - they're in the extended header
        // which we already validated above

        // Parse the rest of the table - data starts after extended header + BET header
        let data_start = 12 + std::mem::size_of::<BetHeader>();
        let mut cursor = std::io::Cursor::new(&table_data[data_start..]);

        // Read file flags
        let mut file_flags = Vec::with_capacity(header.flag_count as usize);
        for _ in 0..header.flag_count {
            file_flags.push(cursor.read_u32::<LittleEndian>()?);
        }

        // Calculate sizes
        let file_table_size =
            (header.file_count as usize * header.table_entry_size as usize).div_ceil(8);
        let mut file_table = vec![0u8; file_table_size];
        cursor.read_exact(&mut file_table)?;

        // Read BET hashes
        let hash_count = header.bet_hash_array_size / 8; // Each hash is 8 bytes
        let mut bet_hashes = Vec::with_capacity(hash_count as usize);
        for _ in 0..hash_count {
            bet_hashes.push(cursor.read_u64::<LittleEndian>()?);
        }

        Ok(Self {
            header,
            file_flags,
            file_table,
            bet_hashes,
        })
    }

    /// Parse header from raw bytes
    fn parse_header(data: &[u8]) -> Result<BetHeader> {
        if data.len() < std::mem::size_of::<BetHeader>() {
            return Err(Error::invalid_format("BET header too small"));
        }

        let mut cursor = std::io::Cursor::new(data);
        Ok(BetHeader {
            table_size: cursor.read_u32::<LittleEndian>()?,
            file_count: cursor.read_u32::<LittleEndian>()?,
            unknown_08: cursor.read_u32::<LittleEndian>()?,
            table_entry_size: cursor.read_u32::<LittleEndian>()?,
            bit_index_file_pos: cursor.read_u32::<LittleEndian>()?,
            bit_index_file_size: cursor.read_u32::<LittleEndian>()?,
            bit_index_cmp_size: cursor.read_u32::<LittleEndian>()?,
            bit_index_flag_index: cursor.read_u32::<LittleEndian>()?,
            bit_index_unknown: cursor.read_u32::<LittleEndian>()?,
            bit_count_file_pos: cursor.read_u32::<LittleEndian>()?,
            bit_count_file_size: cursor.read_u32::<LittleEndian>()?,
            bit_count_cmp_size: cursor.read_u32::<LittleEndian>()?,
            bit_count_flag_index: cursor.read_u32::<LittleEndian>()?,
            bit_count_unknown: cursor.read_u32::<LittleEndian>()?,
            total_bet_hash_size: cursor.read_u32::<LittleEndian>()?,
            bet_hash_size_extra: cursor.read_u32::<LittleEndian>()?,
            bet_hash_size: cursor.read_u32::<LittleEndian>()?,
            bet_hash_array_size: cursor.read_u32::<LittleEndian>()?,
            flag_count: cursor.read_u32::<LittleEndian>()?,
        })
    }

    /// Get file information by index
    pub fn get_file_info(&self, index: u32) -> Option<BetFileInfo> {
        let file_count = self.header.file_count;
        if index >= file_count {
            log::debug!("BET get_file_info: index {index} >= file_count {file_count}");
            return None;
        }

        log::debug!("BET get_file_info: getting info for file index {index}");

        // Calculate the bit position for this entry
        let table_entry_size = self.header.table_entry_size as usize;
        let entry_bit_position = index as usize * table_entry_size;

        log::debug!(
            "BET get_file_info: entry at bit position {entry_bit_position}, entry size {table_entry_size} bits"
        );

        // Read each field directly from the bit stream
        let file_pos = self.read_bits_from_table(
            entry_bit_position + self.header.bit_index_file_pos as usize,
            self.header.bit_count_file_pos,
        )?;

        let file_size = self.read_bits_from_table(
            entry_bit_position + self.header.bit_index_file_size as usize,
            self.header.bit_count_file_size,
        )?;

        let cmp_size = self.read_bits_from_table(
            entry_bit_position + self.header.bit_index_cmp_size as usize,
            self.header.bit_count_cmp_size,
        )?;

        let flag_index = self.read_bits_from_table(
            entry_bit_position + self.header.bit_index_flag_index as usize,
            self.header.bit_count_flag_index,
        )? as u32;

        log::debug!(
            "BET get_file_info: file_pos=0x{file_pos:X}, file_size={file_size}, cmp_size={cmp_size}, flag_index={flag_index}"
        );

        // Get flags
        let flags = if flag_index < self.header.flag_count {
            self.file_flags[flag_index as usize]
        } else {
            0
        };

        Some(BetFileInfo {
            file_pos,
            file_size,
            compressed_size: cmp_size,
            flags,
        })
    }

    /// Read bits from file table at the given bit position
    fn read_bits_from_table(&self, bit_position: usize, bit_count: u32) -> Option<u64> {
        if bit_count == 0 {
            return Some(0);
        }

        if bit_count > 64 {
            log::warn!("read_bits_from_table: bit_count {bit_count} > 64");
            return None;
        }

        let byte_offset = bit_position / 8;
        let bit_shift = bit_position % 8;
        let bytes_needed = ((bit_position % 8) + bit_count as usize).div_ceil(8);

        if byte_offset + bytes_needed > self.file_table.len() {
            return None;
        }

        // Read enough bytes to cover the bits we need
        let mut value = 0u64;
        for i in 0..bytes_needed.min(8) {
            if byte_offset + i < self.file_table.len() {
                value |= (self.file_table[byte_offset + i] as u64) << (i * 8);
            }
        }

        // Extract the bits we want
        let mask = if bit_count >= 64 {
            u64::MAX
        } else {
            (1u64 << bit_count) - 1
        };
        Some((value >> bit_shift) & mask)
    }
}

/// File information from BET table
#[derive(Debug)]
pub struct BetFileInfo {
    /// File position in archive
    pub file_pos: u64,
    /// Uncompressed file size
    pub file_size: u64,
    /// Compressed file size
    pub compressed_size: u64,
    /// File flags
    pub flags: u32,
}
