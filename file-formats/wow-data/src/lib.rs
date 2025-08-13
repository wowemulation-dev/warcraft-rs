pub mod error;
pub mod types;

pub mod prelude {
    pub use crate::types::{
        DataVersion, Read, Seek, VWowDataR, VWowHeaderR, VWowHeaderReader, VWowReaderForData,
        VWowWriterForHeader, WowDataR, WowHeaderR, WowHeaderW, WowReaderForData,
        WowReaderForHeader, WowVec, WowWriterForHeader, Write,
    };
    pub use crate::{v_wow_collection, wow_collection};
    pub use byteorder::{ReadBytesExt, WriteBytesExt};
}
