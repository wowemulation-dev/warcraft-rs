# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **M2 Format**: Version conversion roundtrip for all supported versions
  - Fixed `playable_animation_lookup` field handling for Vanilla (256-263)
  - Header parse expected this field for Vanilla but write omitted it
  - Version conversion now works: Vanilla, TBC, WotLK, Cataclysm, MoP
  - Cross-version conversion validated: WotLK↔Vanilla, WotLK↔Cata, etc.
- **BLP Convert CLI**: Alpha bits auto-detection from input image
  - Made `--alpha-bits` optional; auto-detects based on input image color type
  - DXT1 auto-selects 1-bit alpha for images with transparency, 0 otherwise
  - DXT3/DXT5/JPEG auto-select 8-bit alpha for images with transparency
  - Raw1/Raw3 auto-select 8-bit alpha for full quality
- **BLP Format**: JPEG encoding handles RGBA images
  - JPEG format stores RGB only; RGBA images now convert to RGB with warning
  - Previously crashed with "encoder does not support Rgba8 color type"
  - Alpha channel stripped during JPEG encoding as expected by format
- **M2 Format**: Skin file bone_indices parsing reads correct byte count
  - bone_indices is ubyte4 (4 bone indices per vertex), not single bytes
  - M2Array count represents vertex count; actual data is count × 4 bytes
  - Previously lost 75% of bone influence data during parsing
- **M2 Format**: Old skin format includes boneCountMax field
  - Added missing u32 boneCountMax at end of OldSkinHeader (offset 44)
  - Header size calculation now includes this field
  - Cross-format conversion preserves bone count limits

### Added

- **M2 Format**: Particle emitter animation preservation in roundtrip
  - Added `ParticleTrackType` enum for 10 animation track types
  - Added `ParticleAnimationRaw` struct for raw keyframe storage
  - Particle animation data collected during parse
  - Animation offsets relocated during write
  - Handles offset sharing for multiple tracks pointing to same data
- **M2 Format**: Ribbon emitter animation preservation in roundtrip
  - Added `RibbonTrackType` enum for 4 animation track types
  - Added `RibbonAnimationRaw` struct for raw keyframe storage
  - Ribbon animation data collected during parse
  - Animation offsets relocated during write
- **M2 Format**: Texture animation preservation in roundtrip
  - Added `TextureTrackType` enum for 5 animation track types
  - Added `TextureAnimationRaw` struct for raw keyframe storage
  - Texture animation data collected during parse (translation U/V, rotation, scale U/V)
  - Animation offsets relocated during write
  - Handles UV scrolling, rotation, and scaling effects
- **CLI**: Extended `m2 tree` command to display animation track data
  - Shows bone animation track summary (translation, rotation, scale counts)
  - Shows particle emitter animation track summary
  - Shows ribbon emitter animation track summary
  - Shows texture animation track summary
  - Displays total keyframe counts for each animation type

## [0.5.0] - 2025-01-09

### Added

- **M2 Format**: Animation runtime system for bone transform computation
  - `animation` module with interpolation, state management, and bone transforms
  - `BoneTransformComputer` for skeletal animation evaluation
  - `AnimationManager` for animation playback control
  - Interpolation utilities with quaternion slerp support
- **M2 Format**: Particle system simulation
  - `particles` module with emitter simulation and particle lifecycle
  - `ParticleEmitter` for GPU-style particle generation
  - `EmissionType` variants for different emission patterns
  - Particle and ribbon emitter parsing integrated into `M2Model`
- **WMO Format**: BSP tree point query for group collision
  - `bsp` module with efficient point-in-group testing
  - `BspTree` for accelerated spatial queries
  - `BspNodeExt` trait for node traversal
- **WMO Format**: Portal-based visibility culling
  - `portal` module for interior rendering optimization
  - `PortalCuller` for view frustum clipping through portals
  - `ConvexHull` and `Plane` geometry utilities
- **WMO Format**: Complete MOHD header parsing
  - Ambient color, WMO ID, bounding box, flags, and LOD count
  - Previously skipped 36 bytes now fully parsed
- **ADT Format**: MoP 5.3+ high_res_holes subchunk scanning
  - `parse_with_offset_and_size` for MCVT/MCNR discovery when offsets are zero
  - `scan_for_subchunk` utility for sequential chunk searching
- **ADT Format**: `TextureHeightParams` structure for MTXP entries
  - Height scale/offset for terrain texture blending
  - `uses_height_texture()` helper for `_h.blp` texture loading
- **ADT Format**: `CombinedAlphaMap` public API for alpha map extraction
- **Build**: Cargo configuration with aliases and stricter lints
  - `.cargo/config.toml` with `lint`, `test-all`, `qa` aliases
  - Enhanced clippy lints: `unwrap_used`, `panic`, `todo`, `dbg_macro`

### Changed

