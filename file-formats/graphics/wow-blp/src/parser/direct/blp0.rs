use super::super::error::Error;
use super::super::reader::{ByteReader, Cursor};
use super::super::types::ParseResult;
use crate::types::*;

pub fn parse_blp0<'a, F>(
    blp_header: &BlpHeader,
    mut external_mipmaps: F,
    images: &mut Vec<Raw1Image>,
    _input: &'a [u8],
) -> ParseResult<()>
where
    F: FnMut(usize) -> Result<Option<&'a [u8]>, Box<dyn std::error::Error>>,
{
    let mut read_mipmap = |i| -> ParseResult<()> {
        let image_bytes_opt = external_mipmaps(i).map_err(|e| Error::ExternalMipmap(i, e))?;
        let image_bytes = image_bytes_opt.ok_or(Error::MissingImage(i))?;
        let image = parse_raw1_image(blp_header, i, image_bytes)?;
        images.push(image);

        Ok(())
    };
    read_mipmap(0)?;

    if blp_header.has_mipmaps() {
        for i in 1..(blp_header.mipmaps_count() + 1).min(16) {
            read_mipmap(i)?;
        }
    }
    Ok(())
}

fn parse_raw1_image(
    blp_header: &BlpHeader,
    mimpmap_number: usize,
    input: &[u8],
) -> ParseResult<Raw1Image> {
    let mut reader = Cursor::new(input);

    let n = blp_header.mipmap_pixels(mimpmap_number);
    let indexed_rgb = reader.read_bytes(n as usize)?;

    let an = (n * blp_header.alpha_bits()).div_ceil(8);
    let indexed_alpha = reader.read_bytes(an as usize)?;

    Ok(Raw1Image {
        indexed_rgb,
        indexed_alpha,
    })
}
