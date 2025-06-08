# MPQ Format

MPQ (Mo'PaQ, named after its creator Mike O'Brien) is Blizzard's proprietary archive
format used to store game assets. The format has evolved through multiple versions
to support larger files and improved security.

## Overview

- **Extension**: `.mpq`
- **Magic Numbers**:
  - `MPQ\x1A` (0x1A51504D) - Standard MPQ header
  - `MPQ\x1B` (0x1B51504D) - MPQ user data header
- **Alignment**: Headers must start at offsets aligned to 512 (0x200) bytes
- **Versions**:
  - v1: Original format (up to The Burning Crusade)
  - v2: Extended format with large file support (The Burning Crusade)
  - v3: HET/BET tables support (Cataclysm beta)
  - v4: Enhanced security with MD5 hashes (Cataclysm)

## File Layout

MPQ archives have a flexible structure that allows embedding in other files:

1. **Pre-archive data** (optional) - Allows MPQs to be appended to executables
2. **User data header** (optional) - Custom data used in Starcraft II maps
3. **MPQ header** (required) - Main archive header
4. **File data** - Actual archived file contents
5. **Special files** (optional) - `(listfile)`, `(attributes)`, `(signature)`
6. **HET table** (optional, v3+) - Extended hash table
7. **BET table** (optional, v3+) - Extended block table
8. **Hash table** (optional in v3+) - File lookup table
9. **Block table** (optional in v3+) - File information table
10. **High block table** (optional, v2+) - Upper 16 bits of file offsets
11. **Strong signature** (optional) - RSA signature for security

## Data Structures

### User Data Header

Found in some archives (particularly Starcraft II maps) at aligned offsets:

```rust
#[repr(C)]
struct UserDataHeader {
    /// Magic signature 'MPQ\x1B'
    signature: [u8; 4],

    /// Maximum size of user data area
    user_data_max_size: u32,

    /// Offset to MPQ header from start of this structure
    archive_header_offset: u32,

    /// Size of this user data header
    user_data_header_size: u32,
}
```

### MPQ Headers

The MPQ header structure varies by version:

```rust
#[repr(C)]
struct ArchiveHeaderV1 {
    /// Magic signature 'MPQ\x1A'
    signature: [u8; 4],

    /// Size of this header structure
    header_size: u32,

    /// Size of the entire archive (deprecated in v2+)
    archive_size: u32,

    /// Format version (0 = v1, 1 = v2, 2 = v3, 3 = v4)
    format_version: u16,

    /// Block size as power of two: 512 * 2^block_size_shift
    block_size_shift: u16,

    /// Offset to hash table from archive start
    hash_table_offset: u32,

    /// Offset to block table from archive start
    block_table_offset: u32,

    /// Number of hash table entries (must be power of 2)
    hash_table_count: u32,

    /// Number of block table entries
    block_table_count: u32,
}

#[repr(C)]
struct ArchiveHeaderV2 {
    // ... includes all V1 fields ...

    /// High 32 bits of block table offset for archives > 4GB
    extended_block_table_offset: u64,

    /// High 16 bits of hash table offset
    hash_table_offset_high: u16,

    /// High 16 bits of block table offset
    block_table_offset_high: u16,
}

#[repr(C)]
struct ArchiveHeaderV3 {
    // ... includes all V2 fields ...

    /// 64-bit archive size
    archive_size_64: u64,

    /// Position of BET (Block Extended Table)
    bet_table_offset: u64,

    /// Position of HET (Hash Extended Table)
    het_table_offset: u64,
}

#[repr(C)]
struct ArchiveHeaderV4 {
    // ... includes all V3 fields ...

    /// Compressed size of hash table
    compressed_hash_table_size: u64,

    /// Compressed size of block table
    compressed_block_table_size: u64,

    /// Compressed size of high block table
    compressed_high_block_table_size: u64,

    /// Compressed size of HET table
    compressed_het_table_size: u64,

    /// Compressed size of BET table
    compressed_bet_table_size: u64,

    /// Size of raw data chunks for MD5 calculation
    md5_chunk_size: u32,

    /// MD5 of block table before decryption
    md5_block_table: [u8; 16],

    /// MD5 of hash table before decryption
    md5_hash_table: [u8; 16],

    /// MD5 of high block table
    md5_high_block_table: [u8; 16],

    /// MD5 of BET table before decryption
    md5_bet_table: [u8; 16],

    /// MD5 of HET table before decryption
    md5_het_table: [u8; 16],

    /// MD5 of MPQ header from signature through md5_het_table
    md5_header: [u8; 16],
}
```

