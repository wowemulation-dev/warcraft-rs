# Changelog

All notable changes to wow-m2 will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Support for WotLK (Wrath of the Lich King) M2 model and skin formats
- Texture filename parsing functionality
- Support for old skin format in the skin-info command
- JPEG header information support for BLP textures embedded in M2 files

### Fixed

- Fixed BLP texture parsing in M2 models - corrected header field order, width/height types (u16 to u32), and added version field reading
- Fixed BLP texture parsing to properly handle JPEG header information and palette data for uncompressed formats
- Fixed mipmap offset and size arrays to use fixed-size arrays ([u32; 16]) instead of dynamic vectors

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
