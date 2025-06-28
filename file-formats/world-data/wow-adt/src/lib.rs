//! Parser for World of Warcraft ADT (terrain) files
//!
//! This crate provides functionality to parse, validate, convert, and manipulate
//! ADT (A Dungeon Terrain) files from World of Warcraft. ADT files contain
//! terrain data including heightmaps, textures, water, and object placement
//! information for the game's world maps.
//!
//! ## Supported Versions
//!
//! - Vanilla (1.x)
//! - The Burning Crusade (2.x)
//! - Wrath of the Lich King (3.x)
//! - Cataclysm (4.x)
//! - Mists of Pandaria (5.x)
//!
//! ## Features
//!
//! - Parse ADT files from all supported WoW versions
//! - Convert between different ADT versions
//! - Validate ADT file structure and data
//! - Stream large ADT files chunk by chunk
//! - Export terrain data to 3D model formats
//! - Extract heightmaps, normal maps, and texture data
//! - Merge and split ADT files
//! - Parallel processing support (with `parallel` feature)
//!
//! ## Example
//!
//! ```no_run
//! use wow_adt::{Adt, AdtVersion};
//! use std::fs::File;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Parse an ADT file
//! let adt = Adt::from_path("Azeroth_32_48.adt")?;
//!
//! // Check version
//! println!("ADT version: {}", adt.version());
//!
//! // Access terrain chunks
//! println!("Number of chunks: {}", adt.mcnk_chunks().len());
//! # Ok(())
//! # }
//! ```
//!
//! ## References
//!
//! Based on information from:
//! - <https://wowdev.wiki/ADT>
//! - <https://github.com/WowDevTools/libwarcraft>

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

mod adt_builder;
mod chunk;
mod converter;
mod error;
mod io_helpers;
mod liquid_converter;
mod mcnk_converter;
mod mcnk_subchunks;
mod mcnk_writer;
mod merge;
mod mh2o;
mod model_export;
mod normal_map;
mod split_adt;
mod streaming;
mod texture_converter;
mod validator;
mod version;
mod writer;

#[cfg(feature = "parallel")]
mod parallel;

#[cfg(feature = "extract")]
pub mod extract;

// Import advanced water chunk type
use crate::mh2o::Mh2oChunk as AdvancedMh2oChunk;
pub use mh2o::{Mh2oEntry, Mh2oInstance, WaterLevelData, WaterVertex, WaterVertexData};

pub use adt_builder::{AdtBuilder, create_flat_terrain};
pub use chunk::*;
pub use converter::convert_adt;
pub use error::{AdtError, Result};
pub use mcnk_converter::{convert_mcnk, convert_mcnk_chunks};
pub use mcnk_subchunks::*;
pub use merge::{MergeOptions, extract_portion, merge_adts, merge_chunk};
pub use model_export::{ModelExportOptions, ModelFormat, export_to_3d};
pub use normal_map::{
    NormalChannelEncoding, NormalMapFormat, NormalMapOptions, extract_normal_map,
};
pub use streaming::{
    AdtStreamer, StreamedChunk, count_matching_chunks, iterate_mcnk_chunks, open_adt_stream,
};
pub use texture_converter::{convert_alpha_maps, convert_area_id, convert_texture_layers};
pub use validator::{ValidationLevel, ValidationReport, validate_adt};
pub use version::AdtVersion;

#[cfg(feature = "parallel")]
pub use parallel::{ParallelOptions, batch_convert, batch_validate, process_parallel};

