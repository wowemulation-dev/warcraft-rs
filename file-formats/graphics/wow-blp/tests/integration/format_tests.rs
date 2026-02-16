//! Integration tests for BLP file parsing and encoding

use wow_blp::convert::{
    AlphaBits, BlpOldFormat, BlpTarget, FilterType, blp_to_image, image_to_blp,
};

#[test]
fn test_round_trip_blp1_raw1() {
    use image::DynamicImage;

    // Create a test image
    let test_image = DynamicImage::new_rgba8(64, 64);

    // Convert to BLP
    let make_mipmaps = false;
    let target = BlpTarget::Blp1(BlpOldFormat::Raw1 {
        alpha_bits: AlphaBits::NoAlpha,
    });

    let blp = image_to_blp(
        test_image.clone(),
        make_mipmaps,
        target,
        FilterType::Nearest,
    )
    .expect("Failed to convert image to BLP");

    // Save to memory buffer
    let buffer = wow_blp::encode::encode_blp(&blp).expect("Failed to encode BLP");

    // Load back from buffer
    let loaded_blp = wow_blp::parser::parse_blp(&buffer).expect("Failed to parse BLP");

    // Convert back to image
    let result_image = blp_to_image(&loaded_blp, 0).expect("Failed to convert BLP to image");

    // Basic validation
    assert_eq!(result_image.width(), 64);
    assert_eq!(result_image.height(), 64);
}

#[test]
fn test_raw1_1bit_alpha_scaling() {
    use image::DynamicImage;

    // Create a test image with checkerboard alpha pattern
    let mut img = image::RgbaImage::new(64, 64);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let alpha = if (x + y) % 2 == 0 { 255 } else { 0 };
        *pixel = image::Rgba([128, 128, 128, alpha]);
    }
    let test_image = DynamicImage::ImageRgba8(img);

    // Convert to BLP with 1-bit alpha
    let make_mipmaps = false;
    let target = BlpTarget::Blp1(BlpOldFormat::Raw1 {
        alpha_bits: AlphaBits::Bit1,
    });

    let blp = image_to_blp(
        test_image.clone(),
        make_mipmaps,
        target,
        FilterType::Nearest,
    )
    .expect("Failed to convert image to BLP");

    // Save to memory buffer
    let buffer = wow_blp::encode::encode_blp(&blp).expect("Failed to encode BLP");

    // Load back from buffer
    let loaded_blp = wow_blp::parser::parse_blp(&buffer).expect("Failed to parse BLP");

    // Convert back to image
    let result_image = blp_to_image(&loaded_blp, 0).expect("Failed to convert BLP to image");
    let result_rgba = result_image.to_rgba8();

    // Verify alpha is scaled to 0 or 255
    for (x, y, pixel) in result_rgba.enumerate_pixels() {
        let expected_alpha = if (x + y) % 2 == 0 { 255 } else { 0 };
        assert_eq!(
            pixel[3], expected_alpha,
            "Alpha at ({}, {}) should be {} but was {}",
            x, y, expected_alpha, pixel[3]
        );
    }
}

#[test]
fn test_raw1_4bit_alpha_scaling() {
    use image::DynamicImage;

    // Create a test image with various alpha values
    let mut img = image::RgbaImage::new(64, 64);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        // Create a gradient of alpha values
        let alpha = ((x as u16 * 255) / 63) as u8;
        *pixel = image::Rgba([128, 128, 128, alpha]);
    }
    let test_image = DynamicImage::ImageRgba8(img);

    // Convert to BLP with 4-bit alpha
    let make_mipmaps = false;
    let target = BlpTarget::Blp1(BlpOldFormat::Raw1 {
        alpha_bits: AlphaBits::Bit4,
    });

    let blp = image_to_blp(
        test_image.clone(),
        make_mipmaps,
        target,
        FilterType::Nearest,
    )
    .expect("Failed to convert image to BLP");

    // Save to memory buffer
    let buffer = wow_blp::encode::encode_blp(&blp).expect("Failed to encode BLP");

    // Load back from buffer
    let loaded_blp = wow_blp::parser::parse_blp(&buffer).expect("Failed to parse BLP");

    // Convert back to image
    let result_image = blp_to_image(&loaded_blp, 0).expect("Failed to convert BLP to image");
    let result_rgba = result_image.to_rgba8();

    // Verify alpha is scaled properly (4-bit -> 8-bit via multiplying by 17)
    for (x, _, pixel) in result_rgba.enumerate_pixels() {
        // The original alpha was quantized to 4 bits, then scaled back up
        // We should get values like 0, 17, 34, 51, 68, ... 255
        // Check that alpha is a valid scaled 4-bit value
        let alpha = pixel[3];
        assert!(
            alpha % 17 == 0,
            "Alpha {} at x={} is not a valid scaled 4-bit value",
            alpha, x
        );
    }
}
