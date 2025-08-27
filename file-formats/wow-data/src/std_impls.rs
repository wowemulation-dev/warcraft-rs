use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::error::Result;

use super::types::*;

impl WowHeaderR for u32 {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_u32::<LittleEndian>()?)
    }
}
impl WowHeaderW for u32 {
    fn wow_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32::<LittleEndian>(*self)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        4
    }
}

impl WowHeaderR for (u32, u32) {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok((reader.wow_read()?, reader.wow_read()?))
    }
}
impl WowHeaderW for (u32, u32) {
    fn wow_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.wow_write(&self.0)?;
        writer.wow_write(&self.1)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        8
    }
}

impl WowHeaderR for [u32; 3] {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok([reader.wow_read()?, reader.wow_read()?, reader.wow_read()?])
    }
}
impl WowHeaderW for [u32; 3] {
    fn wow_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.wow_write(&self[0])?;
        writer.wow_write(&self[1])?;
        writer.wow_write(&self[2])?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        0_u32.wow_size() * 3
    }
}

impl WowHeaderR for i32 {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_i32::<LittleEndian>()?)
    }
}
impl WowHeaderW for i32 {
    fn wow_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_i32::<LittleEndian>(*self)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        4
    }
}

impl WowHeaderR for i16 {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_i16::<LittleEndian>()?)
    }
}
impl WowHeaderW for i16 {
    fn wow_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_i16::<LittleEndian>(*self)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        2
    }
}

impl WowHeaderR for [i16; 2] {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok([reader.wow_read()?, reader.wow_read()?])
    }
}
impl WowHeaderW for [i16; 2] {
    fn wow_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.wow_write(&self[0])?;
        writer.wow_write(&self[1])?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        0_i16.wow_size() * 2
    }
}

impl WowHeaderR for u16 {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_u16::<LittleEndian>()?)
    }
}
impl WowHeaderW for u16 {
    fn wow_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u16::<LittleEndian>(*self)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        2
    }
}

impl WowHeaderR for [u16; 3] {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok([reader.wow_read()?, reader.wow_read()?, reader.wow_read()?])
    }
}
impl WowHeaderW for [u16; 3] {
    fn wow_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.wow_write(&self[0])?;
        writer.wow_write(&self[1])?;
        writer.wow_write(&self[2])?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        0_u16.wow_size() * 3
    }
}

impl WowHeaderR for u8 {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_u8()?)
    }
}
impl WowHeaderW for u8 {
    fn wow_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u8(*self)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        1
    }
}

impl WowHeaderR for [u8; 2] {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok([reader.read_u8()?, reader.read_u8()?])
    }
}
impl WowHeaderW for [u8; 2] {
    fn wow_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        for i in 0..2 {
            writer.write_u8((*self)[i])?;
        }
        Ok(())
    }

    fn wow_size(&self) -> usize {
        2
    }
}

impl WowHeaderR for [u8; 4] {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok([
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
            reader.read_u8()?,
        ])
    }
}
impl WowHeaderW for [u8; 4] {
    fn wow_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        for i in 0..4 {
            writer.write_u8((*self)[i])?;
        }
        Ok(())
    }

    fn wow_size(&self) -> usize {
        4
    }
}

impl WowHeaderR for i8 {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_i8()?)
    }
}
impl WowHeaderW for i8 {
    fn wow_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_i8(*self)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        1
    }
}

impl WowHeaderR for f32 {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok(reader.read_f32::<LittleEndian>()?)
    }
}
impl WowHeaderW for f32 {
    fn wow_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_f32::<LittleEndian>(*self)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        4
    }
}

impl WowHeaderR for [f32; 3] {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Ok([reader.wow_read()?, reader.wow_read()?, reader.wow_read()?])
    }
}
impl WowHeaderW for [f32; 3] {
    fn wow_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.wow_write(&self[0])?;
        writer.wow_write(&self[1])?;
        writer.wow_write(&self[2])?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        0_f32.wow_size() * 3
    }
}

impl WowHeaderW for String {
    fn wow_write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.wow_write(self.into())?;
        // write null terminator
        writer.wow_write(&0_u8)?;
        Ok(())
    }

    fn wow_size(&self) -> usize {
        self.len() + 1
    }
}

impl WowString for String {
    fn from_wow_char_array<R: Read + Seek>(
        reader: &mut R,
        wow_string: WowCharArray,
    ) -> Result<String> {
        if wow_string.count == 0 {
            return Ok("".into());
        }

        let bytes = wow_string.wow_read_to_vec(reader)?;
        let str_end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        Ok(String::from_utf8_lossy(&bytes[..str_end]).to_string())
    }

    fn write_wow_char_array<W: Write + Seek>(&self, writer: &mut W) -> Result<WowCharArray> {
        let offset = writer.stream_position()?;
        writer.wow_write(self)?;
        Ok(WowCharArray::new(self.wow_size() as u32, offset as u32))
    }
}

impl WowDataR<WowCharArray> for String {
    fn new_from_header<R: Read + Seek>(reader: &mut R, header: &WowCharArray) -> Result<Self> {
        String::from_wow_char_array(reader, header.clone())
    }
}

impl<T: WowHeaderR + WowHeaderW> WowVec<T> for Vec<T> {
    /// Write vector data to `writer` and return a `WowArray<T>`. The offset property
    /// is set from the current writer position.
    fn wow_write<W: Write + Seek>(&self, writer: &mut W) -> Result<WowArray<T>> {
        let offset = writer.stream_position()?;
        for item in self {
            writer.wow_write(item)?
        }
        Ok(WowArray::<T>::new(self.len() as u32, offset as u32))
    }
}

impl<T> WowDataR<WowArray<T>> for Vec<T>
where
    T: WowHeaderR + WowHeaderW,
{
    fn new_from_header<R: Read + Seek>(reader: &mut R, header: &WowArray<T>) -> Result<Self> {
        Ok(header.wow_read_to_vec(reader)?)
    }
}
