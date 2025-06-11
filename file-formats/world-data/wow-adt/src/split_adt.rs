// split_adt.rs - Support for split ADT files introduced in Cataclysm

use crate::Adt;
use crate::chunk::*;
use crate::error::Result;
use crate::version::AdtVersion;
use std::io::{Read, Seek, SeekFrom};

/// Represents a split ADT file type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitAdtType {
    /// Main terrain file (no suffix)
    Root,
    /// Texture data (_tex0.adt)
    Tex0,
    /// Additional texture data (_tex1.adt)
    Tex1,
    /// Object placement data (_obj0.adt)
    Obj0,
    /// Additional object data (_obj1.adt)
    Obj1,
    /// Level of detail data (_lod.adt)
    Lod,
}

impl SplitAdtType {
    /// Detect the file type from the filename
    pub fn from_filename(filename: &str) -> Self {
        if filename.contains("_tex0") {
            SplitAdtType::Tex0
        } else if filename.contains("_tex1") {
            SplitAdtType::Tex1
        } else if filename.contains("_obj0") {
            SplitAdtType::Obj0
        } else if filename.contains("_obj1") {
            SplitAdtType::Obj1
        } else if filename.contains("_lod") {
            SplitAdtType::Lod
        } else {
            SplitAdtType::Root
        }
    }
}

/// Parser for split ADT files
pub struct SplitAdtParser;

impl SplitAdtParser {
    /// Parse a tex0 file
    pub fn parse_tex0<R: Read + Seek>(_reader: &mut R) -> Result<TexAdtData> {
        // tex0 files contain MTEX chunk and texture-related MCNK subchunks
        let mtex = None;
        let mcnk_tex_data = Vec::new();

        // TODO: Implement tex0 parsing
        // For now, return empty data
        Ok(TexAdtData {
            mtex,
            mcnk_tex_data,
        })
    }

    /// Parse an obj0 file
    pub fn parse_obj0<R: Read + Seek>(reader: &mut R) -> Result<ObjAdtData> {
        // obj0 files contain object placement data (MDDF, MODF chunks)
        let mut mmdx = None;
        let mut mmid = None;
        let mut mwmo = None;
        let mut mwid = None;
        let mut mddf = None;
        let mut modf = None;

        // Get file size
        let file_size = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(0))?;

        // Skip MVER chunk
        let _mver = MverChunk::read(reader)?;

        // Read chunks
        while let Ok(header) = ChunkHeader::read(reader) {
            let current_pos = reader.stream_position()?;

            // Check bounds
            if current_pos + header.size as u64 > file_size {
                break;
            }

            match &header.magic {
                b"MMDX" => {
                    mmdx = Some(MmdxChunk::read_with_header(
                        header,
                        &mut crate::ParserContext {
                            reader,
                            version: AdtVersion::Cataclysm,
                            position: current_pos as usize,
                        },
                    )?);
                }
                b"MMID" => {
                    mmid = Some(MmidChunk::read_with_header(
                        header,
                        &mut crate::ParserContext {
                            reader,
                            version: AdtVersion::Cataclysm,
                            position: current_pos as usize,
                        },
                    )?);
                }
                b"MWMO" => {
                    mwmo = Some(MwmoChunk::read_with_header(
                        header,
                        &mut crate::ParserContext {
                            reader,
                            version: AdtVersion::Cataclysm,
                            position: current_pos as usize,
                        },
                    )?);
                }
                b"MWID" => {
                    mwid = Some(MwidChunk::read_with_header(
                        header,
                        &mut crate::ParserContext {
                            reader,
                            version: AdtVersion::Cataclysm,
                            position: current_pos as usize,
                        },
                    )?);
                }
                b"MDDF" => {
                    mddf = Some(MddfChunk::read_with_header(
                        header,
                        &mut crate::ParserContext {
                            reader,
                            version: AdtVersion::Cataclysm,
                            position: current_pos as usize,
                        },
                    )?);
                }
                b"MODF" => {
                    modf = Some(ModfChunk::read_with_header(
                        header,
                        &mut crate::ParserContext {
                            reader,
                            version: AdtVersion::Cataclysm,
                            position: current_pos as usize,
                        },
                    )?);
                }
                _ => {
                    // Skip unknown chunk
                    reader.seek(SeekFrom::Current(header.size as i64))?;
                }
            }
        }

        Ok(ObjAdtData {
            mmdx,
            mmid,
            mwmo,
            mwid,
            mddf,
            modf,
        })
    }
}

/// Texture data from split ADT files
#[derive(Debug)]
#[allow(dead_code)]
pub struct TexAdtData {
    pub mtex: Option<MtexChunk>,
    #[allow(dead_code)]
    pub mcnk_tex_data: Vec<McnkTexData>,
}

/// MCNK texture data from tex files
#[derive(Debug)]
#[allow(dead_code)]
pub struct McnkTexData {
    #[allow(dead_code)]
    pub index: usize,
    #[allow(dead_code)]
    pub mcly: Option<crate::mcnk_subchunks::MclySubchunk>,
    #[allow(dead_code)]
    pub mcal: Option<crate::mcnk_subchunks::McalSubchunk>,
}

/// Object data from split ADT files
#[derive(Debug)]
pub struct ObjAdtData {
    pub mmdx: Option<MmdxChunk>,
    pub mmid: Option<MmidChunk>,
    pub mwmo: Option<MwmoChunk>,
    pub mwid: Option<MwidChunk>,
    pub mddf: Option<MddfChunk>,
    pub modf: Option<ModfChunk>,
}

/// Merge split ADT data into a complete ADT
#[allow(dead_code)]
pub fn merge_split_adt(
    root: Adt,
    tex0: Option<TexAdtData>,
    _tex1: Option<TexAdtData>,
    obj0: Option<ObjAdtData>,
    _obj1: Option<ObjAdtData>,
) -> Adt {
    let mut merged = root;

    // Merge texture data
    if let Some(tex_data) = tex0 {
        if tex_data.mtex.is_some() {
            merged.mtex = tex_data.mtex;
        }
        // TODO: Merge MCNK texture subchunks
    }

    // Merge object data
    if let Some(obj_data) = obj0 {
        merged.mmdx = obj_data.mmdx.or(merged.mmdx);
        merged.mmid = obj_data.mmid.or(merged.mmid);
        merged.mwmo = obj_data.mwmo.or(merged.mwmo);
        merged.mwid = obj_data.mwid.or(merged.mwid);
        merged.mddf = obj_data.mddf.or(merged.mddf);
        merged.modf = obj_data.modf.or(merged.modf);
    }

    merged
}
