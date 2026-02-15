# wow-blp

Parser and encoder for Blizzard BLP texture files used in Warcraft III and World of Warcraft.

<div align="center">

[![Crates.io Version](https://img.shields.io/crates/v/wow-blp)](https://crates.io/crates/wow-blp)
[![docs.rs](https://img.shields.io/docsrs/wow-blp)](https://docs.rs/wow-blp)
[![License](https://img.shields.io/crates/l/wow-blp.svg)](https://github.com/wowemulation-dev/warcraft-rs#license)

</div>

## Status

✅ **Implemented** - Full BLP parsing and encoding functionality.

## Features

- Parse and encode all BLP versions (BLP0, BLP1, BLP2)
- Support for all compression formats:
  - JPEG compression with alpha channel
  - RAW1 (256-color palettized) with 0/1/4/8-bit alpha
  - RAW3 (uncompressed BGRA)
  - DXT1/3/5 compression (S3TC)
- Mipmap support (internal and external)
- Convert between BLP and standard image formats
- High-performance DXT compression using texpresso

## Supported Versions

- [x] BLP0 - Warcraft III Beta (external mipmaps)
- [x] BLP1 - Warcraft III (1.x+)
- [x] BLP2 - World of Warcraft (all versions)
  - [x] Classic (1.12.1)
  - [x] The Burning Crusade (2.4.3)
  - [x] Wrath of the Lich King (3.3.5a)
  - [x] Cataclysm (4.3.4)
  - [x] Mists of Pandaria (5.4.8)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
wow-blp = "0.6"
```

Or use cargo add:

```bash
cargo add wow-blp
```

## Usage

### Loading BLP Files

```rust,no_run
use wow_blp::{parser::load_blp, convert::blp_to_image};

// Load BLP file
let blp_file = load_blp("texture.blp")?;

// Convert to standard image format
let image = blp_to_image(&blp_file, 0)?; // mipmap level 0

// Save as PNG
image.save("texture.png")?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Creating BLP Files

```rust,no_run
use wow_blp::{
    convert::{image_to_blp, BlpTarget, Blp2Format, DxtAlgorithm},
    encode::save_blp,
};
use image::imageops::FilterType;

// Load source image
let image = image::open("input.png")?;

// Convert to BLP2 with DXT5 compression
let blp = image_to_blp(
    image,
    true, // generate mipmaps
    BlpTarget::Blp2(Blp2Format::Dxt5 {
        has_alpha: true,
        compress_algorithm: DxtAlgorithm::ClusterFit,
    }),
    FilterType::Lanczos3
)?;

// Save BLP file
save_blp(&blp, "output.blp")?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Format Options

```rust
use wow_blp::convert::{BlpTarget, BlpOldFormat, Blp2Format, AlphaBits, DxtAlgorithm};

// BLP0 (Warcraft III Beta) - External mipmaps
let blp0 = BlpTarget::Blp0(BlpOldFormat::Jpeg { has_alpha: true });

// BLP1 (Warcraft III) - Palettized
let blp1 = BlpTarget::Blp1(BlpOldFormat::Raw1 {
    alpha_bits: AlphaBits::Bit8
});

// BLP2 (World of Warcraft) - DXT compression
let blp2_dxt = BlpTarget::Blp2(Blp2Format::Dxt5 {
    has_alpha: true,
    compress_algorithm: DxtAlgorithm::ClusterFit
});

// BLP2 - Uncompressed
let blp2_raw = BlpTarget::Blp2(Blp2Format::Raw3);
```

### Working with Mipmaps

```rust,no_run
use wow_blp::{parser::load_blp, convert::blp_to_image};

// Load BLP file first
let blp_file = load_blp("texture.blp")?;

// Access specific mipmap level
let mipmap_2 = blp_to_image(&blp_file, 2)?;

// Get mipmap count
let count = blp_file.header.mipmaps_count();

// For BLP0, external mipmap files are handled automatically
// texture.blp → texture.b00, texture.b01, etc.
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Examples

See the [examples](examples/) directory for more usage examples:

- `load.rs` - Load and convert BLP to PNG
- `save.rs` - Convert PNG to BLP

## Performance

- DXT compression/decompression is parallelized using rayon
- Direct GPU upload support for DXT formats
- Efficient palette quantization for RAW1 format

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](../../LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Credits

Based on the [image-blp](https://github.com/zloy-tulen/image-blp) crate by zloy_tulen.
