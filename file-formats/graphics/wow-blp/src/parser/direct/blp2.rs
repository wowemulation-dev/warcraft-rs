use super::super::error::Error;
use super::super::reader::{ByteReader, Cursor, read_u32_array};
use super::super::types::ParseResult;
use crate::types::*;
use log::*;

pub fn parse_raw3<'a>(
    blp_header: &BlpHeader,
    original_input: &'a [u8],
    offsets: &[u32],
    sizes: &[u32],
    images: &mut Vec<Raw3Image>,
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

        trace!("Expecting size of image: {size}");
        let image_bytes = &original_input[offset as usize..(offset + size) as usize];
        trace!("We have {} bytes", image_bytes.len());
        let n = blp_header.mipmap_pixels(i);
        trace!(
            "For mipmap size {:?} we should fetch {} bytes",
            blp_header.mipmap_size(i),
            n * 4
        );

        let mut reader = Cursor::new(image_bytes);
        let pixels = read_u32_array(&mut reader, n as usize)?;

        images.push(Raw3Image { pixels });
        Ok(())
    };

    trace!("Mipmaps count: {}", blp_header.mipmaps_count());
    read_image(0)?;
    if blp_header.has_mipmaps() {
        for (i, &size) in sizes
            .iter()
            .enumerate()
            .take((blp_header.mipmaps_count() + 1).min(16))
            .skip(1)
        {
            if size == 0 {
                trace!("Size of mipmap {i} is 0 bytes, I stop reading of images");
                break;
            }
            read_image(i)?;
        }
    }
    Ok(())
}

pub fn parse_dxtn<'a>(
    blp_header: &BlpHeader,
    dxtn: DxtnFormat,
    original_input: &'a [u8],
    offsets: &[u32],
    sizes: &[u32],
    images: &mut Vec<DxtnImage>,
    _input: &'a [u8],
) -> ParseResult<()> {
    trace!("{blp_header:?}");

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
        let n = blp_header.mipmap_pixels(i);
        let blocks_n = ((n as f32) / 16.0).ceil() as usize;
        let mut blocks_size = blocks_n * dxtn.block_size();
        trace!("Dxtn blocks count: {blocks_n}");
        trace!("Dxtn format: {dxtn:?}, block size: {}", dxtn.block_size());
        trace!(
            "Left size: {}, expected size: {}",
            image_bytes.len(),
            blocks_size
        );
        if blocks_size > image_bytes.len() {
            warn!("Data is smaller than expected! Trying to read only whole number of blocks");
            let new_blocks_n = image_bytes.len() / dxtn.block_size();
            warn!("Reading {new_blocks_n} blocks");
            blocks_size = new_blocks_n * dxtn.block_size();
        }

        let mut reader = Cursor::new(image_bytes);
        let content = reader
            .read_bytes(blocks_size)
            .map_err(|e| e.with_context("dxtn blocks"))?;
        images.push(DxtnImage { content });
        Ok(())
    };

    read_image(0)?;
    if blp_header.has_mipmaps() {
        trace!("Mipmaps count: {}", blp_header.mipmaps_count());
        for i in 1..(blp_header.mipmaps_count() + 1).min(16) {
            read_image(i)?;
        }
    }
    Ok(())
}