- **ADT Format**: MH2O vertex storage changed from `Vec` to 9×9 sparse grid (BREAKING)
  - `VertexDataArray` now uses `Box<[Option<T>; 81]>` for grid-aligned access
  - Vertex indexing as `z * 9 + x` matches noggit-red approach
  - `validate_coverage()` replaces `validate_count()` for position-based validation
- **ADT Format**: MTXP chunk parsing updated to proper structure
  - Changed from `Vec<u32>` to `Vec<TextureHeightParams>` (16 bytes per entry)
  - Includes flags, height_scale, height_offset fields
- **DBC Format**: WDB2 parser handles extended header (Cataclysm 4.0+)
  - Basic header (build <= 12880): 28 bytes
  - Extended header (build > 12880): 48 bytes + index arrays
  - Properly skips index/string length arrays before record data
- **Build**: Workspace lints expanded with safety and quality checks
  - `unwrap_used`, `expect_used`, `panic` warnings for library code
  - `print_stdout`, `print_stderr` warnings to prefer logging

### Fixed

- **MPQ Format**: HET/BET file lookup collision resolution
  - Now verifies BET hash for each HET collision candidate
  - Prevents incorrect file matches when 8-bit HET hash collides
- **BLP Format**: DXT decompression handles undersized mipmap buffers
  - Zero-pads compressed data when smaller than required block size
  - Matches SereniaBLPLib behavior for small mipmaps
- **ADT Format**: MCNK position coordinate order documented and exposed
  - File stores `[Z, X, Y]`, added `world_position()` helper returning `[X, Y, Z]`
  - Documentation clarifies raw field access order
- **ADT Format**: MCNK `do_not_fix_alpha_map` flag corrected to bit 15 (0x8000)
  - Was incorrectly checking bit 8 (0x100)
- **ADT Format**: MH2O exists bitmap reads exact byte count
  - Calculates `(width * height + 7) / 8` bytes instead of fixed 8 bytes
  - Prevents reading into vertex data for small liquid instances
- **ADT Format**: CombinedAlphaMap layer access updated for BinRead structure
  - Uses `chunk.layers` (MclyChunk) and `chunk.alpha` (McalChunk)
  - Accesses `layer.flags.alpha_map_compressed()` method
- **ADT Format**: RLE decompression simplified to not enforce row boundaries
  - Runs can span row boundaries per actual WoW file behavior

- **ADT Format**: Complete BinRead-based parser rewrite with two-phase parsing architecture
  - Declarative chunk parsing using BinRead derive macros
  - Two-phase parsing: fast chunk discovery followed by selective typed parsing
  - New modular architecture with 15+ new modules for maintainable code
  - High-level type-safe APIs: `RootAdt`, `Tex0Adt`, `Obj0Adt`, `LodAdt`
  - Enhanced split file support with `AdtSet` for automatic loading and merging
  - Comprehensive builder API for ADT creation and serialization
  - Selective parsing capabilities for performance optimization
- **MPQ Format**: Binary patch file (PTCH) support for Cataclysm+ updates
  - COPY patches for simple file replacement
  - BSD0 patches using bsdiff40 algorithm for binary diffs
  - Automatic patch detection and application in PatchChain
  - MD5 verification for patch integrity
  - RLE compression algorithm support
  - Enhanced PatchChain with transparent patch application
- **Testing**: Expanded test coverage across format parsers
  - ADT compliance tests split by expansion (vanilla, tbc, wotlk, cataclysm, mop)
  - MPQ patch chain integration tests
  - New benchmarks for ADT discovery and parsing phases
- **Examples**: New demonstrations for enhanced functionality
  - `load_split_adt` - ADT split file loading and merging
  - `selective_parsing` - Performance-optimized chunk parsing
  - MPQ patch testing examples (`check_patch_flags`, `test_patch_chain`, etc.)
- **CLI Enhancements**: Improved command handling for ADT and MPQ operations
  - Better ADT format support in CLI commands
  - Enhanced MPQ patch chain handling in extraction

### Changed

- **ADT Parser Architecture**: Complete rewrite for maintainability and performance
  - Migrated from manual binary parsing to declarative BinRead macros
  - Two-phase parsing reduces memory usage via selective chunk loading
  - Module structure reorganized with chunk-specific subdirectories
  - Error handling enhanced with chunk-level context
- **MPQ PatchChain**: Refactored for automatic patch application
  - Transparent PTCH patch handling during file reads
  - Simplified API with automatic patch detection
  - Enhanced priority-based file resolution
- **Test Organization**: Comprehensive test suite restructuring
  - Compliance tests organized by WoW expansion
  - Integration tests separated by functionality
  - Focused benchmarks for performance profiling
- **Dependency Management**: Cargo.lock cleanup
  - Removed obsolete dependencies (2,272 lines reduced)
  - Updated core dependencies for better compatibility

### Fixed

- **ADT Parsing**: All Cataclysm compliance tests now passing
  - Fixed split file version detection (Cataclysm vs WotLK)
  - Corrected MCNK subchunk parsing with proper offsets
  - Improved texture and object chunk handling in split files
  - Enhanced MH2O water chunk parsing across all versions
