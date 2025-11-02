//! Split ADT file set discovery and management.
//!
//! Starting with Cataclysm (4.3.4), ADT files were split into multiple specialized files
//! that must be loaded together as a logical unit. This module provides types and utilities
//! for discovering, managing, and validating complete sets of split ADT files.
//!
//! ## File Naming Convention
//!
//! ```text
//! <InternalMapName>_<BlockX>_<BlockY>.adt        # Root file
//! <InternalMapName>_<BlockX>_<BlockY>_tex0.adt   # Texture file (primary)
//! <InternalMapName>_<BlockX>_<BlockY>_tex1.adt   # Texture file (deprecated BfA+)
//! <InternalMapName>_<BlockX>_<BlockY>_obj0.adt   # Object file (primary)
//! <InternalMapName>_<BlockX>_<BlockY>_obj1.adt   # Object file (secondary)
//! <InternalMapName>_<BlockX>_<BlockY>_lod.adt    # LOD file (Legion+)
//! ```
//!
//! ## Examples
//!
//! ```
//! use std::path::Path;
//! use wow_adt::split_set::SplitFileSet;
//!
//! # fn example() -> std::io::Result<()> {
//! // Discover split files from root path
//! let root_path = Path::new("World/Maps/Azeroth/Azeroth_30_30.adt");
//! let file_set = SplitFileSet::discover(root_path);
//!
//! // Check which files exist
//! let presence = file_set.verify_existence();
//! if presence.has_root && presence.has_tex0 && presence.has_obj0 {
//!     println!("Complete Cataclysm ADT set found!");
//! }
//!
//! // Get all present file paths
//! for path in file_set.present_files() {
//!     println!("Found: {}", path.display());
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## References
//!
//! - [wowdev.wiki ADT/v18](https://wowdev.wiki/ADT/v18) - Split file specification
//! - `SPLIT_FILE_ARCHITECTURE.md` - Comprehensive architecture documentation

use std::path::{Path, PathBuf};

/// Complete set of split ADT files for a single map tile.
///
/// Represents all the files that make up a Cataclysm+ ADT tile. Not all files
/// may be present - use [`verify_existence`](SplitFileSet::verify_existence) to check which files actually exist.
///
/// ## File Responsibilities
///
/// - **root**: Terrain geometry (heightmaps, normals, water)
/// - **tex0**: Primary texture data (texture list, layers, alpha maps)
/// - **tex1**: Additional texture data (deprecated in BfA+)
/// - **obj0**: Primary object placements (M2 models, WMO buildings)
/// - **obj1**: Additional object placements
/// - **lod**: Low-detail geometry for distant rendering (Legion+)
///
/// ## Discovery
///
/// Use [`SplitFileSet::discover`] to automatically derive all split file paths from a root ADT path:
///
/// ```
/// use std::path::{Path, PathBuf};
/// use wow_adt::split_set::SplitFileSet;
///
/// let root = Path::new("Azeroth_30_30.adt");
/// let set = SplitFileSet::discover(root);
///
/// assert_eq!(
///     set.tex0,
///     Some(PathBuf::from("Azeroth_30_30_tex0.adt"))
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitFileSet {
    /// Root ADT file path (terrain geometry).
    ///
    /// This is the base file - always present for all WoW versions.
    pub root: PathBuf,

    /// Primary texture file path (_tex0.adt).
    ///
    /// Contains MTEX, MCLY, MCAL chunks. Expected for Cataclysm+ split files.
    pub tex0: Option<PathBuf>,

    /// Secondary texture file path (_tex1.adt).
    ///
    /// Contains additional texture data. Deprecated in Battle for Azeroth (8.x+).
    /// May be `None` even for Cataclysm-Legion if not used.
    pub tex1: Option<PathBuf>,

    /// Primary object file path (_obj0.adt).
    ///
    /// Contains MMDX, MWMO, MDDF, MODF chunks. Expected for Cataclysm+ split files.
    pub obj0: Option<PathBuf>,

    /// Secondary object file path (_obj1.adt).
    ///
    /// Contains additional object placements. May be `None` if tile has few objects.
    pub obj1: Option<PathBuf>,

    /// LOD file path (_lod.adt).
    ///
    /// Contains low-detail geometry for distant rendering. Legion (7.x+) only.
    /// May be `None` for Cataclysm-WoD tiles.
    pub lod: Option<PathBuf>,
}

