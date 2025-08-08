# Graphics Formats Implementation Plan

Based on comprehensive empirical analysis of BLP, M2, and WMO formats across WoW versions 1.12.1 through 5.4.8, this document outlines the required updates to the graphics format crates.

## Analysis Summary

### BLP Format Findings (143 files analyzed)
- **Version Consistency**: 100% BLP2 across all WoW versions
- **Alpha Type Evolution**: alpha_type=7 introduced in TBC (2.4.3), becomes dominant by Cataclysm
- **Compression Trends**: DXT remains dominant, RAW1 palettized decreases over time
- **Content Type**: 100% Direct format (no JPEG content_type found)
- **New Features**: Enhanced alpha blending modes, non-square texture support

### M2 Format Findings (250+ files analyzed)
- **Version Evolution**: 256 (Vanilla) → 260 (TBC) → 264 (WotLK) → 272 (Cataclysm/MoP)
- **Structure Consistency**: 100% inline data, no external chunks found through MoP 5.4.8
- **Chunked Format**: Capability introduced in v264 but unused until post-MoP expansions
- **Magic Consistency**: All files use MD20 magic across versions

### WMO Format Findings (60+ files analyzed)
- **Version Stability**: Version 17 consistent across all analyzed versions
- **Core Chunks**: 17 standard chunks present in all versions 1.12.1-3.3.5a
- **Cataclysm Addition**: MCVP (Convex Volume Planes) chunk in transport WMOs
- **Group Files**: MOCV (vertex colors) added in Cataclysm group files

## Implementation Tasks

## 1. BLP Format Updates (wow-blp crate)

### Priority 1: Alpha Type Support
```rust
// src/types/header.rs - Add alpha_type=7 support
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaType {
    None = 0,
    OneBit = 1,
    EightBit = 8,
    Enhanced = 7,  // NEW: Enhanced alpha blending (TBC+)
}
```

**Tasks:**
- [ ] Add `AlphaType::Enhanced = 7` variant
- [ ] Update parser logic in `parser/header.rs` to handle alpha_type=7
- [ ] Add conversion logic for enhanced alpha in `convert/` modules
- [ ] Update documentation with TBC+ alpha evolution
- [ ] Add test cases for alpha_type=7 textures

### Priority 2: Version-Specific Validation
```rust
// src/parser/header.rs - Add version-aware validation
pub fn validate_for_wow_version(&self, wow_version: WowVersion) -> Result<(), BlpError> {
    match wow_version {
        WowVersion::Vanilla => {
            // Validate alpha_type is only 0, 1, or 8
            if self.alpha_type == 7 {
                return Err(BlpError::UnsupportedAlphaType(7));
            }
        }
        WowVersion::TBC | WowVersion::WotLK | WowVersion::Cataclysm | WowVersion::MoP => {
            // All alpha types supported
        }
    }
    Ok(())
}
```

### Priority 3: Enhanced Statistics Tracking
```rust
// src/types/mod.rs - Add format statistics
#[derive(Debug, Clone)]
pub struct BlpStatistics {
    pub wow_version: Option<WowVersion>,
    pub compression_ratio: f32,
    pub alpha_coverage: f32,
    pub mipmap_count: u8,
    pub estimated_vram_usage: u32,
}
```

## 2. M2 Format Updates (wow-m2 crate)

### Priority 1: Version-Specific Handling
```rust
// src/version.rs - Add empirically-verified versions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M2Version {
    Vanilla = 256,
    BurningCrusade = 260,
    WrathOfTheLichKing = 264,
    Cataclysm = 272,
    MistsOfPandaria = 272,  // Same as Cataclysm
}

impl M2Version {
    pub fn supports_chunked_format(&self) -> bool {
        *self as u32 >= 264
    }
    
    pub fn supports_external_chunks(&self) -> bool {
        // Based on empirical analysis: no external chunks through MoP
        false
    }
}
```

**Tasks:**
- [ ] Update version enum with empirical findings
- [ ] Add version-specific feature flags
- [ ] Implement chunked format detection (even though unused)
- [ ] Add validation for version consistency
- [ ] Update parser to handle all four verified versions

### Priority 2: Inline Data Structure Optimization
```rust
// src/parser/mod.rs - Optimize for inline data structure
pub struct M2Parser {
    version: M2Version,
    inline_data_mode: bool,  // Always true through MoP
}

impl M2Parser {
    pub fn new(data: &[u8]) -> Result<Self, M2Error> {
        let version = Self::detect_version(data)?;
        let inline_data_mode = version.supports_external_chunks();
        
        Ok(Self {
            version,
            inline_data_mode: !inline_data_mode,  // Invert since no external chunks found
        })
    }
}
```

### Priority 3: Forward Compatibility
```rust
// src/chunks/mod.rs - Prepare for future chunked format
#[derive(Debug, Clone)]
pub enum ChunkData {
    MD21(SkinProfiles),      // Skin profiles
    SFID(Vec<u32>),         // Skin file data IDs
    AFID(Vec<u32>),         // Animation file data IDs
    BFID(u32),              // Bone file data ID
    TXID(Vec<u32>),         // Texture file IDs
    // Add more chunks as needed for post-MoP versions
}
```

## 3. WMO Format Updates (wow-wmo crate)

### Priority 1: MCVP Chunk Support
```rust
// src/chunk.rs - Add MCVP chunk
#[derive(Debug, Clone)]
pub struct McvpChunk {
    pub convex_volume_planes: Vec<ConvexVolumePlane>,
}

#[derive(Debug, Clone, Copy)]
pub struct ConvexVolumePlane {
    pub normal: [f32; 3],
    pub distance: f32,
    pub flags: u32,
}
```

