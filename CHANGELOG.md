# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **wow-mpq**: Complete parallel processing support for multi-threaded archive operations
- **wow-mpq**: `ParallelArchive` struct for thread-safe concurrent file extraction from single archives
- **wow-mpq**: Four parallel processing functions: `extract_from_multiple_archives()`, `search_in_multiple_archives()`, `process_archives_parallel()`, `validate_archives_parallel()`
- **wow-mpq**: Parallel patch chain loading with `from_archives_parallel()` and `add_archives_parallel()` methods
- **wow-mpq**: Thread-safe file handle cloning strategy for concurrent archive access
- **wow-mpq**: Comprehensive parallel processing benchmarks showing up to 6x performance improvements
- **wow-mpq**: Thread safety validation tests with 11 test cases covering concurrent operations
- **wow-mpq**: Buffer pre-allocation optimizations for sector reading operations
- **wow-mpq**: Hash table mask caching for improved file lookup performance
- **wow-mpq**: Multi-compression decompression buffer pre-allocation
- **wow-mpq**: Public `list_files()` and `read_file_with_new_handle()` methods for parallel access
- **wow-mpq**: Rayon integration for CPU-optimal work distribution
- **wow-mpq**: Complete documentation for parallel processing features
- **wow-mpq**: Single archive parallel processing examples and benchmarks
- **warcraft-rs CLI**: Parallel processing enabled by default for `extract` and `validate` commands
- **warcraft-rs CLI**: `--threads N` parameter for customizing parallel thread count
- **warcraft-rs CLI**: Automatic CPU core detection for optimal default threading
- **warcraft-rs CLI**: Added `--patch` parameter to `mpq extract` command for patch chain support
- **warcraft-rs CLI**: Multiple patch archives can be specified with repeated `--patch` flags
- **warcraft-rs CLI**: Patch archives are applied in order with increasing priority (100, 200, 300, etc.)
- **warcraft-rs CLI**: Shows patch chain info after extraction including which archive each file came from

### Changed

- **wow-mpq**: Made parallel processing the default behavior for all CLI operations
- **wow-mpq**: Simplified CLI interface by removing sequential processing options
- **wow-mpq**: Enhanced performance through strategic buffer optimizations
- **wow-mpq**: Improved thread safety architecture for concurrent operations
- **Documentation**: Fixed all API discrepancies between documentation and actual implementation
- **Documentation**: Updated code examples to use correct method names (`list()` vs `list_all()` clarification)
- **Documentation**: Corrected import statements from `wow_dbc` to `wow_cdbc` throughout all guides
- **Documentation**: Fixed M2 model loading examples to use actual `parse()` method instead of non-existent `load()` method
- **Documentation**: Updated path separator usage to consistently use backslashes with auto-conversion explanations
- **Documentation**: All code examples now compile correctly and match actual API
- **Project-wide**: Comprehensive reorganization of tests and examples for better maintainability
- **wow-mpq**: Consolidated examples from 50+ to 15 focused demonstrations
- **wow-mpq**: Enhanced `create_archive.rs` example with comprehensive functionality (basic creation, compression, encryption, attributes, version comparison)
- **wow-mpq**: Merged 7 patch chain examples into single comprehensive `wow_patch_chains.rs` 
- **wow-mpq**: Moved test-like examples (e.g., `test_*_files_comprehensive.rs`) to proper test directories
- **All crates**: Standardized test organization following wow-mpq structure:
  - `component/` - Unit tests for individual components
  - `integration/` - Integration tests for complete workflows
  - `scenarios/` - Real-world usage scenarios
  - `compliance/` - Compatibility and compliance tests
- **All crates**: Updated test README files to reflect new organization
- **Examples**: Improved documentation with clear categories (Beginners, Modding, Advanced)
- **deny.toml**: Added MPL-2.0 and bzip2-1.0.6 to allowed licenses for cbindgen and libbz2-rs-sys dependencies
- **deny.toml**: Added exceptions for RUSTSEC-2023-0071 (rsa timing attack - local use only) and windows-link yanked version

### Fixed

