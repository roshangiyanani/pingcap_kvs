use crate::{Command, LogKvs};
use core::{KvStore, Result};

impl KvStore for LogKvs {
    /// Set a value. If the key already existed, the old value is overwritten.
    ///
    /// ```rust
    /// # use tempfile::TempDir;
    /// # use core::{KvStore, Persistent};
    /// # use log_kvs::LogKvs;
    ///
    /// # let temp_dir =
    /// #    TempDir::new().expect("unable to create temporary working directory");
    /// # let mut store = LogKvs::open(temp_dir.path()).unwrap();
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
    /// # use core::{Persistent, KvStore};
    /// # use log_kvs::LogKvs;
    /// #
    /// # let temp_dir =
    /// #    TempDir::new().expect("unable to create temporary working directory");
    /// # let mut store = LogKvs::open(temp_dir.path()).unwrap();
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
    /// # use core::{Persistent, KvStore};
    /// # use log_kvs::LogKvs;
    /// #
    /// # let temp_dir =
    /// #     TempDir::new().expect("unable to create temporary working directory");
    /// # let mut store = LogKvs::open(temp_dir.path()).unwrap();
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
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::tests::Testable;
    use core::Persistent;
    use std::path::Path;

    impl Testable for LogKvs {
        fn open<P: AsRef<Path>>(dir: P) -> Result<Self> {
            Persistent::open(dir.as_ref().join("kvs"))
        }
    }

    generate_core_tests!(LogKvs);
}
