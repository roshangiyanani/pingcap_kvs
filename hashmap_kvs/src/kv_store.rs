use core::{KvStore, Result};

use crate::HashMapKvs;

// #[cfg_attr(test, test_impl)]
impl KvStore for HashMapKvs {
    /// Set a value. If the key already existed, the old value is overwritten.
    ///
    /// ```rust
    /// # use tempfile::TempDir;
    /// # use core::{KvStore, Persistent};
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
    /// # use core::{KvStore, Persistent};
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
    /// # use core::{Persistent, KvStore};
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
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::tests::Testable;
    use core::Persistent;
    use std::path::Path;

    impl Testable for HashMapKvs {
        fn open<P: AsRef<Path>>(dir: P) -> Result<Self> {
            Persistent::open(dir.as_ref().join("kvs"))
        }
    }

    generate_core_tests!(HashMapKvs);
}
