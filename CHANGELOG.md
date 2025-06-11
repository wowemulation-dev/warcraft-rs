# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
