# Changelog

All notable changes to wow-m2 will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Extended M2 parser validation to The Burning Crusade (2.4.3) models
- Created comprehensive TBC model test suite with Blood Elf and Draenei support
- Added version-specific documentation for vanilla (v256) vs TBC (v260) differences
- Implemented batch testing framework for multi-version model validation

### Fixed

- Fixed vanilla M2 embedded skin submesh parsing to use correct 32-byte alignment (empirically validated)
- Added version-aware submesh parsing to handle vanilla (32-byte) vs modern (48-byte) submesh structures
- Updated `SkinSubmesh::parse_with_version()` to correctly parse vanilla submeshes with 8×u16 + 4×f32 layout
- Enhanced `parse_embedded_skin()` to accept M2 version parameter for proper submesh structure detection

### Validated

- Successfully tested TBC (v260) models: HumanMale, BloodElfMale, DraeneiMale (100% success rate)
- Confirmed embedded skin support for TBC models (v260-263 cutoff validated)
- Verified increased model complexity in TBC: more bones (140 vs 96) and larger file sizes

## [0.3.0] - 2025-08-07

### Changed

- Version bump to 0.3.0 for coordinated workspace release
- Updated dependencies and documentation

### Added

- Support for WotLK (Wrath of the Lich King) M2 model and skin formats
- Texture filename parsing functionality
- Support for old skin format in the skin-info command

### Changed

- Replaced custom BLP texture implementation with dependency on `wow-blp` crate
- `BlpTexture` is now a re-export of `wow_blp::BlpImage` for backwards compatibility

### Fixed

- Fixed mipmap offset and size arrays to use fixed-size arrays ([u32; 16]) instead of dynamic vectors

### Removed

- Custom BLP parsing implementation (moved to using `wow-blp` crate instead)

## [0.2.0] - 2025-06-28

### Added

- Initial release of wow-m2 crate
- Support for M2 model format parsing
- Header parsing with version detection
- Global sequences support
- Texture definitions and lookups
- Bone hierarchy parsing
- Vertex and triangle data access
- Skin file (.skin) support
- Animation sequence data
- Material and render flag support
- Bounding box calculations
- Model validation capabilities
