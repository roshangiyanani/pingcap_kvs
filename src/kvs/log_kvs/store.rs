use std::collections::HashMap;
use std::fs::create_dir;
use std::path::Path;

use super::{Command, LogCommandPointer, LogFile};
use crate::{Error, KvStore, Result};

pub const DEFAULT_LOG_NAME: &str = "1";
pub const DEFAULT_LOG_ID: usize = 1;

/// An implementation of a key-value store using an append-only log store.
#[derive(Debug)]
pub struct LogKvs {
    index: HashMap<String, LogCommandPointer>,
    log: LogFile,
}

impl LogKvs {
    /// Initialize the key value store
    ///
    /// ```rust
    /// use tempfile::TempDir;
    ///
    /// let temp_dir =
    ///     TempDir::new().expect("unable to create temporary working directory");
    /// let mut store = kvs::LogKvs::open(temp_dir.path()).unwrap();
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = Path::new(path.as_ref());

        // create directory if need be
        if let Err(err) = create_dir(path) {
            if err.kind() != std::io::ErrorKind::AlreadyExists {
                return Err(Error::io(err));
            }
        }

        if path.join(DEFAULT_LOG_NAME).is_file() {
            LogKvs::load(path)
        } else {
            LogKvs::new(path)
        }
    }

    fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = Path::new(path.as_ref());
        let default_file = path.join(DEFAULT_LOG_NAME);

        let kvs = LogKvs {
            index: HashMap::new(),
            log: LogFile::new(default_file),
        };

        Ok(kvs)
    }

    fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = Path::new(path.as_ref());
        let default_file = path.join(DEFAULT_LOG_NAME);

        let mut kvs = LogKvs {
            index: HashMap::new(),
            log: LogFile::new(default_file),
        };

        for record in kvs.log.iter()? {
            let (command, pointer) = record?;
            // println!("replaying {:?}, {:?}", command, pointer);
            kvs.replay(command, pointer)?;
        }
        Ok(kvs)
    }

    fn replay(
        &mut self,
        command: Command,
        pointer: LogCommandPointer,
    ) -> Result<()> {
        match command {
            Command::Set { key, .. } => {
                self.index.insert(key, pointer);
            }
            Command::Remove { key } => {
                self.index.remove(&key).ok_or_else(|| {
                    Error::corrupt_database(format!(
                        "attempted removal of nonexistent key '{}' from the \
                         index",
                        key
                    ))
                })?;
            }
        }
        Ok(())
    }

    fn get_key(&self, pointer: &LogCommandPointer) -> Result<String> {
        match self.log.get_command(pointer)? {
            Command::Set { value, .. } => Ok(value),
            Command::Remove { key } => Err(Error::corrupt_database(format!(
                "Command at {:?} should set key '{}', not remove it",
                pointer, key
            ))),
        }
    }

    fn compact<P: AsRef<Path>>(path: P) -> Result<Self> {
        unimplemented!()
    }
}

