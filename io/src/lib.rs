#![deny(missing_docs)]
#![feature(seek_convenience)]

/*!
 * Crate containing useful things for safe io.
 */

mod overwrite;
pub use overwrite::*;

mod tracker;
pub use tracker::*;