- **MPQ Compression**: Minor fixes in compression algorithm integration
  - Better RLE compression handling
  - Improved compression method selection

### Removed

- **Development Tools**: Cleaned up temporary analysis utilities
  - Removed Python M2 analysis tools from `tools/` directory
  - Removed legacy ADT parser benchmark
  - Consolidated redundant test files

## [0.4.0] - 2025-08-29

### Added

- **WMO Format**: Complete chunk support with test-driven implementation
  - GFID (Group File ID) - File references for group data
  - MORI (Portal references) - Portal connectivity information
  - MORB (Bounding boxes) - Spatial boundaries for groups
  - MOTA (Portal attachment) - Portal-to-group associations
  - MOBS (Shadow batches) - Shadow rendering optimization data with signed index support
- **WMO Tree Command**: Comprehensive visualization enhancements
  - Increased chunk coverage from 48.8% to 97.6% (40/41 chunks displayed)
  - Added `--detailed` flag for complete field-level data visibility
  - Enhanced material display with shader, blend mode, and texture data
  - Improved statistics display with chunk counts and sizes
  - Added verbose metadata for all chunk types
- **WMO Chunk ID Mapping**: Extended chunk recognition for cross-version compatibility
  - Added alternative byte patterns for MOVB (VBOM, BVOM)
  - Added alternative byte patterns for MFOG (GFOM, GOFM)
  - Ensures proper chunk identification across all WoW expansions

### Changed

- **WMO Parser Architecture**: Complete rewrite using BinRead with multi-phase parsing
  - Migrated from manual binary parsing to declarative BinRead derive macros
  - Implemented two-phase parsing: chunk discovery followed by typed parsing
  - Added ChunkHeader abstraction for consistent chunk handling
  - Introduced modular parser architecture with separate root and group parsers
  - Enhanced error handling with detailed parsing context and chunk validation
  - Improved maintainability through declarative structure definitions
- **M2 Parser Consolidation**: Streamlined codebase after bone parsing fixes
  - Removed 17 temporary debugging examples used during corruption analysis
  - Consolidated vertex validation and coordinate transformation logic
  - Enhanced production code with lessons learned from debugging phase
  - Improved Python M2 parser with two-pass bone resolution

### Fixed

- **WMO MOBS Structure**: Corrected shadow batch index interpretation
  - Changed `index_count` from u16 to i16 for proper signed value handling
  - Fixed negative index values (0xFFF8 = -8) previously showing as 65528
  - Eliminates impossible index ranges in shadow batch data
- **MPQ SIMD Code**: Resolved Rust version compatibility issues
  - Added lint configuration to handle unsafe block requirements across Rust versions
  - Updated io::Error creation to use newer io::Error::other() method
  - Maintains compatibility with both MSRV (1.86.0) and stable Rust toolchain
- **WMO Parser Robustness**: Comprehensive cross-expansion validation
  - Tested against 149 WMO files from Vanilla, WotLK, and Cataclysm/MoP
  - Achieved 100% parsing success rate with zero errors
  - Properly handles float values stored in texture index fields
  - Correctly processes zero-count doodad references
  - Supports high material counts (100+) in complex models

- **M2 Triangle Index Parsing**: Critical mesh connectivity corruption causing fragmented geometry (CRITICAL FIX)
  - **Double Indirection Bug**: Fixed unnecessary two-level indirection in `get_resolved_indices()`
    - Triangle indices were incorrectly resolved as `indices[triangles[i]]` instead of direct usage
    - Created sequential indices [0,1,2,3...] resulting in flat "airplane" geometry
    - Now correctly uses triangle array values [76,21,23,29...] for proper 3D mesh connectivity
    - Eliminates fragmented rendering and mesh topology corruption
  - **Mesh Topology Restoration**: Models now render with correct 3D geometry
    - Rabbit model displays proper organic shape instead of fragmented artifacts
    - All M2 models benefit from corrected triangle-to-vertex mapping
    - Verified against reference blender-wow-studio triangle connectivity data

- **M2 Quaternion Parsing**: Critical bone rotation corruption across all animated models (CRITICAL FIX)
  - **X-Component Sign Error**: Fixed quaternion X-component negation in `M2CompQuat::parse()`
    - pywowlib reference implementation negates X-component during parsing
    - warcraft-rs was not applying negation, causing incorrect bone rotations
    - All bone animations now match reference implementation exactly
    - Eliminates animation artifacts and incorrect model poses

- **M2 Vertex Validation System**: Enhanced validation with configurable corruption handling
  - **ValidationMode System**: Three validation modes for different use cases
    - `Strict`: Aggressive fixes for all potential issues (legacy behavior)
    - `Permissive`: Smart corruption detection while preserving valid static geometry (default)
    - `None`: Preserve all original data without modifications
  - **Bone Index Validation**: Fixed critical u8 wraparound bug in bone count validation
    - Previously failed for models with >255 bones due to improper type casting
    - Now correctly validates bone indices against full u32 bone count range
    - Prevents out-of-bounds bone references and rendering crashes

