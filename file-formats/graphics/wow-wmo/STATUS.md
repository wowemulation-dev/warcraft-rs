# wow-wmo - WMO Format Implementation Status

**Last Updated:** 2025-06-11

The `wow-wmo` crate provides WMO (World Map Object) support:

- **File Parsing**: Root and group files, all major chunks
- **File Writing**: Binary serialization for all formats
- **Version Support**: v17 Classic through v27 The War Within
- **Validation**: Validation with warnings/errors
- **Conversion**: Version conversion between expansions
- **Export Features**: OBJ/MTL export implemented
- **Testing**: Unit tests, benchmarks, examples

## Detailed Feature Matrix

### 📖 WMO Parsing Operations

| Feature | Status | Notes |
|---------|--------|-------|
| **Root File Parsing** | ✅ Implemented | All chunks supported |
| **Group File Parsing** | ✅ Implemented | Geometry support |
| **Version Detection** | ✅ Implemented | Version identification |
| **Chunk Framework** | ✅ Implemented | Chunk reading/writing |
| **Material Parsing** | ✅ Implemented | Material properties |
| **Doodad Support** | ✅ Implemented | Doodad definitions and sets |
| **Portal Support** | ✅ Implemented | Portal vertices and relationships |
| **Light Support** | ✅ Implemented | Light types and properties |
| **Liquid Support** | ✅ Implemented | Water/lava/slime rendering data |
| **BSP Tree Support** | ✅ Implemented | Collision and visibility data |
| **Texture Names** | ✅ Implemented | Texture path references |
| **Model Names** | ✅ Implemented | M2/MDX model references |

### 🔨 WMO Writing Operations

| Feature | Status | Notes |
|---------|--------|-------|
| **Root File Writing** | ✅ Implemented | All chunks supported |
| **Group File Writing** | ✅ Implemented | Geometry serialization |
| **Chunk Ordering** | ✅ Implemented | Correct chunk sequence |
| **Data Validation** | ✅ Implemented | Pre-write validation |
| **Format Compliance** | ✅ Implemented | Blizzard-compatible output |
| **Compression** | ❌ Not Implemented | No MCLY compression yet |

### 🔄 Version Support

| Expansion | Version | Status | Notes |
|-----------|---------|--------|-------|
| **Classic** | v17 | ✅ Supported | 1.12.x |
| **TBC** | v17 | ✅ Supported | 2.x.x |
| **WotLK** | v17 | ✅ Supported | 3.x.x |
| **Cataclysm** | v17 | ✅ Supported | 4.x.x |
| **MoP** | v17 | ✅ Supported | 5.x.x |
| **WoD** | v18 | ✅ Supported | 6.x.x |
| **Legion** | v20-21 | ✅ Supported | 7.x.x |
| **BfA** | v22 | ✅ Supported | 8.x.x |
| **Shadowlands** | v23-24 | ✅ Supported | 9.x.x |
| **Dragonflight** | v25-26 | ✅ Supported | 10.x.x |
| **The War Within** | v27 | ✅ Supported | 11.x.x |

### 🔧 Advanced Features

| Feature | Status | Notes |
|---------|--------|-------|
| **Version Conversion** | ✅ Implemented | Convert between any versions |
| **Validation System** | ✅ Implemented | Multi-level validation |
| **Builder API** | ✅ Implemented | Programmatic WMO creation |
| **Editor Support** | ⚠️ Partial | Editing capabilities |
| **Visualizer** | ⚠️ Basic | Debug visualization only |
| **Export to OBJ** | ✅ Implemented | Wavefront OBJ format |
| **Export to glTF** | ❌ Planned | Modern 3D format |
| **LOD Generation** | ❌ Not Implemented | LOD creation |
| **Lightmap Generation** | ❌ Not Implemented | Baked lighting |

### 📊 Chunk Support Status

#### Root File Chunks

