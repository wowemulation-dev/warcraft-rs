//! Native byte reading utilities for BLP parsing
//!
//! This module provides a trait and implementations for reading binary data
//! without external parser dependencies.

use super::error::Error;

/// Result type for parsing operations
pub type ParseResult<T> = Result<T, Error>;

/// Trait for reading binary data from a byte slice
pub trait ByteReader {
    /// Read a single unsigned 8-bit integer
    fn read_u8(&mut self) -> ParseResult<u8>;

    /// Read a single unsigned 32-bit integer in little-endian format
    fn read_u32_le(&mut self) -> ParseResult<u32>;

    /// Read exactly `n` bytes
    fn read_bytes(&mut self, n: usize) -> ParseResult<Vec<u8>>;

    /// Read exactly `n` bytes into a pre-allocated buffer
    fn read_into(&mut self, buf: &mut [u8]) -> ParseResult<()>;
}

/// A cursor for reading binary data from a byte slice
pub struct Cursor<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    /// Create a new cursor at the beginning of the data
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }
}

impl<'a> ByteReader for Cursor<'a> {
    fn read_u8(&mut self) -> ParseResult<u8> {
        if self.position >= self.data.len() {
            return Err(Error::UnexpectedEof);
        }
        let value = self.data[self.position];
        self.position += 1;
        Ok(value)
    }

    fn read_u32_le(&mut self) -> ParseResult<u32> {
        if self.position + 4 > self.data.len() {
            return Err(Error::UnexpectedEof);
        }
        let bytes = [
            self.data[self.position],
            self.data[self.position + 1],
            self.data[self.position + 2],
            self.data[self.position + 3],
        ];
        self.position += 4;
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_bytes(&mut self, n: usize) -> ParseResult<Vec<u8>> {
        if self.position + n > self.data.len() {
            return Err(Error::UnexpectedEof);
        }
        let bytes = self.data[self.position..self.position + n].to_vec();
        self.position += n;
        Ok(bytes)
    }

    fn read_into(&mut self, buf: &mut [u8]) -> ParseResult<()> {
        let n = buf.len();
        if self.position + n > self.data.len() {
            return Err(Error::UnexpectedEof);
        }
        buf.copy_from_slice(&self.data[self.position..self.position + n]);
        self.position += n;
        Ok(())
    }
}

/// Helper function to read an array of u32 values
pub fn read_u32_array(reader: &mut impl ByteReader, count: usize) -> ParseResult<Vec<u32>> {
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(reader.read_u32_le()?);
    }
    Ok(values)
}
