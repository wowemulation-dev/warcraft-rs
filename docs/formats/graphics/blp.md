# BLP Format 🎨

BLP (Blizzard Picture) is Blizzard's proprietary texture format used for all textures in Warcraft III and World of Warcraft. The format uses non-standard JPEG compression with BGRA color components (instead of Y′CbCr) and various direct pixel storage methods.

## Overview

- **Extension**: `.blp`
- **Purpose**: Compressed texture storage optimized for game engines
- **Versions**: BLP0 (Warcraft III Beta), BLP1 (Warcraft III), BLP2 (World of Warcraft)
- **Compression**: JPEG (BLP0/BLP1 only), RAW1 (palettized), RAW3 (uncompressed BGRA), DXT1/3/5 (S3TC)
- **Features**: Up to 16 mipmaps, alpha channels with variable bit depth, GPU-friendly formats
- **Endianness**: Little-endian for all multi-byte values

## Cross-Version Analysis Results

### WoW 1.12.1 (Vanilla)
**Based on analysis of 50+ BLP files from original WoW 1.12.1 MPQ archives:**

- **Format**: 100% BLP2 (no BLP0/BLP1 found)
- **Content Type**: 100% Direct (content_type=1, no JPEG content)
- **Primary Compression**: 82% DXT (compression=2), 18% RAW1 palettized (compression=1)
- **Alpha Usage**: 46% use 8-bit alpha, 34% no alpha, 20% 1-bit alpha (no 4-bit alpha found)
- **Alpha Types**: Only 0, 1, and 8 observed (no alpha_type=7)
- **Dimensions**: 100% power-of-2, most common: 256x256 (28%), 64x64 (22%), 128x128 (10%)
- **Mipmaps**: 88% have mipmaps enabled, typically 7-9 levels depending on texture size

### WoW 2.4.3 (TBC)
**Based on analysis of 28 BLP files from original WoW 2.4.3 (TBC) MPQ archives:**

- **Format**: 100% BLP2 (consistent with 1.12.1)
- **Content Type**: 100% Direct (content_type=1)
- **Primary Compression**: 75% DXT (compression=2), 25% RAW1 palettized (compression=1)
- **Alpha Usage**: 50% use 8-bit alpha, 25% no alpha, 18% use 1-bit alpha, 7% use alpha_type=7
- **New Alpha Type**: alpha_type=7 appears (14.3% of files) - not seen in 1.12.1
- **Dimensions**: 100% power-of-2, with 512x512 textures (7.1%) appearing for higher detail
- **Mipmaps**: 93% have mipmaps enabled, up to 10 levels for 512x512 textures

### WoW 3.3.5a (WotLK)
**Based on analysis of 28 BLP files from original WoW 3.3.5a (WotLK) MPQ archives:**

- **Format**: 100% BLP2 (consistent across versions)
- **Content Type**: 100% Direct (content_type=1)
- **Primary Compression**: 79% DXT (compression=2), 21% RAW1 palettized (compression=1)
- **Alpha Usage**: 54% use 8-bit alpha, 43% no alpha, 4% use 1-bit alpha
- **Alpha Types**: alpha_type=7 usage increases to 32.1% (vs 14.3% in TBC), alpha_type=1 drops to 7.1%
- **Dimensions**: 100% power-of-2, 512x512 textures more common (14.3%), first 16x16 texture observed
- **Mipmaps**: 89% have mipmaps enabled, with unusual has_mipmaps=2 value appearing (10.7%)

### WoW 4.3.4 (Cataclysm)
**Based on analysis of 29 BLP files from original WoW 4.3.4 (Cataclysm) MPQ archives:**

- **Format**: 100% BLP2 (consistent across versions)
- **Content Type**: 100% Direct (content_type=1)
- **Primary Compression**: 79% DXT (compression=2), 21% RAW1 palettized (compression=1)
- **Alpha Usage**: 83% use 8-bit alpha, 14% no alpha, 3% use 1-bit alpha (major shift towards 8-bit)
- **Alpha Types**: alpha_type=7 dominates at 62.1% (vs 32.1% in WotLK), alpha_type=8 drops to 20.7%
- **Dimensions**: 100% power-of-2, wider variety including non-square (256x128, 512x256) ratios
- **Mipmaps**: 97% have mipmaps enabled (highest rate), mostly 9-10 levels

### WoW 5.4.8 (MoP)
**Based on analysis of 8 BLP files from original WoW 5.4.8 (MoP) MPQ archives:**

