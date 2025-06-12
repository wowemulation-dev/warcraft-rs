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
