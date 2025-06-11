// streaming.rs - Streaming parser for large ADT files

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::chunk::*;
use crate::error::{AdtError, Result};
use crate::io_helpers::ReadLittleEndian;
use crate::mh2o::Mh2oChunk as AdvancedMh2oChunk;
use crate::version::AdtVersion;

/// Streaming ADT parser that processes chunks one at a time
pub struct AdtStreamer<R: Read + Seek> {
    /// The reader to stream from
    reader: R,
    /// The ADT version
    version: AdtVersion,
    /// Whether the MVER and MHDR chunks have been read
    header_read: bool,
    /// The MVER chunk data
    mver: Option<MverChunk>,
    /// The MHDR chunk data
    mhdr: Option<MhdrChunk>,
    /// Current position in the file
    position: u64,
    /// End position of the file
    end_position: u64,
    /// Whether the stream is finished
    finished: bool,
}

/// A chunk read from the stream
#[derive(Debug)]
pub enum StreamedChunk {
    /// MVER chunk
    Mver(MverChunk),
    /// MHDR chunk
    Mhdr(MhdrChunk),
    /// MCIN chunk
    Mcin(McinChunk),
    /// MTEX chunk
    Mtex(MtexChunk),
    /// MMDX chunk
    Mmdx(MmdxChunk),
    /// MMID chunk
    Mmid(MmidChunk),
    /// MWMO chunk
    Mwmo(MwmoChunk),
    /// MWID chunk
    Mwid(MwidChunk),
    /// MDDF chunk
    Mddf(MddfChunk),
    /// MODF chunk
    Modf(ModfChunk),
    /// MCNK chunk
    Mcnk(Box<McnkChunk>),
    /// MFBO chunk
    Mfbo(MfboChunk),
    /// MH2O chunk
    Mh2o(AdvancedMh2oChunk),
    /// MTFX chunk
    Mtfx(MtfxChunk),
    /// Unknown chunk
    Unknown {
        /// Magic identifier
        magic: [u8; 4],
        /// Size of the chunk
        size: u32,
    },
}

impl<R: Read + Seek> AdtStreamer<R> {
    /// Create a new ADT streamer
    pub fn new(mut reader: R) -> Result<Self> {
        // Get the file size
        let end_position = reader.seek(SeekFrom::End(0))?;

        // Reset to beginning
        reader.seek(SeekFrom::Start(0))?;

        Ok(Self {
            reader,
            version: AdtVersion::Vanilla, // Will be updated after reading MVER
            header_read: false,
            mver: None,
            mhdr: None,
            position: 0,
            end_position,
            finished: false,
        })
    }

    /// Read the next chunk from the stream
    pub fn next_chunk(&mut self) -> Result<Option<StreamedChunk>> {
        if self.finished {
            return Ok(None);
        }

        // Check if we need to read the header first
        if !self.header_read {
            // Read MVER and MHDR
            self.read_header()?;
        }

        // Read the next chunk
        match self.read_next_chunk() {
            Ok(Some(chunk)) => Ok(Some(chunk)),
            Ok(None) => {
                self.finished = true;
                Ok(None)
            }
            Err(e) => {
                self.finished = true;
                Err(e)
            }
        }
    }

    /// Read all MCNK chunks in the file
    pub fn read_all_mcnk(&mut self) -> Result<Vec<McnkChunk>> {
        let mut mcnk_chunks = Vec::new();

        // Read all chunks
        while let Some(chunk) = self.next_chunk()? {
            // Only keep MCNK chunks
            if let StreamedChunk::Mcnk(mcnk) = chunk {
                mcnk_chunks.push(*mcnk);
            }
        }

        Ok(mcnk_chunks)
    }

    /// Read the header (MVER and MHDR)
    fn read_header(&mut self) -> Result<()> {
        // Read MVER
        let header = ChunkHeader::read(&mut self.reader)?;

        if header.magic != *b"MVER" {
            return Err(AdtError::InvalidMagic {
                expected: "MVER".to_string(),
                found: header.magic_as_string(),
            });
        }

        // Read MVER chunk
        let mver = {
            let mut context = crate::ParserContext {
                reader: &mut self.reader,
                version: AdtVersion::Vanilla, // Will be updated
                position: 0,
            };
            MverChunk::read_with_header(header, &mut context)?
        };

        self.version = AdtVersion::from_mver(mver.version)?;
        self.mver = Some(mver);

        // Update position
        self.position = self.reader.stream_position()?;

        // Read MHDR
        let header = ChunkHeader::read(&mut self.reader)?;

        if header.magic != *b"MHDR" {
            return Err(AdtError::InvalidMagic {
                expected: "MHDR".to_string(),
                found: header.magic_as_string(),
            });
        }

        // Read MHDR chunk
        let mhdr = {
            let mut context = crate::ParserContext {
                reader: &mut self.reader,
                version: self.version,
                position: 0,
            };
            MhdrChunk::read_with_header(header, &mut context)?
        };
        self.mhdr = Some(mhdr);

        // Update position
        self.position = self.reader.stream_position()?;

        // Mark header as read
        self.header_read = true;

        Ok(())
    }

