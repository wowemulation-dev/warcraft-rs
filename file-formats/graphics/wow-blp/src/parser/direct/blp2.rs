use super::super::bounds::get_bounded_slice;
use super::super::error::Error;
use super::super::reader::{read_u32_array, ByteReader, Cursor};
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
        trace!("Expecting size of image: {size}");
        let image_bytes = get_bounded_slice(original_input, offset, size, i)?;
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
        let offset = usize::try_from(offset).map_err(|_| Error::OutOfBounds {
            offset: usize::MAX,
            size: 0,
        })?;
        let size = usize::try_from(size).map_err(|_| Error::OutOfBounds {
            offset,
            size: usize::MAX,
        })?;
        let end = offset
            .checked_add(size)
            .ok_or(Error::OutOfBounds { offset, size })?;
        if offset > original_input.len() {
            error!(
                "Offset of mipmap {} is out of bounds! {} >= {}",
                i,
                offset,
                original_input.len()
            );
            return Err(Error::OutOfBounds { offset, size: 0 });
        }
        if end > original_input.len() {
            error!(
                "Offset+size of mipmap {} is out of bounds! {} > {}",
                i,
                end,
                original_input.len()
            );
            return Err(Error::OutOfBounds { offset, size });
        }

        let image_bytes = &original_input[offset..end];
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
            if sizes[i] == 0 {
                trace!("Size of mipmap {i} is 0 bytes, I stop reading of images");
                break;
            }
            read_image(i)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies zero-sized trailing mipmap slots terminate a valid BLP2 chain.
    #[test]
    fn zero_sized_trailing_mipmaps_are_ignored() {
        let header = BlpHeader {
            version: BlpVersion::Blp2,
            content: BlpContentTag::Direct,
            flags: BlpFlags::Blp2 {
                compression: Compression::Dxtc,
                alpha_bits: 8,
                alpha_type: AlphaType::EightBit,
                has_mipmaps: 1,
            },
            width: 128,
            height: 512,
            mipmap_locator: MipmapLocator::Internal {
                offsets: [0; 16],
                sizes: [0; 16],
            },
        };
        let data = vec![0; 65_536];
        let mut offsets = [0; 16];
        let mut sizes = [0; 16];
        sizes[0] = data.len() as u32;
        offsets[1] = data.len() as u32;
        let mut images = Vec::new();

        parse_dxtn(
            &header,
            DxtnFormat::Dxt5,
            &data,
            &offsets,
            &sizes,
            &mut images,
            &data,
        )
        .expect("zero-sized mipmap terminator should be valid");
        assert_eq!(images.len(), 1);
    }
}
