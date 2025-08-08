use custom_debug::Debug;
use wow_utils::debug;

use crate::M2Version;
use crate::error::{M2Error, Result};
use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Seek, SeekFrom, Write};

pub trait ItemParser<T> {
    fn parse<R: Read + Seek>(reader: &mut R) -> Result<T>;
}

pub trait M2DataR: Sized {
    fn m2_read<R: Read + Seek>(reader: &mut R) -> Result<Self>;
}

pub trait M2DataRV: Sized {
    fn m2_read<R: Read + Seek>(reader: &mut R, version: M2Version) -> Result<Self>;
}

pub trait M2DataW {
    fn m2_write<W: Write>(&self, writer: &mut W) -> Result<()>;
    fn m2_size(&self) -> usize;
}

pub trait M2DataConversible: M2DataW + Sized {
    fn m2_write_version<W: Write>(&self, writer: &mut W, version: M2Version) -> Result<()> {
        let converted = self.m2_convert(version)?;
        converted.m2_write(writer)
    }
    fn m2_convert(&self, to_version: M2Version) -> Result<Self>;
}

impl M2DataR for M2Version {
    fn m2_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        M2Version::from_header_version(reader.read_u32_le()?)
    }
}

impl M2DataR for u32 {
    fn m2_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_u32_le()?)
    }
}
impl M2DataW for u32 {
    fn m2_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32_le(*self)?;
        Ok(())
    }

    fn m2_size(&self) -> usize {
        4
    }
}

impl M2DataR for f32 {
    fn m2_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_f32_le()?)
    }
}
impl M2DataW for f32 {
    fn m2_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_f32_le(*self)?;
        Ok(())
    }

    fn m2_size(&self) -> usize {
        4
    }
}

impl M2DataR for i16 {
    fn m2_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_i16_le()?)
    }
}
impl M2DataW for i16 {
    fn m2_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_i16_le(*self)?;
        Ok(())
    }

    fn m2_size(&self) -> usize {
        2
    }
}

impl M2DataR for u16 {
    fn m2_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_u16_le()?)
    }
}
impl M2DataW for u16 {
    fn m2_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u16_le(*self)?;
        Ok(())
    }

    fn m2_size(&self) -> usize {
        2
    }
}

pub trait M2Reader<T: M2DataR>: Read + Seek + Sized {
    fn m2_read(&mut self) -> Result<T> {
        Ok(T::m2_read(self)?)
    }
}
impl<T: M2DataR, R: Read + Seek> M2Reader<T> for R {}

pub trait M2ReaderV<T: M2DataRV>: Read + Seek + Sized {
    fn m2_read_versioned(&mut self, version: M2Version) -> Result<T> {
        Ok(T::m2_read(self, version)?)
    }
}
impl<T: M2DataRV, R: Read + Seek> M2ReaderV<T> for R {}

pub trait M2Writer<T: M2DataW>: Write + Sized {
    fn m2_write(&mut self, value: &T) -> Result<()> {
        value.m2_write(self)?;
        Ok(())
    }
}
impl<T: M2DataW, W: Write> M2Writer<T> for W {}

pub trait M2WriterV<T: M2DataConversible>: Write + Sized {
    fn m2_write_versioned(&mut self, value: &T, version: M2Version) -> Result<()> {
        value.m2_write_version(self, version)?;
        Ok(())
    }
}
impl<T: M2DataConversible, W: Write> M2WriterV<T> for W {}

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

    pub fn add_offset(&mut self, offset: usize) {
        self.offset += offset as u32;
    }
}

impl<T> M2DataR for M2Array<T> {
    fn m2_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok(Self::new(reader.m2_read()?, reader.m2_read()?))
    }
}
impl<T> M2DataW for M2Array<T> {
    fn m2_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.m2_write(&self.count)?;
        writer.m2_write(&self.offset)?;
        Ok(())
    }

    fn m2_size(&self) -> usize {
        self.count.m2_size() + self.offset.m2_size()
    }
}

impl<T: M2DataR> M2Array<T> {
    pub fn m2_read_to_vec<R: Read + Seek>(&self, reader: &mut R) -> Result<Vec<T>> {
        if self.is_empty() {
            return Ok(Vec::new());
        }

        // Seek to the array data
        reader
            .seek(SeekFrom::Start(self.offset as u64))
            .map_err(M2Error::Io)?;

        // Read each element
        let mut result = Vec::with_capacity(self.count as usize);
        for _ in 0..self.count {
            result.push(T::m2_read(reader)?);
        }

        Ok(result)
    }
}