### Hash Table Entry

Used for fast file lookups. Each entry is 16 bytes:

```rust
#[repr(C)]
struct HashTableEntry {
    /// First half of filename hash
    name_hash_a: u32,

    /// Second half of filename hash
    name_hash_b: u32,

    /// File locale (0 = default/neutral)
    locale: u16,

    /// Platform (always 0 in practice)
    platform: u16,

    /// Index into block table, or special values:
    /// 0xFFFFFFFF = Empty, never used
    /// 0xFFFFFFFE = Empty, but was deleted
    block_index: u32,
}
```

### Block Table Entry

Contains file location and metadata. Each entry is 16 bytes:

```rust
#[repr(C)]
struct BlockTableEntry {
    /// File offset from archive start
    file_offset: u32,

    /// Compressed file size
    compressed_size: u32,

    /// Uncompressed file size
    uncompressed_size: u32,

    /// File flags (see FileFlags below)
    flags: u32,
}

bitflags! {
    struct FileFlags: u32 {
        /// File is compressed using PKWARE Data Compression Library
        const IMPLODE       = 0x00000100;
        /// File is compressed using combination of algorithms
        const COMPRESS      = 0x00000200;
        /// File is encrypted
        const ENCRYPTED     = 0x00010000;
        /// Encryption key adjusted by file offset
        const KEY_ADJUSTED  = 0x00020000;
        /// File is a patch file
        const PATCH_FILE    = 0x00100000;
        /// File is stored as single unit (not split into sectors)
        const SINGLE_UNIT   = 0x01000000;
        /// File is marked for deletion
        const DELETE_MARKER = 0x02000000;
        /// File has checksums for each sector (ADLER32, not CRC32)
        const SECTOR_CRC    = 0x04000000;
        /// File exists in the archive
        const EXISTS        = 0x80000000;
    }
}
```

### High Block Table

For archives larger than 4GB (v2+), stores upper 16 bits of file offsets:

```rust
type HighBlockTable = Vec<u16>;
```

### HET (Hash Extended Table)

Improved hash table for v3+ archives:

```rust
#[repr(C)]
struct HetTable {
    /// Signature 'HET\x1A'
    signature: [u8; 4],

    /// Version (always 1)
    version: u32,

    /// Size of the contained data
    data_size: u32,

    /// Total size including header
    table_size: u32,

    /// Maximum number of files
    max_file_count: u32,

    /// Size of hash table in bytes
    hash_table_size: u32,

    /// Size of each hash entry in bits
    hash_entry_size: u32,

    /// Total index size in bits
    index_size_total: u32,

    /// Extra index bits
    index_size_extra: u32,

    /// Effective index size in bits
    index_size: u32,

    /// Size of block index array in bytes
    block_index_size: u32,
}
```

### BET (Block Extended Table)

Improved block table for v3+ archives:

```rust
#[repr(C)]
struct BetTable {
    /// Signature 'BET\x1A'
    signature: [u8; 4],

    /// Version (always 1)
    version: u32,

    /// Size of the contained data
    data_size: u32,

    /// Total size including header
    table_size: u32,

    /// Number of files in table
    file_count: u32,

    /// Unknown field (always 0x10)
    unknown: u32,

    /// Size of one table entry in bits
    table_entry_size: u32,

    /// Bit offset of file position in entry
    file_position_bits: u32,

    /// Bit offset of file size in entry
    file_size_bits: u32,

    /// Bit offset of compressed size in entry
    compressed_size_bits: u32,

    /// Bit offset of flag index in entry
    flag_index_bits: u32,

    /// Bit offset of unknown field in entry
    unknown_bits: u32,

    /// Bit count for file position
    file_position_bit_count: u32,

    /// Bit count for file size
    file_size_bit_count: u32,

    /// Bit count for compressed size
    compressed_size_bit_count: u32,

    /// Bit count for flag index
    flag_index_bit_count: u32,

    /// Bit count for unknown field
    unknown_bit_count: u32,

    /// Total size of name hash
    name_hash_size_total: u32,

    /// Extra bits in name hash
    name_hash_size_extra: u32,

    /// Effective name hash size in bits
    name_hash_size: u32,

    /// Size of name hash array in bytes
    name_hash_array_size: u32,

    /// Number of flag entries following
    flag_count: u32,
}
```

