#![deny(missing_docs)]
#![feature(seek_convenience)]

/*!
A library for key-value storage.
*/

mod kvs;
pub use self::kvs::*;

mod util;
pub use self::util::*;
