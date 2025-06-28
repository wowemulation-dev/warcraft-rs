use wow_blp::{convert::blp_to_image, parser::load_blp};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <input.blp> [output.png]", args[0]);
        println!("\nExample: Convert a BLP texture file to PNG");
        return;
    }

    let blp_filename = &args[1];
    let output_filename = args.get(2).map(String::as_str).unwrap_or("output.png");

    match load_blp(blp_filename) {
        Ok(blp_file) => {
            println!("✓ Loaded BLP file: {blp_filename}");
            println!("  Version: {:?}", blp_file.header.version);
            println!(
                "  Dimensions: {}x{}",
                blp_file.header.width, blp_file.header.height
            );
            println!("  Mipmaps: {}", blp_file.image_count());

            match blp_to_image(&blp_file, 0) {
                Ok(image) => match image.save(output_filename) {
                    Ok(_) => println!("✓ Saved as: {output_filename}"),
                    Err(e) => eprintln!("✗ Failed to save image: {e}"),
                },
                Err(e) => eprintln!("✗ Failed to convert BLP to image: {e}"),
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to load BLP file '{blp_filename}': {e}");
            eprintln!("\nTip: Make sure the file exists and is a valid BLP texture file.");
        }
    }
}
