use std::path::Path;

use crate::{KvStore, Result};

/// Defines a trait for persistent key value stores
pub trait Persistent: KvStore + Sized + Drop {
    /// The type of path the KvStore uses
    const PATH_TYPE: PathType;

    /// Instantiate the Persistent KvStore using the given path.
    /// If the location doesn't exist yet, create it.
    fn open<P: AsRef<Path>>(path: P) -> Result<Self>;

    /// Saves the key value store to some kind of persistant storage
    fn save(&mut self) -> Result<()>;
}

/// The options for the type of path the Persisent KvStore uses
pub enum PathType {
    /// Uses a single file
    File,
    /// Uses a directory
    Directory,
}

#[cfg(feature = "impl-tests")]
/// Functions, traits, and macros for easily testing Persistent KvStore
/// implementations.
pub mod persistent_tests {
    use super::*;

    use std::path::PathBuf;

    use tempfile::TempDir;

    #[macro_export]
    /// Generate tests for the given type using all the PersistentTests
    /// functions
    macro_rules! generate_persistent_tests {
        ( $t: ty ) => {
            use $crate::persistent_tests::PersistentTests;

            test_functions!(
                $t,
                test_storing_values,
                test_overwriting_values,
                test_nonexistent_values,
                test_removals
            );
        };
    }

    impl<P> PersistentTests for P where P: Persistent {}

    /// Functions to test Persistent KvStore implementations.
    pub trait PersistentTests: Persistent {
        /// Should contain previously saved value
        fn test_storing_values() -> Result<()> {
            let (_temp_dir, path) = get_temp_dir_and_path::<Self>();

            {
                let mut store = Self::open(&path)?;
                store.set("key1".to_owned(), "value1".to_owned())?;
            }

            {
                let store = Self::open(&path)?;
                assert_eq!(
                    store.get("key1".to_owned())?,
                    Some("value1".to_owned())
                );
            }

            Ok(())
        }

        /// Should contain overwritten values
        fn test_overwriting_values() -> Result<()> {
            let (_temp_dir, path) = get_temp_dir_and_path::<Self>();

            {
                let mut store = Self::open(&path)?;
                store.set("key1".to_owned(), "value1".to_owned())?;
                store.set("key1".to_owned(), "value2".to_owned())?;
            }

            {
                let mut store = Self::open(&path)?;
                assert_eq!(
                    store.get("key1".to_owned())?,
                    Some("value2".to_owned())
                );
                store.set("key1".to_owned(), "value3".to_owned())?;
            }

            {
                let store = Self::open(&path)?;
                assert_eq!(
                    store.get("key1".to_owned())?,
                    Some("value3".to_owned())
                );
            }

            Ok(())
        }

        /// Should not contain key
        fn test_nonexistent_values() -> Result<()> {
            let (_temp_dir, path) = get_temp_dir_and_path::<Self>();

            {
                let mut store = Self::open(&path)?;
                assert_eq!(store.get("key1".to_owned())?, None);
                store.set("key1".to_owned(), "value1".to_owned())?;
            }

            {
                let store = Self::open(&path)?;
                assert_eq!(store.get("key2".to_owned())?, None);
            }

            Ok(())
        }

        /// Should not contain key after removal
        fn test_removals() -> Result<()> {
            let (_temp_dir, path) = get_temp_dir_and_path::<Self>();

            {
                let mut store = Self::open(&path)?;
                store.set("key1".to_owned(), "value1".to_owned())?;
            }

            {
                let mut store = Self::open(&path)?;
                store.remove("key1".to_owned())?;
            }

            {
                let store = Self::open(&path)?;
                assert_eq!(store.get("key2".to_owned())?, None);
            }

            Ok(())
        }
    }

    /// Get a temporary file or directory for creating a Persistent KvStore.
    /// TempDir needs to be returned because directory is cleaned on it's drop.
    fn get_temp_dir_and_path<P: Persistent>() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");

        let path = match P::PATH_TYPE {
            PathType::File => temp_dir.as_ref().join("tests.kvs"),
            PathType::Directory => PathBuf::from(&temp_dir.as_ref()),
        };
        (temp_dir, path)
    }
}
