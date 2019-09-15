#![deny(missing_docs)]

/*!
A library containing the basic traits and common code for key-value storage.
*/

// TODO: Investigate why feature flags don't seem to be enforced
// must be first because it contains a macro used in kv_store and compactable
#[cfg(feature = "impl-tests")]
pub mod tests;

mod kv_store;
pub use self::kv_store::*;

mod persistent;
pub use self::persistent::*;

mod compactable;
pub use self::compactable::*;

mod errors;
pub use self::errors::*;
