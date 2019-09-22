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

    use walkdir::WalkDir;

    use crate::tests::{
        PersistentTestContext, PersistentTestable, TestContext,
    };

    impl<S: Compactable> CompactableTests for S where
        S: Compactable + PersistentTestable
    {
    }

    #[macro_export]
    /// Generate tests for the given type using the CompactableTests function.
    macro_rules! generate_compactable_tests {
        ( $t: ty ) => {
            use $crate::compactable_tests::CompactableTests;

            test_functions!($t, test_compaction);
        };
    }

    /// Functions to test compactability.
    pub trait CompactableTests: Compactable + PersistentTestable {
        /// Insert data until total size of the directory decreases.
        /// Test data correctness after compaction.
        fn test_compaction() -> Result<()> {
            let context = <Self as PersistentTestable>::Context::init();
            let mut store: Self = context.open_store()?;

            let dir_size = || {
                let entries = WalkDir::new(context.get_path()).into_iter();
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
                let store: Self = context.open_store()?;
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
