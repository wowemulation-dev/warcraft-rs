use super::error::Error;
use super::mipmap::generate_mipmaps;
use super::palette::*;
use crate::types::*;
use ::image::{DynamicImage, RgbImage, RgbaImage, imageops::FilterType};

pub fn raw1_to_image(
    header: &BlpHeader,
    image: &BlpRaw1,
    mipmap_level: usize,
) -> Result<DynamicImage, Error> {
    if mipmap_level >= image.images.len() {
        return Err(Error::MissingImage(mipmap_level));
    }
    if image.cmap.len() != 256 {
        return Err(Error::ColorMapLengthInvalid(image.cmap.len()));
    }
    let raw_image = &image.images[mipmap_level];
    let (width, height) = header.mipmap_size(mipmap_level);
    if (width as usize) * (height as usize) != raw_image.indexed_rgb.len() {
        return Err(Error::MismatchSizes(
            mipmap_level,
            width,
            height,
            raw_image.indexed_rgb.len(),
        ));
    }
    let alpha_bits = header.alpha_bits();
    if alpha_bits == 0 {
        let mut res_image = RgbImage::new(width, height);
        for (i, pixel) in res_image.pixels_mut().enumerate() {
            let ci = raw_image.indexed_rgb[i];
            let color = image.cmap[ci as usize];
            pixel.0[0] = (color & 0xFF) as u8;
            pixel.0[1] = ((color >> 8) & 0xFF) as u8;
            pixel.0[2] = ((color >> 16) & 0xFF) as u8;
        }
        Ok(DynamicImage::ImageRgb8(res_image))
    } else if alpha_bits == 1 {
        let alpha_values = raw_image.indexed_alpha.len() * 8;
        // > as there can be padding bits
        if (width as usize) * (height as usize) > alpha_values {
            return Err(Error::MismatchSizes(
                mipmap_level,
                width,
                height,
                raw_image.indexed_rgb.len(),
            ));
        }

        let mut res_image = RgbaImage::new(width, height);
        for (i, pixel) in res_image.pixels_mut().enumerate() {
            let ci = raw_image.indexed_rgb[i];
            let color = image.cmap[ci as usize];
            pixel.0[0] = (color & 0xFF) as u8;
            pixel.0[1] = ((color >> 8) & 0xFF) as u8;
            pixel.0[2] = ((color >> 16) & 0xFF) as u8;
            let bit = (raw_image.indexed_alpha[i / 8] >> (i % 8)) & 0x01;
            pixel.0[3] = if bit == 1 { 255 } else { 0 };
        }
        Ok(DynamicImage::ImageRgba8(res_image))
    } else if alpha_bits == 4 {
        let alpha_values = raw_image.indexed_alpha.len() * 2;

        // > as there can be padding bits
        if (width as usize) * (height as usize) > alpha_values {
            return Err(Error::MismatchSizes(
                mipmap_level,
                width,
                height,
                raw_image.indexed_rgb.len(),
            ));
        }

        let mut res_image = RgbaImage::new(width, height);
        for (i, pixel) in res_image.pixels_mut().enumerate() {
            let ci = raw_image.indexed_rgb[i];
            let color = image.cmap[ci as usize];
            pixel.0[0] = (color & 0xFF) as u8;
            pixel.0[1] = ((color >> 8) & 0xFF) as u8;
            pixel.0[2] = ((color >> 16) & 0xFF) as u8;
            let alpha_block = raw_image.indexed_alpha[i / 2];
            let nibble = if i % 2 == 0 {
                alpha_block & 0x0F
            } else {
                alpha_block >> 4
            };
            pixel.0[3] = (nibble << 4) | nibble;
        }
        Ok(DynamicImage::ImageRgba8(res_image))
    } else if alpha_bits == 8 {
        let alpha_values = raw_image.indexed_alpha.len();

        if (width as usize) * (height as usize) != alpha_values {
            return Err(Error::MismatchSizes(
                mipmap_level,
                width,
                height,
                raw_image.indexed_rgb.len(),
            ));
        }

        let mut res_image = RgbaImage::new(width, height);
        for (i, pixel) in res_image.pixels_mut().enumerate() {
            let ci = raw_image.indexed_rgb[i];
            let color = image.cmap[ci as usize];
            pixel.0[0] = (color & 0xFF) as u8;
            pixel.0[1] = ((color >> 8) & 0xFF) as u8;
            pixel.0[2] = ((color >> 16) & 0xFF) as u8;
            pixel.0[3] = raw_image.indexed_alpha[i];
        }
        Ok(DynamicImage::ImageRgba8(res_image))
    } else {
        Err(Error::Raw1InvalidAlphaBits(alpha_bits))
    }
}