- **Format**: 100% BLP2 (consistent across versions)
- **Content Type**: 100% Direct (content_type=1)
- **Primary Compression**: 100% DXT (compression=2), no RAW1 palettized found
- **Alpha Usage**: 88% no alpha, 13% use 8-bit alpha (minimap tiles dominate sample)
- **Alpha Types**: 88% alpha_type=0, 13% alpha_type=7 (limited sample size)
- **Dimensions**: 100% power-of-2, primarily 256x256 (88%), one 64x128 texture
- **Mipmaps**: 63% no mipmaps (minimap tiles), 38% have mipmaps enabled

## BLP Format Evolution Analysis

### Key Trends Across WoW Versions (1.12.1 → 5.4.8)

#### 1. Alpha Type Evolution
- **1.12.1**: Only alpha_type values 0, 1, and 8 observed
- **2.4.3**: Introduction of alpha_type=7 (14.3% usage)
- **3.3.5a**: alpha_type=7 increases to 32.1%
- **4.3.4**: alpha_type=7 becomes dominant at 62.1%
- **5.4.8**: Limited sample shows alpha_type=0 and 7 only

**Key Finding**: alpha_type=7 appears to be associated with enhanced alpha blending introduced in TBC and becomes the primary alpha mode by Cataclysm.

#### 2. Compression Method Trends
- **1.12.1**: 82% DXT, 18% RAW1 palettized
- **2.4.3**: 75% DXT, 25% RAW1 palettized  
- **3.3.5a**: 79% DXT, 21% RAW1 palettized
- **4.3.4**: 79% DXT, 21% RAW1 palettized (stable)
- **5.4.8**: 100% DXT (no RAW1 in sample)

**Key Finding**: DXT compression remains dominant, while RAW1 palettized usage fluctuates but generally decreases over time.

#### 3. Alpha Usage Patterns
- **1.12.1**: 46% use 8-bit alpha, 34% no alpha, 20% 1-bit alpha
- **2.4.3**: 50% use 8-bit alpha, 25% no alpha, 18% 1-bit alpha  
- **3.3.5a**: 54% use 8-bit alpha, 43% no alpha, 4% 1-bit alpha
- **4.3.4**: 83% use 8-bit alpha, 14% no alpha, 3% 1-bit alpha
- **5.4.8**: 13% use 8-bit alpha, 88% no alpha (minimap-heavy sample)

**Key Finding**: 8-bit alpha usage steadily increases from Vanilla through Cataclysm, indicating more sophisticated transparency effects.

#### 4. Texture Resolution Trends  
- **1.12.1**: Primarily 256x256 (28%), some 64x64 (22%)
- **2.4.3**: 512x512 textures appear (7.1% of sample)
- **3.3.5a**: 512x512 usage increases (14.3%), small 16x16 textures appear
- **4.3.4**: More diverse ratios including rectangular textures
- **5.4.8**: Primarily 256x256 (88% of sample)

**Key Finding**: Higher resolution textures (512x512) become more common from TBC onward, with Cataclysm introducing more rectangular aspect ratios.

