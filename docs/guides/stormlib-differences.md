# StormLib vs wow-mpq: Technical Differences Guide

This guide outlines the technical differences between StormLib (the reference C++ implementation) and wow-mpq (our Rust implementation). Understanding these differences helps users migrate from StormLib or implement missing features.

## Overview

StormLib has evolved over 20+ years to handle various edge cases, game-specific quirks, and optimizations. Our wow-mpq implementation provides a clean, safe Rust alternative focusing on core MPQ functionality while omitting some specialized features.

## 1. Header Structure and Parsing

### StormLib Approach
- **Unified structure**: Single header struct containing all fields for all versions
- **64-bit fields**: Uses 64-bit fields from the start for forward compatibility
- **Platform handling**: Special logic for different platforms and file types
- **Fields include**:
  - `file_offset_mask` for version-specific offset handling
  - `raw_chunk_size` for MD5 calculation
  - Separate 64-bit size fields for all tables

### wow-mpq Approach
- **Version-specific**: Optional fields added progressively based on version
- **Modular design**: V4 data wrapped in separate struct
- **Simpler parsing**: Focus on standard MPQ format without platform quirks

```rust
// StormLib style (conceptual)
struct MpqHeader {
    signature: u32,
    header_size: u32,
    archive_size: u32,
    format_version: u16,
    sector_size: u16,
    // ... all fields for all versions ...
    raw_chunk_size: u32,         // Always present
    file_offset_mask: u64,       // Calculated based on version
}

// wow-mpq style
struct MpqHeader {
    // Core fields always present
    header_size: u32,
    archive_size: u32,
    format_version: FormatVersion,
    // Version-specific fields as Options
    v4_data: Option<MpqHeaderV4Data>,
}
```

## 2. Archive Opening Workflow

### StormLib: Complex 9-Step Process

1. **Parameter validation** with extensive checks
2. **File size validation** with platform-specific handling
3. **Header search** with game type detection:
   - AVI file detection for Warcraft III cinematics
   - PE header analysis for DLL files
   - Map type detection (Starcraft, Warcraft III, etc.)
4. **Header processing** with compatibility quirks
5. **Cryptography initialization** including storm buffer
6. **Table loading** with defragmentation
7. **File table building** with correlation
8. **Internal files loading** (listfile, attributes)
9. **Finalization** with malformed archive detection

### wow-mpq: Streamlined Process

1. Basic parameter validation
2. Find MPQ header at aligned offsets
3. Parse header based on version
4. Load hash/block tables or HET/BET tables
5. Build file index

**Missing features**:
- Game-specific file type detection
- Automatic defragmentation
- Malformed archive recovery
- Protection scheme bypasses

## 3. HET/BET Table Implementation

### StormLib Features
- **Jenkins hash masking**: Uses `and_mask_64` and `or_mask_64` for optimization
- **Bit-packed storage**: Custom `BitArray` implementation for memory efficiency
- **Detailed bit tracking**: Precise bit positions for all fields
- **Name hash separation**: Stores name hashes separately from file indices

```rust
// StormLib approach
struct HetTable {
    bet_indexes: BitArray,     // Bit-packed file indexes
    name_hashes: Vec<u8>,      // Separate name hash storage
    and_mask_64: u64,          // Hash optimization mask
    or_mask_64: u64,           // Hash optimization mask
    // Detailed bit field tracking
    name_hash_bit_size: u32,
    index_size_total: u32,
    index_size: u32,
}
```

### wow-mpq Implementation
- Simpler structure without optimization masks
- Standard byte arrays instead of bit packing
- Less granular field tracking

## 4. Encryption and Key Detection

### StormLib: Advanced Key Recovery

1. **Filename-based**: Standard key calculation from filename
2. **Sector size detection**: Analyze sector boundaries
3. **Content pattern matching**:
   - WAVE files: `0x46464952` signature
   - EXE files: `0x00905A4D` signature
   - XML files: `0x6D783F3C` signature
4. **Brute force recovery**: Try all 256 possible keys using known patterns

```rust
// StormLib's key detection approach
fn detect_encryption_key(file: &MpqFile) -> Option<u32> {
    // Try filename-based key
    if let Some(key) = try_filename_key(file) {
        return Some(key);
    }
    
    // Try sector size detection
    if let Some(key) = detect_key_by_sector_size(file) {
        return Some(key);
    }
    
    // Try known content patterns
    if let Some(key) = detect_key_by_content(file) {
        return Some(key);
    }
    
    None
}
```

### wow-mpq: Basic Encryption
- Standard filename-based key calculation
- No automatic key detection
- No content-based recovery

## 5. Sector-Based File Reading

### StormLib Optimizations
- **Compression detection**: Analyzes sector size to determine if compressed
- **ADLER32 verification**: Validates each sector's checksum
- **Dynamic loading**: Loads sector offset tables on demand
- **Caching**: LRU cache for frequently accessed sectors

### wow-mpq Implementation
- Basic sector reading and decompression
- No checksum verification
- Simple sequential processing

## 6. Special Behaviors and Edge Cases

### StormLib: Game-Specific Handling

