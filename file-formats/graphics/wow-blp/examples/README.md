# wow-blp Examples

This directory contains examples demonstrating how to use the wow-blp crate for loading and saving BLP texture files.

## Running Examples

All examples accept command-line arguments. If no arguments are provided, they use default filenames.

## Examples

### load.rs - Convert BLP to Standard Image Format

Loads a BLP file and converts it to a standard image format (PNG, JPEG, etc.).

```bash
# Basic usage with default filenames (test.blp → output.png)
cargo run --example load

# Specify input and output files
cargo run --example load -- texture.blp texture.png

# Convert to different formats based on extension
cargo run --example load -- icon.blp icon.jpg
```

**Features demonstrated:**
- Loading BLP files of any version (BLP0/1/2)
- Extracting the main image (mipmap level 0)
- Saving to standard image formats

### save.rs - Convert Standard Image to BLP

Converts a standard image format to BLP format.

```bash
# Basic usage with default filenames (test.png → output.blp)
cargo run --example save

# Specify input and output files
cargo run --example save -- image.png texture.blp

# Works with various input formats
cargo run --example save -- photo.jpg texture.blp
```

**Features demonstrated:**
- Loading standard image formats (PNG, JPEG, etc.)
- Converting to BLP1 format with RAW1 compression
- Generating mipmaps automatically
- Using 1-bit alpha channel

## Advanced Usage

For more advanced examples showing different BLP formats and options, see the integration tests in the `tests/` directory or the main crate documentation.

### Example: Creating BLP2 with DXT5 Compression

```rust
use image::imageops::FilterType;
use wow_blp::{
    convert::{image_to_blp, BlpTarget, Blp2Format, DxtAlgorithm},
    encode::save_blp,
};

let img = image::open("input.png")?;
let blp = image_to_blp(
    img,
    true, // generate mipmaps
    BlpTarget::Blp2(Blp2Format::Dxt5 {
        has_alpha: true,
        compress_algorithm: DxtAlgorithm::ClusterFit,
    }),
    FilterType::Lanczos3
)?;
save_blp(&blp, "output.blp")?;
```

## Test Data

To test these examples, you can:
1. Extract BLP files from WoW MPQ archives using the warcraft-rs CLI
2. Use any PNG/JPEG image as input for the save example
3. Download sample BLP files from WoW modding communities

## Common Issues

- **File not found**: Make sure the input file exists in the current directory
- **Unsupported format**: The output format is determined by the file extension
- **Memory usage**: Large textures with mipmaps can use significant memory