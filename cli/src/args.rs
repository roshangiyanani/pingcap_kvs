use std::path::PathBuf;

use structopt::StructOpt;
use strum_macros::{Display, EnumString};

#[derive(Debug, StructOpt)]
pub(crate) struct Opt {
    /// Which type of backing store to use.
    #[structopt(short, long, default_value = "hashmap")]
    pub(crate) store: Store,
    /// The location to load and save the backing store.
    #[structopt(
        short,
        long,
        parse(from_os_str),
        default_value = "../target/store"
    )]
    pub(crate) location: PathBuf,
    #[structopt(subcommand)]
    pub(crate) command: Command,
}

// TODO: update strum to version 0.16 when it is released and derive
// EnumVariantNames
#[derive(Debug, Display, EnumString, StructOpt)]
pub(crate) enum Store {
    /// Use a hashmap backed to the given file location.
    #[strum(serialize = "hashmap")]
    HashMap,
    /// Use an append-only log store backed in the given directory location.
    #[strum(serialize = "log")]
    Log,
}

#[derive(Debug, Display, StructOpt)]
pub(crate) enum Command {
    #[structopt(name = "get")]
    /// Retrieve a value from the key-value store.
    Get {
        /// The item to retreive the value of.
        key: String,
    },
    #[structopt(name = "set")]
    /// Add a value to the key-value store.
    Set {
        /// The name to store the value under.
        key: String,
        /// The value to store.
        value: String,
    },
    #[structopt(name = "rm")]
    /// Remove a value from the key-value store.
    Remove {
        /// The item to delete.
        key: String,
    },
    #[structopt(name = "compact")]
    /// Compact the key-value store's storage.
    Compact,
}
