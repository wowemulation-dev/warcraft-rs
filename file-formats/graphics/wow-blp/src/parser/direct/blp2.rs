use super::super::error::Error;
use super::super::types::Parser;
use crate::types::*;
use log::*;
use nom::{
    Err,
    error::context,
    multi::count,
    number::complete::{le_u8, le_u32},
};

pub fn parse_raw3<'a>(
    blp_header: &BlpHeader,
    original_input: &'a [u8],
    offsets: &[u32],
    sizes: &[u32],
    images: &mut Vec<Raw3Image>,
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

        trace!("Expecting size of image: {}", size);
        let image_bytes = &original_input[offset as usize..(offset + size) as usize];
        trace!("We have {} bytes", image_bytes.len());
        let n = blp_header.mipmap_pixels(i);
        trace!(
            "For mipmap size {:?} we should fetch {} bytes",
            blp_header.mipmap_size(i),
            n * 4
        );
        let (_, pixels) = count(le_u32, n as usize)(image_bytes)?;

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
                trace!("Size of mipmap {} is 0 bytes, I stop reading of images", i);
                break;
            }
            read_image(i)?;
        }
    }
    Ok((input, ()))
}

pub fn parse_dxtn<'a>(
    blp_header: &BlpHeader,
    dxtn: DxtnFormat,
    original_input: &'a [u8],
    offsets: &[u32],
    sizes: &[u32],
    images: &mut Vec<DxtnImage>,
    input: &'a [u8],
) -> Parser<'a, ()> {
    trace!("{:?}", blp_header);

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
        let (_, content) = context("dxtn blocks", count(le_u8, blocks_size))(image_bytes)?;
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
    Ok((input, ()))
}
