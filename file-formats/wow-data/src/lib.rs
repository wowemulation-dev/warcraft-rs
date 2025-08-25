pub mod error;
pub mod types;
pub mod utils;

pub mod prelude {
    pub use crate::types::{
        DataVersion, Read, Seek, VWowDataR, VWowHeaderR, VWowHeaderReader, VWowReaderForData,
        VWowWriterForHeader, WowDataR, WowHeaderR, WowHeaderW, WowReaderForData,
        WowReaderForHeader, WowVec, WowWriterForHeader, Write,
    };
    pub use byteorder::{ReadBytesExt, WriteBytesExt};
}
