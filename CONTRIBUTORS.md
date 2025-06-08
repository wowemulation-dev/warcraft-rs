# Contributors

Thank you to everyone who has contributed to the `warcraft-rs` project!

## Project Lead

- **Daniel S. Reichenbach** ([@danielsreichenbach](https://github.com/danielsreichenbach)) - Project creator and maintainer

## Core Contributors

*This section will be updated as the project grows and receives contributions.*

## How to Contribute

We welcome contributions from the community! Here are some ways you can help:

### Code Contributions

1. **Fork the repository** and create your feature branch (`git checkout -b feature/amazing-feature`)
2. **Make your changes** following the Rust style guidelines
3. **Add tests** for any new functionality
4. **Ensure all tests pass** (`cargo test --all-features`)
5. **Run quality checks**:
   ```bash
   cargo fmt --all
   cargo check --all-features --all-targets
   cargo clippy --all-targets --all-features
   cargo test
   ```
6. **Update documentation** if you're changing public APIs
7. **Commit your changes** with descriptive commit messages
8. **Push to your branch** and open a Pull Request

### Other Ways to Contribute

- **Report bugs**: Open an issue describing the problem with reproduction steps
- **Suggest features**: Open an issue with your enhancement proposal
- **Improve documentation**: Help make our docs clearer and more comprehensive
- **Add examples**: Create examples showing different use cases
- **Performance improvements**: Profile and optimize the code
- **Test with real game files**: Verify functionality with actual WoW data files

### Areas Where Help is Needed

Here are specific areas where contributions would be especially valuable:

#### 🔧 MPQ Archive Format (wow-mpq)

1. **StormLib Feature Parity**
   - Automatic encryption key detection (see [StormLib differences guide](docs/guides/stormlib-differences.md))
   - Content-based key recovery for encrypted files
   - LZMA compression support
   - Sparse file compression (RLE algorithm)
   - Weak signature verification
   - Game-specific file type detection (AVI files, PE headers)

2. **Performance Optimizations**
   - Memory-mapped I/O support for large archives
   - LRU sector caching implementation
   - Bit-packed data structures for HET/BET tables
   - Custom allocators for large table operations

3. **Archive Protection Handling**
   - BOBA protector support (negative table offsets)
   - w3xMaster protector workarounds
   - Malformed archive recovery mechanisms
   - Starcraft Beta special case handling

#### 🎨 Graphics Formats

1. **BLP Texture Format (wow-blp)**
   - Full format parser implementation
   - Mipmap support
   - Conversion to/from standard image formats (PNG, JPEG)
   - Direct3D/OpenGL texture loading helpers

2. **M2 Model Format (wow-m2)**
   - Complete format parser for all versions
   - Animation system (.anim files)
   - Skin/mesh data (.skin files)
   - Physics data (.phys files)
   - Bone data (.bone files)
   - Skeleton data (.skel files)

3. **WMO World Objects (wow-wmo)**
   - Full format specification implementation
   - Group file support
   - Portal and visibility system
   - Liquid rendering support
   - Doodad placement

#### 🗺️ World Data Formats

1. **ADT Terrain (wow-adt)**
   - Complete chunk parser implementation
   - Height map extraction
   - Texture layer support
   - Water/liquid data
   - Shadow maps
   - Area ID mapping

2. **WDT World Tables (wow-wdt)**
   - Full format parser
   - ADT existence flags
   - Map bounds calculation
   - MPHD flags support

3. **WDL Low-Res Maps (wow-wdl)**
   - Format specification implementation
   - Low-resolution height data
   - Area table support

#### 📊 Database Format

1. **DBC Client Database (wow-dbc)**
   - Generic DBC parser framework
   - Schema definition system
   - Common DBC file implementations:
     - Item.dbc
     - Spell.dbc
     - Map.dbc
     - AreaTable.dbc
   - String block handling
   - Localization support

#### 🛠️ Tooling and CLI

1. **Enhanced CLI Tools**
   - Interactive mode for MPQ manipulation
   - Batch processing capabilities
   - Progress bars for long operations
   - JSON/YAML output formats
   - Shell completion scripts

2. **Integration Tools**
   - Unity/Unreal Engine plugins
   - Blender import/export scripts
   - 3ds Max support
   - Web-based viewers

#### 📚 Documentation and Examples

1. **Format Documentation**
   - Detailed binary format specifications
   - Version differences documentation
   - Visual diagrams of data structures
   - Format evolution history

2. **Usage Examples**
   - Complete game asset extraction pipeline
   - Model viewer implementation
   - Map renderer example
   - Asset conversion workflows

3. **Tutorials**
   - "Building a WoW model viewer" series
   - "Extracting and using WoW textures"
   - "Understanding WoW's coordinate system"
   - "Working with WoW's patch system"

#### 🧪 Testing and Quality

1. **Test Coverage**
   - Increase test coverage to >90%
   - Fuzzing harnesses for format parsers
   - Cross-version compatibility tests
   - Performance benchmarks

2. **Real-World Testing**
   - Test with corrupted/malformed files
   - Verify against all WoW versions (1.12.1 - 5.4.8)
   - Cross-platform testing (Windows, macOS, Linux)
   - Big-endian platform support

### Development Guidelines

- **Code Style**: Follow Rust idioms and conventions
- **Documentation**: Document public APIs with examples
- **Testing**: Write tests for new functionality
- **Performance**: Profile before optimizing
- **Compatibility**: Support WoW versions 1.12.1 through 5.4.8
- **Safety**: Prefer safe Rust, document and isolate unsafe code

### Getting Started with Contributing

1. **Check existing issues** for something you'd like to work on
2. **Comment on the issue** to let others know you're working on it
3. **Ask questions** if you need clarification
4. **Start small** - documentation fixes and small features are great first contributions
5. **Join the discussion** in issues and pull requests

### Recognition

All contributors will be recognized in this file. Significant contributions may also be highlighted in:
- Release notes
- Project README
- Documentation credits

## License

By contributing to this project, you agree that your contributions will be licensed under the same terms as the project (MIT OR Apache-2.0).

## Code of Conduct

Please note that this project follows our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Contact

- Open an issue for questions or discussions
- For security concerns, please see [SECURITY.md](SECURITY.md)

---

*Want to see your name here? We'd love to have your contribution! Check the issues labeled "good first issue" to get started.*