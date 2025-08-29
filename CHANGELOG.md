# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2025-08-29

### Added

- **M2 Format**: Complete chunked format implementation with 25 chunks (Legion+ support)
  - File reference chunks: SFID, AFID, TXID, PFID, SKID, BFID
  - Particle system chunks: PABC, PADC, PSBC, PEDC, PCOL, PFDC
  - Rendering enhancement chunks: TXAC, EDGF, NERF, DETL, RPID, GPID, DBOC
  - Animation system chunks: AFRA, DPIV
  - Export processing chunks: EXPT
- **M2 Format**: Comprehensive chunk validation and cross-reference checking
- **M2 Format**: Physics file (.phys) basic parsing support via PFID references
- **Code Quality**: Enhanced clippy linting with stricter rules (all, pedantic, nursery, cargo groups)
- **Testing**: Comprehensive QA script adapted from cascette-rs for better code quality assurance
- **Security**: Improved dependency security audit configuration with documented exceptions

### Fixed

- **M2 Parsing**: Fixed redundant guard expressions in chunk pattern matching (29 instances)
- **M2 Parsing**: Resolved needless borrows in iterator usage throughout codebase
- **M2 Parsing**: Fixed is_some_and pattern usage for cleaner conditional logic
- **M2 Parsing**: Corrected iterator patterns in rendering enhancement processing
- **M2 Parsing**: Fixed stream position handling in chunk infrastructure
- **CI/CD**: Skip CLI tests requiring MPQ test files in CI environment
- **Dependencies**: Updated slab from 0.4.10 to 0.4.11 to fix security vulnerability
- **Performance**: Optimized MPQ bulk extraction performance
- **Code Quality**: Removed unused imports and variables across test modules

### Changed

- **M2 Format**: Achieved 100% M2 specification compliance with chunked format support
- **Documentation**: Updated M2 format documentation to reflect comprehensive implementation
- **Code Style**: Applied rustfmt formatting consistently across entire codebase
- **Error Handling**: Enhanced error reporting with detailed chunk validation messages
- **Testing**: Expanded test coverage to 135+ unit and integration tests
- **Validation**: Improved cross-chunk reference validation and consistency checking

## [0.3.1] - 2025-08-12

### Fixed

- **wow-mpq**: Replaced custom ReadLittleEndian trait with standard
  byteorder crate across 50+ locations
- **wow-mpq**: Added generic error conversion helpers for compression
  algorithms
- **wow-mpq**: Standardized error handling patterns in compression
  module
- **wow-blp**: Extracted duplicate bounds checking logic into reusable
  module
- **wow-wmo**: Simplified error handling patterns in chunk reading code
- **wow-m2**: Fixed ribbon emitter parsing for Cataclysm/MoP using numeric
  version comparison
- **wow-mpq**: Fixed HET table creation to handle attributes files correctly
- **wow-mpq**: Fixed sector offset validation preventing false positive
  truncation errors

### Added

- **wow-blp**: Support for alpha_type=7 for TBC+ enhanced alpha blending
  compatibility
- **wow-m2**: Empirically verified version numbers (Classic=256, TBC=260,
  WotLK=264, Cata/MoP=272)
- **wow-wmo**: MCVP chunk support for Cataclysm+ transport collision
  volumes

### Changed (0.3.0)

- Applied rustfmt formatting fixes across all crates
- Removed 29 development test files (5,800+ lines) for cleaner codebase
- Refactored code for idiomatic Rust patterns

### Performance

- Achieved 99.5% parser success rate across 200+ test files

## [0.3.0] - 2025-08-07

### Breaking Changes

- **wow-adt**: MFBO chunk structure changed from 8 bytes to 36 bytes
  (2 planes × 9 int16 coordinates)
- **wow-wdt**: MWMO chunk handling changed for Cataclysm+ compatibility
  (only WMO-only maps include MWMO)
- **wow-adt**: Version detection API enhanced with chunk-based detection methods

### Added (0.3.0)

- **wow-adt**: Complete WoW version support (Vanilla through Mists of
  Pandaria) with automatic detection
- **wow-adt**: Split ADT file support for Cataclysm+ (`_tex0`, `_obj0`,
  `_obj1`, `_lod` files) with merge functionality
- **wow-adt**: MAMP chunk parser for 4-byte texture amplifier values
  (Cataclysm+)
- **wow-adt**: MTXP chunk parser for texture parameters with 16-byte
  entries (MoP+)
- **wow-wdt**: Enhanced version detection across all WoW expansions
- **wow-wdt**: Map type detection distinguishing terrain maps from WMO-only maps
- **wow-wdl**: Enhanced chunk support for all documented chunks (MAOF, MAOH,
  MAHO, MWID, MWMO, MODF, ML)
- **wow-mpq**: MutableArchive methods: read_file(), list(), find_file(),
  verify_signature(), load_attributes()
- **wow-mpq**: Complete compact() method implementation for archive
  defragmentation
- **wow-mpq**: Attributes file parsing handles both self-inclusive and
  self-exclusive cases
