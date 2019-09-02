#![deny(missing_docs)]
#![feature(seek_convenience)]

/*!
 * An implemetation of KvStore defined in core using append-only log files.
 */

mod store;
pub use store::LogKvs;
pub(crate) use store::*;

mod command;
pub(crate) use command::*;