- **M2 Bone Parsing**: Critical data corruption issues in vertex and bone parsing (MAJOR FIX)
  - **Invalid Bone Indices**: Vertices referencing non-existent bones (e.g., bones [51, 196, 141, 62] when only 34 exist)
    - Enhanced `M2Vertex::parse_with_validation()` with comprehensive bone index validation
    - Invalid indices automatically clamped to root bone (index 0) for safety
    - Prevents rendering failures and vertex positioning corruption
  - **Missing Bone Weights**: 17% of vertices had zero bone weights making them unmovable
    - Zero-weight vertices now automatically assigned full weight to root bone
    - Ensures all vertices participate in skeletal animation properly
    - Maintains model deformation integrity during animation playback
  - **NaN Bone Pivot Coordinates**: Corrupted floating-point values in bone pivot points
    - Added NaN detection and replacement with safe defaults (0.0) in `M2Bone::parse()`
    - Prevents matrix calculation failures and rendering instability
    - Maintains bone hierarchy integrity across model transformations
  - **Structural Alignment**: Fixed vertex structure parsing across all WoW versions
    - Corrected 48-byte vertex structure with secondary texture coordinates
    - Eliminated 8-byte offset corruption that affected all subsequent vertex data
    - Aligned with WMVx reference implementation for cross-version compatibility
  - **Cross-Version Validation**: All fixes verified with real WoW models
    - Tested against Vanilla (HumanMale, Rabbit), TBC (DraeneiMale), and WotLK models
    - 100% parsing success rate with zero corruption detection
    - Comprehensive test suite (144/144 tests passing) with new validation coverage

### Changed

- **M2 Bone Field Identification**: Documented previously unknown TBC+ bone field
  - Identified `unknown1` field as `boneNameCRC` based on wowdev.wiki documentation
  - Field contains CRC hash of bone name string for debugging/identification
  - Renamed field from `unknown1_tbc` to `bone_name_crc` for semantic clarity
  - Enhanced debug output to display CRC values in hexadecimal format
  - Enables O(1) bone lookup by name hash in game servers and tools

### Added

- **M2 Vertex Skinning System**: Complete vertex transformation pipeline for proper mesh geometry
  - **M2Skinner**: Full skeletal animation system for transforming vertices from bind pose
    - Bone transformation matrix calculation from pivot points and quaternions
    - Multi-bone vertex influences with proper weight normalization (up to 4 bones per vertex)
    - Bone hierarchy support with parent-child relationship handling
    - Linear blend skinning implementation using standard formula
  - **SkinningOptions**: Configurable behavior for different use cases
    - Error handling for invalid bone indices and zero weights
    - Performance optimization settings for batch processing
    - Animation frame support for keyframe evaluation
  - **Coordinate System Integration**: Seamless integration with coordinate transformation system
    - Direct export to Blender, Unity, Unreal Engine coordinate spaces
    - Proper handling of WoW's (North, West, Up) coordinate system
    - Batch transformation capabilities for large models

- **M2 Coordinate System Support**: Complete transformation utilities for cross-application compatibility
  - **CoordinateSystem**: Support for Blender, Unity, and Unreal Engine coordinate systems
    - WoW to Blender: `(y, -x, z)` position transformation
    - WoW to Unity: `(-y, z, x)` position transformation  
    - WoW to Unreal Engine: `(x, -y, z)` position transformation
    - Proper quaternion transformations for each target system
  - **CoordinateTransformer**: High-performance batch transformation system
    - SIMD-optimized operations using glam library
    - Efficient matrix-based transformations for large datasets
    - Support for both individual and batch vertex processing
  - **Comprehensive Documentation**: Complete coordinate system documentation with visual guides
    - ASCII diagrams showing coordinate system orientations
    - Real-world examples with actual coordinate values
    - Performance optimization guidelines and best practices

- **Enhanced Python M2 Parser**: Complete feature parity with Rust implementation
  - **M2CompQuat Support**: Compressed quaternion parsing with X-component negation
    - Matches pywowlib reference implementation exactly
    - Proper conversion to float quaternions with ±32767 scaling
    - Euler angle conversion for debugging and visualization
  - **Animation Track Parsing**: Complete M2Track support for pre-WotLK and WotLK+ formats
    - Rotation, translation, and scale track extraction
    - Keyframe data resolution with proper timestamp handling
    - Support for both embedded and external animation data
  - **Enhanced Validation System**: Three validation modes matching Rust implementation
    - Strict, Permissive, and None validation options
    - Smart corruption detection for bone indices and weights
    - 17% vertex corruption detection and correction capabilities
  - **Coordinate System Integration**: Python coordinate transformation utilities
    - Direct Blender, Unity, Unreal Engine coordinate conversion
    - Batch transformation support for performance
    - Matrix-based transformations for advanced use cases

