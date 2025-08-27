pub mod error;
pub mod std_impls;
pub mod types;
pub mod utils;

pub mod prelude {
    pub use crate::types::{
        DataVersion, Read, Seek, VWowChunkR, VWowDataR, VWowHeaderR, VWowReaderForChunk,
        VWowReaderForData, VWowReaderForHeader, VWowWriterForHeader, WowChunkR, WowDataR,
        WowHeaderR, WowHeaderW, WowReaderForChunk, WowReaderForData, WowReaderForHeader, WowVec,
        WowWriterForHeader, Write,
    };
    pub use byteorder::{ReadBytesExt, WriteBytesExt};
}
