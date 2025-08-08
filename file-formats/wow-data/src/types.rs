use custom_debug::Debug;
use wow_data_derive::{WowDataR, WowDataW};

use crate::error::{Result, WowDataError};
use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Seek, SeekFrom, Write};

pub trait WowDataR: Sized {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> Result<Self>;
}

pub trait DataVersion: Copy {}

pub trait WowDataRV<V: DataVersion>: Sized {
    fn wow_read<R: Read + Seek>(reader: &mut R, version: V) -> Result<Self>;
}

pub trait WowDataW {
    fn wow_write<W: Write>(&self, writer: &mut W) -> Result<()>;
    fn wow_size(&self) -> usize;
}

pub trait WowDataConversible<V: DataVersion>: WowDataW + Sized {
    fn wow_write_version<W: Write>(&self, writer: &mut W, version: V) -> Result<()> {
        let converted = self.wow_convert(version)?;
        converted.wow_write(writer)
    }
    fn wow_convert(&self, to_version: V) -> Result<Self>;
}

impl WowDataR for u32 {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_u32_le()?)
    }
}
impl WowDataW for u32 {
    fn wow_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32_le(*self)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        4
    }
}

impl WowDataR for f32 {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_f32_le()?)
    }
}
impl WowDataW for f32 {
    fn wow_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_f32_le(*self)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        4
    }
}

impl WowDataR for i16 {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_i16_le()?)
    }
}
impl WowDataW for i16 {
    fn wow_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_i16_le(*self)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        2
    }
}

impl WowDataR for u16 {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_u16_le()?)
    }
}
impl WowDataW for u16 {
    fn wow_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u16_le(*self)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        2
    }
}

pub trait WowReader<T: WowDataR>: Read + Seek + Sized {
    fn wow_read(&mut self) -> Result<T> {
        Ok(T::wow_read(self)?)
    }
}
impl<T: WowDataR, R: Read + Seek> WowReader<T> for R {}

pub trait WowReaderV<V: DataVersion, T: WowDataRV<V>>: Read + Seek + Sized {
    fn wow_read_versioned(&mut self, version: V) -> Result<T> {
        Ok(T::wow_read(self, version)?)
    }
}
impl<V: DataVersion, T: WowDataRV<V>, R: Read + Seek> WowReaderV<V, T> for R {}

pub trait WowWriter<T: WowDataW>: Write + Sized {
    fn wow_write(&mut self, value: &T) -> Result<()> {
        value.wow_write(self)?;
        Ok(())
    }
}
impl<T: WowDataW, W: Write> WowWriter<T> for W {}

pub trait WowWriterV<V: DataVersion, T: WowDataConversible<V>>: Write + Sized {
    fn wow_write_versioned(&mut self, value: &T, version: V) -> Result<()> {
        value.wow_write_version(self, version)?;
        Ok(())
    }
}
impl<V: DataVersion, T: WowDataConversible<V>, W: Write> WowWriterV<V, T> for W {}

#[derive(Debug, Clone, Copy, Default, PartialEq, WowDataR, WowDataW)]
pub struct WowArray<T> {
    pub count: u32,
    pub offset: u32,
    #[wow_data(skip = std::marker::PhantomData)]
    _phantom: std::marker::PhantomData<T>,
}

