use std::collections::HashMap;

use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{ChunkHeader, MagicStr, VersionedChunk, WowStructR};
use wow_data::utils::{magic_to_inverted_string, string_to_inverted_magic};

use crate::M2Error;

use super::version::PhysVersion;
use super::{body, joint, phyt, shape};

pub const PHYS: MagicStr = string_to_inverted_magic("PHYS");

#[derive(Debug, Clone)]
pub enum PhysChunk {
    ShapeBox(Vec<shape::ShapeBox>),
    ShapeCapsule(Vec<shape::ShapeCapsule>),
    ShapeSphere(Vec<shape::ShapeSphere>),
    Shape(VersionedChunk<shape::Version, shape::Shape>),
    Body(VersionedChunk<body::Version, body::Body>),
    Joint(Vec<joint::Joint>),
    JointDistance(Vec<joint::JointDistance>),
    JointPrismatic(VersionedChunk<joint::prismatic::Version, joint::JointPrismatic>),
    JointRevolute(VersionedChunk<joint::revolute::Version, joint::JointRevolute>),
    JointShoulder(VersionedChunk<joint::shoulder::Version, joint::JointShoulder>),
    JointSpherical(Vec<joint::JointSpherical>),
    JointWeld(VersionedChunk<joint::weld::Version, joint::JointWeld>),
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
                shape::SHAP | shape::SHP2 => (
                    &shape::SHAP,
                    PhysChunk::Shape(reader.v_wow_read_from_chunk(&chunk_header)?),
                ),
                body::BODY | body::BDY2 | body::BDY3 | body::BDY4 => (
                    &body::BODY,
                    PhysChunk::Body(reader.v_wow_read_from_chunk(&chunk_header)?),
                ),
                joint::JOIN => (
                    &joint::JOIN,
                    PhysChunk::Joint(reader.wow_read_from_chunk(&chunk_header)?),
                ),
                joint::DSTJ => (
                    &joint::DSTJ,
                    PhysChunk::JointDistance(reader.wow_read_from_chunk(&chunk_header)?),
                ),
                joint::PRSJ | joint::PRS2 => (
                    &joint::PRSJ,
                    PhysChunk::JointPrismatic(reader.v_wow_read_from_chunk(&chunk_header)?),
                ),
                joint::REVJ | joint::REV2 => (
                    &joint::REVJ,
                    PhysChunk::JointRevolute(reader.v_wow_read_from_chunk(&chunk_header)?),
                ),
                joint::SHOJ | joint::SHJ2 => (
                    &joint::SHOJ,
                    PhysChunk::JointShoulder(joint::JointShoulder::wow_read_from_chunk(
                        reader,
                        version,
                        &chunk_header,
                    )?),
                ),
                joint::SPHJ => (
                    &joint::SPHJ,
                    PhysChunk::JointSpherical(reader.wow_read_from_chunk(&chunk_header)?),
                ),
                joint::WELJ | joint::WLJ2 | joint::WLJ3 => (
                    &joint::WELJ,
                    PhysChunk::JointWeld(reader.v_wow_read_from_chunk(&chunk_header)?),
                ),
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
