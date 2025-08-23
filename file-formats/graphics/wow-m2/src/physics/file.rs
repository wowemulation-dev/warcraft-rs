use wow_data::error::Result as WDResult;
use wow_data::types::{MagicStr, WowStructR};
use wow_data::{prelude::*, read_chunk_items, v_read_chunk_items};
use wow_data_derive::{WowHeaderR, WowHeaderW};

use crate::M2Error;

use super::version::PhysVersion;
use super::{body, shape};

pub const PHYS: MagicStr = *b"SYHP";

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
pub struct ChunkHeader {
    pub magic: MagicStr,
    pub bytes: u32,
}

#[derive(Debug, Clone)]
pub enum Chunk {
    ShapeBox(Vec<shape::ShapeBox>),
    ShapeCapsule(Vec<shape::ShapeCapsule>),
    ShapeSphere(Vec<shape::ShapeSphere>),
    Shape {
        version: shape::Version,
        items: Vec<shape::Shape>,
    },
    Body {
        version: body::Version,
        items: Vec<body::Body>,
    },
    Unkown(Vec<u8>),
}

#[derive(Debug, Clone, Default)]
pub struct PhysFile {
    pub header: ChunkHeader,
    pub version: PhysVersion,
    pub chunks: Vec<Chunk>,
}

impl WowStructR for PhysFile {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        let header: ChunkHeader = reader.wow_read()?;
        if header.magic != PHYS {
            return Err(M2Error::InvalidMagic {
                expected: String::from_utf8_lossy(&PHYS).into(),
                actual: String::from_utf8_lossy(&header.magic).into(),
            }
            .into());
        }

        let version = reader.wow_read()?;

        let mut chunks = Vec::new();

        loop {
            let chunk_header: ChunkHeader = if let Ok(chunk_header) = reader.wow_read() {
                chunk_header
            } else {
                break;
            };

            chunks.push(match chunk_header.magic {
                shape::BOXS => {
                    Chunk::ShapeBox(read_chunk_items!(reader, chunk_header, shape::ShapeBox))
                }
                shape::CAPS => Chunk::ShapeCapsule(read_chunk_items!(
                    reader,
                    chunk_header,
                    shape::ShapeCapsule
                )),
                shape::SPHS => {
                    Chunk::ShapeSphere(read_chunk_items!(reader, chunk_header, shape::ShapeSphere))
                }
                shape::SHAP | shape::SHP2 => {
                    let version: shape::Version = chunk_header.magic.try_into()?;
                    Chunk::Shape {
                        version,
                        items: v_read_chunk_items!(reader, version, chunk_header, shape::Shape),
                    }
                }
                body::BODY | body::BDY2 | body::BDY3 | body::BDY4 => {
                    let version: body::Version = chunk_header.magic.try_into()?;
                    Chunk::Body {
                        version,
                        items: v_read_chunk_items!(reader, version, chunk_header, body::Body),
                    }
                }
                _ => {
                    let mut vec = Vec::with_capacity(chunk_header.bytes as usize);
                    for _ in 0..chunk_header.bytes {
                        vec.push(reader.read_u8()?);
                    }
                    Chunk::Unkown(vec)
                }
            });
        }

        Ok(Self {
            header,
            version,
            chunks,
        })
    }
}