/// Main ADT structure that holds all the parsed data for a terrain file
#[derive(Debug, Clone)]
pub struct Adt {
    /// Version of the ADT file
    pub version: AdtVersion,
    /// MVER chunk - file version
    pub mver: MverChunk,
    /// MHDR chunk - header with offsets to other chunks
    pub mhdr: Option<MhdrChunk>,
    /// MCNK chunks - map chunk data (terrain height, texturing, etc.)
    pub mcnk_chunks: Vec<McnkChunk>,
    /// MCIN chunk - map chunk index
    pub mcin: Option<McinChunk>,
    /// MTEX chunk - texture filenames
    pub mtex: Option<MtexChunk>,
    /// MMDX chunk - model filenames
    pub mmdx: Option<MmdxChunk>,
    /// MMID chunk - model indices
    pub mmid: Option<MmidChunk>,
    /// MWMO chunk - WMO filenames
    pub mwmo: Option<MwmoChunk>,
    /// MWID chunk - WMO indices
    pub mwid: Option<MwidChunk>,
    /// MDDF chunk - doodad placement information
    pub mddf: Option<MddfChunk>,
    /// MODF chunk - model placement information
    pub modf: Option<ModfChunk>,

    // Version-specific data
    /// TBC and later - flight boundaries
    pub mfbo: Option<MfboChunk>,
    /// WotLK and later - water data
    pub mh2o: Option<AdvancedMh2oChunk>,
    /// Cataclysm and later - texture effects
    pub mtfx: Option<MtfxChunk>,
}

impl Adt {
    /// Parse an ADT file from a path
    ///
    /// For split files (_tex0, _obj0, etc.), this returns a minimal ADT structure
    /// with only the chunks present in that specific file type.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy();
        let file_type = split_adt::SplitAdtType::from_filename(&path_str);

