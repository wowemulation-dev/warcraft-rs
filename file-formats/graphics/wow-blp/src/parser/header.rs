use super::super::types::*;
use super::error::Error;
use super::reader::{ByteReader, Cursor, read_u32_array};
use super::types::ParseResult;
use log::*;
use std::str;

pub fn parse_header(input: &[u8]) -> ParseResult<BlpHeader> {
    let mut reader = Cursor::new(input);

    let version = parse_magic(&mut reader).map_err(|e| e.with_context("version"))?;
    let content_field = reader
        .read_u32_le()
        .map_err(|e| e.with_context("content_field field"))?;
    let content = content_field.try_into().unwrap_or_else(|_| {
        warn!("Unexpected value for content {content_field}, defaulting to jpeg");
        BlpContentTag::Jpeg
    });

    let mut flags = if version >= BlpVersion::Blp2 {
        let compression_field = reader
            .read_u8()
            .map_err(|e| e.with_context("compression field"))?;
        let compression: Compression = compression_field.try_into().map_err(|_| {
            error!("Unexpected value for compression {content_field}, defaulting to jpeg");
            Error::Blp2UnknownCompression(compression_field)
        })?;
        let alpha_bits = reader
            .read_u8()
            .map_err(|e| e.with_context("alpha_bits field"))?;
        let alpha_type_raw = reader
            .read_u8()
            .map_err(|e| e.with_context("alpha_type field"))?;
        let alpha_type = alpha_type_raw.try_into().map_err(|_| {
            warn!("Unknown alpha_type value {alpha_type_raw}, treating as raw value");
            // For now, we'll handle unknown alpha types gracefully
            // In a production system, you might want to return an error or use a fallback
            Error::UnknownAlphaType(alpha_type_raw)
        })?;
        let has_mipmaps = reader
            .read_u8()
            .map_err(|e| e.with_context("has_mipmaps field"))?;

        BlpFlags::Blp2 {
            compression,
            alpha_bits,
            alpha_type,
            has_mipmaps,
        }
    } else {
        let alpha_bits_raw = reader
            .read_u32_le()
            .map_err(|e| e.with_context("alpha_bits field"))?;
        let alpha_bits = if content == BlpContentTag::Jpeg
            && (alpha_bits_raw != 0 && alpha_bits_raw != 8)
        {
            warn!(
                "For jpeg content detected non standard alpha bits value {alpha_bits_raw} when 0 or 8 is expected, defaulting to 0"
            );
            0
        } else if content == BlpContentTag::Direct
            && (alpha_bits_raw != 0
                && alpha_bits_raw != 1
                && alpha_bits_raw != 4
                && alpha_bits_raw != 8)
        {
            warn!(
                "For direct content detected non standard alpha bits value {alpha_bits_raw} when 0, 1, 4 or 8 is expected, defaulting to 0"
            );
            0
        } else {
            alpha_bits_raw
        };

        BlpFlags::Old {
            alpha_bits,
            extra: 0,       // filled later
            has_mipmaps: 0, // filled later
        }
    };

    let width = reader
        .read_u32_le()
        .map_err(|e| e.with_context("width field"))?;
    let height = reader
        .read_u32_le()
        .map_err(|e| e.with_context("height field"))?;

    if let BlpFlags::Old {
        extra, has_mipmaps, ..
    } = &mut flags
    {
        let extra_value = reader
            .read_u32_le()
            .map_err(|e| e.with_context("extra field"))?;
        let has_mipmaps_value = reader
            .read_u32_le()
            .map_err(|e| e.with_context("has_mipmaps field"))?;
        *extra = extra_value;
        *has_mipmaps = has_mipmaps_value;
    }

    // Parse mipmap locator
    let mipmap_locator =
        parse_mipmap_locator(version, &mut reader).map_err(|e| e.with_context("mipmap locator"))?;

    Ok(BlpHeader {
        version,
        content,
        flags,
        width,
        height,
        mipmap_locator,
    })
}

fn parse_magic(reader: &mut impl ByteReader) -> ParseResult<BlpVersion> {
    let mut magic_fixed: [u8; 4] = Default::default();
    reader.read_into(&mut magic_fixed)?;

    let version = BlpVersion::from_magic(magic_fixed).ok_or_else(|| {
        Error::WrongMagic(
            str::from_utf8(&magic_fixed)
                .map(|s| s.to_owned())
                .unwrap_or_else(|_| format!("{magic_fixed:?}")),
        )
    })?;

    Ok(version)
}

fn parse_mipmap_locator(
    version: BlpVersion,
    reader: &mut impl ByteReader,
) -> ParseResult<MipmapLocator> {
    if version >= BlpVersion::Blp1 {
        let mut offsets: [u32; 16] = Default::default();
        let mut sizes: [u32; 16] = Default::default();

        let offsets_vec = read_u32_array(reader, 16)?;
        offsets.copy_from_slice(&offsets_vec);

        let sizes_vec = read_u32_array(reader, 16)?;
        sizes.copy_from_slice(&sizes_vec);

        Ok(MipmapLocator::Internal { offsets, sizes })
    } else {
        // For BLP0 mipmaps are located in external files
        Ok(MipmapLocator::External)
    }
}
