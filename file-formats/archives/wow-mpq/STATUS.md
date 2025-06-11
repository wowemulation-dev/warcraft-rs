# wow-mpq - MPQ Feature Implementation Status

**Last Updated:** 2025-06-10

The `wow-mpq` crate provides MPQ archive support:

- **Archive Reading**: Implemented with StormLib compatibility
- **Archive Creation**: Implemented with HET/BET table support
- **Archive Modification**: Implemented with in-place modification support
- **Compression**: All algorithms implemented including ADPCM with overflow protection
- **Cryptography**: Signature verification and generation implemented
- **Advanced Features**: Patch chains implemented, WoW versions 1.12.1 through 5.4.8 supported
- **Testing**: Verified with WoW versions 1.12.1 through 5.4.8

## Detailed Feature Matrix

### 📖 Archive Reading Operations

| Feature | Status | Notes |
|---------|--------|-------|
| **Header Parsing** | ✅ Implemented | All versions v1-v4 |
| **Hash Table Reading** | ✅ Implemented | With encryption/decryption |
| **Block Table Reading** | ✅ Implemented | With encryption/decryption |
| **Hi-Block Table** | ✅ Implemented | For >4GB archives |
| **HET Table Reading** | ✅ Implemented | v3+ with compression |
| **BET Table Reading** | ✅ Implemented | v3+ with compression |
| **File Extraction** | ✅ Implemented | All file types supported |
| **Multi-sector Files** | ✅ Implemented | With sector CRC validation |
| **Single-unit Files** | ✅ Implemented | Single-sector files |
| **File Encryption** | ✅ Implemented | Including FIX_KEY support |
| **Sector CRC Validation** | ✅ Implemented | Validation on WoW files |
| **Special Files** | ✅ Implemented | (listfile), (attributes) |
| **File Enumeration** | ✅ Implemented | Multiple enumeration methods |
| **Archive Info** | ✅ Implemented | Metadata |

### 🔨 Archive Creation Operations

| Feature | Status | Notes |
|---------|--------|-------|
| **ArchiveBuilder API** | ✅ Implemented | Builder pattern |
| **Hash Table Writing** | ✅ Implemented | Auto-sizing, encryption |
| **Block Table Writing** | ✅ Implemented | With encryption |
| **Hi-Block Table** | ✅ Implemented | v2+ archives |
| **HET Table Creation** | ✅ Implemented | v3+ with bit-packing |
| **BET Table Creation** | ✅ Implemented | v3+ with bit widths |
| **Table Compression** | ✅ Implemented | All compression methods |
| **File Addition** | ✅ Implemented | From disk and memory |
| **File Encryption** | ✅ Implemented | During creation |
| **Sector CRC Generation** | ✅ Implemented | Multi-sector and single-unit |
| **Listfile Generation** | ✅ Implemented | Automatic or manual |
| **v1-v3 Archive Creation** | ✅ Implemented | All versions supported |
| **v4 Archive Creation** | ✅ Implemented | All features including MD5 checksums |

### ✏️ Archive Modification Operations

| Feature | Status | Notes |
|---------|--------|-------|
| **Archive Rebuild** | ✅ Implemented | Rebuild with options |
| **In-place File Addition** | ✅ Implemented | MutableArchive::add_file() |
| **File Removal** | ✅ Implemented | MutableArchive::remove_file() |
| **File Renaming** | ✅ Implemented | MutableArchive::rename_file() |
| **Archive Compacting** | ⚠️ Partial | Stub exists, not implemented |
| **File Replacement** | ✅ Implemented | Via add_file with replace_existing |
| **Listfile Auto-Update** | ✅ Implemented | Automatic on file operations |
| **Attributes Update** | ✅ Implemented | Automatic with timestamp/CRC updates |

**Note:** MutableArchive supports file operations with listfile and
attributes updating. Archive compacting is not implemented.

### 🗜️ Compression Algorithms

| Algorithm | Status | Usage | Implementation |
|-----------|--------|-------|----------------|
| **Zlib/Deflate** | ✅ Implemented | Common compression | Native Rust (flate2) |
| **BZip2** | ✅ Implemented | v2+ archives | Native Rust (bzip2) |
| **LZMA** | ✅ Implemented | v3+ archives | Native Rust (lzma-rs) |
| **Sparse/RLE** | ✅ Implemented | v3+ archives | Custom implementation |
| **ADPCM Mono** | ✅ Implemented | Audio compression | Custom implementation |
| **ADPCM Stereo** | ✅ Implemented | Audio compression | Custom implementation |
| **PKWare Implode** | ✅ Implemented | WoW 4.x+ HET/BET metadata | pklib crate |
| **PKWare DCL** | ✅ Implemented | Legacy compression | pklib crate |
| **Huffman** | ✅ Implemented | WAVE file compression | Custom implementation |

**Note:** MPQ compression algorithms support chaining.

### 🔐 Cryptography & Security

| Feature | Status | Notes |
|---------|--------|-------|
| **File Encryption** | ✅ Implemented | All encryption types |
| **File Decryption** | ✅ Implemented | All encryption types |
| **Table Encryption** | ✅ Implemented | Hash/block tables |
| **Key Calculation** | ✅ Implemented | Including FIX_KEY |
| **Hash Algorithms** | ✅ Implemented | All MPQ hash types |
| **Jenkins Hash** | ✅ Implemented | For HET tables |
| **Weak Signature Verification** | ✅ Implemented | 512-bit RSA + MD5, StormLib compatible |
| **Strong Signature Verification** | ✅ Implemented | 2048-bit RSA + SHA-1 |
| **Weak Signature Generation** | ✅ Implemented | Using well-known Blizzard private key |
| **Strong Signature Generation** | ⚠️ Partial | Framework complete, requires private key |