        match file_type {
            split_adt::SplitAdtType::Obj0 => {
                // Parse obj0 file
                let mut file = File::open(path)?;
                let obj_data = split_adt::SplitAdtParser::parse_obj0(&mut file)?;

                Ok(Adt {
                    version: AdtVersion::Cataclysm,
                    mver: MverChunk { version: 18 },
                    mhdr: Some(MhdrChunk::default()),
                    mcin: None,
                    mtex: None,
                    mmdx: obj_data.mmdx,
                    mmid: obj_data.mmid,
                    mwmo: obj_data.mwmo,
                    mwid: obj_data.mwid,
                    mddf: obj_data.mddf,
                    modf: obj_data.modf,
                    mcnk_chunks: Vec::new(),
                    mfbo: None,
                    mh2o: None,
                    mtfx: None,
                })
            }
            split_adt::SplitAdtType::Tex0 | split_adt::SplitAdtType::Tex1 => {
                // Parse tex file
                let mut file = File::open(path)?;
                let tex_data = split_adt::SplitAdtParser::parse_tex0(&mut file)?;

                Ok(Adt {
                    version: AdtVersion::Cataclysm,
                    mver: MverChunk { version: 18 },
                    mhdr: Some(MhdrChunk::default()),
                    mcin: None,
                    mtex: tex_data.mtex,
                    mmdx: None,
                    mmid: None,
                    mwmo: None,
                    mwid: None,
                    mddf: None,
                    modf: None,
                    mcnk_chunks: Vec::new(),
                    mfbo: None,
                    mh2o: None,
                    mtfx: None,
                })
            }
            split_adt::SplitAdtType::Obj1 | split_adt::SplitAdtType::Lod => {
                // Return minimal ADT for unsupported split file types
                Ok(Adt {
                    version: AdtVersion::Cataclysm,
                    mver: MverChunk { version: 18 },
                    mhdr: Some(MhdrChunk::default()),
                    mcin: None,
                    mtex: None,
                    mmdx: None,
                    mmid: None,
                    mwmo: None,
                    mwid: None,
                    mddf: None,
                    modf: None,
                    mcnk_chunks: Vec::new(),
                    mfbo: None,
                    mh2o: None,
                    mtfx: None,
                })
            }
            split_adt::SplitAdtType::Root => {
                // Parse normal ADT file
                let file = File::open(path)?;
                Self::from_reader(file)
            }
        }
    }

    /// Parse an ADT file from any reader that implements Read + Seek
    pub fn from_reader<R: Read + Seek>(mut reader: R) -> Result<Self> {
        // Get file size for bounds checking
        let file_size = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(0))?;

        // Minimum ADT file size check (at least MVER + MHDR chunks)
        const MIN_ADT_SIZE: u64 = 12 + 8 + 64 + 8; // MVER header + data + MHDR header + data
        if file_size < MIN_ADT_SIZE {
            return Err(AdtError::InvalidFileSize(format!(
                "File too small to be a valid ADT: {file_size} bytes"
            )));
        }

        // First, read the MVER chunk to determine the file version
        let mver = MverChunk::read(&mut reader)?;
        let version = AdtVersion::from_mver(mver.version)?;

        // Seek back to the beginning for full parsing
        reader.seek(SeekFrom::Start(0))?;

        // Create a parser context to track our state during parsing
        let mut context = ParserContext {
            reader: &mut reader,
            version,
            position: 0,
        };

        // Read the full file
        let mut chunks = ChunkMap::new();

        while let Ok(header) = ChunkHeader::read(&mut context.reader) {
            let current_pos = context.reader.stream_position()?;

            match &header.magic {
                b"MVER" => {
                    let chunk = MverChunk::read_with_header(header.clone(), &mut context)?;
                    chunks.mver = Some(chunk);
                }
                b"MHDR" => {
                    let chunk = MhdrChunk::read_with_header(header.clone(), &mut context)?;
                    chunks.mhdr = Some(chunk);
                }
                b"MCIN" => {
                    let chunk = McinChunk::read_with_header(header.clone(), &mut context)?;
                    chunks.mcin = Some(chunk);
                }
                b"MTEX" => {
                    let chunk = MtexChunk::read_with_header(header.clone(), &mut context)?;
                    chunks.mtex = Some(chunk);
                }
                b"MMDX" => {
                    let chunk = MmdxChunk::read_with_header(header.clone(), &mut context)?;
                    chunks.mmdx = Some(chunk);
                }
                b"MMID" => {
                    let chunk = MmidChunk::read_with_header(header.clone(), &mut context)?;
                    chunks.mmid = Some(chunk);
                }
                b"MWMO" => {
                    let chunk = MwmoChunk::read_with_header(header.clone(), &mut context)?;
                    chunks.mwmo = Some(chunk);
                }
                b"MWID" => {
                    let chunk = MwidChunk::read_with_header(header.clone(), &mut context)?;
                    chunks.mwid = Some(chunk);
                }
                b"MDDF" => {
                    let chunk = MddfChunk::read_with_header(header.clone(), &mut context)?;
                    chunks.mddf = Some(chunk);
                }
                b"MODF" => {
                    let chunk = ModfChunk::read_with_header(header.clone(), &mut context)?;
                    chunks.modf = Some(chunk);
                }
                b"MCNK" => {
                    // In Cataclysm+, ADT files may not have MCIN and MCNK chunks appear directly
                    // Store the position and size for later processing
                    let chunk_pos = current_pos - 8; // Subtract header size to get chunk start
                    chunks.mcnk_positions.push((chunk_pos, header.size));
                    // Skip the chunk data for now
                    context.reader.seek(SeekFrom::Current(header.size as i64))?;
                }
                // Version-specific chunks
                b"MFBO" => {
                    // Parse MFBO regardless of initial version - version will be detected later
                    match MfboChunk::read_with_header(header.clone(), &mut context) {
                        Ok(chunk) => {
                            chunks.mfbo = Some(chunk);
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Failed to parse MFBO chunk ({e}), marking as present for version detection"
                            );
                            // For version detection purposes, just mark that we found MFBO
                            // Skip the actual parsing since there are size variations between versions
                            chunks.mfbo = Some(MfboChunk {
                                max: [0, 0],
                                min: [0, 0],
                                additional_data: Vec::new(),
                            });
                            context.reader.seek(SeekFrom::Current(header.size as i64))?;
                        }
                    }
                }
                b"MH2O" => {
                    // MH2O is used for water data in WotLK and later
                    // Get the current position (after reading header)
                    let chunk_data_start = context.reader.stream_position()?;
                    let chunk_start = chunk_data_start - 8; // Subtract header size

                    match AdvancedMh2oChunk::read_full(&mut context, chunk_start, header.size) {
                        Ok(chunk) => {
                            chunks.mh2o = Some(chunk);
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to parse MH2O chunk: {e}");
                            // Skip the chunk data on error
                            context
                                .reader
                                .seek(SeekFrom::Start(chunk_data_start + header.size as u64))?;
                            // Mark as present for version detection
                            chunks.mh2o = Some(AdvancedMh2oChunk { chunks: Vec::new() });
                        }
                    }
                }
                b"MTFX" => {
                    // Parse MTFX regardless of initial version - version will be detected later
                    let chunk = MtfxChunk::read_with_header(header.clone(), &mut context)?;
                    chunks.mtfx = Some(chunk);
                }
                _ => {
                    // Unknown chunk, skip it
                    context.reader.seek(SeekFrom::Current(header.size as i64))?;
                }
            }

            // Update our position
            context.position = current_pos as usize + header.size as usize;
        }

        // Phase 2: Read MCNK chunks using MCIN offsets
        if let Some(ref mcin) = chunks.mcin {
            for (i, entry) in mcin.entries.iter().enumerate() {
                if entry.offset > 0 && entry.size > 0 {
                    // Validate offset is within file bounds
                    if entry.offset as u64 + entry.size as u64 > file_size {
                        eprintln!(
                            "MCNK chunk {} at offset {} exceeds file size {}",
                            i, entry.offset, file_size
                        );
                        continue;
                    }

                    // Seek to the MCNK chunk offset
                    match context.reader.seek(SeekFrom::Start(entry.offset as u64)) {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!(
                                "Error seeking to MCNK chunk {} at offset {}: {}",
                                i, entry.offset, e
                            );
                            continue;
                        }
                    }

                    // Read the MCNK chunk header
                    let header = match ChunkHeader::read(&mut context.reader) {
                        Ok(h) => h,
                        Err(e) => {
                            eprintln!("Error reading MCNK chunk {i} header: {e}");
                            continue;
                        }
                    };

                    // Verify it's actually an MCNK chunk
                    if &header.magic == b"MCNK" {
                        match McnkChunk::read_with_header(header, &mut context) {
                            Ok(chunk) => chunks.mcnk.push(chunk),
                            Err(e) => {
                                eprintln!("Error reading MCNK chunk {i} content: {e}");
                                // Continue with other chunks instead of failing completely
                                continue;
                            }
                        }
                    } else {
                        eprintln!(
                            "Expected MCNK at offset {}, found {:?}",
                            entry.offset,
                            header.magic_as_string()
                        );
                    }
                }
            }
        }

        // Phase 3: If no MCIN but we found direct MCNK chunks, parse them now
        if chunks.mcin.is_none() && !chunks.mcnk_positions.is_empty() {
            for (chunk_pos, _chunk_size) in chunks.mcnk_positions.iter() {
                // Seek to the MCNK chunk
                match context.reader.seek(SeekFrom::Start(*chunk_pos)) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error seeking to direct MCNK chunk at offset {chunk_pos}: {e}");
                        continue;
                    }
                }

                // Read the MCNK chunk header
                let header = match ChunkHeader::read(&mut context.reader) {
                    Ok(h) => h,
                    Err(e) => {
                        eprintln!(
                            "Error reading direct MCNK chunk header at offset {chunk_pos}: {e}"
                        );
                        continue;
                    }
                };

                // Verify it's actually an MCNK chunk
                if &header.magic == b"MCNK" {
                    match McnkChunk::read_with_header(header, &mut context) {
                        Ok(chunk) => {
                            chunks.mcnk.push(chunk);
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Error reading MCNK chunk at offset {chunk_pos}: {e}"
                            );
                            continue;
                        }
                    }
                } else {
                    eprintln!(
                        "Warning: Expected MCNK at offset {}, found {:?}",
                        chunk_pos,
                        header.magic_as_string()
                    );
                }
            }
        }

        // Detect version based on chunk presence
        let has_mcnk_with_mccv = chunks.mcnk.iter().any(|mcnk| mcnk.mccv_offset > 0);
        let detected_version = AdtVersion::detect_from_chunks(
            chunks.mfbo.is_some(),
            chunks.mh2o.is_some(),
            chunks.mtfx.is_some(),
            has_mcnk_with_mccv,
        );

        // Construct the ADT from the parsed chunks
        let adt = Adt {
            version: detected_version,
            mver: chunks.mver.unwrap_or(MverChunk { version: 18 }),
            mhdr: chunks.mhdr,
            mcnk_chunks: chunks.mcnk,
            mcin: chunks.mcin,
            mtex: chunks.mtex,
            mmdx: chunks.mmdx,
            mmid: chunks.mmid,
            mwmo: chunks.mwmo,
            mwid: chunks.mwid,
            mddf: chunks.mddf,
            modf: chunks.modf,
            mfbo: chunks.mfbo,
            mh2o: chunks.mh2o,
            mtfx: chunks.mtfx,
        };

        Ok(adt)
    }

    /// Get the version of this ADT file
    pub fn version(&self) -> AdtVersion {
        self.version
    }

    /// Get the MCNK chunks
    pub fn mcnk_chunks(&self) -> &[McnkChunk] {
        &self.mcnk_chunks
    }

    /// Get the MH2O water chunk (WotLK+)
    pub fn mh2o(&self) -> Option<&AdvancedMh2oChunk> {
        self.mh2o.as_ref()
    }

    /// Convert to a specific version
    pub fn to_version(&self, target_version: AdtVersion) -> Result<Self> {
        if self.version == target_version {
            // No conversion needed
            return Ok(self.clone());
        }

        convert_adt(self, target_version)
    }

    /// Validate the ADT data
    pub fn validate(&self) -> Result<()> {
        validator::validate_adt(self, ValidationLevel::Basic)?;
        Ok(())
    }

    /// Perform comprehensive validation with detailed report
    pub fn validate_with_report(&self, level: ValidationLevel) -> Result<ValidationReport> {
        validator::validate_adt(self, level)
    }

    /// Validate with detailed report and file context
    pub fn validate_with_report_and_context<P: AsRef<Path>>(
        &self,
        level: ValidationLevel,
        file_path: P,
    ) -> Result<ValidationReport> {
        validator::validate_adt_with_context(self, level, Some(file_path))
    }
}

/// Helper structure to collect parsed chunks during reading
#[derive(Default)]
struct ChunkMap {
    mver: Option<MverChunk>,
    mhdr: Option<MhdrChunk>,
    mcin: Option<McinChunk>,
    mtex: Option<MtexChunk>,
    mmdx: Option<MmdxChunk>,
    mmid: Option<MmidChunk>,
    mwmo: Option<MwmoChunk>,
    mwid: Option<MwidChunk>,
    mddf: Option<MddfChunk>,
    modf: Option<ModfChunk>,
    mcnk: Vec<McnkChunk>,
    mcnk_positions: Vec<(u64, u32)>, // (position, size) for direct MCNK chunks
    mfbo: Option<MfboChunk>,
    mh2o: Option<AdvancedMh2oChunk>,
    mtfx: Option<MtfxChunk>,
}

impl ChunkMap {
    fn new() -> Self {
        Self::default()
    }
}

/// Context for parsing chunks
pub(crate) struct ParserContext<'a, R: Read + Seek> {
    pub reader: &'a mut R,
    pub version: AdtVersion,
    pub position: usize,
}
