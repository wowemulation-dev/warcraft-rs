# Changelog

All notable changes to wow-mpq will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-06-28

### Added

- Initial release of wow-mpq crate
- Support for reading MPQ archives from WoW versions 1.12.1 through 5.4.8
- Support for MPQ format versions 1, 2, 3, and 4
- Comprehensive compression support (ZLIB, BZIP2, ADPCM, Huffman, Sparse, LZMA)
- Full encryption/decryption support
- Hash table (HT) and block table (BT) parsing
- Extended attributes support
- HET/BET table support for MPQ v3+
- Patch archive support with proper file resolution
- Parallel processing capabilities for improved performance
- StormLib compatibility layer for validation
- Comprehensive test suite and benchmarks