## Algorithms

### Hash Calculation

MPQ uses a custom hashing algorithm to convert filenames into hash table entries:

```rust
use wow_mpq::crypto::{hash_string, hash_type};

// Hash types used in MPQ
const HASH_TABLE_OFFSET: u32 = 0;
const HASH_NAME_A: u32 = 1;
const HASH_NAME_B: u32 = 2;
const HASH_FILE_KEY: u32 = 3;

// Example: Calculate hashes for a filename
fn calculate_hashes(filename: &str) -> (u32, u32, u32) {
    let hash_index = hash_string(filename, HASH_TABLE_OFFSET);
    let hash_a = hash_string(filename, HASH_NAME_A);
    let hash_b = hash_string(filename, HASH_NAME_B);

    (hash_index, hash_a, hash_b)
}

// The hash_string function is implemented in wow_mpq and uses
// a pre-computed encryption table for performance
```

### File Search Algorithm

Finding a file in an MPQ using hash and block tables:

```rust
use wow_mpq::{Archive, crypto::{hash_string, hash_type}};

// This is how Archive::find_file() works internally
fn find_file_example(archive: &Archive, filename: &str) -> Option<FileInfo> {
    // The archive provides this method directly:
    archive.find_file(filename).ok().flatten()
}

// For educational purposes, here's the algorithm:
fn find_file_manual(archive: &Archive, filename: &str) -> Option<FileInfo> {
    // Calculate three hash values
    let hash_index = hash_string(filename, 0); // HASH_TABLE_OFFSET
    let hash_a = hash_string(filename, 1);     // HASH_NAME_A
    let hash_b = hash_string(filename, 2);     // HASH_NAME_B

    // Get hash table
    let hash_table = archive.hash_table()?;
    let hash_table_size = hash_table.len();
    let mut index = (hash_index & (hash_table_size as u32 - 1)) as usize;

    // Search the hash table
    loop {
        let entry = &hash_table.entries()[index];

        // Check if we found the file
        if entry.name1 == hash_a && entry.name2 == hash_b {
            if entry.block_index != 0xFFFFFFFF {
                // In real implementation, this would build FileInfo
                // from block table entry
                return Some(FileInfo { /* ... */ });
            }
        }

        // Empty slot - file not found
        if entry.block_index == 0xFFFFFFFF {
            return None;
        }

        // Continue searching
        index = (index + 1) % hash_table_size;
    }
}
```

### HET/BET Search Algorithm (v3+)

For MPQ v3+ archives using HET/BET tables:

```rust
use wow_mpq::{Archive, crypto::jenkins_hash};

// HET/BET tables provide more efficient file lookup for v3+ archives
fn find_file_het_bet_example(archive: &Archive, filename: &str) -> Option<FileInfo> {
    // The archive handles HET/BET lookup automatically
    archive.find_file(filename).ok().flatten()
}

// Educational example of the HET/BET algorithm:
fn het_bet_algorithm_overview(archive: &Archive, filename: &str) {
    // Jenkins hash is used for HET/BET tables
    let name_hash = jenkins_hash(filename.to_uppercase().as_bytes());

    // The HET table stores 8-bit hashes for quick lookup
    // The BET table stores extended file information

    // In practice, the Archive implementation handles all of this
    // complexity internally when you call find_file()
}
```

## Compression

### Compression Methods

Files can be compressed using multiple algorithms, identified by the first byte after decompression:

```rust
use wow_mpq::compression::{decompress, compress, flags};

// Compression flags used in MPQ
mod compression_flags {
    pub const HUFFMAN: u8 = 0x01;       // Huffman (WAVE files)
    pub const ZLIB: u8 = 0x02;          // Deflate/zlib
    pub const IMPLODE: u8 = 0x04;       // PKWare Implode
    pub const PKWARE: u8 = 0x08;        // PKWare DCL
    pub const BZIP2: u8 = 0x10;         // BZip2
    pub const SPARSE: u8 = 0x20;        // Sparse/RLE
    pub const ADPCM_MONO: u8 = 0x40;    // IMA ADPCM mono
    pub const ADPCM_STEREO: u8 = 0x80;  // IMA ADPCM stereo
    pub const LZMA: u8 = 0x12;          // LZMA (not a flag combination)
}

// Example: Decompress file data
fn decompress_example(compressed_data: &[u8], method: u8, expected_size: usize) -> Result<Vec<u8>, wow_mpq::Error> {
    // The decompress function handles all compression types
    // including multi-compression (multiple algorithms applied)
    decompress(compressed_data, method, expected_size)
}

// Example: Compress data
fn compress_example(data: &[u8]) -> Result<Vec<u8>, wow_mpq::Error> {
    // Compress with zlib
    compress(data, flags::ZLIB)
}
```

