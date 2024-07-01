use std::fmt::Display;
use std::io::ErrorKind::Other;

use enc::hex::HexEncoder;
use enc::StringEncoder;
use file_storage::{FilePath, FolderPath};
use sha2::digest::{DynDigest, Update};
use sha2::Sha256;

/// A simple file cache.
///
/// # Keys
/// The cache keys can be any type that implements `Display`. The folder & file name is based on
/// the hash of the `Display` string. Hash collisions are highly unlikely.
#[derive(Clone, Debug)]
pub struct SimpleFileCache {
    cache_folder: FolderPath,
}

impl SimpleFileCache {
    //! Construction

    /// Creates a cache in a new temp folder.
    pub fn temp() -> Result<Self, std::io::Error> {
        Ok(Self::from(FolderPath::temp()?))
    }
}

impl From<FolderPath> for SimpleFileCache {
    fn from(cache_folder: FolderPath) -> Self {
        Self { cache_folder }
    }
}

impl SimpleFileCache {
    //! File Names

    /// Gets the file path for the key.
    pub fn file_path<K>(&self, key: K) -> Result<FilePath, std::io::Error>
    where
        K: Display,
    {
        let key: String = key.to_string();

        let mut hasher: Sha256 = Sha256::default();
        Update::update(&mut hasher, key.as_bytes());
        let hash: Box<[u8]> = Box::new(hasher).finalize();
        let hash: String = HexEncoder::LOWER
            .encode_as_string(hash.as_ref())
            .map_err(|error| std::io::Error::new(Other, error))?;
        let extension: String = format!("{}/{}.cache", &hash[..4], &hash[4..]);
        let file_path: FilePath = self
            .cache_folder
            .path()
            .clone_append(extension)
            .to_file()
            .ok_or_else(|| std::io::Error::new(Other, "path not a file"))?;
        Ok(file_path)
    }
}

#[cfg(test)]
mod tests {
    use file_storage::{FilePath, FolderPath, Path};

    use crate::SimpleFileCache;

    #[test]
    fn file_path() {
        let folder: FolderPath = Path::unix_root()
            .with_appended("cache/folder/")
            .make_folder();
        let cache: SimpleFileCache = SimpleFileCache::from(folder);
        let key: &str = "Hello, World!";
        let file_path: FilePath = cache.file_path(key).unwrap();
        let result: &str = file_path.as_str();
        let expected: &str =
            "/cache/folder/dffd/6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f.cache";
        assert_eq!(result, expected);
    }
}
