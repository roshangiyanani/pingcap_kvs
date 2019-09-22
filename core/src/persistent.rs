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

    use crate::tests::{TestContext, Testable};

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

    impl<P> PersistentTests for P where P: Persistent + Testable {}

    /// Functions to test Persistent KvStore implementations.
    pub trait PersistentTests: Persistent + Testable {
        /// Should contain previously saved value
        fn test_storing_values() -> Result<()> {
            let context = Self::Context::init();

            {
                let mut store: Self = context.open_store()?;
                store.set("key1".to_owned(), "value1".to_owned())?;
            }

            {
                let store: Self = context.open_store()?;
                assert_eq!(
                    store.get("key1".to_owned())?,
                    Some("value1".to_owned())
                );
            }

            Ok(())
        }

        /// Should contain overwritten values
        fn test_overwriting_values() -> Result<()> {
            let context = Self::Context::init();

            {
                let mut store: Self = context.open_store()?;
                store.set("key1".to_owned(), "value1".to_owned())?;
                store.set("key1".to_owned(), "value2".to_owned())?;
            }

            {
                let mut store: Self = context.open_store()?;
                assert_eq!(
                    store.get("key1".to_owned())?,
                    Some("value2".to_owned())
                );
                store.set("key1".to_owned(), "value3".to_owned())?;
            }

            {
                let store: Self = context.open_store()?;
                assert_eq!(
                    store.get("key1".to_owned())?,
                    Some("value3".to_owned())
                );
            }

            Ok(())
        }

        /// Should not contain key
        fn test_nonexistent_values() -> Result<()> {
            let context = Self::Context::init();

            {
                let mut store: Self = context.open_store()?;
                assert_eq!(store.get("key1".to_owned())?, None);
                store.set("key1".to_owned(), "value1".to_owned())?;
            }

            {
                let store: Self = context.open_store()?;
                assert_eq!(store.get("key2".to_owned())?, None);
            }

            Ok(())
        }

        /// Should not contain key after removal
        fn test_removals() -> Result<()> {
            let context = Self::Context::init();

            {
                let mut store: Self = context.open_store()?;
                store.set("key1".to_owned(), "value1".to_owned())?;
            }

            {
                let mut store: Self = context.open_store()?;
                store.remove("key1".to_owned())?;
            }

            {
                let store: Self = context.open_store()?;
                assert_eq!(store.get("key2".to_owned())?, None);
            }

            Ok(())
        }
    }
}
