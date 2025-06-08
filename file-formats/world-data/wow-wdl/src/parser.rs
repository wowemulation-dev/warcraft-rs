//! Parser implementation for WDL files
//!
//! This module provides the main functionality for reading and writing WDL files.
//! The [`WdlParser`] struct is the primary entry point for working with WDL files.

use memchr::memchr;
use std::collections::HashMap;
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};

use crate::error::{Result, WdlError};
use crate::types::*;
use crate::version::WdlVersion;

/// Parser for WDL (World Distance Lookup) files
///
/// The `WdlParser` provides methods to read and write WDL files, with support for
/// different game versions. It automatically handles the different chunk formats
/// and structures used across World of Warcraft expansions.
///
/// # Examples
///
/// ```rust,no_run
/// use std::fs::File;
/// use std::io::{BufReader, BufWriter};
/// use wow_wdl::parser::WdlParser;
/// use wow_wdl::version::WdlVersion;
///
/// // Parse a WDL file
/// let file = File::open("input.wdl").unwrap();
/// let mut reader = BufReader::new(file);
/// let parser = WdlParser::new();
/// let wdl_file = parser.parse(&mut reader).unwrap();
///
/// // Write a WDL file
/// let output = File::create("output.wdl").unwrap();
/// let mut writer = BufWriter::new(output);
/// let wotlk_parser = WdlParser::with_version(WdlVersion::Wotlk);
/// wotlk_parser.write(&mut writer, &wdl_file).unwrap();
/// ```
#[derive(Debug, Default)]
pub struct WdlParser {
    /// The version to use for parsing
    version: WdlVersion,
}

impl WdlParser {
    /// Creates a new WDL parser with the latest version
    ///
    /// This creates a parser configured to work with the most recent
    /// WDL format version supported by the library.
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_wdl::parser::WdlParser;
    ///
    /// let parser = WdlParser::new();
    /// ```
    pub fn new() -> Self {
        Self {
            version: WdlVersion::Latest,
        }
    }

    /// Creates a WDL parser with the specified version
    ///
    /// This allows parsing and writing WDL files using a specific game version's
    /// format. This is useful when you know exactly which version you're working with.
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_wdl::parser::WdlParser;
    /// use wow_wdl::version::WdlVersion;
    ///
    /// let parser = WdlParser::with_version(WdlVersion::Wotlk);
    /// ```
    pub fn with_version(version: WdlVersion) -> Self {
        Self { version }
    }

    /// Sets the version for the parser
    ///
    /// This changes the version used by the parser for future operations.
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_wdl::parser::WdlParser;
    /// use wow_wdl::version::WdlVersion;
    ///
    /// let mut parser = WdlParser::new();
    /// parser.set_version(WdlVersion::Legion);
    /// ```
    pub fn set_version(&mut self, version: WdlVersion) {
        self.version = version;
    }

    /// Gets the current version of the parser
    ///
    /// # Examples
    ///
    /// ```
    /// use wow_wdl::parser::WdlParser;
    /// use wow_wdl::version::WdlVersion;
    ///
    /// let parser = WdlParser::with_version(WdlVersion::Wotlk);
    /// assert_eq!(parser.version(), WdlVersion::Wotlk);
    /// ```
    pub fn version(&self) -> WdlVersion {
        self.version
    }

