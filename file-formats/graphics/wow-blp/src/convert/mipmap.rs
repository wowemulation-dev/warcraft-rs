use super::error::Error;
use ::image::{DynamicImage, imageops::FilterType};

pub fn generate_mipmaps(
    image: DynamicImage,
    filter: FilterType,
) -> Result<Vec<DynamicImage>, Error> {
    let mut mipmaps = vec![image.clone()];
    let mut current_image = image;
    loop {
        let width = current_image.width();
        let height = current_image.height();
        if (width <= 1 || height <= 1 || mipmaps.len() >= 16) {
            break;
        }
        let new_width = width >> 1;
        let new_height = height >> 1;
        current_image = current_image.resize_exact(new_width, new_height, filter);
        mipmaps.push(current_image.clone());
    }
    Ok(mipmaps)
}