- **wow-mpq**: Fixed compression method detection for SINGLE_UNIT files (they DO have compression byte prefixes)
- **wow-mpq**: Resolved buffer underrun issues in sparse compression decompression
- **wow-mpq**: Fixed ZLIB decompression failures for specific file types
- **wow-mpq**: Fixed handling of PATCH flag files in update archives - now returns proper error message explaining patch files cannot be read directly
- **wow-mpq**: Fixed `simple_list` example to use `list()` instead of `list_all()` to show proper filenames from listfile
- **Documentation**: Fixed compilation errors in all documentation code examples
- **Documentation**: Resolved API method name mismatches throughout guides
- **Documentation**: Fixed incorrect crate names in import statements

### Performance

- **wow-mpq**: Up to 6x performance improvement for multi-archive operations through parallel processing
- **wow-mpq**: Optimized sector reading with buffer pre-allocation strategies
- **wow-mpq**: Enhanced file lookup performance through hash table mask caching
- **wow-mpq**: Improved memory usage patterns in decompression workflows
- **wow-mpq**: CPU-optimal work distribution using rayon thread pools

### Removed

- **warcraft-rs CLI**: Removed `--parallel` and `--sequential` flags (parallel is now default)
- **warcraft-rs CLI**: Removed `--batch-size` option (automatically optimized)
- **wow-mpq**: Removed redundant examples: `patch_chain_demo.rs`, `wotlk_patch_chain_demo.rs`, `tbc_patch_chain_demo.rs`, `cata_patch_chain_demo.rs`, `mop_patch_chain_demo.rs`, `patch_chain_dbc_demo.rs`
- **wow-mpq**: Removed specialized examples: `create_archive_with_attributes.rs`, `create_comparison_archives.rs`, `create_encrypted_archive.rs`, `create_compressed_tables.rs`
- **wow-mpq**: Removed development-focused examples: `analyze_blizzard_attributes.rs`, `analyze_v4_header.rs`, `generate_test_data.rs`, `comprehensive_archive_verification.rs`, etc.

## [0.1.0](https://github.com/wowemulation-dev/warcraft-rs/releases/tag/v0.1.0) - 2025-06-13

### Added