- **wow-m2**: WotLK M2 model and skin format support
- **wow-m2**: Texture filename parsing functionality
- **wow-m2**: Old skin format support
- **warcraft-rs**: cargo-deny configuration for dependency security scanning

### Fixed (0.3.0)

- **wow-adt**: MFBO flight boundaries now use correct structure matching
  TrinityCore server
- **wow-wdt**: MWMO chunk writing uses version-aware logic for Cataclysm+
  compatibility
- **wow-mpq**: Huffman decompression algorithm matches StormLib linked
  list approach
- **wow-mpq**: IMPLODE compression handling for Warcraft III MPQ archives
- **wow-mpq**: Attributes file parsing handles varying block counts
  across implementations
- **wow-m2**: BLP texture parsing with correct header field order and data types

### Changed

- **wow-mpq**: Enhanced attributes file block count detection with
  automatic fallback logic
- **wow-m2**: Replaced custom BLP implementation with wow-blp crate dependency
- **wow-m2**: BlpTexture now re-exports wow_blp::BlpImage for compatibility

### Removed

- **wow-mpq**: Invalid Huffman test case and obsolete PKWare compression tests
- **wow-m2**: Custom BLP parsing implementation

## [0.2.0] - 2025-06-28

### Added (0.2.0)

- **wow-mpq**: Complete parallel processing support with ParallelArchive struct
- **wow-mpq**: Multi-threaded functions: extract_from_multiple_archives(),
  search_in_multiple_archives(), process_archives_parallel(),
  validate_archives_parallel()
- **wow-mpq**: Thread-safe file handle cloning strategy for concurrent access
- **wow-mpq**: Parallel patch chain loading with from_archives_parallel()
  and add_archives_parallel()
- **wow-mpq**: Buffer pre-allocation optimizations for sector reading
- **wow-mpq**: Hash table mask caching for improved file lookup
- **wow-mpq**: Public list_files() and read_file_with_new_handle() methods
- **wow-mpq**: Rayon integration for CPU-optimal work distribution
- **wow-mpq**: SQLite database support for persistent filename hash storage
  and resolution
- **wow-mpq**: Database import functionality supporting listfiles, MPQ
  archives, and directory scanning
- **wow-mpq**: Automatic filename resolution through database lookup
  during archive operations
- **storm-ffi**: Complete archive modification support through C FFI
- **storm-ffi**: File operations: add, remove, rename with SFileAddFile,
  SFileRemoveFile, SFileRenameFile
- **storm-ffi**: Archive operations: create, flush, compact with
  SFileCreateArchive, SFileFlushArchive, SFileCompactArchive
- **storm-ffi**: File finding functionality with
  SFileFindFirstFile/NextFile/Close
- **storm-ffi**: File and archive attributes API support
- **warcraft-rs**: CLI with mpq subcommands: list, extract, info, verify
- **warcraft-rs**: mpq db subcommand with status, import, analyze, lookup,
  export, list operations
- **warcraft-rs**: Parallel processing enabled by default with --threads
  parameter
- **warcraft-rs**: --patch parameter for patch chain support in extract command
- **warcraft-rs**: BLP commands: convert, info, validate with mipmap
  generation and DXT compression
- **warcraft-rs**: ADT commands: info, validate, convert, tree with
  expansion name support
- **warcraft-rs**: WDT commands: info, validate, convert, tiles
- **warcraft-rs**: WDL commands: validate, convert, info
- **warcraft-rs**: Tree visualization for all formats with emoji icons
  and color support
- **wow-blp**: Complete BLP texture format support (BLP0, BLP1, BLP2)
- **wow-blp**: All compression formats: JPEG, RAW1 (palettized), RAW3, DXT1/3/5
- **wow-blp**: Mipmap support for internal and external mipmaps
- **wow-blp**: Bidirectional conversion between BLP and standard image formats
- **wow-blp**: Alpha channel support with 0, 1, 4, and 8-bit depths
- **wow-m2**: M2 model format parsing with header and version detection
- **wow-m2**: Global sequences, texture definitions, bone hierarchy parsing
- **wow-m2**: Vertex and triangle data access, skin file support
- **wow-m2**: Animation sequence data, material and render flag support
- **wow-wmo**: WMO root and group file parsing and writing support
- **wow-wmo**: Version support from Classic (v17) through The War Within (v27)
- **wow-wmo**: Version conversion capabilities between expansions
- **wow-wmo**: Builder API for programmatic WMO file creation
- **wow-adt**: ADT terrain file parsing for all chunk types
- **wow-adt**: Height maps, texture layers, doodad and WMO placement data
- **wow-adt**: Liquid information, vertex shading, shadow maps, alpha maps
- **wow-adt**: Version conversion between Classic, TBC, WotLK, and
  Cataclysm formats
- **wow-wdt**: WDT file support with MPHD header and MAIN chunk parsing
- **wow-wdt**: MAID chunk support for file data IDs (Legion+)
- **wow-wdt**: WMO-only world support with map metadata
- **wow-wdl**: WDL file support with MAOF, MAOH, MAHO chunk parsing
- **wow-wdl**: Low-resolution height maps and Mare ID mapping
- **wow-cdbc**: Client database (DBC) file parsing with DBD schema support
- **wow-cdbc**: Localized string support and row-based data access

