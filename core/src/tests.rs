/*!
 * Utilities for testing KvStores.
 */

use std::path::PathBuf;

use tempfile::TempDir;

use crate::{KvStore, PathType, Persistent, Result};

/// Mark a KvStore as testable
pub trait Testable: KvStore + Sized {
    /// The context used for testing
    type Context: TestContext<Self>;
}

/// Mark a Persistent KvStore as testable
pub trait PersistentTestable: Testable + Persistent {
    /// The context used for testing
    type Context: PersistentTestContext<Self>;
}

/// Needed functions for tests
pub trait TestContext<S: KvStore>: Sized {
    /// Initialize a new TestContext.
    fn init() -> Self;

    /// Get a new KvStore using this context.
    fn open_store(&self) -> Result<S>;
}

/// Needed functions for persistent tests
pub trait PersistentTestContext<S: KvStore + Persistent>:
    TestContext<S>
{
    /// Get the path to the persistant store in this context
    fn get_path(&self) -> &PathBuf;
}

/// Store the context for testing.
pub struct DefaultTestContext {
    /// needs to be stored b/c the directory is cleaned on drop
    _temp_dir: TempDir,
    pub(crate) path: PathBuf,
}

impl<S: Persistent> TestContext<S> for DefaultTestContext {
    /// Initialize a new TestContext.
    fn init() -> Self {
        let temp_dir = TempDir::new()
            .expect("unable to create temporary working directory");

        let path = match S::PATH_TYPE {
            PathType::File => temp_dir.as_ref().join("tests.kvs"),
            PathType::Directory => PathBuf::from(&temp_dir.as_ref()),
        };

        DefaultTestContext {
            _temp_dir: temp_dir,
            path,
        }
    }

    /// Use the TestContext to get a new KvStore in the same context.
    fn open_store(&self) -> Result<S> {
        Persistent::open(&self.path)
    }
}

impl<S: Persistent> PersistentTestContext<S> for DefaultTestContext {
    /// Get the path for the context backing store
    fn get_path(&self) -> &PathBuf {
        &self.path
    }
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
