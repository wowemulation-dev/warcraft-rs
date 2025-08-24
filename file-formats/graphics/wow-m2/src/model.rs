use wow_data::error::Result as WDResult;
use wow_data::prelude::*;
use wow_data::types::{MagicStr, WowStructR};

use crate::header::MD20_MAGIC;
use crate::{M2Error, MD20Model};

pub const MD21_MAGIC: [u8; 4] = *b"MD21";

#[derive(Debug, Clone)]
pub enum MD21Chunk {
    Unknown { magic: String, data: Vec<u8> },
}

#[derive(Debug, Clone, Default)]
pub struct M2Model {
    pub magic: MagicStr,
    pub bytes: usize,
    pub md20: MD20Model,
    pub chunks: Vec<MD21Chunk>,
}

impl WowStructR for M2Model {
    fn wow_read<R: Read + Seek>(reader: &mut R) -> WDResult<Self> {
        let magic: [u8; 4] = reader.wow_read()?;

        match magic {
            MD21_MAGIC => {
                unreachable!()
            }
            MD20_MAGIC => Ok(Self {
                magic,
                bytes: 0,
                md20: MD20Model::wow_read(reader)?,
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
