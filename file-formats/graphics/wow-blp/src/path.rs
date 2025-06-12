use std::path::{Path, PathBuf};

/// Helper to make a external mipmap path based on given root file name.
pub fn make_mipmap_path<Q>(path: Q, i: usize) -> Option<PathBuf>
where
    Q: AsRef<Path>,
{
    let mut base = path.as_ref().parent()?.join(path.as_ref().file_stem()?);
    let extension = format!("b{:02}", i);
    base.set_extension(extension);
    Some(base)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blp0_path() {
        assert_eq!(
            make_mipmap_path("test.blp", 0).expect("path"),
            Path::new("test.b00")
        );
        assert_eq!(
            make_mipmap_path("/root/dir/test.blp", 2).expect("path"),
            Path::new("/root/dir/test.b02")
        );
        assert_eq!(
            make_mipmap_path("/root/dir/test.blp", 14).expect("path"),
            Path::new("/root/dir/test.b14")
        );
    }
}
