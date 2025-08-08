use std::io::{Read, Result, Write};

/// Extension trait for reading little-endian values from a reader
pub trait ReadExt: Read {
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_u16_le(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    fn read_u32_le(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    fn read_u64_le(&mut self) -> Result<u64> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    fn read_i8(&mut self) -> Result<i8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(i8::from_le_bytes(buf))
    }

    fn read_i16_le(&mut self) -> Result<i16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }

    fn read_i32_le(&mut self) -> Result<i32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }

    fn read_i64_le(&mut self) -> Result<i64> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)?;
        Ok(i64::from_le_bytes(buf))
    }

    fn read_f32_le(&mut self) -> Result<f32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }

    fn read_f64_le(&mut self) -> Result<f64> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)?;
        Ok(f64::from_le_bytes(buf))
    }
}

/// Extension trait for writing little-endian values to a writer
pub trait WriteExt: Write {
    fn write_u8(&mut self, n: u8) -> Result<()> {
        self.write_all(&[n])
    }

    fn write_u16_le(&mut self, n: u16) -> Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    fn write_u32_le(&mut self, n: u32) -> Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    fn write_u64_le(&mut self, n: u64) -> Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    fn write_i8(&mut self, n: i8) -> Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    fn write_i16_le(&mut self, n: i16) -> Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    fn write_i32_le(&mut self, n: i32) -> Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    fn write_i64_le(&mut self, n: i64) -> Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    fn write_f32_le(&mut self, n: f32) -> Result<()> {
        self.write_all(&n.to_le_bytes())
    }

    fn write_f64_le(&mut self, n: f64) -> Result<()> {
        self.write_all(&n.to_le_bytes())
    }
}

// Implement the traits for all types that implement Read/Write
impl<R: Read + ?Sized> ReadExt for R {}
impl<W: Write + ?Sized> WriteExt for W {}