    /// Parses a WDL file from a reader
    ///
    /// This method reads a WDL file from the provided reader and returns a
    /// parsed `WdlFile` structure. It automatically detects the file version
    /// and parses all relevant chunks.
    ///
    /// # Arguments
    ///
    /// * `reader` - Any type that implements `Read + Seek`
    ///
    /// # Returns
    ///
    /// A `Result` containing either the parsed `WdlFile` or an error.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::fs::File;
    /// use std::io::BufReader;
    /// use wow_wdl::parser::WdlParser;
    ///
    /// let file = File::open("input.wdl").unwrap();
    /// let mut reader = BufReader::new(file);
    /// let parser = WdlParser::new();
    /// let wdl_file = parser.parse(&mut reader).unwrap();
    /// ```
    pub fn parse<R: Read + Seek>(&self, reader: &mut R) -> Result<WdlFile> {
        let mut file = WdlFile::new();
        file.version = self.version;

        let mut mver_found = false;
        let mut mwmo_index = None;
        let mut mwid_index = None;
        let mut modf_index = None;
        let mut maof_index = None;
        let mut mldd_index = None;
        let mut mldx_index = None;
        let mut mlmd_index = None;
        let mut mlmx_index = None;

        // First, we read all chunks to get an overview of the file
        let mut chunk_index = 0;
        loop {
            let chunk = match Chunk::read(reader) {
                Ok(chunk) => chunk,
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(WdlError::Io(e)),
            };

            // Check for specific chunks
            match chunk.magic {
                MVER_MAGIC => {
                    mver_found = true;
                    // Parse version number
                    let mut cursor = Cursor::new(&chunk.data);
                    let mut buf = [0u8; 4];
                    cursor.read_exact(&mut buf).map_err(WdlError::Io)?;
                    file.version_number = u32::from_le_bytes(buf);
                }
                MWMO_MAGIC => mwmo_index = Some(chunk_index),
                MWID_MAGIC => mwid_index = Some(chunk_index),
                MODF_MAGIC => modf_index = Some(chunk_index),
                MAOF_MAGIC => maof_index = Some(chunk_index),
                MLDD_MAGIC => mldd_index = Some(chunk_index),
                MLDX_MAGIC => mldx_index = Some(chunk_index),
                MLMD_MAGIC => mlmd_index = Some(chunk_index),
                MLMX_MAGIC => mlmx_index = Some(chunk_index),
                _ => {}
            }

            file.chunks.push(chunk);
            chunk_index += 1;
        }

        // Check if we found the MVER chunk
        if !mver_found {
            return Err(WdlError::InvalidMagic {
                expected: String::from_utf8_lossy(&MVER_MAGIC).to_string(),
                found: "Not found".to_string(),
            });
        }

        // Detect version based on chunks present if not explicitly set by parser
        if self.version == WdlVersion::Latest {
            // If we have ML* chunks, it's Legion or later
            if mldd_index.is_some()
                || mldx_index.is_some()
                || mlmd_index.is_some()
                || mlmx_index.is_some()
            {
                file.version = WdlVersion::Legion;
            }
            // If we have WMO chunks, it's pre-Legion
            else if mwmo_index.is_some() || mwid_index.is_some() || modf_index.is_some() {
                // Check for MAHO to distinguish WotLK+ from Vanilla
                if file.chunks.iter().any(|c| c.magic == MAHO_MAGIC) {
                    file.version = WdlVersion::Wotlk;
                } else {
                    file.version = WdlVersion::Vanilla;
                }
            }
            // Otherwise keep the parser's version
        }

        // Parse MWMO chunk (WMO filenames)
        if let Some(index) = mwmo_index {
            let chunk = &file.chunks[index];
            file.wmo_filenames = self.parse_zero_terminated_strings(&chunk.data)?;
        }

        // Parse MWID chunk (WMO indices)
        if let Some(index) = mwid_index {
            let chunk = &file.chunks[index];
            let mut cursor = Cursor::new(&chunk.data);

            while cursor.position() < chunk.data.len() as u64 {
                let mut buf = [0u8; 4];
                match cursor.read_exact(&mut buf) {
                    Ok(_) => file.wmo_indices.push(u32::from_le_bytes(buf)),
                    Err(_) => break,
                }
            }
        }

        // Parse MODF chunk (WMO placements)
        if let Some(index) = modf_index {
            let chunk = &file.chunks[index];
            let mut cursor = Cursor::new(&chunk.data);

            while cursor.position() < chunk.data.len() as u64 {
                match ModelPlacement::read(&mut cursor) {
                    Ok(placement) => file.wmo_placements.push(placement),
                    Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                    Err(e) => return Err(WdlError::Io(e)),
                }
            }
        }

        // Parse MAOF chunk (Map tile offsets)
        if let Some(index) = maof_index {
            let chunk = &file.chunks[index];
            let mut cursor = Cursor::new(&chunk.data);

            for i in 0..64 * 64 {
                let mut buf = [0u8; 4];
                cursor.read_exact(&mut buf).map_err(WdlError::Io)?;
                file.map_tile_offsets[i] = u32::from_le_bytes(buf);
            }

            // Now parse the MARE and MAHO chunks using the offsets
            self.parse_map_tiles(reader, &mut file)?;
        }

        // Parse Legion+ chunks
        if file.version.has_ml_chunks() {
            // Parse MLDD chunk (M2 placements)
            if let Some(index) = mldd_index {
                let chunk = &file.chunks[index];
                let mut cursor = Cursor::new(&chunk.data);

                while cursor.position() < chunk.data.len() as u64 {
                    match M2Placement::read(&mut cursor) {
                        Ok(placement) => file.m2_placements.push(placement),
                        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                        Err(e) => return Err(WdlError::Io(e)),
                    }
                }
            }

            // Parse MLDX chunk (M2 visibility info)
            if let Some(index) = mldx_index {
                let chunk = &file.chunks[index];
                let mut cursor = Cursor::new(&chunk.data);

                while cursor.position() < chunk.data.len() as u64 {
                    match M2VisibilityInfo::read(&mut cursor) {
                        Ok(info) => file.m2_visibility.push(info),
                        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                        Err(e) => return Err(WdlError::Io(e)),
                    }
                }
            }

            // Parse MLMD chunk (WMO Legion placements)
            if let Some(index) = mlmd_index {
                let chunk = &file.chunks[index];
                let mut cursor = Cursor::new(&chunk.data);

                while cursor.position() < chunk.data.len() as u64 {
                    match M2Placement::read(&mut cursor) {
                        Ok(placement) => file.wmo_legion_placements.push(placement),
                        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                        Err(e) => return Err(WdlError::Io(e)),
                    }
                }
            }

            // Parse MLMX chunk (WMO Legion visibility info)
            if let Some(index) = mlmx_index {
                let chunk = &file.chunks[index];
                let mut cursor = Cursor::new(&chunk.data);

                while cursor.position() < chunk.data.len() as u64 {
                    match M2VisibilityInfo::read(&mut cursor) {
                        Ok(info) => file.wmo_legion_visibility.push(info),
                        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                        Err(e) => return Err(WdlError::Io(e)),
                    }
                }
            }
        }

        // Validate the file
        file.validate()?;

        Ok(file)
    }

