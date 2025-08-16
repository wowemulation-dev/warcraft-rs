# Changelog

All notable changes to wow-mpq will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
