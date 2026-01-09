use super::error::Error;
use super::mipmap::generate_mipmaps;
use crate::types::*;
use ::image::{DynamicImage, RgbaImage, imageops::FilterType};

pub fn dxtn_to_image(
    header: &BlpHeader,
    image: &BlpDxtn,
    mipmap_level: usize,
) -> Result<DynamicImage, Error> {
    if mipmap_level >= image.images.len() {
        return Err(Error::MissingImage(mipmap_level));
    }
    let raw_image = &image.images[mipmap_level];
    let (width, height) = header.mipmap_size(mipmap_level);
    let size = (width as usize) * (height as usize) * 4;

    let decoder: texpresso::Format = image.format.into();

    // Calculate required compressed data size for the DXT format
    // DXT formats work on 4x4 blocks:
    // - DXT1: 8 bytes per block
    // - DXT3/DXT5: 16 bytes per block
    // Formula: ceil((width+3)/4) * ceil((height+3)/4) * block_size
    let required_size = decoder.compressed_size(width as usize, height as usize);

    // If the actual data is smaller than required, pad with zeros
    // This matches SereniaBLPLib behavior - small mipmaps often have undersized data
    // in BLP files, and zero-padding allows decompression to succeed
    let compressed_data: std::borrow::Cow<'_, [u8]> = if raw_image.content.len() < required_size {
        // Create zero-padded buffer and copy available data
        let mut padded = vec![0u8; required_size];
        padded[..raw_image.content.len()].copy_from_slice(&raw_image.content);
        std::borrow::Cow::Owned(padded)
    } else {
        std::borrow::Cow::Borrowed(&raw_image.content)
    };

    let mut output = vec![0; size];
    decoder.decompress(
        &compressed_data,
        width as usize,
        height as usize,
        &mut output,
    );
    let result = RgbaImage::from_raw(width, height, output).ok_or(Error::Dxt1RawConvertFail)?;
    Ok(DynamicImage::ImageRgba8(result))
}

pub fn image_to_dxtn(
    image: DynamicImage,
    format: DxtnFormat,
    make_mipmaps: bool,
    mipmap_filter: FilterType,
    compress_algorithm: texpresso::Algorithm,
) -> Result<BlpDxtn, Error> {
    let raw_images = if make_mipmaps {
        generate_mipmaps(image, mipmap_filter)?.into_iter()
    } else {
        vec![image].into_iter()
    };
    let encoder: texpresso::Format = format.into();
    let mut images = vec![];
    for image in raw_images {
        let rgba = image.into_rgba8();
        let width = rgba.width() as usize;
        let height = rgba.height() as usize;
        let output_size = encoder.compressed_size(width, height);
        let mut output = vec![0; output_size];
        let params = texpresso::Params {
            algorithm: compress_algorithm,
            ..Default::default()
        };
        encoder.compress(rgba.as_raw(), width, height, params, &mut output);
        images.push(DxtnImage { content: output })
    }

    Ok(BlpDxtn {
        format,
        cmap: vec![0; 256],
        images,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        AlphaType, BlpContentTag, BlpFlags, BlpVersion, Compression, MipmapLocator,
    };

    fn create_test_header(width: u32, height: u32) -> BlpHeader {
        BlpHeader {
            version: BlpVersion::Blp2,
            content: BlpContentTag::Direct,
            flags: BlpFlags::Blp2 {
                compression: Compression::Dxtc,
                alpha_bits: 8,
                alpha_type: AlphaType::EightBit,
                has_mipmaps: 0, // no mipmaps
            },
            width,
            height,
            mipmap_locator: MipmapLocator::Internal {
                offsets: [0; 16],
                sizes: [8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            },
        }
    }

    /// Test that undersized DXT3 buffer succeeds with zero-padding
    /// (matches SereniaBLPLib behavior for small mipmaps)
    #[test]
    fn test_dxt3_undersized_buffer_succeeds_with_padding() {
        // DXT3 requires 16 bytes per 4x4 block
        // A 4x4 texture needs 1 block = 16 bytes
        // We provide only 8 bytes - should be zero-padded and succeed

        let header = create_test_header(4, 4);

        let image = BlpDxtn {
            format: DxtnFormat::Dxt3,
            cmap: vec![0; 256],
            images: vec![DxtnImage {
                content: vec![0; 8], // Only 8 bytes instead of required 16
            }],
        };

        let result = dxtn_to_image(&header, &image, 0);
        assert!(
            result.is_ok(),
            "Expected success with zero-padding for undersized buffer"
        );

        let img = result.unwrap();
        assert_eq!(img.width(), 4);
        assert_eq!(img.height(), 4);
    }

    /// Test that undersized DXT5 buffer succeeds with zero-padding
    #[test]
    fn test_dxt5_undersized_buffer_succeeds_with_padding() {
        // DXT5 also requires 16 bytes per 4x4 block

        let header = create_test_header(4, 4);

        let image = BlpDxtn {
            format: DxtnFormat::Dxt5,
            cmap: vec![0; 256],
            images: vec![DxtnImage {
                content: vec![0; 8], // Only 8 bytes instead of required 16
            }],
        };

        let result = dxtn_to_image(&header, &image, 0);
        assert!(
            result.is_ok(),
            "Expected success with zero-padding for undersized buffer"
        );
    }

    /// Test that DXT1 with correctly sized buffer succeeds
    #[test]
    fn test_dxt1_valid_buffer_succeeds() {
        // DXT1 requires 8 bytes per 4x4 block

        let header = create_test_header(4, 4);

        let image = BlpDxtn {
            format: DxtnFormat::Dxt1,
            cmap: vec![0; 256],
            images: vec![DxtnImage {
                content: vec![0; 8], // Correct size for DXT1 4x4
            }],
        };

        let result = dxtn_to_image(&header, &image, 0);
        assert!(
            result.is_ok(),
            "Expected success for correctly sized buffer"
        );
    }

    /// Test that completely empty buffer still works (produces blank image)
    #[test]
    fn test_empty_buffer_produces_blank_image() {
        let header = create_test_header(4, 4);

        let image = BlpDxtn {
            format: DxtnFormat::Dxt1,
            cmap: vec![0; 256],
            images: vec![DxtnImage {
                content: vec![], // Empty buffer
            }],
        };

        let result = dxtn_to_image(&header, &image, 0);
        assert!(
            result.is_ok(),
            "Expected success with zero-padding for empty buffer"
        );
    }
}
