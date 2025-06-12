use super::super::error::Error;
use super::super::reader::{ByteReader, Cursor};
use super::super::types::ParseResult;
use crate::types::*;
use log::*;

pub fn parse_raw1<'a>(
    blp_header: &BlpHeader,
    original_input: &'a [u8],
    offsets: &[u32; 16],
    sizes: &[u32; 16],
    images: &mut Vec<Raw1Image>,
    _input: &'a [u8],
) -> ParseResult<()> {
    let mut read_image = |i: usize| -> ParseResult<()> {
        let offset = offsets[i];
        let size = sizes[i];
        if offset as usize >= original_input.len() {
            error!(
                "Offset of mipmap {} is out of bounds! {} >= {}",
                i,
                offset,
                original_input.len()
            );
            return Err(Error::OutOfBounds {
                offset: offset as usize,
                size: 0,
            });
        }
        if (offset + size) as usize > original_input.len() {
            error!(
                "Offset+size of mipmap {} is out of bounds! {} > {}",
                i,
                offset + size,
                original_input.len()
            );
            return Err(Error::OutOfBounds {
                offset: offset as usize,
                size: size as usize,
            });
        }

        let image_bytes = &original_input[offset as usize..(offset + size) as usize];
        let mut reader = Cursor::new(image_bytes);

        let n = blp_header.mipmap_pixels(i);
        let indexed_rgb = reader.read_bytes(n as usize)?;

        let an = (n * blp_header.alpha_bits()).div_ceil(8);
        let indexed_alpha = reader.read_bytes(an as usize)?;

        images.push(Raw1Image {
            indexed_rgb,
            indexed_alpha,
        });
        Ok(())
    };

    read_image(0)?;
    if blp_header.has_mipmaps() {
        for (i, &size) in sizes.iter().enumerate().skip(1) {
            if size == 0 {
                break;
            }
            if i > blp_header.mipmaps_count() {
                break;
            }
            read_image(i)?;
        }
    }
    Ok(())
}
