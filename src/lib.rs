#![deny(missing_docs)]
#![feature(seek_convenience)]

/*!
A library for key-value storage.
*/

mod kvs;
pub use self::kvs::{HashMapKvs, KvStore, LogKvs};

mod errors;
pub use self::errors::{Error, ErrorKind, Result};