pub fn image_to_raw1(
    image: DynamicImage,
    alpha_bits: u32,
    make_mipmaps: bool,
    mipmap_filter: FilterType,
) -> Result<BlpRaw1, Error> {
    let mut raw_images = if make_mipmaps {
        generate_mipmaps(image, mipmap_filter)?.into_iter()
    } else {
        vec![image].into_iter()
    };

    let mut images = vec![];

    // Create quantized image from the first image.
    let root_image = raw_images
        .next()
        .ok_or(Error::MissingImage(0))?
        .into_rgba8();
    let indexed_alpha = index_alpha(&root_image, alpha_bits)?;
    let (root_quantized, cmap, nq) = quantize_rgba(root_image)?;
    if cmap.len() != 255 {
        return Err(Error::PaletteWrongSize(cmap.len()));
    }
    images.push(Raw1Image {
        indexed_rgb: root_quantized,
        indexed_alpha,
    });

    // Quantize mipmaps
    for image in raw_images {
        let rgba = image.into_rgba8();
        let indexed_alpha = index_alpha(&rgba, alpha_bits)?;
        let quantized = quantize_rgba_known(rgba, &nq)?;
        images.push(Raw1Image {
            indexed_rgb: quantized,
            indexed_alpha,
        });
    }

    Ok(BlpRaw1 { cmap, images })
}

fn index_alpha_1bit(image: &RgbaImage) -> Vec<u8> {
    let pixels_number = (image.width() as usize) * (image.height() as usize);
    let alpha_values = ((pixels_number as f64) / 8.0).ceil() as usize;
    let mut res = Vec::with_capacity(alpha_values);
    if pixels_number > 0 {
        let mut bits = 0;
        let mut i = 0;
        res.push(0);
        for pixel in image.pixels() {
            if bits >= 8 {
                bits = 0;
                i += 1;
                res.push(0);
            }
            res[i] |= if pixel[3] > 0 { 1 << bits } else { 0 };
            bits += 1;
        }
    }

    res
}

fn index_alpha_4bit(image: &RgbaImage) -> Vec<u8> {
    let pixels_number = (image.width() as usize) * (image.height() as usize);
    let alpha_values = ((pixels_number as f64) / 2.0).ceil() as usize;
    let mut res = Vec::with_capacity(alpha_values);
    if pixels_number > 0 {
        let mut bits = 0;
        let mut i = 0;
        res.push(0);
        for pixel in image.pixels() {
            if bits >= 8 {
                bits = 0;
                i += 1;
                res.push(0);
            }
            let scaled_alpha = (((pixel[3] as f64) / 255.0) * 123.0) as u8;
            res[i] |= scaled_alpha << bits;
            bits += 4;
        }
    }

    res
}

fn index_alpha_8bit(image: &RgbaImage) -> Vec<u8> {
    let pixels_number = (image.width() as usize) * (image.height() as usize);
    let mut res = Vec::with_capacity(pixels_number);
    for pixel in image.pixels() {
        res.push(pixel[3]);
    }
    res
}

fn index_alpha(image: &RgbaImage, alpha_bits: u32) -> Result<Vec<u8>, Error> {
    if alpha_bits == 0 {
        Ok(vec![])
    } else if alpha_bits == 1 {
        Ok(index_alpha_1bit(image))
    } else if alpha_bits == 4 {
        Ok(index_alpha_4bit(image))
    } else if alpha_bits == 8 {
        Ok(index_alpha_8bit(image))
    } else {
        Err(Error::Raw1InvalidAlphaBits(alpha_bits))
    }
}
