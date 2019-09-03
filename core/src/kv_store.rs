use crate::Result;

/// Trait for the key value store
pub trait KvStore {
    /// Set a value. If the key already existed, the old value is overwritten.
    fn set(&mut self, key: String, value: String) -> Result<()>;

    /// Retrieve the value of a key. If the key does not exist, return None.
    /// Return an error if the value is not read successfully.
    fn get(&self, key: String) -> Result<Option<String>>;

    /// Remove a key-value, returning the value. If the key does not exist,
    /// return None. Return an error if the key is not removed successfully.
    fn remove(&mut self, key: String) -> Result<Option<String>>;

    /// Close the key-value store. Return an error if unable to close it.
    fn close(self) -> Result<()>;
}

/// Trait for compactable key value stores
pub trait Compactable: KvStore {
    /// Compacts the key value store
    fn compact(&mut self) -> Result<()>;
}

/// Defines a trait for manually persistant key value stores
pub trait ExplicitlyPersistent: KvStore + Drop {
    /// Saves the key value store to some kind of persistant storage
    fn save(&mut self) -> Result<()>;
}