- **M2 Animation Data Extraction**: Complete bone animation and bind pose data resolution
  - **M2TrackResolver**: Follows M2Array offsets to load actual keyframe data from file
    - Resolves timestamps, values, and ranges for animation tracks
    - Extension traits for easy Vec3 and Quaternion track data access
  - **ResolvedBoneAnimation**: Comprehensive bone transform data structure
    - Bind pose translation, rotation (quaternion), and scale extraction
    - Full animation keyframe data with timestamps
    - Automatic fallback to identity transforms when no data exists
  - **M2ModelAnimationExt**: Extension trait for animation data resolution
    - `resolve_bone_animations()` loads all bone animation data
    - `get_bind_pose()` extracts rest position transforms
    - Verified with vanilla WoW models (96 bones, 63 with animation data)
- **M2 Bone Parsing**: Version-aware bone structure parsing with AnimationBlock support
  - Added `ranges` field to M2Track for pre-Wrath versions (v263 and earlier)
  - Version-specific bone parsing that handles size differences between WoW expansions
  - Fixed embedded skin index resolution with proper two-level indirection
  - Comprehensive bone validation with NaN detection and parent hierarchy checks
- **M2 Enhanced Parser**: Comprehensive data extraction for vanilla WoW models
  - Complete vertex parsing with bone weights and UV coordinates
  - Full bone hierarchy extraction with parent-child relationships
  - All animation sequences with version-aware timing (v256 vs v260+)
  - Texture definitions with type classification (Skin, Hair, Monster, etc.)
  - Material properties with blend modes and transparency flags
  - Embedded skin data extraction for pre-WotLK models (v263 and earlier)
  - Model statistics calculation with bounding box and triangle counts
  - Rich visualization with hierarchical bone display and animation lists
- **M2 Python Tools**: Reference implementation for M2 format validation
  - Comprehensive M2 parser with full data structure extraction
  - Batch testing tool for validating multiple models
  - Visual representation with Rich console output
  - JSON export capability for tool integration
  - Support for WoW versions 1.12.1 through 5.4.8
- **M2 Examples**: Test suite and demonstration programs
  - `enhanced_parser_demo` - Shows comprehensive parsing capabilities
  - `test_sample_models` - Validates parser with real game models
  - 100% success rate on vanilla WoW test models
- **MPQ Performance**: Comprehensive 8-phase optimization suite for 700x speedup on large archives
  - Phase 1: Optimized string formatting - removed redundant allocations in hot paths
  - Phase 2: Lazy loading architecture - defer hash/block table loading until needed
  - Phase 3: Security-first validation - early detection of malicious archives
  - Phase 4: Thread-safe buffer pooling - reuse allocations across operations
  - Phase 5: Compression bomb protection - prevent memory exhaustion attacks
  - Phase 6: Async I/O support - non-blocking archive operations with Tokio
  - Phase 7: Memory-mapped file support - zero-copy access for large archives
  - Phase 8: SIMD optimizations - hardware-accelerated CRC32 and hashing
- **MPQ Features**: New optional feature flags for progressive enhancement
  - `async` - Async/await support with Tokio runtime
  - `mmap` - Memory-mapped file access via memmap2
  - `simd` - SIMD acceleration with runtime CPU detection
- **MPQ Security**: Comprehensive security framework
  - Adaptive compression ratio limits based on algorithm
  - Session-wide decompression tracking
  - Pattern-based attack detection
  - Resource exhaustion prevention
- **MPQ Buffer Pool**: High-performance buffer management
  - Size-categorized pools (4KB, 64KB, 1MB)
  - Thread-safe buffer reuse
  - Configurable capacity limits
  - Statistics tracking for optimization

### Fixed

- **MPQ Performance**: Resolved 700x slowdown on Cataclysm/MoP archives
- **MPQ Security**: Fixed overly strict validation preventing empty archive creation
- **MPQ Tests**: Fixed SIMD test failures and async resource detection
- **MPQ Documentation**: Marked feature-gated examples with proper attributes

### Changed

- **MPQ Architecture**: Refactored to lazy-loading with `Arc<RwLock<T>>` for thread safety
- **MPQ Validation**: Security checks now run before any data processing
- **MPQ Compression**: Integrated buffer pooling for all decompression operations
- **MPQ CRC32**: Unified to use crc32fast for consistency across scalar/SIMD

## [0.4.0] - 2025-08-29

### Added

- **M2 Format**: Complete chunked format implementation with 25 chunks (Legion+ support)
  - File reference chunks: SFID, AFID, TXID, PFID, SKID, BFID
  - Particle system chunks: PABC, PADC, PSBC, PEDC, PCOL, PFDC
  - Rendering enhancement chunks: TXAC, EDGF, NERF, DETL, RPID, GPID, DBOC
  - Animation system chunks: AFRA, DPIV
  - Export processing chunks: EXPT
