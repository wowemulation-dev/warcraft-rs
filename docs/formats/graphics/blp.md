# BLP Format 🎨

BLP (Blizzard Picture) is Blizzard's proprietary texture format used for all textures in Warcraft III and World of Warcraft.

## Overview

- **Extension**: `.blp`
- **Purpose**: Compressed texture storage
- **Versions**: BLP0 (Warcraft III Beta), BLP1 (Warcraft III), BLP2 (World of Warcraft)
- **Compression**: JPEG, RAW1 (palettized), RAW3 (uncompressed), DXT1/3/5
- **Features**: Mipmaps, alpha channels, multiple compression options

## Structure

### BLP Header

```rust
struct BlpHeader {
    magic: [u8; 4],          // "BLP0", "BLP1", or "BLP2"
    version: BlpVersion,     // Format version
    content: BlpContentTag,  // JPEG or Direct
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
    Jpeg,   // JPEG compressed
    Direct, // Direct pixel data (RAW1/3, DXT)
}
```

### Compression Types

```rust
enum Compression {
    Jpeg, // JPEG with BGRA color space
    Raw1, // 256-color palettized
    Raw3, // Uncompressed BGRA
    Dxtc, // DXT1/3/5 compression
}
```

## Usage Example

```rust
use wow_blp::{parser::load_blp, convert::blp_to_image, encode::save_blp};
use wow_blp::convert::{image_to_blp, BlpTarget, Blp2Format, DxtAlgorithm, FilterType};

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

```rust
// BLP0 saves mipmaps as separate files
let blp0_target = BlpTarget::Blp0(BlpOldFormat::Jpeg { has_alpha: true });
```

### BLP1 (Warcraft III)

- Internal mipmaps
- JPEG and RAW1 compression

```rust
let blp1_target = BlpTarget::Blp1(BlpOldFormat::Raw1 { 
    alpha_bits: AlphaBits::Bit1 
});
```

### BLP2 (World of Warcraft)

- All compression types supported
- Internal mipmaps
- Most flexible format

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

// Get mipmap count
let count = blp.header.mipmaps_count();

// External mipmap paths (BLP0)
use wow_blp::path::make_mipmap_path;
let mip_path = make_mipmap_path("texture.blp", 3)?; // texture.b03
```

### Alpha Channel Support

```rust
use wow_blp::convert::AlphaBits;

// Different alpha bit depths
AlphaBits::NoAlpha  // No alpha channel
AlphaBits::Bit1     // 1-bit (on/off)
AlphaBits::Bit4     // 4-bit (16 levels)
AlphaBits::Bit8     // 8-bit (full alpha)
```

### Batch Processing

```rust
use std::fs;
use std::path::Path;

fn convert_directory(input_dir: &str, output_dir: &str) -> Result<()> {
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

fn extract_spell_icons() -> Result<()> {
    let mut archive = Archive::open("Interface.mpq")?;
    
    for file in archive.list_files() {
        if file.starts_with("Interface\\Icons\\") && file.ends_with(".blp") {
            let data = archive.read_file(&file)?;
            let blp = wow_blp::parser::parse_blp(&data)?.1;
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
fn create_game_texture(input: &str, output: &str) -> Result<()> {
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

### Power of Two Requirement

- Texture dimensions should be powers of 2 for optimal GPU performance
- Common sizes: 256x256, 512x512, 1024x1024
- Maximum size: 65535x65535 (BLP format limit)

### Color Space

- BLP uses BGRA color order, not RGBA
- JPEG compression in BLP uses BGRA, not standard YCbCr

### Compression Artifacts

- JPEG can cause color bleeding
- DXT can cause block artifacts on gradients
- Use RAW3 (uncompressed) for highest quality

## References

- [BLP Format (wowdev.wiki)](https://wowdev.wiki/BLP)
- [DXT Compression](https://docs.microsoft.com/en-us/windows/win32/direct3d11/texture-block-compression)
- Original image-blp crate documentation

## See Also

- [Texture Loading Guide](../../guides/texture-loading.md)
- [M2 Format](m2.md) - Uses BLP textures
- [WMO Format](wmo.md) - Uses BLP textures