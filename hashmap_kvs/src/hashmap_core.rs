use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use core::{ExplicitlyPersistent, Result};

/// An implementation of a key-value store using an in memory hashmap that
/// only saves the store on close.
#[derive(Debug)]
pub struct HashMapKvs {
    pub(crate) map: HashMap<String, String>,
    pub(crate) backing: PathBuf,
    pub(crate) mutated: bool,
}

impl HashMapKvs {
    /// Initialize the key value store
    ///
    /// ```rust
    /// use tempfile::TempDir;
    ///
    /// let temp_dir =
    ///     TempDir::new().expect("unable to create temporary working directory");
    /// let mut store =
    ///     hashmap_kvs::HashMapKvs::open(temp_dir.path().join("kvs")).unwrap();
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        if path.as_ref().is_file() {
            HashMapKvs::load(path)
        } else {
            HashMapKvs::new(path)
        }
    }

    fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut kvs = HashMapKvs {
            map: HashMap::new(),
            backing: PathBuf::from(path.as_ref()),
            mutated: true,
        };

        kvs.save()?;
        Ok(kvs)
    }

    fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let backing_file = File::open(&path)?;
        let reader = BufReader::new(backing_file);
        let map: HashMap<String, String> = serde_json::from_reader(reader)?;

        Ok(HashMapKvs {
            map,
            backing: PathBuf::from(path.as_ref()),
            mutated: false,
        })
    }
}

impl Drop for HashMapKvs {
    fn drop(&mut self) {
        if self.mutated {
            self.save().expect("error saving HashMapKvs during drop");
        }
    }
}
