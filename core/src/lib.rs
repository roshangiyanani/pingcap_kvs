#![deny(missing_docs)]

/*!
A library containing the basic traits and common code for key-value storage.
*/

mod kv_store;
pub use self::kv_store::*;

mod errors;
pub use self::errors::*;
