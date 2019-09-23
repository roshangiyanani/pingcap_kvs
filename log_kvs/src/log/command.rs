use std::io::{Read, Write};

use serde::{Deserialize, Serialize};
use strum_macros::Display;

use core::{Error, Result};

#[derive(Debug, Display, Serialize, Deserialize)]
pub(crate) enum Command {
    /// Add a value to the key-value store.
    Set {
        /// The name to store the value under.
        key: String,
        /// The value to store.
        value: String,
    },
    /// Remove a value from the key-value store.
    Remove {
        /// The item to delete.
        key: String,
    },
}

impl Command {
    pub fn append<W: Write>(&self, writer: &mut W) -> Result<()> {
        bincode::serialize_into(writer, self).map_err(Error::bincode)
    }

    pub fn read<R: Read>(reader: &mut R) -> Result<Command> {
        bincode::deserialize_from(reader).map_err(Error::bincode)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LogCommandPointer {
    pub(in crate::log) file_id: usize,
    pub(in crate::log) offset: u64,
}

impl LogCommandPointer {
    pub fn new(file_id: usize, offset: u64) -> LogCommandPointer {
        LogCommandPointer { file_id, offset }
    }
}
