# Changelog

All notable changes to wow-adt will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - 0.7.0

### Breaking Changes

- **Complete Parser Rewrite** - Migrated from manual binary parsing to BinRead-based declarative parsing
  - All chunk parsing now uses BinRead derive macros for type safety
  - Changed internal parser architecture to two-phase (discovery + typed parsing)
  - Module structure reorganized with new chunk-specific modules
  - Some internal APIs changed due to new architecture
- **API Reorganization** - New high-level type-safe APIs replace low-level chunk access
  - `RootAdt`, `Tex0Adt`, `Obj0Adt`, `LodAdt` types replace generic chunk maps
  - `parse_adt()` now returns `ParsedAdt` enum instead of single type
  - Chunk access patterns changed to use typed accessors

### Added

- **BinRead-Based Parsing** - Complete rewrite using declarative binary parsing
  - Two-phase parsing: fast chunk discovery followed by selective typed parsing
  - Reduced memory usage through selective parsing of needed chunks
  - Improved error messages with chunk-level context
  - Type-safe chunk parsing with compile-time validation
- **New Modular Architecture** - Comprehensive module reorganization
  - `chunk_discovery` - Fast chunk enumeration phase for selective parsing
  - `chunk_header` - Chunk header abstraction with consistent handling
  - `chunk_id` - Type-safe chunk identifier with validation
  - `chunks/` - Modular chunk parsing directory:
    - `mcnk/` - Complete MCNK and subchunk parsing (header, mcvt, mcnr, mcly, mcal, etc.)
    - `mh2o/` - Water chunk parsing (header, instance, vertex)
    - `placement.rs` - Doodad and WMO placement chunks (MDDF, MODF)
    - `simple.rs` - Simple header-only chunks (MVER, MHDR, etc.)
    - `strings.rs` - String table chunks (MWMO, MMDX)
    - `blend_mesh.rs` - Blend mesh chunks (MBMH, MBBB, MBNV)
  - `root_parser` - Root ADT file parsing with MCNK chunks
  - `split_parser` - Split file parsing for tex0 and obj0 files
  - `adt_set` - High-level API for loading complete split file sets
  - `merger` - Split file merging utilities
  - `split_set` - Split file discovery and validation
  - `api` - Type-safe high-level APIs (RootAdt, Tex0Adt, Obj0Adt, LodAdt)
  - `builder/` - ADT construction and serialization:
    - `adt_builder.rs` - Builder API for ADT creation
    - `built_adt.rs` - Built ADT representation
    - `serializer.rs` - Binary serialization
    - `validation.rs` - Pre-serialization validation
  - `file_type` - ADT file type detection and validation
- **Enhanced Split File Support** - Comprehensive Cataclysm+ split file handling
  - `AdtSet` for loading complete split file sets with automatic discovery
  - Automatic split file discovery from root file path
  - Merged structure combining root, texture, and object data
  - Support for optional LOD files (Legion+)
- **New Examples** - Demonstrate new parsing capabilities
  - `load_split_adt` - Loading and merging split ADT file sets
  - `selective_parsing` - Selective chunk parsing for performance
- **Benchmarks** - Performance testing for new architecture
  - `discovery.rs` - Benchmark chunk discovery phase
  - `parsing.rs` - Benchmark typed chunk parsing

### Changed

- **Parser Architecture** - Two-phase parsing for performance and flexibility
  - Phase 1: Fast chunk discovery scans file and records chunk locations
  - Phase 2: Selective typed parsing of only needed chunks
  - Enables version detection before full parse
  - Reduces memory usage by avoiding unnecessary chunk parsing
- **Error Handling** - Enhanced error types with better context
  - Chunk-level error context with offset and size information
  - More descriptive error messages for parsing failures
  - Better error recovery for malformed chunks
- **Test Organization** - Expanded and reorganized test suite
  - Split compliance tests by expansion:
    - `vanilla.rs` - Vanilla WoW compliance tests
    - `tbc.rs` - The Burning Crusade compliance tests
    - `wotlk.rs` - Wrath of the Lich King compliance tests
    - `cataclysm.rs` - Cataclysm compliance tests (now passing)
    - `mop.rs` - Mists of Pandaria compliance tests
  - Removed old `trinitycore.rs` test file
  - Added integration tests for builder and modification
  - Enhanced test coverage with new chunk parsing tests
- **Benchmark Organization** - Restructured performance tests
  - Removed old `parser_benchmark.rs`
  - Added focused benchmarks for discovery and parsing phases
  - Better performance profiling with separate phases

### Fixed

- **Cataclysm Split File Parsing** - All Cataclysm compliance tests now pass
  - Fixed chunk parsing for split root files
  - Improved version detection for split file architecture
  - Corrected texture and object chunk handling in split files
- **Version Detection** - Enhanced detection for split files
  - Split root files now correctly identified as Cataclysm (not WotLK)
  - Better handling of chunk presence patterns for version detection
- **Chunk Parsing Accuracy** - Fixed multiple chunk parsing issues
  - Corrected MCNK subchunk parsing with proper offsets
  - Fixed MH2O water chunk parsing for all versions
  - Improved texture layer parsing in MCLY chunks

### Removed

- **Legacy Code** - Removed old parsing implementation
  - Old manual binary parsing code replaced by BinRead
  - Redundant chunk parsing utilities consolidated
  - Obsolete examples and tests removed

## [0.6.0] - 2025-10-31

### Added

- **Complete Cataclysm+ split file architecture support** (T080-T087)
  - `AdtSet` high-level API for loading complete split file sets
  - Automatic split file discovery (`SplitFileSet`)
  - Split file merger for combining root, texture, and object files into unified structure
  - Split file parsers for _tex0.adt and _obj0.adt files
- **New modules**:
  - `adt_set` - High-level API for loading and merging split file sets
  - `merger` - Utilities for merging split files into unified RootAdt
  - `split_set` - Split file path discovery and validation
- **Examples**:
  - `load_split_adt.rs` - Demonstrates loading and merging split ADT files

### Fixed

- **Version detection for Cataclysm split root files**
  - Split root files now correctly detected as Cataclysm (not WotLK)
  - Detection logic: files with MCNK but no MCIN are Cataclysm+ split root files
  - Fixes version misidentification for all Cataclysm+ root ADT files

### Changed

- **API documentation updated** with split file support examples
- **Module documentation** enhanced with new split file modules
- **Test suite**: All Cataclysm compliance tests now pass (14 tests enabled)

### Documentation

- Comprehensive module documentation for split file architecture
- Split file loading and merging examples
- Updated API documentation with Cataclysm+ patterns

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