### Fixed (0.2.0)

- **wow-mpq**: Critical sector reading bug truncating large files
- **wow-mpq**: Archive modification to properly update listfile and attributes
- **wow-mpq**: V3 archive compatibility issues with StormLib attributes
  file reading
- **wow-mpq**: V4 malloc crash by checking hi-block table necessity
- **wow-mpq**: ADPCM decompression overflow when bit shift value exceeds 31
- **wow-mpq**: SINGLE_UNIT file compression method detection
- **wow-mpq**: ZLIB decompression failures for specific file types
- **wow-mpq**: PATCH flag file handling with proper error messages
- **wow-wmo**: Integer overflow in group name parsing
- **wow-wmo**: Header size mismatch (60 vs 64 bytes) causing chunk misalignment
- **wow-wmo**: Texture validation to handle special marker values
- **wow-wmo**: Light type parsing to handle unknown types gracefully
- **wow-adt**: MH2O water chunk parsing for incomplete water data
- **wow-adt**: MFBO chunk handling for variable sizes between expansions

### Changed (0.2.0)

- **wow-mpq**: Parallel processing as default behavior for all CLI operations
- **wow-mpq**: Enhanced thread safety architecture for concurrent operations
- **wow-mpq**: Attributes files use StormLib-compatible 149-byte format
- **storm-ffi**: Renamed from storm to storm-ffi while retaining
  libstorm library name
- **storm-ffi**: Archive handles support both read-only and mutable operations
- **Project-wide**: Comprehensive test reorganization with component,
  integration, scenarios, compliance directories
- **Project-wide**: Consolidated examples from 50+ to 15 focused demonstrations
- **All crates**: Replaced byteorder crate with native Rust byte order functions
- **Documentation**: Fixed API discrepancies between documentation and
  implementation
- **Documentation**: Updated all code examples to compile correctly

### Performance (0.2.0)

- Up to 6x performance improvement for multi-archive operations through
  parallel processing
- Optimized sector reading with buffer pre-allocation strategies
- Enhanced file lookup performance through hash table mask caching
- CPU-optimal work distribution using rayon thread pools

### Removed (0.2.0)

- **wow-mpq**: Redundant create_het_table() method replaced by
  create_het_table_with_hash_table()
- **warcraft-rs**: --parallel and --sequential flags (parallel now default)
- **warcraft-rs**: --batch-size option (automatically optimized)
- **wow-mpq**: Redundant examples consolidated into comprehensive demonstrations

## [0.1.0] - 2025-06-13

### Added (0.1.0)

- **wow-mpq**: Complete archive modification API with MutableArchive
- **wow-mpq**: Automatic listfile and attributes updates during modifications
- **wow-mpq**: StormLib bidirectional compatibility for
  created/modified archives
- **wow-mpq**: Support for WoW versions 1.12.1 through 5.4.8
- **wow-mpq**: Support for MPQ format versions (V1, V2, V3 with HET/BET,
  V4 with advanced HET/BET)
- **wow-mpq**: Portable WoW data discovery using environment variables
  and common paths
- **wow-mpq**: test-utils feature for examples requiring WoW game data
- **wow-mpq**: Cross-platform path separator conversion (forward slash
  to backslash)
- **wow-mpq**: Graceful handling of Blizzard's 28-byte attributes file deviation
- **wow-mpq**: HET/BET table generation for V3+ archives with attributes
  file indexing
- **wow-mpq**: V4 archive creation with corrected hi-block table size
  calculation
- **wow-mpq**: ADPCM audio compression support with overflow protection
- **wow-mpq**: Support for all compression formats (ZLIB, BZIP2, ADPCM,
  Huffman, Sparse, LZMA)
- **wow-mpq**: Encryption/decryption support with hash and block table parsing
- **wow-mpq**: Extended attributes and HET/BET table support
- **wow-mpq**: Patch archive support with proper file resolution
- **wow-mpq**: Generic index file extraction when no listfile exists
- Initial workspace structure for World of Warcraft file format parsing
- Rust 2024 edition support with MSRV 1.86
- Comprehensive test organization (unit, integration, compliance, scenarios)

### Fixed (0.1.0)

- **wow-mpq**: V3 archive compatibility with StormLib attributes file reading
- **wow-mpq**: V4 malloc crash by proper hi-block table necessity checking
- **wow-mpq**: HET table creation to properly index attributes files
- **wow-mpq**: ADPCM decompression overflow protection
- **wow-mpq**: Sector reading validation preventing false positive
  truncation errors

### Changed (0.1.0)

- **wow-mpq**: Attributes files use full StormLib-compatible format
  (CRC32+MD5+timestamp) instead of CRC32-only

[0.3.1]: https://github.com/wowemulation-dev/warcraft-rs/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/wowemulation-dev/warcraft-rs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/wowemulation-dev/warcraft-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/wowemulation-dev/warcraft-rs/releases/tag/v0.1.0
