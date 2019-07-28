use std::collections::HashMap;

/// Interface to the key-value store
#[derive(Default, Debug)]
pub struct KvStore {
    inner: HashMap<String, String>,
}

impl KvStore {
    /// Initialize the key value store
    /// ```rust
    /// # fn main() {
    /// let kvs = kvs::KvStore::new();
    /// # }
    /// ```
    pub fn new() -> Self {
        KvStore {
            inner: HashMap::new(),
        }
    }

    /// Set a value. If the key already existed, the old value is overwritten.
    /// ```rust
    /// # fn main() {
    /// let mut kvs = kvs::KvStore::new();
    /// kvs.set("key1".to_owned(), "value1".to_owned());
    /// # }
    /// ```
    pub fn set(&mut self, key: String, value: String) {
        self.inner.insert(key, value);
    }

    /// Retrieve a value
    /// ```rust
    /// # fn main() {
    /// let mut kvs = kvs::KvStore::new();
    /// kvs.set("key1".to_owned(), "value1".to_owned());
    /// kvs.get("key1".to_owned());
    /// # }
    /// ```
    pub fn get(&self, key: String) -> Option<String> {
        self.inner.get(&key).cloned()
    }

    /// Remove a value
    /// ```rust
    /// # fn main() {
    /// let mut kvs = kvs::KvStore::new();
    /// kvs.set("key1".to_owned(), "value1".to_owned());
    /// kvs.remove("key1".to_owned());
    /// # }
    /// ```
    pub fn remove(&mut self, key: String) {
        self.inner.remove(&key);
    }
}
