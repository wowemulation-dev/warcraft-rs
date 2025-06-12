/// Descibes where to search for mipmaps
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MipmapLocator {
    /// Mipmaps are located inside the BLP file with given offsets
    /// and sizes.
    Internal {
        /// Byte offsets to each mipmap level (up to 16)
        offsets: [u32; 16],
        /// Byte sizes of each mipmap level (up to 16)
        sizes: [u32; 16],
    },
    /// Mipmaps are located in external files with
    /// names `<base_name>.b<zero padded number>`. Ex. `.b04`, `.b10`.
    External,
}

impl Default for MipmapLocator {
    fn default() -> Self {
        MipmapLocator::Internal {
            offsets: [0; 16],
            sizes: [0; 16],
        }
    }
}
