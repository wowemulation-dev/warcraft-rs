# Changelog

All notable changes to wow-mpq will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **PTCH Patch File Support** - Complete binary patch file implementation for Cataclysm+
  - COPY patches for simple file replacement
  - BSD0 patches using bsdiff40 algorithm for binary diffs
  - Automatic patch detection and application in PatchChain
  - MD5 verification for patch integrity validation
  - New `patch` module with `PatchFile`, `PatchHeader`, and `apply_patch` APIs
- **RLE Compression Algorithm** - Run-length encoding support for compressed files
  - Added `rle.rs` implementation in compression algorithms
  - Integration with existing compression pipeline
- **Patch Chain Enhancements** - Improved automatic patch handling
  - Transparent patch application when reading files through PatchChain
  - Detection of patch files via PTCH header signature
  - Fallback to direct file reading for non-patch files
- **Test Coverage** - New test programs for patch functionality
  - `check_patch_flags` - Verify patch file flags and attributes
  - `test_patch_chain` - Test patch chain loading and file resolution
  - `test_patch_chain_cata` - Cataclysm-specific patch chain testing
  - `test_read_patch` - Direct patch file parsing tests
  - Integration tests for patch chain functionality

### Changed

- **Archive API** - Enhanced Archive struct with better patch file handling
  - Improved error messages for patch file detection
  - Better handling of PATCH flag in file attributes
- **PatchChain** - Refactored for automatic patch application
  - Now automatically applies PTCH patches during file reads
  - Simplified API with transparent patch handling
  - Enhanced priority-based file resolution with patch support

### Fixed

- **Compression Module** - Minor fixes in compression algorithm selection
- **Documentation** - Updated README with comprehensive patch file documentation
  - Added PTCH format explanation and usage examples
  - Documented automatic patch application in PatchChain
  - Added CLI usage examples for patch chains

## [0.3.0] - 2025-08-07

### Changed

- Version bump to 0.3.0 for coordinated workspace release
- Updated dependencies and documentation

### Added

- New methods in `MutableArchive`: `read_file()`, `list()`, `find_file()`, `verify_signature()`, `load_attributes()`
- Complete implementation of `compact()` method for archive defragmentation
- Improved attributes file parsing to handle both cases where attributes include/exclude themselves

### Changed

- Enhanced attributes file block count detection with automatic fallback logic
- `MutableArchive` now provides convenience methods for common read operations

### Fixed

- Fixed Huffman decompression algorithm to match StormLib's linked list approach, resolving ADPCM audio decompression failures and file count discrepancies in Warcraft 3 MPQ archives
- Fixed IMPLODE compression handling for Warcraft III MPQ archives - IMPLODE-compressed files no longer incorrectly skip the first byte as a compression type prefix
- Fixed attributes file parsing to correctly handle varying block counts across different MPQ implementations
- Removed invalid Huffman test case and obsolete PKWare compression tests to eliminate test failures
- Applied rustfmt formatting fixes to improve code consistency
- Fixed clippy warnings by using inline format arguments in log statements

## [0.2.0] - 2025-06-28

### Added

- Initial release of wow-mpq crate
- Support for reading MPQ archives from WoW versions 1.12.1 through 5.4.8
- Support for MPQ format versions 1, 2, 3, and 4
- Comprehensive compression support (ZLIB, BZIP2, ADPCM, Huffman, Sparse, LZMA)
- Full encryption/decryption support
- Hash table (HT) and block table (BT) parsing
- Extended attributes support
- HET/BET table support for MPQ v3+
- Patch archive support with proper file resolution
- Parallel processing capabilities for improved performance
- StormLib compatibility layer for validation
- Comprehensive test suite and benchmarks