impl SplitFileSet {
    /// Discover split file paths from a root ADT path.
    ///
    /// Given a root ADT file path (ending in `.adt`), this method derives the expected
    /// paths for all associated split files by replacing the extension with the appropriate
    /// suffix.
    ///
    /// # Arguments
    ///
    /// * `root_path` - Path to the root ADT file (e.g., "Azeroth_30_30.adt")
    ///
    /// # Returns
    ///
    /// A `SplitFileSet` with all potential file paths. Use [`verify_existence`](SplitFileSet::verify_existence)
    /// to check which files actually exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::{Path, PathBuf};
    /// use wow_adt::split_set::SplitFileSet;
    ///
    /// let root = Path::new("World/Maps/Azeroth/Azeroth_30_30.adt");
    /// let set = SplitFileSet::discover(root);
    ///
    /// assert_eq!(set.root, PathBuf::from("World/Maps/Azeroth/Azeroth_30_30.adt"));
    /// assert_eq!(
    ///     set.tex0,
    ///     Some(PathBuf::from("World/Maps/Azeroth/Azeroth_30_30_tex0.adt"))
    /// );
    /// assert_eq!(
    ///     set.obj0,
    ///     Some(PathBuf::from("World/Maps/Azeroth/Azeroth_30_30_obj0.adt"))
    /// );
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the root path has no parent directory or no file stem.
    pub fn discover(root_path: impl AsRef<Path>) -> Self {
        let root = root_path.as_ref().to_path_buf();
        let parent = root.parent().expect("Root path must have parent directory");
        let stem = root
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("Root path must have file stem");

        Self {
            root: root.clone(),
            tex0: Some(parent.join(format!("{}_tex0.adt", stem))),
            tex1: Some(parent.join(format!("{}_tex1.adt", stem))),
            obj0: Some(parent.join(format!("{}_obj0.adt", stem))),
            obj1: Some(parent.join(format!("{}_obj1.adt", stem))),
            lod: Some(parent.join(format!("{}_lod.adt", stem))),
        }
    }

