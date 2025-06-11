# M2 Format Implementation Status

## Overview

The `wow-m2` crate provides comprehensive support for parsing, converting, and editing World of Warcraft M2 model files across all major game versions from Classic (1.12.1) through The War Within (11.0+).

## Current Status: 🟢 Production Ready

### ✅ Implemented Features

#### Core Functionality

- [x] **M2 Model Parsing**: Full support for all M2 versions (256-274)
- [x] **Version Detection**: Automatic version detection from file headers
- [x] **Version Conversion**: Convert models between different WoW expansions
- [x] **Validation**: Comprehensive model validation with error/warning reporting
- [x] **Binary I/O**: Efficient reading and writing of M2 files

#### Supported Chunks

- [x] **Header**: Magic, version, model info, bounding boxes
- [x] **Geometry**: Vertices, normals, UV coordinates, vertex colors
- [x] **Bones**: Full skeletal hierarchy with transformations
- [x] **Animations**: Animation sequences with bone tracks
- [x] **Textures**: Texture definitions and references
- [x] **Materials**: Material properties and render flags
- [x] **Attachments**: Attachment points for weapons, effects
- [x] **Events**: Animation events and timings
- [x] **Lights**: Dynamic light sources
- [x] **Cameras**: Camera positions and animations
- [x] **Particles**: Particle emitter definitions
- [x] **Ribbons**: Ribbon trail emitters
- [x] **Collision**: Collision volumes and physics data

#### Related File Formats

- [x] **Skin Files (.skin)**: LOD and submesh data
- [x] **Animation Files (.anim)**: External animation data
- [x] **BLP Textures**: Basic BLP texture info display

#### CLI Features

- [x] **Info Command**: Display model information
- [x] **Convert Command**: Convert between versions
- [x] **Validate Command**: Check model integrity
- [x] **Tree Command**: Visualize model structure
- [x] **Skin Commands**: Info and conversion for .skin files
- [x] **Anim Commands**: Info and conversion for .anim files
- [x] **BLP Info**: Display texture information

### 🚧 Known Limitations

1. **Physics Data**:
   - Basic physics chunk support (PHYS)
   - Advanced cloth simulation not fully implemented

2. **Version-Specific Features**:
   - Some Legion+ features may have limited support
   - Shadowlands/Dragonflight additions are basic

3. **Optimization**:
   - Large model files (>50MB) may be slow to process
   - Memory usage not optimized for batch processing

### 📊 Version Support Matrix

| WoW Version | M2 Version | Read | Write | Convert |
|-------------|------------|------|-------|---------|
| Classic 1.x | 256-264    | ✅   | ✅    | ✅      |
| TBC 2.x     | 264        | ✅   | ✅    | ✅      |
| WotLK 3.x   | 264        | ✅   | ✅    | ✅      |
| Cata 4.x    | 272        | ✅   | ✅    | ✅      |
| MoP 5.x     | 273        | ✅   | ✅    | ✅      |
| WoD 6.x     | 274        | ✅   | ✅    | ✅      |
| Legion 7.x  | 274        | ✅   | ✅    | ✅      |
| BfA 8.x     | 274        | ✅   | ✅    | ✅      |
| SL 9.x      | 274        | ✅   | ⚠️    | ⚠️      |
| DF 10.x     | 274        | ✅   | ⚠️    | ⚠️      |
| TWW 11.x    | 274        | ✅   | ⚠️    | ⚠️      |

⚠️ = Basic support, some features may be missing

### 🔄 Version Conversion Notes

- **Upconversion** (older → newer): Generally safe, adds default values for new features
- **Downconversion** (newer → older): May lose data for features not supported in older versions
- **Lossless Conversions**: Between adjacent versions (e.g., Classic ↔ TBC)
- **Lossy Conversions**: Large version jumps may lose advanced features

## Architecture

### Design Principles

1. **Type Safety**: Strongly typed structures for all M2 components
2. **Zero-Copy Parsing**: Where possible, avoid unnecessary allocations
3. **Version Agnostic Core**: Common structures with version-specific handling
4. **Modular Chunks**: Each chunk type is independently parseable

### Key Components

- `model.rs`: Main M2Model structure and high-level API
- `header.rs`: File header parsing and version detection
- `chunks/`: Individual chunk parsers (bone, texture, animation, etc.)
- `converter.rs`: Version conversion logic
- `version.rs`: Version enumeration and compatibility
- `skin.rs`: Skin file (LOD) handling
- `anim.rs`: External animation file support

### Performance Characteristics

- **Parse Time**: ~10-50ms for typical models (1-10MB)
- **Memory Usage**: ~2-3x file size during parsing
- **Write Time**: Similar to parse time
- **Conversion Time**: ~1.5x parse time

## Usage Examples

### Basic Model Loading

```rust
use wow_m2::M2Model;

let model = M2Model::load("Character/Human/Male/HumanMale.m2")?;
println!("Model version: {:?}", model.header.version());
println!("Vertices: {}", model.vertices.len());
```

### Version Conversion

```rust
use wow_m2::{M2Model, M2Converter, M2Version};

let model = M2Model::load("old_model.m2")?;
let converter = M2Converter::new();
let converted = converter.convert(&model, M2Version::WotLK)?;
converted.save("new_model.m2")?;
```

### Model Validation

```rust
use wow_m2::M2Model;

let model = M2Model::load("model.m2")?;
let errors = model.validate();
if !errors.is_empty() {
    println!("Validation failed: {:?}", errors);
}
```

## Future Enhancements

1. **Advanced Physics**: Full cloth/soft-body simulation support
2. **Batch Processing**: Optimized APIs for processing many models
3. **Streaming Parser**: Support for parsing models larger than memory
4. **Model Editing**: High-level API for modifying model properties
5. **Export Formats**: Export to common 3D formats (OBJ, FBX, glTF)
6. **Animation Blending**: Tools for merging/blending animations
7. **Texture Management**: Automatic BLP texture loading/conversion

## Contributing

When adding new features:

1. Ensure version compatibility is maintained
2. Add tests for each supported version
3. Update this STATUS.md with implementation status
4. Follow existing patterns for chunk parsing
5. Document version-specific behavior

## References

- [WoWDev.wiki M2 Format](https://wowdev.wiki/M2)
- [WoW Model Viewer Source](https://github.com/Marlamin/WoWModelViewer)
- [010 Editor M2 Templates](https://github.com/CucFlavius/Zee-010-Templates)
