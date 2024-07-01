use std::fmt::Display;

use enc::hex::HexEncoder;
use enc::StringEncoder;
use file_storage::{Error, FilePath, FolderPath};
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
    pub fn file_path<K>(&self, key: K) -> FilePath
    where
        K: Display,
    {
        let key: String = key.to_string();

        let mut hasher: Sha256 = Sha256::default();
        Update::update(&mut hasher, key.as_bytes());
        let hash: Box<[u8]> = Box::new(hasher).finalize();
        let hash: String = HexEncoder::LOWER.encode_as_string(hash.as_ref()).unwrap();
        let extension: String = format!("{}/{}.cache", &hash[..4], &hash[4..]);
        let file_path: FilePath = self
            .cache_folder
            .path()
            .clone_append(extension)
            .to_file()
            .unwrap();
        file_path
    }
}

impl SimpleFileCache {
    //! Put

    /// Puts the data into the cache.
    pub fn put<K, D>(&self, key: K, data: D) -> Result<(), Error>
    where
        K: Display,
        D: AsRef<[u8]>,
    {
        let file: FilePath = self.file_path(key);
        file.delete_if_exists()?;
        file.write_data(data)
    }
}

impl SimpleFileCache {
    //! Get

    /// Gets the data in the cache.
    pub fn get<K>(&self, key: K) -> Result<Option<Vec<u8>>, Error>
    where
        K: Display,
    {
        self.file_path(key).read_as_vec_if_exists()
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
        let file_path: FilePath = cache.file_path(key);
        let result: &str = file_path.as_str();
        let expected: &str =
            "/cache/folder/dffd/6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f.cache";
        assert_eq!(result, expected);
    }

    #[test]
    fn put_get() -> Result<(), Box<dyn std::error::Error>> {
        let cache: SimpleFileCache = SimpleFileCache::temp()?;
        assert_eq!(cache.get("key")?, None);

        cache.put("key", "data")?;
        let result: Option<Vec<u8>> = cache.get("key")?;
        assert!(result.is_some());

        let result: String = String::from_utf8(result.unwrap())?;
        assert_eq!(result, "data");

        Ok(())
    }
}
