mod dxtn;
/// Conversion error types
pub mod error;
mod jpeg;
mod mipmap;
mod palette;
mod raw1;
mod raw3;

use crate::types::*;
use ::image::DynamicImage;
pub use ::image::imageops::FilterType;
use dxtn::*;
pub use error::Error;
use jpeg::*;
use raw1::*;
use raw3::*;
use std::fmt;
pub use texpresso::Algorithm as DxtAlgorithm;

/// Convert from parsed raw BLP image to useful [DynamicImage]
pub fn blp_to_image(image: &BlpImage, mipmap_level: usize) -> Result<DynamicImage, Error> {
    match &image.content {
        BlpContent::Raw1(content) => raw1_to_image(&image.header, content, mipmap_level),
        BlpContent::Raw3(content) => raw3_to_image(&image.header, content, mipmap_level),
        BlpContent::Jpeg(content) => jpeg_to_image(content, mipmap_level),
        BlpContent::Dxt1(content) => dxtn_to_image(&image.header, content, mipmap_level),
        BlpContent::Dxt3(content) => dxtn_to_image(&image.header, content, mipmap_level),
        BlpContent::Dxt5(content) => dxtn_to_image(&image.header, content, mipmap_level),
    }
}

/// A way to specify [image_to_blp] which BLP type you want to
/// get in a result.
#[derive(Clone, PartialEq, Eq)]
pub enum BlpTarget {
    /// BLP0 format variation. War3 RoC Beta builds. External
    /// mipmaps.
    Blp0(BlpOldFormat),
    /// BLP1 format variation. War3 TFT usual textures. Internal
    /// mipmaps.
    Blp1(BlpOldFormat),
    /// BLP2 format variation. WoW usual textures. Internal
    /// mipmaps.
    Blp2(Blp2Format),
}

impl Default for BlpTarget {
    fn default() -> Self {
        BlpTarget::Blp1(Default::default())
    }
}

impl fmt::Display for BlpTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlpTarget::Blp0(format) => write!(f, "BLP0 {format}"),
            BlpTarget::Blp1(format) => write!(f, "BLP1 {format}"),
            BlpTarget::Blp2(format) => write!(f, "BLP2 {format}"),
        }
    }
}

/// Encoding options for BLP0 and BLP1 formats.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlpOldFormat {
    /// Paletted 256 colors image with/without alpha.  
    Raw1 {
        /// Alpha channel bit depth
        alpha_bits: AlphaBits,
    },
    /// JPEG encoding with/without alpha.
    Jpeg {
        /// Whether the JPEG has an alpha channel
        has_alpha: bool,
    },
}

impl Default for BlpOldFormat {
    fn default() -> Self {
        BlpOldFormat::Jpeg { has_alpha: true }
    }
}

impl fmt::Display for BlpOldFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlpOldFormat::Raw1 { alpha_bits } => write!(f, "Palleted image with {alpha_bits}"),
            BlpOldFormat::Jpeg { has_alpha } => {
                if *has_alpha {
                    write!(f, "Jpeg image with alpha")
                } else {
                    write!(f, "Jpeg image without alpha")
                }
            }
        }
    }
}

/// Allowed alpha bits values for Raw1 encoding
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum AlphaBits {
    /// No alpha channel. 0 bits.
    NoAlpha,
    /// 1 bit. Pixel is transparent or opaque.
    Bit1,
    /// 4 bits per pixel.
    Bit4,
    /// 8 bits per pixel.
    #[default]
    Bit8,
}

impl fmt::Display for AlphaBits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlphaBits::NoAlpha => write!(f, "no alpha"),
            AlphaBits::Bit1 => write!(f, "1 bit alpha"),
            AlphaBits::Bit4 => write!(f, "4 bits alpha"),
            AlphaBits::Bit8 => write!(f, "8 bits alpha"),
        }
    }
}

impl From<AlphaBits> for u32 {
    fn from(value: AlphaBits) -> u32 {
        match value {
            AlphaBits::NoAlpha => 0,
            AlphaBits::Bit1 => 1,
            AlphaBits::Bit4 => 4,
            AlphaBits::Bit8 => 8,
        }
    }
}

impl From<AlphaBits> for u8 {
    fn from(value: AlphaBits) -> u8 {
        match value {
            AlphaBits::NoAlpha => 0,
            AlphaBits::Bit1 => 1,
            AlphaBits::Bit4 => 4,
            AlphaBits::Bit8 => 8,
        }
    }
}

