use std::collections::HashMap;
use std::fs::{create_dir, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use strum_macros::Display;

use crate::{Error, KvStore, Result};

const DEFAULT_LOG_NAME: &str = "1";
const DEFAULT_LOG_ID: usize = 1;

/// An implementation of a key-value store using an append-only log store.
#[derive(Debug)]
pub struct LogKvs {
    index: HashMap<String, LogCommandPointer>,
    backing_dir: PathBuf,
}

#[derive(Debug, Display, Serialize, Deserialize)]
enum LogCommand {
    /// Add a value to the key-value store.
    Set {
        /// The name to store the value under.
        key: String,
        /// The value to store.
        value: String,
    },
    /// Remove a value from the key-value store.
    Remove {
        /// The item to delete.
        key: String,
    },
}

impl LogCommand {
    fn append<W: Write>(&self, writer: &mut W) -> Result<()> {
        bincode::serialize_into(writer, self).map_err(Error::bincode)
    }

    fn read<R: Read>(reader: &mut R) -> Result<LogCommand> {
        bincode::deserialize_from(reader).map_err(Error::bincode)
    }
}

#[derive(Debug)]
struct LogCommandPointer {
    file_id: usize,
    offset: u64,
}

impl LogCommandPointer {
    fn new(file_id: usize, offset: u64) -> LogCommandPointer {
        LogCommandPointer { file_id, offset }
    }
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
        let kvs = LogKvs {
            index: HashMap::new(),
            backing_dir: PathBuf::from(path.as_ref()),
        };

        Ok(kvs)
    }

    fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut kvs = LogKvs {
            index: HashMap::new(),
            backing_dir: PathBuf::from(path.as_ref()),
        };

        let file = File::open(kvs.backing_dir.join(DEFAULT_LOG_NAME))
            .map_err(Error::io)?;
        let mut reader = BufReader::new(file);
        let end_pos = reader.stream_len().map_err(Error::io)?;
        let mut current_pos = reader.stream_position().map_err(Error::io)?;
        while current_pos < end_pos {
            let command = LogCommand::read(&mut reader)?;
            let pointer = LogCommandPointer::new(DEFAULT_LOG_ID, current_pos);
            kvs.replay(command, pointer)?;
            current_pos = reader.stream_position().map_err(Error::io)?;
        }
        Ok(kvs)
    }

    fn replay(
        &mut self,
        command: LogCommand,
        pointer: LogCommandPointer,
    ) -> Result<()> {
        match command {
            LogCommand::Set { key, value } => {
                self.index.insert(key, pointer);
                Ok(())
            }
            LogCommand::Remove { key } => {
                self.index.remove(&key).ok_or_else(|| {
                    Error::corrupt_database(format!(
                        "attempted removal of nonexistent key '{}' from the \
                         index",
                        key
                    ))
                })?;
                Ok(())
            }
        }
    }

    fn append(&self, command: LogCommand) -> Result<LogCommandPointer> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(self.backing_dir.join(DEFAULT_LOG_NAME))
            .map_err(Error::io)?;
        let mut writer = BufWriter::new(file);
        let current_pos =
            writer.seek(std::io::SeekFrom::End(0)).map_err(Error::io)?;
        command.append(&mut writer)?;
        Ok(LogCommandPointer::new(DEFAULT_LOG_ID, current_pos))
    }

    fn get_command(&self, pointer: &LogCommandPointer) -> Result<LogCommand> {
        let mut file = File::open(self.backing_dir.join(DEFAULT_LOG_NAME))
            .map_err(Error::io)?;
        file.seek(std::io::SeekFrom::Start(pointer.offset))
            .map_err(Error::io)?;
        let mut reader = BufReader::new(file);
        LogCommand::read(&mut reader)
    }

    fn get_key(&self, pointer: &LogCommandPointer) -> Result<String> {
        match self.get_command(pointer)? {
            LogCommand::Set { key, value } => Ok(value),
            LogCommand::Remove { key } => {
                Err(Error::corrupt_database(format!(
                    "LogCommand at {:?} should set key '{}', not remove it",
                    pointer, key
                )))
            }
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
        let pointer = self.append(LogCommand::Set {
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
                self.append(LogCommand::Remove { key })?;
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