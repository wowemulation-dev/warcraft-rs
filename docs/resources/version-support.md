# 🔢 Version Support

World of Warcraft client version compatibility and file format changes.

## Supported Client Versions

| Expansion | Version | Patch | Build | Status |
|-----------|---------|-------|-------|--------|
| Classic (Vanilla) | 1.12.1 | 1.12.1.5875 | 5875 | ✅ Full Support |
| The Burning Crusade | 2.4.3 | 2.4.3.8606 | 8606 | ✅ Full Support |
| Wrath of the Lich King | 3.3.5 | 3.3.5a.12340 | 12340 | ✅ Full Support |
| Cataclysm | 4.3.4 | 4.3.4.15595 | 15595 | ✅ Full Support |
| Mists of Pandaria | 5.4.8 | 5.4.8.18414 | 18414 | ✅ Full Support |

## File Format Versions

### MPQ Archives

| Version | Client | Changes | wow-mpq Support |
|---------|--------|---------|-----------------|
| v1 | 1.x - 3.x | Original format, hash table, block table | ✅ Full |
| v2 | 3.x+ | Extended attributes, larger files | ✅ Full |
| v3 | 4.x+ | HET/BET tables, increased hash table size | ✅ Full |
| v4 | 5.x+ | 64-bit file support, MD5 checksums | ✅ Full |

**Note:** wow-mpq has 98.75% bidirectional compatibility with StormLib (the reference C++ implementation) and full support for all official Blizzard WoW archives.

### M2 Models

| Version | Client | Major Changes |
|---------|--------|---------------|
| 256-257 | 1.x | Original format |
| 260-263 | 2.x | Particle emitters update |
| 264 | 3.0+ | .skin/.anim file separation |
| 272 | 3.3+ | Extended animations |
| 273 | 4.0+ | .phys physics data |
| 274 | 4.x+ | New texture types |
| 276 | 5.x+ | Improved bone structure |

### ADT Terrain

| Version | Client | Changes |
|---------|--------|---------|
| 18 | 1.x - 2.x | Original MCNK format |
| 20 | 3.x | Destructible doodads |
| 21 | 4.x | Terrain streaming, flight |
| 23 | 5.x | New texture blending |

### BLP Textures

| Version | Client | Format Support |
|---------|--------|----------------|
| BLP1 | 1.x - 2.x | JPEG compression, palettized |
| BLP2 | 3.x+ | DXT compression, mipmaps |

### DBC Database

| Client | Records | String Encoding | Features |
|--------|---------|-----------------|----------|
| 1.x | Fixed size | ASCII | Basic structure |
| 2.x | Fixed size | UTF-8 | Extended fields |
| 3.x | Fixed size | UTF-8 | Localization support |
| 4.x | Fixed size | UTF-8 | New index format |
| 5.x | Fixed size | UTF-8 | Compressed strings |

## Version Detection

### File Magic Numbers

```rust
// MPQ detection
const MPQ_MAGIC: &[u8; 4] = b"MPQ\x1A";

// M2 version detection
fn detect_m2_version(data: &[u8]) -> Result<u32, Error> {
    if data.len() < 8 {
        return Err(Error::InvalidFormat("File too small"));
    }

    let magic = &data[0..4];
    if magic != b"MD20" && magic != b"MD21" {
        return Err(Error::InvalidMagic {
            expected: *b"MD20",
            found: magic.try_into().unwrap(),
        });
    }

    let version = u32::from_le_bytes(data[4..8].try_into().unwrap());
    Ok(version)
}

// BLP version detection
fn detect_blp_version(data: &[u8]) -> BlpVersion {
    match &data[0..4] {
        b"BLP1" => BlpVersion::Blp1,
        b"BLP2" => BlpVersion::Blp2,
        _ => BlpVersion::Unknown,
    }
}
```

### Client Version Detection

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientVersion {
    Classic,
    TBC,
    WotLK,
    Cataclysm,
    MoP,
}

impl ClientVersion {
    /// Detect from file characteristics
    pub fn detect_from_m2(version: u32) -> Option<Self> {
        match version {
            256..=257 => Some(ClientVersion::Classic),
            260..=263 => Some(ClientVersion::TBC),
            264..=272 => Some(ClientVersion::WotLK),
            273..=275 => Some(ClientVersion::Cataclysm),
            276..=278 => Some(ClientVersion::MoP),
            _ => None,
        }
    }

    /// Get build number range
    pub fn build_range(&self) -> (u32, u32) {
        match self {
            ClientVersion::Classic => (0, 6005),
            ClientVersion::TBC => (6006, 8606),
            ClientVersion::WotLK => (8607, 12340),
            ClientVersion::Cataclysm => (12341, 15595),
            ClientVersion::MoP => (15596, 18414),
        }
    }
}
```

## Format Compatibility

### Reading Older Formats

```rust
/// Version-aware model loader
pub struct ModelLoader {
    version: ClientVersion,
}

impl ModelLoader {
    pub fn load(&self, path: &str) -> Result<Model, Error> {
        let data = std::fs::read(path)?;
        let version = detect_m2_version(&data)?;

        match version {
            256..=263 => self.load_legacy_m2(&data),
            264..=278 => self.load_modern_m2(&data),
            _ => Err(Error::UnsupportedVersion {
                format: "M2".to_string(),
                version,
                supported: vec![256, 257, 260, 263, 264, 272, 273, 276],
            }),
        }
    }