/// BLP2 format compression options.
#[derive(Clone, PartialEq, Eq)]
pub enum Blp2Format {
    /// Paletted 256 colors image with/without alpha.  
    Raw1 {
        /// Alpha channel bit depth
        alpha_bits: AlphaBits,
    },
    /// RGBA bitmap
    Raw3,
    /// JPEG encoded image. Although, it is never used in real files.
    Jpeg {
        /// Whether the JPEG has an alpha channel
        has_alpha: bool,
    },
    /// ST3C compression, type with 1 bit alpha or 0 bit alpha.
    Dxt1 {
        /// Whether the texture has an alpha channel
        has_alpha: bool,
        /// Compression speed/quality setting
        compress_algorithm: DxtAlgorithm,
    },
    /// ST3C compression, type with paletted alpha.
    Dxt3 {
        /// Whether the texture has an alpha channel
        has_alpha: bool,
        /// Compression speed/quality setting
        compress_algorithm: DxtAlgorithm,
    },
    /// ST3C compression, type with interpolated alpha.
    Dxt5 {
        /// Whether the texture has an alpha channel
        has_alpha: bool,
        /// Compression speed/quality setting
        compress_algorithm: DxtAlgorithm,
    },
}

impl Default for Blp2Format {
    fn default() -> Self {
        Blp2Format::Dxt5 {
            has_alpha: true,
            compress_algorithm: Default::default(),
        }
    }
}

impl fmt::Display for Blp2Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Blp2Format::Raw1 { alpha_bits } => write!(f, "Palleted image with {alpha_bits}"),
            Blp2Format::Raw3 => write!(f, "RGBA raw data"),
            Blp2Format::Jpeg { has_alpha } => {
                if *has_alpha {
                    write!(f, "Jpeg image with alpha")
                } else {
                    write!(f, "Jpeg image without alpha")
                }
            }
            Blp2Format::Dxt1 {
                has_alpha,
                compress_algorithm,
            } => {
                let compress_str = match *compress_algorithm {
                    DxtAlgorithm::RangeFit => "fast/low quality",
                    DxtAlgorithm::ClusterFit => "slow/high quality",
                    DxtAlgorithm::IterativeClusterFit => "very slow/best quality",
                };
                if *has_alpha {
                    write!(f, "DXT1 image with alpha and compression {compress_str}")
                } else {
                    write!(f, "DXT1 image without alpha and compression {compress_str}")
                }
            }
            Blp2Format::Dxt3 {
                has_alpha,
                compress_algorithm,
            } => {
                let compress_str = match *compress_algorithm {
                    DxtAlgorithm::RangeFit => "fast/low quality",
                    DxtAlgorithm::ClusterFit => "slow/high quality",
                    DxtAlgorithm::IterativeClusterFit => "very slow/best quality",
                };
                if *has_alpha {
                    write!(f, "DXT3 image with alpha and compression {compress_str}")
                } else {
                    write!(f, "DXT3 image without alpha and compression {compress_str}")
                }
            }
            Blp2Format::Dxt5 {
                has_alpha,
                compress_algorithm,
            } => {
                let compress_str = match *compress_algorithm {
                    DxtAlgorithm::RangeFit => "fast/low quality",
                    DxtAlgorithm::ClusterFit => "slow/high quality",
                    DxtAlgorithm::IterativeClusterFit => "very slow/best quality",
                };
                if *has_alpha {
                    write!(f, "DXT5 image with alpha and compression {compress_str}")
                } else {
                    write!(f, "DXT5 image without alpha and compression {compress_str}")
                }
            }
        }
    }
}