    /// Check which split files actually exist on the filesystem.
    ///
    /// This method tests each file path in the set to determine which files are present.
    /// Useful for determining if a complete set exists before attempting to load.
    ///
    /// # Returns
    ///
    /// A [`SplitFilePresence`] struct indicating which files exist.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use wow_adt::split_set::SplitFileSet;
    ///
    /// # fn example() -> std::io::Result<()> {
    /// let set = SplitFileSet::discover("Azeroth_30_30.adt");
    /// let presence = set.verify_existence();
    ///
    /// if presence.has_root && presence.has_tex0 && presence.has_obj0 {
    ///     println!("Complete Cataclysm ADT set found!");
    /// } else if presence.has_root && !presence.has_tex0 {
    ///     println!("Pre-Cataclysm monolithic ADT file");
    /// } else {
    ///     println!("Incomplete split file set");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn verify_existence(&self) -> SplitFilePresence {
        SplitFilePresence {
            has_root: self.root.exists(),
            has_tex0: self.tex0.as_ref().is_some_and(|p| p.exists()),
            has_tex1: self.tex1.as_ref().is_some_and(|p| p.exists()),
            has_obj0: self.obj0.as_ref().is_some_and(|p| p.exists()),
            has_obj1: self.obj1.as_ref().is_some_and(|p| p.exists()),
            has_lod: self.lod.as_ref().is_some_and(|p| p.exists()),
        }
    }

    /// Get a vector of all file paths that are present (non-None).
    ///
    /// This returns paths regardless of whether they exist on disk. Use
    /// [`verify_existence`](SplitFileSet::verify_existence) to filter to only existing files.
    ///
    /// # Returns
    ///
    /// Vector of references to all non-None file paths in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use wow_adt::split_set::SplitFileSet;
    ///
    /// let set = SplitFileSet::discover("Azeroth_30_30.adt");
    /// let paths = set.present_files();
    ///
    /// // All paths are present (including root)
    /// assert_eq!(paths.len(), 6); // root + tex0 + tex1 + obj0 + obj1 + lod
    /// ```
    pub fn present_files(&self) -> Vec<&PathBuf> {
        let mut files = vec![&self.root];

        if let Some(ref tex0) = self.tex0 {
            files.push(tex0);
        }
        if let Some(ref tex1) = self.tex1 {
            files.push(tex1);
        }
        if let Some(ref obj0) = self.obj0 {
            files.push(obj0);
        }
        if let Some(ref obj1) = self.obj1 {
            files.push(obj1);
        }
        if let Some(ref lod) = self.lod {
            files.push(lod);
        }

        files
    }

    /// Check if this represents a complete Cataclysm+ split file set.
    ///
    /// A complete set requires root, tex0, and obj0 files to exist.
    /// tex1, obj1, and lod are optional.
    ///
    /// # Returns
    ///
    /// `true` if root, tex0, and obj0 files exist.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use wow_adt::split_set::SplitFileSet;
    ///
    /// # fn example() -> std::io::Result<()> {
    /// let set = SplitFileSet::discover("Azeroth_30_30.adt");
    ///
    /// if set.is_complete_cataclysm_set() {
    ///     println!("Complete Cataclysm+ ADT tile");
    /// } else {
    ///     println!("Incomplete or pre-Cataclysm tile");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_complete_cataclysm_set(&self) -> bool {
        let presence = self.verify_existence();
        presence.has_root && presence.has_tex0 && presence.has_obj0
    }
}

/// Bitmask indicating which split files are present on the filesystem.
///
/// Returned by [`SplitFileSet::verify_existence`] to indicate which files
/// actually exist for a given ADT tile.
///
/// ## Detection Patterns
///
/// - **Pre-Cataclysm**: `has_root` only
/// - **Cataclysm**: `has_root`, `has_tex0`, `has_obj0` (minimum)
/// - **Legion+**: Above plus potentially `has_lod`
/// - **BfA+**: `tex1` typically absent (deprecated)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SplitFilePresence {
    /// Root file exists (`.adt`).
    pub has_root: bool,

    /// Primary texture file exists (`_tex0.adt`).
    pub has_tex0: bool,

    /// Secondary texture file exists (`_tex1.adt`).
    pub has_tex1: bool,

    /// Primary object file exists (`_obj0.adt`).
    pub has_obj0: bool,

    /// Secondary object file exists (`_obj1.adt`).
    pub has_obj1: bool,

    /// LOD file exists (`_lod.adt`).
    pub has_lod: bool,
}

impl SplitFilePresence {
    /// Check if this represents a complete Cataclysm+ split file set.
    ///
    /// Requires root, tex0, and obj0 to be present.
    ///
    /// # Returns
    ///
    /// `true` if minimum required files exist for Cataclysm+ split architecture.
    #[inline]
    pub const fn is_complete_cataclysm_set(&self) -> bool {
        self.has_root && self.has_tex0 && self.has_obj0
    }

    /// Check if this represents a pre-Cataclysm monolithic file.
    ///
    /// Only root file exists, no split files.
    ///
    /// # Returns
    ///
    /// `true` if only root exists (monolithic ADT).
    #[inline]
    pub const fn is_monolithic(&self) -> bool {
        self.has_root
            && !self.has_tex0
            && !self.has_tex1
            && !self.has_obj0
            && !self.has_obj1
            && !self.has_lod
    }

    /// Check if any split files are present.
    ///
    /// # Returns
    ///
    /// `true` if at least one split file (tex/obj/lod) exists.
    #[inline]
    pub const fn has_split_files(&self) -> bool {
        self.has_tex0 || self.has_tex1 || self.has_obj0 || self.has_obj1 || self.has_lod
    }

