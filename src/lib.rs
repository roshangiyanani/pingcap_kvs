#![deny(missing_docs)]

/*!
A library for key-value storage.
*/

mod kvs;
pub use self::kvs::{HashMapKvs, KvStore};

mod errors;
pub use self::errors::{Error, ErrorKind, Result};
