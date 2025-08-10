pub mod error;
pub mod io_ext;
pub mod types;

pub mod prelude {
    pub use crate::io_ext::*;
    pub use crate::types::{
        DataVersion, Read, Seek, VWowDataR, VWowHeaderR, VWowHeaderReader, VWowReaderForData,
        VWowWriterForHeader, WowDataR, WowHeaderR, WowHeaderW, WowReaderForData,
        WowReaderForHeader, WowVec, WowWriterForHeader, Write,
    };
    pub use crate::{vwow_collection, wow_collection};
}
