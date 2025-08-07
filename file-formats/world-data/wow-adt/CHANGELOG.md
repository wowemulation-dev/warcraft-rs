# Changelog

All notable changes to wow-adt will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-08-07

### Breaking Changes

- **MFBO chunk structure**: Fixed from incorrect 8 bytes to correct 36 bytes (2 planes × 9 int16 coordinates)
  - Changed from simple `min: u32, max: u32` to proper `min: [i16; 9], max: [i16; 9]` arrays
  - This matches TrinityCore server implementation and actual WoW client behavior
- **Version detection API**: Enhanced with `detect_from_chunks_extended()` for comprehensive version detection
- **API reorganization**: Some internal chunk structures updated for consistency

### Added

- **Complete WoW version support**: Automatic detection for Vanilla through Mists of Pandaria
- **Split ADT file support**: Cataclysm+ `_tex0`, `_obj0`, `_obj1`, `_lod` file parsing with merge functionality  
- **Enhanced chunk parsing**:
  - MAMP chunk parser (4-byte texture amplifier values for Cataclysm+)
  - MTXP chunk parser (texture parameters for MoP+ with 16-byte entries)
  - Enhanced MH2O water chunk support (already implemented)
- **TrinityCore compliance**: All chunk structures validated against authoritative server implementation
- **Comprehensive test suite**: 30+ tests including version-specific and TrinityCore compliance validation
- **Version-specific features**: Progressive chunk detection based on expansion evolution
- **High-level split file API**: `SplitAdtLoader` for easy handling of modern split terrain files

### Fixed

- **MFBO flight boundaries**: Now use correct 36-byte structure matching game files and server code
- **Version detection**: Enhanced logic can distinguish between all WoW versions based on chunk presence
- **Chunk parsing accuracy**: All chunk sizes and structures verified against real game data
- **Documentation examples**: Updated with correct API usage and field names

### Documentation

- Complete API documentation with technical details and chunk evolution timeline
- Migration guide for breaking changes with code examples
- Split file architecture explanation and usage patterns

## [0.2.0] - 2025-06-28

### Added

- Initial release of wow-adt crate
- Support for ADT terrain files (_obj0,_obj1, _tex0,_tex1)
- Chunk-based terrain parsing (MCNK)
- Height map and normal data
- Texture layer information
- Doodad and WMO placement data
- Liquid information (MCLQ)
- Vertex shading (MCCV)
- Shadow maps (MCSH)
- Alpha maps for texture blending
- Flight bounds and holes
- Parallel processing support for improved performance
- Comprehensive validation tools
