use super::error::Error;
use super::mipmap::generate_mipmaps;
use crate::types::*;
use ::image::{DynamicImage, RgbaImage, imageops::FilterType};

pub fn raw3_to_image(
    header: &BlpHeader,
    image: &BlpRaw3,
    mipmap_level: usize,
) -> Result<DynamicImage, Error> {
    if mipmap_level >= image.images.len() {
        return Err(Error::MissingImage(mipmap_level));
    }
    let raw_image = &image.images[mipmap_level];
    let (width, height) = header.mipmap_size(mipmap_level);
    if (width as usize) * (height as usize) != raw_image.pixels.len() {
        return Err(Error::MismatchSizes(
            mipmap_level,
            width,
            height,
            raw_image.pixels.len(),
        ));
    }

    let mut res_image = RgbaImage::new(width, height);
    for (i, pixel) in res_image.pixels_mut().enumerate() {
        let color = raw_image.pixels[i];
        let blue = (color & 0xFF) as u8;
        let green = ((color >> 8) & 0xFF) as u8;
        let red = ((color >> 16) & 0xFF) as u8;
        let alpha = (color >> 24) as u8;
        pixel.0 = [red, green, blue, alpha];
    }
    Ok(DynamicImage::ImageRgba8(res_image))
}

pub fn image_to_raw3(
    image: DynamicImage,
    make_mipmaps: bool,
    mipmap_filter: FilterType,
) -> Result<BlpRaw3, Error> {
    let raw_images = if make_mipmaps {
        generate_mipmaps(image, mipmap_filter)?.into_iter()
    } else {
        vec![image].into_iter()
    };

    let mut images = vec![];
    for image in raw_images {
        let rgba = image.into_rgba8();
        let pixels_num = (rgba.width() as usize) * (rgba.height() as usize);
        let mut pixels = Vec::with_capacity(pixels_num);
        for pixel in rgba.pixels() {
            let red = (pixel[0] as u32) << 16;
            let green = (pixel[1] as u32) << 8;
            let blue = pixel[2] as u32;
            let alpha = (pixel[3] as u32) << 24;
            pixels.push(red + green + blue + alpha);
        }
        images.push(Raw3Image { pixels })
    }

    Ok(BlpRaw3 {
        cmap: vec![0; 256],
        images,
    })
}