impl KvStore for LogKvs {
    /// Set a value. If the key already existed, the old value is overwritten.
    ///
    /// ```rust
    /// # use tempfile::TempDir;
    /// # use kvs::KvStore;
    /// #
    /// # let temp_dir =
    /// #    TempDir::new().expect("unable to create temporary working directory");
    /// # let mut store = kvs::LogKvs::open(temp_dir.path()).unwrap();
    /// store.set("key1".to_owned(), "value1".to_owned());
    /// ```
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let pointer = self.log.append(Command::Set {
            key: key.clone(),
            value: value.clone(),
        })?;
        self.index.insert(key, pointer);
        Ok(())
    }

    /// Retrieve the value of a key. If the key does not exist, return None.
    /// Return an error if the value is not read successfully.
    ///
    /// ```rust
    /// # use tempfile::TempDir;
    /// # use kvs::KvStore;
    /// #
    /// # let temp_dir =
    /// #    TempDir::new().expect("unable to create temporary working directory");
    /// # let mut store = kvs::LogKvs::open(temp_dir.path()).unwrap();
    /// store.set("key1".to_owned(), "value1".to_owned());
    /// store.get("key1".to_owned());
    /// ```
    fn get(&self, key: String) -> Result<Option<String>> {
        match self.index.get(&key) {
            Some(pointer) => {
                self.get_key(pointer).and_then(|value| Ok(Some(value)))
            }
            None => Ok(None),
        }
    }

    /// Remove a key-value. Return an error if the key does not exist or is not
    /// removed successfully.
    ///
    /// ```rust
    /// # use tempfile::TempDir;
    /// # use kvs::KvStore;
    /// #
    /// # let temp_dir =
    /// #     TempDir::new().expect("unable to create temporary working directory");
    /// # let mut store = kvs::LogKvs::open(temp_dir.path()).unwrap();
    /// store.set("key1".to_owned(), "value1".to_owned());
    /// store.remove("key1".to_owned());
    /// ```
    fn remove(&mut self, key: String) -> Result<Option<String>> {
        match self.index.remove(&key) {
            Some(old_pointer) => {
                // TODO: If append fails, index is now inconsistent
                self.log.append(Command::Remove { key })?;
                self.get_key(&old_pointer).and_then(|value| Ok(Some(value)))
            }
            None => Ok(None),
        }
    }

    /// Save (if it has been changed) and close the key-value store.
    /// ```rust
    /// use kvs::KvStore;
    /// use tempfile::TempDir;
    ///
    /// let temp_dir =
    ///     TempDir::new().expect("unable to create temporary working directory");
    /// let mut store = kvs::LogKvs::open(temp_dir.path()).unwrap();
    /// store.close().unwrap();
    /// ```
    fn close(self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use walkdir::WalkDir;

    // Should get previously stored value
    #[test]
    fn get_stored_value() -> Result<()> {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");
        let mut store = LogKvs::open(temp_dir.path())?;

        store.set("key1".to_owned(), "value1".to_owned())?;
        store.set("key2".to_owned(), "value2".to_owned())?;

        assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
        assert_eq!(store.get("key2".to_owned())?, Some("value2".to_owned()));

        // Open from disk again and check persistent data
        drop(store);
        let store = LogKvs::open(temp_dir.path())?;
        assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
        assert_eq!(store.get("key2".to_owned())?, Some("value2".to_owned()));

        Ok(())
    }

    // Should overwrite existent value
    #[test]
    fn overwrite_value() -> Result<()> {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");
        let mut store = LogKvs::open(temp_dir.path())?;

        store.set("key1".to_owned(), "value1".to_owned())?;
        assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
        store.set("key1".to_owned(), "value2".to_owned())?;
        assert_eq!(store.get("key1".to_owned())?, Some("value2".to_owned()));

        // Open from disk again and check persistent data
        drop(store);
        let mut store = LogKvs::open(temp_dir.path())?;
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
        let mut store = LogKvs::open(temp_dir.path())?;

        store.set("key1".to_owned(), "value1".to_owned())?;
        assert_eq!(store.get("key2".to_owned())?, None);

        // Open from disk again and check persistent data
        drop(store);
        let store = LogKvs::open(temp_dir.path())?;
        assert_eq!(store.get("key2".to_owned())?, None);

        Ok(())
    }

    #[test]
    fn remove_non_existent_key() -> Result<()> {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");
        let mut store = LogKvs::open(temp_dir.path())?;
        let status = store.remove("key1".to_owned());
        assert!(status.is_ok());
        assert!(status.unwrap().is_none());
        Ok(())
    }

    #[test]
    fn remove_key() -> Result<()> {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");
        let mut store = LogKvs::open(temp_dir.path())?;
        store.set("key1".to_owned(), "value1".to_owned())?;
        assert!(store.remove("key1".to_owned()).is_ok());
        assert_eq!(store.get("key1".to_owned())?, None);
        Ok(())
    }

    // Insert data until total size of the directory decreases.
    // Test data correctness after compaction.
    // #[test]
    fn compaction() -> Result<()> {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");
        let mut store = LogKvs::open(temp_dir.path())?;

        let dir_size = || {
            let entries = WalkDir::new(temp_dir.path()).into_iter();
            let len: walkdir::Result<u64> = entries
                .map(|res| {
                    res.and_then(|entry| entry.metadata())
                        .map(|metadata| metadata.len())
                })
                .sum();
            len.expect("fail to get directory size")
        };

        let mut current_size = dir_size();
        for iter in 0..1000 {
            for key_id in 0..1000 {
                let key = format!("key{}", key_id);
                let value = format!("{}", iter);
                store.set(key, value)?;
            }

            let new_size = dir_size();
            if new_size > current_size {
                current_size = new_size;
                continue;
            }
            // Compaction triggered

            drop(store);
            // reopen and check content
            let store = LogKvs::open(temp_dir.path())?;
            for key_id in 0..1000 {
                let key = format!("key{}", key_id);
                assert_eq!(store.get(key)?, Some(format!("{}", iter)));
            }
            return Ok(());
        }

        panic!("No compaction detected");
    }
}
