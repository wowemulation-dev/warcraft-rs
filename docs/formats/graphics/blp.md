# BLP Format 🎨

BLP (Blizzard Picture) is Blizzard's proprietary texture format used for all textures in Warcraft III and World of Warcraft. The format uses non-standard JPEG compression with BGRA color components (instead of Y′CbCr) and various direct pixel storage methods.

## Overview

- **Extension**: `.blp`
- **Purpose**: Compressed texture storage optimized for game engines
- **Versions**: BLP0 (Warcraft III Beta), BLP1 (Warcraft III), BLP2 (World of Warcraft)
- **Compression**: JPEG (non-standard BGRA), RAW1 (palettized), RAW3 (uncompressed BGRA), DXT1/3/5 (S3TC)
- **Features**: Up to 16 mipmaps, alpha channels with variable bit depth, GPU-friendly formats
- **Endianness**: Little-endian for all multi-byte values

## File Structure

### Header Layout

The header structure varies by version:

**BLP0/BLP1 Header (148 bytes)**:
```
Offset  Size  Description
0x00    4     Magic: "BLP0" or "BLP1"
0x04    4     Content type (0=JPEG, 1=Direct)
0x08    4     Alpha bits (0, 1, 4, or 8)
0x0C    4     Width
0x10    4     Height  
0x14    4     Extra field (4 for RAW1, 5 for JPEG)
0x18    4     Has mipmaps (0 or 1)
0x1C    -     No mipmap tables for BLP0 (external mipmaps)
0x1C    128   Mipmap tables for BLP1 (16 offsets + 16 sizes)
```

**BLP2 Header (156 bytes)**:
```
Offset  Size  Description
0x00    4     Magic: "BLP2"
0x04    4     Content type (0=JPEG, 1=Direct)
0x08    1     Compression (0=JPEG, 1=RAW1, 2=DXTC, 3=RAW3)
0x09    1     Alpha bits (0, 1, 4, or 8)
0x0A    1     Alpha type (0, 1, 7, or 8)
0x0B    1     Has mipmaps (0 or 1)
0x0C    4     Width (max 65535)
0x10    4     Height (max 65535)
0x14    64    Mipmap offsets (16 x u32)
0x54    64    Mipmap sizes (16 x u32)
```

### Data Layout

```rust
struct BlpHeader {
    magic: [u8; 4],          // "BLP0", "BLP1", or "BLP2"
    version: BlpVersion,     // Format version
    content: BlpContentTag,  // JPEG (0) or Direct (1)
    flags: BlpFlags,         // Version-specific flags
    width: u32,              // Texture width
    height: u32,             // Texture height
    mipmap_locator: MipmapLocator, // Internal or external mipmaps
}

enum BlpVersion {
    Blp0, // Warcraft III Beta
    Blp1, // Warcraft III  
    Blp2, // World of Warcraft
}

enum BlpContentTag {
    Jpeg = 0,   // JPEG compressed (non-standard BGRA)
    Direct = 1, // Direct pixel data (RAW1/3, DXT)
}

// Version-specific flags
enum BlpFlags {
    // BLP0/BLP1
    Old {
        alpha_bits: u32,    // 0, 1, 4, or 8
        extra: u32,         // 4 for RAW1, 5 for JPEG
        has_mipmaps: u32,   // 0 or 1
    },
    // BLP2
    Blp2 {
        compression: Compression, // See Compression enum
        alpha_bits: u8,          // 0, 1, 4, or 8
        alpha_type: u8,          // Usually 0, affects blending
        has_mipmaps: u8,         // 0 or 1
    }
}
```

### Compression Types (BLP2 only)

```rust
enum Compression {
    Jpeg = 0, // JPEG (rarely/never used in BLP2 files)
    Raw1 = 1, // 256-color palettized
    Dxtc = 2, // DXT1/3/5 compression (S3TC)
    Raw3 = 3, // Uncompressed BGRA
}
```

### Additional Data Sections

**For JPEG content**:
- 4 bytes: JPEG header size (actual size - 2 due to a bug)
- Variable: JPEG header data
- Image data: JPEG compressed mipmaps

**For RAW1 (palettized)**:
- 1024 bytes: Color palette (256 x BGRA, 4 bytes per color)
- Image data: 
  - 8-bit palette indices (1 byte per pixel)
  - Alpha data (format depends on alpha_bits):
    - 0 bits: No alpha data
    - 1 bit: Packed 8 pixels per byte
    - 4 bits: Packed 2 pixels per byte
    - 8 bits: 1 byte per pixel

**For DXT**:
- 1024 bytes: Unused color map (zeroed)
- Image data: DXT compressed blocks

**For RAW3**:
- Image data: Raw BGRA pixels (4 bytes per pixel)

### Complete File Layout Example (BLP2 DXT5)

