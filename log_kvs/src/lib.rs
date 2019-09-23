#![deny(missing_docs)]
#![feature(seek_convenience)]

/*!
 * An implemetation of KvStore defined in core using append-only log files.
 */

#[cfg(test)]
#[macro_use]
extern crate core;

mod log;
pub(crate) use log::*;

mod compactable;
mod kv_store;
mod persistent;

mod log_core;
pub use log_core::LogKvs;