impl<T> WowArray<T> {
    pub fn new(count: u32, offset: u32) -> Self {
        Self {
            count,
            offset,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn add_offset(&mut self, offset: usize) {
        self.offset += offset as u32;
    }
}

impl<T: WowDataR> WowArray<T> {
    pub fn wow_read_to_vec<R: Read + Seek>(&self, reader: &mut R) -> Result<Vec<T>> {
        if self.is_empty() {
            return Ok(Vec::new());
        }

        reader
            .seek(SeekFrom::Start(self.offset as u64))
            .map_err(WowDataError::Io)?;

        let mut result = Vec::with_capacity(self.count as usize);
        for _ in 0..self.count {
            result.push(T::wow_read(reader)?);
        }

        Ok(result)
    }
}

pub trait WowVec<T: WowDataW> {
    fn wow_write<W: Write + Seek>(&self, writer: &mut W) -> Result<WowArray<T>>;
    fn wow_size(&self) -> usize;
}

impl<T: WowDataW> WowVec<T> for Vec<T> {
    fn wow_write<W: Write + Seek>(&self, writer: &mut W) -> Result<WowArray<T>> {
        let offset = writer.stream_position()?;
        for item in self {
            writer.wow_write(item)?
        }
        Ok(WowArray::<T>::new(self.len() as u32, offset as u32))
    }

    fn wow_size(&self) -> usize {
        if self.is_empty() {
            0
        } else {
            self.len() * self[0].wow_size()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, WowDataR, WowDataW)]
pub struct C3Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl C3Vector {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn to_glam(&self) -> glam::Vec3 {
        glam::Vec3::new(self.x, self.y, self.z)
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Default, WowDataR, WowDataW)]
pub struct C2Vector {
    pub x: f32,
    pub y: f32,
}

impl C2Vector {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn to_glam(&self) -> glam::Vec2 {
        glam::Vec2::new(self.x, self.y)
    }

    pub fn from_glam(v: glam::Vec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

#[derive(Debug, Clone, PartialEq, WowDataR, WowDataW)]
pub struct BoundingBox {
    pub min: C3Vector,
    pub max: C3Vector,
}

impl BoundingBox {
    pub fn new(min: C3Vector, max: C3Vector) -> Self {
        Self { min, max }
    }

    pub fn zero() -> Self {
        Self::new(C3Vector::origin(), C3Vector::origin())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, WowDataR, WowDataW)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    pub fn to_glam(&self) -> glam::Quat {
        glam::Quat::from_xyzw(self.x, self.y, self.z, self.w)
    }

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

#[derive(Debug, Clone, Copy, PartialEq, WowDataR, WowDataW)]
pub struct Quaternion16 {
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub w: i16,
}

#[cfg(test)]
mod tests {
    use wow_data_derive::{WowDataRV, WowDataW};

    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_u32_read_write() {
        let data = [0x05, 0x20, 0x00, 0x00];
        let mut cursor = Cursor::new(data);

        let u32_val: u32 = cursor.wow_read().unwrap();
        assert_eq!(u32_val, 0x00002005);

        let mut data = Vec::new();
        let mut writer = Cursor::new(&mut data);
        writer.wow_write(&0x30002100_u32).unwrap();
        assert_eq!(data, [0x00, 0x21, 0x00, 0x30]);
    }

    #[test]
    fn test_u16_read_write() {
        let data = [0x13, 0x43, 0x00, 0x00];
        let mut cursor = Cursor::new(data);

        let u16val: u16 = cursor.wow_read().unwrap();
        assert_eq!(u16val, 0x4313);

        let u16val: u16 = cursor.wow_read().unwrap();
        assert_eq!(u16val, 0x0000);

        let mut data = Vec::new();
        let mut writer = Cursor::new(&mut data);
        writer.wow_write(&0x0330_u16).unwrap();
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
        let vector: C3Vector = cursor.wow_read().unwrap();

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
        let vector: C2Vector = cursor.wow_read().unwrap();

        assert_eq!(vector.x, 1.0);
        assert_eq!(vector.y, 2.0);
    }

    #[test]
    fn test_wowarray_parse() {
        let data = [
            0x05, 0x00, 0x00, 0x00, // count = 5
            0x20, 0x30, 0x00, 0x00, // offset = 0x3020
        ];

        let mut cursor = Cursor::new(data);
        let array: WowArray<u32> = cursor.wow_read().unwrap();

        assert_eq!(array.count, 5);
        assert_eq!(array.offset, 0x3020);
    }

    #[test]
    fn test_wowarray_write() {
        let array = WowArray::<u32>::new(5, 0x3020);
        let mut cursor = Cursor::new(Vec::new());

        cursor.wow_write(&array).unwrap();

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
    fn test_wowarray_to_vec() {
        let data = [
            0x00, 0x00, 0x80, 0x3F, // x = 1.0
            0x00, 0x00, 0x00, 0x40, // y = 2.0
            0x00, 0x00, 0x00, 0x40, // x = 2.0
            0x00, 0x00, 0x80, 0x3F, // y = 1.0
            0x00, 0x00, 0x80, 0x3F, // x = 1.0
            0x00, 0x00, 0x40, 0x40, // y = 3.0
        ];

        let array = WowArray::<C2Vector>::new(3, 0);

        let mut cursor = Cursor::new(data);
        let vec = array.wow_read_to_vec(&mut cursor).unwrap();

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
    fn test_vec_to_wowarray() {
        let mut cursor = Cursor::new(Vec::new());

        cursor.wow_write(&0xB00B5_u32).unwrap();
        cursor.wow_write(&0xDEAD_u16).unwrap();

        let vec = vec![
            C2Vector::new(1., 2.),
            C2Vector::new(2., 1.),
            C2Vector::new(1., 3.),
        ];

        let mut wowarr = vec.wow_write(&mut cursor).unwrap();
        wowarr.add_offset(3);

        assert_eq!(wowarr, WowArray::<C2Vector>::new(3, 9));
    }

    #[derive(super::Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum M2Version {
        Classic,
        TBC,
        WotLK,
        Cataclysm,
        MoP,
        WoD,
        Legion,
        BfA,
    }

    impl DataVersion for M2Version {}

    impl From<M2Version> for u32 {
        fn from(value: M2Version) -> Self {
            match value {
                M2Version::Classic => 1,
                M2Version::TBC => 2,
                M2Version::WotLK => 3,
                M2Version::Cataclysm => 4,
                M2Version::MoP => 5,
                M2Version::WoD => 6,
                M2Version::Legion => 7,
                M2Version::BfA => 8,
            }
        }
    }

    impl TryFrom<u32> for M2Version {
        type Error = WowDataError;

        fn try_from(value: u32) -> Result<Self> {
            match value {
                1 => Ok(Self::Classic),
                2 => Ok(Self::TBC),
                3 => Ok(Self::WotLK),
                4 => Ok(Self::Cataclysm),
                5 => Ok(Self::MoP),
                6 => Ok(Self::WoD),
                7 => Ok(Self::Legion),
                8 => Ok(Self::BfA),
                _ => Err(WowDataError::UnsupportedNumericVersion(value)),
            }
        }
    }

    impl WowDataR for M2Version {
        fn wow_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
            let version: u32 = reader.wow_read()?;
            version.try_into()
        }
    }

    impl WowDataW for M2Version {
        fn wow_write<W: Write>(&self, writer: &mut W) -> Result<()> {
            let version: u32 = (*self).into();
            writer.wow_write(&version)?;
            Ok(())
        }

        fn wow_size(&self) -> usize {
            4
        }
    }

    #[derive(super::Debug, Clone, Copy, WowDataRV, WowDataW)]
    #[wow_data(version = M2Version)]
    enum ExampleVersioned {
        #[wow_data(read_if = version <= M2Version::TBC)]
        UpToTBC(i16, f32),
        Others(u16),
    }

    #[derive(super::Debug, Clone, Copy, WowDataRV, WowDataW)]
    #[wow_data(version = M2Version)]
    enum OptionUpToMoP<T: WowDataR + WowDataW> {
        #[wow_data(read_if = version <= M2Version::MoP)]
        Some(T),
        None,
    }

    #[derive(super::Debug, Clone, Copy, WowDataRV, WowDataW)]
    #[wow_data(version = M2Version)]
    enum OptionAfterMoP<T: WowDataR + WowDataW> {
        #[wow_data(read_if = version > M2Version::MoP)]
        Some(T),
        None,
    }

    #[derive(super::Debug, Clone, WowDataRV, WowDataW)]
    #[wow_data(version = M2Version)]
    struct ExampleHeader {
        #[wow_data(skip = M2Version::Classic)]
        _version: M2Version,
        crc: u32,
        vectors: WowArray<C2Vector>,
        #[wow_data(versioned)]
        versioned_data: ExampleVersioned,
        bounding_box: BoundingBox,
        #[wow_data(versioned)]
        up_to_mop: OptionUpToMoP<i16>,
        #[wow_data(versioned)]
        after_mop: OptionAfterMoP<f32>,
    }

    struct ExampleData {
        header: ExampleHeader,
        vectors: Vec<C2Vector>,
    }

    impl ExampleData {
        fn read<R: Read + Seek>(reader: &mut R, version: u32) -> Result<Self> {
            let version = version.try_into()?;
            let header: ExampleHeader = reader.wow_read_versioned(version)?;
            let vectors = header.vectors.wow_read_to_vec(reader)?;

            Ok(Self { header, vectors })
        }

        fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
            let header_size = self.header.wow_size();

            let mut data_section = Vec::with_capacity(self.vectors.wow_size());
            let mut data_section_writer = Cursor::new(&mut data_section);

            let mut vectors = self.vectors.wow_write(&mut data_section_writer)?;
            vectors.add_offset(header_size);

            let mut new_header = self.header.clone();
            new_header.vectors = vectors;

            writer.wow_write(&new_header)?;
            writer.write_all(&data_section)?;

            Ok(())
        }
    }

    #[test]
    fn test_simple_struct_read_write() {
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
                vectors: WowArray::default(),
                versioned_data: ExampleVersioned::Others(66),
                bounding_box: BoundingBox::new(
                    C3Vector::new(1., 2., 3.),
                    C3Vector::new(3., 2., 1.),
                ),
                up_to_mop: OptionUpToMoP::Some(0x123),
                after_mop: OptionAfterMoP::None,
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
        up_to_mop: OptionUpToMoP<i16>,
        after_mop: OptionAfterMoP<f32>,
    }

    impl ExampleDataNoHeader {
        fn read<R: Read + Seek>(reader: &mut R, version: u32) -> Result<Self> {
            let version = version.try_into()?;
            let header: ExampleHeader = reader.wow_read_versioned(version)?;
            let vectors = header.vectors.wow_read_to_vec(reader)?;

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

    impl WowDataW for ExampleDataNoHeader {
        fn wow_write<W: Write>(&self, writer: &mut W) -> Result<()> {
            let mut new_header = ExampleHeader {
                _version: self._version,
                crc: self.crc,
                vectors: WowArray::default(),
                versioned_data: self.versioned_data.clone(),
                bounding_box: self.bounding_box.clone(),
                up_to_mop: self.up_to_mop,
                after_mop: self.after_mop,
            };

            let header_size = new_header.wow_size();

            let mut data_section = Vec::with_capacity(self.vectors.wow_size());
            let mut data_section_writer = Cursor::new(&mut data_section);

            new_header.vectors = self.vectors.wow_write(&mut data_section_writer)?;
            new_header.vectors.add_offset(header_size);

            writer.wow_write(&new_header)?;
            writer.write_all(&data_section)?;

            Ok(())
        }

        fn wow_size(&self) -> usize {
            todo!()
        }
    }

    #[test]
    fn test_simple_struct_no_header_read_write() {
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
            up_to_mop: OptionUpToMoP::None,
            after_mop: OptionAfterMoP::Some(1.0),
            vectors: vec![C2Vector::new(1., 2.), C2Vector::new(2., 1.)],
        };

        let mut writer = Cursor::new(Vec::new());
        writer.wow_write(&example_data).unwrap();

        assert_eq!(*writer.get_ref(), example_data_bin);

        let mut cursor = Cursor::new(&example_data_bin);
        let decoded = ExampleDataNoHeader::read(&mut cursor, example_data._version.into()).unwrap();
        let mut dec_writer = Cursor::new(Vec::new());
        dec_writer.wow_write(&decoded).unwrap();

        assert_eq!(*dec_writer.get_ref(), example_data_bin);
    }

    impl WowDataConversible<M2Version> for ExampleDataNoHeader {
        fn wow_convert(&self, to_version: M2Version) -> Result<Self> {
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
                        OptionUpToMoP::Some(0)
                    },
                    after_mop: OptionAfterMoP::None,
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
                                OptionUpToMoP::None
                            } else {
                                if self._version <= M2Version::MoP {
                                    self.up_to_mop
                                } else {
                                    OptionUpToMoP::Some(0)
                                }
                            },
                            after_mop: if to_version > M2Version::MoP {
                                if self._version > M2Version::MoP {
                                    self.after_mop
                                } else {
                                    OptionAfterMoP::Some(0.0)
                                }
                            } else {
                                OptionAfterMoP::None
                            },
                            vectors: self.vectors.clone(),
                        })
                    } else {
                        Err(WowDataError::ConversionError {
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
    fn test_simple_conversion() {
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
        dec_writer.wow_write(&decoded).unwrap();

        assert_eq!(*dec_writer.get_ref(), example_data_bin);

        let mut converted_writer = Cursor::new(Vec::new());
        converted_writer
            .wow_write_versioned(&decoded, M2Version::Classic)
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