| Chunk | Read | Write | Notes |
|-------|------|-------|-------|
| MVER | ✅ | ✅ | Version |
| MOHD | ✅ | ✅ | Header |
| MOTX | ✅ | ✅ | Textures |
| MOMT | ✅ | ✅ | Materials |
| MOGN | ✅ | ✅ | Group names |
| MOGI | ✅ | ✅ | Group info |
| MOSB | ✅ | ✅ | Skybox |
| MOPV | ✅ | ✅ | Portal vertices |
| MOPT | ✅ | ✅ | Portal info |
| MOPR | ✅ | ✅ | Portal refs |
| MOVV | ✅ | ✅ | Visible vertices |
| MOVB | ✅ | ✅ | Visible blocks |
| MOLT | ✅ | ✅ | Lights |
| MODS | ✅ | ✅ | Doodad sets |
| MODN | ✅ | ✅ | Doodad names |
| MODD | ✅ | ✅ | Doodad definitions |
| MFOG | ✅ | ✅ | Fog |
| MCVP | ✅ | ✅ | Convex volume planes |

#### Group File Chunks

| Chunk | Read | Write | Notes |
|-------|------|-------|-------|
| MVER | ✅ | ✅ | Version |
| MOGP | ✅ | ✅ | Group header |
| MOPY | ✅ | ✅ | Material info |
| MOVI | ✅ | ✅ | Vertex indices |
| MOVT | ✅ | ✅ | Vertices |
| MONR | ✅ | ✅ | Normals |
| MOTV | ✅ | ✅ | Texture coords |
| MOBA | ✅ | ✅ | Render batches |
| MOLR | ✅ | ✅ | Light refs |
| MODR | ✅ | ✅ | Doodad refs |
| MOBN | ✅ | ✅ | BSP nodes |
| MOBR | ✅ | ✅ | BSP face indices |
| MLIQ | ✅ | ✅ | Liquids |
| MOCV | ✅ | ✅ | Vertex colors |
| MORI | ✅ | ✅ | Triangle strips (legacy) |

### 🧪 Testing & Quality

| Test Category | Status | Notes |
|---------------|--------|-------|
| **Unit Tests** | ✅ Available | Major components |
| **Integration Tests** | ✅ Available | File round-trip tests |
| **Parser Tests** | ✅ Available | Chunk types |
| **Validation Tests** | ✅ Available | Error conditions |
| **Benchmark Tests** | ✅ Available | Performance metrics |
| **Example Code** | ✅ Available | Multiple examples |
| **Documentation** | ✅ Available | API docs + guides |

### 🛠️ CLI Integration

WMO commands in warcraft-rs CLI:

- `wmo info` - Display WMO information
- `wmo validate` - Validate WMO files
- `wmo convert` - Convert between versions
- `wmo tree` - Visualize WMO structure
- `wmo edit` - Basic editing operations
- `wmo build` - Create WMO from config

## Known Limitations

1. **MCLY Compression** - Cataclysm+ alpha layer compression not implemented
2. **LOD Generation** - No automatic level-of-detail creation
3. **Lightmap Baking** - No light baking support
4. **Advanced Editing** - Limited to basic flag/property changes
5. **Physics Data** - No Havok physics export

## Performance

- Parse Time: Varies with WMO complexity
- Memory Usage: Scales with vertex count
- Write Performance: Similar to parse time
- Validation Speed: Fast for small files

## Recent Improvements

1. Version 27 Support for The War Within (11.x.x)
2. OBJ/MTL export with materials
3. Builder API for WMO creation
4. Multi-level validation
5. CLI tree visualization

## Future Improvements

1. glTF Export - 3D format with PBR
2. MCLY Compression - Cataclysm+ alpha
3. Geometry/material editing
4. Level-of-detail generation
5. Collision mesh export

## Summary

The `wow-wmo` crate supports World of Warcraft's WMO format
across game versions. Parsing, writing, and conversion work.
Gaps include compression and editing features.
