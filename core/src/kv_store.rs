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
}

#[cfg(feature = "impl-tests")]
/// Functions, traits, and macros for easily testing KvStore implementation.
pub mod kv_store_tests {
    use super::*;

    use tempfile::TempDir;

    use crate::tests::Testable;

    impl<S> CoreTests for S where S: Testable {}

    #[macro_export]
    /// Generate tests for the given type using all the CoreTest functions
    macro_rules! generate_core_tests {
        ( $t: ty ) => {
            use $crate::kv_store_tests::CoreTests;

            test_function!($t, test_get_stored_value);
            test_function!($t, test_overwrite_value);
            test_function!($t, test_get_nonexistent_value);
            test_function!($t, test_remove_non_existent_key);
            test_function!($t, test_remove_key);
        };
    }

    /// Functions to test core KvStore implementations.
    pub trait CoreTests: Testable {
        /// Should get previously stored value
        fn test_get_stored_value() -> Result<()> {
            let temp_dir = TempDir::new()
                .expect("unable to create temporary working directory");

            let mut store = Self::open(&temp_dir)?;

            store.set("key1".to_owned(), "value1".to_owned())?;
            store.set("key2".to_owned(), "value2".to_owned())?;

            assert_eq!(
                store.get("key1".to_owned())?,
                Some("value1".to_owned())
            );
            assert_eq!(
                store.get("key2".to_owned())?,
                Some("value2".to_owned())
            );
            Ok(())
        }

        /// Should overwrite existent value
        fn test_overwrite_value() -> Result<()> {
            let temp_dir = TempDir::new()
                .expect("unable to create temporary working directory");

            let mut store = Self::open(&temp_dir)?;

            store.set("key1".to_owned(), "value1".to_owned())?;
            assert_eq!(
                store.get("key1".to_owned())?,
                Some("value1".to_owned())
            );
            store.set("key1".to_owned(), "value2".to_owned())?;
            assert_eq!(
                store.get("key1".to_owned())?,
                Some("value2".to_owned())
            );

            Ok(())
        }

        /// Should get `None` when getting a non-existent key
        fn test_get_nonexistent_value() -> Result<()> {
            let temp_dir = TempDir::new()
                .expect("unable to create temporary working directory");

            let mut store = Self::open(&temp_dir)?;

            store.set("key1".to_owned(), "value1".to_owned())?;
            assert_eq!(store.get("key2".to_owned())?, None);

            Ok(())
        }

        /// Should get 'None' when removing
        fn test_remove_non_existent_key() -> Result<()> {
            let temp_dir = TempDir::new()
                .expect("unable to create temporary working directory");
            let mut store = Self::open(&temp_dir)?;
            let status = store.remove("key1".to_owned());
            assert!(status.is_ok());
            assert!(status.unwrap().is_none());
            Ok(())
        }

        /// Shouldn't contain key after removal
        fn test_remove_key() -> Result<()> {
            let temp_dir = TempDir::new()
                .expect("unable to create temporary working directory");
            let mut store = Self::open(&temp_dir)?;
            store.set("key1".to_owned(), "value1".to_owned())?;
            assert_eq!(
                store.remove("key1".to_owned())?,
                Some("value1".to_owned())
            );
            assert_eq!(store.get("key1".to_owned())?, None);
            Ok(())
        }
    }
}
