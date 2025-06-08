//! I/O abstractions for MPQ archives

use crate::Result;
use std::io::{Read, Seek, SeekFrom};

/// Trait for reading from MPQ archives
pub trait MpqRead: Read + Seek {
    /// Read exact number of bytes at the given offset
    fn read_at(&mut self, offset: u64, buf: &mut [u8]) -> Result<()>;
}

/// Buffered reader for MPQ archives
#[derive(Debug)]
pub struct BufferedMpqReader<R> {
    inner: R,
}

impl<R: Read + Seek> BufferedMpqReader<R> {
    /// Create a new buffered reader
    pub fn new(inner: R) -> Self {
        Self { inner }
    }
}

impl<R: Read + Seek> MpqRead for BufferedMpqReader<R> {
    fn read_at(&mut self, offset: u64, buf: &mut [u8]) -> Result<()> {
        self.inner.seek(SeekFrom::Start(offset))?;
        self.inner.read_exact(buf)?;
        Ok(())
    }
}

impl<R: Read> Read for BufferedMpqReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<R: Seek> Seek for BufferedMpqReader<R> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_buffered_mpq_reader_creation() {
        let data = vec![1, 2, 3, 4, 5];
        let cursor = Cursor::new(data.clone());
        let reader = BufferedMpqReader::new(cursor);