- **wow-mpq**: Complete archive modification API with `MutableArchive` for adding, removing, and renaming files
- **wow-mpq**: Automatic listfile and attributes updates during archive modifications
- **wow-mpq**: Full StormLib bidirectional compatibility - archives created/modified by wow-mpq are readable by StormLib and vice versa
- **wow-mpq**: Compatibility with WoW versions 1.12.1 through 5.4.8
- **wow-mpq**: Support for official World of Warcraft MPQ archives (versions 1.12.1 - 5.4.8)
- **wow-mpq**: Support for all MPQ format versions (V1, V2, V3 with HET/BET, V4 with advanced HET/BET)
- **wow-mpq**: Portable WoW data discovery system using environment variables and common paths
- **wow-mpq**: `test-utils` feature for examples requiring WoW game data
- **wow-mpq**: Automatic forward slash to backslash path separator conversion for cross-platform compatibility
- **wow-mpq**: Graceful handling of Blizzard's 28-byte attributes file size deviation
- **wow-mpq**: Fixed HET/BET table generation for V3+ archives with proper attributes file indexing
- **wow-mpq**: Fixed V4 archive creation by correcting hi-block table size calculation
- **wow-mpq**: Changed attributes generation from CRC32-only to full format (CRC32+MD5+timestamp) for StormLib compatibility
- **wow-mpq**: Added ADPCM audio compression support with overflow protection
- **warcraft-rs CLI**: Added path separator conversion in `mpq extract` subcommand for proper cross-platform file extraction
- **wow-wdl**: Initial implementation of WDL (World Data Low-resolution) format support
- **warcraft-rs CLI**: Added `wdl` subcommand with validate, convert, and info commands for WDL file manipulation
- **wow-wdt**: WDT (World Data Table) format support with parsing for WoW versions
- **warcraft-rs CLI**: Added `wdt` subcommand with info, validate, convert, and tiles commands for WDT file manipulation
- **wow-adt**: Integrated comprehensive ADT (terrain) file support
- **wow-adt**: Full parsing support for all ADT chunk types including terrain, textures, water, and object placement
- **wow-adt**: Version conversion support between Classic, TBC, WotLK, and Cataclysm formats
- **wow-adt**: Split file support for Cataclysm+ ADT files (_tex0,_obj0, etc.)
- **wow-adt**: Comprehensive validation with multiple strictness levels
- **wow-adt**: Tree visualization support for hierarchical chunk structure display
- **wow-adt**: Added comprehensive examples: parse_adt, validate_adt, and version_info
- **wow-adt**: Fixed MH2O water chunk parsing for incomplete water data in original Blizzard files
- **wow-adt**: Fixed MFBO chunk handling for variable sizes between expansions (8 bytes in TBC, 36 bytes in Cataclysm+)
- **warcraft-rs CLI**: Added complete ADT command suite: `info`, `validate`, `convert`, and `tree`
- **warcraft-rs CLI**: ADT commands now use expansion names (classic, tbc, wotlk, cataclysm) instead of version numbers
- **warcraft-rs CLI**: Added `tree` subcommand for MPQ, WDT, WDL, and ADT formats to visualize file structure hierarchically
- **warcraft-rs**: Added comprehensive tree visualization utilities for rendering file format structures with emoji icons and color support
- **All crates**: Replaced byteorder crate usage with native Rust byte order functions
- **wow-wmo**: Integrated complete WMO (World Map Object) format support from external crate
- **wow-wmo**: Full parsing and writing support for WMO root and group files
- **wow-wmo**: Support for all WMO versions from Classic (v17) through The War Within (v27)
- **wow-wmo**: Version conversion capabilities for upgrading/downgrading WMO files between expansions
- **wow-wmo**: Comprehensive validation with field-level and structural checks
- **wow-wmo**: Tree visualization support for hierarchical structure display
- **wow-wmo**: Builder API for creating WMO files programmatically
- **wow-wmo**: Fixed integer overflow in group name parsing causing crashes with certain WMO files
- **wow-wmo**: Fixed header size mismatch (60 vs 64 bytes) causing chunk misalignment
- **wow-wmo**: Fixed texture validation to handle special marker values (0xFF000000+)
- **wow-wmo**: Fixed light type parsing to handle unknown types gracefully
- **wow-wmo**: Fixed doodad structure size to always use 40 bytes for proper round-trip conversion
- **warcraft-rs CLI**: Added complete WMO command suite: `info`, `validate`, `convert`, `tree`, `edit`, and `build`
- **wow-blp**: Full BLP texture format support migrated from image-blp crate
- **wow-blp**: Support for all BLP versions (BLP0, BLP1, BLP2) from Warcraft III Beta through WoW 5.4.8
- **wow-blp**: All compression formats: JPEG, RAW1 (palettized), RAW3 (uncompressed), DXT1/3/5
- **wow-blp**: Complete mipmap support for both internal and external mipmaps
- **wow-blp**: Bidirectional conversion between BLP and standard image formats
- **wow-blp**: Alpha channel support with 0, 1, 4, and 8-bit depths
- **wow-blp**: Examples for loading and saving BLP files
- **warcraft-rs CLI**: Added complete BLP command suite: `convert`, `info`, and `validate`
- **warcraft-rs CLI**: BLP convert supports all encoding options with mipmap generation and DXT compression quality control
- **wow-blp**: Replaced nom parser with native Rust implementation for binary parsing
- wow-mpq: allow to extract files by generic index names when no listfile exists

### Fixed

- **wow-mpq**: Fixed critical sector reading bug that was truncating large files in archives
- **wow-mpq**: Fixed archive modification to properly update both listfile and attributes
- **wow-mpq**: Fixed sector offset validation that was causing false positive truncation errors
- **wow-mpq**: Fixed V3 archive compatibility issues where StormLib couldn't read attributes files
- **wow-mpq**: Fixed V4 malloc crash by properly checking if hi-block table is needed before setting size
- **wow-mpq**: Fixed HET table creation to properly index attributes files
- **wow-mpq**: Fixed ADPCM decompression overflow when bit shift value exceeds 31
- **wow-mpq**: Fixed unused imports and casts that were causing compiler warnings
- **wow-mpq**: Fixed test expecting wrong file count (now correctly includes attributes file)

### Changed

- **wow-mpq**: Attributes files now use StormLib-compatible 149-byte format instead of 24-byte format
- **Documentation**: Updated documentation to reflect StormLib compatibility
- **Documentation**: Added notes about Blizzard archive compatibility and common warnings

### Removed

- **wow-mpq**: Removed redundant `create_het_table()` method that was replaced by `create_het_table_with_hash_table()`