- **M2 Format**: Comprehensive chunk validation and cross-reference checking
- **M2 Format**: Physics file (.phys) basic parsing support via PFID references
- **Code Quality**: Enhanced clippy linting with stricter rules (all, pedantic, nursery, cargo groups)
- **Testing**: Comprehensive QA script adapted from cascette-rs for better code quality assurance
- **Security**: Improved dependency security audit configuration with documented exceptions

### Fixed

- **M2 Parsing**: Fixed redundant guard expressions in chunk pattern matching (29 instances)
- **M2 Parsing**: Resolved needless borrows in iterator usage throughout codebase
- **M2 Parsing**: Fixed is_some_and pattern usage for cleaner conditional logic
- **M2 Parsing**: Corrected iterator patterns in rendering enhancement processing
- **M2 Parsing**: Fixed stream position handling in chunk infrastructure
- **CI/CD**: Skip CLI tests requiring MPQ test files in CI environment
- **Dependencies**: Updated slab from 0.4.10 to 0.4.11 to fix security vulnerability
- **Performance**: Optimized MPQ bulk extraction performance
- **Code Quality**: Removed unused imports and variables across test modules

### Changed

- **M2 Format**: Achieved 100% M2 specification compliance with chunked format support
- **Documentation**: Updated M2 format documentation to reflect comprehensive implementation
- **Code Style**: Applied rustfmt formatting consistently across entire codebase
- **Error Handling**: Enhanced error reporting with detailed chunk validation messages
- **Testing**: Expanded test coverage to 135+ unit and integration tests
- **Validation**: Improved cross-chunk reference validation and consistency checking

## [0.3.1] - 2025-08-12

### Fixed

- **wow-mpq**: Replaced custom ReadLittleEndian trait with standard
  byteorder crate across 50+ locations
- **wow-mpq**: Added generic error conversion helpers for compression
  algorithms
- **wow-mpq**: Standardized error handling patterns in compression
  module
- **wow-blp**: Extracted duplicate bounds checking logic into reusable
  module
- **wow-wmo**: Simplified error handling patterns in chunk reading code
- **wow-m2**: Fixed ribbon emitter parsing for Cataclysm/MoP using numeric
  version comparison
- **wow-mpq**: Fixed HET table creation to handle attributes files correctly
- **wow-mpq**: Fixed sector offset validation preventing false positive
  truncation errors

### Added

- **wow-blp**: Support for alpha_type=7 for TBC+ enhanced alpha blending
  compatibility
- **wow-m2**: Empirically verified version numbers (Classic=256, TBC=260,
  WotLK=264, Cata/MoP=272)
- **wow-wmo**: MCVP chunk support for Cataclysm+ transport collision
  volumes

### Changed (0.3.0)

- Applied rustfmt formatting fixes across all crates
- Removed 29 development test files (5,800+ lines) for cleaner codebase
- Refactored code for idiomatic Rust patterns

### Performance

- Achieved 99.5% parser success rate across 200+ test files

## [0.3.0] - 2025-08-07

### Breaking Changes

- **wow-adt**: MFBO chunk structure changed from 8 bytes to 36 bytes
  (2 planes × 9 int16 coordinates)
- **wow-wdt**: MWMO chunk handling changed for Cataclysm+ compatibility
  (only WMO-only maps include MWMO)
- **wow-adt**: Version detection API enhanced with chunk-based detection methods

### Added (0.3.0)

- **wow-adt**: Complete WoW version support (Vanilla through Mists of
  Pandaria) with automatic detection
- **wow-adt**: Split ADT file support for Cataclysm+ (`_tex0`, `_obj0`,
  `_obj1`, `_lod` files) with merge functionality
- **wow-adt**: MAMP chunk parser for 4-byte texture amplifier values
  (Cataclysm+)
- **wow-adt**: MTXP chunk parser for texture parameters with 16-byte
  entries (MoP+)
- **wow-wdt**: Enhanced version detection across all WoW expansions
- **wow-wdt**: Map type detection distinguishing terrain maps from WMO-only maps
- **wow-wdl**: Enhanced chunk support for all documented chunks (MAOF, MAOH,
  MAHO, MWID, MWMO, MODF, ML)
- **wow-mpq**: MutableArchive methods: read_file(), list(), find_file(),
  verify_signature(), load_attributes()
- **wow-mpq**: Complete compact() method implementation for archive
  defragmentation
- **wow-mpq**: Attributes file parsing handles both self-inclusive and
  self-exclusive cases
- **wow-m2**: WotLK M2 model and skin format support
- **wow-m2**: Texture filename parsing functionality
- **wow-m2**: Old skin format support
- **warcraft-rs**: cargo-deny configuration for dependency security scanning

### Fixed (0.3.0)

- **wow-adt**: MFBO flight boundaries now use correct structure matching
  TrinityCore server
- **wow-wdt**: MWMO chunk writing uses version-aware logic for Cataclysm+
  compatibility