        // Verify we can access the inner reader
        assert_eq!(reader.inner.get_ref(), &data);
    }

    #[test]
    fn test_read_at() {
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let cursor = Cursor::new(data);
        let mut reader = BufferedMpqReader::new(cursor);

        let mut buf = [0u8; 3];
        reader.read_at(2, &mut buf).unwrap();
        assert_eq!(buf, [2, 3, 4]);

        let mut buf = [0u8; 2];
        reader.read_at(8, &mut buf).unwrap();
        assert_eq!(buf, [8, 9]);

        let mut buf = [0u8; 1];
        reader.read_at(0, &mut buf).unwrap();
        assert_eq!(buf, [0]);
    }

    #[test]
    fn test_read_at_multiple_calls() {
        let data = vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
        let cursor = Cursor::new(data);
        let mut reader = BufferedMpqReader::new(cursor);

        let mut buf1 = [0u8; 2];
        let mut buf2 = [0u8; 3];
        let mut buf3 = [0u8; 1];

        reader.read_at(1, &mut buf1).unwrap();
        assert_eq!(buf1, [20, 30]);

        reader.read_at(5, &mut buf2).unwrap();
        assert_eq!(buf2, [60, 70, 80]);

        reader.read_at(9, &mut buf3).unwrap();
        assert_eq!(buf3, [100]);
    }

    #[test]
    fn test_read_at_out_of_bounds() {
        let data = vec![1, 2, 3];
        let cursor = Cursor::new(data);
        let mut reader = BufferedMpqReader::new(cursor);

        // This should succeed - reading last byte and one past
        let mut buf = [0u8; 2];
        let result = reader.read_at(2, &mut buf);
        assert!(result.is_err()); // Can't read 2 bytes starting at position 2 from 3-byte data

        // This should definitely fail - starting beyond the data
        let mut buf = [0u8; 1];
        let result = reader.read_at(3, &mut buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_trait() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let cursor = Cursor::new(data);
        let mut reader = BufferedMpqReader::new(cursor);

        let mut buf = [0u8; 3];
        let bytes_read = reader.read(&mut buf).unwrap();
        assert_eq!(bytes_read, 3);
        assert_eq!(buf, [1, 2, 3]);

        let mut buf = [0u8; 4];
        let bytes_read = reader.read(&mut buf).unwrap();
        assert_eq!(bytes_read, 4);
        assert_eq!(buf, [4, 5, 6, 7]);
    }

    #[test]
    fn test_read_empty_buffer() {
        let data = vec![1, 2, 3];
        let cursor = Cursor::new(data);
        let mut reader = BufferedMpqReader::new(cursor);

        let mut buf = [0u8; 0];
        let bytes_read = reader.read(&mut buf).unwrap();
        assert_eq!(bytes_read, 0);
    }

    #[test]
    fn test_read_past_end() {
        let data = vec![1, 2, 3];
        let cursor = Cursor::new(data);
        let mut reader = BufferedMpqReader::new(cursor);

        let mut buf = [0u8; 5];
        let bytes_read = reader.read(&mut buf).unwrap();
        assert_eq!(bytes_read, 3);
        assert_eq!(&buf[..3], &[1, 2, 3]);

        let bytes_read = reader.read(&mut buf).unwrap();
        assert_eq!(bytes_read, 0);
    }

    #[test]
    fn test_seek_trait() {
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let cursor = Cursor::new(data);
        let mut reader = BufferedMpqReader::new(cursor);

        let pos = reader.seek(SeekFrom::Start(5)).unwrap();
        assert_eq!(pos, 5);

        let mut buf = [0u8; 2];
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(buf, [5, 6]);

        let pos = reader.seek(SeekFrom::Current(-3)).unwrap();
        assert_eq!(pos, 4);

        reader.read_exact(&mut buf).unwrap();
        assert_eq!(buf, [4, 5]);

        let pos = reader.seek(SeekFrom::End(-2)).unwrap();
        assert_eq!(pos, 8);

        reader.read_exact(&mut buf).unwrap();
        assert_eq!(buf, [8, 9]);
    }

    #[test]
    fn test_seek_beyond_bounds() {
        let data = vec![1, 2, 3, 4, 5];
        let cursor = Cursor::new(data);
        let mut reader = BufferedMpqReader::new(cursor);

        let pos = reader.seek(SeekFrom::Start(10)).unwrap();
        assert_eq!(pos, 10);

        let mut buf = [0u8; 1];
        let bytes_read = reader.read(&mut buf).unwrap();
        assert_eq!(bytes_read, 0);
    }

    #[test]
    fn test_seek_negative_position() {
        let data = vec![1, 2, 3, 4, 5];
        let cursor = Cursor::new(data);
        let mut reader = BufferedMpqReader::new(cursor);

        reader.seek(SeekFrom::Start(2)).unwrap();

        let result = reader.seek(SeekFrom::Current(-5));
        assert!(result.is_err());
    }

    #[test]
    fn test_combined_operations() {
        let data = vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
        let cursor = Cursor::new(data);
        let mut reader = BufferedMpqReader::new(cursor);

        // Read first 2 bytes: position goes from 0 to 2
        let mut buf = [0u8; 2];
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(buf, [10, 20]);

        // Seek forward 3 bytes: position goes from 2 to 5
        reader.seek(SeekFrom::Current(3)).unwrap();
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(buf, [60, 70]);

        // read_at doesn't change the current position, but after read_at seeks to offset
        let mut buf = [0u8; 3];
        reader.read_at(1, &mut buf).unwrap();
        assert_eq!(buf, [20, 30, 40]);

        // After read_at, the position is at 4 (1 + 3 bytes read)
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(buf, [50, 60, 70]);
    }

    #[test]
    fn test_large_buffer_read() {
        let mut data = Vec::new();
        for i in 0..1000 {
            data.push((i % 256) as u8);
        }

        let cursor = Cursor::new(data.clone());
        let mut reader = BufferedMpqReader::new(cursor);

        let mut buf = vec![0u8; 1000];
        reader.read_at(0, &mut buf).unwrap();
        assert_eq!(buf, data);

        let mut buf = vec![0u8; 500];
        reader.read_at(250, &mut buf).unwrap();
        assert_eq!(buf, data[250..750]);
    }

    #[test]
    fn test_zero_length_read_at() {
        let data = vec![1, 2, 3, 4, 5];
        let cursor = Cursor::new(data);
        let mut reader = BufferedMpqReader::new(cursor);

        let buf = &mut [];
        let result = reader.read_at(2, buf);
        assert!(result.is_ok());
    }

    struct MockSeekRead {
        data: Vec<u8>,
        position: usize,
        read_count: std::cell::Cell<usize>,
        seek_count: std::cell::Cell<usize>,
    }

    impl MockSeekRead {
        fn new(data: Vec<u8>) -> Self {
            Self {
                data,
                position: 0,
                read_count: std::cell::Cell::new(0),
                seek_count: std::cell::Cell::new(0),
            }
        }
    }

    impl Read for MockSeekRead {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.read_count.set(self.read_count.get() + 1);
            let available = self.data.len().saturating_sub(self.position);
            let to_read = buf.len().min(available);
            buf[..to_read].copy_from_slice(&self.data[self.position..self.position + to_read]);
            self.position += to_read;
            Ok(to_read)
        }
    }

    impl Seek for MockSeekRead {
        fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
            self.seek_count.set(self.seek_count.get() + 1);
            match pos {
                SeekFrom::Start(offset) => {
                    self.position = offset as usize;
                    Ok(offset)
                }
                SeekFrom::Current(offset) => {
                    let new_pos = (self.position as i64 + offset).max(0) as usize;
                    self.position = new_pos;
                    Ok(new_pos as u64)
                }
                SeekFrom::End(offset) => {
                    let new_pos = (self.data.len() as i64 + offset).max(0) as usize;
                    self.position = new_pos;
                    Ok(new_pos as u64)
                }
            }
        }
    }

    #[test]
    fn test_read_at_calls_seek_and_read() {
        let mock = MockSeekRead::new(vec![1, 2, 3, 4, 5]);
        let mut reader = BufferedMpqReader::new(mock);

        let mut buf = [0u8; 2];
        reader.read_at(2, &mut buf).unwrap();

        assert_eq!(reader.inner.seek_count.get(), 1);
        assert_eq!(reader.inner.read_count.get(), 1);
        assert_eq!(buf, [3, 4]);
    }
}
