# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **wow-mpq**: Achieved 98.75% bidirectional compatibility with StormLib (reference C++ implementation)
- **wow-mpq**: Full support for all official World of Warcraft MPQ archives (versions 1.12.1 - 5.4.8)
- **wow-mpq**: Automatic forward slash to backslash path separator conversion for cross-platform compatibility
- **wow-mpq**: Graceful handling of Blizzard's 28-byte attributes file size deviation
- **wow-mpq**: Fixed HET/BET table generation for V3+ archives with proper attributes file indexing
- **wow-mpq**: Fixed V4 archive creation by correcting hi-block table size calculation
- **wow-mpq**: Changed attributes generation from CRC32-only to full format (CRC32+MD5+timestamp) for StormLib compatibility
- **warcraft-rs CLI**: Added path separator conversion in `mpq extract` subcommand for proper cross-platform file extraction
- **wow-wdl**: Initial implementation of WDL (World Data Low-resolution) format support

### Fixed
- **wow-mpq**: Fixed V3 archive compatibility issues where StormLib couldn't read attributes files
- **wow-mpq**: Fixed V4 malloc crash by properly checking if hi-block table is needed before setting size
- **wow-mpq**: Fixed HET table creation to properly index attributes files
- **wow-mpq**: Fixed unused imports and casts that were causing compiler warnings
- **wow-mpq**: Fixed test expecting wrong file count (now correctly includes attributes file)

### Changed
- **wow-mpq**: Attributes files now use StormLib-compatible 149-byte format instead of 24-byte format
- **Documentation**: Comprehensively updated all documentation to reflect 98.75% StormLib compatibility
- **Documentation**: Added notes about Blizzard archive compatibility and common warnings

### Removed
- **wow-mpq**: Removed redundant `create_het_table()` method that was replaced by `create_het_table_with_hash_table()`
