use super::super::error::Error;
use super::super::types::Parser;
use crate::types::*;
use log::*;
use nom::{Err, multi::count, number::complete::le_u8};

pub fn parse_raw1<'a>(
    blp_header: &BlpHeader,
    original_input: &'a [u8],
    offsets: &[u32],
    sizes: &[u32],
    images: &mut Vec<Raw1Image>,
    input: &'a [u8],
) -> Parser<'a, ()> {
    let mut read_image = |i: usize| {
        let offset = offsets[i];
        let size = sizes[i];
        if offset as usize >= original_input.len() {
            error!(
                "Offset of mipmap {} is out of bounds! {} >= {}",
                i,
                offset,
                original_input.len()
            );
            return Err(Err::Failure(Error::<&[u8]>::OutOfBounds(0)));
        }
        if (offset + size) as usize > original_input.len() {
            error!(
                "Offset+size of mipmap {} is out of bounds! {} > {}",
                i,
                offset + size,
                original_input.len()
            );
            return Err(Err::Failure(Error::OutOfBounds(0)));
        }

        let image_bytes = &original_input[offset as usize..(offset + size) as usize];
        let n = blp_header.mipmap_pixels(i);
        let (input, indexed_rgb) = count(le_u8, n as usize)(image_bytes)?;
        let an = (n * blp_header.alpha_bits()).div_ceil(8);
        let (_, indexed_alpha) = count(le_u8, an as usize)(input)?;

        images.push(Raw1Image {
            indexed_rgb,
            indexed_alpha,
        });
        Ok(())
    };

    trace!("Mipmaps count: {}", blp_header.mipmaps_count());
    read_image(0)?;
    if blp_header.has_mipmaps() {
        for i in 1..(blp_header.mipmaps_count() + 1).min(16) {
            read_image(i)?;
        }
    }
    Ok((input, ()))
}
