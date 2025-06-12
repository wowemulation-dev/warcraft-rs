use ::image::DynamicImage;
use wow_blp::{convert::blp_to_image, parser::load_blp};

fn main() {
    let blp_filename = std::env::args().nth(1).unwrap_or("test.blp".to_owned());
    let output_filename = std::env::args().nth(2).unwrap_or("output.png".to_owned());
    let blp_file = load_blp(blp_filename).expect("loaded blp");
    let image: DynamicImage = blp_to_image(&blp_file, 0).expect("converted");
    image.save(output_filename).expect("saved");
}