    fn load_legacy_m2(&self, data: &[u8]) -> Result<Model, Error> {
        // Handle embedded skins and animations
        let header = LegacyM2Header::parse(data)?;
        // Convert to modern format
        Ok(header.to_modern_model())
    }

    fn load_modern_m2(&self, data: &[u8]) -> Result<Model, Error> {
        // Handle external .skin/.anim files
        let header = ModernM2Header::parse(data)?;
        Ok(Model::from_header(header))
    }
}
```

### Feature Availability

```rust
/// Check feature support by version
pub trait VersionedFeature {
    fn is_supported(&self, version: ClientVersion) -> bool;
}

pub enum ModelFeature {
    ExternalAnimations,
    PhysicsData,
    ExtendedTextures,
    SharedSkeletons,
}

impl VersionedFeature for ModelFeature {
    fn is_supported(&self, version: ClientVersion) -> bool {
        match self {
            ModelFeature::ExternalAnimations => version >= ClientVersion::WotLK,
            ModelFeature::PhysicsData => version >= ClientVersion::Cataclysm,
            ModelFeature::ExtendedTextures => version >= ClientVersion::Cataclysm,
            ModelFeature::SharedSkeletons => false, // Not in supported versions
        }
    }
}
```

## Migration Guide

### Upgrading File Formats

```rust
/// Convert between format versions
pub trait FormatConverter {
    type Input;
    type Output;

    fn convert(&self, input: Self::Input) -> Result<Self::Output, Error>;
}

/// Convert BLP1 to BLP2
pub struct Blp1ToBlp2Converter;

impl FormatConverter for Blp1ToBlp2Converter {
    type Input = Blp1Texture;
    type Output = Blp2Texture;

    fn convert(&self, input: Self::Input) -> Result<Self::Output, Error> {
        let mut output = Blp2Texture::new(input.width(), input.height());

        // Convert compression
        match input.compression() {
            Blp1Compression::Jpeg => {
                let rgba = input.decompress_jpeg()?;
                output.compress_dxt1(&rgba)?;
            }
            Blp1Compression::Palettized => {
                let rgba = input.decode_palette()?;
                output.set_uncompressed(rgba);
            }
        }

        // Generate mipmaps
        output.generate_mipmaps();

        Ok(output)
    }
}
```

### Handling Missing Features

```rust
/// Gracefully handle version differences
pub struct VersionAdapter {
    version: ClientVersion,
}

impl VersionAdapter {
    pub fn load_model_animations(&self, model: &mut Model, path: &str) -> Result<(), Error> {
        if ModelFeature::ExternalAnimations.is_supported(self.version) {
            // Load from .anim files
            let anim_pattern = format!("{}-*.anim", path.trim_end_matches(".m2"));
            for anim_file in glob::glob(&anim_pattern)? {
                let anim = Animation::load(anim_file?)?;
                model.add_external_animation(anim);
            }
        } else {
            // Animations are embedded in M2
            println!("Using embedded animations for {}", path);
        }
        Ok(())
    }
}
```

## Version-Specific Behavior

### Coordinate Systems

```rust
/// Handle coordinate system changes
pub fn convert_coordinates(pos: Vec3, from: ClientVersion, to: ClientVersion) -> Vec3 {
    // ADT coordinate system changed in Cataclysm
    if from < ClientVersion::Cataclysm && to >= ClientVersion::Cataclysm {
        // Apply transformation
        Vec3 {
            x: pos.x,
            y: -pos.z,  // Y/Z swap
            z: pos.y,
        }
    } else {
        pos
    }
}
```

### String Encoding

```rust
/// Handle string encoding differences
pub fn decode_string(data: &[u8], version: ClientVersion) -> Result<String, Error> {
    match version {
        ClientVersion::Classic => {
            // ASCII only
            String::from_utf8(data.to_vec())
                .map_err(|e| Error::StringDecoding(e))
        }
        _ => {
            // UTF-8 support
            String::from_utf8(data.to_vec())
                .map_err(|e| Error::StringDecoding(e))
        }
    }
}
```

## Testing Across Versions

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_detection() {
        let test_cases = vec![
            (b"MD20\x00\x01\x00\x00", 256, ClientVersion::Classic),
            (b"MD20\x08\x01\x00\x00", 264, ClientVersion::WotLK),
            (b"MD20\x14\x01\x00\x00", 276, ClientVersion::MoP),
        ];

        for (data, expected_version, expected_client) in test_cases {
            let version = detect_m2_version(data).unwrap();
            assert_eq!(version, expected_version);

            let client = ClientVersion::detect_from_m2(version).unwrap();
            assert_eq!(client, expected_client);
        }
    }
}
```

## Best Practices

1. **Always check versions** before parsing format-specific features
2. **Provide fallbacks** for missing features in older versions
3. **Test with multiple client versions** to ensure compatibility
4. **Document version requirements** in your API
5. **Use version adapters** to abstract differences
6. **Log version information** for debugging

## See Also

- [File Format Reference](../formats/README.md)
- [Migration Guide](../guides/migration.md)
- [Testing Guide](../guides/testing.md)