```
Offset  Size    Description
0x00    4       Magic "BLP2"
0x04    4       Content type (1 for Direct)
0x08    1       Compression (2 for DXTC)  
0x09    1       Alpha bits (8 for DXT5)
0x0A    1       Alpha type (0)
0x0B    1       Has mipmaps (1)
0x0C    4       Width (e.g., 512)
0x10    4       Height (e.g., 512)
0x14    64      Mipmap offsets [16 x u32]
0x54    64      Mipmap sizes [16 x u32]
0x94    1024    Unused color map (all zeros for DXT)
0x494   varies  Mipmap 0: DXT5 compressed data
...     ...     Additional mipmaps
```

## Usage Example

```rust
use wow_blp::{parser::load_blp, convert::blp_to_image, encode::save_blp};
use wow_blp::convert::{image_to_blp, BlpTarget, Blp2Format, DxtAlgorithm};
use image::imageops::FilterType;

// Load BLP texture
let blp = load_blp("texture.blp")?;

// Get texture information
println!("Size: {}x{}", blp.header.width, blp.header.height);
println!("Version: {:?}", blp.header.version);
println!("Has mipmaps: {}", blp.header.has_mipmaps());

// Convert to standard format
let image = blp_to_image(&blp, 0)?; // mipmap level 0
image.save("texture.png")?;

// Create BLP from image
let input = image::open("input.png")?;
let new_blp = image_to_blp(
    input,
    true, // generate mipmaps
    BlpTarget::Blp2(Blp2Format::Dxt5 {
        has_alpha: true,
        compress_algorithm: DxtAlgorithm::ClusterFit
    }),
    FilterType::Lanczos3
)?;
save_blp(&new_blp, "output.blp")?;
```

## Compression Types

### DXT Compression (BLP2)

Most common for modern textures:

```rust
use wow_blp::convert::{Blp2Format, DxtAlgorithm};

// DXT1: 4:1 compression, 1-bit alpha
let dxt1 = Blp2Format::Dxt1 {
    has_alpha: false,
    compress_algorithm: DxtAlgorithm::RangeFit // Fast
};

// DXT3: 4:1 compression, 4-bit explicit alpha
let dxt3 = Blp2Format::Dxt3 {
    has_alpha: true,
    compress_algorithm: DxtAlgorithm::ClusterFit // Quality
};

// DXT5: 4:1 compression, interpolated alpha
let dxt5 = Blp2Format::Dxt5 {
    has_alpha: true,
    compress_algorithm: DxtAlgorithm::IterativeClusterFit // Best
};
```

### Palettized (RAW1)

256-color palette format:

```rust
use wow_blp::convert::{BlpOldFormat, AlphaBits};

let palettized = BlpOldFormat::Raw1 {
    alpha_bits: AlphaBits::Bit8  // 0, 1, 4, or 8 bits
};
```

### Uncompressed (RAW3)

Full BGRA format (BLP2 only):

```rust
let uncompressed = Blp2Format::Raw3;
```

## Version-Specific Features

### BLP0 (Warcraft III Beta)

- External mipmaps in .b00-.b15 files
- Limited to JPEG and RAW1 compression
- Header size: 28 bytes (no mipmap tables)
- Mipmap files use format: `basename.b##` where ## is 00-15

```rust
// BLP0 saves mipmaps as separate files
let blp0_target = BlpTarget::Blp0(BlpOldFormat::Jpeg { has_alpha: true });
```

### BLP1 (Warcraft III)

- Internal mipmaps with offset/size tables
- JPEG and RAW1 compression
- Header size: 156 bytes (includes mipmap tables)
- Maximum 16 mipmap levels

```rust
let blp1_target = BlpTarget::Blp1(BlpOldFormat::Raw1 {
    alpha_bits: AlphaBits::Bit1
});
```

### BLP2 (World of Warcraft)

- All compression types supported (though JPEG is rarely used)
- Internal mipmaps with offset/size tables
- Header size: 156 bytes
- Additional alpha_type field for advanced blending
- DXT compression uses texpresso library

```rust
let blp2_target = BlpTarget::Blp2(Blp2Format::Dxt5 {
    has_alpha: true,
    compress_algorithm: DxtAlgorithm::ClusterFit
});
```

## Advanced Features

### Mipmap Handling

```rust
// Access specific mipmap level
let mipmap_2 = blp_to_image(&blp, 2)?;

// Get mipmap count (calculated as max(log2(width), log2(height)))
let count = blp.header.mipmaps_count();

// Mipmap dimensions (each level halves size, minimum 1x1)
let (width, height) = blp.header.mipmap_size(level);

// External mipmap paths (BLP0 only)
use wow_blp::path::make_mipmap_path;
let mip_path = make_mipmap_path("texture.blp", 3)?; // texture.b03
```

### Alpha Channel Support

```rust
use wow_blp::convert::AlphaBits;

// Different alpha bit depths
AlphaBits::NoAlpha  // No alpha channel (0 bits)
AlphaBits::Bit1     // 1-bit (on/off transparency)
AlphaBits::Bit4     // 4-bit (16 transparency levels)
AlphaBits::Bit8     // 8-bit (256 transparency levels)
```

#### Alpha Storage by Format
- **JPEG**: Alpha stored as separate grayscale image after RGB data
- **RAW1**: Alpha bits packed after palette indices
  - 1-bit: 8 pixels per byte
  - 4-bit: 2 pixels per byte  
  - 8-bit: 1 pixel per byte