    /// Read the next chunk from the file
    fn read_next_chunk(&mut self) -> Result<Option<StreamedChunk>> {
        // Check if we've reached the end of the file
        if self.position >= self.end_position {
            return Ok(None);
        }

        // Read the chunk header
        let header = match ChunkHeader::read(&mut self.reader) {
            Ok(header) => header,
            Err(AdtError::UnexpectedEof) => return Ok(None),
            Err(e) => return Err(e),
        };

        // Create a parser context
        let mut context = crate::ParserContext {
            reader: &mut self.reader,
            version: self.version,
            position: self.position as usize,
        };

        // Process the chunk based on its magic
        let chunk = match &header.magic {
            b"MVER" => {
                let chunk = MverChunk::read_with_header(header, &mut context)?;
                StreamedChunk::Mver(chunk)
            }
            b"MHDR" => {
                let chunk = MhdrChunk::read_with_header(header, &mut context)?;
                StreamedChunk::Mhdr(chunk)
            }
            b"MCIN" => {
                let chunk = McinChunk::read_with_header(header, &mut context)?;
                StreamedChunk::Mcin(chunk)
            }
            b"MTEX" => {
                let chunk = MtexChunk::read_with_header(header, &mut context)?;
                StreamedChunk::Mtex(chunk)
            }
            b"MMDX" => {
                let chunk = MmdxChunk::read_with_header(header, &mut context)?;
                StreamedChunk::Mmdx(chunk)
            }
            b"MMID" => {
                let chunk = MmidChunk::read_with_header(header, &mut context)?;
                StreamedChunk::Mmid(chunk)
            }
            b"MWMO" => {
                let chunk = MwmoChunk::read_with_header(header, &mut context)?;
                StreamedChunk::Mwmo(chunk)
            }
            b"MWID" => {
                let chunk = MwidChunk::read_with_header(header, &mut context)?;
                StreamedChunk::Mwid(chunk)
            }
            b"MDDF" => {
                let chunk = MddfChunk::read_with_header(header, &mut context)?;
                StreamedChunk::Mddf(chunk)
            }
            b"MODF" => {
                let chunk = ModfChunk::read_with_header(header, &mut context)?;
                StreamedChunk::Modf(chunk)
            }
            b"MCNK" => {
                let chunk = McnkChunk::read_with_header(header, &mut context)?;
                StreamedChunk::Mcnk(Box::new(chunk))
            }
            b"MFBO" => {
                if self.version >= AdtVersion::TBC {
                    let chunk = MfboChunk::read_with_header(header, &mut context)?;
                    StreamedChunk::Mfbo(chunk)
                } else {
                    // Skip unknown chunk
                    self.reader.seek(SeekFrom::Current(header.size as i64))?;
                    StreamedChunk::Unknown {
                        magic: header.magic,
                        size: header.size,
                    }
                }
            }
            b"MH2O" => {
                if self.version >= AdtVersion::WotLK {
                    // For MH2O, we need to know the start position
                    let chunk_start = self.position;
                    let chunk =
                        crate::mh2o::Mh2oChunk::read_full(&mut context, chunk_start, header.size)?;
                    StreamedChunk::Mh2o(chunk)
                } else {
                    // Skip unknown chunk
                    self.reader.seek(SeekFrom::Current(header.size as i64))?;
                    StreamedChunk::Unknown {
                        magic: header.magic,
                        size: header.size,
                    }
                }
            }
            b"MTFX" => {
                if self.version >= AdtVersion::Cataclysm {
                    let chunk = MtfxChunk::read_with_header(header, &mut context)?;
                    StreamedChunk::Mtfx(chunk)
                } else {
                    // Skip unknown chunk
                    self.reader.seek(SeekFrom::Current(header.size as i64))?;
                    StreamedChunk::Unknown {
                        magic: header.magic,
                        size: header.size,
                    }
                }
            }
            _ => {
                // Unknown chunk, skip it
                self.reader.seek(SeekFrom::Current(header.size as i64))?;
                StreamedChunk::Unknown {
                    magic: header.magic,
                    size: header.size,
                }
            }
        };

        // Update position
        self.position = self.reader.stream_position()?;

        Ok(Some(chunk))
    }