    /// Writes a WDL file to a writer
    ///
    /// This method writes a `WdlFile` structure to the provided writer using
    /// the format specified by the parser's version.
    ///
    /// # Arguments
    ///
    /// * `writer` - Any type that implements `Write + Seek`
    /// * `file` - The `WdlFile` to write
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::fs::File;
    /// use std::io::BufWriter;
    /// use wow_wdl::parser::WdlParser;
    /// use wow_wdl::types::WdlFile;
    /// use wow_wdl::version::WdlVersion;
    ///
    /// // Create a new WDL file
    /// let file = WdlFile::with_version(WdlVersion::Wotlk);
    ///
    /// // Write the file
    /// let output = File::create("output.wdl").unwrap();
    /// let mut writer = BufWriter::new(output);
    /// let parser = WdlParser::with_version(WdlVersion::Wotlk);
    /// parser.write(&mut writer, &file).unwrap();
    /// ```
    pub fn write<W: Write + Seek>(&self, writer: &mut W, file: &WdlFile) -> Result<()> {
        // We'll build the file in memory first to calculate offsets
        let mut chunks = Vec::new();

        // Write MVER chunk
        let mut mver_data = Vec::new();
        mver_data
            .write_all(&file.version.version_number().to_le_bytes())
            .map_err(WdlError::Io)?;
        chunks.push(Chunk::new(MVER_MAGIC, mver_data));

        // Write WMO chunks if supported
        if file.version.has_wmo_chunks() && !file.wmo_filenames.is_empty() {
            // Write MWMO chunk (WMO filenames)
            let mut mwmo_data = Vec::new();
            for name in &file.wmo_filenames {
                mwmo_data.extend_from_slice(name.as_bytes());
                mwmo_data.push(0); // Null terminator
            }
            chunks.push(Chunk::new(MWMO_MAGIC, mwmo_data));

            // Write MWID chunk (WMO indices)
            let mut mwid_data = Vec::new();
            for &idx in &file.wmo_indices {
                mwid_data
                    .write_all(&idx.to_le_bytes())
                    .map_err(WdlError::Io)?;
            }
            chunks.push(Chunk::new(MWID_MAGIC, mwid_data));

            // Write MODF chunk (WMO placements)
            let mut modf_data = Vec::new();
            for placement in &file.wmo_placements {
                placement.write(&mut modf_data).map_err(WdlError::Io)?;
            }
            chunks.push(Chunk::new(MODF_MAGIC, modf_data));
        }

        // Write Legion+ chunks if supported
        if file.version.has_ml_chunks() {
            // Write MLDD chunk (M2 placements)
            if !file.m2_placements.is_empty() {
                let mut mldd_data = Vec::new();
                for placement in &file.m2_placements {
                    placement.write(&mut mldd_data).map_err(WdlError::Io)?;
                }
                chunks.push(Chunk::new(MLDD_MAGIC, mldd_data));
            }

            // Write MLDX chunk (M2 visibility info)
            if !file.m2_visibility.is_empty() {
                let mut mldx_data = Vec::new();
                for info in &file.m2_visibility {
                    info.write(&mut mldx_data).map_err(WdlError::Io)?;
                }
                chunks.push(Chunk::new(MLDX_MAGIC, mldx_data));
            }

            // Write MLMD chunk (WMO Legion placements)
            if !file.wmo_legion_placements.is_empty() {
                let mut mlmd_data = Vec::new();
                for placement in &file.wmo_legion_placements {
                    placement.write(&mut mlmd_data).map_err(WdlError::Io)?;
                }
                chunks.push(Chunk::new(MLMD_MAGIC, mlmd_data));
            }

            // Write MLMX chunk (WMO Legion visibility info)
            if !file.wmo_legion_visibility.is_empty() {
                let mut mlmx_data = Vec::new();
                for info in &file.wmo_legion_visibility {
                    info.write(&mut mlmx_data).map_err(WdlError::Io)?;
                }
                chunks.push(Chunk::new(MLMX_MAGIC, mlmx_data));
            }
        }

        // Now we need to determine the positions of the MARE and MAHO chunks
        // So we can write the correct offsets in the MAOF chunk
        let mut map_tile_offsets = [0u32; 64 * 64];
        let mut mare_chunks = HashMap::new();
        let mut maho_chunks = HashMap::new();

        // Calculate the base offset for the MARE and MAHO chunks
        // which is right after the MAOF chunk
        let mut current_offset = 0;

        // Account for all chunks written so far, plus the MAOF chunk
        for chunk in &chunks {
            current_offset += 8 + chunk.size; // 8 bytes for magic and size
        }

        // Add the MAOF chunk size
        current_offset += 8 + (64 * 64 * 4); // 8 bytes for magic and size, 4 bytes per offset

        // Now calculate offsets for each map tile
        for y in 0..64 {
            for x in 0..64 {
                let index = y * 64 + x;
                let key = (x as u32, y as u32);

                // Skip empty tiles
                if !file.heightmap_tiles.contains_key(&key) {
                    map_tile_offsets[index] = 0;
                    continue;
                }

                // Set the offset for this tile
                map_tile_offsets[index] = current_offset;

                // Create MARE chunk
                let heightmap = file.heightmap_tiles.get(&key).unwrap();
                let mut mare_data = Vec::new();
                heightmap.write(&mut mare_data).map_err(WdlError::Io)?;
                mare_chunks.insert(key, Chunk::new(MARE_MAGIC, mare_data));

                // Update offset for next chunk
                current_offset += 8 + (HeightMapTile::TOTAL_COUNT * 2) as u32; // 8 bytes for magic and size, 2 bytes per height value

                // Create MAHO chunk if needed
                if file.version.has_maho_chunk() {
                    if let Some(holes) = file.holes_data.get(&key) {
                        let mut maho_data = Vec::new();
                        holes.write(&mut maho_data).map_err(WdlError::Io)?;
                        maho_chunks.insert(key, Chunk::new(MAHO_MAGIC, maho_data));

                        // Update offset for next chunk
                        current_offset += 8 + (HolesData::MASK_COUNT * 2) as u32;
                        // 8 bytes for magic and size, 2 bytes per mask
                    }
                }
            }
        }

        // Write MAOF chunk (Map tile offsets)
        let mut maof_data = Vec::new();
        for &offset in &map_tile_offsets {
            maof_data
                .write_all(&offset.to_le_bytes())
                .map_err(WdlError::Io)?;
        }
        chunks.push(Chunk::new(MAOF_MAGIC, maof_data));

        // Write all the chunks that we've prepared
        for chunk in &chunks {
            chunk.write(writer).map_err(WdlError::Io)?;
        }

        // Write the MARE and MAHO chunks for each map tile
        for y in 0..64 {
            for x in 0..64 {
                let key = (x as u32, y as u32);

                // Skip empty tiles
                if !mare_chunks.contains_key(&key) {
                    continue;
                }

                // Write MARE chunk
                mare_chunks
                    .get(&key)
                    .unwrap()
                    .write(writer)
                    .map_err(WdlError::Io)?;

                // Write MAHO chunk if present
                if let Some(maho_chunk) = maho_chunks.get(&key) {
                    maho_chunk.write(writer).map_err(WdlError::Io)?;
                }
            }
        }

        Ok(())
    }

