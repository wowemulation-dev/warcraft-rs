# Changelog

All notable changes to wow-wdl will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-08-07

### Added

- **Enhanced version support**: Improved parsing for different WDL format versions
- **Chunk support**: Full support for all documented chunks (MAOF, MAOH, MAHO, MWID, MWMO, MODF, ML)
- **Version detection**: Automatic detection across WoW expansions based on chunk evolution
- **TrinityCore compliance**: Validated chunk structures and behavior against server implementation

### Fixed

- **Version compatibility**: Better handling of version-specific chunk formats
- **Chunk parsing accuracy**: Enhanced parsing for all supported chunk types
- **Documentation consistency**: Updated examples to match actual API

## [0.2.0] - 2025-06-28

### Added

- Initial release of wow-wdl crate
- Support for WDL (World Data Low-resolution) files
- MAOF chunk parsing for area offsets
- MAOH chunk for ocean heights
- MAHO chunk for holes data
- Low-resolution height map support
- Mare ID mapping
- Version-specific chunk support (WotLK vs Legion)
- Conversion utilities between WDL versions
- Validation capabilities
