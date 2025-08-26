use wow_data::types::{ChunkHeader, MagicStr};
use wow_data::{error::Result as WDResult, prelude::*};
use wow_data_derive::{WowHeaderR, WowHeaderW};

use crate::{
    M2Error,
    header::{M2SkinProfilesHeader, MD20Header},
};

pub const AFID: MagicStr = *b"AFID";
pub const SFID: MagicStr = *b"SFID";

pub const BFID: MagicStr = *b"BFID";
pub const GPID: MagicStr = *b"GPID";
pub const PFID: MagicStr = *b"PFID";
pub const RPID: MagicStr = *b"RPID";
pub const SKID: MagicStr = *b"SKID";
pub const TXID: MagicStr = *b"TXID";

pub type FileId = u32;

#[derive(Debug, Clone, Default, WowHeaderR, WowHeaderW)]
pub struct AnimationFile {
    pub anim_id: u16,
    pub sub_anim_id: u16,
    pub file_id: u32,
}

#[derive(Debug, Clone, Default)]
pub struct SkinFiles {
    pub file_ids: Vec<u32>,
    pub lod_file_ids: Vec<u32>,
}

impl SkinFiles {
    pub fn wow_read_from_chunk<R: Read + Seek>(
        reader: &mut R,
        chunk_header: &ChunkHeader,
        md20_header: &MD20Header,
    ) -> WDResult<Self> {
        let M2SkinProfilesHeader::Later(nviews) = md20_header.skin_profiles else {
            return Err(
                M2Error::ParseError("Unexpected skin profiles in md20 header".into()).into(),
            );
        };

        let first: u32 = reader.wow_read()?;
        let item_size = first.wow_size();
        let items = chunk_header.bytes as usize / item_size;

        let rest = chunk_header.bytes as usize % item_size;
        if rest > 0 {
            dbg!(format!(
                "chunk items size mismatch: chunk={} item_size={}, items={}, rest={}",
                String::from_utf8_lossy(&chunk_header.magic),
                item_size,
                items,
                rest
            ));
        }

        let mut file_ids = Vec::with_capacity(nviews as usize);
        file_ids.push(first);

        for _ in 1..nviews {
            file_ids.push(reader.wow_read()?);
        }

        let lod_file_ids_count = items.saturating_sub(nviews as usize);
        let mut lod_file_ids = Vec::with_capacity(lod_file_ids_count);
        for _ in 0..lod_file_ids_count {
            lod_file_ids.push(reader.wow_read()?);
        }

        reader.seek_relative(rest as i64)?;

        Ok(Self {
            file_ids,
            lod_file_ids,
        })
    }
}