    /// Count how many files are present.
    ///
    /// # Returns
    ///
    /// Number of files that exist (0-6).
    pub const fn count(&self) -> usize {
        let mut count = 0;
        if self.has_root {
            count += 1;
        }
        if self.has_tex0 {
            count += 1;
        }
        if self.has_tex1 {
            count += 1;
        }
        if self.has_obj0 {
            count += 1;
        }
        if self.has_obj1 {
            count += 1;
        }
        if self.has_lod {
            count += 1;
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_discover_split_files() {
        let root = PathBuf::from("World/Maps/Azeroth/Azeroth_30_30.adt");
        let set = SplitFileSet::discover(&root);

        assert_eq!(
            set.root,
            PathBuf::from("World/Maps/Azeroth/Azeroth_30_30.adt")
        );
        assert_eq!(
            set.tex0,
            Some(PathBuf::from("World/Maps/Azeroth/Azeroth_30_30_tex0.adt"))
        );
        assert_eq!(
            set.tex1,
            Some(PathBuf::from("World/Maps/Azeroth/Azeroth_30_30_tex1.adt"))
        );
        assert_eq!(
            set.obj0,
            Some(PathBuf::from("World/Maps/Azeroth/Azeroth_30_30_obj0.adt"))
        );
        assert_eq!(
            set.obj1,
            Some(PathBuf::from("World/Maps/Azeroth/Azeroth_30_30_obj1.adt"))
        );
        assert_eq!(
            set.lod,
            Some(PathBuf::from("World/Maps/Azeroth/Azeroth_30_30_lod.adt"))
        );
    }

    #[test]
    fn test_discover_without_directory() {
        let root = PathBuf::from("Azeroth_30_30.adt");
        let set = SplitFileSet::discover(&root);

        assert_eq!(set.root, PathBuf::from("Azeroth_30_30.adt"));
        assert_eq!(set.tex0, Some(PathBuf::from("Azeroth_30_30_tex0.adt")));
        assert_eq!(set.obj0, Some(PathBuf::from("Azeroth_30_30_obj0.adt")));
    }

    #[test]
    fn test_present_files() {
        let set = SplitFileSet::discover("Azeroth_30_30.adt");
        let files = set.present_files();

        // All files should be present (non-None)
        assert_eq!(files.len(), 6); // root + 5 split files
    }

    #[test]
    fn test_split_file_presence_is_complete() {
        let complete = SplitFilePresence {
            has_root: true,
            has_tex0: true,
            has_tex1: false,
            has_obj0: true,
            has_obj1: false,
            has_lod: false,
        };
        assert!(complete.is_complete_cataclysm_set());

        let incomplete = SplitFilePresence {
            has_root: true,
            has_tex0: false, // Missing required file
            has_tex1: false,
            has_obj0: true,
            has_obj1: false,
            has_lod: false,
        };
        assert!(!incomplete.is_complete_cataclysm_set());
    }

    #[test]
    fn test_split_file_presence_is_monolithic() {
        let monolithic = SplitFilePresence {
            has_root: true,
            has_tex0: false,
            has_tex1: false,
            has_obj0: false,
            has_obj1: false,
            has_lod: false,
        };
        assert!(monolithic.is_monolithic());

        let split = SplitFilePresence {
            has_root: true,
            has_tex0: true,
            has_tex1: false,
            has_obj0: true,
            has_obj1: false,
            has_lod: false,
        };
        assert!(!split.is_monolithic());
    }

    #[test]
    fn test_split_file_presence_count() {
        let presence = SplitFilePresence {
            has_root: true,
            has_tex0: true,
            has_tex1: false,
            has_obj0: true,
            has_obj1: false,
            has_lod: false,
        };
        assert_eq!(presence.count(), 3);

        let all = SplitFilePresence {
            has_root: true,
            has_tex0: true,
            has_tex1: true,
            has_obj0: true,
            has_obj1: true,
            has_lod: true,
        };
        assert_eq!(all.count(), 6);
    }
}
