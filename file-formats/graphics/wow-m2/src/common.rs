use custom_debug::Debug;
use wow_utils::debug;

use crate::error::{M2Error, Result};
use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Seek, SeekFrom, Write};

pub trait ItemParser<T> {
    fn parse<R: Read + Seek>(reader: &mut R) -> Result<T>;
}

/// A reference to an array in the M2 file format
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct M2Array<T> {
    /// Number of elements in the array
    pub count: u32,
    /// Offset from the start of the file to the array
    pub offset: u32,
    /// Phantom data to associate with the type T
    _phantom: std::marker::PhantomData<T>,
}

impl<T> M2Array<T> {
    /// Create a new array reference
    pub fn new(count: u32, offset: u32) -> Self {
        Self {
            count,
            offset,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Parse an array reference from a reader
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let count = reader.read_u32_le()?;
        let offset = reader.read_u32_le()?;

        Ok(Self {
            count,
            offset,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Write an array reference to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32_le(self.count)?;
        writer.write_u32_le(self.offset)?;

        Ok(())
    }

    /// Check if the array is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Convert this reference to a reference of another type
    pub fn convert<U>(&self) -> M2Array<U> {
        M2Array {
            count: self.count,
            offset: self.offset,
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Reads data at an array reference location
pub fn read_array<T, R, F>(reader: &mut R, array: &M2Array<T>, parse_fn: F) -> Result<Vec<T>>
where
    R: Read + Seek,
    F: Fn(&mut R) -> Result<T>,
{
    if array.is_empty() {
        return Ok(Vec::new());
    }

    // Seek to the array data
    reader
        .seek(std::io::SeekFrom::Start(array.offset as u64))
        .map_err(M2Error::Io)?;

    // Read each element
    let mut result = Vec::with_capacity(array.count as usize);
    for _ in 0..array.count {
        result.push(parse_fn(reader)?);
    }

    Ok(result)
}

/// A vector in 3D space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct C3Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl C3Vector {
    /// Parse a C3Vector from a reader
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let x = reader.read_f32_le()?;
        let y = reader.read_f32_le()?;
        let z = reader.read_f32_le()?;

        Ok(Self { x, y, z })
    }

    /// Write a C3Vector to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_f32_le(self.x)?;
        writer.write_f32_le(self.y)?;
        writer.write_f32_le(self.z)?;

        Ok(())
    }

    /// Convert to a glam vector for easier math operations
    pub fn to_glam(&self) -> glam::Vec3 {
        glam::Vec3::new(self.x, self.y, self.z)
    }

    /// Create from a glam vector
    pub fn from_glam(v: glam::Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl ItemParser<C3Vector> for C3Vector {
    fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Self::parse(reader)
    }
}

/// A vector in 2D space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct C2Vector {
    pub x: f32,
    pub y: f32,
}

impl C2Vector {
    /// Parse a C2Vector from a reader
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let x = reader.read_f32_le()?;
        let y = reader.read_f32_le()?;

        Ok(Self { x, y })
    }

    /// Write a C2Vector to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_f32_le(self.x)?;
        writer.write_f32_le(self.y)?;

        Ok(())
    }

    /// Convert to a glam vector for easier math operations
    pub fn to_glam(&self) -> glam::Vec2 {
        glam::Vec2::new(self.x, self.y)
    }

    /// Create from a glam vector
    pub fn from_glam(v: glam::Vec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

/// A fixed-width string with a specified maximum length
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FixedString {
    #[debug(with = debug::trimmed_collection_fmt)]
    pub data: Vec<u8>,
}

impl FixedString {
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the string is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Parse a fixed-width string from a reader
    pub fn parse<R: Read + Seek>(reader: &mut R, len: usize) -> Result<Self> {
        let mut data = vec![0u8; len];
        reader.read_exact(&mut data)?;

        // Find null terminator
        let null_pos = data.iter().position(|&b| b == 0).unwrap_or(len);
        data.truncate(null_pos);

        Ok(Self { data })
    }

    /// Write a fixed-width string to a writer
    pub fn write<W: Write>(&self, writer: &mut W, len: usize) -> Result<()> {
        let mut data = self.data.clone();
        data.resize(len, 0);
        writer.write_all(&data)?;

        Ok(())
    }

    /// Convert to a string, lossy UTF-8 conversion
    pub fn to_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.data).to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct M2ArrayString {
    pub string: FixedString,
    pub array: M2Array<u8>,
}

impl M2ArrayString {
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let array = M2Array::<u8>::parse(reader)?;
        let current_pos = reader.stream_position()?;
        reader.seek(SeekFrom::Start(array.offset as u64))?;
        let string = FixedString::parse(reader, array.count as usize)?;
        reader.seek(SeekFrom::Start(current_pos))?;
        Ok(Self { string, array })
    }

    pub fn is_empty(&self) -> bool {
        self.array.is_empty()
    }

    /// Write a reference to our array to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32_le(self.array.count)?;
        writer.write_u32_le(self.array.offset)?;

        Ok(())
    }
}

/// A quaternion for rotations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    /// Parse a quaternion from a reader
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let x = reader.read_f32_le()?;
        let y = reader.read_f32_le()?;
        let z = reader.read_f32_le()?;
        let w = reader.read_f32_le()?;

