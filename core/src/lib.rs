#![deny(missing_docs)]

/*!
A library containing the basic traits and common code for key-value storage.
*/

mod compactable;
#[cfg(feature = "impl-tests")]
pub use self::compactable::tests::*;
pub use self::compactable::*;

mod kv_store;
#[cfg(feature = "impl-tests")]
pub use self::kv_store::tests::*;
pub use self::kv_store::*;

mod errors;
pub use self::errors::*;

// TODO: Investigate why feature flags don't seem to be enforced
#[cfg(feature = "impl-tests")]
mod tests;
pub use self::tests::*;