/// Convert from unpacked pixels into BLP image ready for writing down
pub fn image_to_blp(
    image: DynamicImage,
    make_mipmaps: bool,
    target: BlpTarget,
    mipmap_filter: FilterType,
) -> Result<BlpImage, Error> {
    if image.width() > BLP_MAX_WIDTH {
        return Err(Error::WidthTooLarge(image.width()));
    }
    if image.height() > BLP_MAX_HEIGHT {
        return Err(Error::HeightTooLarge(image.height()));
    }

    match target {
        BlpTarget::Blp0(format) => match format {
            BlpOldFormat::Raw1 { alpha_bits } => {
                let header = BlpHeader {
                    version: BlpVersion::Blp0,
                    content: BlpContentTag::Direct,
                    flags: BlpFlags::Old {
                        alpha_bits: alpha_bits.into(),
                        extra: 4,
                        has_mipmaps: if make_mipmaps { 1 } else { 0 },
                    },
                    width: image.width(),
                    height: image.height(),
                    mipmap_locator: MipmapLocator::External,
                };
                let blp_raw1 =
                    image_to_raw1(image, alpha_bits.into(), make_mipmaps, mipmap_filter)?;
                Ok(BlpImage {
                    header,
                    content: BlpContent::Raw1(blp_raw1),
                })
            }
            BlpOldFormat::Jpeg { has_alpha } => {
                let alpha_bits = if has_alpha { 8 } else { 0 };
                let blp_jpeg = image_to_jpeg(&image, make_mipmaps, alpha_bits, mipmap_filter)?;
                Ok(BlpImage {
                    header: BlpHeader {
                        version: BlpVersion::Blp0,
                        content: BlpContentTag::Jpeg,
                        flags: BlpFlags::Old {
                            alpha_bits: alpha_bits as u32,
                            extra: 5,
                            has_mipmaps: if make_mipmaps { 1 } else { 0 },
                        },
                        width: image.width(),
                        height: image.height(),
                        mipmap_locator: MipmapLocator::External,
                    },
                    content: BlpContent::Jpeg(blp_jpeg),
                })
            }
        },
        BlpTarget::Blp1(format) => match format {
            BlpOldFormat::Raw1 { alpha_bits } => {
                let width = image.width();
                let height = image.height();
                let blp_raw1 =
                    image_to_raw1(image, alpha_bits.into(), make_mipmaps, mipmap_filter)?;
                let header = BlpHeader {
                    version: BlpVersion::Blp1,
                    content: BlpContentTag::Direct,
                    flags: BlpFlags::Old {
                        alpha_bits: alpha_bits.into(),
                        extra: 4,
                        has_mipmaps: if make_mipmaps { 1 } else { 0 },
                    },
                    width,
                    height,
                    mipmap_locator: blp_raw1.mipmap_locator(BlpVersion::Blp1),
                };
                Ok(BlpImage {
                    header,
                    content: BlpContent::Raw1(blp_raw1),
                })
            }
            BlpOldFormat::Jpeg { has_alpha } => {
                let alpha_bits = if has_alpha { 8 } else { 0 };
                let blp_jpeg = image_to_jpeg(&image, make_mipmaps, alpha_bits, mipmap_filter)?;
                Ok(BlpImage {
                    header: BlpHeader {
                        version: BlpVersion::Blp1,
                        content: BlpContentTag::Jpeg,
                        flags: BlpFlags::Old {
                            alpha_bits: alpha_bits as u32,
                            extra: 5,
                            has_mipmaps: if make_mipmaps { 1 } else { 0 },
                        },
                        width: image.width(),
                        height: image.height(),
                        mipmap_locator: blp_jpeg.mipmap_locator(BlpVersion::Blp1),
                    },
                    content: BlpContent::Jpeg(blp_jpeg),
                })
            }
        },
        BlpTarget::Blp2(format) => match format {
            Blp2Format::Raw1 { alpha_bits } => {
                let width = image.width();
                let height = image.height();
                let blp_raw1 =
                    image_to_raw1(image, alpha_bits.into(), make_mipmaps, mipmap_filter)?;
                let header = BlpHeader {
                    version: BlpVersion::Blp2,
                    content: BlpContentTag::Direct,
                    flags: BlpFlags::Blp2 {
                        compression: Compression::Raw1,
                        alpha_bits: alpha_bits.into(),
                        alpha_type: AlphaType::None,
                        has_mipmaps: if make_mipmaps { 1 } else { 0 },
                    },
                    width,
                    height,
                    mipmap_locator: blp_raw1.mipmap_locator(BlpVersion::Blp2),
                };
                Ok(BlpImage {
                    header,
                    content: BlpContent::Raw1(blp_raw1),
                })
            }
            Blp2Format::Raw3 => {
                let width = image.width();
                let height = image.height();
                let blp_raw3 = image_to_raw3(image, make_mipmaps, mipmap_filter)?;
                Ok(BlpImage {
                    header: BlpHeader {
                        version: BlpVersion::Blp2,
                        content: BlpContentTag::Direct,
                        flags: BlpFlags::Blp2 {
                            compression: Compression::Raw3,
                            alpha_bits: 8,
                            alpha_type: AlphaType::None,
                            has_mipmaps: if make_mipmaps { 1 } else { 0 },
                        },
                        width,
                        height,
                        mipmap_locator: blp_raw3.mipmap_locator(BlpVersion::Blp2),
                    },
                    content: BlpContent::Raw3(blp_raw3),
                })
            }
            Blp2Format::Jpeg { has_alpha } => {
                let alpha_bits = if has_alpha { 8 } else { 0 };
                let blp_jpeg = image_to_jpeg(&image, make_mipmaps, alpha_bits, mipmap_filter)?;
                Ok(BlpImage {
                    header: BlpHeader {
                        version: BlpVersion::Blp2,
                        content: BlpContentTag::Jpeg,
                        flags: BlpFlags::Blp2 {
                            compression: Compression::Jpeg,
                            alpha_bits: 8,
                            alpha_type: AlphaType::None,
                            has_mipmaps: if make_mipmaps { 1 } else { 0 },
                        },
                        width: image.width(),
                        height: image.height(),
                        mipmap_locator: blp_jpeg.mipmap_locator(BlpVersion::Blp2),
                    },
                    content: BlpContent::Jpeg(blp_jpeg),
                })
            }
            Blp2Format::Dxt1 {
                has_alpha,
                compress_algorithm,
            } => {
                let width = image.width();
                let height = image.height();
                let alpha_bits = if has_alpha { 1 } else { 0 };
                let blp_dxtn = image_to_dxtn(
                    image,
                    DxtnFormat::Dxt1,
                    make_mipmaps,
                    mipmap_filter,
                    compress_algorithm,
                )?;
                Ok(BlpImage {
                    header: BlpHeader {
                        version: BlpVersion::Blp2,
                        content: BlpContentTag::Direct,
                        flags: BlpFlags::Blp2 {
                            compression: Compression::Dxtc,
                            alpha_bits,
                            alpha_type: AlphaType::None,
                            has_mipmaps: if make_mipmaps { 1 } else { 0 },
                        },
                        width,
                        height,
                        mipmap_locator: blp_dxtn.mipmap_locator(BlpVersion::Blp2),
                    },
                    content: BlpContent::Dxt1(blp_dxtn),
                })
            }
            Blp2Format::Dxt3 {
                has_alpha,
                compress_algorithm,
            } => {
                let width = image.width();
                let height = image.height();
                let alpha_bits = if has_alpha { 8 } else { 0 };
                let blp_dxtn = image_to_dxtn(
                    image,
                    DxtnFormat::Dxt3,
                    make_mipmaps,
                    mipmap_filter,
                    compress_algorithm,
                )?;
                Ok(BlpImage {
                    header: BlpHeader {
                        version: BlpVersion::Blp2,
                        content: BlpContentTag::Direct,
                        flags: BlpFlags::Blp2 {
                            compression: Compression::Dxtc,
                            alpha_bits,
                            alpha_type: AlphaType::OneBit,
                            has_mipmaps: if make_mipmaps { 1 } else { 0 },
                        },
                        width,
                        height,
                        mipmap_locator: blp_dxtn.mipmap_locator(BlpVersion::Blp2),
                    },
                    content: BlpContent::Dxt3(blp_dxtn),
                })
            }
            Blp2Format::Dxt5 {
                has_alpha,
                compress_algorithm,
            } => {
                let width = image.width();
                let height = image.height();
                let alpha_bits = if has_alpha { 8 } else { 0 };
                let blp_dxtn = image_to_dxtn(
                    image,
                    DxtnFormat::Dxt5,
                    make_mipmaps,
                    mipmap_filter,
                    compress_algorithm,
                )?;
                Ok(BlpImage {
                    header: BlpHeader {
                        version: BlpVersion::Blp2,
                        content: BlpContentTag::Direct,
                        flags: BlpFlags::Blp2 {
                            compression: Compression::Dxtc,
                            alpha_bits,
                            alpha_type: AlphaType::Enhanced,
                            has_mipmaps: if make_mipmaps { 1 } else { 0 },
                        },
                        width,
                        height,
                        mipmap_locator: blp_dxtn.mipmap_locator(BlpVersion::Blp2),
                    },
                    content: BlpContent::Dxt5(blp_dxtn),
                })
            }
        },
    }
}
