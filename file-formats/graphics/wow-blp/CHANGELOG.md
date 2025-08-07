# Changelog

All notable changes to wow-blp will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2025-01-28

### Changed

- Version bump to 0.2.1 for coordinated workspace release
- Updated dependencies and documentation

## [0.2.0] - 2025-06-28

### Added

- Initial release of wow-blp crate
- Support for BLP texture format versions 0, 1, and 2
- JPEG compression support (BLP0/BLP1)
- Palette-based compression support
- DXT1, DXT3, DXT5 compression support (BLP2)
- Uncompressed BGRA format support
- Mipmap support with automatic generation
- Conversion to/from standard image formats (PNG, JPEG)
- Alpha channel preservation
- Command-line tool for BLP conversion
