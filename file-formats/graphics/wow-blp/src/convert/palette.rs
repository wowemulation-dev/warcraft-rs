use super::error::Error;
use ::image::RgbaImage;

/// Create image and palette for rgb image
pub fn quantize_rgba(
    mut img: RgbaImage,
) -> Result<(Vec<u8>, Vec<u32>, color_quant::NeuQuant), Error> {
    let palette_size = 255; // last color in cmap is not used
    let sample_fact = 10; // speed-quality factor. 1 is best quality, 30 is best perf

    // zero alpha values
    for pix in img.pixels_mut() {
        pix[3] = 0;
    }
    // quantize
    let nq = color_quant::NeuQuant::new(sample_fact, palette_size, img.as_raw());
    let quantized: Vec<u8> = img.pixels().map(|pix| nq.index_of(&pix.0) as u8).collect();
    // collect palette
    let palette = nq
        .color_map_rgb()
        .chunks(3)
        .map(|col| {
            let red = col[0] as u32;
            let green = (col[1] as u32) << 8;
            let blue = (col[2] as u32) << 16;
            red + green + blue
        })
        .collect();

    Ok((quantized, palette, nq))
}

pub fn quantize_rgba_known(
    mut img: RgbaImage,
    nq: &color_quant::NeuQuant,
) -> Result<Vec<u8>, Error> {
    // zero alpha values
    for pix in img.pixels_mut() {
        pix[3] = 0;
    }
    let quantized: Vec<u8> = img.pixels().map(|pix| nq.index_of(&pix.0) as u8).collect();
    Ok(quantized)
}
