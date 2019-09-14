/*!
 * Utilities for testing KvStores.
 */

use std::path::Path;

use tempfile::TempDir;

use crate::{KvStore, Result};

/// Used to test KvStore implementations.
#[cfg(feature = "impl-tests")]
pub trait KvStoreTests: KvStore + Sized {
    /// Open the KvStore using the given directory.
    /// If given the same directory, must open the same store.
    fn open<P: AsRef<Path>>(dir: P) -> Result<Self>;

    /// Should get previously stored value
    fn test_get_stored_value() -> Result<()> {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");
        {
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
        }

        // Open from disk again and check persistent data
        {
            let store = Self::open(&temp_dir)?;
            assert_eq!(
                store.get("key1".to_owned())?,
                Some("value1".to_owned())
            );
            assert_eq!(
                store.get("key2".to_owned())?,
                Some("value2".to_owned())
            );
        }
        Ok(())
    }

    /// Should overwrite existent value
    fn test_overwrite_value() -> Result<()> {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");
        {
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
        }

        {
            // Open from disk again and check persistent data
            let mut store = Self::open(&temp_dir)?;
            assert_eq!(
                store.get("key1".to_owned())?,
                Some("value2".to_owned())
            );
            store.set("key1".to_owned(), "value3".to_owned())?;
            assert_eq!(
                store.get("key1".to_owned())?,
                Some("value3".to_owned())
            );
        }
        Ok(())
    }

    /// Should get `None` when getting a non-existent key
    fn test_get_nonexistent_value() -> Result<()> {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");
        {
            let mut store = Self::open(&temp_dir)?;

            store.set("key1".to_owned(), "value1".to_owned())?;
            assert_eq!(store.get("key2".to_owned())?, None);
        }

        {
            // Open from disk again and check persistent data
            let store = Self::open(&temp_dir)?;
            assert_eq!(store.get("key2".to_owned())?, None);
        }

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
        assert!(store.remove("key1".to_owned()).is_ok());
        assert_eq!(store.get("key1".to_owned())?, None);
        Ok(())
    }
}