- **DXT1**: 1-bit alpha encoded in color endpoints
- **DXT3**: 4-bit alpha stored explicitly before color data
- **DXT5**: Alpha endpoints + 3-bit interpolation indices
- **RAW3**: Alpha interleaved as BGRA pixels

### Batch Processing

```rust
use std::fs;
use std::path::Path;
use wow_blp::{parser::load_blp, convert::blp_to_image};

fn convert_directory(input_dir: &str, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension() == Some("blp".as_ref()) {
            let blp = load_blp(&path)?;
            let image = blp_to_image(&blp, 0)?;

            let output_path = Path::new(output_dir)
                .join(path.file_stem().unwrap())
                .with_extension("png");

            image.save(output_path)?;
        }
    }
    Ok(())
}
```

## Common Patterns

### Icon Extraction from MPQ

```rust
use wow_mpq::Archive;
use wow_blp::{parser::parse_blp, convert::blp_to_image};
use std::path::Path;

fn extract_spell_icons() -> Result<(), Box<dyn std::error::Error>> {
    let mut archive = Archive::open("Interface.mpq")?;

    for file in archive.list_files() {
        if file.starts_with("Interface\\Icons\\") && file.ends_with(".blp") {
            let data = archive.read_file(&file)?;
            let blp = parse_blp(&data)?.1;
            let image = blp_to_image(&blp, 0)?;

            let icon_name = Path::new(&file)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap();

            image.save(format!("icons/{}.png", icon_name))?;
        }
    }
    Ok(())
}
```

### Creating Game-Ready Textures

```rust
use wow_blp::{convert::{image_to_blp, BlpTarget, Blp2Format, DxtAlgorithm}, encode::save_blp};
use image::imageops::FilterType;

fn create_game_texture(input: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut img = image::open(input)?;

    // Ensure power-of-two dimensions
    let width = img.width().next_power_of_two();
    let height = img.height().next_power_of_two();

    if width != img.width() || height != img.height() {
        img = img.resize_exact(width, height, FilterType::Lanczos3);
    }

    // Convert to BLP with appropriate settings
    let blp = image_to_blp(
        img,
        true, // mipmaps for 3D use
        BlpTarget::Blp2(Blp2Format::Dxt5 {
            has_alpha: true,
            compress_algorithm: DxtAlgorithm::ClusterFit
        }),
        FilterType::Lanczos3
    )?;

    save_blp(&blp, output)?;
    Ok(())
}
```

## Performance Tips

- DXT textures can be uploaded directly to GPU without decompression
- RAW1 (palettized) provides excellent compression for textures with limited colors
- Use DXT1 for opaque textures to save memory
- Use DXT5 for textures with smooth alpha gradients
- Generate mipmaps for 3D textures to improve rendering performance

## Common Issues

### Technical Limitations

#### Dimension Requirements
- Texture dimensions should be powers of 2 for optimal GPU performance
- Common sizes: 256x256, 512x512, 1024x1024
- Maximum size: 65535x65535 (defined as BLP_MAX_WIDTH/HEIGHT constants)
- Mipmap count: max(log2(width), log2(height))

#### Format-Specific Details
- JPEG header has a 2-byte discrepancy (stored length = actual length - 2)
- DXT formats include a 1024-byte color map that's always zeroed
- RAW1 alpha data is stored separately after the indexed color data
- Alpha type field in BLP2 affects blending (usually 0 for standard alpha)

### Color Space and Encoding

- All formats use BGRA color order (Blue, Green, Red, Alpha)
- JPEG uses non-standard JFIF compression:
  - Compresses raw BGRA values directly
  - Does NOT use standard Y′CbCr color space conversion
  - This is why BLP JPEG files are incompatible with standard JPEG readers
- DXT compression is applied to BGRA data
- RAW formats store pixels in BGRA order

### Compression Characteristics

#### JPEG (BLP0/BLP1, rarely BLP2)
- Non-standard BGRA compression
- Can cause color bleeding at block boundaries
- Alpha stored as separate channel

#### RAW1 (Palettized)
- Limited to 256 colors
- Excellent for textures with limited color palettes
- Alpha precision depends on bit depth (0/1/4/8 bits)

#### DXT (BLP2)
- 4:1 compression ratio (DXT1) or 6:1 (DXT3/5)
- 4x4 pixel block artifacts on gradients
- DXT1: 1-bit alpha or opaque
- DXT3: 4-bit explicit alpha per pixel
- DXT5: Interpolated alpha (best for smooth gradients)
- Hardware accelerated on GPUs

#### RAW3 (BLP2)
- Uncompressed BGRA
- Highest quality, largest file size
- No compression artifacts

## References

- [BLP Format (wowdev.wiki)](https://wowdev.wiki/BLP)
- [DXT Compression](https://docs.microsoft.com/en-us/windows/win32/direct3d11/texture-block-compression)
- Original image-blp crate documentation

## See Also

- [Texture Loading Guide](../../guides/texture-loading.md)
- [M2 Format](m2.md) - Uses BLP textures
- [WMO Format](wmo.md) - Uses BLP textures