pub trait M2Vec<T: M2DataW> {
    fn m2_write<W: Write + Seek>(&self, writer: &mut W) -> Result<M2Array<T>>;
    fn m2_size(&self) -> usize;
}

impl<T: M2DataW> M2Vec<T> for Vec<T> {
    fn m2_write<W: Write + Seek>(&self, writer: &mut W) -> Result<M2Array<T>> {
        let offset = writer.stream_position()?;
        for item in self {
            writer.m2_write(item)?
        }
        Ok(M2Array::<T>::new(self.len() as u32, offset as u32))
    }

    fn m2_size(&self) -> usize {
        if self.is_empty() {
            0
        } else {
            self.len() * self[0].m2_size()
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
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct C3Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl C3Vector {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

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

    pub fn origin() -> Self {
        Self {
            x: 0.,
            y: 0.,
            z: 0.,
        }
    }
}

impl ItemParser<C3Vector> for C3Vector {
    fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Self::parse(reader)
    }
}

impl M2DataR for C3Vector {
    fn m2_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let x = reader.m2_read()?;
        let y = reader.m2_read()?;
        let z = reader.m2_read()?;

        Ok(Self { x, y, z })
    }
}
impl M2DataW for C3Vector {
    fn m2_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.m2_write(&self.x)?;
        writer.m2_write(&self.y)?;
        writer.m2_write(&self.z)?;
        Ok(())
    }

    fn m2_size(&self) -> usize {
        self.x.m2_size() + self.y.m2_size() + self.z.m2_size()
    }
}

/// A vector in 2D space
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct C2Vector {
    pub x: f32,
    pub y: f32,
}

impl C2Vector {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

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

impl M2DataR for C2Vector {
    fn m2_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let x = reader.m2_read()?;
        let y = reader.m2_read()?;

        Ok(Self { x, y })
    }
}
impl M2DataW for C2Vector {
    fn m2_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.m2_write(&self.x)?;
        writer.m2_write(&self.y)?;
        Ok(())
    }

    fn m2_size(&self) -> usize {
        self.x.m2_size() + self.y.m2_size()
    }
}

/// Bounding box used in WoW files
#[derive(Debug, Clone, PartialEq)]
pub struct BoundingBox {
    /// Minimum corner of the bounding box
    pub min: C3Vector,
    /// Maximum corner of the bounding box
    pub max: C3Vector,
}

impl BoundingBox {
    /// Creates a new bounding box
    pub fn new(min: C3Vector, max: C3Vector) -> Self {
        Self { min, max }
    }

    /// Creates a default bounding box at origin with no size
    pub fn zero() -> Self {
        Self::new(C3Vector::origin(), C3Vector::origin())
    }

    /// Reads a BoundingBox from a reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let min = C3Vector::parse(reader)?;
        let max = C3Vector::parse(reader)?;
        Ok(Self::new(min, max))
    }

    /// Writes a BoundingBox to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.min.write(writer)?;
        self.max.write(writer)?;
        Ok(())
    }
}

