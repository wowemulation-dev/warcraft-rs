//! Parser for World of Warcraft BLP (texture) files.
//!
//! This crate provides support for reading and writing BLP texture files used in
//! World of Warcraft and Warcraft III. BLP is Blizzard's proprietary texture format
//! that supports various compression methods including JPEG compression, palettized
//! images, and DXT compression.
//!
//! # Supported Versions
//!
//! - **BLP0** - Used in Warcraft III ROC Beta builds
//! - **BLP1** - Common in Warcraft III TFT (versions 1.12.1+)
//! - **BLP2** - Used in World of Warcraft (versions 1.12.1 through 5.4.8)
//!
//! # Supported Encodings
//!
//! - **RAW1** - Palettized images with 256 colors
//! - **RAW3** - Uncompressed RGBA bitmaps
//! - **JPEG** - JPEG compressed images
//! - **DXTn** - S3TC compression algorithms (BLP2 only)
//!
//! # Examples
//!
//! ## Loading a BLP file
//!
//! ```no_run
//! use wow_blp::{parser::load_blp, convert::blp_to_image};
//!
//! let blp_file = load_blp("texture.blp").expect("Failed to load BLP");
//! let mipmap_level = 0;
//! let image = blp_to_image(&blp_file, mipmap_level).expect("Failed to convert");
//! ```
//!
//! ## Saving an image as BLP
//!
//! ```no_run
//! use image::DynamicImage;
//! use wow_blp::{
//!     convert::{image_to_blp, BlpTarget, BlpOldFormat, AlphaBits, FilterType},
//!     encode::save_blp,
//! };
//!
//! # let image = DynamicImage::new_rgba8(256, 256);
//! let make_mipmaps = true;
//! let target = BlpTarget::Blp1(BlpOldFormat::Raw1 {
//!     alpha_bits: AlphaBits::Bit1,
//! });
//! let blp = image_to_blp(image, make_mipmaps, target, FilterType::Nearest)
//!     .expect("Failed to convert");
//! save_blp(&blp, "output.blp").expect("Failed to save");
//! ```

#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Conversion utilities to/from DynamicImage
pub mod convert;
/// Encoding BLP format into stream of bytes
pub mod encode;
/// Decoding BLP format from raw bytes
pub mod parser;
/// Utilities for mipmaps filename generation
pub mod path;
/// Defines structure of parsed BLP file
pub mod types;

pub use types::*;
