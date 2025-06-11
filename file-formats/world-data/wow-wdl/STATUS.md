# wow-wdl - WDL Format Implementation Status

**Last Updated:** 2025-06-08

The `wow-wdl` crate provides WDL file support:

- **File Reading**: Core chunks implemented
- **File Writing**: Round-trip support
- **Data Parsing**: Documented structures supported
- **Version Support**: All WoW versions supported
- **Validation**: Basic validation implemented
- **Advanced Features**: Height interpolation and coordinate conversion planned
- **Testing**: Unit and integration tests

## Detailed Feature Matrix

### 📖 File Reading Operations

| Feature | Status | Notes |
|---------|--------|-------|
| **MVER Chunk** | ✅ Implemented | Version detection |
| **MAOF Chunk** | ✅ Implemented | 64x64 offset table |
| **MARE Chunk** | ✅ Implemented | 17x17 + 16x16 heights |
| **MAHO Chunk** | ✅ Implemented | 16 uint16 hole bitmasks |
| **MWMO Chunk** | ✅ Implemented | WMO filename list |
| **MWID Chunk** | ✅ Implemented | WMO filename offsets |
| **MODF Chunk** | ✅ Implemented | WMO placement data |
| **MLDD Chunk** | ✅ Implemented | Legion+ M2 placements |
| **MLDX Chunk** | ✅ Implemented | Legion+ M2 visibility |
| **MLMD Chunk** | ✅ Implemented | Legion+ WMO placements |
| **MLMX Chunk** | ✅ Implemented | Legion+ WMO visibility |
| **Unknown Chunks** | ✅ Handled | Preserved but not parsed |
| **Version Detection** | ✅ Implemented | Detects based on chunks |
| **File Validation** | ✅ Implemented | Integrity checks |

### 🔨 File Writing Operations

| Feature | Status | Notes |
|---------|--------|-------|
| **Header Writing** | ✅ Implemented | All chunk types |
| **Offset Calculation** | ✅ Implemented | Automatic MAOF generation |
| **Chunk Ordering** | ✅ Implemented | Correct chunk sequence |
| **Version-Specific Writing** | ✅ Implemented | Conditional chunk inclusion |
| **Data Validation** | ✅ Implemented | Pre-write validation |
| **Round-Trip Support** | ✅ Implemented | Read → Write → Read |
| **Memory Efficiency** | ✅ Implemented | Streaming writes |

### 🔄 Version Support

| Version | Status | Chunks Supported | Notes |
|---------|--------|-----------------|-------|
| **Classic (1.12.1)** | ✅ Supported | MVER, MAOF, MARE | Version 18 |
| **TBC (2.4.3)** | ✅ Supported | + MAHO | Version 18 |
| **WotLK (3.3.5a)** | ✅ Supported | + MWMO, MWID, MODF | Version 18 |
| **Cataclysm (4.3.4)** | ✅ Supported | Same as WotLK | Version 18 |
| **MoP (5.4.8)** | ✅ Supported | Same as WotLK | Version 18 |
| **Legion+ (7.0+)** | ✅ Supported | + ML** chunks | Version 18 |

### 📊 Data Structures

| Structure | Status | Notes |
|-----------|--------|-------|
| **Vec3d** | ✅ Implemented | 3D vector type |
| **BoundingBox** | ✅ Implemented | Min/max corners |
| **HeightMapTile** | ✅ Implemented | 545 height values |
| **HolesData** | ✅ Implemented | 16x16 bitmask |
| **ModelPlacement** | ✅ Implemented | WMO placement info |
| **M2Placement** | ✅ Implemented | Legion+ model data |
| **M2VisibilityInfo** | ✅ Implemented | Legion+ visibility |
| **Chunk** | ✅ Implemented | Generic chunk container |

### 🔍 Validation & Error Handling

| Feature | Status | Notes |
|---------|--------|-------|
| **Version Validation** | ✅ Implemented | Supported versions |
| **Chunk Size Validation** | ✅ Implemented | Data integrity |
| **Map Tile Validation** | ✅ Implemented | Offset/data matching |
| **Coordinate Validation** | ✅ Implemented | 0-63 bounds checking |
| **Cross-Reference Validation** | ⚠️ Partial | WMO index validation |
| **Data Range Validation** | ❌ Not Implemented | Height value ranges |
| **Error Recovery** | ⚠️ Basic | Error handling |

### 🚀 Advanced Features

| Feature | Status | Notes |
|---------|--------|-------|
| **Height Interpolation** | ❌ Planned | Bilinear interpolation |
| **Coordinate Conversion** | ❌ Planned | World ↔ WDL coords |
| **Normal Calculation** | ❌ Planned | Terrain normals |
| **Minimap Generation** | ❌ Planned | Image export |
| **LoD Integration** | ❌ Planned | ADT/WDL switching |
| **Streaming API** | ❌ Not Implemented | Large file support |
| **Memory Mapping** | ❌ Not Implemented | Performance optimization |
| **Async I/O** | ❌ Not Implemented | Non-blocking operations |

### 🧪 Testing & Quality

| Test Category | Status | Notes |
|---------------|--------|-------|
| **Unit Tests** | ✅ Available | Core functions |
| **Integration Tests** | ✅ Available | Round-trip tests |
| **Parser Tests** | ✅ Available | Chunk parsing |
| **Version Tests** | ✅ Available | WoW versions |
| **Validation Tests** | ✅ Available | Error conditions |
| **Benchmark Tests** | ✅ Available | Performance tests |
| **Example Code** | ✅ Available | 2 working examples |
| **Documentation** | ✅ Available | API docs |

## Critical Gaps Analysis

### 1. Advanced Terrain Features

**Impact:** No terrain features like height queries and normal generation.

**Missing Features:**

- Height interpolation at arbitrary coordinates
- Gradient/normal calculation for lighting
- Coordinate system conversion (world ↔ tile ↔ chunk)
- Integration with ADT high-resolution data

### 2. Undocumented Chunks

**Impact:** Some WDL files may contain additional chunks.

**Known Unknowns:**

- MSSN, MSSC, MSSO chunks mentioned in some sources
- Version-specific variations
- Game-specific extensions

### 3. Performance Optimizations

**Impact:** Large continent files may have performance issues.

**Missing Optimizations:**

- Memory-mapped file support
- Lazy loading of chunks
- Parallel processing
- Caching strategies

## Implementation

1. Documented chunks implemented
2. WoW version support with detection
3. Types and methods for WDL manipulation
4. Handles malformed files
5. Test coverage
6. API documentation with examples

## Path to Implementation Completion

### Phase 1: Advanced Features

1. **Coordinate System**
   - Implement world ↔ WDL coordinate conversion
   - Add chunk/tile coordinate helpers

2. **Height Interpolation**
   - Bilinear interpolation for smooth heights
   - Gradient calculation for normals

3. **Data Export**
   - Minimap/heightmap image generation
   - Terrain mesh export

### Phase 2: Performance

1. **I/O Optimizations**
   - Memory-mapped file support
   - Streaming API for large files

2. **Processing Optimizations**
   - Parallel chunk processing
   - Lazy evaluation

### Phase 3: Advanced Integration

1. **LoD System**
   - ADT/WDL transition helpers
   - Unified terrain API

2. **Extended Validation**
   - Cross-file validation
   - Data range checks

## Summary

The `wow-wdl` crate supports reading and writing WDL files for World of Warcraft versions.

Gaps include coordinate conversion and height interpolation needed for game engine implementation.

Current implementation handles WDL file inspection, modification, and conversion.