impl M2DataR for BoundingBox {
    fn m2_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok(Self {
            min: reader.m2_read()?,
            max: reader.m2_read()?,
        })
    }
}
impl M2DataW for BoundingBox {
    fn m2_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.m2_write(&self.min)?;
        writer.m2_write(&self.max)?;
        Ok(())
    }

    fn m2_size(&self) -> usize {
        self.min.m2_size() + self.max.m2_size()
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
    fn test_u32_read_write() {
        let data = [0x05, 0x20, 0x00, 0x00];
        let mut cursor = Cursor::new(data);

        let u32_val: u32 = cursor.m2_read().unwrap();
        assert_eq!(u32_val, 0x00002005);

        let mut data = Vec::new();
        let mut writer = Cursor::new(&mut data);
        writer.m2_write(&0x30002100_u32).unwrap();
        assert_eq!(data, [0x00, 0x21, 0x00, 0x30]);
    }

    #[test]
    fn test_u16_read_write() {
        let data = [0x13, 0x43, 0x00, 0x00];
        let mut cursor = Cursor::new(data);

        let u16val: u16 = cursor.m2_read().unwrap();
        assert_eq!(u16val, 0x4313);

        let u16val: u16 = cursor.m2_read().unwrap();
        assert_eq!(u16val, 0x0000);

        let mut data = Vec::new();
        let mut writer = Cursor::new(&mut data);
        writer.m2_write(&0x0330_u16).unwrap();
        assert_eq!(data, [0x30, 0x03]);
    }

    #[test]
    fn test_c3vector_parse() {
        let data = [
            0x00, 0x00, 0x80, 0x3F, // x = 1.0
            0x00, 0x00, 0x00, 0x40, // y = 2.0
            0x00, 0x00, 0x40, 0x40, // z = 3.0
        ];

        let mut cursor = Cursor::new(data);
        let vector: C3Vector = cursor.m2_read().unwrap();

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
        let vector: C2Vector = cursor.m2_read().unwrap();

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

    #[test]
    fn test_m2array_parse() {
        let data = [
            0x05, 0x00, 0x00, 0x00, // count = 5
            0x20, 0x30, 0x00, 0x00, // offset = 0x3020
        ];

        let mut cursor = Cursor::new(data);
        let array: M2Array<u32> = cursor.m2_read().unwrap();

        assert_eq!(array.count, 5);
        assert_eq!(array.offset, 0x3020);
    }

    #[test]
    fn test_m2array_write() {
        let array = M2Array::<u32>::new(5, 0x3020);
        let mut cursor = Cursor::new(Vec::new());

        cursor.m2_write(&array).unwrap();

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
    fn test_m2array_to_vec() {
        let data = [
            0x00, 0x00, 0x80, 0x3F, // x = 1.0
            0x00, 0x00, 0x00, 0x40, // y = 2.0
            0x00, 0x00, 0x00, 0x40, // x = 2.0
            0x00, 0x00, 0x80, 0x3F, // y = 1.0
            0x00, 0x00, 0x80, 0x3F, // x = 1.0
            0x00, 0x00, 0x40, 0x40, // y = 3.0
        ];

        let array = M2Array::<C2Vector>::new(3, 0);

        let mut cursor = Cursor::new(data);
        let vec = array.m2_read_to_vec(&mut cursor).unwrap();

        assert_eq!(
            vec,
            vec![
                C2Vector::new(1., 2.),
                C2Vector::new(2., 1.),
                C2Vector::new(1., 3.)
            ]
        );
    }

    #[test]
    fn test_vec_to_m2array() {
        let mut cursor = Cursor::new(Vec::new());

        cursor.m2_write(&0xB00B5_u32).unwrap();
        cursor.m2_write(&0xDEAD_u16).unwrap();

        let vec = vec![
            C2Vector::new(1., 2.),
            C2Vector::new(2., 1.),
            C2Vector::new(1., 3.),
        ];

        let mut m2arr = vec.m2_write(&mut cursor).unwrap();
        m2arr.add_offset(3);

        assert_eq!(m2arr, M2Array::<C2Vector>::new(3, 9));
    }

    #[derive(super::Debug, Clone)]
    enum ExampleVersioned {
        UpToTBC(i16, f32),
        Others(u16),
    }

    impl M2DataRV for ExampleVersioned {
        fn m2_read<R: Read + Seek>(reader: &mut R, version: M2Version) -> Result<Self> {
            Ok(if version <= M2Version::TBC {
                Self::UpToTBC(reader.m2_read()?, reader.m2_read()?)
            } else {
                Self::Others(reader.m2_read()?)
            })
        }
    }

    impl M2DataW for ExampleVersioned {
        fn m2_write<W: Write>(&self, writer: &mut W) -> Result<()> {
            match self {
                Self::UpToTBC(a, b) => {
                    writer.m2_write(a)?;
                    writer.m2_write(b)?;
                }
                Self::Others(a) => {
                    writer.m2_write(a)?;
                }
            }
            Ok(())
        }

        fn m2_size(&self) -> usize {
            match self {
                Self::UpToTBC(a, b) => a.m2_size() + b.m2_size(),
                Self::Others(a) => a.m2_size(),
            }
        }
    }

    #[derive(super::Debug, Clone)]
    struct ExampleHeader {
        _version: M2Version,
        crc: u32,
        vectors: M2Array<C2Vector>,
        versioned_data: ExampleVersioned,
        bounding_box: BoundingBox,
        up_to_mop: Option<i16>,
        after_mop: Option<f32>,
    }

    impl M2DataRV for ExampleHeader {
        fn m2_read<R: Read + Seek>(reader: &mut R, version: M2Version) -> Result<Self> {
            Ok(Self {
                _version: version,
                crc: reader.m2_read()?,
                vectors: reader.m2_read()?,
                versioned_data: reader.m2_read_versioned(version)?,
                bounding_box: reader.m2_read()?,
                up_to_mop: if version <= M2Version::MoP {
                    Some(reader.m2_read()?)
                } else {
                    None
                },
                after_mop: if version > M2Version::MoP {
                    Some(reader.m2_read()?)
                } else {
                    None
                },
            })
        }
    }

    impl M2DataW for ExampleHeader {
        fn m2_write<W: Write>(&self, writer: &mut W) -> Result<()> {
            writer.m2_write(&self.crc)?;
            writer.m2_write(&self.vectors)?;
            writer.m2_write(&self.versioned_data)?;
            writer.m2_write(&self.bounding_box)?;
            if self._version <= M2Version::MoP {
                writer.m2_write(self.up_to_mop.as_ref().unwrap())?
            } else {
                writer.m2_write(self.after_mop.as_ref().unwrap())?
            }
            Ok(())
        }

        fn m2_size(&self) -> usize {
            self.crc.m2_size()
                + self.versioned_data.m2_size()
                + self.vectors.m2_size()
                + self.bounding_box.m2_size()
                + if self._version <= M2Version::MoP {
                    self.up_to_mop.as_ref().unwrap().m2_size()
                } else {
                    self.after_mop.as_ref().unwrap().m2_size()
                }
        }
    }

    struct ExampleData {
        header: ExampleHeader,
        vectors: Vec<C2Vector>,
    }

    impl ExampleData {
        fn read<R: Read + Seek>(reader: &mut R, version: u32) -> Result<Self> {
            let version = M2Version::from_header_version(version)?;
            let header: ExampleHeader = reader.m2_read_versioned(version)?;
            let vectors = header.vectors.m2_read_to_vec(reader)?;

            Ok(Self { header, vectors })
        }

        fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
            let header_size = self.header.m2_size();

            let mut data_section = Vec::with_capacity(self.vectors.m2_size());
            let mut data_section_writer = Cursor::new(&mut data_section);

            let mut vectors = self.vectors.m2_write(&mut data_section_writer)?;
            vectors.add_offset(header_size);

            let mut new_header = self.header.clone();
            new_header.vectors = vectors;

            writer.m2_write(&new_header)?;
            writer.write_all(&data_section)?;

            Ok(())
        }
    }

    #[test]
    fn test_m2_eample_struct_read_write() {
        let example_data_bin = [
            // header
            0x16, 0x00, 0x00, 0x00, // crc
            0x02, 0x00, 0x00, 0x00, // vectors count
            0x28, 0x00, 0x00, 0x00, // vectors offset
            0x42, 0x00, // ExampleVersioned::Others(66)
            0x00, 0x00, 0x80, 0x3F, // bounding_box.min.x
            0x00, 0x00, 0x00, 0x40, // bounding_box.min.y
            0x00, 0x00, 0x40, 0x40, // bounding_box.min.z
            0x00, 0x00, 0x40, 0x40, // bounding_box.max.x
            0x00, 0x00, 0x00, 0x40, // bounding_box.max.y
            0x00, 0x00, 0x80, 0x3F, // bounding_box.max.z
            0x23, 0x01, // up_to_mop
            // data section
            0x00, 0x00, 0x80, 0x3F, // vectors.0.x
            0x00, 0x00, 0x00, 0x40, // vectors.0.y
            0x00, 0x00, 0x00, 0x40, // vectors.1.x
            0x00, 0x00, 0x80, 0x3F, // vectors.1.y
        ];

        let example_data = ExampleData {
            header: ExampleHeader {
                _version: M2Version::WotLK,
                crc: 22,
                vectors: M2Array::default(),
                versioned_data: ExampleVersioned::Others(66),
                bounding_box: BoundingBox::new(
                    C3Vector::new(1., 2., 3.),
                    C3Vector::new(3., 2., 1.),
                ),
                up_to_mop: Some(0x123),
                after_mop: None,
            },
            vectors: vec![C2Vector::new(1., 2.), C2Vector::new(2., 1.)],
        };

        let mut writer = Cursor::new(Vec::new());
        example_data.write(&mut writer).unwrap();

        assert_eq!(*writer.get_ref(), example_data_bin);

        let mut cursor = Cursor::new(&example_data_bin);
        let decoded = ExampleData::read(&mut cursor, example_data.header._version.into()).unwrap();
        let mut dec_writer = Cursor::new(Vec::new());
        decoded.write(&mut dec_writer).unwrap();

        assert_eq!(*dec_writer.get_ref(), example_data_bin);
    }

    struct ExampleDataNoHeader {
        _version: M2Version,
        crc: u32,
        vectors: Vec<C2Vector>,
        versioned_data: ExampleVersioned,
        bounding_box: BoundingBox,
        up_to_mop: Option<i16>,
        after_mop: Option<f32>,
    }

    impl ExampleDataNoHeader {
        fn read<R: Read + Seek>(reader: &mut R, version: u32) -> Result<Self> {
            let version = M2Version::from_header_version(version)?;
            let header: ExampleHeader = reader.m2_read_versioned(version)?;
            let vectors = header.vectors.m2_read_to_vec(reader)?;

            Ok(Self {
                _version: version,
                crc: header.crc,
                vectors,
                versioned_data: header.versioned_data,
                bounding_box: header.bounding_box,
                up_to_mop: header.up_to_mop,
                after_mop: header.after_mop,
            })
        }
    }

    impl M2DataW for ExampleDataNoHeader {
        fn m2_write<W: Write>(&self, writer: &mut W) -> Result<()> {
            let mut new_header = ExampleHeader {
                _version: self._version,
                crc: self.crc,
                vectors: M2Array::default(),
                versioned_data: self.versioned_data.clone(),
                bounding_box: self.bounding_box.clone(),
                up_to_mop: self.up_to_mop,
                after_mop: self.after_mop,
            };

            let header_size = new_header.m2_size();

            let mut data_section = Vec::with_capacity(self.vectors.m2_size());
            let mut data_section_writer = Cursor::new(&mut data_section);

            new_header.vectors = self.vectors.m2_write(&mut data_section_writer)?;
            new_header.vectors.add_offset(header_size);

            writer.m2_write(&new_header)?;
            writer.write_all(&data_section)?;

            Ok(())
        }

        fn m2_size(&self) -> usize {
            todo!()
        }
    }

    #[test]
    fn test_m2_eample_struct_no_header_read_write() {
        let example_data_bin = [
            // header
            0x16, 0x00, 0x00, 0x00, // crc
            0x02, 0x00, 0x00, 0x00, // vectors count
            0x2A, 0x00, 0x00, 0x00, // vectors offset
            0x42, 0x00, // ExampleVersioned::Others(66)
            0x00, 0x00, 0x80, 0x3F, // bounding_box.min.x
            0x00, 0x00, 0x00, 0x40, // bounding_box.min.y
            0x00, 0x00, 0x40, 0x40, // bounding_box.min.z
            0x00, 0x00, 0x40, 0x40, // bounding_box.max.x
            0x00, 0x00, 0x00, 0x40, // bounding_box.max.y
            0x00, 0x00, 0x80, 0x3F, // bounding_box.max.z
            0x00, 0x00, 0x80, 0x3F, // after_mop
            // data section
            0x00, 0x00, 0x80, 0x3F, // vectors.0.x
            0x00, 0x00, 0x00, 0x40, // vectors.0.y
            0x00, 0x00, 0x00, 0x40, // vectors.1.x
            0x00, 0x00, 0x80, 0x3F, // vectors.1.y
        ];

        let example_data = ExampleDataNoHeader {
            _version: M2Version::WoD,
            crc: 22,
            versioned_data: ExampleVersioned::Others(66),
            bounding_box: BoundingBox::new(C3Vector::new(1., 2., 3.), C3Vector::new(3., 2., 1.)),
            up_to_mop: None,
            after_mop: Some(1.0),
            vectors: vec![C2Vector::new(1., 2.), C2Vector::new(2., 1.)],
        };

        let mut writer = Cursor::new(Vec::new());
        writer.m2_write(&example_data).unwrap();

        assert_eq!(*writer.get_ref(), example_data_bin);

        let mut cursor = Cursor::new(&example_data_bin);
        let decoded = ExampleDataNoHeader::read(&mut cursor, example_data._version.into()).unwrap();
        let mut dec_writer = Cursor::new(Vec::new());
        dec_writer.m2_write(&decoded).unwrap();

        assert_eq!(*dec_writer.get_ref(), example_data_bin);
    }

    impl M2DataConversible for ExampleDataNoHeader {
        fn m2_convert(&self, to_version: M2Version) -> Result<Self> {
            match to_version {
                M2Version::Classic | M2Version::TBC => Ok(Self {
                    _version: to_version,
                    crc: self.crc,
                    versioned_data: if self._version <= M2Version::TBC {
                        self.versioned_data.clone()
                    } else {
                        ExampleVersioned::UpToTBC(0, 2.0)
                    },
                    bounding_box: self.bounding_box.clone(),
                    up_to_mop: if self._version <= M2Version::MoP {
                        self.up_to_mop
                    } else {
                        Some(0)
                    },
                    after_mop: None,
                    vectors: self.vectors.clone(),
                }),
                _ => {
                    if to_version <= M2Version::WoD {
                        Ok(Self {
                            _version: to_version,
                            crc: self.crc,
                            versioned_data: if self._version <= M2Version::TBC {
                                ExampleVersioned::Others(0x49)
                            } else {
                                self.versioned_data.clone()
                            },
                            bounding_box: self.bounding_box.clone(),
                            up_to_mop: if to_version > M2Version::MoP {
                                None
                            } else {
                                if self._version <= M2Version::MoP {
                                    self.up_to_mop
                                } else {
                                    Some(0)
                                }
                            },
                            after_mop: if to_version > M2Version::MoP {
                                if self._version > M2Version::MoP {
                                    self.after_mop
                                } else {
                                    Some(0.0)
                                }
                            } else {
                                None
                            },
                            vectors: self.vectors.clone(),
                        })
                    } else {
                        Err(M2Error::ConversionError {
                            from: self._version.into(),
                            to: to_version.into(),
                            reason: "not supported".into(),
                        })
                    }
                }
            }
        }
    }

    #[test]
    fn test_m2_eample_conversion() {
        let example_data_bin = [
            // header
            0x16, 0x00, 0x00, 0x00, // crc
            0x02, 0x00, 0x00, 0x00, // vectors count
            0x2A, 0x00, 0x00, 0x00, // vectors offset
            0x42, 0x00, // ExampleVersioned::Others(66)
            0x00, 0x00, 0x80, 0x3F, // bounding_box.min.x
            0x00, 0x00, 0x00, 0x40, // bounding_box.min.y
            0x00, 0x00, 0x40, 0x40, // bounding_box.min.z
            0x00, 0x00, 0x40, 0x40, // bounding_box.max.x
            0x00, 0x00, 0x00, 0x40, // bounding_box.max.y
            0x00, 0x00, 0x80, 0x3F, // bounding_box.max.z
            0x00, 0x00, 0x80, 0x3F, // after_mop
            // data section
            0x00, 0x00, 0x80, 0x3F, // vectors.0.x
            0x00, 0x00, 0x00, 0x40, // vectors.0.y
            0x00, 0x00, 0x00, 0x40, // vectors.1.x
            0x00, 0x00, 0x80, 0x3F, // vectors.1.y
        ];

        let mut cursor = Cursor::new(&example_data_bin);
        let decoded = ExampleDataNoHeader::read(&mut cursor, M2Version::WoD.into()).unwrap();
        let mut dec_writer = Cursor::new(Vec::new());
        dec_writer.m2_write(&decoded).unwrap();

        assert_eq!(*dec_writer.get_ref(), example_data_bin);

        let mut converted_writer = Cursor::new(Vec::new());
        converted_writer
            .m2_write_versioned(&decoded, M2Version::Classic)
            .unwrap();

        let example_converted_data_bin = [
            // header
            0x16, 0x00, 0x00, 0x00, // crc
            0x02, 0x00, 0x00, 0x00, // vectors count
            0x2C, 0x00, 0x00, 0x00, // vectors offset
            0x00, 0x00, // ExampleVersioned::UpToTBC.0 = 0
            0x00, 0x00, 0x00, 0x40, // ExampleVersioned::UpToTBC.1 = 2.0
            0x00, 0x00, 0x80, 0x3F, // bounding_box.min.x
            0x00, 0x00, 0x00, 0x40, // bounding_box.min.y
            0x00, 0x00, 0x40, 0x40, // bounding_box.min.z
            0x00, 0x00, 0x40, 0x40, // bounding_box.max.x
            0x00, 0x00, 0x00, 0x40, // bounding_box.max.y
            0x00, 0x00, 0x80, 0x3F, // bounding_box.max.z
            0x00, 0x00, // up_to_mop
            // data section
            0x00, 0x00, 0x80, 0x3F, // vectors.0.x
            0x00, 0x00, 0x00, 0x40, // vectors.0.y
            0x00, 0x00, 0x00, 0x40, // vectors.1.x
            0x00, 0x00, 0x80, 0x3F, // vectors.1.y
        ];

        assert_eq!(*converted_writer.get_ref(), example_converted_data_bin);
    }
}
