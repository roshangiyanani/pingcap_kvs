#![deny(missing_docs)]

/*!
 * A library exposing a key-value store that uses an in-memory hashmap.
 */

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use serde_json;

use core::{KvStore, Result};
use io::safe_overwrite;

/// An implementation of a key-value store using an in memory hashmap that
/// only saves the store on close.
#[derive(Debug)]
pub struct HashMapKvs {
    map: HashMap<String, String>,
    backing: PathBuf,
    mutated: bool,
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

    fn save(&mut self) -> Result<()> {
        safe_overwrite(self.backing.clone(), |writer: BufWriter<File>| {
            serde_json::to_writer(writer, &self.map)?;
            self.mutated = false;
            Ok(())
        })
    }
}

impl KvStore for HashMapKvs {
    /// Set a value. If the key already existed, the old value is overwritten.
    ///
    /// ```rust
    /// # use tempfile::TempDir;
    /// # use core::KvStore;
    /// # use hashmap_kvs::HashMapKvs;
    /// #
    /// # let temp_dir =
    /// #    TempDir::new().expect("unable to create temporary working directory");
    /// # let mut store = HashMapKvs::open(temp_dir.path().join("kvs")).unwrap();
    /// store.set("key1".to_owned(), "value1".to_owned());
    /// ```
    fn set(&mut self, key: String, value: String) -> Result<()> {
        self.map.insert(key, value);
        self.mutated = true;
        Ok(())
    }

    /// Retrieve the value of a key. If the key does not exist, return None.
    /// Return an error if the value is not read successfully.
    ///
    /// ```rust
    /// # use tempfile::TempDir;
    /// # use core::KvStore;
    /// # use hashmap_kvs::HashMapKvs;
    /// #
    /// # let temp_dir =
    /// #    TempDir::new().expect("unable to create temporary working directory");
    /// # let mut store = HashMapKvs::open(temp_dir.path().join("kvs")).unwrap();
    /// store.set("key1".to_owned(), "value1".to_owned());
    /// store.get("key1".to_owned());
    /// ```
    fn get(&self, key: String) -> Result<Option<String>> {
        Ok(self.map.get(&key).cloned())
    }

    /// Remove a key-value. Return an error if the key does not exist or is not
    /// removed successfully.
    ///
    /// ```rust
    /// # use tempfile::TempDir;
    /// # use core::KvStore;
    /// # use hashmap_kvs::HashMapKvs;
    /// #
    /// # let temp_dir =
    /// #     TempDir::new().expect("unable to create temporary working directory");
    /// # let mut store = HashMapKvs::open(temp_dir.path().join("kvs")).unwrap();
    /// store.set("key1".to_owned(), "value1".to_owned());
    /// store.remove("key1".to_owned());
    /// ```
    fn remove(&mut self, key: String) -> Result<Option<String>> {
        let status = self.map.remove(&key);
        if status.is_some() {
            self.mutated = true;
        }
        Ok(status)
    }

    /// Save (if it has been changed) and close the key-value store.
    /// ```rust
    /// use core::KvStore;
    /// use hashmap_kvs::HashMapKvs;
    /// use tempfile::TempDir;
    ///
    /// let temp_dir =
    ///     TempDir::new().expect("unable to create temporary working directory");
    /// let mut store = HashMapKvs::open(temp_dir.path().join("kvs")).unwrap();
    /// store.close().unwrap();
    /// ```
    fn close(mut self) -> Result<()> {
        if self.mutated {
            self.save()
        } else {
            Ok(())
        }
    }
}

impl Drop for HashMapKvs {
    fn drop(&mut self) {
        if self.mutated {
            self.save().expect("error saving HashMapKvs during drop");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Should get previously stored value
    #[test]
    fn get_stored_value() -> Result<()> {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");
        let mut store = HashMapKvs::open(temp_dir.path().join("kvs"))?;

        store.set("key1".to_owned(), "value1".to_owned())?;
        store.set("key2".to_owned(), "value2".to_owned())?;

        assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
        assert_eq!(store.get("key2".to_owned())?, Some("value2".to_owned()));

        // Open from disk again and check persistent data
        drop(store);
        let store = HashMapKvs::open(temp_dir.path().join("kvs"))?;
        assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
        assert_eq!(store.get("key2".to_owned())?, Some("value2".to_owned()));

        Ok(())
    }

    // Should overwrite existent value
    #[test]
    fn overwrite_value() -> Result<()> {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");
        let mut store = HashMapKvs::open(temp_dir.path().join("kvs"))?;

        store.set("key1".to_owned(), "value1".to_owned())?;
        assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
        store.set("key1".to_owned(), "value2".to_owned())?;
        assert_eq!(store.get("key1".to_owned())?, Some("value2".to_owned()));

        // Open from disk again and check persistent data
        drop(store);
        let mut store = HashMapKvs::open(temp_dir.path().join("kvs"))?;
        assert_eq!(store.get("key1".to_owned())?, Some("value2".to_owned()));
        store.set("key1".to_owned(), "value3".to_owned())?;
        assert_eq!(store.get("key1".to_owned())?, Some("value3".to_owned()));

        Ok(())
    }

    // Should get `None` when getting a non-existent key
    #[test]
    fn get_non_existent_value() -> Result<()> {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");
        let mut store = HashMapKvs::open(temp_dir.path().join("kvs"))?;

        store.set("key1".to_owned(), "value1".to_owned())?;
        assert_eq!(store.get("key2".to_owned())?, None);

        // Open from disk again and check persistent data
        drop(store);
        let store = HashMapKvs::open(temp_dir.path().join("kvs"))?;
        assert_eq!(store.get("key2".to_owned())?, None);

        Ok(())
    }

    #[test]
    fn remove_non_existent_key() -> Result<()> {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");
        let mut store = HashMapKvs::open(temp_dir.path().join("kvs"))?;
        let status = store.remove("key1".to_owned());
        assert!(status.is_ok());
        assert!(status.unwrap().is_none());
        Ok(())
    }

    #[test]
    fn remove_key() -> Result<()> {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");
        let mut store = HashMapKvs::open(temp_dir.path().join("kvs"))?;
        store.set("key1".to_owned(), "value1".to_owned())?;
        assert!(store.remove("key1".to_owned()).is_ok());
        assert_eq!(store.get("key1".to_owned())?, None);
        Ok(())
    }
}