### Sector-Based Storage

Files larger than the sector size are split into sectors:

```rust
// The wow_mpq library handles sector-based storage internally.
// This example shows the concept:

use wow_mpq::Archive;

fn sector_storage_concept(archive: &Archive) {
    // Get the sector size from the archive header
    let sector_size = archive.header().get_sector_size();

    // Files larger than sector_size are split into multiple sectors
    // Each sector is compressed/encrypted independently

    // When you call archive.read_file(), it automatically:
    // 1. Reads the sector offset table (if multi-sector)
    // 2. Processes each sector (decompress/decrypt)
    // 3. Combines sectors into the complete file

    // For single-unit files (smaller than sector size or with
    // SINGLE_UNIT flag), the entire file is one compressed block
}

// The actual implementation is handled internally by Archive::read_file()
```

## Encryption

### Key Generation

Files can be encrypted using a key derived from the filename:

```rust
use wow_mpq::crypto::{hash_string, decrypt_block, encrypt_block};
use wow_mpq::tables::BlockEntry;

// Key generation constants
const HASH_FILE_KEY: u32 = 3;
const BLOCK_TABLE_KEY: u32 = 0xEC83B3A3;

fn calculate_file_key(filename: &str, block: &BlockEntry) -> u32 {
    // Calculate base key from filename
    let mut key = hash_string(filename, HASH_FILE_KEY);

    // Adjust key if FIX_KEY flag is set
    if block.flags & 0x00020000 != 0 { // KEY_ADJUSTED flag
        key = (key + block.file_pos) ^ block.file_size;
    }

    key
}

// Example: Decrypt data
fn decrypt_example(encrypted_data: &mut [u8], key: u32) {
    // Note: decrypt_block works with u32 arrays, so conversion is needed
    // In practice, the Archive handles this internally

    // For working with raw data, you'd need to convert:
    // 1. Convert u8 array to u32 array
    // 2. Call decrypt_block
    // 3. Convert back to u8 array
}

// Example: Encrypt data
fn encrypt_example(data: &mut [u8], key: u32) {
    // Note: encrypt_block works with u32 arrays, so conversion is needed
    // In practice, the ArchiveBuilder handles this internally
}
```

## Usage Example

```rust
use wow_mpq::{Archive, ArchiveBuilder, FormatVersion};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open an existing MPQ archive
    let mut archive = Archive::open("Data/patch.mpq")?;

    // Search for a file
    if let Some(file_info) = archive.find_file("Interface\\Glues\\Models\\UI_Human\\UI_Human.m2")? {
        println!("File found:");
        println!("  Offset: 0x{:08X}", file_info.file_pos);
        println!("  Size: {} bytes", file_info.file_size);
        println!("  Compressed: {} bytes", file_info.compressed_size);
        println!("  Encrypted: {}", file_info.is_encrypted());

        // Extract the file
        let data = archive.read_file("Interface\\Glues\\Models\\UI_Human\\UI_Human.m2")?;
        std::fs::write("UI_Human.m2", data)?;
    }

    // List all files (requires listfile)
    match archive.list() {
        Ok(entries) => {
            for entry in entries {
                println!("{}: {} bytes", entry.name, entry.size);
            }
        }
        Err(_) => println!("No (listfile) found - cannot enumerate files"),
    }

    // Create a new MPQ archive
    ArchiveBuilder::new()
        .version(FormatVersion::V2)
        .add_file_data(b"Hello, MPQ!".to_vec(), "readme.txt")
        .add_file("path/to/file.dat", "data/file.dat")
        .build("new_archive.mpq")?;

    Ok(())
}
```

## Special Files

MPQ archives may contain special metadata files:

- **`(listfile)`**: Plain text list of all filenames in the archive
- **`(attributes)`**: Extended attributes for files (CRC32, timestamps)
- **`(signature)`**: Digital signature for archive verification
- **`(patch_metadata)`**: Information for incremental patching

## Implementation Notes

### Header Search

