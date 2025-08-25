use std::collections::HashMap;

use wow_data::error::Result as WDResult;
use wow_data::types::{ChunkHeader, MagicStr, WowStructR};
use wow_data::utils::chunk_magic_to_type;
use wow_data::{prelude::*, read_chunk_items, v_read_chunk_items};

use crate::M2Error;

use super::version::PhysVersion;
use super::{body, joint, phyt, shape};

pub const PHYS: MagicStr = *b"SYHP";

#[derive(Debug, Clone)]
pub enum PhysChunk {
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
    Joint(Vec<joint::Joint>),
    JointDistance(Vec<joint::JointDistance>),
    JointPrismatic {
        version: joint::prismatic::Version,
        items: Vec<joint::JointPrismatic>,
    },
    JointRevolute {
        version: joint::revolute::Version,
        items: Vec<joint::JointRevolute>,
    },
    JointShoulder {
        version: joint::shoulder::Version,
        items: Vec<joint::JointShoulder>,
    },
    JointSpherical(Vec<joint::JointSpherical>),
    JointWeld {
        version: joint::weld::Version,
        items: Vec<joint::JointWeld>,
    },
    Phyt(Vec<phyt::Phyt>),
    Unknown(Vec<u8>),
}

#[derive(Debug, Clone, Default)]
pub struct PhysFile {
    pub header: ChunkHeader,
    pub version: PhysVersion,
    pub chunk_index: HashMap<String, usize>,
    pub chunks: Vec<PhysChunk>,
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
        let mut chunk_index = HashMap::new();

        loop {
            let chunk_header: ChunkHeader = if let Ok(chunk_header) = reader.wow_read() {
                chunk_header
            } else {
                break;
            };

            let (chunk_type, chunk_vec): (String, PhysChunk) = match chunk_header.magic {
                shape::BOXS => (
                    chunk_magic_to_type(&shape::BOXS),
                    PhysChunk::ShapeBox(read_chunk_items!(reader, chunk_header, shape::ShapeBox)),
                ),
                shape::CAPS => (
                    chunk_magic_to_type(&shape::CAPS),
                    PhysChunk::ShapeCapsule(read_chunk_items!(
                        reader,
                        chunk_header,
                        shape::ShapeCapsule
                    )),
                ),
                shape::SPHS => (
                    chunk_magic_to_type(&shape::SPHS),
                    PhysChunk::ShapeSphere(read_chunk_items!(
                        reader,
                        chunk_header,
                        shape::ShapeSphere
                    )),
                ),
                shape::SHAP | shape::SHP2 => {
                    let version: shape::Version = chunk_header.magic.try_into()?;
                    (
                        chunk_magic_to_type(&shape::SHAP),
                        PhysChunk::Shape {
                            version,
                            items: v_read_chunk_items!(reader, version, chunk_header, shape::Shape),
                        },
                    )
                }
                body::BODY | body::BDY2 | body::BDY3 | body::BDY4 => {
                    let version: body::Version = chunk_header.magic.try_into()?;
                    (
                        chunk_magic_to_type(&body::BODY),
                        PhysChunk::Body {
                            version,
                            items: v_read_chunk_items!(reader, version, chunk_header, body::Body),
                        },
                    )
                }
                joint::JOIN => (
                    chunk_magic_to_type(&joint::JOIN),
                    PhysChunk::Joint(read_chunk_items!(reader, chunk_header, joint::Joint)),
                ),
                joint::DSTJ => (
                    chunk_magic_to_type(&joint::DSTJ),
                    PhysChunk::JointDistance(read_chunk_items!(
                        reader,
                        chunk_header,
                        joint::JointDistance
                    )),
                ),
                joint::PRSJ | joint::PRS2 => {
                    let version: joint::prismatic::Version = chunk_header.magic.try_into()?;
                    (
                        chunk_magic_to_type(&joint::PRSJ),
                        PhysChunk::JointPrismatic {
                            version,
                            items: v_read_chunk_items!(
                                reader,
                                version,
                                chunk_header,
                                joint::JointPrismatic
                            ),
                        },
                    )
                }
                joint::REVJ | joint::REV2 => {
                    let version: joint::revolute::Version = chunk_header.magic.try_into()?;
                    (
                        chunk_magic_to_type(&joint::REVJ),
                        PhysChunk::JointRevolute {
                            version,
                            items: v_read_chunk_items!(
                                reader,
                                version,
                                chunk_header,
                                joint::JointRevolute
                            ),
                        },
                    )
                }
                joint::SHOJ | joint::SHJ2 => {
                    let version: joint::shoulder::Version =
                        (version, chunk_header.magic).try_into()?;
                    (
                        chunk_magic_to_type(&joint::SHOJ),
                        PhysChunk::JointShoulder {
                            version,
                            items: v_read_chunk_items!(
                                reader,
                                version,
                                chunk_header,
                                joint::JointShoulder
                            ),
                        },
                    )
                }
                joint::SPHJ => (
                    chunk_magic_to_type(&joint::SPHJ),
                    PhysChunk::JointSpherical(read_chunk_items!(
                        reader,
                        chunk_header,
                        joint::JointSpherical
                    )),
                ),
                joint::WELJ | joint::WLJ2 | joint::WLJ3 => {
                    let version: joint::weld::Version = chunk_header.magic.try_into()?;
                    (
                        chunk_magic_to_type(&joint::WELJ),
                        PhysChunk::JointWeld {
                            version,
                            items: v_read_chunk_items!(
                                reader,
                                version,
                                chunk_header,
                                joint::JointWeld
                            ),
                        },
                    )
                }
                phyt::PHYT => (
                    chunk_magic_to_type(&phyt::PHYT),
                    PhysChunk::Phyt(read_chunk_items!(reader, chunk_header, phyt::Phyt)),
                ),
                _ => {
                    let mut vec = Vec::with_capacity(chunk_header.bytes as usize);
                    for _ in 0..chunk_header.bytes {
                        vec.push(reader.read_u8()?);
                    }
                    (
                        chunk_magic_to_type(&chunk_header.magic),
                        PhysChunk::Unknown(vec),
                    )
                }
            };

            chunks.push(chunk_vec);
            chunk_index.insert(chunk_type, chunks.len() - 1);
        }

        Ok(Self {
            header,
            version,
            chunks,
            chunk_index,
        })
    }
}
