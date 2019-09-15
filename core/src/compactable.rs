/*!
 * Traits and tests related to Compactability.
 */

use crate::{KvStore, Persistent, Result};

/// Trait for compactable key value stores
pub trait Compactable: KvStore + Persistent {
    /// Compacts the key value store
    fn compact(&mut self) -> Result<()>;
}

#[cfg(feature = "impl-tests")]
/// Contains functions, traits, and macros for easy testing of
/// a Compactable implementation.
pub mod compactable_tests {
    use super::*;

    use tempfile::TempDir;
    use walkdir::WalkDir;

    use crate::tests::Testable;

    impl<S> CompactableTests for S where S: Testable + Compactable {}

    #[macro_export]
    /// Generate tests for the given type using the CompactableTests function.
    macro_rules! generate_compactable_tests {
        ( $t: ty ) => {
            use $crate::compactable_tests::CompactableTests;

            test_function!($t, test_compaction);
        };
    }

    /// Functions to test compactability.
    pub trait CompactableTests: Testable + Compactable {
        /// Insert data until total size of the directory decreases.
        /// Test data correctness after compaction.
        fn test_compaction() -> Result<()> {
            let temp_dir = TempDir::new()
                .expect("unable to create temporary working directory");
            let mut store: Self = Testable::open(temp_dir.path())?;

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
                let store: Self = Testable::open(temp_dir.path())?;
                for key_id in 0..1000 {
                    let key = format!("key{}", key_id);
                    assert_eq!(store.get(key)?, Some(format!("{}", iter)));
                }
                return Ok(());
            }

            panic!("No compaction detected");
        }
    }
}
