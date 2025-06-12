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

    let mut output = vec![0; size];
    let decoder: texpresso::Format = image.format.into();
    decoder.decompress(
        &raw_image.content,
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
