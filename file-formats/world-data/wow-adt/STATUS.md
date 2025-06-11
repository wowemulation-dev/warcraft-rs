# ADT Implementation Status

## Overview

The `wow-adt` crate provides comprehensive parsing, validation, and manipulation support for World of Warcraft ADT (terrain) files across multiple game versions.

## Implementation Progress

### Core Features

| Feature | Status | Notes |
|---------|--------|-------|
| Basic ADT parsing | ✅ Complete | All chunk types supported |
| Version detection | ✅ Complete | Auto-detects based on chunk presence |
| Chunk validation | ✅ Complete | Multi-level validation support |
| Write support | ✅ Complete | Full write capabilities |
| Error handling | ✅ Complete | Comprehensive error types |
| Tree visualization | ✅ Complete | Integrated with warcraft-rs CLI |

### Chunk Support

| Chunk | Read | Write | Notes |
|-------|------|-------|-------|
| MVER | ✅ | ✅ | Version information |
| MHDR | ✅ | ✅ | Header with offsets |
| MCIN | ✅ | ✅ | Chunk index |
| MTEX | ✅ | ✅ | Texture filenames |
| MMDX | ✅ | ✅ | M2 model filenames |
| MMID | ✅ | ✅ | M2 model indices |
| MWMO | ✅ | ✅ | WMO filenames |
| MWID | ✅ | ✅ | WMO indices |
| MDDF | ✅ | ✅ | Doodad placements |
| MODF | ✅ | ✅ | WMO placements |
| MCNK | ✅ | ✅ | Terrain chunks (256 per file) |
| MFBO | ✅ | ✅ | Flight bounds (TBC+) |
| MH2O | ✅ | ⚠️ | Water data (WotLK+, basic write) |
| MTFX | ✅ | ✅ | Texture effects (Cata+) |

### MCNK Subchunks

| Subchunk | Read | Write | Notes |
|----------|------|-------|-------|
| MCVT | ✅ | ✅ | Height values |
| MCNR | ✅ | ✅ | Normal vectors |
| MCLY | ✅ | ✅ | Texture layers |
| MCRF | ✅ | ✅ | Doodad references |
| MCRD | ✅ | ✅ | WMO references |
| MCSH | ✅ | ✅ | Shadow map |
| MCAL | ✅ | ✅ | Alpha maps |
| MCLQ | ✅ | ✅ | Legacy liquid (pre-WotLK) |
| MCCV | ✅ | ✅ | Vertex colors |

### Version Support

| Version | Parsing | Writing | Conversion | Notes |
|---------|---------|---------|------------|-------|
| Classic (1.x) | ✅ | ✅ | ✅ | Full support |
| TBC (2.x) | ✅ | ✅ | ✅ | MFBO chunk added |
| WotLK (3.x) | ✅ | ✅ | ✅ | MH2O water system |
| Cataclysm (4.x) | ✅ | ✅ | ✅ | MTFX, split files |
| MoP (5.x) | ⚠️ | ⚠️ | ⚠️ | Basic support, needs testing |

### Advanced Features

| Feature | Status | Notes |
|---------|--------|-------|
| Split file support | ✅ Complete | _tex0,_obj0, etc. |
| Streaming API | ✅ Complete | Memory-efficient parsing |
| Batch processing | ✅ Complete | With parallel feature |
| Heightmap extraction | ✅ Complete | Multiple formats |
| Texture extraction | ✅ Complete | Reference extraction |
| Model extraction | ✅ Complete | Placement data |
| Normal map generation | ✅ Complete | From heightmap data |
| 3D export | 🚧 Partial | Basic mesh export |

## CLI Integration

The ADT functionality is fully integrated into the warcraft-rs CLI with the following commands:

- `adt info` - Display ADT file information
- `adt validate` - Validate ADT files with configurable strictness
- `adt convert` - Convert between WoW versions
- `adt tree` - Visualize ADT structure
- `adt extract` - Extract data (with extract feature)
- `adt batch` - Batch processing (with parallel feature)

## Known Limitations

1. **MH2O Write Support** - Basic implementation, complex water configurations may not be fully preserved
2. **MoP+ Support** - Versions beyond Cataclysm have basic support but need more testing
3. **Texture Blending** - Alpha map decompression for compressed formats not fully implemented
4. **Terrain Holes** - Hole detection works but editing support is limited

## Performance

- **Parse Time**: ~5-50ms per ADT file (depending on complexity)
- **Memory Usage**: ~5-20MB per loaded ADT
- **Batch Processing**: Can process 100+ files/second with parallel feature

## Testing

- Unit tests for all chunk types
- Integration tests with real WoW ADT files
- Round-trip tests (read → write → read)
- Cross-version conversion tests
- Validation suite with multiple strictness levels

## Future Improvements

1. Enhanced MoP+ version support
2. Advanced water editing capabilities
3. Texture blending visualization
4. Integration with heightmap editors
5. Direct MPQ archive support for batch operations