#### Map Type Detection
```rust
enum MapType {
    NotRecognized,
    AviFile,      // Warcraft III cinematics
    Starcraft,    // .scm, .scx extensions
    Warcraft3,    // HM3W signature
    Starcraft2,   // .s2ma, .sc2map extensions
}
```

#### Protection Bypasses
- **BOBA protector**: Handles negative table offsets
- **w3xMaster**: Fixes invalid hi-word positions
- **Integer overflow**: Masks table sizes to prevent overflow
- **Starcraft Beta**: Special handling for specific archive sizes

### wow-mpq: Standard Format Only
- No game-specific detection
- No protection bypass mechanisms
- Assumes well-formed archives

## 7. Memory Management and Performance

### StormLib Optimizations

1. **Custom allocators**: Specialized allocation for large tables
2. **Memory mapping**: `MmapStream` for large read-only archives
3. **Sector caching**: LRU cache with configurable size
4. **Thread safety**: parking_lot mutexes for better performance
5. **Bit packing**: Memory-efficient storage for table data

```rust
// StormLib's optimized approach
pub struct ThreadSafeMpqArchive {
    archive: Arc<ParkingRwLock<MpqArchive>>,
    sector_cache: Arc<Mutex<SectorCache>>,
    mmap_stream: Option<MmapStream>,
}
```

### wow-mpq: Standard Rust Patterns
- Standard `Arc<Mutex<>>` for thread safety
- No specialized memory mapping
- Basic caching strategies
- Standard allocators

## 8. Error Handling and Recovery

### StormLib: Comprehensive Recovery

- **Malformed archive detection**: Marks archives as read-only
- **Automatic recovery**: Attempts to work with damaged archives
- **Detailed error context**: Extensive error information
- **Graceful degradation**: Continues operation with reduced functionality

### wow-mpq: Fail-Fast Approach
- Returns errors on malformed data
- No automatic recovery mechanisms
- Clear error messages
- Predictable behavior

## 9. File Operations

### StormLib: Full Read/Write Support

#### Advanced Write Features
- **Atomic writes**: File writer with Drop trait finalization
- **Free space management**: Finds optimal positions for new files
- **Incremental updates**: Modifies archives without full rebuild
- **Automatic compression**: Chooses best compression method

#### Metadata Support
- MD5 calculation during write
- CRC32 for integrity
- Timestamp preservation
- Extended attributes

### wow-mpq: Read-Focused Design
- Comprehensive read support
- Basic write functionality
- Archive creation support
- Limited in-place modification

## 10. Additional StormLib Features

### Features Not in wow-mpq

1. **Patch metadata**: Special support for WoW patch chains
2. **Compile-time tables**: Storm buffer as const array
3. **Extended attributes**: File timestamps and custom metadata
4. **Sparse compression**: RLE algorithm for sparse data
5. **LZMA compression**: Modern compression support
6. **RSA signatures**: Digital signature verification
7. **Listfile management**: Automatic listfile updates
8. **Weak signature verification**: Support for older signatures

### Implementation Considerations

If you need these features, consider:
- Implementing them as separate crates
- Using FFI to call StormLib for specific operations
- Contributing implementations to wow-mpq

## Migration Guide

### From StormLib to wow-mpq

1. **Check feature requirements**: Ensure wow-mpq supports your needs
2. **Update error handling**: Adapt to Rust's Result type
3. **Adjust for missing features**: Implement workarounds or alternatives
4. **Test thoroughly**: Especially with protected or malformed archives

### Common Migration Issues

```rust
// StormLib style
HANDLE hMpq;
if (!SFileOpenArchive("archive.mpq", 0, 0, &hMpq)) {
    // Handle error
}

// wow-mpq style
let archive = match Archive::open("archive.mpq") {
    Ok(a) => a,
    Err(e) => {
        // Handle error
        return;
    }
};
```

## Performance Comparison

| Feature | StormLib | wow-mpq |
|---------|----------|---------|
| Memory usage | Lower (bit packing) | Higher (standard types) |
| Thread safety | Manual with callbacks | Built-in with Rust |
| Large archives | Memory mapped | Standard I/O |
| Caching | Configurable LRU | Basic |
| Startup time | Slower (more checks) | Faster |

## Recommendations

### Use StormLib when:
- Working with protected archives
- Need game-specific features
- Require key recovery
- Handle malformed archives
- Need all compression methods

### Use wow-mpq when:
- Want memory safety guarantees
- Prefer Rust ecosystem
- Work with standard MPQ files
- Value clean API design
- Need async support (future)

## Contributing Missing Features

If you need StormLib features in wow-mpq:

1. **Open an issue**: Discuss the feature need
2. **Reference StormLib**: Link to relevant StormLib code
3. **Propose design**: Suggest Rust-idiomatic implementation
4. **Submit PR**: Include tests and documentation

## See Also

- [MPQ Format Documentation](../formats/archives/mpq.md)
- [Working with MPQ Archives](./mpq-archives.md)
- [StormLib GitHub Repository](https://github.com/ladislav-zezula/StormLib)
- [wow-mpq API Reference](../api/mpq.md)