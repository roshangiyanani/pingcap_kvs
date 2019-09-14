use core::{ExplicitlyPersistent, KvStore, Result};

use crate::HashMapKvs;

// #[cfg_attr(test, test_impl)]
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

#[cfg(test)]
mod tests {
    use super::*;

    use core::{CoreTests, Testable};
    use std::path::Path;

    impl Testable for HashMapKvs {
        fn open<P: AsRef<Path>>(dir: P) -> Result<Self> {
            HashMapKvs::open(dir.as_ref().join("kvs"))
        }
    }

    #[test]
    fn get_stored_value() -> Result<()> {
        HashMapKvs::test_get_stored_value()
    }

    #[test]
    fn overwrite_value() -> Result<()> {
        HashMapKvs::test_overwrite_value()
    }

    #[test]
    fn get_non_existent_value() -> Result<()> {
        HashMapKvs::test_get_nonexistent_value()
    }

    #[test]
    fn remove_non_existent_key() -> Result<()> {
        HashMapKvs::test_remove_non_existent_key()
    }

    #[test]
    fn remove_key() -> Result<()> {
        HashMapKvs::test_remove_key()
    }
}