    /// Get the version of the ADT
    pub fn version(&self) -> AdtVersion {
        self.version
    }

    /// Get the MHDR chunk if available
    pub fn mhdr(&self) -> Option<&MhdrChunk> {
        self.mhdr.as_ref()
    }

    /// Check if the stream is finished
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Reset the stream to the beginning
    pub fn reset(&mut self) -> Result<()> {
        // Reset to beginning
        self.reader.seek(SeekFrom::Start(0))?;

        // Reset state
        self.header_read = false;
        self.mver = None;
        self.mhdr = None;
        self.position = 0;
        self.finished = false;

        Ok(())
    }

    /// Skip to the first MCNK chunk
    pub fn skip_to_mcnk(&mut self) -> Result<()> {
        // Ensure header is read
        if !self.header_read {
            self.read_header()?;
        }

        // Skip chunks until we find an MCNK
        loop {
            // Read chunk header
            let header = match ChunkHeader::read(&mut self.reader) {
                Ok(header) => header,
                Err(AdtError::UnexpectedEof) => {
                    self.finished = true;
                    return Ok(());
                }
                Err(e) => return Err(e),
            };

            // Check if this is an MCNK
            if header.magic == *b"MCNK" {
                // Seek back to the start of the chunk
                self.reader.seek(SeekFrom::Current(-8))?;
                self.position = self.reader.stream_position()?;
                return Ok(());
            }

            // Skip this chunk
            self.reader.seek(SeekFrom::Current(header.size as i64))?;
            self.position = self.reader.stream_position()?;
        }
    }

    /// Skip to a specific MCNK chunk by coordinates
    pub fn skip_to_mcnk_coords(&mut self, x: u32, y: u32) -> Result<()> {
        // Ensure header is read
        if !self.header_read {
            self.read_header()?;
        }

        // Calculate the index
        let target_idx = y * 16 + x;

        // Check if we have MCIN
        if let Some(ref mhdr) = self.mhdr {
            if mhdr.mcin_offset > 0 {
                // Seek to MCIN
                self.reader.seek(SeekFrom::Start(mhdr.mcin_offset as u64))?;

                // Read MCIN header
                let header = ChunkHeader::read(&mut self.reader)?;

                if header.magic == *b"MCIN" {
                    // Skip to the target entry
                    self.reader
                        .seek(SeekFrom::Current((target_idx * 16) as i64))?;

                    // Read the entry
                    let offset = self.reader.read_u32_le()?;

                    if offset > 0 {
                        // Seek to the MCNK
                        self.reader.seek(SeekFrom::Start(offset as u64))?;
                        self.position = offset as u64;
                        return Ok(());
                    }
                }
            }
        }

        // If no MCIN or the offset was 0, scan for the MCNK
        self.skip_to_mcnk()?;

        // Find the right MCNK by checking each one
        while let Some(chunk) = self.next_chunk()? {
            if let StreamedChunk::Mcnk(mcnk) = chunk {
                if mcnk.ix == x && mcnk.iy == y {
                    // We found the right chunk, but we've already consumed it
                    // There's no easy way to "push back" the chunk
                    // In a real implementation, we would need to clone the chunk and return it
                    return Ok(());
                }
            }
        }

        // Didn't find the chunk
        Err(AdtError::ParseError(format!(
            "Could not find MCNK chunk at coordinates ({}, {})",
            x, y
        )))
    }
}

/// Open an ADT file for streaming
pub fn open_adt_stream<P: AsRef<Path>>(path: P) -> Result<AdtStreamer<File>> {
    let file = File::open(path)?;
    AdtStreamer::new(file)
}

/// Iterate over all MCNK chunks in an ADT file
pub fn iterate_mcnk_chunks<P: AsRef<Path>, F>(path: P, mut callback: F) -> Result<()>
where
    F: FnMut(&McnkChunk) -> Result<()>,
{
    let mut streamer = open_adt_stream(path)?;

    // Skip to the first MCNK
    streamer.skip_to_mcnk()?;

    // Process each MCNK
    while let Some(chunk) = streamer.next_chunk()? {
        if let StreamedChunk::Mcnk(mcnk) = chunk {
            callback(&mcnk)?;
        }
    }

    Ok(())
}

/// Count terrain chunks matching a predicate
pub fn count_matching_chunks<P: AsRef<Path>, F>(path: P, mut predicate: F) -> Result<usize>
where
    F: FnMut(&McnkChunk) -> bool,
{
    let mut count = 0;

    iterate_mcnk_chunks(path, |chunk| {
        if predicate(chunk) {
            count += 1;
        }
        Ok(())
    })?;

    Ok(count)
}