    /// Parses zero-terminated strings from a buffer
    ///
    /// Internal helper method to parse null-terminated strings from a byte buffer.
    ///
    /// # Arguments
    ///
    /// * `data` - The byte buffer containing the strings
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of strings or an error.
    fn parse_zero_terminated_strings(&self, data: &[u8]) -> Result<Vec<String>> {
        let mut strings = Vec::new();
        let mut start = 0;

        while start < data.len() {
            match memchr(0, &data[start..]) {
                Some(end) => {
                    match String::from_utf8(data[start..start + end].to_vec()) {
                        Ok(s) => strings.push(s),
                        Err(_) => {
                            return Err(WdlError::ParseError(
                                "Invalid UTF-8 in string".to_string(),
                            ));
                        }
                    }
                    start += end + 1; // Skip the null terminator
                }
                None => break,
            }
        }

        Ok(strings)
    }

    /// Parses map tiles (MARE and MAHO chunks) from the reader
    ///
    /// Internal helper method to parse map tile data referenced by offsets in the MAOF chunk.
    ///
    /// # Arguments
    ///
    /// * `reader` - The reader containing the file data
    /// * `file` - The WdlFile being populated
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error.
    fn parse_map_tiles<R: Read + Seek>(&self, reader: &mut R, file: &mut WdlFile) -> Result<()> {
        for y in 0..64 {
            for x in 0..64 {
                let index = y * 64 + x;
                let offset = file.map_tile_offsets[index];

                if offset == 0 {
                    continue; // No data for this tile
                }

                // Seek to the offset
                reader
                    .seek(SeekFrom::Start(offset as u64))
                    .map_err(WdlError::Io)?;

                // Read the MARE chunk
                let chunk = Chunk::read(reader).map_err(WdlError::Io)?;

                if chunk.magic != MARE_MAGIC {
                    return Err(WdlError::UnexpectedChunk(
                        String::from_utf8_lossy(&chunk.magic).to_string(),
                    ));
                }

                // Parse the heightmap
                let mut cursor = Cursor::new(&chunk.data);
                let heightmap = HeightMapTile::read(&mut cursor).map_err(WdlError::Io)?;

                file.heightmap_tiles.insert((x as u32, y as u32), heightmap);

                // Check for MAHO chunk
                if self.version.has_maho_chunk() {
                    match Chunk::read(reader) {
                        Ok(chunk) => {
                            if chunk.magic == MAHO_MAGIC {
                                let mut cursor = Cursor::new(&chunk.data);
                                let holes = HolesData::read(&mut cursor).map_err(WdlError::Io)?;

                                file.holes_data.insert((x as u32, y as u32), holes);
                            } else {
                                // Seek back, this wasn't a MAHO chunk
                                reader
                                    .seek(SeekFrom::Current(-(8 + chunk.size as i64)))
                                    .map_err(WdlError::Io)?;
                            }
                        }
                        Err(_) => {
                            // No MAHO chunk, that's fine
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_zero_terminated_strings() {
        let parser = WdlParser::new();

        let data = b"test1\0test2\0test3\0";
        let strings = parser.parse_zero_terminated_strings(data).unwrap();

        assert_eq!(strings.len(), 3);
        assert_eq!(strings[0], "test1");
        assert_eq!(strings[1], "test2");
        assert_eq!(strings[2], "test3");
    }

    #[test]
    fn test_empty_write_read() {
        let parser = WdlParser::new();
        let file = WdlFile::new();

        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        parser.write(&mut cursor, &file).unwrap();

        let mut cursor = Cursor::new(buffer);
        let parsed_file = parser.parse(&mut cursor).unwrap();

        assert_eq!(parsed_file.version, file.version);
        assert_eq!(parsed_file.version_number, file.version_number);
    }
}
