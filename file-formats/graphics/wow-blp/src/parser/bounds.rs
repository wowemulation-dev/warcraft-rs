//! Bounds checking utilities for BLP parsers

use super::error::Error;
use super::types::ParseResult;
use log::error;

/// Check if a given offset and size are within the bounds of input data
///
/// This helper function consolidates the bounds checking logic that was duplicated
/// across multiple BLP parsing functions.
pub fn check_bounds(input: &[u8], offset: u32, size: u32, mipmap_index: usize) -> ParseResult<()> {
    // Check if offset is within input bounds
    if offset as usize >= input.len() {
        error!(
            "Offset of mipmap {} is out of bounds! {} >= {}",
            mipmap_index,
            offset,
            input.len()
        );
        return Err(Error::OutOfBounds {
            offset: offset as usize,
            size: 0,
        });
    }

    // Check if offset + size extends beyond input bounds
    if (offset + size) as usize > input.len() {
        error!(
            "Offset+size of mipmap {} is out of bounds! {} > {}",
            mipmap_index,
            offset + size,
            input.len()
        );
        return Err(Error::OutOfBounds {
            offset: offset as usize,
            size: size as usize,
        });
    }

    Ok(())
}

/// Get a slice from input data after bounds checking
///
/// Convenience function that checks bounds and returns the slice if valid.
pub fn get_bounded_slice(
    input: &[u8],
    offset: u32,
    size: u32,
    mipmap_index: usize,
) -> ParseResult<&[u8]> {
    check_bounds(input, offset, size, mipmap_index)?;
    Ok(&input[offset as usize..(offset + size) as usize])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_bounds() {
        let data = vec![1, 2, 3, 4, 5];
        assert!(check_bounds(&data, 0, 3, 0).is_ok());
        assert!(check_bounds(&data, 2, 3, 0).is_ok());
        assert!(check_bounds(&data, 0, 5, 0).is_ok());
    }

    #[test]
    fn test_offset_out_of_bounds() {
        let data = vec![1, 2, 3];
        assert!(check_bounds(&data, 5, 1, 0).is_err());
        assert!(check_bounds(&data, 3, 1, 0).is_err()); // offset == len is out of bounds
    }

    #[test]
    fn test_size_out_of_bounds() {
        let data = vec![1, 2, 3];
        assert!(check_bounds(&data, 1, 3, 0).is_err()); // 1 + 3 > 3
        assert!(check_bounds(&data, 0, 4, 0).is_err()); // 0 + 4 > 3
    }

    #[test]
    fn test_get_bounded_slice() {
        let data = vec![1, 2, 3, 4, 5];
        let slice = get_bounded_slice(&data, 1, 3, 0).unwrap();
        assert_eq!(slice, &[2, 3, 4]);
    }
}
