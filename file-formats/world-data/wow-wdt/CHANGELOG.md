# Changelog

All notable changes to wow-wdt will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-08-07

### Breaking Changes

- **Conditional MWMO chunk handling**: Fixed for Cataclysm+ compatibility
  - Pre-Cataclysm: All maps have MWMO chunks (even if empty for terrain-only maps)
  - Cataclysm+: Only WMO-only maps have MWMO chunks, terrain maps don't include them
  - This matches TrinityCore server behavior and actual client files

### Added  

- **Enhanced version detection**: Automatic detection across all WoW versions (Vanilla through MoP)
- **Smart chunk handling**: Version-aware logic for conditional chunk presence
- **TrinityCore compliance**: Validated against server implementation for chunk behavior
- **Map type detection**: Better distinction between terrain maps and WMO-only maps
- **Comprehensive test suite**: Version-specific tests covering all WoW expansions

### Fixed

- **MWMO chunk writing**: Now uses version-aware logic to determine if MWMO should be written
- **Version compatibility**: Properly handles Cataclysm+ breaking changes for terrain maps
- **Chunk validation**: Enhanced validation for version-specific chunk requirements

## [0.2.0] - 2025-06-28

### Added

- Initial release of wow-wdt crate
- Support for WDT (World Data Table) files
- MPHD header parsing with world flags
- MAIN chunk for ADT tile information
- MAID chunk support for file data IDs (Legion+)
- WMO-only world support
- Map metadata and properties
- Tile existence checking
- Version-specific feature support
- Validation tools
