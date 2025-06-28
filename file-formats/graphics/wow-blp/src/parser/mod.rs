mod direct;
/// Error types for BLP parsing operations
pub mod error;
mod header;
mod jpeg;
/// Native byte reading utilities
mod reader;
/// Type definitions used by the BLP parser
pub mod types;

use super::types::*;
use crate::path::make_mipmap_path;
use direct::parse_direct_content;
pub use error::{Error, LoadError};
use header::parse_header;
use jpeg::parse_jpeg_content;
use std::path::{Path, PathBuf};
use types::ParseResult;

/// Read BLP file from file system. If it BLP0 format, uses the mipmaps near the root file.
pub fn load_blp<Q>(path: Q) -> Result<BlpImage, LoadError>
where
    Q: AsRef<Path>,
{
    let input =
        std::fs::read(&path).map_err(|e| LoadError::FileSystem(path.as_ref().to_owned(), e))?;
    load_blp_ex(Some(path), &input)
}

/// Read BLP file from buffer(`Vec<u8>`). If it BLP0 format, uses the mipmaps in the temp dir.
///
/// Since: 1.2.0
pub fn load_blp_from_buf(buf: &[u8]) -> Result<BlpImage, LoadError> {
    let input = buf;
    let path: Option<PathBuf> = None;
    load_blp_ex(path, input)
}

fn load_blp_ex<Q>(path: Option<Q>, input: &[u8]) -> Result<BlpImage, LoadError>
where
    Q: AsRef<Path>,
{
    // We have to preload all mipmaps in memory as we are constrained with lifetime that
    // should be equal of lifetime of root input stream.
    let mut mipmaps = vec![];
    if let Some(path) = path.as_ref() {
        for i in 0..16 {
            let mipmap_path = make_mipmap_path(path, i)
                .ok_or_else(|| LoadError::InvalidFilename(path.as_ref().to_owned()))?;
            if mipmap_path.is_file() {
                let mipmap = std::fs::read(mipmap_path)
                    .map_err(|e| LoadError::FileSystem(path.as_ref().to_owned(), e))?;
                mipmaps.push(mipmap);
            } else {
                break;
            }
        }
    }

    let image = parse_blp_with_externals(input, |i| preloaded_mipmaps(&mipmaps, i))
        .map_err(|e| LoadError::Parsing(format!("{e}")))?;
    Ok(image)
}

/// Parse BLP file from slice and fail if we require parse external files (case BLP0)
pub fn parse_blp(input: &[u8]) -> ParseResult<BlpImage> {
    parse_blp_with_externals(input, no_mipmaps)
}

/// Helper for `parse_blp` when no external mipmaps are needed
pub fn no_mipmaps<'a>(_: usize) -> Result<Option<&'a [u8]>, Box<dyn std::error::Error>> {
    Ok(None)
}

/// Helper for `parse_blp` when external mipmaps are located in filesystem near the
/// root file and loaded in memory when reading the main file.
pub fn preloaded_mipmaps(
    mipmaps: &[Vec<u8>],
    i: usize,
) -> Result<Option<&[u8]>, Box<dyn std::error::Error>> {
    if i >= mipmaps.len() {
        Ok(None)
    } else {
        Ok(Some(&mipmaps[i]))
    }
}

/// Parse BLP file from slice and use user provided callback to read mipmaps
pub fn parse_blp_with_externals<'a, F>(
    root_input: &'a [u8],
    external_mipmaps: F,
) -> ParseResult<BlpImage>
where
    F: FnMut(usize) -> Result<Option<&'a [u8]>, Box<dyn std::error::Error>> + Clone,
{
    // Parse header
    let header = parse_header(root_input).map_err(|e| e.with_context("header"))?;

    // Calculate where content starts (after header)
    let header_size = BlpHeader::size(header.version);
    if root_input.len() < header_size {
        return Err(Error::UnexpectedEof);
    }
    let content_input = &root_input[header_size..];

    // Parse image content
    let content = parse_content(&header, external_mipmaps.clone(), root_input, content_input)
        .map_err(|e| e.with_context("image content"))?;

    Ok(BlpImage { header, content })
}

fn parse_content<'a, F>(
    blp_header: &BlpHeader,
    external_mipmaps: F,
    original_input: &'a [u8],
    input: &'a [u8],
) -> ParseResult<BlpContent>
where
    F: FnMut(usize) -> Result<Option<&'a [u8]>, Box<dyn std::error::Error>> + Clone,
{
    match blp_header.content {
        BlpContentTag::Jpeg => {
            let content =
                parse_jpeg_content(blp_header, external_mipmaps.clone(), original_input, input)
                    .map_err(|e| e.with_context("jpeg content"))?;
            Ok(BlpContent::Jpeg(content))
        }
        BlpContentTag::Direct => {
            let content =
                parse_direct_content(blp_header, external_mipmaps.clone(), original_input, input)
                    .map_err(|e| e.with_context("direct content"))?;
            Ok(content)
        }
    }
}