When opening an MPQ, search for the header at aligned offsets:

```rust
use wow_mpq::signatures::{MPQ_ARCHIVE, MPQ_USERDATA};

fn find_mpq_header(data: &[u8]) -> Option<usize> {
    // MPQ headers must be aligned to 512-byte boundaries
    const HEADER_ALIGNMENT: usize = 0x200;

    for offset in (0..data.len()).step_by(HEADER_ALIGNMENT) {
        if data.len() >= offset + 4 {
            // Read potential signature
            let mut sig_bytes = [0u8; 4];
            sig_bytes.copy_from_slice(&data[offset..offset + 4]);
            let signature = u32::from_le_bytes(sig_bytes);

            match signature {
                MPQ_USERDATA => {
                    // User data header - read offset to actual MPQ header
                    if data.len() >= offset + 12 {
                        let header_offset = u32::from_le_bytes(
                            data[offset + 8..offset + 12].try_into().unwrap()
                        ) as usize;
                        return Some(offset + header_offset);
                    }
                }
                MPQ_ARCHIVE => {
                    // Found MPQ header directly
                    return Some(offset);
                }
                _ => continue,
            }
        }
    }
    None
}
```

### Table Encryption

Hash and block tables are encrypted and must be decrypted after reading:

```rust
use wow_mpq::crypto::decrypt_block;

// Key constants for table decryption
const HASH_TABLE_KEY: u32 = 0xC3AF3770;
const BLOCK_TABLE_KEY: u32 = 0xEC83B3A3;

// Decrypt hash table with fixed key
fn decrypt_hash_table(table_data: &mut [u32]) {
    decrypt_block(table_data, HASH_TABLE_KEY);
}

// Decrypt block table with fixed key
fn decrypt_block_table(table_data: &mut [u32]) {
    decrypt_block(table_data, BLOCK_TABLE_KEY);
}

// Note: The wow_mpq library handles table decryption automatically
// when loading archives, so you don't need to do this manually.
// The decrypt_block function works with u32 arrays for efficiency.
```

## References

- [MPQ Format Documentation (Zezula)](http://www.zezula.net/en/mpq/mpqformat.html)
- [MPQ Format (wowdev.wiki)](https://wowdev.wiki/MPQ)
- [StormLib Source Code](https://github.com/ladislav-zezula/StormLib)

## Patch Chaining in World of Warcraft

MPQ archives in World of Warcraft use a patch chain system where multiple archives are loaded in a specific order, with higher priority archives overriding files from lower priority ones. This system evolved significantly across WoW versions.

### Loading Order and Priorities

The patch chain system uses numeric priorities where higher numbers override lower ones:

- **0-99**: Base game archives
- **100-999**: Locale-specific base archives
- **1000-1999**: General patch archives
- **2000-2999**: Locale-specific patch archives
- **3000+**: Update archives (Cataclysm and later)

### Version-Specific Implementation

#### WoW 1.12.1 (Vanilla)

- Simple 2-tier system: base archives → patches
- 7 total archives with linear override
- Example: `dbc.MPQ` → `patch.MPQ` → `patch-2.MPQ`

#### WoW 2.4.3 (The Burning Crusade)

- Introduced locale system with 4-tier priority
- Archives: `common.MPQ`, `expansion.MPQ`, locale archives, patches
- Locale patches have highest priority (2000+)

#### WoW 3.3.5a (Wrath of the Lich King)

- Most organized structure with clear expansion separation
- TrinityCore documented exact loading order
- 13 archives: base → expansion → lichking → patches

#### WoW 4.3.4 (Cataclysm)

- Reorganized by content type: `art.MPQ`, `sound.MPQ`, `world.MPQ`
- Introduced `wow-update-#####.MPQ` system
- Added DB2 format alongside DBC

#### WoW 5.4.8 (Mists of Pandaria)

- Peak complexity with 100+ potential archives
- Extensive `wow-update` system (13156-18500)
- Last version before switching to CASC (6.0)

For detailed information about patch chaining implementation and examples, see the [WoW Patch Chain Summary](../../guides/wow-patch-chain-summary.md).

## See Also

- [Working with MPQ Archives Guide](../../guides/mpq-archives.md)
- [WoW Patch Chain Summary](../../guides/wow-patch-chain-summary.md)
- [MPQ API Reference](../../api/mpq.md)
- [MPQ Sector CRC Implementation Details](./mpq-sector-crc.md)
