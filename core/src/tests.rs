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
