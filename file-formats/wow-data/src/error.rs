use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WowDataError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Unsupported numeric version: {0}")]
    UnsupportedNumericVersion(u32),

    #[error("Conversion error: cannot convert from version {from} to {to}: {reason}")]
    ConversionError { from: u32, to: u32, reason: String },

    #[error("Generic error: {0}")]
    GenericError(String),
}

pub type Result<T> = std::result::Result<T, WowDataError>;

#[cfg(test)]
mod tests {

    use wow_data_derive::{WowHeaderRV, WowHeaderW};

    use crate::types::WowHeaderReaderV;
    mod wow_data {
        pub use crate::*;
    }

    use super::*;
    use crate::types::*;

    #[derive(Clone, Copy)]
    struct TestV {}
    impl DataVersion for TestV {}

    #[derive(Clone, Copy, WowHeaderRV, WowHeaderW)]
    #[wow_data(version = TestV)]
    struct ItemHeader {
        item: u32,
        omg: WowArray<u32>,
    }

    struct ItemData {
        header: ItemHeader,
        omg: Vec<u32>,
    }

    impl ItemData {
        fn read<R: Read + Seek>(reader: &mut R, version: TestV) -> Self {
            let header = reader.wow_read_versioned(version).unwrap();

            Self {
                header,
                omg: header.omg.wow_read_to_vec(reader).unwrap(),
            }
        }
    }

    #[derive(Clone, WowHeaderRV, WowHeaderW)]
    #[wow_data(version = TestV)]
    struct TestHeader {
        #[wow_data(versioned)]
        test: WowArrayV<TestV, ItemHeader>,
    }

    struct Test {
        header: TestHeader,
        test: Vec<ItemData>,
    }

    impl Test {
        fn read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
            let version = TestV {};
            let header: TestHeader = reader.wow_read_versioned(version).unwrap();

            let item_headers = header.test.wow_read_to_vec(reader, version).unwrap();
            let items: Vec<_> = item_headers
                .iter()
                .map(|header| ItemData {
                    omg: header.omg.wow_read_to_vec(reader).unwrap(),
                    header: *header,
                })
                .collect();

            // let mut iter = header.test.new_iterator(reader, version).unwrap();
            // let mut items = Vec::new();
            // loop {
            //     match iter.next(|reader, item_header| {
            //         let item_header = match item_header {
            //             Some(item) => item,
            //             None => reader.wow_read_versioned(version)?,
            //         };
            //         let omg = item_header.omg.wow_read_to_vec(reader)?;
            //         items.push(ItemData {
            //             header: item_header,
            //             omg,
            //         });
            //         Ok(())
            //     }) {
            //         Ok(is_active) => {
            //             if !is_active {
            //                 break;
            //             }
            //         }
            //         Err(err) => return Err(err),
            //     }
            // }
            Ok(Self {
                header,
                test: items,
            })
        }
    }
}
