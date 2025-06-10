# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **wow-mpq**: Complete archive modification API with `MutableArchive` for adding, removing, and renaming files
- **wow-mpq**: Automatic listfile and attributes updates during archive modifications
- **wow-mpq**: Full StormLib bidirectional compatibility - archives created/modified by wow-mpq are readable by StormLib and vice versa
- **wow-mpq**: Achieved 100% compatibility with all WoW versions (1.12.1 through 5.4.8)
- **wow-mpq**: Full support for all official World of Warcraft MPQ archives (versions 1.12.1 - 5.4.8)
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
- **wow-wdt**: Complete WDT (World Data Table) format support with 100% parsing success rate across all WoW versions
- **warcraft-rs CLI**: Added `wdt` subcommand with info, validate, convert, and tiles commands for WDT file manipulation
- **warcraft-rs CLI**: Added `tree` subcommand for MPQ, WDT, and WDL formats to visualize file structure hierarchically
- **warcraft-rs**: Added comprehensive tree visualization utilities for rendering file format structures with emoji icons and color support

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
- **Documentation**: Comprehensively updated all documentation to reflect 100% StormLib compatibility
- **Documentation**: Added notes about Blizzard archive compatibility and common warnings

### Removed

- **wow-mpq**: Removed redundant `create_het_table()` method that was replaced by `create_het_table_with_hash_table()`
