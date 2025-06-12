use super::locator::*;
use super::{BlpHeader, BlpVersion};
use log::*;

/// There is a limit on size of JPEG header as some tools might crash.
///
/// Larger values are prone to causing image corruption and crashes in some BLP
/// reader implementations like Warcraft III 1.27b where buffer bounds are
/// not strongly enforced. This limit applies especially when generating a
/// BLP file with JPEG content and without mipmaps as it can prevent dumping
/// the entire full scale image JPEG file into jpegHeaderChunk and using an
/// empty mipmap block. If values larger than 624 are encountered it is
/// recommended that a warning be generated and loading continues as normal
/// using the larger size.
pub const MAX_JPEG_HEADER: usize = 624;

/// JPEG encoded BLP image data
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlpJpeg {
    /// JPEG header that is appended to each mipmap level data
    pub header: Vec<u8>,
    /// Image itself and all mipmaps levels. If there are no mipmaps,
    /// the length of the vector is 1.
    pub images: Vec<Vec<u8>>,
}

impl BlpJpeg {
    /// Concat JPEG header with body and get the required mipmap level.
    pub fn full_jpeg(&self, i: usize) -> Option<Vec<u8>> {
        if i >= self.images.len() {
            None
        } else {
            // Remove those bugged 2 bytes from the end
            let header_size = self.header.len() - 2;
            trace!(
                "Getting JPEG with header size {} and body size {}",
                header_size,
                self.images[i].len()
            );
            let mut buffer = Vec::with_capacity(header_size + self.images[i].len());
            buffer.extend(&self.header[0..header_size]);
            buffer.extend(self.images[i].as_slice());
            Some(buffer)
        }
    }

    /// Predict internal locator to write down mipmaps
    pub fn mipmap_locator(&self, version: BlpVersion) -> MipmapLocator {
        let mut offsets = [0; 16];
        let mut sizes = [0; 16];
        let mut cur_offset = BlpHeader::size(version) + self.header.len() + 4;
        for (i, image) in self.images.iter().take(16).enumerate() {
            offsets[i] = cur_offset as u32;
            sizes[i] = image.len() as u32;
            cur_offset += image.len();
        }

        MipmapLocator::Internal { offsets, sizes }
    }
}
