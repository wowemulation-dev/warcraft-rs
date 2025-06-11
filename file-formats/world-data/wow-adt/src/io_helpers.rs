// io_helpers.rs - Helper functions for reading/writing little-endian values

use crate::error::{AdtError, Result};
use std::io::{Read, Write};

/// Extension trait for reading little-endian values
pub trait ReadLittleEndian: Read {
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)
            .map_err(|_| AdtError::UnexpectedEof)?;
        Ok(buf[0])
    }

    fn read_u16_le(&mut self) -> Result<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)
            .map_err(|_| AdtError::UnexpectedEof)?;
        Ok(u16::from_le_bytes(buf))
    }

    fn read_u32_le(&mut self) -> Result<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)
            .map_err(|_| AdtError::UnexpectedEof)?;
        Ok(u32::from_le_bytes(buf))
    }

    fn read_u64_le(&mut self) -> Result<u64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)
            .map_err(|_| AdtError::UnexpectedEof)?;
        Ok(u64::from_le_bytes(buf))
    }

    fn read_i8(&mut self) -> Result<i8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)
            .map_err(|_| AdtError::UnexpectedEof)?;
        Ok(i8::from_le_bytes(buf))
    }

    #[allow(dead_code)]
    fn read_i16_le(&mut self) -> Result<i16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)
            .map_err(|_| AdtError::UnexpectedEof)?;
        Ok(i16::from_le_bytes(buf))
    }

    #[allow(dead_code)]
    fn read_i32_le(&mut self) -> Result<i32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)
            .map_err(|_| AdtError::UnexpectedEof)?;
        Ok(i32::from_le_bytes(buf))
    }

    fn read_f32_le(&mut self) -> Result<f32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)
            .map_err(|_| AdtError::UnexpectedEof)?;
        Ok(f32::from_le_bytes(buf))
    }
}

/// Extension trait for writing little-endian values
pub trait WriteLittleEndian: Write {
    fn write_u8(&mut self, value: u8) -> Result<()> {
        self.write_all(&[value]).map_err(AdtError::Io)
    }

    fn write_u16_le(&mut self, value: u16) -> Result<()> {
        self.write_all(&value.to_le_bytes()).map_err(AdtError::Io)
    }

    fn write_u32_le(&mut self, value: u32) -> Result<()> {
        self.write_all(&value.to_le_bytes()).map_err(AdtError::Io)
    }

    #[allow(dead_code)]
    fn write_u64_le(&mut self, value: u64) -> Result<()> {
        self.write_all(&value.to_le_bytes()).map_err(AdtError::Io)
    }

    #[allow(dead_code)]
    fn write_i16_le(&mut self, value: i16) -> Result<()> {
        self.write_all(&value.to_le_bytes()).map_err(AdtError::Io)
    }

    #[allow(dead_code)]
    fn write_i32_le(&mut self, value: i32) -> Result<()> {
        self.write_all(&value.to_le_bytes()).map_err(AdtError::Io)
    }

    fn write_f32_le(&mut self, value: f32) -> Result<()> {
        self.write_all(&value.to_le_bytes()).map_err(AdtError::Io)
    }
}

// Implement for all types that implement Read/Write
impl<R: Read + ?Sized> ReadLittleEndian for R {}
impl<W: Write + ?Sized> WriteLittleEndian for W {}
