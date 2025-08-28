use std::collections::HashMap;
use std::io::{Cursor, SeekFrom};

use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{ChunkHeader, MagicStr, WowStructR};
use wow_data::utils::magic_to_string;

use crate::chunks::{file_id, misc};
use crate::header::MD20_MAGIC;
use crate::{M2Error, MD20Model};

pub const MD21_MAGIC: MagicStr = *b"MD21";

#[derive(Debug, Clone)]
pub enum M2Chunk {
    AFID(Vec<file_id::AnimationFile>),
    SFID(file_id::SkinFiles),
    BFID(Vec<file_id::FileId>),
    GPID(Vec<file_id::FileId>),
    PFID(Vec<file_id::FileId>),
    RPID(Vec<file_id::FileId>),
    SKID(Vec<file_id::FileId>),
    TXID(Vec<file_id::FileId>),
    TXAC(Vec<misc::TXACData>),
    Unknown(Vec<u8>),
}

#[derive(Debug, Clone, Default)]
pub struct M2Model {
    pub magic: MagicStr,
    pub md20: MD20Model,
    pub chunk_index: HashMap<String, usize>,
    pub chunks: Vec<M2Chunk>,
}

impl WowStructR for M2Model {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        let magic: MagicStr = reader.wow_read()?;

        match magic {
            MD21_MAGIC => {
                let md20_size: u32 = reader.wow_read()?;
                let pos = reader.stream_position()?;

                let read_magic: MagicStr = reader.wow_read()?;
                if read_magic != MD20_MAGIC {
                    return Err(M2Error::InvalidMagic {
                        expected: magic_to_string(&MD20_MAGIC),
                        actual: magic_to_string(&read_magic),
                    }
                    .into());
                }

                let magic_len = MD20_MAGIC.len() as i64;

                let mut md20_data = vec![0_u8; md20_size as usize];
                reader.seek_relative(-magic_len)?;
                reader.read_exact(&mut md20_data)?;

                let mut md20_reader = Cursor::new(md20_data);
                md20_reader.seek_relative(magic_len)?;

                let md20 = MD20Model::wow_read(&mut md20_reader)?;

                // go to start of other chunks
                reader.seek(SeekFrom::Start(pos + md20_size as u64))?;

                let mut chunks = Vec::new();
                let mut chunk_index = HashMap::new();

                loop {
                    let Ok(chunk_header) = ChunkHeader::wow_read(reader) else {
                        break;
                    };

                    let (chunk_magic, chunk_data): (&MagicStr, M2Chunk) = match chunk_header.magic {
                        file_id::AFID => (
                            &chunk_header.magic,
                            M2Chunk::AFID(reader.wow_read_from_chunk(&chunk_header)?),
                        ),
                        file_id::SFID => (
                            &chunk_header.magic,
                            M2Chunk::SFID(file_id::SkinFiles::wow_read_from_chunk(
                                reader,
                                &chunk_header,
                                &md20.header,
                            )?),
                        ),
                        file_id::BFID => (
                            &chunk_header.magic,
                            M2Chunk::BFID(reader.wow_read_from_chunk(&chunk_header)?),
                        ),
                        file_id::GPID => (
                            &chunk_header.magic,
                            M2Chunk::GPID(reader.wow_read_from_chunk(&chunk_header)?),
                        ),
                        file_id::PFID => (
                            &chunk_header.magic,
                            M2Chunk::PFID(reader.wow_read_from_chunk(&chunk_header)?),
                        ),
                        file_id::RPID => (
                            &chunk_header.magic,
                            M2Chunk::RPID(reader.wow_read_from_chunk(&chunk_header)?),
                        ),
                        file_id::SKID => (
                            &chunk_header.magic,
                            M2Chunk::SKID(reader.wow_read_from_chunk(&chunk_header)?),
                        ),
                        file_id::TXID => (
                            &chunk_header.magic,
                            M2Chunk::TXID(reader.wow_read_from_chunk(&chunk_header)?),
                        ),
                        misc::TXAC => (
                            &chunk_header.magic,
                            M2Chunk::TXAC(reader.wow_read_from_chunk(&chunk_header)?),
                        ),
                        _ => (
                            &chunk_header.magic,
                            M2Chunk::Unknown(reader.wow_read_from_chunk(&chunk_header)?),
                        ),
                    };
                    chunks.push(chunk_data);
                    chunk_index.insert(magic_to_string(chunk_magic), chunks.len() - 1);
                }

                Ok(Self {
                    magic,
                    md20,
                    chunk_index,
                    chunks,
                })
            }
            MD20_MAGIC => Ok(Self {
                magic,
                md20: MD20Model::wow_read(reader)?,
                chunk_index: HashMap::new(),
                chunks: Vec::with_capacity(0),
            }),
            _ => {
                Err(M2Error::InvalidMagic {
                    expected: "MD20 or MD21".into(),
                    actual: magic_to_string(&magic),
                }
                .into())
            }
        }
    }
}
