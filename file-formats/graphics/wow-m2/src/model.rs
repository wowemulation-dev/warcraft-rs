use std::collections::HashMap;
use std::io::{Cursor, SeekFrom};

use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{ChunkHeader, MagicStr, WowStructR};
use wow_data::utils::chunk_magic_to_type;

use crate::header::MD20_MAGIC;
use crate::{M2Error, MD20Model};

pub const MD21_MAGIC: MagicStr = *b"MD21";

#[derive(Debug, Clone)]
pub enum M2Chunk {
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

                let md20_magic: MagicStr = reader.wow_read()?;
                if md20_magic != MD20_MAGIC {
                    return Err(M2Error::ParseError(format!(
                        "Expected {:?}, got {:?}",
                        MD20_MAGIC, md20_magic
                    ))
                    .into());
                }

                let magic_len = MD20_MAGIC.len() as i64;

                let mut vec = vec![0_u8; md20_size as usize];
                reader.seek_relative(-magic_len)?;
                reader.read_exact(&mut vec)?;

                let mut md20_reader = Cursor::new(vec);
                md20_reader.seek_relative(magic_len)?;

                let md20 = MD20Model::wow_read(&mut md20_reader)?;

                reader.seek(SeekFrom::Start(pos + md20_size as u64))?;

                let mut chunks = Vec::new();
                let mut chunk_index = HashMap::new();
                loop {
                    let chunk_header: ChunkHeader = if let Ok(chunk_header) = reader.wow_read() {
                        chunk_header
                    } else {
                        break;
                    };

                    let (chunk_type, chunk_vec): (String, M2Chunk) = match chunk_header.magic {
                        _ => {
                            let mut vec = Vec::with_capacity(chunk_header.bytes as usize);
                            for _ in 0..chunk_header.bytes {
                                vec.push(reader.read_u8()?);
                            }
                            (
                                chunk_magic_to_type(&chunk_header.magic),
                                M2Chunk::Unknown(vec),
                            )
                        }
                    };
                    chunks.push(chunk_vec);
                    chunk_index.insert(chunk_type, chunks.len() - 1);
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
                return Err(M2Error::InvalidMagic {
                    expected: format!("MD20 or MD21"),
                    actual: String::from_utf8_lossy(&magic).into(),
                }
                .into());
            }
        }
    }
}
