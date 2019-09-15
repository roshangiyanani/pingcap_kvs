/*!
 * Utilities for testing KvStores.
 */

use std::path::Path;

use crate::{KvStore, Result};

/// Utility functions to test KvStore trait implementations
pub trait Testable: KvStore + Sized {
    /// Open the KvStore using the given directory.
    /// If given the same directory, must open the same store.
    fn open<P: AsRef<Path>>(dir: P) -> Result<Self>;
}

#[macro_export]
/// Generate a test that calls the given function on the given type
macro_rules! test_functions {
    ( $t: ty, $n: ident ) => {
        #[test]
        fn $n() -> Result<()> {
            <$t>::$n()
        }
    };
    ( $t: ty, $n: ident, $($rest:tt),*) => {
        test_functions!($t, $n);
        test_functions!($t, $($rest),*);
    }
}
