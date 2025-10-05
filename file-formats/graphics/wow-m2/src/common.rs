use crate::error::{M2Error, Result};
use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Seek, SeekFrom, Write};

/// Trait for parsing and writing types to and from the M2 binary file format.
///
/// Types implementing `M2Parse` can be deserialized from a binary reader and serialized to a binary writer.
/// This trait is used throughout the M2 parsing code to provide a generic interface for reading and writing
/// primitive types, vectors, and complex structures in a version-agnostic way.
pub trait M2Parse {
    /// Parse an instance of the type from the given reader.
    ///
    /// # Arguments
    /// * `reader` - A mutable reference to a type implementing `Read + Seek`.
    ///
    /// # Returns
    /// * `Result<Self>` - The parsed instance or an error.
    fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self>
    where
        Self: Sized;

    /// Write this instance to the given writer.
    ///
    /// # Arguments
    /// * `writer` - A mutable reference to a type implementing `Write`.
    ///
    /// # Returns
    /// * `Result<()>` - Ok if successful, or an error.
    fn write<W: Write>(&self, writer: &mut W) -> Result<()>;
}

impl M2Parse for f32 {
    fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_f32_le()?)
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_f32_le(*self)?;
        Ok(())
    }
}

impl M2Parse for u16 {
    fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_u16_le()?)
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u16_le(*self)?;
        Ok(())
    }
}

impl M2Parse for u32 {
    fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_u32_le()?)
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32_le(*self)?;
        Ok(())
    }
}

/// Represents a vector of elements in the M2 file format, along with its array reference.
///
/// `M2Vec` stores both the `M2Array` reference (count and offset in the file) and the actual data elements.
/// This is used for fields in M2 structs that point to arrays of data elsewhere in the file.
///
/// The `data` field is populated when parsing, but only the `array` field is written when serializing,
/// as the actual data is written separately at the correct offset.
#[derive(Debug, Clone, Default)]
pub struct M2Vec<T: M2Parse> {
    /// Reference to the array in the file (count and offset)
    pub array: M2Array<T>,
    /// The actual data elements (populated when parsing)
    pub data: Vec<T>,
}

impl<T: M2Parse> M2Vec<T> {
    /// Create a new, empty `M2Vec` with no data and a zeroed array reference.
    pub fn new() -> Self {
        Self {
            array: M2Array::new(0, 0),
            data: Vec::new(),
        }
    }
}

impl<T: M2Parse> M2Parse for M2Vec<T> {
    fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let array = M2Array::<T>::parse(reader)?;
        let pos = reader.stream_position()?;
        let data = read_array(reader, &array, |r| T::parse(r))?;
        reader.seek(SeekFrom::Start(pos))?;
        Ok(Self { array, data })
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.array.write(writer)?;
        Ok(())
    }
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

impl M2Parse for C3Vector {
    fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        C3Vector::parse(reader)
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.write(writer)
    }
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

/// A vector in 2D space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct C2Vector {
    pub x: f32,
    pub y: f32,
}

impl M2Parse for C2Vector {
    fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        C2Vector::parse(reader)
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.write(writer)
    }
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