- **wow-mpq**: Huffman decompression algorithm matches StormLib linked
  list approach
- **wow-mpq**: IMPLODE compression handling for Warcraft III MPQ archives
- **wow-mpq**: Attributes file parsing handles varying block counts
  across implementations
- **wow-m2**: BLP texture parsing with correct header field order and data types

### Changed

- **wow-mpq**: Enhanced attributes file block count detection with
  automatic fallback logic
- **wow-m2**: Replaced custom BLP implementation with wow-blp crate dependency
- **wow-m2**: BlpTexture now re-exports wow_blp::BlpImage for compatibility

### Removed

- **wow-mpq**: Invalid Huffman test case and obsolete PKWare compression tests
- **wow-m2**: Custom BLP parsing implementation

## [0.2.0] - 2025-06-28

### Added (0.2.0)

- **wow-mpq**: Complete parallel processing support with ParallelArchive struct
- **wow-mpq**: Multi-threaded functions: extract_from_multiple_archives(),
  search_in_multiple_archives(), process_archives_parallel(),
  validate_archives_parallel()
- **wow-mpq**: Thread-safe file handle cloning strategy for concurrent access
- **wow-mpq**: Parallel patch chain loading with from_archives_parallel()
  and add_archives_parallel()
- **wow-mpq**: Buffer pre-allocation optimizations for sector reading
- **wow-mpq**: Hash table mask caching for improved file lookup
- **wow-mpq**: Public list_files() and read_file_with_new_handle() methods
- **wow-mpq**: Rayon integration for CPU-optimal work distribution
- **wow-mpq**: SQLite database support for persistent filename hash storage
  and resolution
- **wow-mpq**: Database import functionality supporting listfiles, MPQ
  archives, and directory scanning
- **wow-mpq**: Automatic filename resolution through database lookup
  during archive operations
- **storm-ffi**: Complete archive modification support through C FFI
- **storm-ffi**: File operations: add, remove, rename with SFileAddFile,
  SFileRemoveFile, SFileRenameFile
- **storm-ffi**: Archive operations: create, flush, compact with
  SFileCreateArchive, SFileFlushArchive, SFileCompactArchive
- **storm-ffi**: File finding functionality with
  SFileFindFirstFile/NextFile/Close
- **storm-ffi**: File and archive attributes API support
- **warcraft-rs**: CLI with mpq subcommands: list, extract, info, verify
- **warcraft-rs**: mpq db subcommand with status, import, analyze, lookup,
  export, list operations
- **warcraft-rs**: Parallel processing enabled by default with --threads
  parameter
- **warcraft-rs**: --patch parameter for patch chain support in extract command
- **warcraft-rs**: BLP commands: convert, info, validate with mipmap
  generation and DXT compression
- **warcraft-rs**: ADT commands: info, validate, convert, tree with
  expansion name support
- **warcraft-rs**: WDT commands: info, validate, convert, tiles
- **warcraft-rs**: WDL commands: validate, convert, info
- **warcraft-rs**: Tree visualization for all formats with emoji icons
  and color support
- **wow-blp**: Complete BLP texture format support (BLP0, BLP1, BLP2)
- **wow-blp**: All compression formats: JPEG, RAW1 (palettized), RAW3, DXT1/3/5
- **wow-blp**: Mipmap support for internal and external mipmaps
- **wow-blp**: Bidirectional conversion between BLP and standard image formats
- **wow-blp**: Alpha channel support with 0, 1, 4, and 8-bit depths
- **wow-m2**: M2 model format parsing with header and version detection
- **wow-m2**: Global sequences, texture definitions, bone hierarchy parsing
- **wow-m2**: Vertex and triangle data access, skin file support
- **wow-m2**: Animation sequence data, material and render flag support
- **wow-wmo**: WMO root and group file parsing and writing support
- **wow-wmo**: Version support from Classic (v17) through The War Within (v27)
- **wow-wmo**: Version conversion capabilities between expansions
- **wow-wmo**: Builder API for programmatic WMO file creation
- **wow-adt**: ADT terrain file parsing for all chunk types
- **wow-adt**: Height maps, texture layers, doodad and WMO placement data
- **wow-adt**: Liquid information, vertex shading, shadow maps, alpha maps
- **wow-adt**: Version conversion between Classic, TBC, WotLK, and
  Cataclysm formats
- **wow-wdt**: WDT file support with MPHD header and MAIN chunk parsing
- **wow-wdt**: MAID chunk support for file data IDs (Legion+)
- **wow-wdt**: WMO-only world support with map metadata
- **wow-wdl**: WDL file support with MAOF, MAOH, MAHO chunk parsing
- **wow-wdl**: Low-resolution height maps and Mare ID mapping
- **wow-cdbc**: Client database (DBC) file parsing with DBD schema support
- **wow-cdbc**: Localized string support and row-based data access

### Fixed (0.2.0)

