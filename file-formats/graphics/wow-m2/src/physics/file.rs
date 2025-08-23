use wow_data::error::Result as WDResult;
use wow_data::types::{ChunkHeader, MagicStr, WowStructR};
use wow_data::{prelude::*, read_chunk_items, v_read_chunk_items};

use crate::M2Error;

use super::version::PhysVersion;
use super::{body, joint, shape};

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
    Unkown(Vec<u8>),
}

#[derive(Debug, Clone, Default)]
pub struct PhysFile {
    pub header: ChunkHeader,
    pub version: PhysVersion,
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

        loop {
            let chunk_header: ChunkHeader = if let Ok(chunk_header) = reader.wow_read() {
                chunk_header
            } else {
                break;
            };

            chunks.push(match chunk_header.magic {
                shape::BOXS => {
                    PhysChunk::ShapeBox(read_chunk_items!(reader, chunk_header, shape::ShapeBox))
                }
                shape::CAPS => PhysChunk::ShapeCapsule(read_chunk_items!(
                    reader,
                    chunk_header,
                    shape::ShapeCapsule
                )),
                shape::SPHS => PhysChunk::ShapeSphere(read_chunk_items!(
                    reader,
                    chunk_header,
                    shape::ShapeSphere
                )),
                shape::SHAP | shape::SHP2 => {
                    let version: shape::Version = chunk_header.magic.try_into()?;
                    PhysChunk::Shape {
                        version,
                        items: v_read_chunk_items!(reader, version, chunk_header, shape::Shape),
                    }
                }
                body::BODY | body::BDY2 | body::BDY3 | body::BDY4 => {
                    let version: body::Version = chunk_header.magic.try_into()?;
                    PhysChunk::Body {
                        version,
                        items: v_read_chunk_items!(reader, version, chunk_header, body::Body),
                    }
                }
                joint::JOIN => {
                    PhysChunk::Joint(read_chunk_items!(reader, chunk_header, joint::Joint))
                }
                joint::DSTJ => PhysChunk::JointDistance(read_chunk_items!(
                    reader,
                    chunk_header,
                    joint::JointDistance
                )),
                joint::PRSJ | joint::PRS2 => {
                    let version: joint::prismatic::Version = chunk_header.magic.try_into()?;
                    PhysChunk::JointPrismatic {
                        version,
                        items: v_read_chunk_items!(
                            reader,
                            version,
                            chunk_header,
                            joint::JointPrismatic
                        ),
                    }
                }
                joint::REVJ | joint::REV2 => {
                    let version: joint::revolute::Version = chunk_header.magic.try_into()?;
                    PhysChunk::JointRevolute {
                        version,
                        items: v_read_chunk_items!(
                            reader,
                            version,
                            chunk_header,
                            joint::JointRevolute
                        ),
                    }
                }
                joint::SHOJ | joint::SHJ2 => {
                    let version: joint::shoulder::Version = chunk_header.magic.try_into()?;
                    PhysChunk::JointShoulder {
                        version,
                        items: v_read_chunk_items!(
                            reader,
                            version,
                            chunk_header,
                            joint::JointShoulder
                        ),
                    }
                }
                joint::SPHJ => PhysChunk::JointSpherical(read_chunk_items!(
                    reader,
                    chunk_header,
                    joint::JointSpherical
                )),
                joint::WELJ | joint::WLJ2 | joint::WLJ3 => {
                    let version: joint::weld::Version = chunk_header.magic.try_into()?;
                    PhysChunk::JointWeld {
                        version,
                        items: v_read_chunk_items!(reader, version, chunk_header, joint::JointWeld),
                    }
                }
                _ => {
                    let mut vec = Vec::with_capacity(chunk_header.bytes as usize);
                    for _ in 0..chunk_header.bytes {
                        vec.push(reader.read_u8()?);
                    }
                    PhysChunk::Unkown(vec)
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
