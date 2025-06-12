use super::super::types::*;
use super::error::Error;
pub use super::types::Parser;
use log::*;
use nom::{
    Err,
    bytes::complete::take,
    error::context,
    multi::count,
    number::complete::{le_u8, le_u32},
};
use std::str;

pub fn parse_header(input: &[u8]) -> Parser<BlpHeader> {
    let (input, version) = context("version", parse_magic)(input)?;
    let (input, content_field) = context("content_field field", le_u32)(input)?;
    let content = content_field.try_into().unwrap_or_else(|_| {
        warn!(
            "Unexpected value for content {}, defaulting to jpeg",
            content_field
        );
        BlpContentTag::Jpeg
    });
    let (input, mut flags) = if version >= BlpVersion::Blp2 {
        let (input, compression_field) = context("compression field", le_u8)(input)?;
        let compression: Compression = compression_field.try_into().map_err(|_| {
            error!(
                "Unexpected value for compression {}, defaulting to jpeg",
                content_field
            );
            Err::Failure(Error::<&[u8]>::Blp2UnknownCompression(compression_field))
        })?;
        let (input, alpha_bits) = context("alpha_bits field", le_u8)(input)?;
        let (input, alpha_type) = context("alpha_type field", le_u8)(input)?;
        let (input, has_mipmaps) = context("has_mipmaps field", le_u8)(input)?;
        (
            input,
            BlpFlags::Blp2 {
                compression,
                alpha_bits,
                alpha_type,
                has_mipmaps,
            },
        )
    } else {
        let (input, alpha_bits_raw) = context("alpha_bits field", le_u32)(input)?;
        let alpha_bits = if content == BlpContentTag::Jpeg
            && (alpha_bits_raw != 0 && alpha_bits_raw != 8)
        {
            warn!(
                "For jpeg content detected non standard alpha bits value {} when 0 or 8 is expected, defaulting to 0",
                alpha_bits_raw
            );
            0
        } else if content == BlpContentTag::Direct
            && (alpha_bits_raw != 0
                && alpha_bits_raw != 1
                && alpha_bits_raw != 4
                && alpha_bits_raw != 8)
        {
            warn!(
                "For direct content detected non standard alpha bits value {} when 0, 1, 4 or 8 is expected, defaulting to 0",
                alpha_bits_raw
            );
            0
        } else {
            alpha_bits_raw
        };
        (
            input,
            BlpFlags::Old {
                alpha_bits,
                extra: 0,       // filled later
                has_mipmaps: 0, // filled later
            },
        )
    };
    let (input, width) = context("width field", le_u32)(input)?;
    let (input, height) = context("height field", le_u32)(input)?;
    let input = if let BlpFlags::Old {
        extra, has_mipmaps, ..
    } = &mut flags
    {
        let (input, extra_value) = context("extra field", le_u32)(input)?;
        let (input, has_mipmaps_value) = context("has_mipmaps field", le_u32)(input)?;
        *extra = extra_value;
        *has_mipmaps = has_mipmaps_value;
        input
    } else {
        input
    };

    // Parse mipmap locator
    let (input, mipmap_locator) = context("mipmap locator", |input| {
        parse_mipmap_locator(version, input)
    })(input)?;

    Ok((
        input,
        BlpHeader {
            version,
            content,
            flags,
            width,
            height,
            mipmap_locator,
        },
    ))
}

fn parse_magic(input: &[u8]) -> Parser<BlpVersion> {
    let mut magic_fixed: [u8; 4] = Default::default();
    let (input, magic) = take(4_u32)(input)?;
    magic_fixed.copy_from_slice(magic);
    let version = BlpVersion::from_magic(magic_fixed).ok_or_else(|| {
        Err::Failure(Error::WrongMagic(
            str::from_utf8(magic)
                .map(|s| s.to_owned())
                .unwrap_or_else(|_| format!("{:?}", magic)),
        ))
    })?;

    Ok((input, version))
}

fn parse_mipmap_locator(version: BlpVersion, input: &[u8]) -> Parser<MipmapLocator> {
    if version >= BlpVersion::Blp1 {
        let mut offsets: [u32; 16] = Default::default();
        let mut sizes: [u32; 16] = Default::default();
        let (input, offsets_vec) = count(le_u32, 16)(input)?;
        offsets.copy_from_slice(&offsets_vec);
        let (input, sizes_vec) = count(le_u32, 16)(input)?;
        sizes.copy_from_slice(&sizes_vec);
        Ok((input, MipmapLocator::Internal { offsets, sizes }))
    } else {
        // For BLP0 mipmaps are located in external files
        Ok((input, MipmapLocator::External))
    }
}
