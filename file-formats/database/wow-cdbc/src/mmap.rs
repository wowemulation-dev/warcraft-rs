//! Memory-mapped DBC file handling

use crate::{
    DbcHeader, DbcParser, DbcVersion, Error, Result, Schema, StringBlock,
    versions::{Wdb2Header, Wdb5Header},
};
use memmap2::{Mmap, MmapOptions};
use std::fs::File;
use std::io::{Cursor, Seek, SeekFrom};
use std::path::Path;

/// A memory-mapped DBC file
pub struct MmapDbcFile {
    /// The memory-mapped file
    mmap: Mmap,
    /// The DBC version
    version: DbcVersion,
    /// The DBC header
    header: DbcHeader,
}

impl MmapDbcFile {
    /// Open a DBC file using memory mapping
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };

        // Create a cursor to read from the memory-mapped file
        let mut cursor = Cursor::new(&mmap[..]);

        // Detect the DBC version
        let version = DbcVersion::detect(&mut cursor)?;

        // Parse the header based on the version
        let header = match version {
            DbcVersion::WDBC => DbcHeader::parse(&mut cursor)?,
            DbcVersion::WDB2 => {
                let wdb2_header = Wdb2Header::parse(&mut cursor)?;
                wdb2_header.to_dbc_header()
            }
            DbcVersion::WDB5 => {
                let wdb5_header = Wdb5Header::parse(&mut cursor)?;
                wdb5_header.to_dbc_header()
            }
            _ => {
                return Err(Error::InvalidHeader(format!(
                    "Unsupported DBC version: {:?}",
                    version
                )));
            }
        };

        Ok(Self {
            mmap,
            version,
            header,
        })
    }

    /// Get a slice of the memory-mapped file
    pub fn as_slice(&self) -> &[u8] {
        &self.mmap[..]
    }

    /// Get the DBC header
    pub fn header(&self) -> &DbcHeader {
        &self.header
    }

    /// Get the DBC version
    pub fn version(&self) -> DbcVersion {
        self.version
    }

    /// Create a DBC parser from the memory-mapped file
    pub fn parser(&self) -> DbcParser {
        DbcParser::parse_bytes(self.as_slice()).unwrap()
    }

    /// Create a DBC parser with a schema from the memory-mapped file
    pub fn parser_with_schema(&self, schema: Schema) -> Result<DbcParser> {
        self.parser().with_schema(schema)
    }

    /// Get the string block from the memory-mapped file
    pub fn string_block(&self) -> Result<StringBlock> {
        let mut cursor = Cursor::new(self.as_slice());
        cursor.seek(SeekFrom::Start(self.header.string_block_offset()))?;
        StringBlock::parse(
            &mut cursor,
            self.header.string_block_offset(),
            self.header.string_block_size,
        )
    }
}
