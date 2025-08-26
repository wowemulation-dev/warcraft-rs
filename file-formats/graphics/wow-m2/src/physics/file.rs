use std::collections::HashMap;

use wow_data::error::Result as WDResult;
use wow_data::types::{ChunkHeader, MagicStr, VersionedChunk, WowStructR};
use wow_data::utils::{magic_to_inverted_string, string_to_inverted_magic};
use wow_data::{prelude::*, v_read_chunk_items};

use crate::M2Error;

use super::version::PhysVersion;
use super::{body, joint, phyt, shape};

pub const PHYS: MagicStr = string_to_inverted_magic("PHYS");

#[derive(Debug, Clone)]
pub enum PhysChunk {
    ShapeBox(Vec<shape::ShapeBox>),
    ShapeCapsule(Vec<shape::ShapeCapsule>),
    ShapeSphere(Vec<shape::ShapeSphere>),
    Shape2 {
        version: shape::Version,
        items: Vec<shape::Shape>,
    },
    Shape(VersionedChunk<shape::Version, shape::Shape>),
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
                expected: magic_to_inverted_string(&PHYS),
                actual: magic_to_inverted_string(&header.magic),
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

            let (chunk_magic, chunk_vec): (&MagicStr, PhysChunk) = match chunk_header.magic {
                shape::BOXS => (
                    &shape::BOXS,
                    PhysChunk::ShapeBox(reader.wow_read_from_chunk(&chunk_header)?),
                ),
                shape::CAPS => (
                    &shape::CAPS,
                    PhysChunk::ShapeCapsule(reader.wow_read_from_chunk(&chunk_header)?),
                ),
                shape::SPHS => (
                    &shape::SPHS,
                    PhysChunk::ShapeSphere(reader.wow_read_from_chunk(&chunk_header)?),
                ),
                shape::SHAP | shape::SHP2 => {
                    let version: shape::Version = chunk_header.magic.try_into()?;
                    (
                        &shape::SHAP,
                        PhysChunk::Shape(reader.v_wow_read_from_chunk(version, &chunk_header)?),
                    )
                }
                body::BODY | body::BDY2 | body::BDY3 | body::BDY4 => {
                    let version: body::Version = chunk_header.magic.try_into()?;
                    (
                        &body::BODY,
                        PhysChunk::Body {
                            version,
                            items: v_read_chunk_items!(reader, version, chunk_header, body::Body),
                        },
                    )
                }
                joint::JOIN => (
                    &joint::JOIN,
                    PhysChunk::Joint(reader.wow_read_from_chunk(&chunk_header)?),
                ),
                joint::DSTJ => (
                    &joint::DSTJ,
                    PhysChunk::JointDistance(reader.wow_read_from_chunk(&chunk_header)?),
                ),
                joint::PRSJ | joint::PRS2 => {
                    let version: joint::prismatic::Version = chunk_header.magic.try_into()?;
                    (
                        &joint::PRSJ,
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
                        &joint::REVJ,
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
                        &joint::SHOJ,
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
                    &joint::SPHJ,
                    PhysChunk::JointSpherical(reader.wow_read_from_chunk(&chunk_header)?),
                ),
                joint::WELJ | joint::WLJ2 | joint::WLJ3 => {
                    let version: joint::weld::Version = chunk_header.magic.try_into()?;
                    (
                        &joint::WELJ,
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
                    &phyt::PHYT,
                    PhysChunk::Phyt(reader.wow_read_from_chunk(&chunk_header)?),
                ),
                _ => (
                    &chunk_header.magic,
                    PhysChunk::Unknown(reader.wow_read_from_chunk(&chunk_header)?),
                ),
            };

            chunks.push(chunk_vec);
            chunk_index.insert(magic_to_inverted_string(chunk_magic), chunks.len() - 1);
        }

        Ok(Self {
            header,
            version,
            chunks,
            chunk_index,
        })
    }
}
