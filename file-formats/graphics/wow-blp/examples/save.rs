use ::image::{DynamicImage, ImageReader, imageops::FilterType};
use wow_blp::{
    convert::{AlphaBits, BlpOldFormat, BlpTarget, image_to_blp},
    encode::save_blp,
};

fn main() {
    let input_filename = std::env::args().nth(1).unwrap_or("test.png".to_owned());
    let output_filename = std::env::args().nth(2).unwrap_or("output.blp".to_owned());
    let img_file: DynamicImage = ImageReader::open(input_filename)
        .expect("open")
        .decode()
        .expect("decode");
    let blp = image_to_blp(
        img_file,
        true,
        BlpTarget::Blp1(BlpOldFormat::Raw1 {
            alpha_bits: AlphaBits::Bit1,
        }),
        FilterType::Nearest,
    )
    .expect("converted");
    save_blp(&blp, output_filename).expect("saved");
}
