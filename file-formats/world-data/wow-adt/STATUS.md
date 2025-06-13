# ADT Implementation Status

## Overview

The `wow-adt` crate provides parsing, validation, and manipulation for World of Warcraft ADT (terrain) files.

## Implementation Progress

### Core Features

| Feature | Status | Notes |
|---------|--------|-------|
| Basic ADT parsing | ✅ Implemented | All chunk types supported |
| Version detection | ✅ Implemented | Detects based on chunk presence |
| Chunk validation | ✅ Implemented | Multi-level validation support |
| Write support | ✅ Implemented | Write capabilities |
| Error handling | ✅ Implemented | Error types |
| Tree visualization | ✅ Implemented | Integrated with warcraft-rs CLI |

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
| Classic (1.x) | ✅ | ✅ | ✅ | Supported |
| TBC (2.x) | ✅ | ✅ | ✅ | MFBO chunk added |
| WotLK (3.x) | ✅ | ✅ | ✅ | MH2O water system |
| Cataclysm (4.x) | ✅ | ✅ | ✅ | MTFX, split files |
| MoP (5.x) | ⚠️ | ⚠️ | ⚠️ | Basic support, needs testing |

### Advanced Features

| Feature | Status | Notes |
|---------|--------|-------|
| Split file support | ✅ Implemented | _tex0,_obj0, etc. |
| Streaming API | ✅ Implemented | Streaming parser |
| Batch processing | ✅ Implemented | With parallel feature |
| Heightmap extraction | ✅ Implemented | Multiple formats |
| Texture extraction | ✅ Implemented | Texture references |
| Model extraction | ✅ Implemented | Placement data |
| Normal map generation | ✅ Implemented | From heightmap data |
| 3D export | 🚧 Partial | OBJ export works, PLY/STL not implemented |

## CLI Integration

ADT commands in warcraft-rs CLI:

- `adt info` - Display ADT file information
- `adt validate` - Validate ADT files with configurable strictness
- `adt convert` - Convert between WoW versions
- `adt tree` - Visualize ADT structure
- `adt extract` - Extract data (with extract feature)
- `adt batch` - Batch processing (with parallel feature)

## Known Limitations

1. **MH2O Write Support** - Basic implementation, complex water configurations may not be preserved
2. **MoP+ Support** - Versions beyond Cataclysm need testing
3. **Texture Blending** - Alpha map decompression for compressed formats not implemented
4. **Terrain Holes** - Hole detection works, editing support limited

## Performance

- **Parse Time**: Varies by file complexity
- **Memory Usage**: Scales with ADT content
- **Batch Processing**: Parallel processing available

## Testing

- Unit tests for all chunk types
- Integration tests with WoW ADT files
- Round-trip tests (read → write → read)
- Cross-version conversion tests
- Validation suite with strictness levels

## Future Improvements

1. MoP+ version support
2. Water editing
3. Texture blending visualization
4. Heightmap editor integration
5. MPQ archive support for batch operations
