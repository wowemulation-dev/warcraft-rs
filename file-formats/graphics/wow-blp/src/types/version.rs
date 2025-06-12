use std::fmt;
use std::str;

/// Version of type format that determines structure of file. Encodes
/// magic bytes that are expected at start of the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BlpVersion {
    /// BLP0 format - Used in Warcraft III ROC Beta builds
    Blp0,
    /// BLP1 format - Common in Warcraft III TFT
    Blp1,
    /// BLP2 format - Used in World of Warcraft
    Blp2,
}

impl BlpVersion {
    /// Convert to 4 ASCII symbols that are put into beginning of file
    /// to mark that the file has specific version of the format.
    pub fn to_magic(&self) -> [u8; 4] {
        let mut magic: [u8; 4] = Default::default();
        match self {
            BlpVersion::Blp0 => magic.copy_from_slice("BLP0".as_bytes()),
            BlpVersion::Blp1 => magic.copy_from_slice("BLP1".as_bytes()),
            BlpVersion::Blp2 => magic.copy_from_slice("BLP2".as_bytes()),
        }
        magic
    }

    /// Convert from 4 ASCII symbols from the start of file to known
    /// tag of version.
    pub fn from_magic(magic: [u8; 4]) -> Option<BlpVersion> {
        // Use that ascii is valid subset of utf8
        let magic_str = str::from_utf8(&magic).ok()?;

        match magic_str {
            "BLP0" => Some(BlpVersion::Blp0),
            "BLP1" => Some(BlpVersion::Blp1),
            "BLP2" => Some(BlpVersion::Blp2),
            _ => None,
        }
    }
}

impl fmt::Display for BlpVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlpVersion::Blp0 => write!(f, "BLP0"),
            BlpVersion::Blp1 => write!(f, "BLP1"),
            BlpVersion::Blp2 => write!(f, "BLP2"),
        }
    }
}

/// Error type for unknown BLP version strings
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnknownBlpVersion(String);

impl fmt::Display for UnknownBlpVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unknown BLP version: {}", self.0)
    }
}

impl std::str::FromStr for BlpVersion {
    type Err = UnknownBlpVersion;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_ref() {
            "blp0" => Ok(BlpVersion::Blp0),
            "blp1" => Ok(BlpVersion::Blp1),
            "blp2" => Ok(BlpVersion::Blp2),
            _ => Err(UnknownBlpVersion(s.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_magics() {
        let magic1 = [0x42, 0x4c, 0x50, 0x31];
        let version1 = BlpVersion::from_magic(magic1);
        assert_eq!(version1, Some(BlpVersion::Blp1));
        assert_eq!(version1.unwrap().to_magic(), magic1);

        let magic2 = [0x42, 0x4c, 0x50, 0x32];
        let version2 = BlpVersion::from_magic(magic2);
        assert_eq!(version2, Some(BlpVersion::Blp2));
        assert_eq!(version2.unwrap().to_magic(), magic2);

        let magic3 = [0x42, 0x4c, 0x50, 0x30];
        let version3 = BlpVersion::from_magic(magic3);
        assert_eq!(version3, Some(BlpVersion::Blp0));
        assert_eq!(version3.unwrap().to_magic(), magic3);
    }
}