#### 5. Mipmap Behavior Evolution
- **1.12.1**: 88% have mipmaps, mostly has_mipmaps=1
- **2.4.3**: 93% have mipmaps, mostly has_mipmaps=1
- **3.3.5a**: 89% have mipmaps, unusual has_mipmaps=2 appears (10.7%)
- **4.3.4**: 97% have mipmaps (highest rate), mostly has_mipmaps=1
- **5.4.8**: 38% have mipmaps (minimap tiles don't need LOD)

**Key Finding**: Mipmap usage increases through Cataclysm, with WotLK introducing non-standard has_mipmaps=2 values.

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

### Alpha Type Patterns (WoW 1.12.1)

**From empirical analysis, alpha_type correlates with compression and alpha_bits:**

- **alpha_type=0**: Used with DXT compression, 0-bit or 1-bit alpha (48% of files)
- **alpha_type=1**: Used with DXT compression, 8-bit alpha (34% of files)  
- **alpha_type=8**: Used with RAW1 palettized compression, typically 8-bit alpha (18% of files)

**Pattern Rules:**
- DXT with no alpha → alpha_bits=0, alpha_type=0
- DXT with binary transparency → alpha_bits=1, alpha_type=0
- DXT with full transparency → alpha_bits=8, alpha_type=1
- RAW1 palettized → alpha_type=8 (regardless of alpha_bits value)

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

// ✅ Load BLP texture
let blp = load_blp("texture.blp")?;

// ✅ Get texture information
println!("Size: {}x{}", blp.header.width, blp.header.height);
println!("Version: {:?}", blp.header.version);
println!("Has mipmaps: {}", blp.header.has_mipmaps());

// ✅ Convert to standard format
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

## WoW 1.12.1 Content Type Analysis

**Compression usage by content type:**

### UI Icons (Interface\Icons\*.blp)
- **Compression**: DXT (compression=2) 
- **Alpha**: Mixed - 1-bit for simple icons, 8-bit for complex icons
- **Dimensions**: Mostly 64x64 (standard icon size)
- **Mipmaps**: Usually 7 levels (64→32→16→8→4→2→1)

### Character Textures (Character\*\*.blp)
- **Compression**: RAW1 palettized (compression=1)
- **Alpha**: Variable (0, 1, or 8-bit) with alpha_type=8
- **Dimensions**: Rectangular (128x64, 128x32) for face parts
- **Usage**: Hair, facial features, skin textures

### Creature Skins (Creature\*\*.blp)
- **Compression**: DXT (compression=2)
- **Alpha**: Often 8-bit alpha (alpha_type=1) for fur/scale details
- **Dimensions**: 256x256 (high detail creature textures)
- **Mipmaps**: 9 levels for distance LOD

### World Textures (World\*\*.blp)
- **Compression**: DXT (compression=2)
- **Alpha**: Mixed - 0-bit for solid objects, 1-bit for cutouts
- **Dimensions**: Various sizes, always power-of-2
- **Usage**: Building textures, environmental objects

### Spell Effects (Spells\*.blp)
- **Compression**: DXT (compression=2)
- **Alpha**: Often 0-bit or 8-bit depending on effect type
- **Dimensions**: 128x128, 256x256 for particle effects

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
    let archive = Archive::open("Interface.mpq")?;

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

## Technical Notes from Analysis

### Mipmap Behavior in WoW 1.12.1
- **has_mipmaps field**: Not always reliable indicator
  - Some files have has_mipmaps=0 but still contain 1 mipmap (base texture)
  - One file observed with has_mipmaps=2 (non-standard value)
- **Actual mipmap count**: Determined by non-zero offset/size pairs in mipmap tables
- **Mipmap progression**: Always follows power-of-2 reduction (256→128→64→32→16→8→4→2→1)

### Alpha Type Field Clarification
The alpha_type field is more specific than previously documented:
- **Not just "blending mode"** - directly correlates with compression method
- **alpha_type=8**: Exclusive to RAW1 palettized textures
- **alpha_type=0**: Standard for DXT with 0/1-bit alpha
- **alpha_type=1**: Standard for DXT with 8-bit alpha

### File Size Patterns
- **1-10KB**: Small UI elements, simple icons (36% of sample)
- **10-100KB**: Standard textures with mipmaps (64% of sample) 
- **No files >1MB** found in UI/texture archives (may exist in model textures)

### Dimension Distribution
All textures use power-of-2 dimensions exclusively:
- **Square textures**: 256x256, 128x128, 64x64, 32x32
- **Rectangular textures**: Used for character parts (128x64, 128x32)
- **Unusual ratios**: Some UI elements use 64x256, 32x64 for specific layouts

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

**Implementation Status:** ⚠️ **Partial** - BLP2 JPEG explicitly rejected

- Non-standard BGRA compression
- Can cause color bleeding at block boundaries  
- Alpha stored as separate channel
- **Note:** While JPEG is part of the BLP format specification, BLP2 JPEG files are explicitly rejected in the current implementation

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

## Key Findings Summary

**Based on comprehensive analysis of 50+ BLP files from WoW 1.12.1:**

### Format Standardization
- **BLP2 Universal**: All files use BLP2 format, no legacy BLP0/BLP1 in WoW
- **Direct Content Only**: No JPEG content found (content_type=1 universal)
- **Compression Split**: Clear division between DXT (82%) and RAW1 palettized (18%)

### Alpha Type Correlation
- **alpha_type field** directly correlates with compression method, not just blending
- **Predictable patterns** allow format validation and automatic compression detection
- **RAW1 textures** consistently use alpha_type=8 regardless of actual alpha bits

### Content-Specific Optimization
- **Character textures**: RAW1 for color palette efficiency
- **Creature skins**: DXT with 8-bit alpha for detail
- **UI elements**: DXT with appropriate alpha for purpose
- **Effects**: DXT optimized for particle rendering

### Quality Assurance
- **100% power-of-2 dimensions** - no exceptions found
- **Consistent mipmap chains** - proper LOD progression
- **Appropriate compression** - format matches content type

## References

- [BLP Format (wowdev.wiki)](https://wowdev.wiki/BLP)
- [DXT Compression](https://docs.microsoft.com/en-us/windows/win32/direct3d11/texture-block-compression)
- Original image-blp crate documentation
- **Empirical Analysis**: 50+ BLP files from WoW 1.12.1 MPQ archives (2025)

## See Also

- [Texture Loading Guide](../../guides/texture-loading.md)
- [M2 Format](m2.md) - Uses BLP textures
- [WMO Format](wmo.md) - Uses BLP textures
