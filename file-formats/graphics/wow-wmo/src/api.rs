use crate::chunk_discovery::{ChunkDiscovery, discover_chunks};
use crate::error::WmoError;
use crate::file_type::{WmoFileType, detect_file_type};
use crate::group_parser::{WmoGroup, parse_group_file};
use crate::root_parser::{WmoRoot, parse_root_file};
use std::io::{Read, Seek, SeekFrom};

/// Result of parsing a WMO file
#[derive(Debug, Clone)]
pub enum ParsedWmo {
    /// Root WMO file containing header and references
    Root(WmoRoot),
    /// Group file containing geometry data
    Group(WmoGroup),
}

impl ParsedWmo {
    /// Get the file type
    pub fn file_type(&self) -> WmoFileType {
        match self {
            ParsedWmo::Root(_) => WmoFileType::Root,
            ParsedWmo::Group(_) => WmoFileType::Group,
        }
    }

    /// Get the version (always 17 for supported files)
    pub fn version(&self) -> u32 {
        match self {
            ParsedWmo::Root(r) => r.version,
            ParsedWmo::Group(g) => g.version,
        }
    }
}

/// Result of parsing with additional metadata
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// The parsed WMO data
    pub wmo: ParsedWmo,
    /// Chunk discovery information
    pub discovery: ChunkDiscovery,
}

impl ParseResult {
    /// Get metadata about the file parsing
    pub fn metadata(&self) -> Option<&ChunkDiscovery> {
        Some(&self.discovery)
    }
}

/// Parse a WMO file (root or group) with automatic type detection
///
/// This is the main entry point for parsing WMO files. It will:
/// 1. Discover all chunks in the file (Stage 1)
/// 2. Determine if it's a root or group file
/// 3. Parse with the appropriate parser (Stage 2)
///
/// # Example
/// ```no_run
/// use std::fs::File;
/// use std::io::BufReader;
/// use wow_wmo::{parse_wmo, ParsedWmo};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let file = File::open("example.wmo")?;
/// let mut reader = BufReader::new(file);
/// let wmo = parse_wmo(&mut reader)?;
///
/// match wmo {
///     ParsedWmo::Root(root) => {
///         println!("Root file with {} groups", root.n_groups);
///     }
///     ParsedWmo::Group(group) => {
///         println!("Group {} with {} vertices", group.group_index, group.n_vertices);
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub fn parse_wmo<R: Read + Seek>(reader: &mut R) -> Result<ParsedWmo, WmoError> {
    // Stage 1: Discover chunks
    let discovery = discover_chunks(reader).map_err(|e| {
        WmoError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to discover chunks: {}", e),
        ))
    })?;

    // Detect file type
    let file_type = detect_file_type(&discovery);

    // Reset reader to beginning
    reader.seek(SeekFrom::Start(0))?;

    // Stage 2: Parse based on file type
    match file_type {
        WmoFileType::Root => {
            let root = parse_root_file(reader, discovery)
                .map_err(|e| WmoError::InvalidFormat(format!("Failed to parse root: {}", e)))?;
            Ok(ParsedWmo::Root(root))
        }
        WmoFileType::Group => {
            let group = parse_group_file(reader, discovery)
                .map_err(|e| WmoError::InvalidFormat(format!("Failed to parse group: {}", e)))?;
            Ok(ParsedWmo::Group(group))
        }
    }
}

/// Parse a WMO file with full metadata
///
/// Returns both the parsed data and chunk discovery information
pub fn parse_wmo_with_metadata<R: Read + Seek>(reader: &mut R) -> Result<ParseResult, WmoError> {
    // Stage 1: Discover chunks
    let discovery = discover_chunks(reader).map_err(|e| {
        WmoError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to discover chunks: {}", e),
        ))
    })?;

    // Clone discovery for return
    let discovery_clone = discovery.clone();

    // Detect file type
    let file_type = detect_file_type(&discovery);

    // Reset reader to beginning
    reader.seek(SeekFrom::Start(0))?;

    // Stage 2: Parse based on file type
    let wmo = match file_type {
        WmoFileType::Root => {
            let root = parse_root_file(reader, discovery)
                .map_err(|e| WmoError::InvalidFormat(format!("Failed to parse root: {}", e)))?;
            ParsedWmo::Root(root)
        }
        WmoFileType::Group => {
            let group = parse_group_file(reader, discovery)
                .map_err(|e| WmoError::InvalidFormat(format!("Failed to parse group: {}", e)))?;
            ParsedWmo::Group(group)
        }
    };

    Ok(ParseResult {
        wmo,
        discovery: discovery_clone,
    })
}

/// Discover chunks in a WMO file without parsing content
///
/// This performs Stage 1 parsing only, useful for inspecting file structure.
pub fn discover_wmo_chunks<R: Read + Seek>(reader: &mut R) -> Result<ChunkDiscovery, WmoError> {
    discover_chunks(reader).map_err(|e| {
        WmoError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to discover chunks: {}", e),
        ))
    })
}