        Ok(Self { x, y, z, w })
    }

    /// Write a quaternion to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_f32_le(self.x)?;
        writer.write_f32_le(self.y)?;
        writer.write_f32_le(self.z)?;
        writer.write_f32_le(self.w)?;

        Ok(())
    }

    /// Convert to a glam quaternion for easier math operations
    pub fn to_glam(&self) -> glam::Quat {
        glam::Quat::from_xyzw(self.x, self.y, self.z, self.w)
    }

    /// Create from a glam quaternion
    pub fn from_glam(q: glam::Quat) -> Self {
        Self {
            x: q.x,
            y: q.y,
            z: q.z,
            w: q.w,
        }
    }
}

#[inline]
pub fn i16_to_f32(value: i16) -> f32 {
    (value as f32 / i16::MAX as f32) - 1.0
}

impl From<Quaternion16> for Quaternion {
    fn from(value: Quaternion16) -> Self {
        Self {
            x: i16_to_f32(value.x),
            y: i16_to_f32(value.y),
            z: i16_to_f32(value.z),
            w: i16_to_f32(value.w),
        }
    }
}

impl ItemParser<Quaternion> for Quaternion {
    fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Self::parse(reader)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion16 {
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub w: i16,
}

impl Quaternion16 {
    /// Parse a quaternion from a reader
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let x = reader.read_i16_le()?;
        let y = reader.read_i16_le()?;
        let z = reader.read_i16_le()?;
        let w = reader.read_i16_le()?;

        Ok(Self { x, y, z, w })
    }

    /// Write a quaternion to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_i16_le(self.x)?;
        writer.write_i16_le(self.y)?;
        writer.write_i16_le(self.z)?;
        writer.write_i16_le(self.w)?;

        Ok(())
    }
}

impl ItemParser<Quaternion16> for Quaternion16 {
    fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Self::parse(reader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_m2array_parse() {
        let data = [
            0x05, 0x00, 0x00, 0x00, // count = 5
            0x20, 0x30, 0x00, 0x00, // offset = 0x3020
        ];

        let mut cursor = Cursor::new(data);
        let array = M2Array::<u32>::parse(&mut cursor).unwrap();

        assert_eq!(array.count, 5);
        assert_eq!(array.offset, 0x3020);
    }

    #[test]
    fn test_m2array_write() {
        let array = M2Array::<u32>::new(5, 0x3020);
        let mut cursor = Cursor::new(Vec::new());

        array.write(&mut cursor).unwrap();

        let data = cursor.into_inner();
        assert_eq!(
            data,
            [
                0x05, 0x00, 0x00, 0x00, // count = 5
                0x20, 0x30, 0x00, 0x00, // offset = 0x3020
            ]
        );
    }

    #[test]
    fn test_c3vector_parse() {
        let data = [
            0x00, 0x00, 0x80, 0x3F, // x = 1.0
            0x00, 0x00, 0x00, 0x40, // y = 2.0
            0x00, 0x00, 0x40, 0x40, // z = 3.0
        ];

        let mut cursor = Cursor::new(data);
        let vector = C3Vector::parse(&mut cursor).unwrap();

        assert_eq!(vector.x, 1.0);
        assert_eq!(vector.y, 2.0);
        assert_eq!(vector.z, 3.0);
    }

    #[test]
    fn test_c2vector_parse() {
        let data = [
            0x00, 0x00, 0x80, 0x3F, // x = 1.0
            0x00, 0x00, 0x00, 0x40, // y = 2.0
        ];

        let mut cursor = Cursor::new(data);
        let vector = C2Vector::parse(&mut cursor).unwrap();

        assert_eq!(vector.x, 1.0);
        assert_eq!(vector.y, 2.0);
    }

    #[test]
    fn test_fixed_string_parse() {
        let data = [b'T', b'e', b's', b't', 0, 0, 0, 0];

        let mut cursor = Cursor::new(data);
        let string = FixedString::parse(&mut cursor, 8).unwrap();

        assert_eq!(string.data, b"Test");
        assert_eq!(string.to_string_lossy(), "Test");
    }
}