**Note:** Weak signature generation uses the known Blizzard private key.

### 🚀 Performance & I/O

| Feature | Status | Notes |
|---------|--------|-------|
| **Memory-mapped Reading** | ❌ Not Implemented | Standard I/O only |
| **Buffered I/O** | ✅ Implemented | File operations |
| **Zero-copy Operations** | ⚠️ Partial | Where possible |
| **Streaming API** | ❌ Not Implemented | For large files |
| **Progress Callbacks** | ⚠️ Partial | Only in rebuild operations |
| **Memory-mapped Writing** | ❌ Not Implemented | For archive creation |
| **Async I/O** | ❌ Not Implemented | Non-blocking operations |
| **Parallel Compression** | ❌ Not Implemented | Multi-threaded |

### 🔧 Advanced Features

| Feature | Status | Notes |
|---------|--------|-------|
| **Digital Signatures** | ✅ Implemented | Verification only |
| **User Data Headers** | ✅ Implemented | Reading and writing |
| **Special Files** | ✅ Implemented | (listfile), (attributes) |
| **Locale Support** | ⚠️ Partial | Locale handling |
| **Platform Support** | ⚠️ Partial | Field present but vestigial |
| **Patch Archives** | ✅ Implemented | Patch chain support with priority ordering |
| **Protected MPQs** | ❌ Not Implemented | Copy-protected archives |
| **Archive Verification** | ⚠️ Partial | Signature verification only |
| **Unicode Support** | ⚠️ Partial | UTF-8 handling |

### 🧪 Testing & Quality

| Test Category | Status | Notes |
|---------------|--------|-------|
| **Unit Tests** | ✅ Available | Per-module testing |
| **Integration Tests** | ✅ Available | MPQ file testing |
| **Compression Tests** | ✅ Available | Algorithm testing |
| **Security Tests** | ✅ Available | Crypto, CRC, signatures |
| **Benchmark Tests** | ✅ Available | Performance tests |
| **Real MPQ Files** | ✅ Tested | WoW archives tested |
| **Edge Cases** | ✅ Tested | Malformed/corrupted files |
| **Cross-platform** | ✅ Tested | Linux, Windows, macOS |
| **StormLib Comparison** | ✅ Available | C++ comparison tests |

## Limitations

Features not implemented:

### Performance Features

- Memory-mapped I/O
- Streaming APIs for large files
- Async I/O support
- Parallel compression

### Specialized Features

- Protected MPQ support
- Archive compacting
- Strong signature generation (requires private key)

## Features

1. Archive reading and creation
2. Rust implementation with testing
3. StormLib compatibility where implemented
4. Standard algorithms and data structures
5. API documentation with examples
6. Test suite with game files and StormLib comparison
7. Archive rebuild with format upgrades
8. MPQ compression algorithms
9. Patch archive management with priority ordering

## Recent Improvements

1. **WoW Version Compatibility**:
   - Fixed ADPCM decompression overflow for audio files
   - Tested with WoW 1.12.1, 2.4.3, 3.3.5a, 4.3.4, and 5.4.8
   - Files from these versions extract and repack
   - StormLib verification passes

2. **Archive Modification Support**: Implementation of in-place archive modification:
   - ✅ In-place file addition with MutableArchive API
   - ✅ File removal with hash table updates
   - ✅ File renaming with proper rehashing
   - ✅ Automatic listfile updates for all operations
   - ✅ Automatic attributes updates with timestamps and CRCs
   - ✅ Block table reuse for special files to prevent bloat
   - ✅ Proper encryption key generation for modified files

3. **Archive Rebuild**: Added rebuild with options for:
   - Format version upgrades/downgrades
   - Compression method changes
   - File filtering (encrypted, signatures)
   - Progress callbacks
   - Verification against original

4. **Patch Chain Support**: Implemented World of Warcraft patch chain management:
   - Priority-based file resolution
   - Multiple archive handling
   - File override behavior
   - Works with supported WoW versions

5. **Cross-Implementation Compatibility**:
   - StormLib can read all wow-mpq created archives (V1-V4)
   - wow-mpq can read all StormLib created archives (V1-V4)
   - Attributes file format compatibility (both 120-byte and 149-byte formats)
   - HET/BET table generation fixed for V3+ archives
   - Path separator handling (automatic forward slash to backslash conversion)

6. **Blizzard Archive Support**:
   - Handles Blizzard's 28-byte attributes file size deviation
   - Tested with WoW versions 1.12.1, 2.4.3, 3.3.5a, 4.3.4, and 5.4.8
   - Handles non-standard implementations

7. **Compression Support**:
   - LZMA, PKWare Implode/DCL, Huffman added
   - Multi-compression chaining support
   - Algorithm selection

8. **Digital Signature Support**: Signature implementation added:
   - ✅ Weak signature generation (512-bit RSA + MD5)
   - ✅ Strong signature verification (2048-bit RSA + SHA-1)
   - ✅ StormLib-compatible hash calculation
   - ✅ PKCS#1 v1.5 padding support
   - ✅ Private key handling for weak signatures

9. **Documentation**: StormLib differences guide:
   - Technical implementation differences
   - Feature gaps and workarounds
   - Migration guidance
   - Signature module documentation

## Summary

The `wow-mpq` crate implements MPQ archives for World of Warcraft versions 1.12.1 through 5.4.8.

Features:

- Files from WoW versions extract and repack
- In-place add/remove/rename operations
- Compression algorithms including ADPCM
- Bidirectional StormLib compatibility
- Testing against WoW archives and StormLib

Not implemented:

- Memory-mapped I/O
- Streaming APIs
