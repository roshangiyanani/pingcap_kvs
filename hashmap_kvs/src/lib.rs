#![deny(missing_docs)]

/*!
 * A library exposing a key-value store that uses an in-memory hashmap.
 */

#[cfg(test)]
#[macro_use]
extern crate core;

mod hashmap_core;
mod kv_store;
mod persistence;

pub use hashmap_core::HashMapKvs;