- **wow-mpq**: Critical sector reading bug truncating large files
- **wow-mpq**: Archive modification to properly update listfile and attributes
- **wow-mpq**: V3 archive compatibility issues with StormLib attributes
  file reading
- **wow-mpq**: V4 malloc crash by checking hi-block table necessity
- **wow-mpq**: ADPCM decompression overflow when bit shift value exceeds 31
- **wow-mpq**: SINGLE_UNIT file compression method detection
- **wow-mpq**: ZLIB decompression failures for specific file types
- **wow-mpq**: PATCH flag file handling with proper error messages
- **wow-wmo**: Integer overflow in group name parsing
- **wow-wmo**: Header size mismatch (60 vs 64 bytes) causing chunk misalignment
- **wow-wmo**: Texture validation to handle special marker values
- **wow-wmo**: Light type parsing to handle unknown types gracefully
- **wow-adt**: MH2O water chunk parsing for incomplete water data
- **wow-adt**: MFBO chunk handling for variable sizes between expansions

### Changed (0.2.0)

- **wow-mpq**: Parallel processing as default behavior for all CLI operations
- **wow-mpq**: Enhanced thread safety architecture for concurrent operations
- **wow-mpq**: Attributes files use StormLib-compatible 149-byte format
- **storm-ffi**: Renamed from storm to storm-ffi while retaining
  libstorm library name
- **storm-ffi**: Archive handles support both read-only and mutable operations
- **Project-wide**: Comprehensive test reorganization with component,
  integration, scenarios, compliance directories
- **Project-wide**: Consolidated examples from 50+ to 15 focused demonstrations
- **All crates**: Replaced byteorder crate with native Rust byte order functions
- **Documentation**: Fixed API discrepancies between documentation and
  implementation
- **Documentation**: Updated all code examples to compile correctly

### Performance (0.2.0)

- Up to 6x performance improvement for multi-archive operations through
  parallel processing
- Optimized sector reading with buffer pre-allocation strategies
- Enhanced file lookup performance through hash table mask caching
- CPU-optimal work distribution using rayon thread pools

### Removed (0.2.0)

- **wow-mpq**: Redundant create_het_table() method replaced by
  create_het_table_with_hash_table()
- **warcraft-rs**: --parallel and --sequential flags (parallel now default)
- **warcraft-rs**: --batch-size option (automatically optimized)
- **wow-mpq**: Redundant examples consolidated into comprehensive demonstrations

## [0.1.0] - 2025-06-13

### Added (0.1.0)

- **wow-mpq**: Complete archive modification API with MutableArchive
- **wow-mpq**: Automatic listfile and attributes updates during modifications
- **wow-mpq**: StormLib bidirectional compatibility for
  created/modified archives
- **wow-mpq**: Support for WoW versions 1.12.1 through 5.4.8
- **wow-mpq**: Support for MPQ format versions (V1, V2, V3 with HET/BET,
  V4 with advanced HET/BET)
- **wow-mpq**: Portable WoW data discovery using environment variables
  and common paths
- **wow-mpq**: test-utils feature for examples requiring WoW game data
- **wow-mpq**: Cross-platform path separator conversion (forward slash
  to backslash)
- **wow-mpq**: Graceful handling of Blizzard's 28-byte attributes file deviation
- **wow-mpq**: HET/BET table generation for V3+ archives with attributes
  file indexing
- **wow-mpq**: V4 archive creation with corrected hi-block table size
  calculation
- **wow-mpq**: ADPCM audio compression support with overflow protection
- **wow-mpq**: Support for all compression formats (ZLIB, BZIP2, ADPCM,
  Huffman, Sparse, LZMA)
- **wow-mpq**: Encryption/decryption support with hash and block table parsing
- **wow-mpq**: Extended attributes and HET/BET table support
- **wow-mpq**: Patch archive support with proper file resolution
- **wow-mpq**: Generic index file extraction when no listfile exists
- Initial workspace structure for World of Warcraft file format parsing
- Rust 2024 edition support with MSRV 1.86
- Comprehensive test organization (unit, integration, compliance, scenarios)

### Fixed (0.1.0)

- **wow-mpq**: V3 archive compatibility with StormLib attributes file reading
- **wow-mpq**: V4 malloc crash by proper hi-block table necessity checking
- **wow-mpq**: HET table creation to properly index attributes files
- **wow-mpq**: ADPCM decompression overflow protection
- **wow-mpq**: Sector reading validation preventing false positive
  truncation errors

### Changed (0.1.0)

- **wow-mpq**: Attributes files use full StormLib-compatible format
  (CRC32+MD5+timestamp) instead of CRC32-only

[0.5.0]: https://github.com/wowemulation-dev/warcraft-rs/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/wowemulation-dev/warcraft-rs/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/wowemulation-dev/warcraft-rs/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/wowemulation-dev/warcraft-rs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/wowemulation-dev/warcraft-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/wowemulation-dev/warcraft-rs/releases/tag/v0.1.0
