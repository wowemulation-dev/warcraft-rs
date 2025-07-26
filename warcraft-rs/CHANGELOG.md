# Changelog

All notable changes to warcraft-rs CLI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Support for old skin format in the m2 skin-info command
- Added `cargo-deny` configuration for dependency security scanning

### Changed

- Updated all dependencies to latest compatible versions

## [0.2.0] - 2025-06-28

### Added

- Initial release of warcraft-rs CLI tool
- Unified command-line interface for all WoW file formats
- MPQ archive operations (list, extract, info, verify)
- BLP texture conversion (to PNG/JPEG)
- ADT terrain file validation and information
- WDT map file analysis
- WDL low-resolution data inspection
- Colored output and progress indicators
- Cross-platform support (Windows, macOS, Linux)
- Comprehensive help and examples