**Tasks:**
- [ ] Add MCVP chunk parsing for Cataclysm+ WMOs
- [ ] Update chunk parser to handle optional MCVP
- [ ] Add transport WMO detection logic
- [ ] Implement convex volume plane calculations
- [ ] Add tests with Transport_Pirate_ship02.wmo

### Priority 2: Group File MOCV Support
```rust
// src/wmo_group_types.rs - Add vertex colors
#[derive(Debug, Clone)]
pub struct GroupVertexColors {
    pub colors: Vec<[u8; 4]>,  // BGRA format
}

// Update MOGP sub-chunk parsing
pub enum MogpSubChunk {
    MOPY(MaterialInfo),
    MOVI(VertexIndices),
    MOVT(Vertices),
    MOCV(GroupVertexColors),  // NEW: Cataclysm+
    // ... other sub-chunks
}
```

### Priority 3: Version-Aware Parsing
```rust
// src/parser.rs - Add WoW version context
pub struct WmoParser {
    wow_version: WowVersion,
    support_mcvp: bool,
    support_mocv: bool,
}

impl WmoParser {
    pub fn new(wow_version: WowVersion) -> Self {
        Self {
            wow_version,
            support_mcvp: wow_version >= WowVersion::Cataclysm,
            support_mocv: wow_version >= WowVersion::Cataclysm,
        }
    }
}
```

## 4. Comprehensive Testing Strategy

### Test Data Organization
```
tests/
├── compliance/
│   ├── blp/
│   │   ├── vanilla/     # BLP files from 1.12.1
│   │   ├── tbc/         # BLP files from 2.4.3
│   │   ├── wotlk/       # BLP files from 3.3.5a
│   │   ├── cataclysm/   # BLP files from 4.3.4
│   │   └── mop/         # BLP files from 5.4.8
│   ├── m2/
│   │   └── [same structure]
│   └── wmo/
│       └── [same structure]
├── scenarios/
│   ├── cross_version_compatibility/
│   ├── format_evolution/
│   └── edge_cases/
└── integration/
    ├── mpq_integration/
    └── real_world_usage/
```

### Test Implementation Tasks
- [ ] Create version-specific test suites for each format
- [ ] Add cross-version compatibility tests
- [ ] Implement format evolution validation tests
- [ ] Add performance benchmarks with real data
- [ ] Create integration tests with warcraft-rs CLI

## 5. Documentation Updates

### API Documentation
- [ ] Update crate-level documentation with empirical findings
- [ ] Add version compatibility matrices
- [ ] Document format evolution timelines
- [ ] Add examples for each WoW version
- [ ] Create migration guides for version handling

### Format Specifications
- [ ] Update format specs with empirical data
- [ ] Add version-specific feature matrices
- [ ] Document chunk evolution timelines
- [ ] Add implementation notes for edge cases

## 6. Performance Optimizations

### Version-Specific Optimizations
```rust
// Example: BLP parser optimization
match self.wow_version {
    WowVersion::Vanilla => {
        // Skip alpha_type=7 checks, optimize for common patterns
        self.parse_vanilla_optimized(data)
    }
    WowVersion::TBC | WowVersion::WotLK => {
        // Enable alpha_type=7 support
        self.parse_with_enhanced_alpha(data)
    }
    WowVersion::Cataclysm | WowVersion::MoP => {
        // Optimize for DXT-dominant, high alpha usage
        self.parse_modern_optimized(data)
    }
}
```

### Tasks
- [ ] Add version-specific parsing paths
- [ ] Optimize common format patterns per version
- [ ] Implement fast-path validation for known patterns
- [ ] Add memory usage optimizations
- [ ] Create benchmarks comparing version-specific vs generic parsing

## 7. Error Handling Improvements

### Version-Aware Error Messages
```rust
#[derive(Debug, thiserror::Error)]
pub enum BlpError {
    #[error("Alpha type {0} not supported in {1:?}")]
    UnsupportedAlphaTypeForVersion(u8, WowVersion),
    
    #[error("Compression type {0} rarely used in {1:?}, possible data corruption")]
    UncommonCompressionForVersion(u8, WowVersion),
    
    #[error("Feature requires WoW version {required:?} or later, got {actual:?}")]
    InsufficientWowVersion { required: WowVersion, actual: WowVersion },
}
```

## Implementation Priority

### Phase 1 (High Priority)
1. BLP alpha_type=7 support
2. M2 version handling updates
3. WMO MCVP chunk support
4. Basic version-aware testing

### Phase 2 (Medium Priority)
1. Performance optimizations
2. Comprehensive test suites
3. Enhanced error handling
4. Documentation updates

### Phase 3 (Future Enhancement)
1. Advanced format statistics
2. Cross-version migration tools
3. Format validation utilities
4. Integration with warcraft-rs CLI improvements

## Success Criteria

- [ ] All graphics crates handle empirically-verified format variations
- [ ] Version-specific parsing provides correct results for all analyzed samples
- [ ] Performance improvements measurable for common use cases
- [ ] Comprehensive test coverage with real MPQ data
- [ ] Documentation accurately reflects empirical findings
- [ ] API compatibility maintained while adding new features

This implementation plan ensures our graphics format crates accurately reflect the real-world format usage patterns discovered through empirical analysis of original WoW MPQ archives.