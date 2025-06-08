# BLP Format 🎨

BLP (Blizzard Picture) is Blizzard's proprietary texture format used for all
textures in World of Warcraft.

## Overview

- **Extension**: `.blp`
- **Purpose**: Compressed texture storage
- **Versions**: BLP1 (legacy), BLP2 (current)
- **Compression**: JPEG, DXT1, DXT3, DXT5, uncompressed
- **Features**: Mipmaps, alpha channels, palettized textures

## Structure

### BLP Header

```rust
struct BlpHeader {
    magic: [u8; 4],          // "BLP2"
    version: u32,            // Always 1 for BLP2
    compression: u8,         // 0=JPEG, 1=Palette, 2=DXT, 3=Uncompressed
    alpha_depth: u8,         // 0, 1, 4, or 8
    alpha_compression: u8,   // 0=DXT1, 1=DXT3, 7=DXT5
    mipmap_level: u8,        // 0 = no mipmap, 1 = has mipmaps
    width: u32,              // Texture width
    height: u32,             // Texture height
    mipmap_offsets: [u32; 16], // File offsets to mipmap levels
    mipmap_sizes: [u32; 16],   // Sizes of mipmap levels
}

enum Compression {
    Jpeg = 0,
    Palettized = 1,
    Dxt = 2,
    Uncompressed = 3,
}
```

### Palette (BLP1)

```rust
struct BlpPalette {
    colors: [Rgba; 256],  // 256 RGBA colors
}
```

## Usage Example

```rust
use warcraft_rs::blp::{Blp, ImageFormat};

// Load BLP texture
let blp = Blp::open("Textures/Models/Armor/Armor.blp")?;

// Get texture information
println!("Size: {}x{}", blp.width(), blp.height());
println!("Compression: {:?}", blp.compression());
println!("Mipmap levels: {}", blp.mipmap_count());

// Convert to standard format
let image = blp.to_rgba8()?;
image.save("armor_texture.png")?;

// Access specific mipmap level
let mipmap_2 = blp.get_mipmap(2)?;
println!("Mipmap 2 size: {}x{}", mipmap_2.width, mipmap_2.height);

// Create BLP from image
let new_blp = Blp::from_image(&image, Compression::Dxt5)?;
new_blp.save("new_texture.blp")?;
```

## Compression Types

### DXT Compression

Most common for modern textures:

```rust
use warcraft_rs::blp::DxtFormat;

match blp.dxt_format() {
    Some(DxtFormat::Dxt1) => {
        // 4:1 compression, 1-bit alpha
        println!("DXT1: Good for opaque textures");
    }
    Some(DxtFormat::Dxt3) => {
        // 4:1 compression, 4-bit explicit alpha
        println!("DXT3: Good for sharp alpha transitions");
    }
    Some(DxtFormat::Dxt5) => {
        // 4:1 compression, interpolated alpha
        println!("DXT5: Good for smooth alpha gradients");
    }
    None => println!("Not DXT compressed"),
}
```

### Palettized (BLP1)

Legacy format with 256-color palette:

```rust
if let Some(palette) = blp.get_palette() {
    // Convert palettized to RGBA
    let rgba_data = blp.depalettize()?;
}
```

## Advanced Features

### Mipmap Generation

```rust
use warcraft_rs::blp::MipmapGenerator;

let generator = MipmapGenerator::new()
    .filter(FilterType::Lanczos3)
    .max_level(10);

let blp_with_mipmaps = generator.generate(&original_blp)?;
```

### Alpha Channel Manipulation

```rust
// Extract alpha channel
let alpha_mask = blp.extract_alpha_channel()?;

// Replace alpha channel
let new_blp = blp.with_alpha_channel(&new_alpha)?;

// Check if texture has transparency
if blp.has_alpha() {
    println!("Texture uses transparency");
}
```

### Batch Processing

```rust
use warcraft_rs::blp::BatchProcessor;

let processor = BatchProcessor::new()
    .output_format(ImageFormat::Png)
    .resize(512, 512)
    .compression(Compression::Dxt5);

// Convert all BLPs in directory
processor.process_directory("Textures/", "Output/")?;
```

## Common Patterns

### Texture Atlas Creation

```rust
use warcraft_rs::blp::AtlasBuilder;

let mut atlas = AtlasBuilder::new(2048, 2048);

// Add textures to atlas
let uv1 = atlas.add_texture("icon1.blp")?;
let uv2 = atlas.add_texture("icon2.blp")?;

// Build final atlas
let atlas_blp = atlas.build(Compression::Dxt5)?;
atlas_blp.save("texture_atlas.blp")?;

// Save UV mappings
atlas.save_mappings("atlas_uvs.json")?;
```

### Icon Extraction

```rust
fn extract_spell_icons() -> Result<()> {
    let mpq = Archive::open("Interface.mpq")?;

    for entry in mpq.list_files() {
        if entry.name.starts_with("Interface\\Icons\\")
            && entry.name.ends_with(".blp") {

            let blp_data = mpq.read_file(&entry.name)?;
            let blp = Blp::from_bytes(&blp_data)?;

            let icon_name = Path::new(&entry.name)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap();

            let image = blp.to_rgba8()?;
            image.save(format!("icons/{}.png", icon_name))?;
        }
    }
    Ok(())
}
```

## Performance Tips

- DXT textures can be uploaded directly to GPU
- Keep original BLP for best quality
- Use appropriate compression for content type
- Generate mipmaps for 3D textures

## Common Issues

### Power of Two Requirement

- Texture dimensions must be powers of 2
- Common sizes: 256x256, 512x512, 1024x1024
- Non-power-of-2 textures need padding

### Alpha Blending

- DXT1 only supports 1-bit alpha
- Use DXT3/DXT5 for smooth transparency
- Check `alpha_depth` field for precision

### Color Banding

- JPEG compression can cause artifacts
- DXT compression can cause block artifacts
- Use uncompressed for high quality needs

## References

- [BLP Format (wowdev.wiki)](https://wowdev.wiki/BLP)
- [DXT Compression Guide](https://docs.microsoft.com/en-us/windows/win32/direct3d11/texture-block-compression)

## See Also

- [Texture Loading Guide](../../guides/texture-loading.md)
- [M2 Format](m2.md) - Uses BLP textures
- [WMO Format](wmo.md) - Uses BLP textures
